use crate::error::{ErrorCodeT, ErrorT};
use crate::future::{FutureExt, FutureT};
use crate::safe_drop;
use crate::safe_optional_as_ref;
use crate::safe_optional_error_as_mut;
use crate::{safe_as_ref, safe_error};

use event_listener::{Event, Listener};
use futures_util::stream::{FuturesUnordered, StreamExt};
use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

#[macro_export]
macro_rules! safe_from_runtime_with_error {
    ($future:expr, $runtime:expr, $error:expr) => {{
        let future = $future;
        if !future.is_from_runtime($runtime) {
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
    state: Arc<RuntimeState>,
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
pub extern "C" fn mongoac_runtime_make_progress(runtime: *mut RuntimeT) -> bool {
    safe_as_ref!(runtime).make_progress()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress_with_timeout(
    runtime: *mut RuntimeT,
    timeout_ms: u64,
) -> bool {
    safe_as_ref!(runtime).make_progress_with_timeout(Duration::from_millis(timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_request_stop(runtime: *mut RuntimeT) {
    safe_as_ref!(runtime).request_stop()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_stop_requested(runtime: *const RuntimeT) -> bool {
    safe_as_ref!(runtime).stop_requested()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_wait(runtime: *mut RuntimeT) {
    safe_as_ref!(runtime).wait();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_wait_with_timeout(
    runtime: *mut RuntimeT,
    timeout_ms: u64,
) -> bool {
    safe_as_ref!(runtime).wait_with_timeout(Duration::from_millis(timeout_ms))
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on(
    runtime: *mut RuntimeT,
    future: *const FutureT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);
    let future = safe_from_runtime_with_error!(safe_as_ref!(future), runtime, error);

    runtime.block_on_future(future);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_with_timeout(
    runtime: *mut RuntimeT,
    future: *const FutureT,
    timeout_ms: u64,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);
    let future = safe_from_runtime_with_error!(safe_as_ref!(future), runtime, error);

    safe_error!(
        runtime.block_on_future_with_timeout(future, Duration::from_millis(timeout_ms)),
        error
    );
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mongoac_runtime_block_on_any(
    runtime: *mut RuntimeT,
    futures: *const *const FutureT,
    count: usize,
    error: *mut ErrorT,
) -> *const *const FutureT {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let refs = match safe_error!(futures_as_refs_for_any(futures, count, runtime), error) {
        Some(refs) => refs,
        None => return Default::default(),
    };

    match runtime.block_on_any(&refs) {
        Some(i) => unsafe { futures.add(i) },
        None => Default::default(),
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn mongoac_runtime_block_on_any_with_timeout(
    runtime: *mut RuntimeT,
    futures: *const *const FutureT,
    count: usize,
    timeout_ms: u64,
    error: *mut ErrorT,
) -> *const *const FutureT {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let refs = match safe_error!(futures_as_refs_for_any(futures, count, runtime), error) {
        Some(refs) => refs,
        None => return Default::default(),
    };

    match safe_error!(
        runtime.block_on_any_with_timeout(&refs, Duration::from_millis(timeout_ms)),
        error
    ) {
        Some(i) => unsafe { futures.add(i) },
        None => Default::default(),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_all(
    runtime: *mut RuntimeT,
    futures: *const *const FutureT,
    count: usize,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let refs = match safe_error!(futures_as_refs_for_all(futures, count, runtime), error) {
        Some(refs) => refs,
        None => return,
    };

    runtime.block_on_all(&refs);
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_all_with_timeout(
    runtime: *mut RuntimeT,
    futures: *const *const FutureT,
    count: usize,
    timeout_ms: u64,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_ref!(runtime);

    let refs = match safe_error!(futures_as_refs_for_all(futures, count, runtime), error) {
        Some(refs) => refs,
        None => return,
    };

    safe_error!(
        runtime.block_on_all_with_timeout(&refs, Duration::from_millis(timeout_ms)),
        error
    )
}

impl PartialEq for RuntimeT {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.state, &other.state)
    }
}

impl Eq for RuntimeT {}

impl RuntimeT {
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            state: Arc::new(RuntimeState::new(runtime)),
        })
    }

    pub(crate) fn make_progress(&self) -> bool {
        self.try_with_progress_lock(|runtime| {
            runtime.block_on(tokio::task::yield_now());
        })
    }

    pub(crate) fn make_progress_with_timeout(&self, timeout: Duration) -> bool {
        self.try_with_progress_lock(|runtime| {
            runtime.block_on(async {
                let _ = tokio::time::timeout(timeout, tokio::task::yield_now()).await;
            });
        })
    }

    pub(crate) fn request_stop(&self) {
        self.state.stop_requested.store(true, Ordering::Release);
        self.state.wait_flag.store(true, Ordering::Release);
        self.state.wait_event.notify(usize::MAX);
    }

    pub(crate) fn stop_requested(&self) -> bool {
        self.state.stop_requested.load(Ordering::Acquire)
    }

    pub(crate) fn wait(&self) {
        let _ = self.wait_impl(None);
    }

    pub(crate) fn wait_with_timeout(&self, timeout: Duration) -> bool {
        self.wait_impl(Some(timeout))
    }

    pub(crate) fn block_on<F: Future>(&self, future: F) -> F::Output {
        let _guard = self.state.progress_lock.lock();
        self.state.runtime.block_on(future)
    }

    pub(crate) fn block_on_future(&self, future: &FutureT) {
        self.with_progress_lock(
            || future.is_ready().then_some(()),
            |runtime| runtime.block_on(future.poll()),
        );
    }

    pub(crate) fn block_on_future_with_timeout(
        &self,
        future: &FutureT,
        timeout: Duration,
    ) -> Result<(), ErrorT> {
        self.with_progress_lock(
            || future.is_ready().then_some(Ok(())),
            |runtime| {
                runtime.block_on(async {
                    tokio::time::timeout(timeout, future.poll())
                        .await
                        .map_err(|_| {
                            ErrorT::from_mongoac(
                                ErrorCodeT::Timeout,
                                "block_on_future_with_timeout",
                            )
                        })
                })
            },
        )
    }

    pub(crate) fn block_on_any<'a>(&self, futures: &'a [(usize, &'a FutureT)]) -> Option<usize> {
        // Skip double-checked lock: futures are expected to be pending.
        let _guard = self.state.progress_lock.lock();

        // Check for completion before executing `block_on()`.
        if let Some(i) = any_ready(futures) {
            return Some(i);
        }

        self.state
            .runtime
            .block_on(async { futures_unordered_for_any(futures).next().await })
    }

    pub(crate) fn block_on_any_with_timeout<'a>(
        &self,
        futures: &'a [(usize, &'a FutureT)],
        timeout: Duration,
    ) -> Result<Option<usize>, ErrorT> {
        // Skip double-checked lock: futures are expected to be pending.
        let _guard = self.state.progress_lock.lock();

        // Check for completion before executing `block_on()`.
        if let Some(i) = any_ready(futures) {
            return Ok(Some(i));
        }

        self.state.runtime.block_on(async {
            tokio::time::timeout(timeout, futures_unordered_for_any(futures).next())
                .await
                .map_err(|_| ErrorT::from_mongoac(ErrorCodeT::Timeout, "block_on_any_with_timeout"))
        })
    }

    pub(crate) fn block_on_all(&self, futures: &[&FutureT]) {
        // Skip double-checked lock: futures are expected to be pending.
        let _guard = self.state.progress_lock.lock();

        // Check for completion before executing `block_on()`.
        if all_ready(futures) {
            return;
        }

        self.state.runtime.block_on(async {
            let mut fut_set = futures_unordered_for_all(futures);
            while fut_set.next().await.is_some() {}
        });
    }

    pub(crate) fn block_on_all_with_timeout(
        &self,
        futures: &[&FutureT],
        timeout: Duration,
    ) -> Result<(), ErrorT> {
        // Skip double-checked lock: futures are expected to be pending.
        let _guard = self.state.progress_lock.lock();

        // Check for completion before executing `block_on()`.
        if all_ready(futures) {
            return Ok(());
        }

        self.state.runtime.block_on(async {
            let mut fut_set = futures_unordered_for_all(futures);
            tokio::time::timeout(timeout, async { while fut_set.next().await.is_some() {} })
                .await
                .map_err(|_| ErrorT::from_mongoac(ErrorCodeT::Timeout, "block_on_all_with_timeout"))
        })
    }

    pub(crate) fn from_raw(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            state: Arc::new(RuntimeState::new(runtime)),
        }
    }

    pub(crate) fn get_runtime(&self) -> &tokio::runtime::Runtime {
        &self.state.runtime
    }

    pub(crate) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.state.runtime.spawn(future);
        self.state.wait_flag.store(true, Ordering::Release);
        self.state.wait_event.notify(usize::MAX);
        handle
    }

    fn with_progress_lock<R>(
        &self,
        mut cond: impl FnMut() -> Option<R>,
        run: impl FnOnce(&tokio::runtime::Runtime) -> R,
    ) -> R {
        // Double-checked lock.
        if let Some(result) = cond() {
            return result;
        }

        let _guard = self.state.progress_lock.lock();

        // Double-checked lock.
        if let Some(result) = cond() {
            return result;
        }

        run(&self.state.runtime)
    }

    fn try_with_progress_lock<Op>(&self, op: Op) -> bool
    where
        Op: FnOnce(&tokio::runtime::Runtime),
    {
        let Some(_guard) = self.state.progress_lock.try_lock() else {
            return false;
        };

        op(&self.state.runtime);
        true
    }

    fn wait_impl(&self, timeout: Option<Duration>) -> bool {
        // Double-checked predicate against listener registration.
        if self.state.stop_requested.load(Ordering::Acquire)
            || self.state.wait_flag.swap(false, Ordering::Acquire)
        {
            return true;
        }

        let listener = self.state.wait_event.listen();

        // Double-checked predicate against listener registration.
        if self.state.stop_requested.load(Ordering::Acquire)
            || self.state.wait_flag.swap(false, Ordering::Acquire)
        {
            return true;
        }

        match timeout {
            None => {
                listener.wait();
                true
            }
            Some(timeout) => listener.wait_timeout(timeout).is_some(),
        }
    }
}

struct RuntimeState {
    runtime: tokio::runtime::Runtime,
    // Only one thread can `block_on*()` or `make_progress*()` at a time.
    progress_lock: Mutex<()>,
    // Used to signal waiting threads that no more work should be done.
    stop_requested: AtomicBool,
    // Used to signal waiting threads when work is available.
    wait_flag: AtomicBool,
    // used to wake waiting threads when work is available or stop is requested.
    wait_event: Event,
}

impl RuntimeState {
    fn new(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            runtime,
            progress_lock: Mutex::new(()),
            stop_requested: AtomicBool::new(false),
            wait_flag: AtomicBool::new(false),
            wait_event: Event::new(),
        }
    }
}

fn futures_unordered_for_any<'a>(
    futures: &'a [(usize, &'a FutureT)],
) -> FuturesUnordered<FutureExt<'a>> {
    futures
        .iter()
        .map(|(i, f)| FutureExt::new_with_index(f, *i))
        .collect()
}

fn futures_unordered_for_all<'a>(futures: &'a [&'a FutureT]) -> FuturesUnordered<FutureExt<'a>> {
    futures.iter().map(|f| FutureExt::new(f)).collect()
}

fn any_ready(futures: &[(usize, &FutureT)]) -> Option<usize> {
    futures.iter().find(|(_, f)| f.is_ready()).map(|(i, _)| *i)
}

fn all_ready(futures: &[&FutureT]) -> bool {
    futures.iter().all(|f| f.is_ready())
}

fn futures_as_refs_for_any<'a>(
    futures: *const *const FutureT,
    count: usize,
    runtime: &RuntimeT,
) -> Result<Option<Vec<(usize, &'a FutureT)>>, ErrorT> {
    if futures.is_null() || count == 0 {
        return Ok(None); // No work to do.
    }

    // SAFETY: `futures` and `count` validity is an uncheckable precondition.
    let refs = futures_as_refs(
        unsafe { std::slice::from_raw_parts(futures, count) },
        runtime,
    )?;

    if refs.is_empty() {
        return Ok(None); // No work to do.
    }

    Ok(Some(refs))
}

fn futures_as_refs_for_all<'a>(
    futures: *const *const FutureT,
    count: usize,
    runtime: &RuntimeT,
) -> Result<Option<Vec<&'a FutureT>>, ErrorT> {
    if futures.is_null() || count == 0 {
        return Ok(None); // No work to do.
    }

    // SAFETY: `futures` and `count` validity is an uncheckable precondition.
    let refs: Vec<&'a FutureT> = futures_as_refs(
        unsafe { std::slice::from_raw_parts(futures, count) },
        runtime,
    )
    .map(|v| v.into_iter().map(|(_, f)| f).collect())?;

    if refs.is_empty() {
        return Ok(None); // No work to do.
    }

    Ok(Some(refs))
}

fn futures_as_refs<'a>(
    futures: &'a [*const FutureT],
    runtime: &RuntimeT,
) -> Result<Vec<(usize, &'a FutureT)>, ErrorT> {
    let mut ret = Vec::with_capacity(futures.len());

    for (i, ptr) in futures.iter().enumerate() {
        let Some(future) = safe_optional_as_ref!(*ptr) else {
            return Err(ErrorT::from_mongoac(
                ErrorCodeT::InvalidArgument,
                &format!("futures array element at index {i}: must not be null"),
            ));
        };

        if !future.is_from_runtime(runtime) {
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
    use crate::private::test_util::make_runtime;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn make_progress_returns_true_when_lock_available() {
        let runtime = make_runtime();
        assert!(runtime.make_progress());
    }

    #[test]
    fn make_progress_with_timeout_returns_true_when_lock_available() {
        let runtime = make_runtime();
        assert!(runtime.make_progress_with_timeout(Duration::from_millis(50)));
    }

    #[test]
    fn make_progress_with_timeout_returns_immediately_without_contention() {
        let runtime = make_runtime();

        let t0 = Instant::now();
        let _ = runtime.make_progress_with_timeout(Duration::from_millis(50));
        let elapsed = t0.elapsed();

        assert!(
            elapsed < Duration::from_millis(75),
            "expected near-immediate return, but call took {elapsed:?}"
        );
    }

    #[test]
    fn make_progress_with_timeout_returns_early_with_contention() {
        let runtime = make_runtime();

        for _ in 0..100 {
            runtime.get_runtime().spawn(async {
                let t = Instant::now() + Duration::from_millis(1);
                while Instant::now() < t {
                    std::hint::spin_loop();
                }
                tokio::task::yield_now().await;
            });
        }

        let started = Instant::now();
        let _ = runtime.make_progress_with_timeout(Duration::from_millis(50));
        let elapsed = started.elapsed();

        assert!(
            elapsed < Duration::from_millis(75),
            "expected timeout to bound block_on, but call took {elapsed:?}"
        );
    }

    #[test]
    fn wait_returns_immediately_when_work_already_available() {
        let runtime = make_runtime();
        runtime.spawn(async { tokio::task::yield_now().await });

        let t0 = Instant::now();
        runtime.wait();
        let elapsed = t0.elapsed();

        assert!(
            elapsed < Duration::from_millis(10),
            "expected immediate return, but call took {elapsed:?}"
        );
    }

    #[test]
    fn wait_blocks_until_work_is_spawned() {
        let runtime = make_runtime();
        let ready = Arc::new(AtomicBool::new(false));

        let worker = thread::spawn({
            let runtime = runtime.clone();
            let ready = ready.clone();

            move || {
                runtime.wait();
                ready.store(true, Ordering::Release);
            }
        });

        thread::sleep(Duration::from_millis(10));
        assert!(!ready.load(Ordering::Acquire));

        runtime.spawn(async { tokio::task::yield_now().await });
        worker.join().unwrap();

        assert!(ready.load(Ordering::Acquire));
    }

    #[test]
    fn wait_with_timeout_returns_false_when_no_work() {
        let runtime = make_runtime();

        let t0 = Instant::now();
        assert!(!runtime.wait_with_timeout(Duration::from_millis(10)));
        let elapsed = t0.elapsed();

        assert!(
            elapsed >= Duration::from_millis(10),
            "expected full 10ms timeout, but got {elapsed:?}"
        );
    }

    #[test]
    fn wait_with_timeout_returns_true_when_work_is_spawned() {
        let runtime = make_runtime();

        let worker = thread::spawn({
            let runtime = runtime.clone();
            move || {
                assert!(runtime.wait_with_timeout(Duration::from_secs(10)));
            }
        });

        thread::sleep(Duration::from_millis(10));
        runtime.spawn(async { tokio::task::yield_now().await });

        worker.join().unwrap();
    }

    #[test]
    fn wait_can_be_reused_after_wake() {
        let runtime = make_runtime();

        for _ in 0..3 {
            let rt = runtime.clone();
            let worker = thread::spawn(move || {
                rt.wait();
            });

            thread::sleep(Duration::from_millis(1));
            runtime.spawn(async { tokio::task::yield_now().await });

            worker.join().unwrap();
        }
    }

    #[test]
    fn request_stop_wakes_all_waiting_workers() {
        let runtime = make_runtime();
        let num_workers = 3;
        let mut handles = Vec::new();

        for _ in 0..num_workers {
            let rt = runtime.clone();
            handles.push(thread::spawn(move || {
                rt.wait();
                rt.stop_requested()
            }));
        }

        // Give workers time to start waiting on the runtime.
        thread::sleep(Duration::from_millis(50));

        assert!(!runtime.stop_requested());
        runtime.request_stop();

        let stopped = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|result| *result)
            .count();

        assert_eq!(
            stopped, num_workers,
            "request_stop should wake all workers and each should observe the stop request"
        );
        assert!(runtime.stop_requested());
    }

    #[test]
    fn request_stop_makes_subsequent_wait_return_immediately() {
        let runtime = make_runtime();

        runtime.request_stop();
        let t0 = Instant::now();
        runtime.wait();
        let elapsed = t0.elapsed();

        assert!(
            elapsed < Duration::from_millis(10),
            "wait should return immediately after request_stop, but took {elapsed:?}"
        );
        assert!(runtime.stop_requested());
    }

    #[test]
    fn request_stop_makes_wait_with_timeout_return_true() {
        let runtime = make_runtime();

        runtime.request_stop();
        let t0 = Instant::now();
        assert!(runtime.wait_with_timeout(Duration::from_secs(10)));
        let elapsed = t0.elapsed();

        assert!(
            elapsed < Duration::from_millis(10),
            "wait_with_timeout should return immediately after request_stop, but took {elapsed:?}"
        );
    }

    #[test]
    fn wait_does_not_wake_for_timer_without_running_runtime() {
        let runtime = make_runtime();
        let done = Arc::new(AtomicBool::new(false));
        let done_clone = done.clone();

        runtime.spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            done_clone.store(true, Ordering::Release);
        });

        // spawn() leaves the work-available flag true. Consume it first so that the
        // next wait() is actually waiting for a new signal, not the stale one.
        runtime.wait();

        // Park outside the runtime for 200ms. The timer deadline is 100ms, but the
        // current_thread runtime is not being driven, so the timer cannot fire.
        let start = Instant::now();
        assert!(
            !runtime.wait_with_timeout(Duration::from_millis(200)),
            "wait should time out because no new task was spawned"
        );
        let elapsed = start.elapsed();
        assert!(
            elapsed >= Duration::from_millis(180),
            "wait should have parked for the full 200ms (elapsed: {elapsed:?})"
        );

        // The timer task has not run yet because the runtime was not driven.
        assert!(
            !done.load(Ordering::Acquire),
            "timer should not fire while the runtime is parked outside make_progress"
        );

        // After the wait timeout, make_progress() must be called repeatedly to drive
        // the runtime enough to notice the expired timer and run the task to completion.
        let start = Instant::now();
        while !done.load(Ordering::Acquire) {
            runtime.make_progress();
            if start.elapsed() > Duration::from_secs(5) {
                break;
            }
        }
        assert!(
            done.load(Ordering::Acquire),
            "timer task should complete after make_progress drives the runtime"
        );
    }

    /// Viability check: a current_thread Tokio runtime can block_on a future that
    /// selects over multiple spawned tasks. Verifies both that the first ready task
    /// is returned promptly and that the remaining task is still driven to completion.
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
}
