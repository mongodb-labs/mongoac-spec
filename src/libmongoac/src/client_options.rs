use crate::{
    safe_as_mut, safe_as_mut_with_error, safe_drop, safe_optional_as_ref,
    safe_optional_error_as_mut,
};

use crate::error::ErrorT;
use crate::private::bson::bson_t;
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

    pub fn set_server_api(
        &mut self,
        api: Option<&bson_t>,
    ) -> Result<(), mongodb::bson::error::Error> {
        match api {
            Some(bson_ref) => {
                let doc = mongodb::bson::Document::try_from(bson_ref)?;
                let server_api = mongodb::bson::deserialize_from_document::<ServerApi>(doc)?;
                self.server_api = Some(server_api);
                Ok(())
            }
            None => {
                self.server_api = None;
                Ok(())
            }
        }
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
    let api = safe_optional_as_ref!(api);

    if let Err(err) = options.set_server_api(api) {
        if let Some(e) = error {
            *e = ErrorT::from_bson(&err);
        }
    }
}
