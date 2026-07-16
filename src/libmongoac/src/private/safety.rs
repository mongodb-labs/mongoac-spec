use crate::error::{ErrorCodeT, ErrorT};

pub(crate) fn invalid_argument(error: Option<&mut ErrorT>, msg: &str) {
    if let Some(e) = error {
        *e = ErrorT::from_mongoac(ErrorCodeT::InvalidArgument, msg);
    }
}

#[macro_export]
macro_rules! safe_drop {
    ($ptr:expr) => {
        if !$ptr.is_null() {
            unsafe { drop(Box::from_raw($ptr)) }
        }
    };
}

#[macro_export]
macro_rules! safe_as_mut {
    ($ptr:expr) => {
        match unsafe { $ptr.as_mut() } {
            Some(r) => r,
            None => return Default::default(),
        }
    };
}

#[macro_export]
macro_rules! safe_optional_as_mut {
    ($ptr:expr) => {
        unsafe { $ptr.as_mut() }
    };
}

#[macro_export]
macro_rules! safe_optional_error_as_mut {
    ($ptr:expr) => {
        match unsafe { $ptr.as_mut() } {
            Some(e) => {
                e.clear();
                Some(e)
            }
            None => None,
        }
    };
}

#[macro_export]
macro_rules! safe_as_mut_with_error {
    ($ptr:expr, $error:expr) => {
        match unsafe { $ptr.as_mut() } {
            Some(r) => r,
            None => {
                $crate::private::safety::invalid_argument(
                    $error,
                    concat!(stringify!($ptr), ": must not be null"),
                );
                return Default::default();
            }
        }
    };
}

#[macro_export]
macro_rules! safe_optional_as_ref {
    ($ptr:expr) => {
        unsafe { $ptr.as_ref() }
    };
}

#[macro_export]
macro_rules! safe_as_ref {
    ($ptr:expr) => {
        match unsafe { $ptr.as_ref() } {
            Some(r) => r,
            None => return Default::default(),
        }
    };
}

#[macro_export]
macro_rules! safe_as_ref_with_error {
    ($ptr:expr, $error:expr) => {
        match unsafe { $ptr.as_ref() } {
            Some(r) => r,
            None => {
                $crate::private::safety::invalid_argument(
                    $error,
                    concat!(stringify!($ptr), ": must not be null"),
                );
                return Default::default();
            }
        }
    };
}

#[macro_export]
macro_rules! safe_cstr_from_ptr_with_error {
    ($ptr:expr, $error:expr) => {{
        if $ptr.is_null() {
            $crate::private::safety::invalid_argument(
                $error,
                concat!(stringify!($ptr), ": must not be null"),
            );
            return Default::default();
        }

        match unsafe { std::ffi::CStr::from_ptr($ptr) }.to_str() {
            Ok(s) => s.to_string(),
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
macro_rules! safe_optional_cstr_from_ptr_with_error {
    ($ptr:expr, $error:expr) => {{
        if $ptr.is_null() {
            None
        } else {
            match unsafe { std::ffi::CStr::from_ptr($ptr) }.to_str() {
                Ok(s) => Some(s.to_string()),
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
macro_rules! safe_bson {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            $crate::private::safety::invalid_argument(
                $error,
                concat!(stringify!($ptr), ": must not be null"),
            );
            return Default::default();
        }
        $crate::private::bson::BsonT::from(ptr)
    }};
}

#[macro_export]
macro_rules! safe_optional_bson {
    ($ptr:expr) => {{
        match unsafe { $ptr.as_mut() } {
            Some(r) => Some($crate::private::bson::BsonT::from(
                r as *mut $crate::private::bson::bson_t,
            )),
            None => None,
        }
    }};
}

#[macro_export]
macro_rules! safe_const_bson {
    ($ptr:expr, $error:expr) => {{
        let ptr = $ptr;
        if ptr.is_null() {
            $crate::private::safety::invalid_argument(
                $error,
                concat!(stringify!($ptr), ": must not be null"),
            );
            return Default::default();
        }
        $crate::private::bson::ConstBsonT::from(ptr)
    }};
}

#[macro_export]
macro_rules! safe_optional_const_bson {
    ($ptr:expr) => {{
        match unsafe { $ptr.as_ref() } {
            Some(r) => Some($crate::private::bson::ConstBsonT::from(r)),
            None => None,
        }
    }};
}
