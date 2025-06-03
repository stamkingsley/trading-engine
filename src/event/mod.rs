pub mod trade_event;
pub mod processor;

pub use trade_event::{TradeEvent, EventType, OrderSide};
pub use processor::TradeEventProcessor;