use crate::runtime::RuntimeT;
use std::time::Duration;
use tokio::runtime;

/// Build a single-threaded Tokio runtime configured for responsive tests.
pub(crate) fn make_runtime() -> RuntimeT {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .event_interval(1)
        .build()
        .expect("failed to build runtime");
    RuntimeT::from_raw(rt)
}

/// Build a single-threaded Tokio runtime with a custom per-tick interval.
#[allow(dead_code)]
pub(crate) fn make_runtime_with_event_interval(interval: Duration) -> RuntimeT {
    let rt = runtime::Builder::new_current_thread()
        .enable_all()
        .event_interval(interval.as_millis() as u32)
        .build()
        .expect("failed to build runtime");
    RuntimeT::from_raw(rt)
}
