use crate::safe_drop;

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
