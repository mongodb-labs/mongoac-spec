use crate::error::ErrorT;
use crate::future::FutureT;
use crate::private::bson::{BsonT, bson_t};
use crate::private::macros::*;
use crate::runtime::RuntimeT;
use crate::spawn;

use mongodb::ClientSession;
use mongodb::bson::{RawDocument, RawDocumentBuf};
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

enum InnerCursor {
    Plain(mongodb::Cursor<RawDocumentBuf>),
    Session {
        cursor: mongodb::SessionCursor<RawDocumentBuf>,
        session: Arc<AsyncMutex<ClientSession>>,
    },
}

impl InnerCursor {
    async fn advance(&mut self) -> Result<bool, mongodb::error::Error> {
        match self {
            InnerCursor::Plain(c) => c.advance().await,
            InnerCursor::Session { cursor, session } => {
                let mut guard = session.lock().await;
                cursor.advance(&mut guard).await
            }
        }
    }

    fn current(&self) -> &RawDocument {
        match self {
            InnerCursor::Plain(c) => c.current(),
            InnerCursor::Session { cursor, .. } => cursor.current(),
        }
    }
}

#[derive(Clone)]
pub struct CursorT {
    inner: Arc<AsyncMutex<InnerCursor>>,
    runtime: RuntimeT,
}

impl CursorT {
    pub(crate) fn new(cursor: mongodb::Cursor<RawDocumentBuf>, runtime: RuntimeT) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(InnerCursor::Plain(cursor))),
            runtime,
        }
    }

    pub(crate) fn new_with_session(
        cursor: mongodb::SessionCursor<RawDocumentBuf>,
        session: Arc<AsyncMutex<ClientSession>>,
        runtime: RuntimeT,
    ) -> Self {
        Self {
            inner: Arc::new(AsyncMutex::new(InnerCursor::Session { cursor, session })),
            runtime,
        }
    }

    fn next_sync(&self) -> Result<bool, mongodb::error::Error> {
        self.runtime
            .block_on(async { self.inner.lock().await.advance().await })
    }

    fn next_async(&self) -> FutureT {
        let inner = self.inner.clone();
        spawn!(
            self,
            Bool,
            async move { inner.lock().await.advance().await }
        )
    }

    fn get_document_bson(&self) -> RawDocumentBuf {
        self.runtime.block_on(async {
            let cursor = self.inner.lock().await;
            cursor.current().to_owned()
        })
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_destroy(cursor: *mut CursorT) {
    safe_drop!(cursor);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_next(cursor: *mut CursorT, error: *mut ErrorT) -> bool {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_mut_with_error!(cursor, error);

    safe_error!(cursor.next_sync(), error)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_next_async(
    cursor: *mut CursorT,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    let future = cursor.next_async();
    Box::into_raw(Box::new(future))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_cursor_get_document(
    cursor: *const CursorT,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let cursor = safe_as_ref_with_error!(cursor, error);

    safe_error!(BsonT::try_from(&cursor.get_document_bson()), error).into()
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
