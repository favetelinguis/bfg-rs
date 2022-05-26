use crate::decider::order::WorkingOrder::{PositionExited, WOCloseAccepted, WOOpenRejected};
use crate::decider::order::{WorkingOrder, WorkingOrderFactory};
use crate::decider::{Command, Event, MarketInfo, OrderReference};
use std::marker::PhantomData;

pub struct SystemMachine<S> {
    state: PhantomData<S>,
    market_info: MarketInfo,
}

pub struct Setup;
pub struct DecideOrderPlacement;
pub struct ManageLongOrder;
pub struct ManageShortOrder;
pub struct ManageLongAndShortOrder;

// Starting state
impl SystemMachine<Setup> {
    fn new(market_info: MarketInfo) -> Self {
        SystemMachine { state: PhantomData, market_info }
    }
}

impl From<SystemMachine<Setup>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<Setup>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongAndShortOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageShortOrder> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<ManageLongOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageLongOrder>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<ManageLongAndShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageLongAndShortOrder>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<ManageShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(val: SystemMachine<ManageShortOrder>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<Setup> {
    fn from(val: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData, market_info: val.market_info}
    }
}

pub struct Long(WorkingOrder);
pub struct Short(WorkingOrder);

pub enum System {
    Setup(SystemMachine<Setup>),
    DecideOrderPlacement(SystemMachine<DecideOrderPlacement>),
    ManageLong(SystemMachine<ManageLongOrder>, Long),
    ManageLongAndShort(SystemMachine<ManageLongAndShortOrder>, Long, Short),
    ManageShort(SystemMachine<ManageShortOrder>, Short),
}

impl System {
    fn step(self, event: Event) -> (System, Vec<Command>) {
        match (self, event) {
            // Setup -> DecideOrderPlacement []
            (System::Setup(val), Event::Market()) if is_setup_complete() => {
                (System::DecideOrderPlacement(val.into()), vec![])
            }
            // DecideOrderPlacement -> ManageLong [CreateWorkingOrder]
            (System::DecideOrderPlacement(val), Event::Market()) if is_price_over() => (
                System::ManageLong(val.into(), Long(WorkingOrderFactory::new())),
                vec![Command::CreateWorkingOrder],
            ),
            // DecideOrderPlacement -> ManageLongAndShortOrder [CreateWorkingOrder]
            (System::DecideOrderPlacement(val), Event::Market()) if is_price_between() => (
                System::ManageLongAndShort(
                    val.into(),
                    Long(WorkingOrderFactory::new()),
                    Short(WorkingOrderFactory::new()),
                ),
                vec![Command::CreateWorkingOrder],
            ),
            // DecideOrderPlacement -> ManageShortOrder [CreateWorkingOrder]
            (System::DecideOrderPlacement(val), Event::Market()) if is_price_under() => (
                System::ManageShort(val.into(), Short(WorkingOrderFactory::new())),
                vec![Command::CreateWorkingOrder],
            ),
            // ManageLong -> DecideOrderPlacement []
            (System::ManageLong(val, Long(PositionExited(_) | WOOpenRejected(_))), _) => {
                (System::DecideOrderPlacement(val.into()), vec![])
            }
            // ManageLong -> ManageLong [...]
            (
                System::ManageLong(val, Long(long)),
                ref event @ (Event::Market() | Event::Order(_, OrderReference::OVER_LONG)),
            ) => {
                let (long, commands) = long.step(event);
                (System::ManageLong(val, Long(long)), commands)
            }
            // ManageShort -> DecideOrderPlacement []
            (
                System::ManageShort(val, Short(PositionExited(_) | WOOpenRejected(_))),
                Event::Market(),
            ) => (System::DecideOrderPlacement(val.into()), vec![]),
            // ManageShort -> ManageShort [...]
            (
                System::ManageShort(val, Short(short)),
                ref event @ (Event::Market() | Event::Order(_, OrderReference::UNDER_SHORT)),
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
                Event::Market(),
            ) => (System::DecideOrderPlacement(val.into()), vec![]),
            // ManageLongAndShort -> ManageLongAndShort []
            (
                System::ManageLongAndShort(val, Long(mut long), Short(mut short)),
                ref event @ (Event::Market()
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
                    Event::Market() => {
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

fn is_setup_complete() -> bool {
    true
}

fn is_price_over() -> bool {
    true
}

fn is_price_between() -> bool {
    true
}

fn is_price_under() -> bool {
    true
}

pub struct SystemFactory;

impl SystemFactory {
    pub fn new(market_info: MarketInfo) -> System {
        System::Setup(SystemMachine::new(market_info))
    }
}
