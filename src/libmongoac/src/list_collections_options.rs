use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::bson::Bson;
use mongodb::options::ListCollectionsOptions;

pub struct ListCollectionsOptionsT(ListCollectionsOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_new() -> *mut ListCollectionsOptionsT {
    Box::into_raw(Box::new(ListCollectionsOptionsT(
        ListCollectionsOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_destroy(opts: *mut ListCollectionsOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_set_batch_size(
    opts: *mut ListCollectionsOptionsT,
    v: u32,
) {
    safe_as_mut!(opts).0.batch_size = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_set_authorized_collections(
    opts: *mut ListCollectionsOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.authorized_collections = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_set_filter(
    opts: *mut ListCollectionsOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.filter = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)), // Deep-copy!
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_list_collections_options_set_comment(
    opts: *mut ListCollectionsOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.comment = match v {
        Some(ref v) => Some(Bson::Document(safe_error!(v.try_into(), error))), // Deep-copy!
        None => None,
    };
}

impl From<&ListCollectionsOptionsT> for ListCollectionsOptions {
    fn from(value: &ListCollectionsOptionsT) -> Self {
        value.0.clone()
    }
}
