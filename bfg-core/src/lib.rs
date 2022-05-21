extern crate core;

use crate::models::{BfgTradeStatus, ConfirmsStatus, DataUpdate, DealStatus, Decision, Direction, EntryMode, FetchDataDetails, LimitOrderDetails, MarketOrderDetails, MarketState, MarketUpdate, OrderState, PriceRelativeOr, SystemState, SystemValues, TradeConfirmation, TradeUpdate, WorkingOrderDetails, WorkingOrderPlacement, WorkingOrderReference, WorkingOrderSystemDetails, WorkingOrderUpdate};
use chrono::{NaiveDateTime, NaiveTime, Utc};
use log::{error, warn};
use std::borrow::{Borrow, BorrowMut};
use std::collections::HashMap;
use std::str::FromStr;

pub mod models;

#[derive(Clone, Debug)]
pub enum BfgEvent {
    Trade(TradeUpdate),
    WorkingOrder(WorkingOrderUpdate),
    TradeConfirmation(TradeConfirmation),
    Market(MarketUpdate),
    Data(DataUpdate),
}

pub fn step_system(state: SystemState, action: BfgEvent) -> (SystemState, Vec<Decision>) {
    match (state, action) {
        // Wait until we are 1min after open
        (SystemState::Setup, BfgEvent::Market(m)) => {
            let trigger_time = NaiveTime::from_hms(8, 1, 0); // London time
            let update_time = NaiveTime::from_str(m.update_time.unwrap().as_str()).unwrap();
            if update_time > trigger_time {
                let now = Utc::now();
                let start_time = NaiveTime::from_hms(9, 0, 0); // TODO doubble check that im getting the correct OR and not next bar
                let end_time = NaiveTime::from_hms(9, 1, 0);
                let dt_start = NaiveDateTime::new(now.naive_utc().date(), start_time);
                let dt_start_format = dt_start.format("%Y-%m-%d %H:%M:%S").to_string();
                let dt_end = NaiveDateTime::new(now.naive_utc().date(), end_time);
                let dt_end_format = dt_end.format("%Y-%m-%d %H:%M:%S").to_string();
                return (
                    SystemState::Setup,
                    vec![Decision::FetchData(FetchDataDetails {
                        start: dt_start_format, //high 14203.1 low 14189.6
                        end: dt_end_format,
                    })],
                );
            }
            (SystemState::Setup, vec![Decision::NoOp])
        }
        // Load data into system
        (SystemState::Setup, BfgEvent::Data(d)) => {
            let or_bar = d.prices.get(0).expect("There should always be one element");
            let or_high_ask = or_bar.high.ask;
            let or_low_ask = or_bar.low.ask;
            let or_high_bid = or_bar.high.bid;
            let or_low_bid = or_bar.low.bid;
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
        (SystemState::SetupWorkingOrder(mut system_values), BfgEvent::Market(MarketUpdate {bid, offer, market_state: Some(MarketState::TRADEABLE), ..})) => {
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
                return (SystemState::ManageOrder(system_values), decisions);
            }
            // Its not possible to setup working order so do nothing
            return (
                SystemState::SetupWorkingOrder(system_values),
                vec![Decision::NoOp],
            );
        }
        (
            SystemState::ManageOrder(mut system_values),
            BfgEvent::TradeConfirmation(TradeConfirmation {
                deal_reference,
                deal_id,
                deal_status,
                status,
                reason,
            }),
        ) => match deal_reference {
            WorkingOrderReference::BETWEEN_SHORT | WorkingOrderReference::UNDER_SHORT => {
                if deal_status == DealStatus::ACCEPTED && status == Some(ConfirmsStatus::OPEN) {
                    system_values.working_orders.1 = Some(OrderState::AcceptedAtOpen(deal_id));
                } else {
                    system_values.working_orders.1 =
                        Some(OrderState::RejectedAtOpen(deal_reference, deal_id, reason));
                }
                return (
                    SystemState::ManageOrder(system_values),
                    vec![Decision::NoOp],
                );
            }
            WorkingOrderReference::BETWEEN_LONG | WorkingOrderReference::OVER_LONG => {
                if deal_status == DealStatus::ACCEPTED && status == Some(ConfirmsStatus::OPEN) {
                    system_values.working_orders.0 = Some(OrderState::AcceptedAtOpen(deal_id));
                } else {
                    system_values.working_orders.0 =
                        Some(OrderState::RejectedAtOpen(deal_reference, deal_id, reason));
                }
                return (
                    SystemState::ManageOrder(system_values),
                    vec![Decision::NoOp],
                );
            }
        },

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
                status: Some(ConfirmsStatus::DELETED),
                reason,
            }),
        ) => {
            system_values.working_orders.1 =
                Some(OrderState::PositionOpenWithTrailingStop(deal_id));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }

        // WO -> Position - first WO gets deleted - long
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::AcceptedAtOpen(_)), None),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                deal_reference: WorkingOrderReference::OVER_LONG,
                deal_id,
                deal_status: DealStatus::ACCEPTED,
                status: BfgTradeStatus::DELETED,
            }),
        ) => {
            system_values.working_orders.0 = Some(OrderState::WODeleted(deal_id));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }
        // WO -> Position - first WO gets deleted - short
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::AcceptedAtOpen(_))),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                deal_reference: WorkingOrderReference::UNDER_SHORT,
                deal_id,
                deal_status: DealStatus::ACCEPTED,
                status: BfgTradeStatus::DELETED,
            }),
        ) => {
            system_values.working_orders.1 = Some(OrderState::WODeleted(deal_id));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::NoOp],
            )
        }
        // WO -> Position - first WO gets deleted - long between so cancel other
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::AcceptedAtOpen(_)), None),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                deal_reference: WorkingOrderReference::BETWEEN_LONG,
                deal_id,
                deal_status: DealStatus::ACCEPTED,
                status: BfgTradeStatus::DELETED,
            }),
        ) => {
            system_values.working_orders.0 = Some(OrderState::WODeleted(deal_id.clone()));
            system_values.working_orders.1 =
                Some(OrderState::AwaitCancelConfirmation(deal_id.clone()));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::CancelWorkingOrder(deal_id)],
            )
        }
        // WO -> Position - first WO gets deleted - short between so cancel other
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::AcceptedAtOpen(_))),
                    ..
                },
            ),
            BfgEvent::Trade(TradeUpdate {
                deal_reference: WorkingOrderReference::BETWEEN_SHORT,
                deal_id,
                deal_status: DealStatus::ACCEPTED,
                status: BfgTradeStatus::DELETED,
            }),
        ) => {
            system_values.working_orders.1 = Some(OrderState::WODeleted(deal_id.clone()));
            system_values.working_orders.0 =
                Some(OrderState::AwaitCancelConfirmation(deal_id.clone()));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::CancelWorkingOrder(deal_id)],
            )
        }
        // WO -> Positon - after delete new order get accepted - short so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::WODeleted(_))),
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
            system_values.working_orders.1 = Some(OrderState::PositionOpen(deal_id.clone()));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_high_bid + 10.)],
            )
        }
        // WO -> Positon - after delete new order get accepted - short so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::WODeleted(_))),
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
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_low_bid + 10.)],
            )
        }
        // WO -> Positon - after delete new order get accepted - long so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::WODeleted(_)), _),
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
            system_values.working_orders.0 = Some(OrderState::PositionOpen(deal_id.clone()));
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_low_ask - 10.)],
            )
        }
        // WO -> Positon - after delete new order get accepted - long so update with trailing stop
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::WODeleted(_)), _),
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
            (
                SystemState::ManageOrder(system_values),
                vec![Decision::UpdateWithTrailingStop(deal_id, or_high_ask - 10.)],
            )
        }
        // WO -> Position - first WO gets deleted - long
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (Some(OrderState::PositionOpenWithTrailingStop(_)), None),
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
            (
                SystemState::SetupWorkingOrder(SystemValues { ..system_values }),
                vec![Decision::NoOp],
            )
        }
        // WO -> Position - first WO gets deleted - short
        (
            SystemState::ManageOrder(
                mut system_values @ SystemValues {
                    working_orders: (_, Some(OrderState::PositionOpenWithTrailingStop(_))),
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
            (
                SystemState::SetupWorkingOrder(SystemValues { ..system_values }),
                vec![Decision::NoOp],
            )
        }
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
    let buffer = 5.;
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
    #[test]
    fn over() {
        let system_values = SystemValues {
            or_high_ask: 10.,
            or_low_ask: 5.,
            or_high_bid: 9.,
            or_low_bid: 4.,
            working_orders: (None, None), // long, short
        };
        let trade_update = TradeUpdate {
            deal_reference: WorkingOrderReference::UNDER_SHORT,
            deal_id: "adfad".to_string(),
            deal_status: DealStatus::ACCEPTED,
            status: BfgTradeStatus::OPEN,
        };
        let state = SystemState::ManageOrder(system_values);
        let action = BfgEvent::Trade(trade_update);
        let (result_state, decisions) = step_system(state, action);
    }
}
