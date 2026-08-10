use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::read_concern::ReadConcernT;

use mongodb::bson::Bson;
use mongodb::options::EstimatedDocumentCountOptions;

use std::time::Duration;

pub struct EstimatedDocumentCountOptionsT(EstimatedDocumentCountOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_estimated_document_count_options_new()
-> *mut EstimatedDocumentCountOptionsT {
    Box::into_raw(Box::new(EstimatedDocumentCountOptionsT(
        EstimatedDocumentCountOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_estimated_document_count_options_destroy(
    opts: *mut EstimatedDocumentCountOptionsT,
) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_estimated_document_count_options_set_max_time_ms(
    opts: *mut EstimatedDocumentCountOptionsT,
    v: u64,
) {
    safe_as_mut!(opts).0.max_time = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_estimated_document_count_options_set_read_concern(
    opts: *mut EstimatedDocumentCountOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_estimated_document_count_options_set_comment(
    opts: *mut EstimatedDocumentCountOptionsT,
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

impl From<&EstimatedDocumentCountOptionsT> for EstimatedDocumentCountOptions {
    fn from(value: &EstimatedDocumentCountOptionsT) -> Self {
        value.0.clone()
    }
}
