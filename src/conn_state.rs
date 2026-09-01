use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

use coarsetime::Instant;
use tokio::sync::Notify;

#[derive(Debug)]
pub struct ConnState {
    last_activity: AtomicU64,
    shutdown_notify: Arc<Notify>,
}

impl ConnState {
    #[tracing::instrument]
    pub fn new(shutdown_notify: Arc<Notify>) -> ConnState {
        ConnState {
            last_activity: AtomicU64::new(coarsetime::Instant::recent().as_ticks()),
            shutdown_notify,
        }
    }

    #[tracing::instrument]
    pub fn touch(&self) {
        self.last_activity
            .store(coarsetime::Instant::recent().as_ticks(), Ordering::Relaxed);
    }

    #[tracing::instrument]
    pub fn is_timed_out(&self, threshold_ms: u64) -> bool {
        let last_instant = Instant::from_ticks(self.last_activity.load(Ordering::Relaxed));
        last_instant.elapsed_since_recent().as_millis() > threshold_ms
    }

    #[tracing::instrument]
    pub fn shutdown(&self) {
        self.shutdown_notify.notify_one();
    }
}
