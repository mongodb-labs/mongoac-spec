use crate::safe_as_ref;
use crate::safe_drop;

use parking_lot::Mutex;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct RuntimeT {
    runtime: Arc<tokio::runtime::Runtime>,
    progress_lock: Arc<Mutex<()>>,
}

impl RuntimeT {
    pub fn new() -> Result<Self, std::io::Error> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;

        Ok(Self {
            runtime: Arc::new(runtime),
            progress_lock: Arc::new(Mutex::new(())),
        })
    }

    pub(crate) fn from_raw(
        runtime: Arc<tokio::runtime::Runtime>,
        progress_lock: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            runtime,
            progress_lock,
        }
    }

    pub(crate) fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: std::future::Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime.spawn(future)
    }

    pub fn block_on<F: Future>(&self, f: F) -> F::Output {
        let _guard = self.progress_lock.lock();
        self.runtime.block_on(f)
    }

    pub fn make_progress(&self) -> bool {
        let _guard = match self.progress_lock.try_lock() {
            Some(guard) => guard,
            None => return false,
        };

        self.runtime.block_on(async move {
            tokio::task::yield_now().await;
        });

        true
    }

    pub fn make_progress_with_timeout(&self, timeout: Duration) -> bool {
        let _guard = match self.progress_lock.try_lock() {
            Some(guard) => guard,
            None => return false,
        };

        self.runtime.block_on(async move {
            let _ = tokio::time::timeout(timeout, async {
                tokio::task::yield_now().await;
            })
            .await;
        });

        true
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress(runtime: *mut RuntimeT) -> bool {
    let runtime = safe_as_ref!(runtime);

    runtime.make_progress()
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_make_progress_with_timeout(
    runtime: *mut RuntimeT,
    timeout_ms: u64,
) -> bool {
    let runtime = safe_as_ref!(runtime);
    let timeout = Duration::from_millis(timeout_ms);

    runtime.make_progress_with_timeout(timeout)
}

#[unsafe(no_mangle)]
pub extern "C" fn mongoac_runtime_destroy(runtime: *mut RuntimeT) {
    safe_drop!(runtime);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn make_runtime() -> RuntimeT {
        let rt = Arc::new(
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .event_interval(1)
                .build()
                .expect("failed to build runtime"),
        );
        let lock = Arc::new(Mutex::new(()));

        RuntimeT::from_raw(rt, lock)
    }

    fn spawn_tasks(runtime: &RuntimeT, count: usize) {
        for _ in 0..count {
            runtime.runtime.spawn(async {
                let t = Instant::now() + Duration::from_millis(1);
                while Instant::now() < t {
                    std::hint::spin_loop();
                }
                tokio::task::yield_now().await;
            });
        }
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

        spawn_tasks(&runtime, 100);

        let started = Instant::now();
        let _ = runtime.make_progress_with_timeout(Duration::from_millis(50));
        let elapsed = started.elapsed();

        assert!(
            elapsed < Duration::from_millis(75),
            "expected timeout to bound block_on, but call took {elapsed:?}"
        );
    }
}
