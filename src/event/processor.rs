use crate::event::TradeEvent;
use crate::queue::LockFreeQueue;
use crate::serialization::FlatBufCodec;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

pub struct TradeEventProcessor {
    input_queue: LockFreeQueue<Vec<u8>>,
    output_queue: LockFreeQueue<TradeEvent>,
    codec: FlatBufCodec,
    processed_count: Arc<AtomicUsize>,
    error_count: Arc<AtomicUsize>,
    running: Arc<AtomicBool>,
}

impl TradeEventProcessor {
    pub fn new(queue_capacity: usize) -> Self {
        Self {
            input_queue: LockFreeQueue::bounded(queue_capacity),
            output_queue: LockFreeQueue::bounded(queue_capacity),
            codec: FlatBufCodec::new(),
            processed_count: Arc::new(AtomicUsize::new(0)),
            error_count: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn get_input_queue(&self) -> LockFreeQueue<Vec<u8>> {
        self.input_queue.clone()
    }

    pub fn get_output_queue(&self) -> LockFreeQueue<TradeEvent> {
        self.output_queue.clone()
    }

    pub async fn start(&self, worker_count: usize) {
        self.running.store(true, Ordering::SeqCst);
        info!("Starting trade event processor with {} workers", worker_count);

        let mut handles = Vec::new();

        for worker_id in 0..worker_count {
            let input_queue = self.input_queue.clone();
            let output_queue = self.output_queue.clone();
            let codec = self.codec.clone();
            let processed_count = self.processed_count.clone();
            let error_count = self.error_count.clone();
            let running = self.running.clone();

            let handle = tokio::spawn(async move {
                Self::worker_loop(
                    worker_id,
                    input_queue,
                    output_queue,
                    codec,
                    processed_count,
                    error_count,
                    running,
                ).await;
            });

            handles.push(handle);
        }

        // 等待所有worker完成
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Worker task failed: {}", e);
            }
        }
    }

    async fn worker_loop(
        worker_id: usize,
        input_queue: LockFreeQueue<Vec<u8>>,
        output_queue: LockFreeQueue<TradeEvent>,
        codec: FlatBufCodec,
        processed_count: Arc<AtomicUsize>,
        error_count: Arc<AtomicUsize>,
        running: Arc<AtomicBool>,
    ) {
        info!("Worker {} started", worker_id);

        while running.load(Ordering::SeqCst) {
            if let Some(data) = input_queue.dequeue() {
                match codec.deserialize(&data) {
                    Ok(event) => {
                        if let Err(_) = output_queue.enqueue(event) {
                            warn!("Worker {}: Output queue full, dropping event", worker_id);
                        } else {
                            processed_count.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(e) => {
                        error!("Worker {}: Failed to deserialize event: {}", worker_id, e);
                        error_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            } else {
                // 队列为空，短暂休眠
                sleep(Duration::from_micros(10)).await;
            }
        }

        info!("Worker {} stopped", worker_id);
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::SeqCst);
        info!("Trade event processor stopped");
    }

    pub fn get_processed_count(&self) -> usize {
        self.processed_count.load(Ordering::Relaxed)
    }

    pub fn get_error_count(&self) -> usize {
        self.error_count.load(Ordering::Relaxed)
    }

    pub fn get_queue_stats(&self) -> (usize, usize, usize, usize) {
        (
            self.input_queue.len(),
            self.output_queue.len(),
            self.input_queue.enqueue_count(),
            self.output_queue.dequeue_count(),
        )
    }
}