use crate::{
    error::ErrorT, future::FutureT, safe_as_ref_with_error, safe_drop, safe_optional_error_as_mut,
};

#[derive(Clone)]
pub struct ClientSessionT {
    pub(crate) inner: std::sync::Arc<tokio::sync::Mutex<mongodb::ClientSession>>,
}

impl ClientSessionT {
    pub fn new(session: mongodb::ClientSession) -> Self {
        Self {
            inner: std::sync::Arc::new(tokio::sync::Mutex::new(session)),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_session_destroy(session: *mut ClientSessionT) {
    safe_drop!(session);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_client_session(
    future: *mut FutureT,
    error: *mut ErrorT,
) -> *mut ClientSessionT {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    match future.value().get_client_session() {
        Ok(session) => Box::into_raw(Box::new(session.clone())),
        Err(err) => {
            if let Some(e) = error {
                *e = err;
            }
            Default::default()
        }
    }
}
