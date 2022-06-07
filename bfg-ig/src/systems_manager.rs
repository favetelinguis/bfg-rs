use std::borrow::Borrow;
use std::collections::HashMap;
use log::{error, warn};
use bfg_core::decider::{Command, Event, MarketInfo};
use bfg_core::decider::order::WorkingOrder;
use bfg_core::decider::system::{System, SystemFactory};
use bfg_core::models::{OrderReference, TradeUpdate};
use ig_brokerage_adapter::realtime::models::{MarketUpdate, OpenPositionUpdate, TradeConfirmationUpdate};
use crate::{IgEvent, MarketCache, MarketView, OpenPositionCache, OrderView, SystemView, TradeConfirmationCache};

#[derive(Debug)]
pub struct SystemManager {
    system: System,
    trade_confirmation_cache: TradeConfirmationCache,
    open_position_cache: OpenPositionCache,
    market_cache: MarketCache,
}

#[derive(Debug, Default)]
pub struct SystemsManager {
    systems: HashMap<String, SystemManager>, // Epic, System
}

impl SystemsManager {
    pub fn new(markets: &[MarketInfo]) -> Self {
        let market_iter = markets.iter()
            .cloned()
            .map(|m| (m.epic.clone(), SystemManager {system: SystemFactory::new(m), trade_confirmation_cache: Default::default(), open_position_cache: Default::default(), market_cache: Default::default()}));
        Self {
            systems: HashMap::from_iter(market_iter),
        }
    }
    pub fn step_one(&mut self, epic: String, event: &Event) -> Vec<Command> {
        if let Some(mut system_manager) = self.systems.remove(&epic) {
            let (new_system, commands) = system_manager.system.step(event);
            system_manager.system = new_system;
            self.systems.insert(epic, system_manager);
            commands
        } else {
            warn!("Unable to update system for epic {} with event {:?}", epic, event);
            vec![]
        }
    }

    pub fn update_market(&mut self, epic: String, update: MarketUpdate) -> Option<(String, Event)> {
        if let Some(system_manager) = self.systems.get_mut(epic.as_str()) {
            system_manager.market_cache.update(update)
        } else {
            error!("Could not find epic {} for event {:?}", epic, update);
            None
        }
    }

    pub fn get_market_view(&mut self, epic: String) -> IgEvent {
        if let Some(system_manager) = self.systems.get(epic.as_str()) {
            system_manager.market_cache.get_current_view()
        } else {
            error!("Could not find epic {} form market view", epic);
            // TODO not good should not have this dummy return many Option or unwrap and make sure there is a value
            IgEvent::MarketView("DUMMY".to_string(), MarketView {
                epic,
                bid: None,
                ask: None,
                market_delay: None,
                market_state: None,
                update_time: None
            })
        }
    }

    pub fn update_confirms(&mut self, epic: String, update: TradeConfirmationUpdate) -> Option<(String, Event)> {
        if let Some(system_manager) = self.systems.get_mut(&epic) {
            system_manager.trade_confirmation_cache.update(update)
        } else {
            error!("Could not find epic {} for event {:?}", epic, update);
            None
        }
    }

    pub fn update_account_position(&mut self, epic: String, update: OpenPositionUpdate) -> Option<(String, Event)> {
        if let Some(mut system_manager) = self.systems.get_mut(&epic) {
            system_manager.open_position_cache.update(update)
        } else {
            error!("Could not find epic {} for event {:?}", epic, update);
            None
        }
    }

    pub(crate) fn get_deal_id(&self, epic: String, reference: &OrderReference) -> Option<String> {
        if let Some(system_manager) = self.systems.get(epic.as_str()) {
            system_manager.trade_confirmation_cache.get_deal_id(reference)
        } else {
            error!("Could not find epic {} for reference {:?}", epic, reference);
            None
        }
    }

    pub fn get_current_system_view(&self, epic: String) -> IgEvent {
        if let Some(ref system_manager) = self.systems.get(&epic) {
            let view = match system_manager.system.borrow() {
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
            return IgEvent::SystemView(epic, view)
        }
        // TODO this is not nice should reutrn optional if this can happen
        IgEvent::SystemView(epic.clone(), SystemView {
            state: "".to_string(),
            epic,
            opening_range_high_ask: None,
            opening_range_high_bid: None,
            opening_range_low_ask: None,
            opening_range_low_bid: None,
            orders: vec![]
        })
    }
}

fn create_order_view(orders: &HashMap<OrderReference, WorkingOrder>) -> Vec<OrderView> {
    orders.iter()
        .map(|(k, v)| OrderView {
            reference: format!("{:?}", k),
            state: v.to_string(),
        }).collect()
}

