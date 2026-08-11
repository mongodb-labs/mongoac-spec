use crate::private::macros::*;
use crate::string::StringT;

use mongodb::error::{ErrorKind, WriteFailure};
use strum::EnumMessage;

#[allow(non_camel_case_types)]
pub type mongoac_error_category_t = i32;
pub const MONGOAC_ERROR_CATEGORY_NONE: mongoac_error_category_t = 0;
pub const MONGOAC_ERROR_CATEGORY_MONGOAC: mongoac_error_category_t = 1;
pub const MONGOAC_ERROR_CATEGORY_SERVER: mongoac_error_category_t = 2;
pub const MONGOAC_ERROR_CATEGORY_RUST: mongoac_error_category_t = 3;
pub const MONGOAC_ERROR_CATEGORY_BSON: mongoac_error_category_t = 4;
pub const MONGOAC_ERROR_CATEGORY_UNKNOWN: mongoac_error_category_t = i32::MIN;

#[allow(non_camel_case_types)]
pub type mongoac_error_code_t = i32;
pub const MONGOAC_ERROR_CODE_OK: mongoac_error_code_t = 0;
pub const MONGOAC_ERROR_CODE_INVALID_ARGUMENT: mongoac_error_code_t = 1;
pub const MONGOAC_ERROR_CODE_RUNTIME_ERROR: mongoac_error_code_t = 2;
pub const MONGOAC_ERROR_CODE_TIMEOUT: mongoac_error_code_t = 3;
pub const MONGOAC_ERROR_CODE_UNKNOWN: mongoac_error_code_t = i32::MIN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum ErrorCategoryT {
    None = MONGOAC_ERROR_CATEGORY_NONE,
    MongoAC = MONGOAC_ERROR_CATEGORY_MONGOAC,
    Server = MONGOAC_ERROR_CATEGORY_SERVER,
    Rust = MONGOAC_ERROR_CATEGORY_RUST,
    Bson = MONGOAC_ERROR_CATEGORY_BSON,

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
pub extern "C" fn mongoac_error_message(error: *const ErrorT) -> StringT {
    safe_as_ref!(error)
        .message()
        .map(Into::into)
        .unwrap_or_default()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_error_contains_label(
    error: *const ErrorT,
    label: *const std::ffi::c_char,
) -> bool {
    safe_as_ref!(error).contains_label(safe_cstr_from_ptr!(label))
}

#[derive(Clone, Debug, Default)]
pub enum ErrorT {
    #[default]
    None,
    MongoAC {
        code: ErrorCodeT,
        message: Option<String>,
    },
    Server(mongodb::error::Error),
    Rust(mongodb::error::Error),
    Bson(mongodb::bson::error::Error),
}

impl ErrorT {
    #[must_use]
    pub fn new() -> Self {
        Self::None
    }

    pub(crate) fn from_mongoac(code: ErrorCodeT, msg: &str) -> Self {
        Self::MongoAC {
            code,
            message: Some(msg.to_string()),
        }
    }

    pub fn clear(&mut self) {
        *self = Self::None;
    }

    #[must_use]
    pub fn category(&self) -> ErrorCategoryT {
        match self {
            Self::None => ErrorCategoryT::None,
            Self::MongoAC { .. } => ErrorCategoryT::MongoAC,
            Self::Server(_) => ErrorCategoryT::Server,
            Self::Rust(_) => ErrorCategoryT::Rust,
            Self::Bson(_) => ErrorCategoryT::Bson,
        }
    }

    #[must_use]
    pub fn code(&self) -> ErrorCodeT {
        match self {
            Self::None => ErrorCodeT::Ok,

            // mongoac defines its own integral error codes.
            Self::MongoAC { code, .. } => *code,

            // Only support variants which provide a single unambiguous `err.code`.
            Self::Server(err) => match err.kind.as_ref() {
                ErrorKind::Command(err) => ErrorCodeT::from(err.code),
                ErrorKind::Write(err) => match err {
                    WriteFailure::WriteError(we) => ErrorCodeT::from(we.code),
                    WriteFailure::WriteConcernError(wce) => ErrorCodeT::from(wce.code),
                    _ => ErrorCodeT::Unknown(MONGOAC_ERROR_CODE_UNKNOWN), // #[non_exhaustive]
                },
                _ => ErrorCodeT::Unknown(MONGOAC_ERROR_CODE_UNKNOWN), // #[non_exhaustive]
            },

            Self::Rust(_) => ErrorCodeT::Unknown(MONGOAC_ERROR_CODE_UNKNOWN),
            Self::Bson(_) => ErrorCodeT::Unknown(MONGOAC_ERROR_CODE_UNKNOWN),
        }
    }

    #[must_use]
    pub fn message(&self) -> Option<String> {
        match self {
            Self::None => None,

            Self::MongoAC { code, message, .. } => {
                // All variants must have `#[strum(message = "...")]`.
                let prefix = code.get_message().unwrap_or_default();

                Some(match message {
                    Some(msg) => format!("{}: {}", prefix, msg),
                    None => prefix.to_string(),
                })
            }

            Self::Server(err) | Self::Rust(err) => Some(err.to_string()),
            Self::Bson(err) => Some(err.to_string()),
        }
    }

    #[must_use]
    pub fn contains_label(&self, label: &str) -> bool {
        if let Self::Server(err) | Self::Rust(err) = self {
            return err.contains_label(label);
        }

        false
    }
}

// TODO: remove in favor of descriptive mongoac errors?
impl From<mongodb::bson::error::Error> for ErrorT {
    fn from(err: mongodb::bson::error::Error) -> Self {
        Self::Bson(err)
    }
}

impl From<mongodb::error::Error> for ErrorT {
    fn from(err: mongodb::error::Error) -> Self {
        match err.kind.as_ref() {
            ErrorKind::Command(_) | ErrorKind::Write(_) => Self::Server(err),
            _ => Self::Rust(err),
        }
    }
}

impl From<tokio::time::error::Elapsed> for ErrorT {
    fn from(error: tokio::time::error::Elapsed) -> Self {
        Self::MongoAC {
            code: ErrorCodeT::Timeout,
            message: Some(error.to_string()),
        }
    }
}
