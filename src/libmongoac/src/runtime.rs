use crate::error::ErrorT;
use crate::future::FutureT;
use crate::safe_as_mut;
use crate::safe_as_ref;
use crate::safe_drop;
use crate::safe_optional_as_mut;
use crate::safe_optional_error_as_mut;

use parking_lot::{Condvar, Mutex};
use std::future::{Future, poll_fn};
use std::sync::Arc;
use std::task::Poll;
use std::time::{Duration, Instant};

#[macro_export]
macro_rules! safe_from_runtime_with_error {
    ($future:expr, $runtime:expr, $error:expr) => {{
        if !$future.from_runtime($runtime) {
            $crate::private::safety::invalid_argument(
                $error,
                concat!("future is not associated with the given runtime"),
            );
            return Default::default();
        }
        $future
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
pub extern "C" fn mongoac_runtime_notify_one(runtime: *mut RuntimeT) {
    safe_as_ref!(runtime).notify_one()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_notify_all(runtime: *mut RuntimeT) {
    safe_as_ref!(runtime).notify_all()
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
    future: *mut FutureT,
    error: *mut ErrorT,
) {
    let error = safe_optional_error_as_mut!(error);
    let future = safe_from_runtime_with_error!(safe_as_mut!(future), safe_as_mut!(runtime), error);

    future.block_on();
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_any<'a>(
    runtime: *mut RuntimeT,
    futures: *mut *mut FutureT,
    count: usize,
    error: *mut ErrorT,
) -> *mut FutureT {
    let error = safe_optional_error_as_mut!(error);
    let runtime = safe_as_mut!(runtime);
    let futures = safe_as_mut!(futures);

    if count == 0 {
        return std::ptr::null_mut();
    }

    // Precondition: valid range.
    let slice = unsafe { std::slice::from_raw_parts_mut(futures, count) };
    let mut futures: Vec<Option<&mut FutureT>> = Vec::with_capacity(count);
    for ptr in slice.iter_mut() {
        if let Some(f) = safe_optional_as_mut!(*ptr) {
            futures.push(Some(safe_from_runtime_with_error!(f, runtime, error)));
        } else {
            futures.push(None);
        }
    }

    runtime
        .block_on_any(&mut futures)
        .map(|f| f as *mut FutureT)
        .unwrap_or(std::ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_block_on_all(
    runtime: *mut RuntimeT,
    futures: *mut *mut FutureT,
    count: usize,
) {
    let runtime = safe_as_ref!(runtime);
    let futures = safe_as_mut!(futures);

    if count == 0 {
        return;
    }

    // Precondition: valid range.
    let slice = unsafe { std::slice::from_raw_parts_mut(futures, count) };
    let mut futures: Vec<Option<&mut FutureT>> = Vec::with_capacity(count);
    for ptr in slice.iter_mut() {
        if let Some(f) = safe_optional_as_mut!(*ptr) {
            futures.push(Some(f));
        } else {
            futures.push(None);
        }
    }

    runtime.block_on_all(&mut futures);
}

struct RuntimeState {
    runtime: tokio::runtime::Runtime,
    progress_lock: Mutex<()>,
    spawn_mut: Mutex<bool>,
    spawn_cv: Condvar,
}

impl RuntimeState {
    fn new(runtime: tokio::runtime::Runtime, progress_lock: Mutex<()>) -> Self {
        Self {
            runtime,
            progress_lock,
            spawn_mut: Mutex::new(false),
            spawn_cv: Condvar::new(),
        }
    }
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
            state: Arc::new(RuntimeState::new(runtime, Mutex::new(()))),
        })
    }

    pub(crate) fn make_progress(&self) -> bool {
        self.with_progress_lock(|runtime| {
            runtime.block_on(async { tokio::task::yield_now().await });
        })
    }

    pub(crate) fn make_progress_with_timeout(&self, timeout: Duration) -> bool {
        self.with_progress_lock(|runtime| {
            runtime.block_on(async {
                let _ = tokio::time::timeout(timeout, tokio::task::yield_now()).await;
            });
        })
    }

    fn notify_one(&self) {
        {
            let mut guard = self.state.spawn_mut.lock();
            *guard = true;
        }
        self.state.spawn_cv.notify_one();
    }

    fn notify_all(&self) {
        {
            let mut guard = self.state.spawn_mut.lock();
            *guard = true;
        }
        self.state.spawn_cv.notify_all();
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

    pub(crate) fn block_on_any<'a>(
        &self,
        futures: &'a mut [Option<&'a mut FutureT>],
    ) -> Option<&'a mut FutureT> {
        let _guard = self.state.progress_lock.lock();
        self.state.runtime.block_on(poll_fn(|ctx| {
            for f in futures.iter_mut() {
                if let Some(future) = f.take() {
                    if future.poll_with_context(ctx) {
                        return Poll::Ready(Some(future));
                    }
                    *f = Some(future);
                }
            }
            Poll::Pending
        }))
    }

    pub(crate) fn block_on_all(&self, futures: &mut [Option<&mut FutureT>]) {
        let _guard = self.state.progress_lock.lock();
        self.state.runtime.block_on(poll_fn(|ctx| {
            for f in futures.iter_mut() {
                if let Some(future) = f {
                    if !future.poll_with_context(ctx) {
                        return Poll::Pending;
                    }
                }
            }
            Poll::Ready(())
        }));
    }

    pub(crate) fn from_raw(runtime: tokio::runtime::Runtime) -> Self {
        Self {
            state: Arc::new(RuntimeState::new(runtime, Mutex::new(()))),
        }
    }

    pub(crate) fn get_runtime(&self) -> &tokio::runtime::Runtime {
        &self.state.runtime
    }

    pub(crate) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        let handle = self.state.runtime.spawn(future);

        // IMPORTANT: this guard MAY be blocked by other spawners or by a worker thread, but the lock is held for a very
        // short time in all cases, so this is *effectively* non-blocking in most scenarios. Nevertheless, it may need
        // to be replaced with a semaphores or channels (truly non-blocking) if spawner contention becomes a measurable
        // bottleneck.
        {
            let mut guard = self.state.spawn_mut.lock();
            *guard = true;
        }
        self.notify_one();

        handle
    }

    fn with_progress_lock<F>(&self, func: F) -> bool
    where
        F: FnOnce(&tokio::runtime::Runtime),
    {
        let Some(_guard) = self.state.progress_lock.try_lock() else {
            return false;
        };

        func(&self.state.runtime);
        true
    }

    fn wait_impl(&self, timeout: Option<Duration>) -> bool {
        let deadline = timeout.map(|t| Instant::now() + t);

        // IMPORTANT: this guard MUST be held for as short as possible to avoid blocking spawning threads.
        let mut guard = self.state.spawn_mut.lock();
        while !*guard {
            match deadline {
                None => self.state.spawn_cv.wait(&mut guard),
                Some(deadline) => {
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        return false; // Early return on timeout.
                    }

                    if self
                        .state
                        .spawn_cv
                        .wait_for(&mut guard, remaining)
                        .timed_out()
                    {
                        return false; // Early return on timeout.
                    }
                }
            }
        }

        // Reset after successful wakeup.
        *guard = false;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::{Duration, Instant};

    fn make_runtime() -> RuntimeT {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .event_interval(1)
            .build()
            .expect("failed to build runtime");

        RuntimeT::from_raw(rt)
    }

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
    fn notify_all_only_unblocks_one_worker_with_bool_predicate() {
        let runtime = make_runtime();
        let num_workers = 3;
        let mut handles = Vec::new();

        for _ in 0..num_workers {
            let rt = runtime.clone();
            handles.push(thread::spawn(move || {
                rt.wait_with_timeout(Duration::from_secs(1))
            }));
        }

        // Give workers time to start waiting on the runtime.
        thread::sleep(Duration::from_millis(50));

        runtime.notify_all();

        let unblocked = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|result| *result)
            .count();

        assert_eq!(
            unblocked, 1,
            "with a Mutex<bool> predicate, notify_all wakes all threads but only one consumes the flag; the rest time out"
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
