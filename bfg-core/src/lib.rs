extern crate core;

use std::borrow::BorrowMut;
use std::collections::HashMap;
use crate::decider::{Command, Event, MarketInfo};
use crate::decider::system::{System, SystemFactory};
use crate::models::OhlcPrice;

pub mod decider;
pub mod models;

#[derive(Hash, Debug, Clone, Default, Eq, PartialEq)]
struct Epic(String);

#[derive(Debug, Clone, Default)]
struct AccountManager {
    id: String,
    available: f64,
}

impl AccountManager {
    fn is_trade_allowed(&self, risk: f64) -> bool {
        true
    }

    fn update(&mut self, update: f64) {
    }

}

#[derive(Debug, Default)]
struct SystemManager {
    systems: HashMap<Epic, System>,
}

impl SystemManager {
    /// Gets a list for markets and create a system for each
    pub fn new(markets: Vec<MarketInfo>) -> Self {
        let mut manager: SystemManager = Default::default();
        for market in markets {
            manager.add_system(market);
        }
        manager
    }
    fn add_system(&mut self, market_info: MarketInfo) {
        self.systems.insert(Epic(market_info.epic.clone()), SystemFactory::new(market_info));
    }
    pub fn step_one(&mut self, epic: Epic, event: Event) -> Vec<Command> {
        let system = self.systems.remove(&epic).expect("Only knows epics here");
        let (new_system, commands) = system.step(&event);
        self.systems.insert(epic, new_system);
        commands
    }
}

#[derive(Debug, Clone, Default)]
struct Market {
    candles: Vec<OhlcPrice>,
}

impl Market {
    fn add_candle(&mut self, price: OhlcPrice) {}
    /// Return the ATR for period if enogh candles
    /// Also if now is not < 1min from last candle none we have inconsistent candles
    fn atr(&self, period: u8) -> Option<f64> {
        None
    }
}

#[derive(Debug, Clone, Default)]
struct MarketManager {
    markets: HashMap<Epic, Market>
}
impl MarketManager {
    fn update(&mut self, epic: Epic, update: f64) {}
}


#[derive(Debug, Default)]
struct TradeManager {
    pub system_manager: SystemManager, // TODO where should candle info be for ATR calculation?
    account: AccountManager,
    market: MarketManager,
}

struct Bid(f64);
struct Ask(f64);

impl TradeManager {
    fn new(markets: Vec<MarketInfo>) -> Self {
        Self {
            system_manager: SystemManager::new(markets),
            ..Default::default()
        }
    }
    /// This should be all the system care about
    /// Market state and delay should be the concert to filter on in bfg_ig
    fn update_bid_ask(&mut self, epic: Epic, bid: Option<Bid>, ask: Option<Ask>) -> Command {
        // self.system_manager.step_one(epic, Event::Market {})
        todo!()
    }
}
#[cfg(test)]
mod tests {
    use chrono::Utc;
    use super::*;

    #[test]
    fn tester() {
        let infos = vec![MarketInfo {
            epic: "market a".to_string(),
            bars_in_opening_range: 0,
            expiry: "".to_string(),
            currency: "".to_string(),
            stop_distance: 0,
            lot_size: 0,
            open_time: Utc::now().time(),
            close_time: Utc::now().time(),
            start_fetch_data: Utc::now().time(),
            utc_close_working_order: Utc::now().time(),
            non_trading_days: vec![Utc::now().date().naive_utc()]
        }, MarketInfo {
            epic: "market b".to_string(),
            bars_in_opening_range: 0,
            expiry: "".to_string(),
            currency: "".to_string(),
            stop_distance: 0,
            lot_size: 0,
            open_time: Utc::now().time(),
            close_time: Utc::now().time(),
            start_fetch_data: Utc::now().time(),
            utc_close_working_order: Utc::now().time(),
            non_trading_days: vec![Utc::now().date().naive_utc()]
        }];
        let tm = TradeManager::new(infos);
    }
}
