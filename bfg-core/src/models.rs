use std::collections::HashMap;
use std::str::FromStr;

#[derive(Clone, Debug)]
pub enum EntryMode {
    OverBuyEntry,
    BetweenSellEntry,
    BetweenBuyEntry,
    UnderSellEntry,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PriceRelativeOr {
    Over,
    Between,
    Under,
}

#[derive(Debug, Clone)]
pub struct SystemValues {
    pub or_high_ask: f64,                                         // Sälj kurs
    pub or_low_ask: f64,                                          // Sälj kurs
    pub or_high_bid: f64,                                         // Köp kurs
    pub or_low_bid: f64,                                          // Köp kurs
    pub working_orders: (Option<OrderState>, Option<OrderState>), // long, short
}

#[derive(Debug, Clone)]
pub enum OrderState {
    AwaitingWorkingOrderCreateConfirmation(WorkingOrderSystemDetails),
    RejectedAtOpen(WorkingOrderReference, String, String),
    AcceptedAtOpen(String),
    WODeleted(String),
    PositionOpen(String),
    PositionOpenWithTrailingStop(String),
    AwaitCancelConfirmation(String), // When in between and other side get filled
}

#[derive(Clone, Debug, Default)]
pub struct WorkingOrderSystemDetails {
    pub deal_id: Option<String>,
    pub requested_entry_level: f64,
    pub actual_entry_level: Option<f64>,
    pub requested_exit_level: Option<f64>,
    pub actual_exit_level: Option<f64>,
}

impl WorkingOrderSystemDetails {
    pub fn new(requested_entry_level: f64) -> Self {
        Self {
            requested_entry_level,
            ..WorkingOrderSystemDetails::default()
        }
    }
}

#[derive(Debug)]
pub enum WorkingOrderPlacement {
    Over,
    Between,
    Under,
}

#[derive(Debug, Clone)]
pub enum SystemState {
    Setup,
    SetupWorkingOrder(SystemValues),
    ManageOrder(SystemValues),
}

impl Default for SystemState {
    fn default() -> Self {
        SystemState::Setup
    }
}

#[derive(Debug, Clone, Default)]
pub struct MarketUpdate {
    pub bid: Option<f64>,
    pub offer: Option<f64>,
    pub market_delay: Option<usize>,
    pub market_state: Option<String>,
    pub update_time: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AccountUpdate {
    pub account: Option<String>,
    pub pnl: Option<f64>,
    pub deposit: Option<f64>,
    pub available_cash: Option<f64>,
    pub pnl_lr: Option<f64>,
    pub pnl_nlr: Option<f64>,
    pub funds: Option<f64>,
    pub margin: Option<f64>,
    pub margin_lr: Option<f64>,
    pub margin_nlr: Option<f64>,
    pub available_to_deal: Option<f64>,
    pub equity: Option<f64>,
    pub equity_used: Option<f64>,
}

impl Default for AccountUpdate {
    fn default() -> Self {
        AccountUpdate {
            account: Some("ZQVBB".to_string()),
            pnl: None,
            deposit: None,
            available_cash: None,
            pnl_lr: None,
            pnl_nlr: None,
            funds: None,
            margin: None,
            margin_lr: None,
            margin_nlr: None,
            available_to_deal: None,
            equity: None,
            equity_used: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TradeUpdate {
    pub deal_status: DealStatus,
    pub status: BfgTradeStatus,
    pub deal_id: String,
    pub deal_reference: WorkingOrderReference,
}

#[derive(Debug, Clone)]
pub struct WorkingOrderUpdate {
    pub status: BfgTradeStatus,
    pub deal_id: String,
    pub deal_reference: String,
    pub deal_status: DealStatus,
    pub level: f64,
}

impl FromStr for WorkingOrderReference {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "OVER_LONG" => Ok(WorkingOrderReference::OVER_LONG),
            "BETWEEN_LONG" => Ok(WorkingOrderReference::BETWEEN_LONG),
            "BETWEEN_SHORT" => Ok(WorkingOrderReference::BETWEEN_SHORT),
            "UNDER_SHORT" => Ok(WorkingOrderReference::UNDER_SHORT),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DealStatus {
    ACCEPTED,
    REJECTED,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ConfirmsStatus {
    AMENDED,
    CLOSED,
    DELETED,
    OPEN,
    PARTIALLY_CLOSED,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BfgTradeStatus {
    OPEN,
    UPDATED,
    DELETED,
}

#[derive(Debug, Clone)]
pub struct TradeConfirmation {
    pub deal_status: DealStatus,
    pub status: Option<ConfirmsStatus>,
    pub deal_id: String,
    pub deal_reference: WorkingOrderReference,
    pub reason: String,
}

#[derive(Debug, Eq, PartialEq)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkingOrderReference {
    OVER_LONG,
    BETWEEN_LONG,
    BETWEEN_SHORT,
    UNDER_SHORT,
}

#[derive(Debug)]
pub struct MarketOrderDetails {
    pub direction: Direction,
    pub size: usize,
}

#[derive(Debug)]
pub struct WorkingOrderDetails {
    pub direction: Direction,
    pub price: f64,
    pub reference: WorkingOrderReference,
}

#[derive(Debug)]
pub struct LimitOrderDetails {
    pub direction: Direction,
    pub size: usize,
    pub price: f64,
}

impl LimitOrderDetails {
    pub fn new(direction: Direction, price: f64) -> Self {
        LimitOrderDetails {
            direction,
            size: 1,
            price,
        }
    }
}

#[derive(Debug)]
pub enum Decision {
    NoOp,
    CreateWorkingOrder(WorkingOrderDetails),
    FetchData(FetchDataDetails),
    CancelWorkingOrder(String),     // deal_id
    UpdateWithTrailingStop(String, f64), // deal_id, stop_level
}

#[derive(Debug, Clone)]
pub struct DataUpdate {
    pub prices: Vec<OhlcPrice>,
}

#[derive(Debug, Clone)]
pub struct OhlcPrice {
    pub open: Price,
    pub close: Price,
    pub high: Price,
    pub low: Price,
}

#[derive(Debug, Clone)]
pub struct Price {
    pub bid: f64,
    pub ask: f64,
}

#[derive(Debug)]
pub struct FetchDataDetails {
    pub start: String, // yyy-MM-ddTHH:mm:ss
    pub end: String,
}
#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use chrono::{Local, NaiveDateTime, NaiveTime, SecondsFormat, Utc};
    use std::str::FromStr;

    #[test]
    fn initial_market() {
        let now = Utc::now();
        let time_of_day = now.time();
        let open_time = NaiveTime::from_hms(9, 0, 0);
        let close_time = NaiveTime::from_hms(16, 30, 0);
        let local_now = Local::now();
        let dt = NaiveDateTime::new(now.naive_utc().date(), open_time);
        let dt_format = dt.format("%Y-%m-%dT%H:%M:%S").to_string();
        let t3 = format!(
            "UTC now in RFC 3339 is: {}",
            now.to_rfc3339_opts(SecondsFormat::Secs, false)
        );
        let updatetime = NaiveTime::from_str("06: 59: 27").unwrap();

        let dd = 5;
    }
}
