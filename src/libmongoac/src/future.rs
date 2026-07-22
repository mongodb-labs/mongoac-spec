use crate::client_session::ClientSessionT;
use crate::cursor::CursorT;
use crate::error::{ErrorCodeT, ErrorT};
use crate::private::bson::{BsonT, bson_t};
use crate::private::completion::CompletionWaker;
use crate::runtime::RuntimeT;
use crate::{
    safe_as_mut, safe_as_ref, safe_as_ref_with_error, safe_drop, safe_error,
    safe_optional_error_as_mut,
};

use std::future::Future;
use std::pin::Pin;

use std::time::{Duration, Instant};

pub struct FutureT {
    runtime: RuntimeT,
    value: FutureValue,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_destroy(future: *mut FutureT) {
    safe_drop!(future);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_is_ready(future: *const FutureT) -> bool {
    safe_as_ref!(future).is_ready()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_poll(future: *mut FutureT) -> bool {
    safe_as_mut!(future).poll()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_wait(future: *mut FutureT) {
    safe_as_mut!(future).wait();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_wait_with_timeout(future: *mut FutureT, timeout_ms: u64) -> bool {
    safe_as_mut!(future).wait_with_timeout(Duration::from_millis(timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_block_on(future: *mut FutureT) {
    safe_as_mut!(future).block_on();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_int32(future: *const FutureT, error: *mut ErrorT) -> i32 {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    *safe_error!(future.value().get_int32(), error)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_bson(
    future: *const FutureT,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    let doc = safe_error!(future.value().get_bson(), error);
    safe_error!(BsonT::try_from(doc), error).into()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_void(future: *const FutureT, error: *mut ErrorT) {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    safe_error!(future.value().get_void(), error);
}

impl FutureT {
    pub(crate) fn new(runtime: RuntimeT, value: FutureValue) -> Self {
        Self { runtime, value }
    }

    pub(crate) fn value(&self) -> &FutureValue {
        &self.value
    }

    fn is_ready(&self) -> bool {
        self.value.is_ready()
    }

    fn poll(&mut self) -> bool {
        self.value.poll()
    }

    fn wait(&mut self) {
        self.value.wait()
    }

    fn wait_with_timeout(&mut self, timeout: Duration) -> bool {
        self.value.wait_with_timeout(timeout)
    }

    fn block_on(&mut self) {
        self.value.block_on(&self.runtime);
    }
}

#[macro_export]
macro_rules! spawn {
    ($client:expr, $value_type:ident, $op:expr) => {{
        let rt = $client.runtime.clone();
        let handle = rt.spawn($op);
        $crate::future::FutureT::new(
            rt,
            $crate::future::FutureValue::$value_type($crate::future::FutureValueType::new(handle)),
        )
    }};
}

pub(crate) enum FutureValue {
    Bool(FutureValueType<bool>),
    Bson(FutureValueType<mongodb::bson::RawDocumentBuf>),
    ClientSession(FutureValueType<ClientSessionT>),
    Cursor(FutureValueType<CursorT>),
    Int32(FutureValueType<i32>),
    Void(FutureValueType<()>),
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
impl FutureValue {
    fn is_ready(&self) -> bool {
        future_value_op!(&*self, v => v.is_ready())
    }

    fn poll(&mut self) -> bool {
        future_value_op!(&mut *self, v => v.poll())
    }

    fn wait(&mut self) {
        future_value_op!(&mut *self, v => v.wait())
    }

    fn wait_with_timeout(&mut self, timeout: Duration) -> bool {
        future_value_op!(&mut *self, v => v.wait_with_timeout(timeout))
    }

    fn block_on(&mut self, runtime: &RuntimeT) {
        future_value_op!(&mut *self, v => v.block_on(runtime))
    }

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

// The dynamic type of an `async` block which returns a `Result<T, ErrorT>`.
type Async<T> = Pin<Box<dyn Future<Output = Result<T, ErrorT>> + Send>>;

pub(crate) enum FutureValueType<T> {
    Result(Result<T, ErrorT>),
    Future(Async<T>),
}

impl<T: Send + 'static> FutureValueType<T> {
    pub(crate) fn new<E: Into<ErrorT> + Send + 'static>(
        handle: tokio::task::JoinHandle<Result<T, E>>,
    ) -> Self {
        FutureValueType::Future(Box::pin(async move {
            match handle.await {
                std::result::Result::Ok(Ok(val)) => Ok(val),
                std::result::Result::Ok(Err(err)) => Err(err.into()),
                std::result::Result::Err(err) => Err(ErrorT::from_mongoac(
                    ErrorCodeT::RuntimeError,
                    &format!("tokio::task::JoinError: {err}"),
                )),
            }
        }))
    }

    fn is_ready(&self) -> bool {
        match self {
            FutureValueType::Result(_) => true,
            FutureValueType::Future(_) => false,
        }
    }

    pub(crate) fn result(&self) -> Result<&T, ErrorT> {
        match self {
            FutureValueType::Result(result) => match result {
                Ok(val) => Ok(val),
                Err(err) => Err(err.clone()),
            },
            FutureValueType::Future(_) => Err(ErrorT::from_mongoac(
                ErrorCodeT::RuntimeError,
                "future is not ready",
            )),
        }
    }

    /// Non-blocking poll.  Returns `true` if the future has resolved.
    ///
    /// Under the logical const-correctness contract, `*mut` operations on the
    /// same `FutureT` are not safe to call concurrently, so no external
    /// synchronization is needed here.
    fn poll(&mut self) -> bool {
        match self {
            FutureValueType::Result(_) => true,
            FutureValueType::Future(future) => {
                let mut ctx = std::task::Context::from_waker(std::task::Waker::noop());
                match future.as_mut().poll(&mut ctx) {
                    std::task::Poll::Ready(val) => {
                        *self = FutureValueType::Result(val);
                        true
                    }
                    std::task::Poll::Pending => false,
                }
            }
        }
    }

    /// Poll with a caller-provided waker, used by `wait()` / `wait_with_timeout()`.
    ///
    /// Registers `ctx`'s waker with the inner future chain so the condvar-based
    /// wait loops work correctly.
    fn poll_with_context(&mut self, ctx: &mut std::task::Context<'_>) -> bool {
        match self {
            FutureValueType::Result(_) => true,
            FutureValueType::Future(future) => match future.as_mut().poll(ctx) {
                std::task::Poll::Ready(val) => {
                    *self = FutureValueType::Result(val);
                    true
                }
                std::task::Poll::Pending => false,
            },
        }
    }

    /// Blocking wait using a condvar-backed waker.
    ///
    /// Parks the thread via `CompletionWaker` until a progress thread drives
    /// the runtime and the future resolves.  Returns only when ready.
    fn wait(&mut self) {
        match self {
            FutureValueType::Result(_) => {}
            FutureValueType::Future(_) => {
                let completion = CompletionWaker::new();
                let waker = completion.waker();
                let mut ctx = std::task::Context::from_waker(&waker);

                loop {
                    if self.poll_with_context(&mut ctx) {
                        return;
                    }
                    completion.wait(None);
                }
            }
        }
    }

    /// Blocking wait with a wall-clock timeout.
    ///
    /// Parks the thread via `CompletionWaker` until ready or the deadline
    /// expires.  Returns `true` if the future completed, `false` if the
    /// timeout expired first.
    fn wait_with_timeout(&mut self, timeout: Duration) -> bool {
        match self {
            FutureValueType::Result(_) => true,
            FutureValueType::Future(_) => {
                let deadline = Instant::now() + timeout;
                let completion = CompletionWaker::new();
                let waker = completion.waker();
                let mut ctx = std::task::Context::from_waker(&waker);

                loop {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return false;
                    }

                    if self.poll_with_context(&mut ctx) {
                        return true;
                    }

                    completion.wait(Some(remaining));
                }
            }
        }
    }

    /// Blocking resolution by entering the Tokio runtime.
    ///
    /// Extracts the inner future and calls `runtime.block_on()` on it.
    fn block_on(&mut self, runtime: &RuntimeT) {
        match self {
            FutureValueType::Result(_) => {}
            FutureValueType::Future(future) => {
                *self = FutureValueType::Result(runtime.block_on(future))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::RuntimeT;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Once};
    use std::thread;
    use std::time::Duration;

    static INIT: Once = Once::new();

    fn make_runtime() -> RuntimeT {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .event_interval(1)
            .build()
            .expect("failed to build runtime");
        RuntimeT::from_raw(rt)
    }

    fn spawn_immediate(runtime: &RuntimeT) -> FutureT {
        let handle = runtime.spawn(async { Ok::<(), ErrorT>(()) });
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(handle)),
        )
    }

    fn spawn_delayed(runtime: &RuntimeT, delay: Duration) -> FutureT {
        let handle = runtime.spawn(async move {
            tokio::time::sleep(delay).await;
            Ok::<(), ErrorT>(())
        });
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(handle)),
        )
    }

    /// Runs `op` while a background thread repeatedly calls `make_progress()`
    /// on a clone of `runtime`. This keeps the single-thread Tokio runtime
    /// advancing so that `wait()` / `wait_with_timeout()` tests can resolve
    /// without entering the runtime directly.
    fn run_with_progress_thread<F, R>(runtime: &RuntimeT, op: F) -> R
    where
        F: FnOnce() -> R + Send,
        R: Send,
    {
        let rt = runtime.clone();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();
        let started = Arc::new(AtomicBool::new(false));
        let started_clone = started.clone();

        let progress_thread = thread::spawn(move || {
            while !started_clone.load(Ordering::Acquire) {
                thread::yield_now();
            }
            while !done_clone.load(Ordering::Acquire) {
                rt.make_progress();
                thread::yield_now();
            }
        });

        started.store(true, Ordering::Release);
        let result = op();
        done.store(true, Ordering::Release);
        progress_thread.join().unwrap();
        result
    }

    // -- block_on tests --

    #[test]
    fn block_on_immediate() {
        let runtime = make_runtime();
        let mut future = spawn_immediate(&runtime);
        future.block_on();
        assert!(future.value.is_ready());
    }

    #[test]
    fn block_on_delayed() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_millis(10));
        future.block_on();
        assert!(future.value.is_ready());
    }

    #[test]
    fn block_on_already_ready() {
        let runtime = make_runtime();
        let mut future = spawn_immediate(&runtime);
        future.block_on();
        future.block_on();
        assert!(future.value.is_ready());
    }

    #[test]
    fn block_on_holds_progress_lock() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_millis(20));
        let rt = runtime.clone();
        let blocker = thread::spawn(move || {
            assert!(!rt.make_progress());
        });
        future.block_on();
        blocker.join().unwrap();
    }

    // -- wait tests --

    #[test]
    fn wait_with_progress_thread() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_millis(50));

        run_with_progress_thread(&runtime, || {
            future.wait();
            assert!(future.value.is_ready());
        });
    }

    #[test]
    fn wait_already_ready() {
        let runtime = make_runtime();
        let mut future = spawn_immediate(&runtime);
        while !future.value.poll() {
            runtime.make_progress();
        }
        future.wait();
        assert!(future.value.is_ready());
    }

    // -- wait_with_timeout tests --

    #[test]
    fn wait_with_timeout_already_ready() {
        let runtime = make_runtime();
        let mut future = spawn_immediate(&runtime);
        while !future.value.poll() {
            runtime.make_progress();
        }
        assert!(future.wait_with_timeout(Duration::from_millis(0)));
    }

    #[test]
    fn wait_with_timeout_expires() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_secs(60));
        assert!(!future.wait_with_timeout(Duration::from_millis(1)));
    }

    #[test]
    fn wait_with_timeout_with_progress() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_millis(50));

        let completed = run_with_progress_thread(&runtime, || {
            future.wait_with_timeout(Duration::from_secs(5))
        });
        assert!(completed);
    }
}
