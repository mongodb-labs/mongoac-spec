use crate::safe_as_ref;
use crate::safe_drop;

use parking_lot::{Condvar, Mutex};
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RuntimeT {
    state: Arc<RuntimeState>,
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_destroy(runtime: *mut RuntimeT) {
    safe_drop!(runtime);
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

impl RuntimeT {
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            state: Arc::new(RuntimeState::new(runtime, Mutex::new(()))),
        })
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
        let mut guard = self.state.spawn_mut.lock(); // TODO: replace with non-blocking alternative.
        *guard = true;
        self.state.spawn_cv.notify_one();
        handle
    }

    pub(crate) fn block_on<F: Future>(&self, future: F) -> F::Output {
        let _guard = self.state.progress_lock.lock();
        self.state.runtime.block_on(future)
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

    pub(crate) fn wait(&self) {
        let _ = self.wait_impl(None);
    }

    pub(crate) fn wait_with_timeout(&self, timeout: Duration) -> bool {
        self.wait_impl(Some(timeout))
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
        let mut guard = self.state.spawn_mut.lock();

        // Loop while no work is available.
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
}
