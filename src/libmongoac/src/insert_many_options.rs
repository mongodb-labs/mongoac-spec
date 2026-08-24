use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::Bson;
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

impl From<&InsertManyOptionsT> for InsertManyOptions {
    fn from(value: &InsertManyOptionsT) -> Self {
        value.0.clone()
    }
}
