use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use crate::decider::order::{RISK_REWARD_RATION, WorkingOrder, WorkingOrderFactory};
use crate::decider::{Command, Event, MarketInfo};
use crate::models::OhlcPrice;
use crate::models::{Direction, OrderReference};
use chrono::{Duration, NaiveDateTime, Utc};
use log::{error, info, warn};

const OPENING_RANGE_MULTIPLIER: f64 = 3.;

#[derive(Debug)]
pub struct SystemMachine<S> {
    pub state: S,
    pub market_info: MarketInfo,
    pub last_position_reference: Option<OrderReference>,
}

#[derive(Debug)]
pub struct Setup;
#[derive(Debug)]
pub struct Error;
#[derive(Debug)]
pub struct AwaitData;
#[derive(Debug)]
pub struct DecideOrderPlacement {
    pub opening_range: OpeningRange,
    pub order_manager: OrderManager,
}
#[derive(Debug)]
pub struct ManageOrders {
    pub opening_range: OpeningRange,
    pub order_manager: OrderManager,
}

// Starting state
impl SystemMachine<Setup> {
    fn new(market_info: MarketInfo) -> Self {
        SystemMachine {
            state: Setup,
            market_info,
            last_position_reference: None,
        }
    }
}

impl From<SystemMachine<Setup>> for SystemMachine<AwaitData> {
    fn from(val: SystemMachine<Setup>) -> Self {
        Self {
            state: AwaitData,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

impl From<SystemMachine<AwaitData>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<AwaitData>) -> Self {
        Self {
            state: DecideOrderPlacement {
                opening_range: Default::default(),
                order_manager: Default::default(),
            },
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageOrders> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: ManageOrders {
                opening_range: val.state.opening_range,
                order_manager: val.state.order_manager,
            },
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

impl From<SystemMachine<ManageOrders>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageOrders>) -> Self {
        Self {
            state: DecideOrderPlacement {
                opening_range: val.state.opening_range,
                order_manager: Default::default(), // Reset all orders
            },
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<Setup> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: Setup,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

// All states can transition to error
impl From<SystemMachine<Setup>> for SystemMachine<Error> {
    fn from(val: SystemMachine<Setup>) -> Self {
        Self {
            state: Error,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}
impl From<SystemMachine<AwaitData>> for SystemMachine<Error> {
    fn from(val: SystemMachine<AwaitData>) -> Self {
        Self {
            state: Error,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}
impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<Error> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: Error,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}
impl From<SystemMachine<ManageOrders>> for SystemMachine<Error> {
    fn from(val: SystemMachine<ManageOrders>) -> Self {
        Self {
            state: Error,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}
impl From<SystemMachine<ManageOrders>> for SystemMachine<Setup> {
    fn from(val: SystemMachine<ManageOrders>) -> Self {
        Self {
            state: Setup,
            market_info: val.market_info,
            last_position_reference: val.last_position_reference,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct OpeningRange {
    pub high_ask: f64,
    pub high_bid: f64,
    pub low_ask: f64,
    pub low_bid: f64,
}

impl OpeningRange {
    pub fn get_middle_price_high(&self) -> f64 {
        (self.high_bid + self.high_ask) / 2.
    }
    pub fn get_middle_price_low(&self) -> f64 {
        (self.low_bid + self.low_ask) / 2.
    }
    pub fn range_size(&self) -> f64 {
        self.get_middle_price_high() - self.get_middle_price_low()
    }
    pub fn spread(&self) -> f64 {
        self.low_ask - self.low_bid
    }
}

#[derive(Debug)]
pub enum System {
    Setup(SystemMachine<Setup>),
    AwaitData(SystemMachine<AwaitData>),
    DecideOrderPlacement(SystemMachine<DecideOrderPlacement>),
    ManageOrders(SystemMachine<ManageOrders>),
    Error(SystemMachine<Error>),
}

impl System {
    pub fn step(self, event: &Event) -> (System, Vec<Command>) {
        match (self, event) {
            // Setup -> AwaitData [FetchData]
            (
                System::Setup(val),
                Event::Market {
                    ref update_time, ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time) => {
                let command = create_fetch_data_command(val.market_info.borrow());
                (
                    System::AwaitData(val.into()),
                    vec![command],
                )
            },
            // AwaitData -> DecideOrderPlacement []
            (System::AwaitData(val), Event::Data { prices, .. }) if !prices.is_empty() => {
                if prices.len() != val.market_info.bars_in_opening_range {
                    error!("Opening range bars are {} expected {}", prices.len(), val.market_info.bars_in_opening_range);
                }
                let opening_range = create_opening_range_from_ohlcs(prices);
                // We add +1 since we always want to leave a 1pip open in the or channel for orders to open in between mode
                // since we can use 1R and 2R in decide order placement that leave 1pip left.
                // NOW I DONT HAVE 2X so remove +1
                if (opening_range.range_size() >= ((val.market_info.min_stop * OPENING_RANGE_MULTIPLIER))) && (opening_range.range_size() <= (((  val.market_info.min_stop * val.market_info.max_stop_multiplier) * OPENING_RANGE_MULTIPLIER))) {
                    info!("Trading range is {} for {}", opening_range.range_size(), val.market_info.epic);
                    let mut new_state: SystemMachine<DecideOrderPlacement> = val.into();
                    new_state.state.opening_range = opening_range;
                    (System::DecideOrderPlacement(new_state), vec![])
                } else {
                    warn!("Trading range is to small or large for {}", val.market_info.epic);
                    let mut new_state: SystemMachine<Error> = val.into();
                    (System::Error(new_state), vec![])
                }
            }
            // DecideOrderPlacement -> Setup []
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time, ..
                },
            ) if !val.market_info.is_inside_trading_hours(update_time) => {
                (System::Setup(val.into()), vec![])
            }
            // DecideOrderPlacement -> ManageOrders [CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_over(val.market_info.stop_distance(val.state.opening_range.range_size()), &val.state.opening_range, *bid, *ask, &val.last_position_reference) =>
            {
                let stop_distance = val.market_info.stop_distance(val.state.opening_range.range_size());
                let command = Command::CreateWorkingOrder {
                    direction: Direction::BUY,
                    price: val.state.opening_range.high_ask + val.state.opening_range.spread(), // I add spread since i noticed so many trade just outside my line
                    reference: OrderReference::OVER_LONG,
                    market_info: val.market_info.clone(),
                    target_distance: stop_distance * RISK_REWARD_RATION,
                    stop_distance,
                };
                let market_info_clone = val.market_info.clone(); // TODO should save a reference to market_info not clone it
                let opening_range_clone = val.state.opening_range.clone(); // TODO should save a reference to market_info not clone it
                let mut new_system: SystemMachine<ManageOrders> = val.into();
                new_system.state.order_manager.create_order(OrderReference::OVER_LONG, market_info_clone, opening_range_clone);
                (
                    System::ManageOrders(new_system),
                    vec![command],
                    )
            }
            // DecideOrderPlacement -> ManageOrders [CreateWorkingOrder, CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_between(val.market_info.stop_distance(val.state.opening_range.range_size()), &val.state.opening_range, *bid, *ask, &val.last_position_reference) =>
            {
                let stop_distance = val.market_info.stop_distance(val.state.opening_range.range_size());
                let commands = vec![
                    Command::CreateWorkingOrder {
                        direction: Direction::BUY,
                        price: val.state.opening_range.low_ask + val.state.opening_range.spread(),
                        reference: OrderReference::BETWEEN_LONG,
                        market_info: val.market_info.clone(),
                        stop_distance,
                        target_distance: stop_distance * RISK_REWARD_RATION,
                    },
                    Command::CreateWorkingOrder {
                        direction: Direction::SELL,
                        price: val.state.opening_range.high_bid - val.state.opening_range.spread(),
                        reference: OrderReference::BETWEEN_SHORT,
                        market_info: val.market_info.clone(),
                        stop_distance,
                        target_distance: stop_distance * RISK_REWARD_RATION,
                    },
                ];
                let market_info_clone1 = val.market_info.clone(); // TODO should save a reference to market_info not clone it
                let market_info_clone2 = val.market_info.clone(); // TODO should save a reference to market_info not clone it
                let opening_range_clone1 = val.state.opening_range.clone(); // TODO should save a reference to market_info not clone it
                let opening_range_clone2 = val.state.opening_range.clone(); // TODO should save a reference to market_info not clone it
                let mut new_system: SystemMachine<ManageOrders>= val.into();
                new_system.state.order_manager.create_order(OrderReference::BETWEEN_LONG, market_info_clone1, opening_range_clone1);
                new_system.state.order_manager.create_order(OrderReference::BETWEEN_SHORT, market_info_clone2, opening_range_clone2);
                (
                    System::ManageOrders(new_system),
                    commands,
                )
            }
            // DecideOrderPlacement -> ManageOrders [CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_under(val.market_info.stop_distance(val.state.opening_range.range_size()), &val.state.opening_range, *bid, *ask, &val.last_position_reference) =>
            {
                let stop_distance = val.market_info.stop_distance(val.state.opening_range.range_size());
                let command = Command::CreateWorkingOrder {
                    direction: Direction::SELL,
                    price: val.state.opening_range.low_bid - val.state.opening_range.spread(),
                    reference: OrderReference::UNDER_SHORT,
                    market_info: val.market_info.clone(),
                    stop_distance,
                    target_distance: stop_distance * RISK_REWARD_RATION,
                };
                let market_info_clone = val.market_info.clone(); // TODO should save a reference to market_info not clone it
                let opening_range_clone = val.state.opening_range.clone(); // TODO should save a reference to market_info not clone it
                let mut new_system: SystemMachine<ManageOrders >= val.into();
                new_system.state.order_manager.create_order(OrderReference::UNDER_SHORT, market_info_clone, opening_range_clone);
                (
                    System::ManageOrders(new_system),
                    vec![command],
                )
            }
            // ManageOrders -> ManageOrders [...] - Market
            (
                System::ManageOrders(mut val),
                event @ Event::Market { update_time, .. }
            ) if val.market_info.is_inside_trading_hours(update_time) => {
                let commands = val.borrow_mut().state.order_manager.step_all(event);
                (System::ManageOrders(val), commands)
            }
            // ManageOrders -> ManageOrders [...] - Order
            (
                System::ManageOrders(mut val),
                event @ Event::Order(_, reference)
            ) => {
                let commands = val.borrow_mut().state.order_manager.step_one(reference.clone(), event);
                (System::ManageOrders(val), commands)
            }
            // ManageOrders -> Setup [...] - Order
            (
                System::ManageOrders(val),
                event @ Event::Market { update_time, .. }
            ) if !val.market_info.is_inside_trading_hours(update_time) => {
                (System::Setup(val.into()), vec![])
            }
            // ManageOrders -> DecideOrderPlacement [] - WOCancel
            (
                System::ManageOrders(mut val),
                Event::WOCancel(reference)
            ) => {
                let mut new_system: SystemMachine<DecideOrderPlacement> = val.into();
                new_system.last_position_reference = Some(reference.clone());
                (System::DecideOrderPlacement(new_system), vec![])
            }
            // ManageOrders -> DecideOrderPlacement []
            (System::ManageOrders(val), Event::PositionExit(reference)) => {
                let mut new_system: SystemMachine<DecideOrderPlacement> = val.into();
                new_system.last_position_reference = Some(reference.clone());
                (
                    System::DecideOrderPlacement(new_system),
                    vec![],
                )
            },
            // Error transitions - START
            (System::Setup(val), Event::Error(reason)) => (
                System::Error(val.into()),
                vec![Command::FatalFailure(reason.clone())],
            ),
            (System::AwaitData(val), Event::Error(reason)) => (
                System::Error(val.into()),
                vec![Command::FatalFailure(reason.clone())],
            ),
            (System::ManageOrders(val), Event::Error(reason)) => (
                System::Error(val.into()),
                vec![Command::FatalFailure(reason.clone())],
            ),
            (System::DecideOrderPlacement(val), Event::Error(reason)) => (
                System::Error(val.into()),
                vec![Command::FatalFailure(reason.clone())],
            ),
            // Error transitions - END
            (val, _) => (val, vec![]),
        }
    }
}

/// Get the data for the first open minute
fn create_fetch_data_command(market_info: &MarketInfo) -> Command {
    Command::FetchData {
        epic: market_info.epic.clone(),
        start: market_info.utc_open_time,
        duration: Duration::minutes((market_info.bars_in_opening_range - 1) as i64),
    }
}

fn is_price_over(stop_distance: f64, opening_range: &OpeningRange, bid: f64, ask: f64, last_trade_reference: &Option<OrderReference>) -> bool {
    let level = (bid + ask) / 2.;
    let buffer: f64;
    if let Some(OrderReference::BETWEEN_SHORT | OrderReference::UNDER_SHORT) = last_trade_reference {
        // We have twice the buffer when changing direction
        buffer = stop_distance as f64 * 1.; //* 2.; // We only have 1x for both buffers now
    } else {
        buffer = stop_distance as f64;
    }
    level > (opening_range.get_middle_price_high() + buffer)
}

/// Opening range must be 3.4x stop distance
/// If we try to change direction we have a buffer of 3x stop distance so that leave some distance when we force a 3.4x opening range to always have room to trigger.
fn is_price_between(stop_distance: f64, opening_range: &OpeningRange, bid: f64, ask: f64, last_trade_reference: &Option<OrderReference>) -> bool {
    let level = (bid + ask) / 2.;
    let mut long_buffer = stop_distance as f64;
    let mut short_buffer = stop_distance as f64;
    // To change direction we require twice the buffer
    if let Some(OrderReference::BETWEEN_LONG | OrderReference::OVER_LONG) = last_trade_reference {
        short_buffer = 1. * stop_distance as f64;
    }
    // To change direction we require twice the buffer
    if let Some(OrderReference::BETWEEN_SHORT | OrderReference::UNDER_SHORT) = last_trade_reference {
        long_buffer = 1. * stop_distance as f64;
    }
    (level < (opening_range.get_middle_price_high() - short_buffer))
        && (level > (opening_range.get_middle_price_low() + long_buffer))
}

fn is_price_under(stop_distance: f64, opening_range: &OpeningRange, bid: f64, ask: f64, last_trade_reference: &Option<OrderReference>) -> bool {
    let level = (bid + ask) / 2.;
    let buffer;
    if let Some(OrderReference::BETWEEN_LONG | OrderReference::OVER_LONG) = last_trade_reference {
        // We have twice the buffer when changing direction
        buffer = stop_distance as f64 * 1.;
    } else {
        buffer = stop_distance as f64;
    }
    level < (opening_range.get_middle_price_low() - buffer)
}

pub struct SystemFactory;

impl SystemFactory {
    pub fn new(market_info: MarketInfo) -> System {
        System::Setup(SystemMachine::new(market_info))
    }
}

#[derive(Debug, Default)]
pub struct OrderManager {
    orders: HashMap<OrderReference, WorkingOrder>,
}

impl OrderManager {
    fn create_order(&mut self, reference: OrderReference, market_info: MarketInfo, opening_range: OpeningRange) {
        self.orders.insert(reference, WorkingOrderFactory::new(market_info, opening_range));
    }

    /// An event that only affects a single order
    fn step_one(&mut self, reference: OrderReference, event: &Event) -> Vec<Command> {
        if let Some(order) = self.orders.remove(reference.borrow()) {
            let (new_state, commands) = order.step(event);
            self.orders.insert(reference, new_state);
            return commands
        }
        vec![]
    }

    /// An event that should be triggered for all orders
    fn step_all(&mut self, event: &Event) -> Vec<Command> {
        let mut commands = vec![];
        let keys: Vec<OrderReference> = self.orders.keys().cloned().collect();
        for reference in keys {
            commands.extend(self.step_one(reference, event));
        }
        commands
    }

    // Get a view of the orders
    pub fn get_orders(&self) -> &HashMap<OrderReference, WorkingOrder> {
        self.orders.borrow()
    }
}

// Assumes prices is never empty this must be checked elsewhere we always need to be able to create
fn create_opening_range_from_ohlcs(prices: &Vec<OhlcPrice>) -> OpeningRange {
    let or_bar = prices.get(0).expect("There should always be one element");
    let mut highest_ask = 0.;
    let mut highest_bid = 0.;
    let mut lowest_ask = 1000000.; // We assume no epic will have a low higher then this
    let mut lowest_bid = 1000000.; // We assume no epic will have a low higher then this
    for ohlc in prices {
        if ohlc.high.ask > highest_ask {
            highest_ask = ohlc.high.ask;
            highest_bid = ohlc.high.bid;
        }
        if ohlc.low.ask < lowest_ask {
            lowest_ask = ohlc.low.ask;
            lowest_bid = ohlc.low.bid;
        }
    }
    OpeningRange {
        high_ask: highest_ask,
        high_bid: highest_bid,
        low_ask: lowest_ask,
        low_bid: lowest_bid,
    }
}


#[cfg(test)]
mod tests {
    use crate::decider::system::{create_opening_range_from_ohlcs, OpeningRange, Setup, System, SystemFactory, SystemMachine};
    use crate::decider::{Command, Event, OrderEvent, OrderReference};
    use crate::models::{OhlcPrice, Price};
    use chrono::Utc;

    #[test]
    fn will_get_corect_opening_range() {
        let prices = vec![
            OhlcPrice {
                high: Price {bid: 2., ask: 3.,},
                low: Price {bid: 1., ask: 2.,},
                open: Price {bid: 2., ask: 3.,},
                close: Price {bid: 2., ask: 3.,},
            },
            OhlcPrice {
                high: Price {bid: 2., ask: 3.,},
                low: Price {bid: 2., ask: 3.,},
                open: Price {bid: 2., ask: 3.,},
                close: Price {bid: 2., ask: 3.,},
            },
            OhlcPrice {
                high: Price {bid: 4., ask: 5.,},
                low: Price {bid: 2., ask: 3.,},
                open: Price {bid: 2., ask: 3.,},
                close: Price {bid: 2., ask: 3.,},
            },
        ];
        let opening_range = create_opening_range_from_ohlcs(&prices);
        let a = 1;
    }
    #[test]
    fn it_works_basic() {
        let sut = SystemFactory::new(Default::default());
        let (result_state, commands) = sut.step(&Event::Order(
            OrderEvent::ConfirmationAmendedAccepted,
            OrderReference::BETWEEN_LONG,
        ));
        match result_state {
            System::Setup(SystemMachine { state: Setup, .. }) => {}
            _ => panic!("Wrong system state"),
        }
        match commands[..] {
            [] => {}
            _ => panic!("Wrong command"),
        }
    }

    #[test]
    fn over_long_to_exit() {
        let r = OrderReference::OVER_LONG;
        let (s, cs) = run_sequence_from_start(vec![
            e_market_inside_trading_hours(1., 2.),
            e_data(),
            e_market_inside_trading_hours(128., 130.),
            e_o_confirmation_open_accepted(r.clone()),
            e_o_position_open(r.clone()),
            e_market_inside_trading_hours(140., 141.),
            e_o_position_trailing_stop(r.clone()),
            e_o_position_close(r.clone()),
            e_market_inside_trading_hours(140., 141.),
        ]);
        // Assert state
        match s {
            System::ManageOrders(_) => {}
            _ => panic!("Wrong system state {:#?}", s),
        }
        // Assert commands
        match cs[..] {
            [] => {}
            _ => panic!("Wrong command"),
        }
    }

    pub fn run_sequence_from_start(events: Vec<Event>) -> (System, Vec<Command>) {
        let mut sut = SystemFactory::new(Default::default());
        let mut commands: Vec<Command> = vec![];

        for event in &events {
            let (new_sut, cs) = sut.step(event);
            sut = new_sut;
            commands.extend(cs);
        }
        (sut, commands)
    }

    fn e_market_inside_trading_hours(bid: f64, ask: f64) -> Event {
        Event::Market {
            epic: "".to_string(),
            update_time: Utc::now(),
            bid,
            ask,
        }
    }

    fn e_data() -> Event {
        Event::Data {
            prices: vec![OhlcPrice {
                high: Price {
                    bid: 98.0,
                    ask: 100.0,
                },
                open: Price {
                    bid: 71.0,
                    ask: 72.0,
                },
                close: Price {
                    bid: 72.0,
                    ask: 73.0,
                },
                low: Price {
                    bid: 68.0,
                    ask: 70.0,
                },
            }],
        }
    }

    fn e_o_confirmation_open_accepted(reference: OrderReference) -> Event {
        Event::Order(
            OrderEvent::ConfirmationOpenAccepted { level: 0.0, deal_id: "".to_string() },
            reference,
        )
    }

    fn e_o_position_open(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::PositionEntry { entry_level: 22. }, reference)
    }

    fn e_o_position_trailing_stop(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::ConfirmationAmendedAccepted, reference)
    }

    fn e_o_position_close(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::PositionExit { exit_level: 23. }, reference)
    }
}
