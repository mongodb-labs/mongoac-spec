use crate::bson::BsonViewT;
use crate::error::{ErrorCodeT, ErrorT};
use crate::private::macros::*;
use crate::string::StringViewT;

use mongodb::bson::Document;
use mongodb::options::{AuthMechanism, Credential};

#[allow(non_camel_case_types)]
pub type mongoac_auth_mechanism_t = i32;
pub const MONGOAC_AUTH_MECHANISM_SCRAM_SHA_1: mongoac_auth_mechanism_t = 0;
pub const MONGOAC_AUTH_MECHANISM_SCRAM_SHA_256: mongoac_auth_mechanism_t = 1;
pub const MONGOAC_AUTH_MECHANISM_MONGODB_X509: mongoac_auth_mechanism_t = 2;
pub const MONGOAC_AUTH_MECHANISM_PLAIN: mongoac_auth_mechanism_t = 3;
pub const MONGOAC_AUTH_MECHANISM_MONGODB_AWS: mongoac_auth_mechanism_t = 4;
pub const MONGOAC_AUTH_MECHANISM_MONGODB_OIDC: mongoac_auth_mechanism_t = 5;
#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum AuthMechanismT {
    ScramSha1 = MONGOAC_AUTH_MECHANISM_SCRAM_SHA_1,
    ScramSha256 = MONGOAC_AUTH_MECHANISM_SCRAM_SHA_256,
    MongoDbX509 = MONGOAC_AUTH_MECHANISM_MONGODB_X509,
    Plain = MONGOAC_AUTH_MECHANISM_PLAIN,
    MongoDbAws = MONGOAC_AUTH_MECHANISM_MONGODB_AWS,
    MongoDbOidc = MONGOAC_AUTH_MECHANISM_MONGODB_OIDC,

    #[num_enum(catch_all)]
    Unknown(i32),
}

pub struct CredentialT(Credential);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_new() -> *mut CredentialT {
    Box::into_raw(Box::new(CredentialT(Credential::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_destroy(cred: *mut CredentialT) {
    safe_drop!(cred);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_set_username(
    cred: *mut CredentialT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let cred = safe_as_mut_with_error!(cred, error);

    cred.0.username = safe_optional_string_view_with_error!(v, error).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_set_source(
    cred: *mut CredentialT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let cred = safe_as_mut_with_error!(cred, error);

    cred.0.source = safe_optional_string_view_with_error!(v, error).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_set_password(
    cred: *mut CredentialT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let cred = safe_as_mut_with_error!(cred, error);

    cred.0.password = safe_optional_string_view_with_error!(v, error).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_set_mechanism(
    cred: *mut CredentialT,
    v: mongoac_auth_mechanism_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let cred = safe_as_mut_with_error!(cred, error);

    cred.0.mechanism = Some(safe_error!(AuthMechanismT::from(v).try_into(), error));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_credential_set_mechanism_properties(
    cred: *mut CredentialT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let cred = safe_as_mut_with_error!(cred, error);

    cred.0.mechanism_properties = safe_optional_bson_opts_with_error!(Document, v, error);
}

impl From<&CredentialT> for Credential {
    fn from(value: &CredentialT) -> Self {
        value.0.clone()
    }
}

impl TryFrom<AuthMechanismT> for AuthMechanism {
    type Error = ErrorT;

    fn try_from(v: AuthMechanismT) -> Result<Self, Self::Error> {
        match v {
            AuthMechanismT::ScramSha1 => Ok(AuthMechanism::ScramSha1),
            AuthMechanismT::ScramSha256 => Ok(AuthMechanism::ScramSha256),
            AuthMechanismT::MongoDbX509 => Ok(AuthMechanism::MongoDbX509),
            AuthMechanismT::Plain => Ok(AuthMechanism::Plain),
            AuthMechanismT::MongoDbAws => Ok(AuthMechanism::MongoDbAws),
            AuthMechanismT::MongoDbOidc => Ok(AuthMechanism::MongoDbOidc),
            AuthMechanismT::Unknown(v) => Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &format!("unknown mongoac_auth_mechanism_t: {v}"),
            )),
        }
    }
}
