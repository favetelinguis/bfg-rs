use std::borrow::Borrow;
use std::collections::HashMap;
use crate::decider::order::WorkingOrder;
use crate::decider::{Command, OrderReference};

pub struct OrderManager {
    orders: HashMap<OrderReference, WorkingOrder>,
}

impl OrderManager {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn get_order(&self, reference: &OrderReference) -> Option<&WorkingOrder> {
        self.orders.borrow().get(reference)
    }

    pub fn add_order(&mut self, reference: OrderReference, order: WorkingOrder) {
        self.orders.insert(reference, order);
    }

    pub fn get_deal_id(&self, reference: OrderReference) -> Command {
        todo!()
    }
}
