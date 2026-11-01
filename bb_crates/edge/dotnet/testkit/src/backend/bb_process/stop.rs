use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// Shared signal to stop a running test and its child processes.
#[derive(Clone, Debug, Default)]
pub struct StopFlag(Arc<AtomicBool>);

impl StopFlag {
    /// Create an unset signal.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stop the run.
    pub fn stop(&self) {
        self.0.store(true, Ordering::Release);
    }

    /// Whether the caller asked the run to stop.
    #[must_use]
    pub fn stopped(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }

    pub(crate) async fn wait(&self) {
        while !self.stopped() {
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}
