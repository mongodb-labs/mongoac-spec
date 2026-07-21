use crate::client_session::ClientSessionT;
use crate::cursor::CursorT;
use crate::runtime::RuntimeT;
use crate::{
    safe_as_mut, safe_as_ref_with_error, safe_drop, safe_error, safe_optional_error_as_mut,
};

use crate::error::{ErrorCodeT, ErrorT};
use crate::private::bson::{BsonT, bson_t};

use async_ffi::FfiFuture;

use std::future::Future;
use std::pin::pin;
use std::sync::atomic::{AtomicBool, Ordering};

#[macro_export]
macro_rules! spawn {
    ($client:expr, $value_type:ident, $op:expr) => {{
        let rt = $client.runtime.clone();
        let handle = rt.spawn($op);

        let future = async move {
            match handle.await {
                std::result::Result::Ok(res) => res,
                std::result::Result::Err(e) => Err(ErrorT::from_mongoac(
                    $crate::error::ErrorCodeT::RuntimeError,
                    &format!("tokio::task::JoinError: {e}"),
                )),
            }
        };

        $crate::future::FutureT::new(
            rt,
            $crate::future::FutureValue::$value_type($crate::future::FutureValueType::new(future)),
        )
    }};
}

macro_rules! future_value_op {
    ($value:expr, $v:ident => $e:expr) => {
        match $value {
            FutureValue::Bool($v) => $e,
            FutureValue::Bson($v) => $e,
            FutureValue::Int32($v) => $e,
            FutureValue::ClientSession($v) => $e,
            FutureValue::Void($v) => $e,
            FutureValue::Cursor($v) => $e,
        }
    };
}

trait Pollable {
    fn is_ready(&self) -> bool;
    fn poll(&mut self) -> bool;
}

pub(crate) struct FutureValueType<T> {
    future: FfiFuture<Result<T, ErrorT>>,
    result: Option<Result<T, ErrorT>>,
    ready: AtomicBool,
    polling: AtomicBool,
}

impl<T: Send + 'static> FutureValueType<T> {
    pub(crate) fn new(
        future: impl Future<Output = Result<T, ErrorT>> + Send + 'static,
    ) -> Self {
        Self {
            future: FfiFuture::new(future),
            result: None,
            ready: AtomicBool::new(false),
            polling: AtomicBool::new(false),
        }
    }

    pub(crate) fn result(&self) -> Result<&T, ErrorT> {
        if !self.is_ready() {
            return Err(ErrorT::from_mongoac(
                ErrorCodeT::RuntimeError,
                "future is not ready",
            ));
        }

        // Invariant: `ready == true` -> `Some(result)`.
        match self.result.as_ref().unwrap() {
            Ok(val) => Ok(val),
            Err(err) => Err(err.clone()),
        }
    }
}

impl<T: Send + 'static> Pollable for FutureValueType<T> {
    fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }

    fn poll(&mut self) -> bool {
        if self.is_ready() {
            return true;
        }

        if self
            .polling
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return self.is_ready();
        }

        let is_ready = match pin!(&mut self.future)
            .as_mut()
            .poll(&mut std::task::Context::from_waker(std::task::Waker::noop()))
        {
            std::task::Poll::Ready(val) => {
                self.result = Some(val);
                self.ready.store(true, Ordering::Release);
                true
            }
            std::task::Poll::Pending => false,
        };

        self.polling.store(false, Ordering::Release);

        is_ready
    }
}

pub(crate) enum FutureValue {
    Bool(FutureValueType<bool>),
    Bson(FutureValueType<mongodb::bson::RawDocumentBuf>),
    ClientSession(FutureValueType<ClientSessionT>),
    Cursor(FutureValueType<CursorT>),
    Int32(FutureValueType<i32>),
    Void(FutureValueType<()>),
}

impl Pollable for FutureValue {
    fn poll(&mut self) -> bool {
        future_value_op!(&mut *self, v => v.poll())
    }

    fn is_ready(&self) -> bool {
        future_value_op!(&*self, v => v.is_ready())
    }
}

impl FutureValue {
    pub(crate) fn get_bool(&self) -> Result<&bool, ErrorT> {
        match self {
            Self::Bool(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched bool getter on non-bool future",
            )
            .into()),
        }
    }

    pub(crate) fn get_int32(&self) -> Result<&i32, ErrorT> {
        match self {
            Self::Int32(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched int32 getter on non-int32 future",
            )
            .into()),
        }
    }

    pub(crate) fn get_bson(&self) -> Result<&mongodb::bson::RawDocumentBuf, ErrorT> {
        match self {
            Self::Bson(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched bson getter on non-bson future",
            )
            .into()),
        }
    }

    pub(crate) fn get_client_session(&self) -> Result<&ClientSessionT, ErrorT> {
        match self {
            Self::ClientSession(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched client session getter on non-client session future",
            )
            .into()),
        }
    }

    pub(crate) fn get_void(&self) -> Result<&(), ErrorT> {
        match self {
            Self::Void(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched void getter on non-void future",
            )
            .into()),
        }
    }

    pub(crate) fn get_cursor(&self) -> Result<&CursorT, ErrorT> {
        match self {
            Self::Cursor(fvt) => fvt.result(),
            _ => Err(mongodb::error::Error::custom(
                "called mismatched cursor getter on non-cursor future",
            )
            .into()),
        }
    }
}

pub struct FutureT {
    pub(crate) runtime: RuntimeT,
    pub(crate) value: FutureValue,
}

impl FutureT {
    pub(crate) fn new(runtime: RuntimeT, value: FutureValue) -> Self {
        Self { runtime, value }
    }

    pub(crate) fn value(&self) -> &FutureValue {
        &self.value
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_poll(future: *mut FutureT) -> bool {
    let f = safe_as_mut!(future);
    f.value.poll()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_int32(future: *mut FutureT, error: *mut ErrorT) -> i32 {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    *safe_error!(future.value().get_int32(), error)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_bson(future: *mut FutureT, error: *mut ErrorT) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    let doc = safe_error!(future.value().get_bson(), error);
    safe_error!(BsonT::try_from(doc), error).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_void(future: *mut FutureT, error: *mut ErrorT) -> bool {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    safe_error!(future.value().get_void(), error);
    true
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_destroy(future: *mut FutureT) {
    safe_drop!(future);
}
