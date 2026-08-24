use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::bson::{Document, RawDocument, RawDocumentBuf};

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct BsonViewT {
    pub ptr: *const u8,
    pub len: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct BsonT {
    pub ptr: *const u8,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_bson_destroy(bson: BsonT) {
    if bson.ptr.is_null() {
        return;
    }

    // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
    // SAFETY: `bson.ptr` is always allocated as a `Box<[u8]>`.
    safe_drop!(std::ptr::slice_from_raw_parts_mut(
        bson.ptr.cast_mut(),
        bson.len,
    ));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_bson_view_destroy(_bson: BsonViewT) {
    // No-op: this function is only to force cbindgen to declare `BsonViewT` in the crate header.
}

pub(crate) const EMPTY_DOC: [u8; 5] = [5, 0, 0, 0, 0];

impl BsonViewT {
    #[must_use]
    pub const fn empty_doc() -> Self {
        BsonViewT {
            ptr: EMPTY_DOC.as_ptr(),
            len: EMPTY_DOC.len(),
        }
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        if self.ptr.is_null() {
            &[]
        } else {
            // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
            unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
        }
    }
}

impl BsonT {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        if self.ptr.is_null() {
            &[]
        } else {
            // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
            unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
        }
    }
}

impl<'a> TryFrom<&'a BsonViewT> for &'a RawDocument {
    type Error = ErrorT;

    fn try_from(view: &'a BsonViewT) -> Result<Self, Self::Error> {
        RawDocument::from_bytes(view.as_bytes()).map_err(Into::into)
    }
}

impl TryFrom<&BsonViewT> for RawDocumentBuf {
    type Error = ErrorT;

    fn try_from(view: &BsonViewT) -> Result<Self, Self::Error> {
        RawDocumentBuf::from_bytes(view.as_bytes().to_vec()).map_err(Into::into)
    }
}

impl TryFrom<&BsonViewT> for Document {
    type Error = ErrorT;

    fn try_from(view: &BsonViewT) -> Result<Self, Self::Error> {
        Document::from_reader(view.as_bytes()).map_err(Into::into)
    }
}

impl From<&RawDocument> for BsonViewT {
    fn from(doc: &RawDocument) -> Self {
        let bytes = doc.as_bytes();

        BsonViewT {
            ptr: bytes.as_ptr().cast::<u8>(),
            len: bytes.len(),
        }
    }
}

impl From<&RawDocumentBuf> for BsonViewT {
    fn from(doc: &RawDocumentBuf) -> Self {
        let bytes = doc.as_bytes();

        BsonViewT {
            ptr: bytes.as_ptr().cast::<u8>(),
            len: bytes.len(),
        }
    }
}

impl From<RawDocumentBuf> for BsonT {
    fn from(doc: RawDocumentBuf) -> Self {
        let bytes = doc.into_bytes();
        let len = bytes.len();

        BsonT {
            ptr: Box::into_raw(bytes.into_boxed_slice()).cast::<u8>(),
            len,
        }
    }
}

impl From<&RawDocumentBuf> for BsonT {
    fn from(doc: &RawDocumentBuf) -> Self {
        let bytes = doc.as_bytes().to_vec(); // Deep-copy!
        let len = bytes.len();

        BsonT {
            ptr: Box::into_raw(bytes.into_boxed_slice()).cast::<u8>(),
            len,
        }
    }
}
