use std::sync::OnceLock;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct ClockOrigin {
    instant: Instant,
    unix_ms: u64,
}

fn origin() -> &'static ClockOrigin {
    static ORIGIN: OnceLock<ClockOrigin> = OnceLock::new();
    ORIGIN.get_or_init(|| ClockOrigin {
        instant: Instant::now(),
        unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before UNIX_EPOCH")
            .as_millis() as u64,
    })
}

pub fn instant_to_unix_ms(t: Instant) -> u64 {
    let o = origin();
    if t >= o.instant {
        o.unix_ms + (t - o.instant).as_millis() as u64
    } else {
        o.unix_ms.saturating_sub((o.instant - t).as_millis() as u64)
    }
}

pub fn unix_ms_to_instant(ms: u64) -> Instant {
    let o = origin();
    if ms >= o.unix_ms {
        o.instant + Duration::from_millis(ms - o.unix_ms)
    } else {
        o.instant - Duration::from_millis(o.unix_ms - ms)
    }
}
