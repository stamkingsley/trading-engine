use crate::event::{TradeEvent, EventType, OrderSide};
use flatbuffers::{FlatBufferBuilder, WIPOffset};
use std::error::Error;
use std::fmt;

// 引入生成的FlatBuffers代码
include!(concat!(env!("OUT_DIR"), "/trade_event_generated.rs"));

#[derive(Debug)]
pub struct SerializationError {
    message: String,
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Serialization error: {}", self.message)
    }
}

impl Error for SerializationError {}

#[derive(Clone)]
pub struct FlatBufCodec;

impl FlatBufCodec {
    pub fn new() -> Self {
        Self
    }

    pub fn serialize(&self, event: &TradeEvent) -> Result<Vec<u8>, SerializationError> {
        let mut builder = FlatBufferBuilder::new();

        // 创建字符串
        let event_id = builder.create_string(&event.event_id);
        let symbol = builder.create_string(&event.symbol);
        let order_id = builder.create_string(&event.order_id);
        let user_id = builder.create_string(&event.user_id);
        let exchange_id = builder.create_string(&event.exchange_id);

        // 转换枚举类型
        let event_type = match event.event_type {
            EventType::OrderPlaced => trade_events::EventType::OrderPlaced,
            EventType::OrderCancelled => trade_events::EventType::OrderCancelled,
            EventType::OrderFilled => trade_events::EventType::OrderFilled,
            EventType::TradeExecuted => trade_events::EventType::TradeExecuted,
            EventType::PriceUpdate => trade_events::EventType::PriceUpdate,
        };

        let side = match event.side {
            OrderSide::Buy => trade_events::OrderSide::Buy,
            OrderSide::Sell => trade_events::OrderSide::Sell,
        };

        // 创建TradeEvent
        let trade_event = trade_events::TradeEvent::create(&mut builder, &trade_events::TradeEventArgs {
            event_id: Some(event_id),
            timestamp: event.timestamp,
            event_type,
            symbol: Some(symbol),
            price: event.price,
            quantity: event.quantity,
            order_id: Some(order_id),
            side,
            user_id: Some(user_id),
            exchange_id: Some(exchange_id),
        });

        builder.finish(trade_event, None);
        Ok(builder.finished_data().to_vec())
    }

    pub fn deserialize(&self, data: &[u8]) -> Result<TradeEvent, SerializationError> {
        let trade_event = trade_events::root_as_trade_event(data)
            .map_err(|e| SerializationError {
                message: format!("Failed to parse FlatBuffer: {}", e),
            })?;

        // 转换枚举类型
        let event_type = match trade_event.event_type() {
            trade_events::EventType::OrderPlaced => EventType::OrderPlaced,
            trade_events::EventType::OrderCancelled => EventType::OrderCancelled,
            trade_events::EventType::OrderFilled => EventType::OrderFilled,
            trade_events::EventType::TradeExecuted => EventType::TradeExecuted,
            trade_events::EventType::PriceUpdate => EventType::PriceUpdate,
            _ => {
                return Err(SerializationError {
                    message: format!("Unknown event type: {:?}", trade_event.event_type()),
                });
            }
        };

        let side = match trade_event.side() {
            trade_events::OrderSide::Buy => OrderSide::Buy,
            trade_events::OrderSide::Sell => OrderSide::Sell,
            _ => {
                return Err(SerializationError {
                    message: format!("Unknown order side: {:?}", trade_event.side()),
                });
            }
        };

        Ok(TradeEvent {
            event_id: trade_event.event_id().unwrap_or("").to_string(),
            timestamp: trade_event.timestamp(),
            event_type,
            symbol: trade_event.symbol().unwrap_or("").to_string(),
            price: trade_event.price(),
            quantity: trade_event.quantity(),
            order_id: trade_event.order_id().unwrap_or("").to_string(),
            side,
            user_id: trade_event.user_id().unwrap_or("").to_string(),
            exchange_id: trade_event.exchange_id().unwrap_or("").to_string(),
        })
    }
}