use trading_engine::{TradeEvent, TradeEventProcessor, FlatBufCodec, LockFreeQueue};
use trading_engine::event::{EventType, OrderSide};
use trading_engine::utils::MetricsCollector;
use tokio::time::{sleep, Duration, Instant};
use tracing::{info, Level};
use tracing_subscriber;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .init();

    info!("Starting high-performance trading event processor");

    // 创建处理器
    let processor = Arc::new(TradeEventProcessor::new(10000));
    let codec = FlatBufCodec::new();
    let metrics = Arc::new(MetricsCollector::new());

    // 获取队列引用
    let input_queue = processor.get_input_queue();
    let output_queue = processor.get_output_queue();

    // 启动指标收集
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        metrics_clone.start_reporting(5).await;
    });

    // 启动处理器
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        processor_clone.start(num_cpus::get()).await;
    });

    // 生产者任务
    let input_queue_clone = input_queue.clone();
    let codec_clone = codec.clone();
    tokio::spawn(async move {
        let mut counter = 0u64;
        loop {
            let event = TradeEvent::trade_executed(
                format!("BTCUSDT"),
                50000.0 + (counter as f64 % 1000.0),
                1.0 + (counter as f64 % 10.0),
                format!("order_{}", counter),
                if counter % 2 == 0 { OrderSide::Buy } else { OrderSide::Sell },
                format!("user_{}", counter % 100),
                "binance".to_string(),
            );

            if let Ok(serialized) = codec_clone.serialize(&event) {
                if input_queue_clone.enqueue(serialized).is_err() {
                    // 队列满了，稍等
                    sleep(Duration::from_micros(1)).await;
                    continue;
                }
            }

            counter += 1;
            
            // 控制生产速度
            if counter % 1000 == 0 {
                sleep(Duration::from_millis(1)).await;
            }
        }
    });

    // 消费者任务
    let output_queue_clone = output_queue.clone();
    let metrics_clone = metrics.clone();
    tokio::spawn(async move {
        loop {
            let start = Instant::now();
            
            if let Some(event) = output_queue_clone.dequeue() {
                let processing_time = start.elapsed().as_nanos() as u64;
                metrics_clone.record_event(processing_time);
                
                // 模拟业务处理
                // 这里可以添加实际的交易逻辑
            } else {
                sleep(Duration::from_micros(10)).await;
            }
        }
    });

    // 运行一段时间后显示统计信息
    sleep(Duration::from_secs(30)).await;

    let (input_len, output_len, input_total, output_total) = processor.get_queue_stats();
    info!(
        "Final stats - Input queue: {}, Output queue: {}, Total input: {}, Total output: {}, Processed: {}, Errors: {}",
        input_len,
        output_len,
        input_total,
        output_total,
        processor.get_processed_count(),
        processor.get_error_count()
    );

    processor.stop();
    Ok(())
}