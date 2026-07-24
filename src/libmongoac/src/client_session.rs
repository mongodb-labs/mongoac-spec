use std::sync::Arc;

use tokio::sync::Mutex;

use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::macros::*;

#[derive(Clone)]
pub struct ClientSessionT {
    pub(crate) inner: Arc<Mutex<mongodb::ClientSession>>,
}

impl ClientSessionT {
    pub fn new(session: mongodb::ClientSession) -> Self {
        Self {
            inner: Arc::new(Mutex::new(session)),
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_client_session_destroy(session: *mut ClientSessionT) {
    safe_drop!(session);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_client_session(
    future: *const FutureT,
    error: *mut ErrorT,
) -> *mut ClientSessionT {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);
    let session = safe_error!(future.get_client_session(), error);
    Box::into_raw(Box::new(session.clone()))
}
