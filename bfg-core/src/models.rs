#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemValues {
    pub count: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SystemState {
    Setup,               // Await LTP to go over or_high or below or_low
    Entry(SystemValues), // LTP touches or_high or or_low
    AwaitingEntryConfirmation(SystemValues),
    Exit(SystemValues), // After 10 seconds or if LTP is over or_low och below or_high
    AwaitingExitConfirmation(SystemValues),
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
    pub status: BfgTradeStatus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BfgTradeStatus {
    OPEN,
    UPDATED,
    DELETED,
}

#[derive(Debug, Clone)]
pub struct TradeConfirmation {
    pub status: BfgTradeConfirmationStatus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum BfgTradeConfirmationStatus {
    ACCEPTED, REJECTED
}

#[derive(Debug, Eq, PartialEq)]
pub enum Direction {
    BUY,
    SELL,
}

#[derive(Debug)]
pub struct OrderDetails {
    pub direction: Direction,
    pub size: usize,
    pub price: f64,
}

impl OrderDetails {
    pub fn new(direction: Direction, price: f64) -> Self {
        OrderDetails {
            direction,
            size: 1,
            price,
        }
    }
}

#[derive(Debug)]
pub enum Decision {
    NoOp,
    Buy(OrderDetails),
    Sell(OrderDetails),
    SetupOr,
    Quit,
}
