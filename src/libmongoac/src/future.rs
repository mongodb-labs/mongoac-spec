use crate::client_session::ClientSessionT;
use crate::cursor::CursorT;
use crate::error::{ErrorCodeT, ErrorT};
use crate::private::bson::{BsonT, bson_t};
use crate::private::macros::*;
use crate::runtime::RuntimeT;

use mongodb::bson::RawDocumentBuf;
use parking_lot::Mutex;

use std::future::{Future, poll_fn};
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::task::{Context, Poll};

#[derive(Clone)]
pub struct FutureT {
    runtime: RuntimeT,
    value: Arc<FutureValue>,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_destroy(future: *mut FutureT) {
    safe_drop!(future);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_clone(future: *const FutureT) -> *mut FutureT {
    Box::into_raw(Box::new(safe_as_ref!(future).clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_is_ready(future: *const FutureT) -> bool {
    safe_as_ref!(future).is_ready()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_int32(future: *const FutureT, error: *mut ErrorT) -> i32 {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    *safe_error!(future.get_int32(), error)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_bson(
    future: *const FutureT,
    error: *mut ErrorT,
) -> *mut bson_t {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    let doc = safe_error!(future.get_bson(), error);
    safe_error!(BsonT::try_from(doc), error).into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_future_get_void(future: *const FutureT, error: *mut ErrorT) {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_as_ref_with_error!(future, error);

    safe_error!(future.get_void(), error);
}

pub enum FutureValue {
    Bool(FutureValueType<bool>),
    Bson(FutureValueType<RawDocumentBuf>),
    ClientSession(FutureValueType<ClientSessionT>),
    Cursor(FutureValueType<CursorT>),
    Int32(FutureValueType<i32>),
    Void(FutureValueType<()>),
}

macro_rules! future_value_op {
    ($value:expr, $v:ident => $e:expr) => {
        match &*$value {
            FutureValue::Bool($v) => $e,
            FutureValue::Bson($v) => $e,
            FutureValue::ClientSession($v) => $e,
            FutureValue::Cursor($v) => $e,
            FutureValue::Int32($v) => $e,
            FutureValue::Void($v) => $e,
        }
    };
}

macro_rules! future_value_result {
    ($self:expr, $variant:ident, $name:literal) => {
        match &*$self.value {
            FutureValue::$variant(fvt) => fvt.result(),
            _ => Err(ErrorT::from_mongoac(
                ErrorCodeT::RuntimeError,
                concat!("future does not return a ", $name),
            )),
        }
    };
}

impl FutureT {
    pub fn new(runtime: RuntimeT, value: FutureValue) -> Self {
        Self {
            runtime,
            value: Arc::new(value),
        }
    }

    pub fn get_runtime(&self) -> &RuntimeT {
        &self.runtime
    }

    pub fn is_ready(&self) -> bool {
        future_value_op!(self.value, v => v.is_ready())
    }

    pub fn get_int32(&self) -> Result<&i32, ErrorT> {
        future_value_result!(self, Int32, "int32")
    }

    pub fn get_bool(&self) -> Result<&bool, ErrorT> {
        future_value_result!(self, Bool, "bool")
    }

    pub fn get_bson(&self) -> Result<&RawDocumentBuf, ErrorT> {
        future_value_result!(self, Bson, "bson")
    }

    pub fn get_client_session(&self) -> Result<&ClientSessionT, ErrorT> {
        future_value_result!(self, ClientSession, "client session")
    }

    pub fn get_void(&self) -> Result<&(), ErrorT> {
        future_value_result!(self, Void, "void")
    }

    pub fn get_cursor(&self) -> Result<&CursorT, ErrorT> {
        future_value_result!(self, Cursor, "cursor")
    }

    pub fn poll_with_context(&self, ctx: &mut Context<'_>) -> bool {
        future_value_op!(self.value, v => v.poll_with_context(ctx))
    }

    pub fn poll(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(|ctx| {
            if self.poll_with_context(ctx) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        })
    }
}

/// The dynamic type of an `async` block which returns a `Result<T, ErrorT>`.
type Async<T> = Pin<Box<dyn Future<Output = Result<T, ErrorT>> + Send>>;

pub struct FutureValueType<T> {
    result: OnceLock<Result<T, ErrorT>>,
    handle: Mutex<Option<Async<T>>>,
}

impl<T: Send + 'static> FutureValueType<T> {
    pub fn new(handle: tokio::task::JoinHandle<Result<T, ErrorT>>) -> Self {
        Self {
            result: OnceLock::new(),
            handle: Mutex::new(Some(Box::pin(async move {
                match handle.await {
                    Ok(result) => result,
                    Err(err) => Err(ErrorT::from_mongoac(
                        ErrorCodeT::RuntimeError,
                        &format!("tokio::task::JoinError: {err}"),
                    )),
                }
            }))),
        }
    }

    pub fn is_ready(&self) -> bool {
        self.result.get().is_some()
    }

    pub fn result(&self) -> Result<&T, ErrorT> {
        match self.result.get() {
            Some(Ok(val)) => Ok(val),
            Some(Err(err)) => Err(err.clone()),
            None => Err(ErrorT::from_mongoac(
                ErrorCodeT::RuntimeError,
                "future is not ready",
            )),
        }
    }

    pub fn poll_with_context(&self, ctx: &mut Context<'_>) -> bool {
        if self.is_ready() {
            return true;
        }

        let mut guard = self.handle.lock();
        let Some(handle) = guard.as_mut() else {
            return self.is_ready();
        };

        match handle.as_mut().poll(ctx) {
            Poll::Ready(val) => {
                let _ = self.result.set(val);
                *guard = None;
                true
            }
            Poll::Pending => false,
        }
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

#[derive(Clone, Copy)]
pub struct FutureExt<'a> {
    pub future: &'a FutureT,
    pub index: usize,
}

impl<'a> FutureExt<'a> {
    pub fn new(future: &'a FutureT) -> Self {
        Self::new_with_index(future, 0)
    }

    pub fn new_with_index(future: &'a FutureT, index: usize) -> Self {
        Self { future, index }
    }
}

impl<'a> Future for FutureExt<'a> {
    type Output = usize;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        if this.future.poll_with_context(cx) {
            Poll::Ready(this.index)
        } else {
            Poll::Pending
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::private::test_util::make_runtime;
    use crate::runtime::RuntimeT;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier};
    use std::thread;
    use std::time::{Duration, Instant};
    use tokio::sync::Notify;

    fn spawn_immediate(runtime: &RuntimeT) -> FutureT {
        let handle = runtime.spawn(async move { Ok(()) });
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(handle)),
        )
    }

    fn spawn_delayed(runtime: &RuntimeT, delay: Duration) -> FutureT {
        let handle = runtime.spawn(async move { Ok(tokio::time::sleep(delay).await) });
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(handle)),
        )
    }

    fn spawn_notified(runtime: &RuntimeT, notify: Arc<Notify>) -> FutureT {
        let handle = runtime.spawn(async move { Ok(notify.notified().await) });
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(handle)),
        )
    }

    // -- block_on tests (via RuntimeT) --

    #[test]
    fn block_on_immediate() {
        let runtime = make_runtime();
        let future = spawn_immediate(&runtime);
        runtime.block_on_future(&future);
        assert!(future.is_ready());
    }

    #[test]
    fn block_on_delayed() {
        let runtime = make_runtime();
        let future = spawn_delayed(&runtime, Duration::from_millis(10));
        runtime.block_on_future(&future);
        assert!(future.is_ready());
    }

    #[test]
    fn block_on_already_ready() {
        let runtime = make_runtime();
        let future = spawn_immediate(&runtime);
        runtime.block_on_future(&future);
        runtime.block_on_future(&future);
        assert!(future.is_ready());
    }

    #[test]
    fn block_on_long_sleep_parks_until_ready() {
        let runtime = make_runtime();
        let future = spawn_delayed(&runtime, Duration::from_millis(500));
        let start = Instant::now();
        runtime.block_on_future(&future);
        let elapsed = start.elapsed();

        assert!(future.is_ready(), "future should be ready after block_on");
        assert!(
            elapsed >= Duration::from_millis(400) && elapsed < Duration::from_millis(1500),
            "block_on should park until the sleep fires, not spin or return early (elapsed: {elapsed:?})"
        );
    }

    #[test]
    fn runtime_block_on_any_does_not_take_ownership() {
        let runtime = make_runtime();
        let notify1 = Arc::new(Notify::new());
        let notify2 = Arc::new(Notify::new());
        let f1 = spawn_notified(&runtime, notify1.clone());
        let f2 = spawn_notified(&runtime, notify2.clone());

        // Synchronize with a notifier thread so that f1 becomes ready only after
        // block_on_any has begun. The elapsed-time assertion is replaced by this
        // explicit barrier/flag coordination, which is reliable under Miri.
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        thread::spawn(move || {
            barrier_clone.wait();
            notify1.notify_one();
        });

        let futures = [(0, &f1), (1, &f2)];
        barrier.wait();
        let index = runtime
            .block_on_any(&futures)
            .expect("one future should be ready");

        assert_eq!(index, 0);

        // The futures are borrowed, not consumed. The ready one can be queried and the
        // other one can still be driven to completion.
        assert!(f1.is_ready());
        assert!(!f2.is_ready());

        notify2.notify_one();
        runtime.block_on_future(&f2);
        assert!(f2.is_ready());
    }

    #[test]
    fn runtime_block_on_all_waits_for_all_futures() {
        let runtime = make_runtime();
        let f1 = spawn_delayed(&runtime, Duration::from_millis(50));
        let f2 = spawn_delayed(&runtime, Duration::from_millis(200));

        let futures = [&f1, &f2];
        let start = Instant::now();
        runtime.block_on_all(&futures);
        let elapsed = start.elapsed();

        // block_on_all should wait for the slowest task, not the fastest.
        assert!(
            elapsed >= Duration::from_millis(180) && elapsed < Duration::from_millis(400),
            "block_on_all should wait for the slowest task (elapsed: {elapsed:?})"
        );

        // Both futures are borrowed, not consumed, and are now ready.
        assert!(f1.is_ready());
        assert!(f2.is_ready());
    }

    // -- block_on*_with_timeout tests (via RuntimeT) --

    #[test]
    fn block_on_future_with_timeout_completes_before_deadline() {
        let runtime = make_runtime();
        let future = spawn_delayed(&runtime, Duration::from_millis(10));

        let result = runtime.block_on_future_with_timeout(&future, Duration::from_millis(500));

        assert!(result.is_ok(), "future should complete before timeout");
        assert!(future.is_ready());
    }

    #[test]
    fn block_on_future_with_timeout_times_out() {
        let runtime = make_runtime();
        let future = spawn_delayed(&runtime, Duration::from_millis(500));

        let result = runtime.block_on_future_with_timeout(&future, Duration::from_millis(10));

        assert!(
            result.is_err(),
            "should time out before the future completes"
        );
        assert!(
            !future.is_ready(),
            "future should not be ready after timeout"
        );
    }

    #[test]
    fn block_on_any_with_timeout_returns_first_ready_future() {
        let runtime = make_runtime();
        let notify1 = Arc::new(Notify::new());
        let notify2 = Arc::new(Notify::new());
        let f1 = spawn_notified(&runtime, notify1.clone());
        let f2 = spawn_notified(&runtime, notify2);

        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        thread::spawn(move || {
            barrier_clone.wait();
            notify1.notify_one();
        });

        let futures = [(0, &f1), (1, &f2)];
        barrier.wait();
        let result = runtime.block_on_any_with_timeout(&futures, Duration::from_millis(200));

        let index = result
            .expect("should not time out")
            .expect("one future should be ready");
        assert_eq!(index, 0);

        assert!(f1.is_ready());
        assert!(!f2.is_ready());
    }

    #[test]
    fn block_on_any_with_timeout_times_out() {
        let runtime = make_runtime();
        let f1 = spawn_delayed(&runtime, Duration::from_millis(500));
        let f2 = spawn_delayed(&runtime, Duration::from_millis(600));

        let futures = [(0, &f1), (1, &f2)];
        let result = runtime.block_on_any_with_timeout(&futures, Duration::from_millis(10));

        assert!(
            result.is_err(),
            "should time out before either future completes"
        );
        assert!(!f1.is_ready());
        assert!(!f2.is_ready());
    }

    #[test]
    fn block_on_all_with_timeout_completes_before_deadline() {
        let runtime = make_runtime();
        let f1 = spawn_delayed(&runtime, Duration::from_millis(50));
        let f2 = spawn_delayed(&runtime, Duration::from_millis(100));

        let futures = [&f1, &f2];
        let result = runtime.block_on_all_with_timeout(&futures, Duration::from_millis(500));

        assert!(result.is_ok(), "all futures should complete before timeout");
        assert!(f1.is_ready());
        assert!(f2.is_ready());
    }

    #[test]
    fn block_on_all_with_timeout_times_out() {
        let runtime = make_runtime();
        let f1 = spawn_delayed(&runtime, Duration::from_millis(500));
        let f2 = spawn_delayed(&runtime, Duration::from_millis(600));

        let futures = [&f1, &f2];
        let result = runtime.block_on_all_with_timeout(&futures, Duration::from_millis(10));

        assert!(
            result.is_err(),
            "should time out before all futures complete"
        );
        assert!(!f1.is_ready());
        assert!(!f2.is_ready());
    }

    // -- clone tests --

    #[test]
    fn clone_shares_ready_state() {
        let runtime = make_runtime();
        let future = spawn_immediate(&runtime);
        runtime.block_on_future(&future);
        assert!(future.is_ready());

        let clone = future.clone();
        assert!(clone.is_ready());
    }

    #[test]
    fn clone_outlives_original() {
        let runtime = make_runtime();
        let future = spawn_immediate(&runtime);
        runtime.block_on_future(&future);

        let clone = future.clone();
        drop(future);
        assert!(clone.is_ready());
    }

    // -- getter tests --

    #[test]
    fn get_int32_ready() {
        let runtime = make_runtime();
        let handle = runtime.spawn(async move { Ok(42) });
        let future = FutureT::new(
            runtime.clone(),
            FutureValue::Int32(FutureValueType::new(handle)),
        );
        runtime.block_on_future(&future);
        assert_eq!(
            *future
                .get_int32()
                .unwrap_or_else(|_| panic!("int32 getter should succeed")),
            42
        );
    }

    #[test]
    fn get_int32_not_ready() {
        let runtime = make_runtime();
        let handle = runtime.spawn(async move {
            tokio::time::sleep(Duration::from_secs(10)).await;
            Ok(42)
        });
        let future = FutureT::new(
            runtime.clone(),
            FutureValue::Int32(FutureValueType::new(handle)),
        );
        let err = future.get_int32().unwrap_err();
        assert_eq!(err.code(), ErrorCodeT::RuntimeError);
    }

    // -- is_ready / poll_with_context tests --

    #[test]
    fn is_ready_after_block_on() {
        let runtime = make_runtime();
        let future = spawn_immediate(&runtime);
        assert!(!future.is_ready());
        runtime.block_on_future(&future);
        assert!(future.is_ready());
    }

    #[test]
    fn poll_with_context_eventually_resolves() {
        let runtime = make_runtime();
        let future = spawn_delayed(&runtime, Duration::from_millis(10));
        let start = Instant::now();
        while !future.poll_with_context(&mut Context::from_waker(std::task::Waker::noop())) {
            if start.elapsed() > Duration::from_secs(5) {
                panic!("future did not resolve");
            }
            runtime.make_progress();
        }
        assert!(future.is_ready());
    }

    #[test]
    fn make_progress_does_not_alone_resolve_future() {
        // A future with a result is only resolved when explicitly driven by
        // RuntimeT::block_on*. make_progress alone advances the runtime but does
        // not poll the future's JoinHandle.
        let runtime = make_runtime();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        let future = FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(runtime.spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                done_clone.store(true, Ordering::Release);
                Ok(())
            }))),
        );

        // Drive the runtime for up to 200ms. The spawned task should complete,
        // but the future's JoinHandle is not polled.
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(200) {
            runtime.make_progress();
        }
        assert!(done.load(Ordering::Acquire), "spawned task should have run");
        assert!(
            !future.is_ready(),
            "future should not be ready without block_on*"
        );

        // Now drive the future explicitly.
        runtime.block_on_future(&future);
        assert!(future.is_ready());
    }
}
