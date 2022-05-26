use chrono::{DateTime, NaiveDate, NaiveTime};
use crate::decider::system::{System, SystemFactory};

pub mod order;
pub mod system;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OrderReference {
    OVER_LONG,
    BETWEEN_LONG,
    BETWEEN_SHORT,
    UNDER_SHORT,
}

pub enum OrderEvent {
    ConfirmationOpenAccepted,
    //{deal_id: String, deal_reference: WorkingOrderReference, level: f64},
    ConfirmationOpenRejected,
    // {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationCloseAccepted,
    // {deal_reference: WorkingOrderReference},
    ConfirmationCloseRejected,
    // {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationAmendedAccepted,
    // {deal_reference: WorkingOrderReference, level: f64},
    ConfirmationAmendedRejected,
    //{deal_reference: WorkingOrderReference, reason: String},
    PositionUpdateOpen,
    // {deal_reference: WorkingOrderReference, level: f64},
    PositionUpdateDelete, // {deal_reference: WorkingOrderReference, level: f64},
}

pub enum Event {
    Order(OrderEvent, OrderReference),
    Market(), // TODO this should be filterd so core only gets market updated when market is Tradable, filtering should be done in bfg_ig
    Account(),
}

pub enum Command {
    FetchData,//(FetchDataDetails),
    CreateWorkingOrder,//(WorkingOrderDetails),
    CancelWorkingOrder,//(String),          // deal_id
    UpdatePosition,//(String, f64), // deal_id, stop_level
    PublishTradeResults,
}

pub struct MarketInfo {
    pub epic: String,
    pub expiry: String,
    pub name: String,
    pub currency: String,
    pub min_stop_distance: u16,
    pub min_lot_size: u16,
    pub open_time_utc: NaiveTime,
    pub close_time_utc: NaiveTime,
    pub non_trading_days: Vec<NaiveDate>,
}

pub fn dax_system() -> System {
    SystemFactory::new(MarketInfo {
        epic: "IX.D.DAX.IFMM.IP".to_string(),
        expiry: "-".to_string(),
        name: "Tyskland 40 Cash (1€)".to_string(),
        currency: "EUR".to_string(),
        min_stop_distance: 5,
        min_lot_size: 1,
        open_time_utc: NaiveTime::from_hms(7,0,0),
        close_time_utc: NaiveTime::from_hms(15,30,0),
        non_trading_days: vec![],
    })
}