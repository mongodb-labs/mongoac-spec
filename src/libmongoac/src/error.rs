use crate::private::macros::*;
use std::ffi::CString;
use strum::EnumMessage;

#[allow(non_camel_case_types)]
pub type mongoac_error_category_t = i32;
pub const MONGOAC_ERROR_CATEGORY_NONE: mongoac_error_category_t = 0;
pub const MONGOAC_ERROR_CATEGORY_MONGOAC: mongoac_error_category_t = 1;
pub const MONGOAC_ERROR_CATEGORY_BSON: mongoac_error_category_t = 2;
pub const MONGOAC_ERROR_CATEGORY_RUST: mongoac_error_category_t = 3;

#[allow(non_camel_case_types)]
pub type mongoac_error_code_t = i32;
pub const MONGOAC_ERROR_CODE_OK: mongoac_error_code_t = 0;
pub const MONGOAC_ERROR_CODE_UNKNOWN_CATEGORY: mongoac_error_code_t = 1;
pub const MONGOAC_ERROR_CODE_INVALID_ARGUMENT: mongoac_error_code_t = 2;
pub const MONGOAC_ERROR_CODE_RUNTIME_ERROR: mongoac_error_code_t = 3;
pub const MONGOAC_ERROR_CODE_TIMEOUT: mongoac_error_code_t = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum ErrorCategoryT {
    None = MONGOAC_ERROR_CATEGORY_NONE,
    MongoAC = MONGOAC_ERROR_CATEGORY_MONGOAC,
    Bson = MONGOAC_ERROR_CATEGORY_BSON,
    Rust = MONGOAC_ERROR_CATEGORY_RUST,

    #[num_enum(catch_all)]
    Unknown(i32) = i32::MIN,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    num_enum::FromPrimitive,
    num_enum::IntoPrimitive,
    strum::EnumMessage,
)]
#[repr(i32)]
pub enum ErrorCodeT {
    #[strum(message = "ok")]
    Ok = MONGOAC_ERROR_CODE_OK,
    #[strum(message = "invalid argument")]
    InvalidArgument = MONGOAC_ERROR_CODE_INVALID_ARGUMENT,
    #[strum(message = "runtime error")]
    RuntimeError = MONGOAC_ERROR_CODE_RUNTIME_ERROR,
    #[strum(message = "timeout")]
    Timeout = MONGOAC_ERROR_CODE_TIMEOUT,

    #[strum(message = "unknown error code")]
    #[num_enum(catch_all)]
    Unknown(i32),
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_new() -> *mut ErrorT {
    Box::into_raw(Box::new(ErrorT::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_destroy(error: *mut ErrorT) {
    safe_drop!(error);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_category(error: *const ErrorT) -> i32 {
    safe_as_ref!(error).category().into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_code(error: *const ErrorT) -> i32 {
    safe_as_ref!(error).code().into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_message(error: *const ErrorT) -> *const std::ffi::c_char {
    safe_as_ref!(error)
        .message()
        .map_or(EMPTY_MSG.as_ptr() as *const std::ffi::c_char, |m| {
            m.as_ptr()
        })
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_has_label(
    error: *const ErrorT,
    label: *const std::ffi::c_char,
) -> bool {
    safe_as_ref!(error).has_label(&safe_cstr_from_ptr!(label))
}

impl ErrorCodeT {
    pub fn message(self) -> &'static str {
        self.get_message().unwrap_or("unknown mongoac error")
    }
}

#[derive(Clone, Debug, Default)]
pub enum ErrorT {
    #[default]
    None,
    MongoAC {
        code: ErrorCodeT,
        message: Option<CString>,
    },
    Bson(mongodb::bson::error::Error, Option<CString>),
    Rust(mongodb::error::Error, Option<CString>),
}

impl ErrorT {
    pub fn new() -> Self {
        Self::None
    }

    pub(crate) fn from_mongoac(code: ErrorCodeT, msg: &str) -> Self {
        Self::MongoAC {
            code,
            message: CString::new(msg).ok(),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::None;
    }

    pub fn category(&self) -> ErrorCategoryT {
        match self {
            Self::None => ErrorCategoryT::None,
            Self::MongoAC { .. } => ErrorCategoryT::MongoAC,
            Self::Bson(..) => ErrorCategoryT::Bson,
            Self::Rust(..) => ErrorCategoryT::Rust,
        }
    }

    pub fn code(&self) -> ErrorCodeT {
        match self {
            Self::None => ErrorCodeT::Ok,
            // mongodb::bson::error::Error does not use integral error codes.
            Self::Bson(..) => ErrorCodeT::Unknown(i32::MIN),
            Self::MongoAC { code, .. } => *code,
            Self::Rust(err, _) => match err.kind.as_ref() {
                mongodb::error::ErrorKind::Command(cmd) => ErrorCodeT::from(cmd.code),
                _ => ErrorCodeT::Unknown(i32::MIN),
            },
        }
    }

    pub fn message(&self) -> Option<&CString> {
        match self {
            Self::None => None,
            Self::MongoAC { message, .. } => message.as_ref(),
            Self::Bson(_, msg) | Self::Rust(_, msg) => msg.as_ref(),
        }
    }

    pub fn has_label(&self, label: &str) -> bool {
        match self {
            Self::Rust(err, _) => err.contains_label(label),
            _ => false,
        }
    }
}

// TODO: remove in favor of descriptive mongoac errors?
impl From<mongodb::bson::error::Error> for ErrorT {
    fn from(err: mongodb::bson::error::Error) -> Self {
        let msg = CString::new(err.to_string()).ok();
        Self::Bson(err, msg)
    }
}

impl From<mongodb::error::Error> for ErrorT {
    fn from(err: mongodb::error::Error) -> Self {
        let msg = CString::new(err.to_string()).ok();
        Self::Rust(err, msg)
    }
}

impl From<tokio::time::error::Elapsed> for ErrorT {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        let msg = CString::new(error.to_string()).ok();
        Self::MongoAC {
            code: ErrorCodeT::Timeout,
            message: msg,
        }
    }
}

static EMPTY_MSG: [u8; 1] = [0];
