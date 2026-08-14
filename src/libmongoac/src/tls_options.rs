use crate::error::ErrorT;
use crate::private::macros::*;
use crate::string::StringViewT;

use mongodb::options::TlsOptions;
use std::path::PathBuf;
pub struct TlsOptionsT(TlsOptions);

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_tls_options_new() -> *mut TlsOptionsT {
    Box::into_raw(Box::new(TlsOptionsT(TlsOptions::default())))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_tls_options_destroy(opts: *mut TlsOptionsT) {
    safe_drop!(opts);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_tls_options_set_allow_invalid_certificates(
    opts: *mut TlsOptionsT,
    v: bool,
) {
    safe_as_mut!(opts).0.allow_invalid_certificates = Some(v);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_tls_options_set_ca_file_path(
    opts: *mut TlsOptionsT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_string_view_with_error!(v, error);

    opts.0.ca_file_path = v.map(PathBuf::from);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_tls_options_set_cert_key_file_path(
    opts: *mut TlsOptionsT,
    v: StringViewT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let opts = safe_as_mut_with_error!(opts, error);
    let v = safe_optional_string_view_with_error!(v, error);

    opts.0.cert_key_file_path = v.map(PathBuf::from);
}

impl From<&TlsOptionsT> for TlsOptions {
    fn from(value: &TlsOptionsT) -> Self {
        value.0.clone()
    }
}
