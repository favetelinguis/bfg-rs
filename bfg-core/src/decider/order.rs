use std::borrow::Borrow;
use chrono::{DateTime, Utc};
use crate::decider::{Command, Event, OrderEvent, OrderReference};
use crate::{DealStatus, Direction};

#[derive(Debug)]
pub struct WorkingOrderMachine<S> {
    state: S,
}

#[derive(Debug)]
pub struct AwaitingWOOpenConfirmation;
#[derive(Debug)]
pub struct WOOpenRejected;
#[derive(Debug)]
pub struct WOOpenAccepted {pub wanted_entry_level: f64, reference: OrderReference, deal_id: String}
#[derive(Debug)]
pub struct PositionOpened {pub wanted_entry_level: f64, pub actual_entry_level: f64, pub entry_time: DateTime<Utc>, reference: OrderReference, deal_id: String}
#[derive(Debug)]
pub struct AwaitingTrailingStopConfirmation{pub wanted_entry_level: f64, pub actual_entry_level: f64, pub entry_time: DateTime<Utc>, reference: OrderReference, deal_id: String}
#[derive(Debug)]
pub struct PositionTrailingStopRejected{pub wanted_entry_level: f64, pub actual_entry_level: f64, pub entry_time: DateTime<Utc>, reference: OrderReference, deal_id: String}
#[derive(Debug)]
pub struct PositionTrailingStopAccepted{pub wanted_entry_level: f64, pub actual_entry_level: f64, pub entry_time: DateTime<Utc>, reference: OrderReference, deal_id: String}
#[derive(Debug)]
pub struct AwaitingWOCloseConfirmation;
#[derive(Debug)]
pub struct WOCloseRejected;
#[derive(Debug)]
pub struct WOCloseAccepted;
#[derive(Debug)]
pub struct PositionExited{pub wanted_entry_level: f64, pub actual_entry_level: f64,pub entry_time: DateTime<Utc>,pub exit_time: DateTime<Utc>, pub exit_level: f64, reference: OrderReference, deal_id: String}

impl From<WorkingOrderMachine<AwaitingWOOpenConfirmation>> for WorkingOrderMachine<WOOpenAccepted> {
    fn from(_: WorkingOrderMachine<AwaitingWOOpenConfirmation>) -> Self {
        Self {
            state: WOOpenAccepted {wanted_entry_level: Default::default(), reference: OrderReference::BETWEEN_LONG, deal_id: Default::default()},
        }
    }
}

impl From<WorkingOrderMachine<AwaitingWOOpenConfirmation>> for WorkingOrderMachine<WOOpenRejected> {
    fn from(_: WorkingOrderMachine<AwaitingWOOpenConfirmation>) -> Self {
        Self {
            state: WOOpenRejected,
        }
    }
}

impl From<WorkingOrderMachine<WOOpenAccepted>> for WorkingOrderMachine<PositionOpened> {
    fn from(val: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: PositionOpened {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: Default::default(),
                entry_time: Utc::now(),
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<PositionOpened>> for WorkingOrderMachine<AwaitingTrailingStopConfirmation> {
    fn from(val: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: AwaitingTrailingStopConfirmation {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>> for WorkingOrderMachine<PositionTrailingStopAccepted> {
    fn from(val: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PositionTrailingStopAccepted {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>> for WorkingOrderMachine<PositionTrailingStopRejected> {
    fn from(val: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PositionTrailingStopRejected {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>> for WorkingOrderMachine<PositionExited> {
    fn from(val: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PositionExited {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                exit_time: Utc::now(),
                exit_level: Default::default(),
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<PositionTrailingStopAccepted>> for WorkingOrderMachine<PositionExited> {
    fn from(val: WorkingOrderMachine<PositionTrailingStopAccepted>) -> Self {
        Self {
            state: PositionExited {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                exit_time: Utc::now(),
                exit_level: Default::default(),
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<PositionOpened>> for WorkingOrderMachine<PositionExited> {
    fn from(val: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: PositionExited {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                exit_time: Utc::now(),
                exit_level: Default::default(),
                reference: val.state.reference,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<WOOpenAccepted>> for WorkingOrderMachine<AwaitingWOCloseConfirmation> {
    fn from(_: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: AwaitingWOCloseConfirmation,
        }
    }
}
impl From<WorkingOrderMachine<PositionOpened>> for WorkingOrderMachine<WOCloseAccepted> {
    fn from(_: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: WOCloseAccepted,
        }
    }
}
impl From<WorkingOrderMachine<PositionOpened>> for WorkingOrderMachine<WOCloseRejected> {
    fn from(_: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: WOCloseRejected,
        }
    }
}

// Starting state
impl WorkingOrderMachine<AwaitingWOOpenConfirmation> {
    fn new() -> Self {
        Self {
            state: AwaitingWOOpenConfirmation,
        }
    }
}

#[derive(Debug)]
pub enum WorkingOrder {
    AwaitingWOOpenConfirmation(WorkingOrderMachine<AwaitingWOOpenConfirmation>),
    WOOpenRejected(WorkingOrderMachine<WOOpenRejected>),
    WOOpenAccepted(WorkingOrderMachine<WOOpenAccepted>),
    PositionOpened(WorkingOrderMachine<PositionOpened>),
    AwaitingTrailingStopConfirmation(WorkingOrderMachine<AwaitingTrailingStopConfirmation>),
    PositionTrailingStopRejected(WorkingOrderMachine<PositionTrailingStopRejected>),
    PositionTrailingStopAccepted(WorkingOrderMachine<PositionTrailingStopAccepted>),
    WOCloseRejected(WorkingOrderMachine<WOCloseRejected>),
    WOCloseAccepted(WorkingOrderMachine<WOCloseAccepted>),
    PositionExited(WorkingOrderMachine<PositionExited>),
}

impl WorkingOrder {
    pub fn step(self, event: &Event) -> (Self, Vec<Command>) {
        match (self, event) {
            (WorkingOrder::AwaitingWOOpenConfirmation(val), Event::Order(OrderEvent::ConfirmationOpenAccepted {level, deal_id}, reference)) => {
                let mut new_state: WorkingOrderMachine<WOOpenAccepted> = val.into();
                new_state.state.wanted_entry_level = level.clone();
                new_state.state.reference = reference.clone();
                new_state.state.deal_id = deal_id.clone();
                (WorkingOrder::WOOpenAccepted(new_state), vec![])
            },
            (WorkingOrder::AwaitingWOOpenConfirmation(val), Event::Order(OrderEvent::ConfirmationOpenRejected,_)) => {
                (WorkingOrder::WOOpenRejected(val.into()), vec![])
            },
            (WorkingOrder::WOOpenAccepted(val), Event::Order(OrderEvent::PositionOpen {entry_level}, OrderReference::OVER_LONG | OrderReference::UNDER_SHORT)) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = entry_level.clone();
                (WorkingOrder::PositionOpened(new_state), vec![])
            },
            (WorkingOrder::WOOpenAccepted(val), Event::Order(OrderEvent::PositionOpen {entry_level}, OrderReference::BETWEEN_SHORT)) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = entry_level.clone();
                (WorkingOrder::PositionOpened(new_state), vec![Command::CancelWorkingOrder {reference_to_cancel: OrderReference::BETWEEN_LONG}])
            },
            (WorkingOrder::WOOpenAccepted(val), Event::Order(OrderEvent::PositionOpen {entry_level}, OrderReference::BETWEEN_LONG)) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = entry_level.clone();
                (WorkingOrder::PositionOpened(new_state), vec![Command::CancelWorkingOrder {reference_to_cancel: OrderReference::BETWEEN_SHORT}])
            },
            (WorkingOrder::PositionOpened(val), Event::Order(OrderEvent::ConfirmationCloseAccepted, _)) => {
                (WorkingOrder::WOCloseAccepted(val.into()), vec![])
            },
            (WorkingOrder::PositionOpened(val), Event::Order(OrderEvent::ConfirmationCloseRejected, _)) => {
                (WorkingOrder::WOCloseRejected(val.into()), vec![])
            },
            (WorkingOrder::PositionOpened(val), Event::Market{bid, ask, ..}) if is_add_trailing_stop_triggered(bid, ask, val.state.reference.borrow(), val.state.actual_entry_level) => {
                let stop_distance;
                if let OrderReference::OVER_LONG | OrderReference::BETWEEN_LONG = val.state.reference.clone() {
                    stop_distance = 5.;
                } else {
                    stop_distance = -5.;
                }
                let command = Command::UpdatePosition {
                    level: val.state.actual_entry_level + stop_distance,
                    deal_id: val.state.deal_id.clone(),
                };
                (WorkingOrder::AwaitingTrailingStopConfirmation(val.into()), vec![command])
            },
            (WorkingOrder::AwaitingTrailingStopConfirmation(val), Event::Order(OrderEvent::ConfirmationAmendedAccepted, _)) => {
                (WorkingOrder::PositionTrailingStopAccepted(val.into()), vec![])
            },
            (WorkingOrder::AwaitingTrailingStopConfirmation(val), Event::Order(OrderEvent::ConfirmationAmendedRejected, _)) => {
                (WorkingOrder::PositionTrailingStopRejected(val.into()), vec![])
            },
            // If position is closed while waiting to order to update
            (WorkingOrder::AwaitingTrailingStopConfirmation(val), Event::Order(OrderEvent::PositionClose {exit_level}, _)) => {
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();

                let command = Command::PublishTradeResults {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                };
                (WorkingOrder::PositionExited(new_state), vec![command])
            },
            // If position with trailing stop
            (WorkingOrder::PositionTrailingStopAccepted(val), Event::Order(OrderEvent::PositionClose {exit_level}, _)) => {
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();
                let command = Command::PublishTradeResults {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                };
                (WorkingOrder::PositionExited(new_state), vec![command])
            },
            // Position is cloded before updating with trailing stop
            (WorkingOrder::PositionOpened(val), Event::Order(OrderEvent::PositionClose {exit_level}, _)) => {
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();
                let command = Command::PublishTradeResults {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                };
                (WorkingOrder::PositionExited(new_state), vec![command])
            },
            (val, _) => (val, vec![])
        }
    }
}

fn is_add_trailing_stop_triggered(bid: &f64, ask: &f64, reference: &OrderReference, level: f64) -> bool {
    if let OrderReference::BETWEEN_LONG | OrderReference::OVER_LONG = reference {
        bid.clone() > level
    } else {
        ask.clone() < level
    }
}

pub struct WorkingOrderFactory;

impl WorkingOrderFactory {
    pub fn new() -> WorkingOrder {
             WorkingOrder::AwaitingWOOpenConfirmation(WorkingOrderMachine::new())
    }
}
