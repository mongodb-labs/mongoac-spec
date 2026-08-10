use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::Bson;
use mongodb::options::Hint;
use mongodb::options::ReplaceOptions;

pub struct ReplaceOptionsT(ReplaceOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_new() -> *mut ReplaceOptionsT {
    Box::into_raw(Box::new(ReplaceOptionsT(ReplaceOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_destroy(opts: *mut ReplaceOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_set_bypass_document_validation(
    opts: *mut ReplaceOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.bypass_document_validation = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_set_upsert(opts: *mut ReplaceOptionsT, v: bool) {
    safe_as_mut!(opts).0.upsert = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_set_write_concern(
    opts: *mut ReplaceOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_set_hint(
    opts: *mut ReplaceOptionsT,
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
pub extern "C" fn mongoac_replace_options_set_let(
    opts: *mut ReplaceOptionsT,
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
pub extern "C" fn mongoac_replace_options_set_comment(
    opts: *mut ReplaceOptionsT,
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

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_replace_options_set_sort(
    opts: *mut ReplaceOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.sort = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)),
        None => None,
    };
}

impl From<&ReplaceOptionsT> for ReplaceOptions {
    fn from(value: &ReplaceOptionsT) -> Self {
        value.0.clone()
    }
}
