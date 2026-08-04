use crate::error::ErrorT;
use crate::private::bson::bson_t;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::{Bson, Document};
use mongodb::options::InsertOneOptions;

pub struct InsertOneOptionsT(InsertOneOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_one_options_new() -> *mut InsertOneOptionsT {
    Box::into_raw(Box::new(InsertOneOptionsT(InsertOneOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_one_options_destroy(opts: *mut InsertOneOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_one_options_set_bypass_document_validation(
    opts: *mut InsertOneOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.bypass_document_validation = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_one_options_set_write_concern(
    opts: *mut InsertOneOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_one_options_set_comment(
    opts: *mut InsertOneOptionsT,
    v: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_const_bson!(v);

    opts.0.comment = match v {
        Some(ref b) => Some(Bson::Document(safe_error!(Document::try_from(b), error))),
        None => None,
    };
}

impl From<&InsertOneOptionsT> for InsertOneOptions {
    fn from(value: &InsertOneOptionsT) -> Self {
        value.0.clone()
    }
}
