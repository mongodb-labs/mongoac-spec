use crate::private::macros::*;
use crate::read_preference::ReadPreferenceT;
use crate::server_selector::ServerSelectorT;

use mongodb::options::RunCommandOptions;

pub struct RunCommandOptionsT(RunCommandOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_command_options_new() -> *mut RunCommandOptionsT {
    Box::into_raw(Box::new(RunCommandOptionsT(RunCommandOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_command_options_destroy(opts: *mut RunCommandOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_command_options_set_read_preference(
    opts: *mut RunCommandOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_run_command_options_set_server_selector(
    opts: *mut RunCommandOptionsT,
    v: *const ServerSelectorT,
) {
    safe_as_mut!(opts).0.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&RunCommandOptionsT> for RunCommandOptions {
    fn from(value: &RunCommandOptionsT) -> Self {
        value.0.clone()
    }
}
