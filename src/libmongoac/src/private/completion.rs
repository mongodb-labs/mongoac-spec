//! Safe, allocation-free (beyond the initial `Arc`) condvar-backed waker for awaiting
//! future completion without CPU spinning.
//!
//! Uses `std::task::Wake` (stable since Rust 1.51) instead of manual `RawWakerVTable`,
//! eliminating all `unsafe` code that would otherwise be required for waker construction.
//! The standard library handles the vtable internally via `From<Arc<W>> for Waker`.

use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Wake, Waker};
use std::time::Duration;

struct Inner {
    ready: AtomicBool,
    condvar: Condvar,
    mutex: Mutex<()>,
}

impl Inner {
    /// Marks the waker as ready and notifies any parked waiter.
    ///
    /// Must be called while holding the mutex so that `wait()` cannot
    /// check the flag and park on the condvar between the store and the
    /// notification, which would lose the wakeup.
    fn notify_ready(&self) {
        let _guard = self.mutex.lock();
        self.ready.store(true, Ordering::Release);
        self.condvar.notify_one();
    }
}

impl Wake for Inner {
    fn wake(self: Arc<Self>) {
        self.notify_ready();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.notify_ready();
    }
}

/// A condvar-backed completion signal.
///
/// Designed for the one-caller-one-signal pattern used by
/// `mongoac_future_wait()` and `mongoac_future_wait_with_timeout()`:
/// a single thread calls `wait()`, and a waker (registered via
/// `FutureValueType::poll_with_context`) signals the condvar when the
/// underlying future completes.
pub(crate) struct CompletionWaker {
    inner: Arc<Inner>,
}

impl CompletionWaker {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(Inner {
                ready: AtomicBool::new(false),
                condvar: Condvar::new(),
                mutex: Mutex::new(()),
            }),
        }
    }

    /// Returns a `Waker` that signals this `CompletionWaker` when woken.
    ///
    /// Calling this method is cheap: it clones the inner `Arc` and delegates to
    /// `Waker::from()`, which performs a direct `RawWaker` construction with no
    /// heap allocations or atomic operations beyond the `Arc::clone`.
    pub(crate) fn waker(&self) -> Waker {
        Waker::from(self.inner.clone())
    }

    /// Blocks until the waker fires or the optional timeout expires.
    ///
    /// Returns `true` if the waker fired (the future is ready), `false` if the
    /// timeout expired first.
    ///
    /// Uses a double-checked locking pattern: consumes the `ready` flag with
    /// an atomic `swap(false, AcqRel)` at each check point. This ensures the
    /// flag is consumed by each `wait()` call so that a subsequent call will
    /// correctly park on the condvar rather than busy-waiting on a stale `true`
    /// flag. The shared `notify_ready()` helper acquires the same mutex before
    /// setting the flag and notifying the condvar, preventing lost-wakeup races
    /// on reuse.
    pub(crate) fn wait(&self, timeout: Option<Duration>) -> bool {
        // Fast path: atomically consume the ready flag.
        if self.inner.ready.swap(false, Ordering::AcqRel) {
            return true;
        }

        let mut guard = self.inner.mutex.lock();

        // Double-check under the lock: a wake() that ran between the fast path
        // and the mutex acquire set the flag and was blocked waiting for the
        // mutex, so the ready flag is visible once we hold the lock.
        if self.inner.ready.swap(false, Ordering::AcqRel) {
            return true;
        }

        match timeout {
            Some(dur) => {
                self.inner.condvar.wait_for(&mut guard, dur);
            }
            None => {
                self.inner.condvar.wait(&mut guard);
            }
        }

        // After waking (spurious or real), atomically consume the ready flag.
        self.inner.ready.swap(false, Ordering::AcqRel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;
    use std::time::Duration;

    /// Blocks until `flag` becomes true. The waiter thread must call
    /// `main_thread.unpark()` after setting the flag to wake this thread.
    /// Using `thread::park()` instead of a `yield_now()` spin loop avoids
    /// non-deterministic scheduler behavior and deadlocks under Miri.
    fn wait_for_flag(flag: &AtomicBool) {
        while !flag.load(Ordering::Acquire) {
            thread::park();
        }
    }

    /// Spawns a thread that calls `cw.wait(timeout)` and blocks the current
    /// thread until the waiter has reached the point right before `wait()`.
    /// Returns the join handle so the caller can fire the waker and join.
    fn spawn_waiter(
        cw: &Arc<CompletionWaker>,
        timeout: Option<Duration>,
    ) -> thread::JoinHandle<bool> {
        let cw_clone = cw.clone();
        let waiting = Arc::new(AtomicBool::new(false));
        let waiting_clone = waiting.clone();
        let main_thread = thread::current();

        let waiter = thread::spawn(move || {
            waiting_clone.store(true, Ordering::Release);
            main_thread.unpark();
            cw_clone.wait(timeout)
        });

        wait_for_flag(&waiting);
        waiter
    }

    #[test]
    fn new_returns_unready() {
        let cw = CompletionWaker::new();
        assert!(!cw.inner.ready.load(Ordering::Acquire));
    }

    #[test]
    fn waker_triggers_ready() {
        let cw = CompletionWaker::new();
        let waker = cw.waker();
        waker.wake();
        assert!(cw.inner.ready.load(Ordering::Acquire));
    }

    #[test]
    fn wait_returns_immediately_when_already_ready() {
        let cw = CompletionWaker::new();
        let waker = cw.waker();
        waker.wake();
        assert!(cw.wait(None));
    }

    #[test]
    fn wait_blocks_until_wake() {
        let cw = Arc::new(CompletionWaker::new());
        let waiter = spawn_waiter(&cw, None);

        let waker = cw.waker();
        waker.wake();

        assert!(waiter.join().unwrap());
    }

    #[test]
    fn wait_with_timeout_expires() {
        let cw = CompletionWaker::new();
        assert!(!cw.wait(Some(Duration::from_millis(10))));
    }

    #[test]
    fn wait_with_timeout_wakes_before_expiry() {
        let cw = Arc::new(CompletionWaker::new());
        let waiter = spawn_waiter(&cw, Some(Duration::from_secs(5)));

        let waker = cw.waker();
        waker.wake();

        assert!(waiter.join().unwrap());
    }

    #[test]
    fn reuse_after_wake() {
        // Multiple wake-wait cycles with the same CompletionWaker.
        // wait() consumes the ready flag via swap(false, AcqRel), so
        // no manual reset is needed between cycles.
        let cw = Arc::new(CompletionWaker::new());

        for _ in 0..3 {
            let waiter = spawn_waiter(&cw, None);

            let waker = cw.waker();
            waker.wake();

            assert!(waiter.join().unwrap());
        }
    }
}
