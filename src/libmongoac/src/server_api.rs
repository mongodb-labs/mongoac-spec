use crate::private::macros::*;

use mongodb::options::{ServerApi, ServerApiVersion};

pub struct ServerApiT(ServerApi);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_api_new() -> *mut ServerApiT {
    Box::into_raw(Box::new(ServerApiT(
        ServerApi::builder().version(ServerApiVersion::V1).build(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_api_destroy(api: *mut ServerApiT) {
    safe_drop!(api);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_api_set_strict(api: *mut ServerApiT, v: bool) {
    safe_as_mut!(api).0.strict = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_server_api_set_deprecation_errors(api: *mut ServerApiT, v: bool) {
    safe_as_mut!(api).0.deprecation_errors = Some(v);
}

impl From<&ServerApiT> for ServerApi {
    fn from(value: &ServerApiT) -> Self {
        value.0.clone()
    }
}
