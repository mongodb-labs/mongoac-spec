use crate::error::ErrorT;
use crate::private::bson::bson_t;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::{Bson, Document};
use mongodb::options::InsertManyOptions;

pub struct InsertManyOptionsT(InsertManyOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_new() -> *mut InsertManyOptionsT {
    Box::into_raw(Box::new(InsertManyOptionsT(InsertManyOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_destroy(opts: *mut InsertManyOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_set_bypass_document_validation(
    opts: *mut InsertManyOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.bypass_document_validation = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_set_ordered(opts: *mut InsertManyOptionsT, v: bool) {
    safe_as_mut!(opts).0.ordered = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_set_write_concern(
    opts: *mut InsertManyOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_insert_many_options_set_comment(
    opts: *mut InsertManyOptionsT,
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

impl From<&InsertManyOptionsT> for InsertManyOptions {
    fn from(value: &InsertManyOptionsT) -> Self {
        value.0.clone()
    }
}
