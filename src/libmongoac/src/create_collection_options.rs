use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::options::CreateCollectionOptions;

pub struct CreateCollectionOptionsT(CreateCollectionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_create_collection_options_destroy(opts: *mut CreateCollectionOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_create_collection_options_new_from_bson(
    v: BsonViewT,
    error: *mut ErrorT,
) -> *mut CreateCollectionOptionsT {
    let error = safe_optional_error_as_mut!(error);

    // TODO: typed struct.
    let opts =
        safe_optional_bson_opts_with_error!(CreateCollectionOptions, v, error).unwrap_or_default();

    Box::into_raw(Box::new(CreateCollectionOptionsT(opts)))
}

impl From<&CreateCollectionOptionsT> for CreateCollectionOptions {
    fn from(value: &CreateCollectionOptionsT) -> Self {
        value.0.clone()
    }
}
