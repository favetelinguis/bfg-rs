use std::collections::HashMap;
use bfg_core::decider::{Command, Event, MarketInfo};
use bfg_core::decider::order::WorkingOrder;
use bfg_core::decider::system::{System, SystemFactory};
use bfg_core::models::OrderReference;
use crate::{IgEvent, OrderView, SystemView};

#[derive(Debug, Default)]
pub struct SystemsManager {
    systems: HashMap<String, System>, // Epic, System
}

impl SystemsManager {
    pub fn new(markets: &[MarketInfo]) -> Self {
        let market_iter = markets.iter()
            .cloned()
            .map(|m| (m.epic.clone(), SystemFactory::new(m)));
        Self {
            systems: HashMap::from_iter(market_iter),
        }
    }
    pub fn step_one(&mut self, epic: String, event: &Event) -> Vec<Command> {
        let mut system = self.systems.remove(&epic).unwrap();
        let (new_system, commands) = system.step(event);
        self.systems.insert(epic, new_system);
        commands
    }

    pub fn get_current_system_view(&self, epic: String) -> IgEvent {
        let view = match self.systems.get(&epic).unwrap() {
            System::Setup(val) => SystemView {
                state: String::from("Setup"),
                epic: val.market_info.epic.clone(),
                ..Default::default()
            },
            System::AwaitData(val) => SystemView {
                state: String::from("AwaitData"),
                epic: val.market_info.epic.clone(),
                ..Default::default()
            },
            System::DecideOrderPlacement(val) =>
                SystemView {
                    state: String::from("DecideOrderPlacement"),
                    opening_range_high_ask: Some(val.state.opening_range.high_ask),
                    opening_range_high_bid: Some(val.state.opening_range.high_bid),
                    opening_range_low_ask: Some(val.state.opening_range.low_ask),
                    opening_range_low_bid: Some(val.state.opening_range.low_bid),
                    epic: val.market_info.epic.clone(),
                    ..Default::default()
                },
            System::ManageOrders(val) => SystemView {
                state: String::from("ManageOrders"),
                opening_range_high_ask: Some(val.state.opening_range.high_ask),
                opening_range_high_bid: Some(val.state.opening_range.high_bid),
                opening_range_low_ask: Some(val.state.opening_range.low_ask),
                opening_range_low_bid: Some(val.state.opening_range.low_bid),
                orders: create_order_view(val.state.order_manager.get_orders()),
                epic: val.market_info.epic.clone(),
                ..Default::default()
            },
            System::Error(val) => SystemView {
                state: String::from("Error"),
                epic: val.market_info.epic.clone(),
                ..Default::default()
            },
        };
        IgEvent::SystemView(epic, view)
    }
}

fn create_order_view(orders: &HashMap<OrderReference, WorkingOrder>) -> Vec<OrderView> {
    orders.iter()
        .map(|(k, v)| OrderView {
            reference: format!("{:?}", k),
            state: v.to_string(),
        }).collect()
}

