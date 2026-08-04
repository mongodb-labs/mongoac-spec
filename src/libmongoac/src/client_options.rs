use crate::error::ErrorT;
use crate::private::macros::*;
use crate::read_concern::ReadConcernT;
use crate::read_preference::ReadPreferenceT;
use crate::server_api::ServerApiT;
use crate::tls::TlsOptionsT;
use crate::write_concern::WriteConcernT;
use crate::{credential::CredentialT, error::ErrorCodeT};

use mongodb::options::{
    ClientOptions, Compressor, DriverInfo, ServerAddress, ServerMonitoringMode, Tls,
};
use std::ffi::c_char;
use std::time::Duration;

#[allow(non_camel_case_types)]
pub type mongoac_server_monitoring_mode_t = i32;
pub const MONGOAC_SERVER_MONITORING_MODE_AUTO: mongoac_server_monitoring_mode_t = 0;
pub const MONGOAC_SERVER_MONITORING_MODE_STREAM: mongoac_server_monitoring_mode_t = 1;
pub const MONGOAC_SERVER_MONITORING_MODE_POLL: mongoac_server_monitoring_mode_t = 2;
#[derive(Debug, Clone, Copy, PartialEq, Eq, num_enum::FromPrimitive, num_enum::IntoPrimitive)]
#[repr(i32)]
pub enum ServerMonitoringModeT {
    Auto = MONGOAC_SERVER_MONITORING_MODE_AUTO,
    Stream = MONGOAC_SERVER_MONITORING_MODE_STREAM,
    Poll = MONGOAC_SERVER_MONITORING_MODE_POLL,

    #[num_enum(catch_all)]
    Unknown(i32),
}

#[derive(Default)]
pub struct ClientOptionsT {
    pub inner: ClientOptions,
    pub capture_command_events: bool,
    pub capture_cmap_events: bool,
    pub capture_sdam_events: bool,
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
    v: bool,
) {
    safe_as_mut!(opts).capture_command_events = v;
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_capture_cmap_events(
    opts: *mut ClientOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).capture_cmap_events = v;
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_capture_sdam_events(
    opts: *mut ClientOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).capture_sdam_events = v;
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_server_api(
    options: *mut ClientOptionsT,
    v: *const ServerApiT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let options = safe_as_mut_with_error!(options, error);

    options.inner.server_api = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_app_name(
    opts: *mut ClientOptionsT,
    v: *const c_char,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);

    opts.inner.app_name = safe_optional_cstr_from_ptr_with_error!(v, error).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_connect_timeout_ms(opts: *mut ClientOptionsT, v: u64) {
    safe_as_mut!(opts).inner.connect_timeout = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_direct_connection(opts: *mut ClientOptionsT, v: bool) {
    safe_as_mut!(opts).inner.direct_connection = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_driver_info(
    opts: *mut ClientOptionsT,
    name: *const c_char,
    version: *const c_char,
    platform: *const c_char,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let name = safe_cstr_from_ptr_with_error!(name, error);
    let version = safe_optional_cstr_from_ptr_with_error!(version, error);
    let platform = safe_optional_cstr_from_ptr_with_error!(platform, error);

    opts.inner.driver_info = Some(
        DriverInfo::builder()
            .name(name)
            .version(version.map(Into::into))
            .platform(platform.map(Into::into))
            .build(),
    );
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_enable_overload_retargeting(
    opts: *mut ClientOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).inner.enable_overload_retargeting = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_heartbeat_freq_ms(opts: *mut ClientOptionsT, v: u64) {
    safe_as_mut!(opts).inner.heartbeat_freq = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_host(opts: *mut ClientOptionsT, host: *const c_char) {
    let opts = safe_as_mut!(opts);
    let host = safe_cstr_from_ptr!(host);

    opts.inner.hosts.push(ServerAddress::Tcp {
        host: host.to_string(),
        port: None,
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_host_and_port(
    opts: *mut ClientOptionsT,
    host: *const c_char,
    port: u16,
) {
    let opts = safe_as_mut!(opts);
    let host = safe_cstr_from_ptr!(host);

    opts.inner.hosts.push(ServerAddress::Tcp {
        host: host.to_string(),
        port: Some(port),
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_unix_path(
    opts: *mut ClientOptionsT,
    path: *const c_char,
) {
    let opts = safe_as_mut!(opts);
    let path = safe_cstr_from_ptr!(path);

    opts.inner
        .hosts
        .push(ServerAddress::Unix { path: path.into() });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_load_balanced(opts: *mut ClientOptionsT, v: bool) {
    safe_as_mut!(opts).inner.load_balanced = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_local_threshold_ms(opts: *mut ClientOptionsT, v: u64) {
    safe_as_mut!(opts).inner.local_threshold = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_max_adaptive_retries(
    opts: *mut ClientOptionsT,
    v: u32,
) {
    safe_as_mut!(opts).inner.max_adaptive_retries = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_max_connecting(opts: *mut ClientOptionsT, v: u32) {
    safe_as_mut!(opts).inner.max_connecting = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_max_idle_time_ms(opts: *mut ClientOptionsT, v: u64) {
    safe_as_mut!(opts).inner.max_idle_time = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_max_pool_size(opts: *mut ClientOptionsT, v: u32) {
    safe_as_mut!(opts).inner.max_pool_size = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_min_pool_size(opts: *mut ClientOptionsT, v: u32) {
    safe_as_mut!(opts).inner.min_pool_size = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_read_concern(
    opts: *mut ClientOptionsT,
    v: *const ReadConcernT,
) {
    safe_as_mut!(opts).inner.read_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_write_concern(
    opts: *mut ClientOptionsT,
    v: *const WriteConcernT,
) {
    safe_as_mut!(opts).inner.write_concern = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_read_preference(
    opts: *mut ClientOptionsT,
    v: *const ReadPreferenceT,
) {
    safe_as_mut!(opts).inner.selection_criteria = safe_optional_as_ref!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_repl_set_name(
    opts: *mut ClientOptionsT,
    v: *const c_char,
) {
    safe_as_mut!(opts).inner.repl_set_name = safe_optional_cstr_from_ptr!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_retry_reads(opts: *mut ClientOptionsT, v: bool) {
    safe_as_mut!(opts).inner.retry_reads = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_retry_writes(opts: *mut ClientOptionsT, v: bool) {
    safe_as_mut!(opts).inner.retry_writes = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_server_monitoring_mode(
    opts: *mut ClientOptionsT,
    v: mongoac_server_monitoring_mode_t,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = ServerMonitoringModeT::from(v);

    opts.inner.server_monitoring_mode = Some(safe_error!(v.try_into(), error));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_server_selection_timeout_ms(
    opts: *mut ClientOptionsT,
    v: u64,
) {
    safe_as_mut!(opts).inner.server_selection_timeout = Some(Duration::from_millis(v));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_default_database(
    opts: *mut ClientOptionsT,
    v: *const c_char,
) {
    safe_as_mut!(opts).inner.default_database = safe_optional_cstr_from_ptr!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_srv_max_hosts(opts: *mut ClientOptionsT, v: u32) {
    safe_as_mut!(opts).inner.srv_max_hosts = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_srv_service_name(
    opts: *mut ClientOptionsT,
    v: *const c_char,
) {
    safe_as_mut!(opts).inner.srv_service_name = safe_optional_cstr_from_ptr!(v).map(Into::into);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_compressor_snappy(opts: *mut ClientOptionsT) {
    safe_as_mut!(opts)
        .inner
        .compressors
        .get_or_insert_default()
        .push(Compressor::Snappy);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_compressor_zlib(opts: *mut ClientOptionsT) {
    safe_as_mut!(opts)
        .inner
        .compressors
        .get_or_insert_default()
        .push(Compressor::Zlib { level: None });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_compressor_zlib_with_level(
    opts: *mut ClientOptionsT,
    v: u32,
) {
    safe_as_mut!(opts)
        .inner
        .compressors
        .get_or_insert_default()
        .push(Compressor::Zlib { level: Some(v) });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_compressor_zstd(opts: *mut ClientOptionsT) {
    safe_as_mut!(opts)
        .inner
        .compressors
        .get_or_insert_default()
        .push(Compressor::Zstd { level: None });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_add_compressor_zstd_with_level(
    opts: *mut ClientOptionsT,
    v: i32,
) {
    safe_as_mut!(opts)
        .inner
        .compressors
        .get_or_insert_default()
        .push(Compressor::Zstd { level: Some(v) });
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_tls(opts: *mut ClientOptionsT, v: *const TlsOptionsT) {
    safe_as_mut!(opts).inner.tls = safe_optional_as_ref!(v).map(|v| Tls::Enabled(v.into()));
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_options_set_credential(
    opts: *mut ClientOptionsT,
    v: *const CredentialT,
) {
    safe_as_mut!(opts).inner.credential = safe_optional_as_ref!(v).map(Into::into);
}

impl From<&ClientOptionsT> for ClientOptions {
    fn from(opts: &ClientOptionsT) -> Self {
        opts.inner.clone()
    }
}

impl TryFrom<ServerMonitoringModeT> for ServerMonitoringMode {
    type Error = ErrorT;

    fn try_from(v: ServerMonitoringModeT) -> Result<Self, Self::Error> {
        match v {
            ServerMonitoringModeT::Auto => Ok(ServerMonitoringMode::Auto),
            ServerMonitoringModeT::Stream => Ok(ServerMonitoringMode::Stream),
            ServerMonitoringModeT::Poll => Ok(ServerMonitoringMode::Poll),
            ServerMonitoringModeT::Unknown(v) => Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &format!("unknown mongoac_server_monitoring_mode_t: {v}"),
            )),
        }
    }
}
