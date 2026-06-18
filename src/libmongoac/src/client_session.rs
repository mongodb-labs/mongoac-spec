use crate::safe_drop;

/// Opaque handle representing a client session.
///
/// Wraps `Arc<tokio::sync::Mutex<ClientSession>>` for thread safety across the
/// two-thread polling model (application thread polls futures, worker thread
/// drives `make_progress()`). All session access — both async spawned tasks and
/// synchronous accessors — goes through the mutex.
///
/// The `Arc` enables session sharing with cursors: cursor-creating operations
/// clone the `Arc` into the cursor so the session outlives the user's handle.
/// `mongoac_client_session_destroy()` drops the handle, but the `Arc` inside
/// the cursor keeps the session alive until cursor destruction completes.
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
