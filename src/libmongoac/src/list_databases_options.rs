use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::bson::Bson;
use mongodb::options::ListDatabasesOptions;

pub struct ListDatabasesOptionsT(ListDatabasesOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_databases_options_new() -> *mut ListDatabasesOptionsT {
    Box::into_raw(Box::new(ListDatabasesOptionsT(
        ListDatabasesOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_databases_options_destroy(opts: *mut ListDatabasesOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_databases_options_set_authorized_databases(
    opts: *mut ListDatabasesOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.authorized_databases = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_databases_options_set_filter(
    opts: *mut ListDatabasesOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.filter = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)),
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_databases_options_set_comment(
    opts: *mut ListDatabasesOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.comment = match v {
        Some(ref v) => Some(Bson::Document(safe_error!(v.try_into(), error))),
        None => None,
    };
}

impl From<&ListDatabasesOptionsT> for ListDatabasesOptions {
    fn from(value: &ListDatabasesOptionsT) -> Self {
        value.0.clone()
    }
}
