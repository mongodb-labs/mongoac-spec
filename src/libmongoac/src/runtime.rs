use crate::error::{ErrorCodeT, ErrorT};
use crate::future::{FutureExt, FutureT};
use crate::private::macros::*;

use futures_util::stream::{FuturesUnordered, StreamExt};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

#[macro_export]
macro_rules! safe_from_runtime_with_error {
    ($future:expr, $runtime:expr, $error:expr) => {{
        let future = $future;
        let runtime = $runtime;
        if future.get_runtime() != runtime {
            $crate::private::safety::invalid_argument(
                $error,
                "future is not associated with the given runtime",
            );
            return Default::default();
        }
        future
    }};
}

#[derive(Clone)]
pub struct RuntimeT {
    runtime: Arc<tokio::runtime::Runtime>,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_destroy(runtime: *mut RuntimeT) {
    safe_drop!(runtime);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_clone(runtime: *const RuntimeT) -> *mut RuntimeT {
    Box::into_raw(Box::new(safe_as_ref!(runtime).clone()))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_address(runtime: *const RuntimeT) -> usize {
    Arc::as_ptr(&safe_as_ref!(runtime).runtime) as usize
}

// Make progress on all tasks scheduled on this runtime.
//
// Use this function to make *some* progress on all scheduled tasks without blocking the current thread for an extended
// period of time (conceptually, a "single pass" through the task queue), such as in an event loop or between other work
// on a worker thread.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress(runtime: *const RuntimeT) {
    safe_as_ref!(runtime).make_progress();
}

// Like `make_progress()`, but (soft) upper-bounded by `timeout_ms`.
//
// Returns a timeout error when the runtime is unable to finish making progress on all scheduled tasks (conceptually, a
// "single pass" through the task queue) before the timeout deadline.
//
// Use this function when a (soft) upper-bound is required on the time spent potentially blocked on `make_progress()`.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress_with_timeout(
    runtime: *const RuntimeT,
    timeout_ms: u64,
    error: *mut ErrorT,
) {
    safe_error!(
        safe_as_ref!(runtime).make_progress_with_timeout(Duration::from_millis(timeout_ms)),
        safe_optional_error_as_mut!(error)
    );
}

// Like `make_progress()`, but lower-bounded by `duration_ms`.
//
// Use this function when a worker thread or event loop can budget a minimum amount of time spent making progress on
// scheduled tasks (conceptually, repeatedly making a "single pass" through the task queue).
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress_for(runtime: *const RuntimeT, duration_ms: u64) {
    safe_as_ref!(runtime).make_progress_for(Duration::from_millis(duration_ms));
}

// Block the current thread by making progress until the `future` is ready.
//
// Use this function when the current thread can make progress on all scheduled tasks until the `future` is ready.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on(
    runtime: *const RuntimeT,
    future: *mut FutureT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);
    let future = safe_from_runtime_with_error!(safe_as_mut!(future), runtime, error);

    runtime.block_on_future(future);
}

// Like `block_on()`, but (soft) upper-bounded by `timeout_ms`.
//
// Use this function when a (soft) upper bound is required on the time spent potentially blocked on `block_on()`.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_with_timeout(
    runtime: *const RuntimeT,
    future: *mut FutureT,
    timeout_ms: u64,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);
    let future = safe_from_runtime_with_error!(safe_as_mut!(future), runtime, error);

    safe_error!(
        runtime.block_on_future_with_timeout(future, Duration::from_millis(timeout_ms)),
        error
    );
}

// Like `block_on()`, but returns when *any* future is ready.
//
// Returns a pointer to the element in `futures` that is ready.
//
// Use this function when the current thread can make progress on all scheduled tasks until *any* of the given futures
// is ready.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mongoac_runtime_block_on_any(
    runtime: *const RuntimeT,
    futures: *const *mut FutureT,
    count: usize,
    error: *mut ErrorT,
) -> *const *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let Some(mut futs) = safe_error!(futures_as_muts_for_any(futures, count, runtime), error)
    else {
        return Default::default();
    };

    match runtime.block_on_any(&mut futs) {
        Some(i) => unsafe { futures.add(i) },
        None => Default::default(),
    }
}

// Like `block_on_any()``, but (soft) upper-bounded by `timeout_ms`.
//
// Use this function when a (soft) upper bound is required on the time spent potentially blocked on `block_on_any()`.
#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mongoac_runtime_block_on_any_with_timeout(
    runtime: *const RuntimeT,
    futures: *const *mut FutureT,
    count: usize,
    timeout_ms: u64,
    error: *mut ErrorT,
) -> *const *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let Some(mut futs) = safe_error!(futures_as_muts_for_any(futures, count, runtime), error)
    else {
        return Default::default();
    };

    match safe_error!(
        runtime.block_on_any_with_timeout(&mut futs, Duration::from_millis(timeout_ms)),
        error
    ) {
        Some(i) => unsafe { futures.add(i) },
        None => Default::default(),
    }
}

// Like `block_on()`, but returns when *all* futures are ready.
//
// Use this function when the current thread can make progress on all scheduled tasks until *all* of the given futures
// are ready.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_all(
    runtime: *const RuntimeT,
    futures: *const *mut FutureT,
    count: usize,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let Some(mut futs) = safe_error!(futures_as_muts_for_all(futures, count, runtime), error)
    else {
        return;
    };

    runtime.block_on_all(&mut futs);
}

// Like `block_on_all()`, but (soft) upper-bounded by `timeout_ms`.
//
// Use this function when a (soft) upper bound is required on the time spent potentially blocked on `block_on_all()`.
#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_all_with_timeout(
    runtime: *const RuntimeT,
    futures: *const *mut FutureT,
    count: usize,
    timeout_ms: u64,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let Some(mut futs) = safe_error!(futures_as_muts_for_all(futures, count, runtime), error)
    else {
        return;
    };

    safe_error!(
        runtime.block_on_all_with_timeout(&mut futs, Duration::from_millis(timeout_ms)),
        error
    );
}

impl PartialEq for RuntimeT {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.runtime, &other.runtime)
    }
}

impl Eq for RuntimeT {}

impl RuntimeT {
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            runtime: Arc::new(runtime),
        })
    }

    pub(crate) fn make_progress(&self) {
        self.runtime.block_on(tokio::task::yield_now());
    }

    pub(crate) fn make_progress_with_timeout(&self, timeout: Duration) -> Result<(), ErrorT> {
        let deadline = tokio::time::Instant::now() + timeout;

        self.runtime.block_on(async {
            tokio::time::timeout_at(deadline, tokio::task::yield_now()).await?;
            Ok(())
        })
    }

    pub(crate) fn make_progress_for(&self, duration: Duration) {
        let deadline = tokio::time::Instant::now() + duration;

        self.runtime
            .block_on(async { tokio::time::sleep_until(deadline).await });
    }

    pub(crate) fn block_on<F: Future>(&self, future: F) -> F::Output {
        self.runtime.block_on(future)
    }

    pub(crate) fn block_on_future(&self, future: &mut FutureT) {
        if future.is_ready() {
            return; // No work to do.
        }

        self.runtime.block_on(future.poll_async());
    }

    pub(crate) fn block_on_future_with_timeout(
        &self,
        future: &mut FutureT,
        timeout: Duration,
    ) -> Result<(), ErrorT> {
        let deadline = tokio::time::Instant::now() + timeout;

        if future.is_ready() {
            return Ok(()); // No work to do.
        }

        self.runtime.block_on(async {
            tokio::time::timeout_at(deadline, future.poll_async()).await?;
            Ok(())
        })
    }

    pub(crate) fn block_on_any<'a>(
        &self,
        futures: &'a mut [(usize, &'a mut FutureT)],
    ) -> Option<usize> {
        // Check for completion before executing `block_on()`.
        if let Some(i) = any_ready(futures) {
            return Some(i);
        }

        self.runtime
            .block_on(futures_unordered_for_any(futures).next())
    }

    pub(crate) fn block_on_any_with_timeout<'a>(
        &self,
        futures: &'a mut [(usize, &'a mut FutureT)],
        timeout: Duration,
    ) -> Result<Option<usize>, ErrorT> {
        let deadline = tokio::time::Instant::now() + timeout;

        // Check for completion before executing `block_on()`.
        if let Some(i) = any_ready(futures) {
            return Ok(Some(i));
        }

        self.runtime.block_on(async {
            Ok(
                tokio::time::timeout_at(deadline, futures_unordered_for_any(futures).next())
                    .await?,
            )
        })
    }

    pub(crate) fn block_on_all<'a>(&self, futures: &'a mut [&'a mut FutureT]) {
        // Check for completion before executing `block_on()`.
        if all_ready(futures) {
            return;
        }

        self.runtime.block_on(async {
            let mut fut_set = futures_unordered_for_all(futures);
            while fut_set.next().await.is_some() {}
        });
    }

    pub(crate) fn block_on_all_with_timeout<'a>(
        &self,
        futures: &'a mut [&'a mut FutureT],
        timeout: Duration,
    ) -> Result<(), ErrorT> {
        let deadline = tokio::time::Instant::now() + timeout;

        // Check for completion before executing `block_on()`.
        if all_ready(futures) {
            return Ok(());
        }

        self.runtime.block_on(async {
            let mut fut_set = futures_unordered_for_all(futures);
            tokio::time::timeout_at(deadline, async { while fut_set.next().await.is_some() {} })
                .await?;
            Ok(())
        })
    }

    pub(crate) fn from_raw(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            runtime: Arc::new(runtime),
        }
    }

    pub(crate) fn get_runtime(&self) -> &tokio::runtime::Runtime {
        &self.runtime
    }

    pub(crate) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }
}

fn futures_unordered_for_any<'a>(
    futures: &'a mut [(usize, &'a mut FutureT)],
) -> FuturesUnordered<FutureExt<'a>> {
    futures
        .iter_mut()
        .map(|(i, f)| FutureExt::new_with_index(f, *i))
        .collect()
}

fn futures_unordered_for_all<'a>(
    futures: &'a mut [&'a mut FutureT],
) -> FuturesUnordered<FutureExt<'a>> {
    futures.iter_mut().map(|f| FutureExt::new(f)).collect()
}

fn any_ready(futures: &[(usize, &mut FutureT)]) -> Option<usize> {
    futures.iter().find(|(_, f)| f.is_ready()).map(|(i, _)| *i)
}

fn all_ready(futures: &[&mut FutureT]) -> bool {
    futures.iter().all(|f| f.is_ready())
}

fn futures_as_muts_for_any<'a>(
    futures: *const *mut FutureT,
    count: usize,
    runtime: &RuntimeT,
) -> Result<Option<Vec<(usize, &'a mut FutureT)>>, ErrorT> {
    if futures.is_null() || count == 0 {
        return Ok(None); // No work to do.
    }

    // SAFETY: `futures` and `count` validity is an uncheckable precondition.
    let refs = futures_as_muts(
        unsafe { std::slice::from_raw_parts(futures, count) },
        runtime,
    )?;

    Ok(Some(refs))
}

fn futures_as_muts_for_all<'a>(
    futures: *const *mut FutureT,
    count: usize,
    runtime: &RuntimeT,
) -> Result<Option<Vec<&'a mut FutureT>>, ErrorT> {
    if futures.is_null() || count == 0 {
        return Ok(None); // No work to do.
    }

    // SAFETY: `futures` and `count` validity is an uncheckable precondition.
    let refs: Vec<&'a mut FutureT> = futures_as_muts(
        unsafe { std::slice::from_raw_parts(futures, count) },
        runtime,
    )
    .map(|v| v.into_iter().map(|(_, f)| f).collect())?;

    Ok(Some(refs))
}

fn futures_as_muts<'a>(
    futures: &[*mut FutureT],
    runtime: &RuntimeT,
) -> Result<Vec<(usize, &'a mut FutureT)>, ErrorT> {
    let mut ret = Vec::with_capacity(futures.len());

    for (i, ptr) in futures.iter().enumerate() {
        let Some(future) = safe_optional_as_mut!(*ptr) else {
            return Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &format!("futures array element at index {i}: must not be null"),
            ));
        };

        if future.get_runtime() != runtime {
            return Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &format!(
                    "futures array element at index {i}: future is not associated with the given runtime"
                ),
            ));
        }

        ret.push((i, future));
    }

    Ok(ret)
}

#[cfg(test)]
mod tests {
    use crate::future::{FutureT, FutureValue, FutureValueType};
    use crate::private::test_util::make_runtime;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn make_progress_with_timeout_returns_ok() {
        let runtime = make_runtime();
        assert!(
            runtime
                .make_progress_with_timeout(Duration::from_millis(50))
                .is_ok()
        );
    }

    #[test]
    fn make_progress_with_timeout_returns_immediately() {
        let runtime = make_runtime();

        let result = runtime.make_progress_with_timeout(Duration::from_millis(0));

        assert!(
            result.is_ok(),
            "make_progress_with_timeout should complete before the timeout, not time out"
        );
    }

    #[test]
    fn make_progress_for_blocks_for_at_least_duration() {
        let runtime = make_runtime();

        // Use a barrier-synchronized helper thread to verify that
        // make_progress_for blocks the calling thread (giving other threads
        // a chance to run) instead of returning immediately. This replaces
        // the wall-clock timing assertion which is unreliable under Miri.
        //
        // The duration is generous (2 seconds) because Miri's interpretation
        // overhead can consume the entire 50ms budget during setup, causing
        // sleep_until to return Ready before the helper thread is scheduled.
        let flag = Arc::new(AtomicBool::new(false));
        let flag_clone = flag.clone();
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            flag_clone.store(true, Ordering::Release);
        });

        barrier.wait();
        runtime.make_progress_for(Duration::from_secs(2));

        assert!(
            flag.load(Ordering::Acquire),
            "make_progress_for should block, allowing other threads to run"
        );
        handle.join().unwrap();
    }

    #[test]
    fn make_progress_for_drives_background_task() {
        let runtime = make_runtime();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        runtime.spawn(async move {
            // Use yield_now instead of sleep: the task completes after
            // yielding, which make_progress_for can drive without relying on
            // real-time timers.
            tokio::task::yield_now().await;
            done_clone.store(true, Ordering::Release);
        });

        // The duration is generous (2 seconds) because Miri's interpretation
        // overhead can consume a shorter budget during setup, causing
        // sleep_until to return Ready before the event loop processes the
        // spawned task.
        runtime.make_progress_for(Duration::from_secs(2));

        assert!(
            done.load(Ordering::Acquire),
            "background task should complete while the runtime is driven"
        );
    }

    #[test]
    fn current_thread_runtime_can_select_over_multiple_futures() {
        let runtime = make_runtime();

        let short_done = Arc::new(AtomicBool::new(false));
        let long_done = Arc::new(AtomicBool::new(false));
        let short_done_clone = short_done.clone();
        let long_done_clone = long_done.clone();

        let short = runtime.spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            short_done_clone.store(true, Ordering::Release);
            1u32
        });
        let long = runtime.spawn(async move {
            tokio::time::sleep(Duration::from_millis(500)).await;
            long_done_clone.store(true, Ordering::Release);
            2u32
        });

        let result = runtime.block_on(async move {
            tokio::select! {
                v = short => v,
                v = long => v,
            }
        });

        assert_eq!(result.unwrap(), 1);
        assert!(
            short_done.load(Ordering::Acquire),
            "short task should have completed"
        );

        // The long task is still in the runtime even though its JoinHandle was
        // dropped by select!. Drive the runtime until it completes too.
        let start = Instant::now();
        while !long_done.load(Ordering::Acquire) {
            runtime.make_progress();
            if start.elapsed() > Duration::from_secs(5) {
                break;
            }
        }
        assert!(
            long_done.load(Ordering::Acquire),
            "the long task should eventually complete after the runtime is driven"
        );
    }

    // Helpers for the concurrency tests below.

    fn long_future(runtime: &super::RuntimeT) -> FutureT {
        FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(runtime.spawn(async move {
                Ok(tokio::time::sleep(Duration::from_secs(10)).await)
            }))),
        )
    }

    fn spawn_driver_thread(
        rt: super::RuntimeT,
        duration: Duration,
    ) -> (Arc<std::sync::Barrier>, thread::JoinHandle<()>) {
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            rt.block_on(async {
                tokio::time::sleep(duration).await;
            });
        });
        (barrier, handle)
    }

    fn spawn_driver_thread_until_flag(
        rt: super::RuntimeT,
        stop_flag: Arc<AtomicBool>,
    ) -> (Arc<std::sync::Barrier>, thread::JoinHandle<()>) {
        let barrier = Arc::new(std::sync::Barrier::new(2));
        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            barrier_clone.wait();
            rt.block_on(async {
                while !stop_flag.load(Ordering::Acquire) {
                    tokio::task::yield_now().await;
                }
            });
        });
        (barrier, handle)
    }

    // -- Concurrency safety claims for the runtime model --

    #[test]
    fn spawn_is_safe_while_runtime_is_driven() {
        let runtime = make_runtime();
        let rt = runtime.clone();
        let spawned_count = Arc::new(AtomicUsize::new(0));
        let completed_count = Arc::new(AtomicUsize::new(0));
        let all_spawned = Arc::new(AtomicBool::new(false));
        let spawned_count_clone = spawned_count.clone();
        let completed_count_clone = completed_count.clone();
        let all_spawned_clone = all_spawned.clone();

        let handle = thread::spawn(move || {
            for _ in 0..10 {
                let completed = completed_count_clone.clone();
                rt.spawn(async move {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    completed.fetch_add(1, Ordering::Release);
                });
                spawned_count_clone.fetch_add(1, Ordering::Release);
            }
            all_spawned_clone.store(true, Ordering::Release);
        });

        // Drive the runtime until the secondary thread has spawned all tasks
        // and those tasks have had a chance to run. The yield loop is robust
        // under Miri's deterministic scheduler because each yield point gives
        // Miri an opportunity to switch threads.
        runtime.block_on(async {
            let deadline = Instant::now() + Duration::from_secs(5);
            while spawned_count.load(Ordering::Acquire) < 10 && Instant::now() < deadline {
                tokio::task::yield_now().await;
            }
            while completed_count.load(Ordering::Acquire) < 10 && Instant::now() < deadline {
                tokio::task::yield_now().await;
            }
            assert_eq!(
                spawned_count.load(Ordering::Acquire),
                10,
                "secondary thread should spawn all tasks while runtime is driven"
            );
            assert_eq!(
                completed_count.load(Ordering::Acquire),
                10,
                "all spawned tasks should complete while runtime is driven"
            );
        });

        handle.join().unwrap();
    }

    #[test]
    fn future_is_ready_and_clone_are_safe_while_runtime_is_driven() {
        let runtime = make_runtime();
        let mut future = FutureT::new(
            runtime.clone(),
            FutureValue::Void(FutureValueType::new(runtime.spawn(async move {
                Ok(tokio::time::sleep(Duration::from_millis(50)).await)
            }))),
        );

        let future_for_thread = future.clone();
        let handle = thread::spawn(move || {
            let start = Instant::now();
            while start.elapsed() < Duration::from_millis(100) {
                let _ = future_for_thread.is_ready();
                let _ = future_for_thread.clone();
                thread::sleep(Duration::from_millis(1));
            }
        });

        runtime.block_on_future(&mut future);
        handle.join().unwrap();
        assert!(future.is_ready());
    }

    // -- Tokio-current_thread behavior: concurrent progress-driving is allowed --
    //
    // Multiple threads may call `Runtime::block_on` / `make_progress` on the
    // same current_thread runtime. Tokio coordinates driver ownership and
    // stealing between threads. The tests below document and guard that
    // supported behavior.

    #[test]
    fn concurrent_make_progress_is_allowed_by_tokio() {
        let runtime = make_runtime();
        let (barrier, driver) = spawn_driver_thread(runtime.clone(), Duration::from_millis(100));

        barrier.wait();
        runtime.make_progress();

        driver.join().unwrap();
    }

    #[test]
    fn concurrent_block_on_is_allowed_by_tokio() {
        let runtime = make_runtime();
        let (barrier, driver) = spawn_driver_thread(runtime.clone(), Duration::from_millis(100));

        barrier.wait();
        runtime.block_on(async {
            tokio::task::yield_now().await;
        });

        driver.join().unwrap();
    }

    // -- Timeout API correctness under concurrent runtime driving --

    #[test]
    fn block_on_future_with_timeout_fires_while_another_thread_drives_runtime() {
        let runtime = make_runtime();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (barrier, driver) = spawn_driver_thread_until_flag(runtime.clone(), stop_flag.clone());
        let mut future = long_future(&runtime);

        barrier.wait();
        let result = runtime.block_on_future_with_timeout(&mut future, Duration::from_millis(50));

        stop_flag.store(true, Ordering::Release);
        driver.join().unwrap();

        assert!(result.is_err(), "expected timeout error");
        assert!(!future.is_ready());
    }

    #[test]
    fn make_progress_with_timeout_succeeds_while_another_thread_drives_runtime() {
        let runtime = make_runtime();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (barrier, driver) = spawn_driver_thread_until_flag(runtime.clone(), stop_flag.clone());

        barrier.wait();
        let result = runtime.make_progress_with_timeout(Duration::from_millis(50));

        stop_flag.store(true, Ordering::Release);
        driver.join().unwrap();

        assert!(result.is_ok(), "make_progress should yield before timeout");
    }

    #[test]
    fn block_on_any_with_timeout_fires_while_another_thread_drives_runtime() {
        let runtime = make_runtime();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (barrier, driver) = spawn_driver_thread_until_flag(runtime.clone(), stop_flag.clone());
        let mut f1 = long_future(&runtime);
        let mut f2 = long_future(&runtime);

        let mut futures = [(0, &mut f1), (1, &mut f2)];
        barrier.wait();
        let result = runtime.block_on_any_with_timeout(&mut futures, Duration::from_millis(50));

        stop_flag.store(true, Ordering::Release);
        driver.join().unwrap();

        assert!(result.is_err(), "expected timeout error");
        assert!(!f1.is_ready());
        assert!(!f2.is_ready());
    }

    #[test]
    fn block_on_all_with_timeout_fires_while_another_thread_drives_runtime() {
        let runtime = make_runtime();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let (barrier, driver) = spawn_driver_thread_until_flag(runtime.clone(), stop_flag.clone());
        let mut f1 = long_future(&runtime);
        let mut f2 = long_future(&runtime);

        let mut futures = [&mut f1, &mut f2];
        barrier.wait();
        let result = runtime.block_on_all_with_timeout(&mut futures, Duration::from_millis(50));

        stop_flag.store(true, Ordering::Release);
        driver.join().unwrap();

        assert!(result.is_err(), "expected timeout error");
        assert!(!f1.is_ready());
        assert!(!f2.is_ready());
    }
}
