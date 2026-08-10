use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::Bson;
use mongodb::options::DeleteOptions;
use mongodb::options::Hint;

pub struct DeleteOptionsT(DeleteOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_new() -> *mut DeleteOptionsT {
    Box::into_raw(Box::new(DeleteOptionsT(DeleteOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_destroy(opts: *mut DeleteOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_set_write_concern(
    opts: *mut DeleteOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_set_hint(
    opts: *mut DeleteOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.hint = match v {
        Some(ref v) => Some(Hint::Keys(safe_error!(v.try_into(), error))),
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_set_let(
    opts: *mut DeleteOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.let_vars = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)),
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_delete_options_set_comment(
    opts: *mut DeleteOptionsT,
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

impl From<&DeleteOptionsT> for DeleteOptions {
    fn from(value: &DeleteOptionsT) -> Self {
        value.0.clone()
    }
}
