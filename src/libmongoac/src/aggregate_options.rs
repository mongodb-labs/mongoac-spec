use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::read_concern::ReadConcernT;
use crate::write_concern::WriteConcernT;

use mongodb::bson::Bson;
use mongodb::options::AggregateOptions;
use mongodb::options::Hint;

use std::time::Duration;

pub struct AggregateOptionsT(AggregateOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_new() -> *mut AggregateOptionsT {
    Box::into_raw(Box::new(AggregateOptionsT(AggregateOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_destroy(opts: *mut AggregateOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_allow_disk_use(
    opts: *mut AggregateOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.allow_disk_use = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_batch_size(opts: *mut AggregateOptionsT, v: u32) {
    safe_as_mut!(opts).0.batch_size = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_bypass_document_validation(
    opts: *mut AggregateOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.bypass_document_validation = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_max_time_ms(opts: *mut AggregateOptionsT, v: u64) {
    safe_as_mut!(opts).0.max_time = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_read_concern(
    opts: *mut AggregateOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_write_concern(
    opts: *mut AggregateOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_aggregate_options_set_hint(
    opts: *mut AggregateOptionsT,
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
pub extern "C" fn mongoac_aggregate_options_set_let(
    opts: *mut AggregateOptionsT,
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
pub extern "C" fn mongoac_aggregate_options_set_comment(
    opts: *mut AggregateOptionsT,
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

impl From<&AggregateOptionsT> for AggregateOptions {
    fn from(value: &AggregateOptionsT) -> Self {
        value.0.clone()
    }
}
