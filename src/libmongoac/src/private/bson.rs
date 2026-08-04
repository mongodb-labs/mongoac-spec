use mongodb::bson::{Document, RawDocument, RawDocumentBuf};

use crate::error::ErrorT;

#[repr(C)]
pub struct bson_t {
    _private: [u8; 0],
}

unsafe extern "C" {
    pub fn bson_destroy(bson: *mut bson_t);
    pub fn bson_get_data(bson: *const bson_t) -> *const u8;
    pub fn bson_new_from_data(data: *const u8, length: usize) -> *mut bson_t;
}

impl bson_t {
    // Library-wide precondition: BSON data is always valid.
    fn as_bytes(&self) -> &[u8] {
        let data = unsafe { bson_get_data(self as *const bson_t) };
        let raw_len = unsafe { std::slice::from_raw_parts(data, 4) };
        let len = u32::from_le_bytes([raw_len[0], raw_len[1], raw_len[2], raw_len[3]]) as usize;
        unsafe { std::slice::from_raw_parts(data, len) }
    }
}

pub struct BsonT(*mut bson_t);
pub struct ConstBsonT(*const bson_t);

impl Drop for BsonT {
    fn drop(&mut self) {
        unsafe { bson_destroy(self.0) }
    }
}

impl BsonT {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ErrorT> {
        let ptr = unsafe { bson_new_from_data(bytes.as_ptr(), bytes.len()) };

        if ptr.is_null() {
            Err(ErrorT::from_mongoac(
                crate::error::ErrorCodeT::RuntimeError,
                "bson_new_from_data failed",
            ))
        } else {
            Ok(Self(ptr))
        }
    }

    // Library-wide precondition: BsonT is never null.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { self.0.as_ref() }.unwrap().as_bytes()
    }
}

impl ConstBsonT {
    // Library-wide precondition: ConstBsonT is never null.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { self.0.as_ref() }.unwrap().as_bytes()
    }
}

impl BsonT {
    pub fn into_raw(self) -> *mut bson_t {
        let ptr = self.0;
        std::mem::forget(self); // Release ownership.
        ptr
    }
}

impl From<&bson_t> for ConstBsonT {
    fn from(bson: &bson_t) -> Self {
        Self(bson)
    }
}

impl From<&BsonT> for ConstBsonT {
    fn from(bson: &BsonT) -> Self {
        Self(bson.0)
    }
}

impl From<ConstBsonT> for *const bson_t {
    fn from(bson: ConstBsonT) -> Self {
        bson.0
    }
}

impl TryFrom<&BsonT> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &BsonT) -> Result<Self, Self::Error> {
        Document::from_reader(bson.as_bytes()) // Deep-copy!
    }
}

impl TryFrom<&ConstBsonT> for Document {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &ConstBsonT) -> Result<Self, Self::Error> {
        Document::from_reader(bson.as_bytes()) // Deep-copy!
    }
}

impl TryFrom<&BsonT> for RawDocumentBuf {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &BsonT) -> Result<Self, Self::Error> {
        RawDocumentBuf::from_reader(bson.as_bytes()) // Deep-copy!
    }
}

impl<'a> TryFrom<&'a BsonT> for &'a RawDocument {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &'a BsonT) -> Result<Self, Self::Error> {
        RawDocument::from_bytes(bson.as_bytes())
    }
}

impl<'a> TryFrom<&'a ConstBsonT> for &'a RawDocument {
    type Error = mongodb::bson::error::Error;

    fn try_from(bson: &'a ConstBsonT) -> Result<Self, Self::Error> {
        RawDocument::from_bytes(bson.as_bytes())
    }
}

impl TryFrom<&RawDocumentBuf> for BsonT {
    type Error = ErrorT;

    fn try_from(doc: &RawDocumentBuf) -> Result<Self, Self::Error> {
        Self::from_bytes(doc.as_bytes()) // Deep-copy!
    }
}

impl TryFrom<&RawDocument> for BsonT {
    type Error = ErrorT;

    fn try_from(doc: &RawDocument) -> Result<Self, Self::Error> {
        Self::from_bytes(doc.as_bytes()) // Deep-copy!
    }
}

impl TryFrom<&RawDocumentBuf> for ConstBsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(doc: &RawDocumentBuf) -> Result<Self, Self::Error> {
        Ok(Self(doc.as_bytes().as_ptr() as *const bson_t))
    }
}

impl TryFrom<&RawDocument> for ConstBsonT {
    type Error = mongodb::bson::error::Error;

    fn try_from(doc: &RawDocument) -> Result<Self, Self::Error> {
        Ok(Self(doc.as_bytes().as_ptr() as *const bson_t))
    }
}
