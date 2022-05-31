use crate::decider::system::{System, SystemFactory};
use crate::models::{Direction, OhlcPrice, OrderReference};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use std::ops::{Add, Sub};

pub mod order;
pub mod system;

pub enum OrderEvent {
    ConfirmationOpenAccepted { level: f64, deal_id: String },
    ConfirmationDeleteAccepted,
    ConfirmationAmendedAccepted,
    ConfirmationRejection,
    PositionEntry { entry_level: f64 }, // TODO There is a timestamp in RealtimeEvent i should use
    PositionExit { exit_level: f64 },   // TODO There is a timestamp in RealtimeEvent i should use
}

pub enum Event {
    Order(OrderEvent, OrderReference),
    Market {
        epic: String,
        update_time: NaiveTime,
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
        start: NaiveDateTime,
        duration: Duration,
        epic: String,
    },
    CreateWorkingOrder {
        direction: Direction,
        price: f64,
        reference: OrderReference,
        market_info: MarketInfo,
        target_price: Option<f64>,
    },
    CancelWorkingOrder {
        reference_to_cancel: OrderReference,
    },
    UpdatePosition {
        deal_id: String,
        level: f64,
        trailing_stop_distance: u8,
        target_level: Option<f64>,
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
}

#[derive(Debug, Clone)]
pub struct MarketInfo {
    pub epic: String,
    pub expiry: String,
    pub name: String,
    pub currency: String,
    pub min_stop_distance: u8,
    pub min_lot_size: u8,
    pub min_step_size: u8,
    pub open_time: NaiveTime,
    pub close_time: NaiveTime,
    pub start_fetch_data: NaiveTime,
    pub utc_close_working_order: NaiveTime,
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
            min_step_size: 0,
            open_time: Utc::now().time().sub(Duration::minutes(10)),
            close_time: Utc::now().time().add(Duration::hours(5)),
            start_fetch_data: Utc::now().time().add(Duration::hours(5)),
            utc_close_working_order: Utc::now().time().add(Duration::hours(5)),
            non_trading_days: vec![],
        }
    }
}

impl MarketInfo {
    pub fn is_inside_trading_hours(&self, now: &NaiveTime) -> bool {
        *now > self.open_time.add(Duration::minutes(1))
            && *now < self.close_time.sub(Duration::minutes(15))
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
        min_step_size: 1, // Not sure this is correct but need to for buffer calc etc
        open_time: NaiveTime::from_hms(8, 0, 0), // London time
        close_time: NaiveTime::from_hms(16, 30, 0), // London time
        utc_close_working_order: NaiveTime::from_hms(15, 15, 0), // Utc
        start_fetch_data: NaiveTime::from_hms(9, 0, 0), // Account time
        non_trading_days: vec![],
    })
}
