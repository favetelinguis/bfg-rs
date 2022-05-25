extern crate core;

use crate::models::{
    BfgTradeStatus, ConfirmsStatus, DataUpdate, DealStatus, Decision, Direction, EntryMode,
    FetchDataDetails, LimitOrderDetails, MarketOrderDetails, MarketState, MarketUpdate, OrderState,
    PriceRelativeOr, SystemState, SystemValues, TradeConfirmation, TradeUpdate,
    WorkingOrderDetails, WorkingOrderPlacement, WorkingOrderReference, WorkingOrderSystemDetails,
    WorkingOrderUpdate,
};
use chrono::{NaiveDateTime, NaiveTime, Utc};
use log::{error, info, warn};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::str::FromStr;

pub mod models;
pub mod decider;

// TODO make ig::realtime emit the following events insted, will make pattern matching much clearer
#[derive(Clone, Debug)]
pub enum BfgEventV2 {
    ConfirmationOpenAccepted {deal_id: String, deal_reference: WorkingOrderReference, level: f64},
    ConfirmationOpenRejected {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationCloseAccepted {deal_reference: WorkingOrderReference},
    ConfirmationCloseRejected {deal_reference: WorkingOrderReference, reason: String},
    ConfirmationAmendedAccepted {deal_reference: WorkingOrderReference, level: f64},
    ConfirmationAmendedRejected {deal_reference: WorkingOrderReference, reason: String},
    PositionUpdateOpen {deal_reference: WorkingOrderReference, level: f64},
    PositionUpdateDelete {deal_reference: WorkingOrderReference, level: f64},
    ClosedTrade {wanted_entry: f64, actual_entry: f64, epic: String, mfe: f64, exit: f64, direction: Direction, trade_type: WorkingOrderReference, entry_time: Utc, exit_time: Utc}, // Also need to add a Decision
    Market {state: MarketState, ask: f64, bid: f64},
    Data {},
}

#[derive(Clone, Debug)]
pub enum BfgEvent {
    Trade(TradeUpdate),
    WorkingOrder(WorkingOrderUpdate),
    TradeConfirmation(TradeConfirmation),
    Market(MarketUpdate),
    Data(DataUpdate),
}

pub fn step_system(mut state: SystemState, action: BfgEvent) -> (SystemState, Vec<Decision>) {
    match (state, action) {
        // Wait until we are 1min after open
        (SystemState::Setup, BfgEvent::Market(m)) => {
            let trigger_time = NaiveTime::from_hms(8, 1, 0); // London time 1min after open
            let close_time = NaiveTime::from_hms(16, 15, 0); // London time 15 min before close
            let update_time = NaiveTime::from_str(m.update_time.unwrap().as_str()).unwrap();
            if update_time > trigger_time && update_time < close_time {
                let now = Utc::now();
                let start_time = NaiveTime::from_hms(9, 0, 0); // TODO doubble check that im getting the correct OR and not next bar
                let end_time = NaiveTime::from_hms(9, 1, 0);
                let dt_start = NaiveDateTime::new(now.naive_utc().date(), start_time);
                let dt_start_format = dt_start.format("%Y-%m-%d %H:%M:%S").to_string();
                let dt_end = NaiveDateTime::new(now.naive_utc().date(), end_time);
                let dt_end_format = dt_end.format("%Y-%m-%d %H:%M:%S").to_string();
                info!("Setup, Market -> Setup [FetchData]");
                return (
                    SystemState::Setup,
                    vec![Decision::FetchData(FetchDataDetails {
                        start: dt_start_format, //high 14203.1 low 14189.6
                        end: dt_end_format,
                    })],
                );
            }
            info!("Setup, Market -> Setup [NoOp]");
            (SystemState::Setup, vec![Decision::NoOp])
        }
        // Load data into system
        (SystemState::Setup, BfgEvent::Data(d)) => {
            let or_bar = d.prices.get(0).expect("There should always be one element");
            let or_high_ask = or_bar.high.ask;
            let or_low_ask = or_bar.low.ask;
            let or_high_bid = or_bar.high.bid;
            let or_low_bid = or_bar.low.bid;
            info!("Setup, Data -> SetupWorkingOrder [NoOp]");
            (
                SystemState::SetupWorkingOrder(SystemValues {
                    or_high_ask,
                    or_low_ask,
                    or_low_bid,
                    or_high_bid,
                    working_orders: (None, None),
                }),
                vec![Decision::NoOp],
            )
        }
        // Decide where to place working order based on OR and current price
        (
            SystemState::SetupWorkingOrder(mut system_values),
            BfgEvent::Market(MarketUpdate {
                                 bid,
                                 offer,
                                 market_state: Some(MarketState::TRADEABLE),
                                 update_time,
                                 ..
                             }),
        ) => {
            let trigger_time = NaiveTime::from_hms(8, 1, 0); // London time 1min after open
            let close_time = NaiveTime::from_hms(16, 15, 0); // London time 15 min before close
            let update_time = NaiveTime::from_str(update_time.unwrap().as_str()).unwrap();
            if update_time > trigger_time && update_time < close_time {
                let curr_price = (bid.unwrap() + offer.unwrap()) / 2.;
                let or_high = (system_values.or_high_bid + system_values.or_high_ask) / 2.;
                let or_low = (system_values.or_low_bid + system_values.or_low_ask) / 2.;
                if let Some(working_order_placement) =
                find_working_order_placement(curr_price, or_high, or_low)
                {
                    // Its possible to create a working order based on current price
                    let decisions = match working_order_placement {
                        WorkingOrderPlacement::Over => {
                            system_values.working_orders = (
                                Some(OrderState::AwaitingWorkingOrderCreateConfirmation(
                                    WorkingOrderSystemDetails::new(system_values.or_high_ask),
                                )),
                                None,
                            );
                            vec![Decision::CreateWorkingOrder(WorkingOrderDetails {
                                direction: Direction::BUY,
                                price: system_values.or_high_ask,
                                reference: WorkingOrderReference::OVER_LONG,
                            })]
                        }
                        WorkingOrderPlacement::Between => {
                            system_values.working_orders = (
                                Some(OrderState::AwaitingWorkingOrderCreateConfirmation(
                                    WorkingOrderSystemDetails::new(system_values.or_low_ask),
                                )),
                                Some(OrderState::AwaitingWorkingOrderCreateConfirmation(
                                    WorkingOrderSystemDetails::new(system_values.or_high_bid),
                                )),
                            );
                            let wo1 = Decision::CreateWorkingOrder(WorkingOrderDetails {
                                direction: Direction::BUY,
                                price: system_values.or_low_ask,
                                reference: WorkingOrderReference::BETWEEN_LONG,
                            });
                            let wo2 = Decision::CreateWorkingOrder(WorkingOrderDetails {
                                direction: Direction::SELL,
                                price: system_values.or_high_bid,
                                reference: WorkingOrderReference::BETWEEN_SHORT,
                            });
                            vec![wo1, wo2]
                        }
                        WorkingOrderPlacement::Under => {
                            system_values.working_orders = (
                                None,
                                Some(OrderState::AwaitingWorkingOrderCreateConfirmation(
                                    WorkingOrderSystemDetails::new(system_values.or_low_bid),
                                )),
                            );
                            vec![Decision::CreateWorkingOrder(WorkingOrderDetails {
                                direction: Direction::SELL,
                                price: system_values.or_low_bid,
                                reference: WorkingOrderReference::UNDER_SHORT,
                            })]
                        }
                    };
                    info!("SetupWorkingOrder, Market -> ManageOrder(AwaitingWorkingOrderCreatedConfirmation) [CreateWorkingOrder]");
                    return (SystemState::ManageOrder(system_values), decisions);
                }
                // Its not possible to setup working order so do nothing
                return (
                    SystemState::SetupWorkingOrder(system_values),
                    vec![Decision::NoOp],
                );
            } else {
                return (
                    SystemState::Setup,
                    vec![Decision::NoOp],
                );
            }
        }
        // Working order awaiting open confirmation Accepted short
        (
            SystemState::ManageOrder(mut system_values @ SystemValues {
                working_orders: (_, Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_))), ..
            }),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_SHORT | WorkingOrderReference::UNDER_SHORT,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::OPEN),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.1 = Some(OrderState::AcceptedAtOpen(deal_id));
            info!("ManageOrder(AwaitingWorkingOrderCreatedConfirmation), Confirms -> ManageOrder(AcceptedAtOpen) [NoOp]");
            return (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            );
        }
        // Working order awaiting open confirmation Rejected short
        (
            SystemState::ManageOrder(mut system_values @ SystemValues {
                working_orders: (_, Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_))), ..
            }),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_SHORT | WorkingOrderReference::UNDER_SHORT,
                                            deal_id,
                                            deal_status: DealStatus::REJECTED,
                                            status: Some(ConfirmsStatus::OPEN),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.1 = Some(OrderState::RejectedAtOpen);
            info!("ManageOrder(AwaitingWorkingOrderCreatedConfirmation), Confirms -> ManageOrder(RejectedAtOpen) [NoOp]");
            return (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            );
        }
        // Working order awaiting open confirmation Accepted long
        (
            SystemState::ManageOrder(mut system_values @ SystemValues {
                working_orders: (Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_)), _), ..
            }),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_LONG | WorkingOrderReference::OVER_LONG,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::OPEN),
                                            reason,
                                        }),
        ) => {
            info!("ManageOrder(AwaitingWorkingOrderCreatedConfirmation), Confirms -> ManageOrder(AcceptedAtOpen) [NoOp]");
            system_values.working_orders.0 = Some(OrderState::AcceptedAtOpen(deal_id));
            return (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            );
        }
        // Working order awaiting open confirmation Rejected long
        (
            SystemState::ManageOrder(mut system_values @ SystemValues {
                working_orders: (Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_)), _), ..
            }),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_LONG | WorkingOrderReference::OVER_LONG,
                                            deal_id,
                                            deal_status: DealStatus::REJECTED,
                                            status: Some(ConfirmsStatus::OPEN),
                                            reason,
                                        }),
        ) => {
            info!("ManageOrder(AwaitingWorkingOrderCreatedConfirmation), Confirms -> ManageOrder(RejectedAtOpen) [NoOp]");
            system_values.working_orders.0 = Some(OrderState::RejectedAtOpen);
            return (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            );
        }
        // WO -> Cancel long
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::AwaitCancelConfirmation(_)), _),
                    ..
                },
            ),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_LONG,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::DELETED),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.0 = None;
            info!("ManageOrder(AwaitingCancelConfirmation), Confirms -> ManageOrder(None) [NoOp]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }
        // WO -> Cancel short
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::AwaitCancelConfirmation(_))),
                    ..
                },
            ),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference: WorkingOrderReference::BETWEEN_SHORT,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::DELETED),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.1 = None;
            info!("ManageOrder(AwaitingCancelConfirmation), Confirms -> ManageOrder(None) [NoOp]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }

        // WO ->  Update with dynamic stop long
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::PositionOpen(_)), _),
                    ..
                },
            ),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference:
                                            WorkingOrderReference::OVER_LONG | WorkingOrderReference::BETWEEN_LONG,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::AMENDED),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.0 =
                Some(OrderState::PositionOpenWithTrailingStop(deal_id));
            info!("ManageOrder(PositionOpen), Confirms -> ManageOrder(PositionOpenWithTrailingStop) [NoOp]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }
        // Position -> Update with dynamic stop short
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::PositionOpen(_))),
                    ..
                },
            ),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                                            deal_reference:
                                            WorkingOrderReference::UNDER_SHORT | WorkingOrderReference::BETWEEN_SHORT,
                                            deal_id,
                                            deal_status: DealStatus::ACCEPTED,
                                            status: Some(ConfirmsStatus::AMENDED),
                                            reason,
                                        }),
        ) => {
            system_values.working_orders.1 =
                Some(OrderState::PositionOpenWithTrailingStop(deal_id));
            info!("ManageOrder(PositionOpen), Confirms -> ManageOrder(PositionOpenWithTrailingStop) [NoOp]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }

        // WO -> Positon - short - update with trailing stop and delete other
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::AcceptedAtOpen(_))),
                    or_high_bid,
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference: WorkingOrderReference::BETWEEN_SHORT,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::OPEN,
                            }),
        ) => {
            let mut decisions = vec![];
            // Was hard to get the other_deal id out of pattern match so this is the best i have atm
            // Used to make sure we close other possition and create a tailing for the opened if we have two WO open
            let mut maybe_long: Option<OrderState> = None;
            if let SystemValues { working_orders: (Some(OrderState::AcceptedAtOpen(other_deal_id)), _), .. } = system_values.borrow() {
                maybe_long = Some(OrderState::AwaitCancelConfirmation(other_deal_id.to_string()));
                decisions.push(Decision::CancelWorkingOrder(other_deal_id.to_string()));
            }
            // -------
            system_values.working_orders.0 = maybe_long;
            system_values.working_orders.1 = Some(OrderState::PositionOpen(deal_id.clone()));
            decisions.push(Decision::UpdateWithTrailingStop(deal_id, or_high_bid + 10.));
            info!("ManageOrder(WoDeleted), OPU -> ManageOrder(PositionOpen) [UpdateWithTrailingStop]");
            (
                SystemState::ManageOrder(system_values),
                decisions
            )
        }
        // WO -> Positon - short so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::AcceptedAtOpen(_))),
                    or_low_bid,
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference: WorkingOrderReference::UNDER_SHORT,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::OPEN,
                            }),
        ) => {
            system_values.working_orders.1 = Some(OrderState::PositionOpen(deal_id.clone()));
            info!("ManageOrder(WoDeleted), OPU -> ManageOrder(PositionOpen) [UpdateWithTrailingStop]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_low_bid + 10.)],
            )
        }
        // WO -> Positon - after delete new order get accepted - long so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::AcceptedAtOpen(_)), _),
                    or_low_ask,
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference: WorkingOrderReference::BETWEEN_LONG,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::OPEN,
                            }),
        ) => {
            let mut decisions = vec![];
            // Was hard to get the other_deal id out of pattern match so this is the best i have atm
            // Used to make sure we close other possition and create a tailing for the opened if we have two WO open
            let mut maybe_short: Option<OrderState> = None;
            if let SystemValues { working_orders: (_, Some(OrderState::AcceptedAtOpen(other_deal_id))), .. } = system_values.borrow() {
                maybe_short = Some(OrderState::AwaitCancelConfirmation(other_deal_id.to_string()));
                decisions.push(Decision::CancelWorkingOrder(other_deal_id.to_string()));
            }
            // ---------------
            system_values.working_orders.0 = Some(OrderState::PositionOpen(deal_id.clone()));
            system_values.working_orders.1 = maybe_short;
            decisions.push(Decision::UpdateWithTrailingStop(deal_id, or_low_ask - 10.));
            info!("ManageOrder(WoDeleted), OPU -> ManageOrder(PositionOpen) [UpdateWithTrailingStop]");
            (
                SystemState::ManageOrder(system_values),
                decisions
            )
        }
        // WO -> Positon - after delete new order get accepted - long so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::AcceptedAtOpen(_)), _),
                    or_high_ask,
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference: WorkingOrderReference::OVER_LONG,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::OPEN,
                            }),
        ) => {
            system_values.working_orders.0 = Some(OrderState::PositionOpen(deal_id.clone()));
            info!("ManageOrder(WoDeleted), OPU -> ManageOrder(PositionOpen) [UpdateWithTrailingStop]");
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_high_ask - 10.)],
            )
        }
        // WO -> Position - first WO gets deleted - long
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(_), _),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference:
                                WorkingOrderReference::OVER_LONG | WorkingOrderReference::BETWEEN_LONG,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::DELETED,
                            }),
        ) => {
            system_values.working_orders.0 = None;
            info!("ManageOrder(PositionOpenWithTrailingStop), OPU -> ManageOrder(None) [NoOp]");
            (
                SystemState::SetupWorkingOrder(system_values),
                vec![Decision::NoOp],
            )
        }
        // WO -> Position - first WO gets deleted - short
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(_)),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                                deal_reference:
                                WorkingOrderReference::UNDER_SHORT | WorkingOrderReference::BETWEEN_SHORT,
                                deal_id,
                                deal_status: DealStatus::ACCEPTED,
                                status: BfgTradeStatus::DELETED,
                            }),
        ) => {
            system_values.working_orders.1 = None;
            info!("ManageOrder(PositionOpenWithTrailingStop), OPU -> ManageOrder(None) [NoOp]");
            (
                SystemState::SetupWorkingOrder(SystemValues { ..system_values }),
                vec![Decision::NoOp],
            )
        }
        // Since working orders dont send events when closed i have to check on each market update if we are after market hours long
        (
            system_state@ SystemState::ManageOrder(SystemValues { working_orders: (Some(OrderState::AcceptedAtOpen(_)), _), .. }),
            BfgEvent::Market(MarketUpdate {
                                 bid,
                                 offer,
                                 market_state: Some(MarketState::TRADEABLE),
                                 update_time,
                                 ..
                             }),
        ) => {
            let trigger_time = NaiveTime::from_hms(8, 1, 0); // London time 1min after open
            let close_time = NaiveTime::from_hms(16, 15, 0); // London time 15 min before close
            let update_time = NaiveTime::from_str(update_time.unwrap().as_str()).unwrap();
            if update_time > trigger_time && update_time < close_time {
                return (system_state, vec![Decision::NoOp])
            } else {
                return (SystemState::Setup, vec![Decision::NoOp])
            }
        }
        // Since working orders dont send events when closed i have to check on each market update if we are after market hours short
        (
            system_state@ SystemState::ManageOrder(SystemValues { working_orders: (_, Some(OrderState::AcceptedAtOpen(_))), .. }),
            BfgEvent::Market(MarketUpdate {
                                 bid,
                                 offer,
                                 market_state: Some(MarketState::TRADEABLE),
                                 update_time,
                                 ..
                             }),
        ) => {
            let trigger_time = NaiveTime::from_hms(8, 1, 0); // London time 1min after open
            let close_time = NaiveTime::from_hms(16, 15, 0); // London time 15 min before close
            let update_time = NaiveTime::from_str(update_time.unwrap().as_str()).unwrap();
            if update_time > trigger_time && update_time < close_time {
                return (system_state, vec![Decision::NoOp])
            } else {
                return (SystemState::Setup, vec![Decision::NoOp])
            }
        }
        (s, BfgEvent::Market(_)) => (s, vec![Decision::NoOp]),
        (s, a) => {
            warn!(
                "Not implemented in trading system state {:?} action {:?}",
                s, a
            );
            (s, vec![Decision::NoOp])
        }
    }
}

fn find_working_order_placement(
    curr_price: f64,
    or_high: f64,
    or_low: f64,
) -> Option<WorkingOrderPlacement> {
    let buffer = 10.;
    if curr_price > (or_high + buffer) {
        Some(WorkingOrderPlacement::Over)
    } else if (curr_price < (or_high - buffer)) && (curr_price > (or_low + buffer)) {
        Some(WorkingOrderPlacement::Between)
    } else if curr_price < (or_low - buffer) {
        Some(WorkingOrderPlacement::Under)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{OhlcPrice, Price};

    #[test]
    fn before_open_no_action() {
        let state = setup();
        let action = market(10., 5., "04:11:11");
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::Setup => {}
            _ => panic!("Wrong system state"),
        }

        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn after_open_fetch_or() {
        let state = setup();
        let action = market(10., 5., "08:11:11");
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::Setup => {}
            _ => panic!("Wrong system state"),
        }

        match decisions[..] {
            [Decision::FetchData(_)] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn after_fetch_date_get_date_response() {
        let state = setup();
        let action = data(10., 5.);
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::SetupWorkingOrder(SystemValues {
                working_orders: (None, None),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }

        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn decide_wo_over_long() {
        let state = setup_working_order(None, None);
        let action = market(200., 150., "");
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {
                working_orders: (Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_)), None),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }

        match decisions[..] {
            [Decision::CreateWorkingOrder(WorkingOrderDetails {
                direction: Direction::BUY,
                reference: WorkingOrderReference::OVER_LONG,
                ..
            })] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn decide_wo_between() {
        let state = setup_working_order(None, None);
        let action = market(70., 60., "");
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {
                working_orders:
                    (
                        Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_)),
                        Some(OrderState::AwaitingWorkingOrderCreateConfirmation(_)),
                    ),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }

        match decisions[..] {
            [Decision::CreateWorkingOrder(WorkingOrderDetails {
                direction: Direction::BUY,
                reference: WorkingOrderReference::BETWEEN_LONG,
                ..
            }), Decision::CreateWorkingOrder(WorkingOrderDetails {
                direction: Direction::SELL,
                reference: WorkingOrderReference::BETWEEN_SHORT,
                ..
            })] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn decide_wo_under_short() {
        let state = setup_working_order(None, None);
        let action = market(10., 5., "");
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {
                working_orders: (None, Some(_)),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }
        match decisions[..] {
            [Decision::CreateWorkingOrder(WorkingOrderDetails {
                direction: Direction::SELL,
                reference: WorkingOrderReference::UNDER_SHORT,
                ..
            })] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn manage_order_wo_open_confirmation() {
        let state = manage_order(
            None,
            Some(OrderState::AwaitingWorkingOrderCreateConfirmation(
                WorkingOrderSystemDetails {
                    deal_id: Some("DEAL_ID".to_string()),
                    requested_entry_level: 0.0,
                    actual_entry_level: None,
                    requested_exit_level: None,
                    actual_exit_level: None,
                },
            )),
        );
        let action = confirms(
            ConfirmsStatus::OPEN,
            DealStatus::ACCEPTED,
            WorkingOrderReference::UNDER_SHORT,
        );
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {
                working_orders: (None, Some(OrderState::AcceptedAtOpen(_))),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }
        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn manage_order_trailing_stop_confirmation() {
        let state = manage_order(None, Some(OrderState::PositionOpen("BLA".to_string())));
        let action = confirms(
            ConfirmsStatus::AMENDED,
            DealStatus::ACCEPTED,
            WorkingOrderReference::UNDER_SHORT,
        );
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {
                working_orders: (None, Some(OrderState::PositionOpenWithTrailingStop(_))),
                ..
            }) => {}
            _ => panic!("Wrong system state"),
        }
        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn manage_order_stop_is_hit_or_order_deleted_manually() {
        let state = manage_order(None, Some(OrderState::PositionOpenWithTrailingStop("BLA".to_string())));
        let action = opu(
            BfgTradeStatus::DELETED,
            DealStatus::ACCEPTED,
            WorkingOrderReference::UNDER_SHORT,
        );
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::SetupWorkingOrder(SystemValues {working_orders: (None, None), ..}) => {}
            _ => panic!("Wrong system state"),
        }
        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    #[test]
    fn manage_order_over_long_awaiting_open_confirm() {
        let state = manage_order(Some(OrderState::AwaitingWorkingOrderCreateConfirmation(WorkingOrderSystemDetails {
            deal_id: None,
            requested_entry_level: 0.0,
            actual_entry_level: None,
            requested_exit_level: None,
            actual_exit_level: None
        })), None);
        let action = confirms(
            ConfirmsStatus::OPEN,
            DealStatus::ACCEPTED,
            WorkingOrderReference::OVER_LONG,
        );
        let (result_state, decisions) = step_system(state, action);
        match result_state {
            SystemState::ManageOrder(SystemValues {working_orders: (Some(OrderState::AcceptedAtOpen(_)), None), ..}) => {}
            _ => panic!("Wrong system state"),
        }
        match decisions[..] {
            [Decision::NoOp] => {}
            _ => panic!("Wrong decision"),
        }
    }

    fn data(high: f64, low: f64) -> BfgEvent {
        BfgEvent::Data(DataUpdate {
            prices: vec![OhlcPrice {
                open: Price { bid: 0., ask: 0. },
                high: Price {
                    bid: high,
                    ask: high,
                },
                low: Price { bid: low, ask: low },
                close: Price { bid: 0., ask: 0. },
            }],
        })
    }

    fn market(ask: f64, bid: f64, time: &str) -> BfgEvent {
        BfgEvent::Market(MarketUpdate {
            offer: Some(ask),
            bid: Some(bid),
            market_delay: Some(0),
            market_state: Some(MarketState::TRADEABLE),
            update_time: Some(time.to_string()),
        })
    }

    fn opu(
        status: BfgTradeStatus,
        deal_status: DealStatus,
        deal_reference: WorkingOrderReference,
    ) -> BfgEvent {
        BfgEvent::Trade(TradeUpdate {
            deal_status,
            status,
            deal_reference,
            deal_id: "".to_string(),
        })
    }

    fn confirms(
        status: ConfirmsStatus,
        deal_status: DealStatus,
        deal_reference: WorkingOrderReference,
    ) -> BfgEvent {
        BfgEvent::TradeConfirmation(TradeConfirmation {
            deal_status,
            status: Some(status),
            deal_reference,
            deal_id: "".to_string(),
            reason: "".to_string(),
        })
    }

    fn setup() -> SystemState {
        SystemState::Setup
    }

    fn setup_working_order(long: Option<OrderState>, short: Option<OrderState>) -> SystemState {
        SystemState::SetupWorkingOrder(SystemValues {
            or_high_ask: 100.0,
            or_high_bid: 80.0,
            or_low_ask: 40.0,
            or_low_bid: 20.0,
            working_orders: (long, short),
        })
    }

    fn manage_order(long: Option<OrderState>, short: Option<OrderState>) -> SystemState {
        SystemState::ManageOrder(SystemValues {
            or_high_ask: 0.0,
            or_low_ask: 0.0,
            or_high_bid: 0.0,
            or_low_bid: 0.0,
            working_orders: (long, short),
        })
    }
}