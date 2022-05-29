use crate::decider::system::{System, SystemFactory};
use crate::models::{Direction, OhlcPrice, OrderReference};
use chrono::{DateTime, Duration, NaiveDate, NaiveTime, Utc};
use std::ops::{Add, Sub};

pub mod order;
pub mod system;

pub enum OrderEvent {
    ConfirmationOpenAccepted { level: f64 },
    ConfirmationOpenRejected,
    ConfirmationDeleteAccepted,
    ConfirmationDeleteRejected,
    ConfirmationAmendedAccepted,
    ConfirmationAmendedRejected,
    PositionEntry { entry_level: f64 }, // TODO There is a timestamp in RealtimeEvent i should use
    PositionExit { exit_level: f64 },   // TODO There is a timestamp in RealtimeEvent i should use
}

pub enum Event {
    Order(OrderEvent, OrderReference),
    Market {
        update_time: NaiveTime,
        bid: f64,
        ask: f64,
    },
    Account(),
    Data {
        prices: Vec<OhlcPrice>,
    },
    Error(String),
    PositionExit,
}

#[derive(Debug)]
pub enum Command {
    FetchData {
        start: String,
        end: String,
    },
    CreateWorkingOrder {
        direction: Direction,
        price: f64,
        reference: OrderReference,
    },
    CancelWorkingOrder {
        reference_to_cancel: OrderReference,
    },
    UpdatePosition {
        deal_id: String,
        level: f64,
    },
    PublishTradeResults(TradeResult),
    FatalFailure(String),
}

#[derive(Debug)]
pub struct TradeResult {
pub wanted_entry_level: f64,
pub actual_entry_level: f64,
pub entry_time: DateTime<Utc>,
pub exit_time: DateTime<Utc>,
pub exit_level: f64,
pub reference: OrderReference,
}

#[derive(Debug, Clone)]
pub struct MarketInfo {
    pub epic: String,
    pub expiry: String,
    pub name: String,
    pub currency: String,
    pub min_stop_distance: u16,
    pub min_lot_size: u16,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub non_trading_days: Vec<NaiveDate>,
}

impl Default for MarketInfo {
    fn default() -> Self {
        Self {
            epic: "".to_string(),
            expiry: "".to_string(),
            name: "".to_string(),
            currency: "".to_string(),
            min_stop_distance: 0,
            min_lot_size: 0,
            open_time: Utc::now().time().sub(Duration::minutes(10)),
            close_time: Utc::now().time().add(Duration::hours(5)),
            non_trading_days: vec![],
        }
    }
}

impl MarketInfo {
    pub fn is_inside_trading_hours(&self, now: &NaiveTime) -> bool {
        now.clone() > self.open_time.clone().add(Duration::minutes(1))
            && now.clone() < self.close_time.clone().sub(Duration::minutes(15))
    }
}

pub fn dax_system() -> System {
    SystemFactory::new(MarketInfo {
        epic: "IX.D.DAX.IFMM.IP".to_string(),
        expiry: "-".to_string(),
        name: "Tyskland 40 Cash (1€)".to_string(),
        currency: "EUR".to_string(),
        min_stop_distance: 5,
        min_lot_size: 1,
        open_time: NaiveTime::from_hms(8, 0, 0), // London time
        close_time: NaiveTime::from_hms(16, 30, 0), // London time
        non_trading_days: vec![],
    })
}
