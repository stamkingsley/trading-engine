use trading_engine::{TradeEvent, TradeEventProcessor, FlatBufCodec, LockFreeQueue};
use trading_engine::event::{EventType, OrderSide};
use tokio::time::{sleep, Duration};
use std::sync::Arc;

#[tokio::test]
async fn test_queue_operations() {
    let queue = LockFreeQueue::<i32>::bounded(100);
    
    // 测试入队
    assert!(queue.enqueue(1).is_ok());
    assert!(queue.enqueue(2).is_ok());
    assert_eq!(queue.len(), 2);
    
    // 测试出队
    assert_eq!(queue.dequeue(), Some(1));
    assert_eq!(queue.dequeue(), Some(2));
    assert_eq!(queue.dequeue(), None);
    assert!(queue.is_empty());
}

#[tokio::test]
async fn test_serialization_roundtrip() {
    let codec = FlatBufCodec::new();
    let original_event = TradeEvent::trade_executed(
        "ETHUSDT".to_string(),
        3000.0,
        2.5,
        "order_789".to_string(),
        OrderSide::Sell,
        "user_123".to_string(),
        "coinbase".to_string(),
    );

    // 序列化
    let serialized = codec.serialize(&original_event).unwrap();
    assert!(!serialized.is_empty());

    // 反序列化
    let deserialized = codec.deserialize(&serialized).unwrap();
    
    // 验证数据完整性
    assert_eq!(original_event.symbol, deserialized.symbol);
    assert_eq!(original_event.price, deserialized.price);
    assert_eq!(original_event.quantity, deserialized.quantity);
    assert_eq!(original_event.order_id, deserialized.order_id);
    assert_eq!(original_event.side, deserialized.side);
    assert_eq!(original_event.user_id, deserialized.user_id);
    assert_eq!(original_event.exchange_id, deserialized.exchange_id);
    assert_eq!(original_event.event_type, deserialized.event_type);
}

#[tokio::test]
async fn test_concurrent_queue_access() {
    let queue = Arc::new(LockFreeQueue::<u64>::bounded(1000));
    let producer_count = 4;
    let consumer_count = 2;
    let items_per_producer = 1000;
    
    let mut handles = Vec::new();
    
    // 启动生产者
    for producer_id in 0..producer_count {
        let queue_clone = queue.clone();
        let handle = tokio::spawn(async move {
            for i in 0..items_per_producer {
                let value = (producer_id * items_per_producer + i) as u64;
                while queue_clone.enqueue(value).is_err() {
                    tokio::task::yield_now().await;
                }
            }
        });
        handles.push(handle);
    }
    
    // 启动消费者
    let consumed_items = Arc::new(std::sync::Mutex::new(Vec::new()));
    for _ in 0..consumer_count {
        let queue_clone = queue.clone();
        let consumed_clone = consumed_items.clone();
        let handle = tokio::spawn(async move {
            let mut local_items = Vec::new();
            let total_expected = producer_count * items_per_producer;
            
            while local_items.len() < total_expected / consumer_count + 100 {
                if let Some(item) = queue_clone.dequeue() {
                    local_items.push(item);
                } else {
                    tokio::task::yield_now().await;
                }
            }
            
            let mut consumed = consumed_clone.lock().unwrap();
            consumed.extend(local_items);
        });
        handles.push(handle);
    }
    
    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }
    
    // 验证结果
    let consumed = consumed_items.lock().unwrap();
    assert!(consumed.len() >= producer_count * items_per_producer);
}

#[tokio::test]
async fn test_event_processor_integration() {
    let processor = Arc::new(TradeEventProcessor::new(1000));
    let codec = FlatBufCodec::new();
    
    let input_queue = processor.get_input_queue();
    let output_queue = processor.get_output_queue();
    
    // 启动处理器
    let processor_clone = processor.clone();
    tokio::spawn(async move {
        processor_clone.start(2).await;
    });
    
    // 等待处理器启动
    sleep(Duration::from_millis(100)).await;
    
    // 发送测试事件
    let test_events = vec![
        TradeEvent::order_placed(
            "BTCUSDT".to_string(),
            50000.0,
            1.0,
            "order_1".to_string(),
            OrderSide::Buy,
            "user_1".to_string(),
            "binance".to_string(),
        ),
        TradeEvent::trade_executed(
            "ETHUSDT".to_string(),
            3000.0,
            2.0,
            "order_2".to_string(),
            OrderSide::Sell,
            "user_2".to_string(),
            "coinbase".to_string(),
        ),
    ];
    
    for event in &test_events {
        let serialized = codec.serialize(event).unwrap();
        input_queue.enqueue(serialized).unwrap();
    }
    
    // 等待处理完成
    sleep(Duration::from_millis(500)).await;
    
    // 验证输出
    let mut processed_events = Vec::new();
    while let Some(event) = output_queue.dequeue() {
        processed_events.push(event);
    }
    
    assert_eq!(processed_events.len(), test_events.len());
    assert!(processor.get_processed_count() >= test_events.len());
    
    processor.stop();
}

#[test]
fn test_event_creation() {
    let event = TradeEvent::new(
        EventType::OrderPlaced,
        "ADAUSDT".to_string(),
        1.0,
        100.0,
        "order_456".to_string(),
        OrderSide::Buy,
        "user_789".to_string(),
        "kraken".to_string(),
    );
    
    assert!(!event.event_id.is_empty());
    assert!(event.timestamp > 0);
    assert_eq!(event.event_type, EventType::OrderPlaced);
    assert_eq!(event.symbol, "ADAUSDT");
    assert_eq!(event.price, 1.0);
    assert_eq!(event.quantity, 100.0);
}