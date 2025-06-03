use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::time::interval;
use tracing::info;

#[derive(Clone)]
pub struct MetricsCollector {
    events_processed: Arc<AtomicUsize>,
    events_per_second: Arc<AtomicUsize>,
    total_latency_ns: Arc<AtomicU64>,
    latency_count: Arc<AtomicUsize>,
    start_time: Arc<Instant>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            events_processed: Arc::new(AtomicUsize::new(0)),
            events_per_second: Arc::new(AtomicUsize::new(0)),
            total_latency_ns: Arc::new(AtomicU64::new(0)),
            latency_count: Arc::new(AtomicUsize::new(0)),
            start_time: Arc::new(Instant::now()),
        }
    }

    pub fn record_event(&self, processing_time_ns: u64) {
        self.events_processed.fetch_add(1, Ordering::Relaxed);
        self.total_latency_ns.fetch_add(processing_time_ns, Ordering::Relaxed);
        self.latency_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_events_processed(&self) -> usize {
        self.events_processed.load(Ordering::Relaxed)
    }

    pub fn get_average_latency_ns(&self) -> f64 {
        let total = self.total_latency_ns.load(Ordering::Relaxed);
        let count = self.latency_count.load(Ordering::Relaxed);
        if count > 0 {
            total as f64 / count as f64
        } else {
            0.0
        }
    }

    pub fn get_throughput(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        let processed = self.events_processed.load(Ordering::Relaxed);
        if elapsed > 0.0 {
            processed as f64 / elapsed
        } else {
            0.0
        }
    }

    pub async fn start_reporting(&self, interval_secs: u64) {
        let mut interval = interval(Duration::from_secs(interval_secs));
        let mut last_count = 0;

        loop {
            interval.tick().await;
            
            let current_count = self.events_processed.load(Ordering::Relaxed);
            let events_in_interval = current_count - last_count;
            let eps = events_in_interval / interval_secs as usize;
            
            self.events_per_second.store(eps, Ordering::Relaxed);
            
            info!(
                "Metrics - Processed: {}, EPS: {}, Avg Latency: {:.2}μs, Throughput: {:.2}/s",
                current_count,
                eps,
                self.get_average_latency_ns() / 1000.0,
                self.get_throughput()
            );
            
            last_count = current_count;
        }
    }
}