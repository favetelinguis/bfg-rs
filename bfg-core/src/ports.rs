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
    id: usize,
    open_time: usize,
    close_time: usize,
    spread: usize,
    cost_to_trade: usize
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
    fn setup_market(market: MarketValues);
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
    fn get_or(&self) -> Option<Or>;
    fn place_order(&self, order: OrderDetails);
}
