use std::sync::atomic::{AtomicU64, Ordering};

use coarsetime::Instant;
use tokio::sync::mpsc;

pub struct ConnState {
    last_activity: AtomicU64,
    shutdown_tx: mpsc::Sender<()>,
}

impl ConnState {
    pub fn new(shutdown_tx: mpsc::Sender<()>) -> ConnState {
        ConnState {
            last_activity: AtomicU64::new(coarsetime::Instant::recent().as_ticks()),
            shutdown_tx,
        }
    }

    pub fn touch(&self) {
        self.last_activity
            .store(coarsetime::Instant::recent().as_ticks(), Ordering::Relaxed);
    }

    pub fn is_timed_out(&self, threshold_ms: u64) -> bool {
        let last_instant = Instant::from_ticks(self.last_activity.load(Ordering::Relaxed));
        last_instant.elapsed_since_recent().as_millis() > threshold_ms
    }

    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.try_send(());
    }
}
