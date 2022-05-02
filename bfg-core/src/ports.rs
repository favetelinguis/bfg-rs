use crate::errors::BrokerageError;

#[derive(Copy, Clone)]
pub struct MarketUpdate {
    pub high: usize,
}

#[derive(Copy, Clone)]
pub struct AccountUpdate {
    pub money: usize,
}

#[derive(Copy, Clone)]
pub struct TradeUpdate {
    pub entry: usize,
}

#[derive(Copy, Clone)]
pub enum Action {
    Start,
    MarketEvent(MarketUpdate),
    AccountEvent(AccountUpdate),
    TradeEvent(TradeUpdate),
    OrSetup(Option<Or>),
    Quit,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MarketValues {
    pub id: usize,
    pub open_time: usize,
    pub close_time: usize,
    pub spread: usize,
    pub cost_to_trade: usize
}

impl MarketValues {
    pub fn new() -> Self {
        MarketValues {
            id: 33,
            open_time: 5,
            close_time: 8,
            spread: 33,
            cost_to_trade: 4
        }
    }

}

#[cfg_attr(test, mockall::automock)]
pub trait BfgService {
    fn market_details(&self) -> MarketValues;
    fn setup_market(&mut self, market: MarketValues);
    fn publish_update_event(&mut self, update: Action);
}

#[derive(Debug, Eq, PartialEq)]
pub enum Direction {
    Long,
    Short,
}

#[derive(Debug, Eq, PartialEq)]
pub struct OrderDetails {
    direction: Direction,
    amount: usize,
    price: usize,
}

impl OrderDetails {
    pub fn new(direction: Direction, price: usize) -> Self {
        OrderDetails {
            direction,
            amount: 1,
            price
        }
    }
}

#[derive(Copy, Clone)]
pub struct Or {
    high: usize,
    low: usize,
}

impl Or {
    pub fn new(high: usize, low: usize) -> Or {
        Or {high, low}
    }
}

#[cfg_attr(test, mockall::automock)]
pub trait BrokerageApi {
    fn get_or(&mut self) -> Option<Or>;
    fn place_order(&mut self, order: OrderDetails);
    fn get_market_details(&mut self) -> Result<(), BrokerageError>;
}
