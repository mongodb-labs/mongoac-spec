use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::options::FindOptions;

pub struct FindOptionsT(FindOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_options_new() -> *mut FindOptionsT {
    Box::into_raw(Box::new(FindOptionsT(FindOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_options_destroy(opts: *mut FindOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_options_set_from_bson(
    opts: *mut FindOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);

    // TODO: typed struct.
    if let Some(v) = safe_optional_bson_opts_with_error!(FindOptions, v, error) {
        opts.0 = v;
    }
}

impl From<&FindOptionsT> for FindOptions {
    fn from(value: &FindOptionsT) -> Self {
        value.0.clone()
    }
}
