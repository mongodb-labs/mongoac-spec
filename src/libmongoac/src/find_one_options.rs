use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::options::FindOneOptions;

pub struct FindOneOptionsT(FindOneOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_one_options_destroy(opts: *mut FindOneOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_find_one_options_new_from_bson(
    v: BsonViewT,
    error: *mut ErrorT,
) -> *mut FindOneOptionsT {
    let error = safe_optional_error_as_mut!(error);

    // TODO: typed struct.
    let opts = safe_optional_bson_opts_with_error!(FindOneOptions, v, error).unwrap_or_default();

    Box::into_raw(Box::new(FindOneOptionsT(opts)))
}

impl From<&FindOneOptionsT> for FindOneOptions {
    fn from(value: &FindOneOptionsT) -> Self {
        value.0.clone()
    }
}
