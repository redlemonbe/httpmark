use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;

use hdrhistogram::Histogram;

pub struct Stats {
    pub requests:   AtomicU64,
    pub errors:     AtomicU64,
    pub bytes_recv: AtomicU64,
    pub status_2xx: AtomicU64,
    pub status_3xx: AtomicU64,
    pub status_4xx: AtomicU64,
    pub status_5xx: AtomicU64,
    pub stopped:    AtomicBool,
    histogram: Mutex<Histogram<u64>>,
    // Interval snapshot counters (reset on each read)
    interval_reqs:   AtomicU64,
    interval_bytes:  AtomicU64,
    interval_errors: AtomicU64,
    interval_2xx:    AtomicU64,
}

pub struct Snapshot {
    pub requests:   u64,
    pub errors:     u64,
    pub bytes_recv: u64,
    pub status_2xx: u64,
    pub status_3xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
    pub interval_reqs:   u64,
    pub interval_bytes:  u64,
    pub interval_errors: u64,
    pub interval_2xx:    u64,
}

pub struct FinalStats {
    pub requests:   u64,
    pub errors:     u64,
    pub bytes_recv: u64,
    pub status_2xx: u64,
    pub status_3xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
    pub p50_us:  u64,
    pub p90_us:  u64,
    pub p95_us:  u64,
    pub p99_us:  u64,
    pub p999_us: u64,
    pub max_us:  u64,
    pub mean_us: f64,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            requests:        AtomicU64::new(0),
            errors:          AtomicU64::new(0),
            bytes_recv:      AtomicU64::new(0),
            status_2xx:      AtomicU64::new(0),
            status_3xx:      AtomicU64::new(0),
            status_4xx:      AtomicU64::new(0),
            status_5xx:      AtomicU64::new(0),
            stopped:         AtomicBool::new(false),
            histogram:       Mutex::new(Histogram::new(3).unwrap()),
            interval_reqs:   AtomicU64::new(0),
            interval_bytes:  AtomicU64::new(0),
            interval_errors: AtomicU64::new(0),
            interval_2xx:    AtomicU64::new(0),
        }
    }

    pub fn record(&self, latency_us: u64, bytes: u64, status: u16) {
        self.requests.fetch_add(1, Ordering::Relaxed);
        self.bytes_recv.fetch_add(bytes, Ordering::Relaxed);
        self.interval_reqs.fetch_add(1, Ordering::Relaxed);
        self.interval_bytes.fetch_add(bytes, Ordering::Relaxed);

        match status {
            200..=299 => { self.status_2xx.fetch_add(1, Ordering::Relaxed); self.interval_2xx.fetch_add(1, Ordering::Relaxed); }
            300..=399 => { self.status_3xx.fetch_add(1, Ordering::Relaxed); }
            400..=499 => { self.status_4xx.fetch_add(1, Ordering::Relaxed); }
            500..=599 => { self.status_5xx.fetch_add(1, Ordering::Relaxed); }
            _ => {}
        }

        if let Ok(mut h) = self.histogram.lock() {
            let _ = h.record(latency_us.max(1));
        }
    }

    pub fn record_error(&self) {
        self.errors.fetch_add(1, Ordering::Relaxed);
        self.interval_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Relaxed)
    }

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            requests:        self.requests.load(Ordering::Relaxed),
            errors:          self.errors.load(Ordering::Relaxed),
            bytes_recv:      self.bytes_recv.load(Ordering::Relaxed),
            status_2xx:      self.status_2xx.load(Ordering::Relaxed),
            status_3xx:      self.status_3xx.load(Ordering::Relaxed),
            status_4xx:      self.status_4xx.load(Ordering::Relaxed),
            status_5xx:      self.status_5xx.load(Ordering::Relaxed),
            interval_reqs:   self.interval_reqs.swap(0, Ordering::Relaxed),
            interval_bytes:  self.interval_bytes.swap(0, Ordering::Relaxed),
            interval_errors: self.interval_errors.swap(0, Ordering::Relaxed),
            interval_2xx:    self.interval_2xx.swap(0, Ordering::Relaxed),
        }
    }

    pub fn final_snapshot(&self) -> FinalStats {
        let h = self.histogram.lock().unwrap();
        FinalStats {
            requests:   self.requests.load(Ordering::Relaxed),
            errors:     self.errors.load(Ordering::Relaxed),
            bytes_recv: self.bytes_recv.load(Ordering::Relaxed),
            status_2xx: self.status_2xx.load(Ordering::Relaxed),
            status_3xx: self.status_3xx.load(Ordering::Relaxed),
            status_4xx: self.status_4xx.load(Ordering::Relaxed),
            status_5xx: self.status_5xx.load(Ordering::Relaxed),
            p50_us:  h.value_at_percentile(50.0),
            p90_us:  h.value_at_percentile(90.0),
            p95_us:  h.value_at_percentile(95.0),
            p99_us:  h.value_at_percentile(99.0),
            p999_us: h.value_at_percentile(99.9),
            max_us:  h.max(),
            mean_us: h.mean(),
        }
    }
}
