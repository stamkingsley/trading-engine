use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventType {
    OrderPlaced,
    OrderCancelled,
    OrderFilled,
    TradeExecuted,
    PriceUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeEvent {
    pub event_id: String,
    pub timestamp: u64,
    pub event_type: EventType,
    pub symbol: String,
    pub price: f64,
    pub quantity: f64,
    pub order_id: String,
    pub side: OrderSide,
    pub user_id: String,
    pub exchange_id: String,
}

impl TradeEvent {
    pub fn new(
        event_type: EventType,
        symbol: String,
        price: f64,
        quantity: f64,
        order_id: String,
        side: OrderSide,
        user_id: String,
        exchange_id: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().timestamp_millis() as u64,
            event_type,
            symbol,
            price,
            quantity,
            order_id,
            side,
            user_id,
            exchange_id,
        }
    }

    pub fn order_placed(
        symbol: String,
        price: f64,
        quantity: f64,
        order_id: String,
        side: OrderSide,
        user_id: String,
        exchange_id: String,
    ) -> Self {
        Self::new(
            EventType::OrderPlaced,
            symbol,
            price,
            quantity,
            order_id,
            side,
            user_id,
            exchange_id,
        )
    }

    pub fn trade_executed(
        symbol: String,
        price: f64,
        quantity: f64,
        order_id: String,
        side: OrderSide,
        user_id: String,
        exchange_id: String,
    ) -> Self {
        Self::new(
            EventType::TradeExecuted,
            symbol,
            price,
            quantity,
            order_id,
            side,
            user_id,
            exchange_id,
        )
    }
}