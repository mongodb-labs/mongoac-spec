use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::options::FindOptions;

pub struct FindOptionsT(FindOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_options_destroy(opts: *mut FindOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_options_new_from_bson(
    v: BsonViewT,
    error: *mut ErrorT,
) -> *mut FindOptionsT {
    let error = safe_optional_error_as_mut!(error);

    // TODO: typed struct.
    let opts = safe_optional_bson_opts_with_error!(FindOptions, v, error).unwrap_or_default();

    Box::into_raw(Box::new(FindOptionsT(opts)))
}

impl From<&FindOptionsT> for FindOptions {
    fn from(value: &FindOptionsT) -> Self {
        value.0.clone()
    }
}
