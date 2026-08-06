use crate::error::{ErrorCodeT, ErrorT};
use crate::private::macros::*;

use std::ffi::c_char;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct StringViewT {
    pub data: *const c_char,
    pub len: usize,
}

#[repr(C)]
#[derive(Default)]
pub struct StringT {
    pub data: *const c_char,
    pub len: usize,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_string_destroy(string: StringT) {
    if string.data.is_null() {
        return;
    }

    // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
    // SAFETY: `string.data` is always allocated as a `Box<[u8]>`.
    safe_drop!(std::ptr::slice_from_raw_parts_mut(
        string.data as *mut u8,
        string.len,
    ));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_string_view_destroy(_string: StringViewT) {
    // No-op: this function is only to force cbindgen to declare `StringViewT` in the crate header.
}

impl StringViewT {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
            unsafe { std::slice::from_raw_parts(self.data.cast::<u8>(), self.len) }
        }
    }
}

impl StringT {
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            // SAFETY: bytes `[0, len)` at `data` MUST be accessible when `data` is not null.
            unsafe { std::slice::from_raw_parts(self.data.cast::<u8>(), self.len) }
        }
    }
}

impl<'a> TryFrom<&'a StringViewT> for &'a str {
    type Error = ErrorT;

    fn try_from(view: &'a StringViewT) -> Result<Self, Self::Error> {
        match std::str::from_utf8(view.as_bytes()) {
            Ok(s) => Ok(s),
            Err(e) => Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &e.to_string(),
            )),
        }
    }
}

impl TryFrom<&StringViewT> for String {
    type Error = ErrorT;

    fn try_from(view: &StringViewT) -> Result<Self, Self::Error> {
        match std::str::from_utf8(view.as_bytes()) {
            Ok(s) => Ok(s.to_string()),
            Err(e) => Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &e.to_string(),
            )),
        }
    }
}

impl From<&str> for StringViewT {
    fn from(s: &str) -> Self {
        StringViewT {
            data: s.as_ptr().cast::<c_char>(),
            len: s.len(),
        }
    }
}

impl From<&String> for StringViewT {
    fn from(s: &String) -> Self {
        StringViewT {
            data: s.as_ptr().cast::<c_char>(),
            len: s.len(),
        }
    }
}

impl<'a> TryFrom<&'a StringT> for &'a str {
    type Error = ErrorT;

    fn try_from(string: &'a StringT) -> Result<Self, Self::Error> {
        match std::str::from_utf8(string.as_bytes()) {
            Ok(s) => Ok(s),
            Err(e) => Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &e.to_string(),
            )),
        }
    }
}

impl From<String> for StringT {
    fn from(s: String) -> Self {
        let bytes = s.into_bytes();
        let len = bytes.len();

        StringT {
            data: Box::into_raw(bytes.into_boxed_slice()).cast::<c_char>(),
            len,
        }
    }
}

impl From<&String> for StringT {
    fn from(s: &String) -> Self {
        let bytes = s.as_bytes().to_vec();
        let len = bytes.len();

        StringT {
            data: Box::into_raw(bytes.into_boxed_slice()).cast::<c_char>(),
            len,
        }
    }
}

impl From<&str> for StringT {
    fn from(s: &str) -> Self {
        let bytes = s.as_bytes().to_vec();
        let len = bytes.len();

        StringT {
            data: Box::into_raw(bytes.into_boxed_slice()).cast::<c_char>(),
            len,
        }
    }
}
