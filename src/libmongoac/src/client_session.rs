use std::sync::Arc;

use mongodb::ClientSession;
use tokio::sync::Mutex;

use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::macros::*;

#[derive(Clone)]
pub struct ClientSessionT {
    state: Arc<Mutex<ClientSession>>,
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

#[macro_export]
macro_rules! op_with_session {
    ($op:expr, $session:expr) => {{
        let op = $op;
        let session = $session;
        match session {
            Some(session) => {
                let mut guard = session.state().lock().await;
                op.session(&mut *guard).await.map_err(ErrorT::from)
            }
            None => op.await.map_err(ErrorT::from),
        }
    }};
}
