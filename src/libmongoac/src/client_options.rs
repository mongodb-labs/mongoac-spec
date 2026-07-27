use crate::error::ErrorT;
use crate::private::bson::bson_t;
use crate::private::macros::*;
use mongodb::options::ServerApi;

#[derive(Default)]
pub struct ClientOptionsT {
    capture_command_events: bool,
    capture_cmap_events: bool,
    capture_sdam_events: bool,
    server_api: Option<ServerApi>,
}
impl ClientOptionsT {
    pub fn set_capture_command_events(&mut self, value: bool) {
        self.capture_command_events = value;
    }

    pub fn set_capture_cmap_events(&mut self, value: bool) {
        self.capture_cmap_events = value;
    }

    pub fn set_capture_sdam_events(&mut self, value: bool) {
        self.capture_sdam_events = value;
    }

    pub fn set_server_api(&mut self, api: Option<ServerApi>) {
        self.server_api = api
    }

    pub(crate) fn capture_command_events(&self) -> bool {
        self.capture_command_events
    }

    pub(crate) fn capture_cmap_events(&self) -> bool {
        self.capture_cmap_events
    }

    pub(crate) fn capture_sdam_events(&self) -> bool {
        self.capture_sdam_events
    }

    pub(crate) fn server_api(&self) -> Option<&ServerApi> {
        self.server_api.as_ref()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_new() -> *mut ClientOptionsT {
    Box::into_raw(Box::new(ClientOptionsT::default()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_destroy(opts: *mut ClientOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_capture_command_events(
    opts: *mut ClientOptionsT,
    value: bool,
) {
    let opts = safe_as_mut!(opts);

    opts.set_capture_command_events(value);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_capture_cmap_events(
    opts: *mut ClientOptionsT,
    value: bool,
) {
    let opts = safe_as_mut!(opts);

    opts.set_capture_cmap_events(value);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_capture_sdam_events(
    opts: *mut ClientOptionsT,
    value: bool,
) {
    let opts = safe_as_mut!(opts);

    opts.set_capture_sdam_events(value);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_server_api(
    options: *mut ClientOptionsT,
    api: *const bson_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let options = safe_as_mut_with_error!(options, error);
    let api = safe_optional_bson_opts_with_error!(ServerApi, api, error);

    options.set_server_api(api)
}
