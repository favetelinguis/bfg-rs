use std::borrow::{Borrow, BorrowMut};
use crate::decider::order::{WorkingOrder, WorkingOrderFactory};
use crate::decider::{Command, Event};
use std::marker::PhantomData;
use crate::decider::order::WorkingOrder::{PositionExited, WOCloseAccepted, WOOpenRejected};

struct SystemMachine<S> {
    state: PhantomData<S>,
}

struct Setup;
struct DecideOrderPlacement;
struct ManageLongOrder;
struct ManageShortOrder;
struct ManageLongAndShortOrder;

// Starting state
impl SystemMachine<Setup> {
    fn new() -> Self {
        SystemMachine { state: PhantomData }
    }
}

impl From<SystemMachine<Setup>> for SystemMachine<DecideOrderPlacement> {
    fn from(_: SystemMachine<Setup>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongOrder> {
    fn from(_: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageLongAndShortOrder> {
    fn from(_: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<ManageShortOrder> {
    fn from(_: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<ManageLongOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(_: SystemMachine<ManageLongOrder>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<ManageLongAndShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(_: SystemMachine<ManageLongAndShortOrder>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<ManageShortOrder>> for SystemMachine<DecideOrderPlacement> {
    fn from(_: SystemMachine<ManageShortOrder>) -> Self {
        Self { state: PhantomData }
    }
}

impl From<SystemMachine<DecideOrderPlacement>> for SystemMachine<Setup> {
    fn from(_: SystemMachine<DecideOrderPlacement>) -> Self {
        Self { state: PhantomData }
    }
}

struct Long(WorkingOrder);
struct Short(WorkingOrder);

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
                System::ManageLongAndShort(val.into(), Long(WorkingOrderFactory::new()), Short(WorkingOrderFactory::new())),
                vec![Command::CreateWorkingOrder],
            ),
            // DecideOrderPlacement -> ManageShortOrder [CreateWorkingOrder]
            (System::DecideOrderPlacement(val), Event::Market()) if is_price_under() => (
                System::ManageShort(val.into(), Short(WorkingOrderFactory::new())),
                vec![Command::CreateWorkingOrder],
            ),
            // ManageLong -> DecideOrderPlacement []
            (System::ManageLong(val, Long(PositionExited(_) | WOOpenRejected(_))), _) => {
                    (
                        System::DecideOrderPlacement(val.into()),
                        vec![],
                    )
            },
            // ManageLong -> ManageLong [...]
            (System::ManageLong(val, Long(long)), event) if is_manage_long_event(event.borrow()) => {
                let (long, commands) = long.step(event);
                (
                    System::ManageLong(val.into(), Long(long)),
                    commands,
                )
            },
            // ManageShort -> DecideOrderPlacement []
            (System::ManageShort(val, Short(PositionExited(_) | WOOpenRejected(_))), _) => {
                (
                    System::DecideOrderPlacement(val.into()),
                    vec![],
                )
            },
            // ManageShort -> ManageShort [...]
            (System::ManageShort(val, Short(short)), event) if is_manage_short_event(event.borrow()) => {
                let (short, commands) = short.step(event);
                (
                    System::ManageShort(val.into(), Short(short)),
                    commands,
                )
            },
            // ManageLongAndShort -> DecideOrderPlacement []
            (System::ManageLongAndShort(val, Long(PositionExited(_) | WOOpenRejected(_) | WOCloseAccepted(_)), Short(PositionExited(_) | WOOpenRejected(_) | WOCloseAccepted(_))), _) => {
                (
                    System::DecideOrderPlacement(val.into()),
                    vec![],
                )
            },
            // ManageLongAndShort -> ManageLongAndShort []
            (System::ManageLongAndShort(val, Long(mut long), Short(mut short)), event) if is_manage_long_and_short_event(event.borrow()) => {
                let mut commands: Vec<Command> = vec![];
                if is_between_long_event(event.borrow()) {
                    let (new_long, mut long_commands) = long.step(event);
                    long = new_long;
                    commands.borrow_mut().append(long_commands);

                } else {
                    let (new_short, short_commands) = short.step(event);
                    long = new_short;
                    commands.borrow_mut().append(short_commands);
                }
                (
                    System::ManageLongAndShort(val.into(), Long(long), Short(short)),
                    commands,
                )
            },
            (val, _) => (val, vec![]),
        }
    }
}

fn is_between_long_event(event: &Event) -> bool {
    // Use large pattern match with if let to return tru else false
    todo!()
}

fn is_manage_long_event(event: &Event) -> bool {
    // Use large pattern match with if let to return tru else false
    todo!()
}

fn is_manage_short_event(event: &Event) -> bool {
    todo!()
}

fn is_manage_long_and_short_event(event: &Event) -> bool {
    todo!()
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
    pub fn new() -> System {
        System::Setup(SystemMachine::new())
    }
}
