use std::marker::PhantomData;
use crate::decider::{Command, Event};

struct WorkingOrderMachine<S> {
    state: PhantomData<S>,
}

struct AwaitingWOOpenConfirmation;
struct WOOpenRejected;
struct WOOpenAccepted;
struct PositionOpened;
struct AwaitingTrailingStopConfirmation;
struct PositionTrailingStopRejected;
struct PositionTrailingStopAccepted;
struct AwaitingWOCloseConfirmation;
struct WOCloseRejected;
struct WOCloseAccepted;
struct PositionExited;

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
    pub fn step(self, event: Event) -> (Self, Vec<Command>) {
        match (self, event) {
            (WorkingOrder::AwaitingWOOpenConfirmation(val), Event::ConfirmationOpenAccepted) => {
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
