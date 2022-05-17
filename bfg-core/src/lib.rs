use log::{error, warn};
use crate::models::{BfgTradeConfirmationStatus, BfgTradeStatus, Decision, Direction, MarketUpdate, OrderDetails, SystemState, SystemValues, TradeConfirmation, TradeUpdate};

pub mod models;

pub enum BfgEvent {
    Trade(TradeUpdate),
    TradeConfirmation(TradeConfirmation),
    Market(MarketUpdate),
}

pub fn step_system(state: SystemState, action: BfgEvent) -> (SystemState, Decision) {
    match (state, action) {
        (SystemState::Setup, BfgEvent::Market(_)) => (
            SystemState::Entry(SystemValues { count: 0 }),
            Decision::NoOp,
        ),
        (SystemState::Entry(system_values), BfgEvent::Market(m)) => {
            if system_values.count > 10 {
                let price = m.offer.expect("Offer should never be None");
                (
                    SystemState::AwaitingEntryConfirmation(system_values),
                    Decision::Buy(OrderDetails {
                        size: 1,
                        direction: Direction::BUY,
                        price,
                    }),
                )
            } else {
                let new_count = system_values.count + 1;
                (
                    SystemState::Entry(SystemValues { count: new_count }),
                    Decision::NoOp,
                )
            }
        }
        (
            SystemState::AwaitingEntryConfirmation(system_values),
            BfgEvent::TradeConfirmation(TradeConfirmation { status }),
        ) => {
            if status == BfgTradeConfirmationStatus::ACCEPTED {
                (SystemState::Exit(system_values), Decision::NoOp)
            } else if status == BfgTradeConfirmationStatus::REJECTED {
                warn!("Failed to place order due to rejection");
                (SystemState::Setup, Decision::NoOp)
            } else {
                (
                    SystemState::AwaitingEntryConfirmation(system_values),
                    Decision::NoOp,
                )
            }
        }
        (SystemState::Exit(system_values), BfgEvent::Market(_)) => {
            if system_values.count > 100 {
                (
                    SystemState::AwaitingExitConfirmation(system_values),
                    Decision::Sell(OrderDetails {
                        size: 1,
                        direction: Direction::SELL,
                        price: 0.,
                    }),
                )
            } else {
                let new_count = system_values.count + 1;
                (
                    SystemState::Exit(SystemValues { count: new_count }),
                    Decision::NoOp,
                )
            }
        }
        (
            SystemState::AwaitingExitConfirmation(system_values),
            BfgEvent::TradeConfirmation(TradeConfirmation { status }),
        ) => {
            if status == BfgTradeConfirmationStatus::ACCEPTED {
                (SystemState::Setup, Decision::NoOp)
            } else if status == BfgTradeConfirmationStatus::REJECTED {
                error!("Failed close order due to rejection, RETRYING");
                (
                    SystemState::AwaitingExitConfirmation(system_values),
                    Decision::Sell(OrderDetails {
                        size: 1,
                        direction: Direction::SELL,
                        price: 0.,
                    }),
                )
            } else {
                (
                    SystemState::AwaitingExitConfirmation(system_values),
                    Decision::NoOp,
                )
            }
        }
        // If i do a manual close or exit limit target hit
        (SystemState::Exit(system_values), BfgEvent::Trade(TradeUpdate { status })) => {
            if status == BfgTradeStatus::DELETED{
                (SystemState::Setup, Decision::NoOp)
            } else {
                (
                    SystemState::Exit(system_values),
                    Decision::NoOp,
                )
            }
        }
        (st, _) => (st, Decision::NoOp),
    }
}
