use crate::decider::{Command, Event, MarketInfo, OrderEvent, OrderReference, TradeResult};
use chrono::{DateTime, Utc};
use std::borrow::Borrow;
use std::fmt::{Display, Formatter};
use crate::decider::system::OpeningRange;

#[derive(Debug)]
pub struct WorkingOrderMachine<S> {
    state: S,
}

#[derive(Debug)]
pub struct AwaitingWOOpenConfirmation {
    market_info: MarketInfo,
    opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct WOOpenRejected;
#[derive(Debug)]
pub struct WOOpenAccepted {
    pub wanted_entry_level: f64,
    reference: OrderReference,
    deal_id: String,
    market_info: MarketInfo,
    opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct PositionOpened {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: DateTime<Utc>,
    reference: OrderReference,
    deal_id: String,
    market_info: MarketInfo,
    opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct AwaitingTrailingStopConfirmation {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: DateTime<Utc>,
    reference: OrderReference,
    deal_id: String,
    market_info: MarketInfo,
    opening_range: OpeningRange,
}
#[derive(Debug)]
pub struct PositionTrailingStopAccepted {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: DateTime<Utc>,
    reference: OrderReference,
    deal_id: String,
    market_info: MarketInfo,
}
#[derive(Debug)]
pub struct WOCloseRejected;
#[derive(Debug)]
pub struct WOCloseAccepted;
#[derive(Debug)]
pub struct PositionExited {
    pub wanted_entry_level: f64,
    pub actual_entry_level: f64,
    pub entry_time: DateTime<Utc>,
    pub exit_time: DateTime<Utc>,
    pub exit_level: f64,
    reference: OrderReference,
    pub market_info: MarketInfo,
    deal_id: String,
}

impl From<WorkingOrderMachine<AwaitingWOOpenConfirmation>> for WorkingOrderMachine<WOOpenAccepted> {
    fn from(val: WorkingOrderMachine<AwaitingWOOpenConfirmation>) -> Self {
        Self {
            state: WOOpenAccepted {
                wanted_entry_level: Default::default(),
                reference: OrderReference::BETWEEN_LONG,
                market_info: val.state.market_info,
                deal_id: Default::default(),
                opening_range: val.state.opening_range,
            },
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
                market_info: val.state.market_info,
                deal_id: val.state.deal_id,
                opening_range: val.state.opening_range,
            },
        }
    }
}

impl From<WorkingOrderMachine<PositionOpened>>
    for WorkingOrderMachine<AwaitingTrailingStopConfirmation>
{
    fn from(val: WorkingOrderMachine<PositionOpened>) -> Self {
        Self {
            state: AwaitingTrailingStopConfirmation {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                deal_id: val.state.deal_id,
                market_info: val.state.market_info,
                opening_range: val.state.opening_range,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>>
    for WorkingOrderMachine<PositionTrailingStopAccepted>
{
    fn from(val: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PositionTrailingStopAccepted {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                market_info: val.state.market_info,
                deal_id: val.state.deal_id,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>>
    for WorkingOrderMachine<PositionOpened>
{
    fn from(val: WorkingOrderMachine<AwaitingTrailingStopConfirmation>) -> Self {
        Self {
            state: PositionOpened {
                wanted_entry_level: val.state.wanted_entry_level,
                actual_entry_level: val.state.actual_entry_level,
                entry_time: val.state.entry_time,
                reference: val.state.reference,
                deal_id: val.state.deal_id,
                market_info: val.state.market_info,
                opening_range: val.state.opening_range,
            },
        }
    }
}

impl From<WorkingOrderMachine<AwaitingTrailingStopConfirmation>>
    for WorkingOrderMachine<PositionExited>
{
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
                market_info: val.state.market_info,
            },
        }
    }
}

impl From<WorkingOrderMachine<PositionTrailingStopAccepted>>
    for WorkingOrderMachine<PositionExited>
{
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
                market_info: val.state.market_info,
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
                market_info: val.state.market_info,
            },
        }
    }
}

impl From<WorkingOrderMachine<WOOpenAccepted>>
    for WorkingOrderMachine<WOCloseAccepted>
{
    fn from(_: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: WOCloseAccepted,
        }
    }
}
impl From<WorkingOrderMachine<WOOpenAccepted>>
for WorkingOrderMachine<WOCloseRejected>
{
    fn from(_: WorkingOrderMachine<WOOpenAccepted>) -> Self {
        Self {
            state: WOCloseRejected,
        }
    }
}

// Starting state
impl WorkingOrderMachine<AwaitingWOOpenConfirmation> {
    fn new(market_info: MarketInfo, opening_range: OpeningRange) -> Self {
        Self {
            state: AwaitingWOOpenConfirmation {market_info, opening_range},
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
    PositionTrailingStopAccepted(WorkingOrderMachine<PositionTrailingStopAccepted>),
    WOCloseRejected(WorkingOrderMachine<WOCloseRejected>),
    WOCloseAccepted(WorkingOrderMachine<WOCloseAccepted>),
    PositionExited(WorkingOrderMachine<PositionExited>),
}

impl Display for WorkingOrder {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkingOrder::AwaitingWOOpenConfirmation(_) => write!(f, "AwaitingWOOpenConfirmation"),
            WorkingOrder::WOOpenRejected(_) => write!(f, "WOOpenRejected"),
            WorkingOrder::WOOpenAccepted(_) => write!(f, "WOOpenAccepted"),
            WorkingOrder::PositionOpened(_) => write!(f, "PositionOpened"),
            WorkingOrder::AwaitingTrailingStopConfirmation(_) => write!(f, "AwaitingTrailingStopConfirmation"),
            WorkingOrder::PositionTrailingStopAccepted(_) => write!(f, "PositionTrailingStopAccepted"),
            WorkingOrder::WOCloseRejected(_) => write!(f, "WOCloseRejected"),
            WorkingOrder::WOCloseAccepted(_) => write!(f, "WOCloseAccepted"),
            WorkingOrder::PositionExited(_) => write!(f, "PositionExited"),
        }
    }
}

impl WorkingOrder {
    pub fn step(self, event: &Event) -> (Self, Vec<Command>) {
        match (self, event) {
            // AwaitingWOOpenConfirmation -> WOOpenAccepted []
            (
                WorkingOrder::AwaitingWOOpenConfirmation(val),
                Event::Order(OrderEvent::ConfirmationOpenAccepted {level, deal_id }, reference),
            ) => {
                let mut new_state: WorkingOrderMachine<WOOpenAccepted> = val.into();
                new_state.state.wanted_entry_level = *level;
                new_state.state.reference = reference.clone();
                new_state.state.deal_id = deal_id.clone();
                (WorkingOrder::WOOpenAccepted(new_state), vec![])
            }
            // AwaitingWOOpenConfirmation -> WOOpenRejected []
            (
                WorkingOrder::AwaitingWOOpenConfirmation(val),
                Event::Order(OrderEvent::ConfirmationRejection, _),
            ) => (WorkingOrder::WOOpenRejected(val.into()), vec![]),
            // WOOpenAccepted -> PositionOpened []
            (
                WorkingOrder::WOOpenAccepted(val),
                Event::Order(
                    OrderEvent::PositionEntry { entry_level },
                    OrderReference::OVER_LONG | OrderReference::UNDER_SHORT,
                ),
            ) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = *entry_level;
                (WorkingOrder::PositionOpened(new_state), vec![])
            }
            // WOOpenAccepted -> PositionOpened [CancelWorkingOrder]
            (
                WorkingOrder::WOOpenAccepted(val),
                Event::Order(
                    OrderEvent::PositionEntry { entry_level },
                    OrderReference::BETWEEN_SHORT,
                ),
            ) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = *entry_level;
                (
                    WorkingOrder::PositionOpened(new_state),
                    vec![Command::CancelWorkingOrder {
                        reference_to_cancel: OrderReference::BETWEEN_LONG,
                    }],
                )
            }
            // WOOpenAccepted -> PositionOpened [CancelWorkingOrder]
            (
                WorkingOrder::WOOpenAccepted(val),
                Event::Order(
                    OrderEvent::PositionEntry { entry_level },
                    OrderReference::BETWEEN_LONG,
                ),
            ) => {
                let mut new_state: WorkingOrderMachine<PositionOpened> = val.into();
                new_state.state.actual_entry_level = *entry_level;
                (
                    WorkingOrder::PositionOpened(new_state),
                    vec![Command::CancelWorkingOrder {
                        reference_to_cancel: OrderReference::BETWEEN_SHORT,
                    }],
                )
            }
            // WOOpenAccepted -> WOCloseAccepted []
            (
                WorkingOrder::WOOpenAccepted(val),
                Event::Order(OrderEvent::ConfirmationDeleteAccepted, _),
            ) => (WorkingOrder::WOCloseAccepted(val.into()), vec![]),
            // PositionOpened -> WOCloseRejected []
            (
                WorkingOrder::WOOpenAccepted(val),
                Event::Order(OrderEvent::ConfirmationRejection, _),
            ) => (WorkingOrder::WOCloseRejected(val.into()), vec![]),
            // PositionOpened -> AwaitingTrailingStopConfirmation [UpdatePosition]
            (WorkingOrder::PositionOpened(val), Event::Market { bid, ask, .. })
                if is_add_trailing_stop_triggered(
                    bid,
                    ask,
                    val.state.reference.borrow(),
                    val.state.actual_entry_level,
                ) =>
            {
                let stop_distance= val.state.market_info.min_stop_distance;
                let direction_multiple;
                if let OrderReference::OVER_LONG | OrderReference::BETWEEN_LONG =
                val.state.reference.clone()
                {
                    direction_multiple = -1.;
                } else {
                    direction_multiple = 1.;
                }
                // We need to keep the fixed target when adding trailing stop if this is a between order
                let mut target = None;
                if let OrderReference::BETWEEN_SHORT =
                val.state.reference.clone()
                {
                    target = Some(val.state.opening_range.low_ask);
                };
                if let OrderReference::BETWEEN_LONG =
                val.state.reference.clone()
                {
                    target = Some(val.state.opening_range.high_bid);
                };
                let command = Command::UpdatePosition {
                    level: val.state.actual_entry_level + (stop_distance as f64 * direction_multiple),
                    deal_id: val.state.deal_id.clone(),
                    trailing_stop_distance: stop_distance,
                    target_level: target,
                };
                (
                    WorkingOrder::AwaitingTrailingStopConfirmation(val.into()),
                    vec![command],
                )
            }
            // AwaitingTrailingStopConfirmation -> PositionTrailingStopAccepted []
            (
                WorkingOrder::AwaitingTrailingStopConfirmation(val),
                Event::Order(OrderEvent::ConfirmationAmendedAccepted, _),
            ) => (
                WorkingOrder::PositionTrailingStopAccepted(val.into()),
                vec![],
            ),
            // AwaitingTrailingStopConfirmation -> PositionOpened (retry trailing stop)
            (
                WorkingOrder::AwaitingTrailingStopConfirmation(val),
                Event::Order(OrderEvent::ConfirmationRejection, _),
            ) => (
                WorkingOrder::PositionOpened(val.into()),
                vec![],
            ),
            // If position is closed while waiting to order to update
            (
                WorkingOrder::AwaitingTrailingStopConfirmation(val),
                Event::Order(OrderEvent::PositionExit { exit_level }, _),
            ) => {
                let epic = val.state.market_info.epic.clone();
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();

                let command = Command::PublishTradeResults(TradeResult {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                    epic,
                });
                (WorkingOrder::PositionExited(new_state), vec![command])
            }
            // If position with trailing stop
            (
                WorkingOrder::PositionTrailingStopAccepted(val),
                Event::Order(OrderEvent::PositionExit { exit_level }, _),
            ) => {
                let epic = val.state.market_info.epic.clone();
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();
                let command = Command::PublishTradeResults(TradeResult {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                    epic,
                });
                (WorkingOrder::PositionExited(new_state), vec![command])
            }
            // Position is cloded before updating with trailing stop
            (
                WorkingOrder::PositionOpened(val),
                Event::Order(OrderEvent::PositionExit { exit_level }, _),
            ) => {
                let epic = val.state.market_info.epic.clone();
                let mut new_state: WorkingOrderMachine<PositionExited> = val.into();
                new_state.state.exit_level = exit_level.clone();
                let command = Command::PublishTradeResults(TradeResult {
                    wanted_entry_level: new_state.state.wanted_entry_level,
                    actual_entry_level: new_state.state.actual_entry_level,
                    entry_time: new_state.state.entry_time.clone(),
                    exit_time: new_state.state.exit_time.clone(),
                    exit_level: new_state.state.exit_level,
                    reference: new_state.state.reference.clone(),
                    epic,
                });
                (WorkingOrder::PositionExited(new_state), vec![command])
            }
            (val, _) => (val, vec![]),
        }
    }
}

fn is_add_trailing_stop_triggered(
    bid: &f64,
    ask: &f64,
    reference: &OrderReference,
    level: f64,
) -> bool {
    if let OrderReference::BETWEEN_LONG | OrderReference::OVER_LONG = reference {
        bid.clone() > level
    } else {
        ask.clone() < level
    }
}

pub struct WorkingOrderFactory;

impl WorkingOrderFactory {
    pub fn new(market_info: MarketInfo, opening_range: OpeningRange) -> WorkingOrder {
        WorkingOrder::AwaitingWOOpenConfirmation(WorkingOrderMachine::new(market_info, opening_range))
    }
}
