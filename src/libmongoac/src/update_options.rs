use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::write_concern::WriteConcernT;

use mongodb::bson::Bson;
use mongodb::options::Hint;
use mongodb::options::UpdateOptions;

pub struct UpdateOptionsT(UpdateOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_new() -> *mut UpdateOptionsT {
    Box::into_raw(Box::new(UpdateOptionsT(UpdateOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_destroy(opts: *mut UpdateOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_add_array_filter(
    opts: *mut UpdateOptionsT,
    filter: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let filter = safe_bson_view_with_error!(filter, error);

    opts.0
        .array_filters
        .get_or_insert_with(Vec::new)
        .push(safe_error!((&filter).try_into(), error)); // Deep-copy!
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_bypass_document_validation(
    opts: *mut UpdateOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.bypass_document_validation = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_upsert(opts: *mut UpdateOptionsT, v: bool) {
    safe_as_mut!(opts).0.upsert = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_write_concern(
    opts: *mut UpdateOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_hint(
    opts: *mut UpdateOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.hint = match v {
        Some(ref v) => Some(Hint::Keys(safe_error!(v.try_into(), error))), // Deep-copy!
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_let(
    opts: *mut UpdateOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.let_vars = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)), // Deep-copy!
        None => None,
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_comment(
    opts: *mut UpdateOptionsT,
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

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_update_options_set_sort(
    opts: *mut UpdateOptionsT,
    v: BsonViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_bson_view!(v);

    opts.0.sort = match v {
        Some(ref v) => Some(safe_error!(v.try_into(), error)), // Deep-copy!
        None => None,
    };
}

impl From<&UpdateOptionsT> for UpdateOptions {
    fn from(value: &UpdateOptionsT) -> Self {
        value.0.clone()
    }
}
