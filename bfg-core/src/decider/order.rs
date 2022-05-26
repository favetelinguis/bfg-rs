use std::marker::PhantomData;
use crate::decider::{Command, Event, OrderEvent};

pub struct WorkingOrderMachine<S> {
    state: PhantomData<S>,
}

pub struct AwaitingWOOpenConfirmation;
pub struct WOOpenRejected;
pub struct WOOpenAccepted;
pub struct PositionOpened;
pub struct AwaitingTrailingStopConfirmation;
pub struct PositionTrailingStopRejected;
pub struct PositionTrailingStopAccepted;
pub struct AwaitingWOCloseConfirmation;
pub struct WOCloseRejected;
pub struct WOCloseAccepted;
pub struct PositionExited;

impl From<WorkingOrderMachine<AwaitingWOOpenConfirmation>> for WorkingOrderMachine<WOOpenAccepted> {
    fn from(_: WorkingOrderMachine<AwaitingWOOpenConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<AwaitingWOOpenConfirmation>> for WorkingOrderMachine<WOOpenRejected> {
    fn from(_: WorkingOrderMachine<AwaitingWOOpenConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<WOOpenAccepted>> for WorkingOrderMachine<PositionOpened> {
    fn from(_: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<PositionOpened>> for WorkingOrderMachine<AwaitingTrailingStopConfirmation> {
    fn from(_: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>> for WorkingOrderMachine<PositionTrailingStopAccepted> {
    fn from(_: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>> for WorkingOrderMachine<PositionTrailingStopRejected> {
    fn from(_: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<PositionTrailingStopAccepted>> for WorkingOrderMachine<PositionExited> {
    fn from(_: WorkingOrderMachine<PositionTrailingStopAccepted>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

impl From<WorkingOrderMachine<WOOpenAccepted>> for WorkingOrderMachine<AwaitingWOCloseConfirmation> {
    fn from(_: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}
impl From<WorkingOrderMachine<AwaitingWOCloseConfirmation>> for WorkingOrderMachine<WOCloseAccepted> {
    fn from(_: WorkingOrderMachine<AwaitingWOCloseConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}
impl From<WorkingOrderMachine<AwaitingWOCloseConfirmation>> for WorkingOrderMachine<WOCloseRejected> {
    fn from(_: WorkingOrderMachine<AwaitingWOCloseConfirmation>) -> Self {
        Self {
            state: PhantomData,
        }
    }
}

// Starting state
impl WorkingOrderMachine<AwaitingWOOpenConfirmation> {
    fn new() -> Self {
        Self {
            state: PhantomData,
        }
    }
}

pub enum WorkingOrder {
    AwaitingWOOpenConfirmation(WorkingOrderMachine<AwaitingWOOpenConfirmation>),
    WOOpenRejected(WorkingOrderMachine<WOOpenRejected>),
    WOOpenAccepted(WorkingOrderMachine<WOOpenAccepted>),
    PositionOpened(WorkingOrderMachine<PositionOpened>),
    AwaitingTrailingStopConfirmation(WorkingOrderMachine<AwaitingTrailingStopConfirmation>),
    PositionTrailingStopRejected(WorkingOrderMachine<PositionTrailingStopRejected>),
    PositionTrailingStopAccepted(WorkingOrderMachine<PositionTrailingStopAccepted>),
    AwaitingWOCloseConfirmation(WorkingOrderMachine<AwaitingWOCloseConfirmation>),
    WOCloseRejected(WorkingOrderMachine<WOCloseRejected>),
    WOCloseAccepted(WorkingOrderMachine<WOCloseAccepted>),
    PositionExited(WorkingOrderMachine<PositionExited>),
}

impl WorkingOrder {
    pub fn step(self, event: &Event) -> (Self, Vec<Command>) {
        match (self, event) {
            (WorkingOrder::AwaitingWOOpenConfirmation(val), Event::Order(OrderEvent::ConfirmationOpenAccepted, _)) => {
                (WorkingOrder::WOOpenAccepted(val.into()), vec![])
            },
            // TODO fillout with rest???
            (val, _) => (val, vec![])
        }
    }
}

pub struct WorkingOrderFactory;

impl WorkingOrderFactory {
    pub fn new() -> WorkingOrder {
             WorkingOrder::AwaitingWOOpenConfirmation(WorkingOrderMachine::new())
    }
}



// Alternative test
pub struct AMachine<S> {
    state: PhantomData<S>,
}

pub struct S1;
pub struct S2;
pub struct S3;

pub struct A1;
pub struct A2;
pub struct A3;

trait Transition<S, A> {
    fn transition(state: S, action: A) -> Self;
}

impl Transition<AMachine<S1>, A1> for AMachine<S2> {
    fn transition(state: AMachine<S1>, action: A1) -> Self { // Do we need a result, should we also return Command?
        // Do logic
        Self {
            state: PhantomData,
        }
    }
}

// Make enum wrapper to make nice types for channels etc to send any action
pub enum ActionWrapper {
    A1(A1),
    A2(A2),
    A3(A3),
}