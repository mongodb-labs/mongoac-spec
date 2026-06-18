use mongodb::bson::{Document, RawDocument, RawDocumentBuf};

#[repr(C)]
pub struct bson_t {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn bson_destroy(bson: *mut bson_t);
    pub fn bson_get_data(bson: *const bson_t) -> *const u8;
    pub fn bson_init_static(bson: *mut bson_t, data: *const u8, length: usize) -> bool;
    pub fn bson_new_from_data(data: *const u8, length: usize) -> *mut bson_t;
}

unsafe fn bson_t_to_slice(bson: &bson_t) -> &[u8] {
    let data = unsafe { bson_get_data(bson as *const bson_t) };
    let raw_len = unsafe { std::slice::from_raw_parts(data, 4) };
    let len = u32::from_le_bytes([raw_len[0], raw_len[1], raw_len[2], raw_len[3]]) as usize;
    unsafe { std::slice::from_raw_parts(data, len) }
}
pub struct BsonT {
    ptr: *mut bson_t,
}

impl BsonT {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, mongodb::bson::error::Error> {
        let ptr = unsafe { bson_new_from_data(bytes.as_ptr(), bytes.len()) };
        if ptr.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "bson_new_from_data: allocation failure",
            )
            .into());
        }
        Ok(Self { ptr })
    }
    pub fn release(mut self) -> *mut bson_t {
        let ptr = self.ptr;
        self.ptr = std::ptr::null_mut();
        ptr
    }
    pub fn as_ptr(&self) -> *const bson_t {
        self.ptr as *const bson_t
    }
}

impl Drop for BsonT {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe { bson_destroy(self.ptr) };
        }
    }
}

impl TryFrom<&RawDocumentBuf> for BsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(raw: &RawDocumentBuf) -> Result<Self, Self::Error> {
        let bytes = raw.as_bytes();
        Self::from_bytes(bytes)
    }
}

impl TryFrom<&Document> for BsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(doc: &Document) -> Result<Self, Self::Error> {
        let bytes = doc.to_vec()?;
        Self::from_bytes(&bytes)
    }
}

impl From<*mut bson_t> for BsonT {
    fn from(ptr: *mut bson_t) -> Self {
        Self { ptr }
    }
}

pub struct ConstBsonT {
    ptr: *const bson_t,
}

impl ConstBsonT {
    pub fn to_document(&self) -> Result<Document, mongodb::bson::error::Error> {
        let slice = unsafe { bson_t_to_slice(&*self.ptr) };
        Document::from_reader(&mut &slice[..])
    }
}
unsafe impl Send for ConstBsonT {}
unsafe impl Sync for ConstBsonT {}

impl<'a> TryInto<&'a RawDocument> for &'a ConstBsonT {
    type Error = mongodb::bson::error::Error;

    fn try_into(self) -> Result<&'a RawDocument, Self::Error> {
        let slice = unsafe { bson_t_to_slice(&*self.ptr) };
        RawDocument::from_bytes(slice)
    }
}

impl TryFrom<&bson_t> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &bson_t) -> Result<Self, Self::Error> {
        let slice = unsafe { bson_t_to_slice(bson) };
        Self::from_reader(&mut &slice[..])
    }
}

impl bson_t {
    fn from_bytes(bytes: &[u8]) -> Result<*mut bson_t, mongodb::bson::error::Error> {
        let ptr = unsafe { bson_new_from_data(bytes.as_ptr(), bytes.len()) };
        if ptr.is_null() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "bson_new_from_data: allocation failure",
            )
            .into());
        }
        Ok(ptr)
    }

    pub(crate) fn from_raw_document_buf(
        raw: &RawDocumentBuf,
    ) -> Result<*mut bson_t, mongodb::bson::error::Error> {
        Self::from_bytes(raw.as_bytes())
    }

    pub(crate) fn from_document(
        doc: &Document,
    ) -> Result<*mut bson_t, mongodb::bson::error::Error> {
        Self::from_bytes(&doc.to_vec()?)
    }

    pub(crate) fn to_raw_document_buf(
        &self,
    ) -> Result<RawDocumentBuf, mongodb::bson::error::Error> {
        RawDocumentBuf::try_from(self)
    }

    pub(crate) fn to_document(&self) -> Result<Document, mongodb::bson::error::Error> {
        Document::try_from(self)
    }
}

impl TryFrom<&bson_t> for RawDocumentBuf {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &bson_t) -> Result<Self, Self::Error> {
        let slice = unsafe { bson_t_to_slice(bson) };
        Self::from_bytes(slice.to_vec()).map_err(|e| {
            mongodb::bson::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })
    }
}
