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
use std::task::Context;

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

    pub(crate) fn from_runtime(&self, runtime: &RuntimeT) -> bool {
        runtime == &self.runtime
    }

    fn is_ready(&self) -> bool {
        self.value.is_ready()
    }

    pub(crate) fn poll_with_context(&mut self, ctx: &mut Context<'_>) -> bool {
        self.value.poll_with_context(ctx)
    }

    fn poll(&mut self) -> bool {
        self.value.poll()
    }

    fn wait(&mut self) {
        if self.value.is_ready() {
            return;
        }

        let completion = CompletionWaker::new();
        let waker = completion.waker();
        let mut ctx = Context::from_waker(&waker);

        loop {
            if self.value.poll_with_context(&mut ctx) {
                return;
            }
            completion.wait(None);
        }
    }

    fn wait_with_timeout(&mut self, timeout: Duration) -> bool {
        if self.value.is_ready() {
            return true;
        }

        let deadline = Instant::now() + timeout;
        let completion = CompletionWaker::new();
        let waker = completion.waker();
        let mut ctx = Context::from_waker(&waker);

        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return false;
            }

            if self.value.poll_with_context(&mut ctx) {
                return true;
            }

            completion.wait(Some(remaining));
        }
    }

    pub(crate) fn block_on(&mut self) {
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
    pub(crate) fn is_ready(&self) -> bool {
        future_value_op!(&*self, v => v.is_ready())
    }

    fn poll(&mut self) -> bool {
        future_value_op!(&mut *self, v => v.poll())
    }

    fn poll_with_context(&mut self, ctx: &mut std::task::Context<'_>) -> bool {
        future_value_op!(&mut *self, v => v.poll_with_context(ctx))
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

    #[test]
    fn block_on_long_sleep_parks_until_ready() {
        let runtime = make_runtime();
        let mut future = spawn_delayed(&runtime, Duration::from_millis(500));
        let start = Instant::now();
        future.block_on();
        let elapsed = start.elapsed();

        assert!(
            future.value.is_ready(),
            "future should be ready after block_on"
        );
        assert!(
            elapsed >= Duration::from_millis(400) && elapsed < Duration::from_millis(1500),
            "block_on should park until the sleep fires, not spin or return early (elapsed: {elapsed:?})"
        );
    }

    #[test]
    fn runtime_block_on_any_does_not_take_ownership() {
        let runtime = make_runtime();
        let mut f1 = spawn_delayed(&runtime, Duration::from_millis(50));
        let mut f2 = spawn_delayed(&runtime, Duration::from_millis(500));

        let mut futures = [Some(&mut f1 as &mut FutureT), Some(&mut f2 as &mut FutureT)];
        let start = Instant::now();
        // Both spawned futures are short, so at least one will become ready; None is impossible here.
        let future = runtime
            .block_on_any(&mut futures)
            .expect("one future should be ready");
        let elapsed = start.elapsed();

        assert!(std::ptr::eq(future, &mut f1));
        assert!(
            elapsed >= Duration::from_millis(40) && elapsed < Duration::from_millis(150),
            "block_on_any should return as soon as the short task fires (elapsed: {elapsed:?})"
        );

        // The futures are borrowed, not consumed. The ready one can be queried and the
        // other one can still be driven to completion.
        assert!(f1.value.is_ready());
        assert!(!f2.value.is_ready());

        f2.block_on();
        assert!(f2.value.is_ready());
    }

    #[test]
    fn runtime_block_on_all_waits_for_all_futures() {
        let runtime = make_runtime();
        let mut f1 = spawn_delayed(&runtime, Duration::from_millis(50));
        let mut f2 = spawn_delayed(&runtime, Duration::from_millis(200));

        let mut futures = [Some(&mut f1 as &mut FutureT), Some(&mut f2 as &mut FutureT)];
        let start = Instant::now();
        runtime.block_on_all(&mut futures);
        let elapsed = start.elapsed();

        // block_on_all should wait for the slowest task, not the fastest.
        assert!(
            elapsed >= Duration::from_millis(180) && elapsed < Duration::from_millis(400),
            "block_on_all should wait for the slowest task (elapsed: {elapsed:?})"
        );

        // Both futures are borrowed, not consumed, and are now ready.
        assert!(f1.value.is_ready());
        assert!(f2.value.is_ready());
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
