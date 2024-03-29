use std::ops::{Index, Range};
use bfg_core::decider::MarketInfo;

pub struct SubscriptionManager {
    markets: Vec<String>,
}

impl SubscriptionManager {
    /// Subscription id starts from 3 since 1,2 is used by trade and account subscription
    pub fn new(epics: Vec<String>) -> Self {
        Self {
            markets: epics,
        }
    }

    pub fn get_epic_from_subscription_id(&self, subscription_id: usize) -> String {
        // -3 since we start att subscription id 3 and we need to index 0
        self.markets.index(subscription_id-3).clone()
    }

    pub fn get_subscription_id_range(&self) -> Range<usize >{
        3..(3+self.markets.len())
    }
}