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

impl AsRef<[u8]> for bson_t {
    fn as_ref(&self) -> &[u8] {
        let data = unsafe { bson_get_data(self as *const bson_t) };
        let raw_len = unsafe { std::slice::from_raw_parts(data, 4) };
        let len = u32::from_le_bytes([raw_len[0], raw_len[1], raw_len[2], raw_len[3]]) as usize;
        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

impl TryFrom<&bson_t> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &bson_t) -> Result<Self, Self::Error> {
        let slice: &[u8] = bson.as_ref();
        Self::from_reader(&mut &slice[..])
    }
}

impl TryFrom<&bson_t> for RawDocumentBuf {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &bson_t) -> Result<Self, Self::Error> {
        let slice: &[u8] = bson.as_ref();
        Self::from_bytes(slice.to_vec()).map_err(|e| {
            mongodb::bson::error::Error::from(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                e.to_string(),
            ))
        })
    }
}

impl<'a> TryFrom<&'a bson_t> for &'a RawDocument {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &'a bson_t) -> Result<Self, Self::Error> {
        let slice: &[u8] = bson.as_ref();
        RawDocument::from_bytes(slice)
    }
}

pub struct BsonT(*mut bson_t);

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
        Ok(Self(ptr))
    }

    pub fn as_ptr(&self) -> *const bson_t {
        self.0 as *const bson_t
    }

    pub fn as_mut_ptr(&mut self) -> *mut bson_t {
        self.0
    }
}

impl Drop for BsonT {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe { bson_destroy(self.0) };
        }
    }
}

impl AsRef<[u8]> for BsonT {
    fn as_ref(&self) -> &[u8] {
        unsafe { (*self.0).as_ref() }
    }
}

impl AsRef<bson_t> for BsonT {
    fn as_ref(&self) -> &bson_t {
        unsafe { &*self.0 }
    }
}

impl From<*mut bson_t> for BsonT {
    fn from(ptr: *mut bson_t) -> Self {
        Self(ptr)
    }
}

impl From<BsonT> for *mut bson_t {
    fn from(wrapper: BsonT) -> Self {
        let ptr = wrapper.0;
        std::mem::forget(wrapper);
        ptr
    }
}

impl TryFrom<BsonT> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: BsonT) -> Result<Self, Self::Error> {
        Document::try_from(unsafe { &*bson.0 })
    }
}

impl TryFrom<BsonT> for RawDocumentBuf {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: BsonT) -> Result<Self, Self::Error> {
        RawDocumentBuf::try_from(unsafe { &*bson.0 })
    }
}

impl TryFrom<&RawDocument> for BsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(raw: &RawDocument) -> Result<Self, Self::Error> {
        Self::from_bytes(raw.as_bytes())
    }
}

impl TryFrom<&RawDocumentBuf> for BsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(raw: &RawDocumentBuf) -> Result<Self, Self::Error> {
        Self::from_bytes(raw.as_bytes())
    }
}

impl TryFrom<&Document> for BsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(doc: &Document) -> Result<Self, Self::Error> {
        let bytes = doc.to_vec()?;
        Self::from_bytes(&bytes)
    }
}

#[derive(Clone, Copy)]
pub struct ConstBsonT(*const bson_t);

impl ConstBsonT {
    pub fn as_ptr(&self) -> *const bson_t {
        self.0
    }
}

impl AsRef<[u8]> for ConstBsonT {
    fn as_ref(&self) -> &[u8] {
        unsafe { (*self.0).as_ref() }
    }
}

impl AsRef<bson_t> for ConstBsonT {
    fn as_ref(&self) -> &bson_t {
        unsafe { &*self.0 }
    }
}

impl From<*const bson_t> for ConstBsonT {
    fn from(ptr: *const bson_t) -> Self {
        Self(ptr)
    }
}

impl<'a> From<&'a bson_t> for ConstBsonT {
    fn from(bson: &'a bson_t) -> Self {
        Self(bson as *const bson_t)
    }
}

impl From<&BsonT> for ConstBsonT {
    fn from(bson: &BsonT) -> Self {
        Self(bson.as_ptr())
    }
}

impl TryFrom<&ConstBsonT> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &ConstBsonT) -> Result<Self, Self::Error> {
        Document::try_from(unsafe { &*bson.0 })
    }
}

impl TryFrom<&ConstBsonT> for RawDocumentBuf {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &ConstBsonT) -> Result<Self, Self::Error> {
        RawDocumentBuf::try_from(unsafe { &*bson.0 })
    }
}

impl<'a> TryFrom<&'a ConstBsonT> for &'a RawDocument {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &'a ConstBsonT) -> Result<Self, Self::Error> {
        RawDocument::from_bytes(bson)
    }
}
