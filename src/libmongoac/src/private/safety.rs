use crate::error::{ErrorCodeT, ErrorT};

pub(crate) fn invalid_argument(error: Option<&mut ErrorT>, msg: &str) {
    if let Some(e) = error {
        *e = ErrorT::from_mongoac(ErrorCodeT::InvalidArgument, msg);
    }
}

#[macro_export]
macro_rules! safe_drop {
    ($ptr:expr) => {{
        let ptr = $ptr;
        if !ptr.is_null() {
            unsafe { drop(Box::from_raw(ptr)) }
        }
    }};
}

#[macro_export]
macro_rules! safe_as_mut {
    ($ptr:expr) => {{
        let ptr = $ptr;
        match unsafe { ptr.as_mut() } {
            Some(r) => r,
            None => return Default::default(),
        }
    }};
}

#[macro_export]
macro_rules! safe_optional_as_mut {
    ($ptr:expr) => {{
        let ptr = $ptr;
        unsafe { ptr.as_mut() }
    }};
}

#[macro_export]
macro_rules! safe_optional_error_as_mut {
    ($ptr:expr) => {{
        let ptr = $ptr;
        match unsafe { ptr.as_mut() } {
            Some(e) => {
                e.clear();
                Some(e)
            }
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! safe_as_mut_with_error {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        match unsafe { ptr.as_mut() } {
            Some(r) => r,
            None => {
                $crate::private::safety::invalid_argument(
                    $error,
                    concat!(stringify!($ptr), ": must not be null"),
                );
                return Default::default();
            }
        }
    }};
}

#[macro_export]
macro_rules! safe_optional_as_ref {
    ($ptr:expr) => {{
        let ptr = $ptr;
        unsafe { ptr.as_ref() }
    }};
}

#[macro_export]
macro_rules! safe_as_ref {
    ($ptr:expr) => {{
        let ptr = $ptr;
        match unsafe { ptr.as_ref() } {
            Some(r) => r,
            None => return Default::default(),
        }
    }};
}

#[macro_export]
macro_rules! safe_as_ref_with_error {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        match unsafe { ptr.as_ref() } {
            Some(r) => r,
            None => {
                $crate::private::safety::invalid_argument(
                    $error,
                    concat!(stringify!($ptr), ": must not be null"),
                );
                return Default::default();
            }
        }
    }};
}

#[macro_export]
macro_rules! safe_cstr_from_ptr {
    ($ptr:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            return Default::default();
        }

        match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
            Ok(s) => s,
            Err(_) => return Default::default(),
        }
    }};
}

#[macro_export]
macro_rules! safe_cstr_from_ptr_with_error {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            $crate::private::safety::invalid_argument(
                $error,
                concat!(stringify!($ptr), ": must not be null"),
            );
            return Default::default();
        }

        match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
            Ok(s) => s,
            Err(_) => {
                $crate::private::safety::invalid_argument(
                    $error,
                    concat!(stringify!($ptr), ": must be valid UTF-8"),
                );
                return Default::default();
            }
        }
    }};
}

#[macro_export]
macro_rules! safe_optional_cstr_from_ptr {
    ($ptr:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            None
        } else {
            match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
                Ok(s) => Some(s),
                Err(_) => None,
            }
        }
    }};
}

#[macro_export]
macro_rules! safe_optional_cstr_from_ptr_with_error {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            None
        } else {
            match unsafe { std::ffi::CStr::from_ptr(ptr) }.to_str() {
                Ok(s) => Some(s),
                Err(_) => {
                    $crate::private::safety::invalid_argument(
                        $error,
                        concat!(stringify!($ptr), ": must be valid UTF-8"),
                    );
                    return Default::default();
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! safe_optional_bson_opts_with_error {
    ($target:ty, $options:expr, $error:expr) => {{
        let options = safe_optional_bson_view!($options);
        match options {
            Some(ref o) => Some($crate::safe_error!(
                ::mongodb::bson::deserialize_from_slice::<$target>(o.as_bytes()),
                $error
            )),
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! safe_bson_view_with_error {
    ($ptr:expr, $error:expr) => {{
        let bson: $crate::bson::BsonViewT = $ptr;
        if bson.data.is_null() {
            $crate::private::safety::invalid_argument(
                $error,
                concat!(stringify!($ptr), ": must not be null"),
            );
            return Default::default();
        }
        bson
    }};
}

#[macro_export]
macro_rules! safe_optional_bson_view {
    ($ptr:expr) => {{
        let bson: $crate::bson::BsonViewT = $ptr;
        if bson.data.is_null() {
            None
        } else {
            Some(bson)
        }
    }};
}

#[macro_export]
macro_rules! safe_bson_view_array_as_vec_with_error {
    ($ptr:expr, $len:expr, $error:expr) => {{
        let ptr: *const $crate::bson::BsonViewT = $ptr;
        let len: usize = $len;

        if ptr.is_null() || len == 0 {
            return Default::default();
        }

        // SAFETY: `ptr` and `len` validity is an uncheckable precondition.
        let arr = unsafe { std::slice::from_raw_parts(ptr, len) };

        let mut vec: Vec<$crate::bson::BsonViewT> = Vec::with_capacity(len);
        for (i, e) in arr.iter().enumerate() {
            if e.data.is_null() {
                $crate::private::safety::invalid_argument(
                    $error,
                    &format!("BSON array element at index {i}: must not be null"),
                );
                return Default::default();
            }
            vec.push(*e);
        }
        vec
    }};
}

#[macro_export]
macro_rules! safe_error {
    ($expr:expr, $error:expr) => {
        match $expr {
            Ok(val) => val,
            Err(err) => {
                if let Some(e) = $error {
                    *e = ::std::convert::Into::into(err);
                }
                return Default::default();
            }
        }
    };
}
