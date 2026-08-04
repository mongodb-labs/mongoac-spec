use crate::private::macros::*;
use crate::transaction_options::TransactionOptionsT;

use mongodb::bson::Timestamp;
use mongodb::options::SessionOptions;

pub struct SessionOptionsT(SessionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_new() -> *mut SessionOptionsT {
    Box::into_raw(Box::new(SessionOptionsT(SessionOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_destroy(opts: *mut SessionOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_set_causal_consistency(
    opts: *mut SessionOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.causal_consistency = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_set_snapshot(opts: *mut SessionOptionsT, v: bool) {
    safe_as_mut!(opts).0.snapshot = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_set_snapshot_time(
    opts: *mut SessionOptionsT,
    time: u32,
    increment: u32,
) {
    safe_as_mut!(opts).0.snapshot_time = Some(Timestamp { time, increment });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_session_options_set_default_transaction_options(
    opts: *mut SessionOptionsT,
    v: *const TransactionOptionsT,
) {
    safe_as_mut!(opts).0.default_transaction_options = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&SessionOptionsT> for SessionOptions {
    fn from(value: &SessionOptionsT) -> Self {
        value.0.clone()
    }
}
