pub struct MarketUpdate {
    pub high: usize,
}

pub struct AccountUpdate {
    pub money: usize,
}

pub struct TradeUpdate {
    pub entry: String,
}

#[cfg_attr(test, mockall::automock)]
pub trait BfgService {
    fn publish_market_update_event(&mut self, update: MarketUpdate);
    fn publish_account_update_event(&mut self, update: AccountUpdate);
    fn publish_trade_update_event(&mut self, update: TradeUpdate);
}

pub enum Direction {
    Long,
    Short,
}
pub struct OrderDetails {
    direction: Direction,
    amount: usize,
    price: usize,
}

pub struct Or {
    high: usize,
    low: usize,
}

#[cfg_attr(test, mockall::automock)]
pub trait BrokerageApi {
    fn get_or(&self) -> Option<Or>;
    fn place_order(&self, order: OrderDetails);
}
