use crate::bson::BsonViewT;
use crate::client_session::ClientSessionT;
use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;

use mongodb::bson::{RawDocument, RawDocumentBuf};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[derive(Clone)]
pub struct CursorT {
    state: Arc<AsyncMutex<CursorState>>,
    runtime: RuntimeT,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_cursor(
    future: *const FutureT,
    error: *mut ErrorT,
) -> *mut CursorT {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);
    let cursor = safe_error!(future.get_cursor(), error);
    Box::into_raw(Box::new(cursor.clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_destroy(cursor: *mut CursorT) {
    safe_drop!(cursor);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_clone(cursor: *const CursorT) -> *mut CursorT {
    Box::into_raw(Box::new(safe_as_ref!(cursor).clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_next(cursor: *const CursorT, error: *mut ErrorT) -> bool {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    safe_error!(cursor.next(), error)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_next_async(
    cursor: *const CursorT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    Box::into_raw(Box::new(cursor.next_async()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_current_async(
    cursor: *const CursorT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    Box::into_raw(Box::new(cursor.current_async()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_current(cursor: *const CursorT, error: *mut ErrorT) -> BsonViewT {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    cursor.current()
}

impl CursorT {
    pub(crate) fn new(cursor: mongodb::Cursor<RawDocumentBuf>, runtime: RuntimeT) -> Self {
        Self {
            state: Arc::new(AsyncMutex::new(CursorState::Plain(cursor))),
            runtime,
        }
    }

    pub(crate) fn new_with_session(
        cursor: mongodb::SessionCursor<RawDocumentBuf>,
        session: ClientSessionT,
        runtime: RuntimeT,
    ) -> Self {
        Self {
            state: Arc::new(AsyncMutex::new(CursorState::Session { cursor, session })),
            runtime,
        }
    }

    fn next_async(&self) -> FutureT {
        let state = self.state.clone();

        spawn!(self, Bool, async move {
            state.lock().await.advance().await.map_err(ErrorT::from)
        })
    }

    fn next(&self) -> Result<bool, mongodb::error::Error> {
        self.runtime
            .block_on(async { self.state.lock().await.advance().await })
    }

    fn current_async(&self) -> FutureT {
        let state = self.state.clone();

        spawn!(self, Bson, async move {
            Ok(state.lock().await.current().to_owned())
        })
    }

    fn current(&self) -> BsonViewT {
        self.runtime
            .block_on(async { self.state.lock().await.current().into() })
    }
}

enum CursorState {
    Plain(mongodb::Cursor<RawDocumentBuf>),
    Session {
        cursor: mongodb::SessionCursor<RawDocumentBuf>,
        session: ClientSessionT,
    },
}

impl CursorState {
    async fn advance(&mut self) -> Result<bool, mongodb::error::Error> {
        match self {
            CursorState::Plain(c) => c.advance().await,
            CursorState::Session { cursor, session } => {
                let mut guard = session.state().lock().await;
                cursor.advance(&mut guard).await
            }
        }
    }

    // Precondition: `advance()` returned `true`.
    fn current(&self) -> &RawDocument {
        match self {
            CursorState::Plain(cursor) => cursor.current(),
            CursorState::Session { cursor, .. } => cursor.current(),
        }
    }
}

#[macro_export]
macro_rules! cursor_op_with_session {
    ($op:expr, $session:expr, $runtime:expr) => {{
        let op = $op;
        let session = $session;
        let runtime = $runtime;
        match session {
            Some(session) => {
                let mut guard = session.state().lock().await;
                op.session(&mut *guard)
                    .await
                    .map_err(ErrorT::from)
                    .map(|cursor| {
                        $crate::cursor::CursorT::new_with_session(
                            cursor.with_type(),
                            session.clone(),
                            runtime,
                        )
                    })
            }
            None => op
                .await
                .map_err(ErrorT::from)
                .map(|cursor| $crate::cursor::CursorT::new(cursor.with_type(), runtime)),
        }
    }};
}
