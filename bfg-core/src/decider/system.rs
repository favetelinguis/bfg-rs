use crate::decider::order::WorkingOrder::{PositionExited, WOCloseAccepted, WOOpenRejected};
use crate::decider::order::{WorkingOrder, WorkingOrderFactory};
use crate::decider::order_manager::OrderManager;
use crate::decider::{Command, Event, MarketInfo, OrderEvent, OrderReference};
use crate::models::OhlcPrice;
use crate::{Direction, OrderReference};
use chrono::{Duration, NaiveDateTime, NaiveTime, Utc};

#[derive(Debug)]
pub struct SystemMachine<S> {
    pub state: S,
    pub market_info: MarketInfo,
}

#[derive(Debug)]
pub struct Setup;
#[derive(Debug)]
pub struct AwaitData;
#[derive(Debug)]
pub struct DataFailure;
#[derive(Debug)]
pub struct DecideOrderPlacement {
    pub opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct ManageLongOrder {
    pub opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct ManageShortOrder {
    pub opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct ManageLongAndShortOrder {
    pub opening_range: OpeningRange,
}

// Starting state
impl SystemMachine<Setup> {
    fn new(market_info: MarketInfo) -> Self {
        SystemMachine {
            state: Setup,
            market_info,
        }
    }
}

impl From<SystemMachine<Setup>> for SystemMachine<AwaitData> {
    fn from(val: SystemMachine<Setup>) -> Self {
        Self {
            state: AwaitData,
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<AwaitData>> for SystemMachine<DataFailure> {
    fn from(val: SystemMachine<AwaitData>) -> Self {
        Self {
            state: DataFailure,
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<AwaitData>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<AwaitData>) -> Self {
        // Shortcoming of using from trait, cant create opening_range just give dummy value
        // I will not create DecideOrderPlacement with the from trait i will create manually, just need this to
        // propagate opening_range.
        Self {
            state: DecideOrderPlacement {
                opening_range: Default::default(),
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: ManageLongOrder {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongAndShortOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: ManageLongAndShortOrder {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageShortOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: ManageShortOrder {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<ManageLongOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageLongOrder>) -> Self {
        Self {
            state: DecideOrderPlacement {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<ManageLongAndShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageLongAndShortOrder>) -> Self {
        Self {
            state: DecideOrderPlacement {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<ManageShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageShortOrder>) -> Self {
        Self {
            state: DecideOrderPlacement {
                opening_range: val.state.opening_range,
            },
            market_info: val.market_info,
        }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<Setup> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self {
            state: Setup,
            market_info: val.market_info,
        }
    }
}

#[derive(Debug)]
pub struct Long(WorkingOrder);
#[derive(Debug)]
pub struct Short(WorkingOrder);

#[derive(Debug, Default)]
pub struct OpeningRange {
    high_ask: f64,
    high_bid: f64,
    low_ask: f64,
    low_bid: f64,
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
}

#[derive(Debug)]
pub enum System {
    Setup(SystemMachine<Setup>),
    AwaitData(SystemMachine<AwaitData>),
    AwaitDataFailure(SystemMachine<DataFailure>),
    DecideOrderPlacement(SystemMachine<DecideOrderPlacement>),
    ManageLong(SystemMachine<ManageLongOrder>, Long),
    ManageLongAndShort(SystemMachine<ManageLongAndShortOrder>, Long, Short),
    ManageShort(SystemMachine<ManageShortOrder>, Short),
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
            ) if val.market_info.is_inside_trading_hours(update_time) => (
                System::AwaitData(val.into()),
                vec![create_fetch_data_command()],
            ),
            // AwaitData -> DataFailure []
            (System::AwaitData(val), Event::Data { prices, .. }) if prices.is_empty() => {
                (System::AwaitDataFailure(val.into()), vec![])
            }
            // AwaitData -> DecideOrderPlacement []
            (System::AwaitData(val), Event::Data { prices, .. }) if !prices.is_empty() => {
                let new_state = create_decide_order_placement_from_opening_range(val, prices);
                (System::DecideOrderPlacement(new_state), vec![])
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

            // DecideOrderPlacement -> ManageLong [CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_over(&val.state.opening_range, *bid, *ask) =>
            {
                let command = Command::CreateWorkingOrder {
                    direction: Direction::BUY,
                    price: val.state.opening_range.high_ask,
                    reference: OrderReference::OVER_LONG,
                };
                (
                    System::ManageLong(val.into(), Long(WorkingOrderFactory::new())),
                    vec![command],
                )
            }
            // DecideOrderPlacement -> ManageLongAndShortOrder [CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_between(&val.state.opening_range, *bid, *ask) =>
            {
                let commands = vec![
                    Command::CreateWorkingOrder {
                        direction: Direction::BUY,
                        price: val.state.opening_range.low_ask,
                        reference: OrderReference::BETWEEN_LONG,
                    },
                    Command::CreateWorkingOrder {
                        direction: Direction::SELL,
                        price: val.state.opening_range.high_bid,
                        reference: OrderReference::BETWEEN_SHORT,
                    },
                ];
                (
                    System::ManageLongAndShort(
                        val.into(),
                        Long(WorkingOrderFactory::new()),
                        Short(WorkingOrderFactory::new()),
                    ),
                    commands,
                )
            }
            // DecideOrderPlacement -> ManageShortOrder [CreateWorkingOrder]
            (
                System::DecideOrderPlacement(val),
                Event::Market {
                    ref update_time,
                    bid,
                    ask,
                    ..
                },
            ) if val.market_info.is_inside_trading_hours(update_time)
                && is_price_under(&val.state.opening_range, *bid, *ask) =>
            {
                let command = Command::CreateWorkingOrder {
                    direction: Direction::SELL,
                    price: val.state.opening_range.low_bid,
                    reference: OrderReference::UNDER_SHORT,
                };
                (
                    System::ManageShort(val.into(), Short(WorkingOrderFactory::new())),
                    vec![command],
                )
            }
            // ManageLong -> DecideOrderPlacement []
            (System::ManageLong(val, Long(PositionExited(_) | WOOpenRejected(_))), _) => {
                (System::DecideOrderPlacement(val.into()), vec![])
            }

            // ManageLong -> ManageLong [...]
            (
                System::ManageLong(val, Long(long)),
                ref event @ (Event::Market { .. } | Event::Order(_, OrderReference::OVER_LONG)),
            ) => {
                let (long, commands) = long.step(event);
                (System::ManageLong(val, Long(long)), commands)
            }
            // ManageShort -> DecideOrderPlacement []
            (
                System::ManageShort(val, Short(PositionExited(_) | WOOpenRejected(_))),
                Event::Market { .. },
            ) => (System::DecideOrderPlacement(val.into()), vec![]),
            // ManageShort -> ManageShort [...]
            (
                System::ManageShort(val, Short(short)),
                ref event @ (Event::Market { .. } | Event::Order(_, OrderReference::UNDER_SHORT)),
            ) => {
                let (short, commands) = short.step(event);
                (System::ManageShort(val, Short(short)), commands)
            }
            // ManageLongAndShort -> DecideOrderPlacement []
            (
                System::ManageLongAndShort(
                    val,
                    Long(PositionExited(_) | WOOpenRejected(_) | WOCloseAccepted(_)),
                    Short(PositionExited(_) | WOOpenRejected(_) | WOCloseAccepted(_)),
                ),
                Event::Market { .. },
            ) => (System::DecideOrderPlacement(val.into()), vec![]),
            // ManageLongAndShort -> ManageLongAndShort []
            (
                System::ManageLongAndShort(val, Long(mut long), Short(mut short)),
                ref event @ (Event::Market { .. }
                | Event::Order(
                    _,
                    OrderReference::BETWEEN_LONG | OrderReference::BETWEEN_SHORT,
                )),
            ) => {
                let mut commands = vec![];
                match event {
                    Event::Order(_, OrderReference::BETWEEN_LONG) => {
                        let (new_long, long_commands) = long.step(event);
                        long = new_long;
                        commands.extend(long_commands);
                    }
                    Event::Order(_, OrderReference::BETWEEN_SHORT) => {
                        let (new_short, short_commands) = short.step(event);
                        short = new_short;
                        commands.extend(short_commands);
                    }
                    Event::Market { .. } => {
                        let (new_long, long_commands) = long.step(event);
                        long = new_long;
                        let (new_short, short_commands) = short.step(event);
                        short = new_short;
                        commands.extend(long_commands);
                        commands.extend(short_commands);
                    }
                    _ => unreachable!(),
                }
                (
                    System::ManageLongAndShort(val, Long(long), Short(short)),
                    commands,
                )
            }
            (val, _) => (val, vec![]),
        }
    }
}

fn create_decide_order_placement_from_opening_range(
    val: SystemMachine<AwaitData>,
    prices: &Vec<OhlcPrice>,
) -> SystemMachine<DecideOrderPlacement> {
    let or_bar = prices.get(0).expect("There should always be one element");
    let opening_range = OpeningRange {
        high_ask: or_bar.high.ask,
        high_bid: or_bar.high.bid,
        low_ask: or_bar.low.ask,
        low_bid: or_bar.low.bid,
    };
    let mut new_state: SystemMachine<DecideOrderPlacement> = val.into();
    new_state.state.opening_range = opening_range;
    new_state
}

/// Get the data for the first open minute
fn create_fetch_data_command() -> Command {
    let now = Utc::now();
    let start_time = NaiveTime::from_hms(9, 0, 0);
    let end_time = NaiveTime::from_hms(9, 1, 0);
    let dt_start = NaiveDateTime::new(now.naive_utc().date(), start_time);
    let dt_start_format = dt_start.format("%Y-%m-%d %H:%M:%S").to_string();
    let dt_end = NaiveDateTime::new(now.naive_utc().date(), end_time);
    let dt_end_format = dt_end.format("%Y-%m-%d %H:%M:%S").to_string();
    Command::FetchData {
        start: dt_start_format,
        end: dt_end_format,
    }
}

fn is_price_over(opening_range: &OpeningRange, bid: f64, ask: f64) -> bool {
    let level = (bid + ask) / 2.;
    let buffer = 10.;
    level > (opening_range.get_middle_price_high() + buffer)
}

/// Opening range must be 15pips to ever trigger between
/// The buffer will always leave 4 pip in the middle where we will trigger
fn is_price_between(opening_range: &OpeningRange, bid: f64, ask: f64) -> bool {
    let level = (bid + ask) / 2.;
    let buffer = opening_range.range_size() / 2. - 2.;
    let or_large_enough = opening_range.range_size() >= 15.;
    let price_between = (level < (opening_range.get_middle_price_high() - buffer))
        && (level > (opening_range.get_middle_price_low() + buffer));
    or_large_enough && price_between
}

fn is_price_under(opening_range: &OpeningRange, bid: f64, ask: f64) -> bool {
    let level = (bid + ask) / 2.;
    let buffer = 10.;
    level < (opening_range.get_middle_price_low() - buffer)
}

pub struct SystemFactory;

impl SystemFactory {
    pub fn new(market_info: MarketInfo) -> System {
        System::Setup(SystemMachine::new(market_info))
    }
}

#[cfg(test)]
mod tests {
    use crate::decider::system::{Long, Setup, System, SystemFactory, SystemMachine};
    use crate::decider::{Command, Event, OrderEvent, OrderReference};
    use chrono::Utc;
    use crate::decider::order::WorkingOrder;
    use crate::models::{OhlcPrice, Price};

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
            System::ManageLong(_, Long(WorkingOrder::WOOpenAccepted(_))) => {}
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
            update_time: Utc::now().time(),
            bid,
            ask,
        }
    }

    fn e_data() -> Event {
        Event::Data {
            prices: vec![OhlcPrice {
                high: Price { bid: 98.0, ask: 100.0 },
                open: Price { bid: 71.0, ask: 72.0 },
                close: Price { bid: 72.0, ask: 73.0 },
                low: Price { bid: 68.0, ask: 70.0 }
            }]
        }
    }

    fn e_o_confirmation_open_accepted(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::ConfirmationOpenAccepted { deal_id: "".to_string(), level: 0.0 }, reference)
    }

    fn e_o_position_open(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::PositionEntry {entry_level: 22.}, reference)
    }

    fn e_o_position_trailing_stop(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::ConfirmationAmendedAccepted, reference)
    }

    fn e_o_position_close(reference: OrderReference) -> Event {
        Event::Order(OrderEvent::PositionExit {exit_level: 23.}, reference)
    }
}
