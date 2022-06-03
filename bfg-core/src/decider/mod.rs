use crate::decider::system::{System, SystemFactory};
use crate::models::{Direction, OhlcPrice, OrderReference};
use chrono::{DateTime, Duration, Utc};
use std::ops::{Add, Sub};

pub mod order;
pub mod system;

#[derive(Debug)]
pub enum OrderEvent {
    ConfirmationOpenAccepted { level: f64, deal_id: String },
    ConfirmationDeleteAccepted,
    ConfirmationAmendedAccepted,
    ConfirmationRejection,
    PositionEntry { entry_level: f64 }, // TODO There is a timestamp in RealtimeEvent i should use
    PositionExit { exit_level: f64 },   // TODO There is a timestamp in RealtimeEvent i should use
}

#[derive(Debug)]
pub enum Event {
    Order(OrderEvent, OrderReference),
    Market {
        epic: String,
        update_time: DateTime<Utc>,
        bid: f64,
        ask: f64,
    },
    Account(),
    Data {
        prices: Vec<OhlcPrice>,
    },
    Error(String),
    PositionExit(OrderReference),
}

#[derive(Debug)]
pub enum Command {
    FetchData {
        start: DateTime<Utc>,
        duration: Duration,
        epic: String,
    },
    CreateWorkingOrder {
        direction: Direction,
        price: f64,
        reference: OrderReference,
        market_info: MarketInfo,
        target_distance: f64,
        stop_distance: f64,
    },
    CancelWorkingOrder {
        epic: String,
        reference_to_cancel: OrderReference,
    },
    UpdatePosition {
        epic: String,
        deal_id: String,
        level: f64,
        trailing_stop_distance: f64,
        target_distance: f64,
    },
    PublishTradeResults(TradeResult),
    FatalFailure(String),
}

#[derive(Debug, Clone)]
pub struct TradeResult {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub exit_level: f64,
    pub reference: OrderReference,
    pub epic: String,
    pub opening_range_size: f64,
    pub strategy_version: usize
}

#[derive(Debug, Clone)]
pub struct MarketInfo {
    pub epic: String,
    pub bars_in_opening_range: u8,
    pub min_tradable_opening_range: f64,
    pub expiry: String,
    pub currency: String,
    pub lot_size: u8,
    pub utc_open_time: DateTime<Utc>,
    pub utc_close_time: DateTime<Utc>,
}

impl Default for MarketInfo {
    fn default() -> Self {
        Self {
            epic: "".to_string(),
            bars_in_opening_range: 0,
            min_tradable_opening_range: 0.0,
            expiry: "".to_string(),
            currency: "".to_string(),
            lot_size: 0,
            utc_open_time: Utc::now().sub(Duration::minutes(10)),
            utc_close_time: Utc::now().add(Duration::hours(5)),
        }
    }
}

impl MarketInfo {
    pub fn is_inside_trading_hours(&self, now: &DateTime<Utc>) -> bool {
        *now > self.utc_open_time.add(Duration::minutes(1))
            && *now < self.utc_close_time.sub(Duration::minutes(15))
    }
    pub fn stop_distance(&self, opening_range_size: f64) -> f64 {
        ((opening_range_size - 1.) / 3.)
    }
}