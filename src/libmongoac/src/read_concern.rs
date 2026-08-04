use std::ffi::c_char;

use crate::error::ErrorT;
use crate::private::macros::*;

use mongodb::options::ReadConcern;

pub struct ReadConcernT(ReadConcern);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_new() -> *mut ReadConcernT {
    Box::into_raw(Box::new(ReadConcernT(ReadConcern::local())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_destroy(rc: *mut ReadConcernT) {
    safe_drop!(rc);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_local(rc: *mut ReadConcernT) {
    safe_as_mut!(rc).0 = ReadConcern::local();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_majority(rc: *mut ReadConcernT) {
    safe_as_mut!(rc).0 = ReadConcern::majority();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_linearizable(rc: *mut ReadConcernT) {
    safe_as_mut!(rc).0 = ReadConcern::linearizable();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_available(rc: *mut ReadConcernT) {
    safe_as_mut!(rc).0 = ReadConcern::available();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_snapshot(rc: *mut ReadConcernT) {
    safe_as_mut!(rc).0 = ReadConcern::snapshot();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_read_concern_set_level_string(
    rc: *mut ReadConcernT,
    v: *const c_char,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let rc = safe_as_mut_with_error!(rc, error);
    let v = safe_cstr_from_ptr_with_error!(v, error);

    rc.0 = ReadConcern::custom(v);
}

impl From<&ReadConcernT> for ReadConcern {
    fn from(value: &ReadConcernT) -> Self {
        value.0.clone()
    }
}
