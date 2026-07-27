use std::sync::Arc;

use mongodb::ClientSession;
use tokio::sync::Mutex;

use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::macros::*;

#[derive(Clone)]
pub struct ClientSessionT {
    pub(crate) state: Arc<Mutex<ClientSession>>,
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

impl ClientSessionT {
    pub fn new(session: ClientSession) -> Self {
        Self {
            state: Arc::new(Mutex::new(session)),
        }
    }

    pub(crate) fn state(&self) -> &Arc<Mutex<ClientSession>> {
        &self.state
    }
}
