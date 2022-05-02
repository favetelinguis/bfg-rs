use actix_web::web;
use bfg_core::ports::{MarketUpdate, MarketValues};
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct MarketDetails {
    id: usize,
    open_time: usize,
    close_time: usize,
    spread: usize,
    cost_to_trade: usize
}

#[derive(Deserialize)]
pub struct MarketEvent {
    high: usize,
}

impl From<web::Json<MarketEvent>> for MarketEvent {
    fn from(event: web::Json<MarketEvent>) -> Self {
        Self {
            high: event.high,
        }
    }
}

impl From<MarketEvent> for MarketUpdate {
    fn from(event: MarketEvent) -> Self {
        Self {
            high: event.high,
        }
    }
}

impl From<MarketValues> for MarketDetails {
    fn from(original: MarketValues) -> Self {
        Self {
            id: original.id,
            open_time: original.open_time,
            close_time: original.close_time,
            spread: original.spread,
            cost_to_trade: original.cost_to_trade
        }
    }
}
