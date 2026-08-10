use crate::bson::BsonViewT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::read_concern::ReadConcernT;

use mongodb::bson::Bson;
use mongodb::options::CountOptions;
use mongodb::options::Hint;

use std::time::Duration;

pub struct CountOptionsT(CountOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_new() -> *mut CountOptionsT {
    Box::into_raw(Box::new(CountOptionsT(CountOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_destroy(opts: *mut CountOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_set_limit(opts: *mut CountOptionsT, v: u64) {
    safe_as_mut!(opts).0.limit = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_set_skip(opts: *mut CountOptionsT, v: u64) {
    safe_as_mut!(opts).0.skip = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_set_max_time_ms(opts: *mut CountOptionsT, v: u64) {
    safe_as_mut!(opts).0.max_time = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_set_read_concern(
    opts: *mut CountOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_count_options_set_hint(
    opts: *mut CountOptionsT,
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
pub extern "C" fn mongoac_count_options_set_comment(
    opts: *mut CountOptionsT,
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

impl From<&CountOptionsT> for CountOptions {
    fn from(value: &CountOptionsT) -> Self {
        value.0.clone()
    }
}
