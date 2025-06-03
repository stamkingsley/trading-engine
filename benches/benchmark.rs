use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use trading_engine::{TradeEvent, FlatBufCodec, LockFreeQueue};
use trading_engine::event::{EventType, OrderSide};

fn serialize_benchmark(c: &mut Criterion) {
    let codec = FlatBufCodec::new();
    let event = TradeEvent::trade_executed(
        "BTCUSDT".to_string(),
        50000.0,
        1.0,
        "order_123".to_string(),
        OrderSide::Buy,
        "user_456".to_string(),
        "binance".to_string(),
    );

    c.bench_function("serialize_trade_event", |b| {
        b.iter(|| {
            let result = codec.serialize(black_box(&event));
            black_box(result)
        })
    });
}

fn deserialize_benchmark(c: &mut Criterion) {
    let codec = FlatBufCodec::new();
    let event = TradeEvent::trade_executed(
        "BTCUSDT".to_string(),
        50000.0,
        1.0,
        "order_123".to_string(),
        OrderSide::Buy,
        "user_456".to_string(),
        "binance".to_string(),
    );
    let serialized = codec.serialize(&event).unwrap();

    c.bench_function("deserialize_trade_event", |b| {
        b.iter(|| {
            let result = codec.deserialize(black_box(&serialized));
            black_box(result)
        })
    });
}

fn queue_throughput_benchmark(c: &mut Criterion) {
    let queue = LockFreeQueue::<u64>::bounded(10000);
    
    let mut group = c.benchmark_group("queue_throughput");
    group.throughput(Throughput::Elements(1000));
    
    group.bench_function("enqueue_dequeue", |b| {
        b.iter(|| {
            for i in 0..1000 {
                queue.enqueue(black_box(i)).ok();
            }
            for _ in 0..1000 {
                black_box(queue.dequeue());
            }
        })
    });
    
    group.finish();
}

criterion_group!(benches, serialize_benchmark, deserialize_benchmark, queue_throughput_benchmark);
criterion_main!(benches);