use crate::private::macros::*;
use crate::read_concern::ReadConcernT;
use crate::read_preference::ReadPreferenceT;
use crate::write_concern::WriteConcernT;

use mongodb::options::TransactionOptions;
use std::time::Duration;

pub struct TransactionOptionsT(TransactionOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_new() -> *mut TransactionOptionsT {
    Box::into_raw(Box::new(TransactionOptionsT(TransactionOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_destroy(to: *mut TransactionOptionsT) {
    safe_drop!(to);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_set_read_concern(
    to: *mut TransactionOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(to).0.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_set_write_concern(
    to: *mut TransactionOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(to).0.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_set_read_preference(
    to: *mut TransactionOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(to).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_transaction_options_set_max_commit_time_ms(
    to: *mut TransactionOptionsT,
    v: u64,
) {
    safe_as_mut!(to).0.max_commit_time = Some(Duration::from_millis(v));
}

impl From<&TransactionOptionsT> for TransactionOptions {
    fn from(value: &TransactionOptionsT) -> Self {
        value.0.clone()
    }
}
