use crate::error::ErrorT;
use crate::private::macros::*;
use crate::string::StringViewT;

use mongodb::options::{Acknowledgment, WriteConcern};
use std::time::Duration;

pub struct WriteConcernT(WriteConcern);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_new() -> *mut WriteConcernT {
    Box::into_raw(Box::new(WriteConcernT(WriteConcern::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_destroy(wc: *mut WriteConcernT) {
    safe_drop!(wc);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_set_w_majority(wc: *mut WriteConcernT) {
    safe_as_mut!(wc).0.w = Some(Acknowledgment::Majority);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_set_w_nodes(wc: *mut WriteConcernT, v: u32) {
    safe_as_mut!(wc).0.w = Some(Acknowledgment::Nodes(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_set_w_custom(
    wc: *mut WriteConcernT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let wc = safe_as_mut_with_error!(wc, error);
    let v = safe_string_view_with_error!(v, error);

    wc.0.w = Some(Acknowledgment::Custom(v.to_string()));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_set_w_timeout_ms(wc: *mut WriteConcernT, v: u64) {
    safe_as_mut!(wc).0.w_timeout = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_write_concern_set_journal(wc: *mut WriteConcernT, v: bool) {
    safe_as_mut!(wc).0.journal = Some(v);
}

impl From<&WriteConcernT> for WriteConcern {
    fn from(value: &WriteConcernT) -> Self {
        value.0.clone()
    }
}
