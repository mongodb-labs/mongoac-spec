use crate::error::ErrorT;
use crate::private::bson::bson_t;
use crate::private::macros::*;

use mongodb::options::CreateCollectionOptions;

pub struct CreateCollectionOptionsT(CreateCollectionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_create_collection_options_new() -> *mut CreateCollectionOptionsT {
    Box::into_raw(Box::new(CreateCollectionOptionsT(
        CreateCollectionOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_create_collection_options_destroy(opts: *mut CreateCollectionOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_create_collection_options_set_from_bson(
    opts: *mut CreateCollectionOptionsT,
    v: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);

    // TODO: typed struct.
    if let Some(v) = safe_optional_bson_opts_with_error!(CreateCollectionOptions, v, error) {
        opts.0 = v;
    }
}

impl From<&CreateCollectionOptionsT> for CreateCollectionOptions {
    fn from(value: &CreateCollectionOptionsT) -> Self {
        value.0.clone()
    }
}
