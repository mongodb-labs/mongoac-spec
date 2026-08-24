use crate::bson::BsonViewT;
use crate::cursor_type::CursorTypeT;
use crate::error::ErrorT;
use crate::private::macros::*;
use crate::read_preference::ReadPreferenceT;
use crate::server_selector::ServerSelectorT;

use mongodb::bson::Bson;
use mongodb::options::RunCursorCommandOptions;
use std::time::Duration;

pub struct RunCursorCommandOptionsT(RunCursorCommandOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_new() -> *mut RunCursorCommandOptionsT {
    Box::into_raw(Box::new(RunCursorCommandOptionsT(
        RunCursorCommandOptions::default(),
    )))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_destroy(opts: *mut RunCursorCommandOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_read_preference(
    opts: *mut RunCursorCommandOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_server_selector(
    opts: *mut RunCursorCommandOptionsT,
    v: *const ServerSelectorT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_cursor_type(
    opts: *mut RunCursorCommandOptionsT,
    v: CursorTypeT,
) {
    safe_as_mut!(opts).0.cursor_type = Some(v.into());
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_batch_size(
    opts: *mut RunCursorCommandOptionsT,
    v: u32,
) {
    safe_as_mut!(opts).0.batch_size = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_max_time_ms(
    opts: *mut RunCursorCommandOptionsT,
    v: u64,
) {
    safe_as_mut!(opts).0.max_time = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_cursor_command_options_set_comment(
    opts: *mut RunCursorCommandOptionsT,
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

impl From<&RunCursorCommandOptionsT> for RunCursorCommandOptions {
    fn from(value: &RunCursorCommandOptionsT) -> Self {
        value.0.clone()
    }
}
