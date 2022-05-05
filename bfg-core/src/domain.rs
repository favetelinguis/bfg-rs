use log::error;
use crate::domain::State::Setup;
use crate::ports::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemValues {
    pub market: usize,
    pub account: usize,
    pub system: usize,
    pub trade: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum State {
    Setup(SystemValues), // Await LTP to go over or_high or below or_low
    Entry(SystemValues), // LTP touches or_high or or_low
    AwaitingEntryConfirmation(SystemValues),
    Exit(SystemValues), // After 10 seconds or if LTP is over or_low och below or_high
    AwaitingExitConfirmation(SystemValues),
}

impl State {
    pub fn new(systemValues: SystemValues) -> Self {
        Setup(systemValues)
    }
}


#[derive(Debug, Eq, PartialEq)]
pub enum Decision {
    NoOp, Buy(OrderDetails), Sell(OrderDetails), SetupOr
}

pub fn do_action(state: State, action: Action) -> (State, Decision) {
    match (state, action) {
        (s@State::Setup(_), Action::Start) => (s, Decision::SetupOr),
        (State::Setup(system_values), Action::OrSetup(or)) => (State::Entry(system_values), Decision::NoOp),
        (State::Setup(v), Action::AccountEvent(account_update)) => (State::Setup(SystemValues {account: account_update.money, ..v}), Decision::NoOp),
        (State::Setup(v), Action::TradeEvent(trade_update)) => (State::Setup(SystemValues {trade: trade_update.entry, ..v}), Decision::NoOp),
        (State::Setup(v), Action::MarketEvent(market_update)) => (State::Setup(SystemValues {market: market_update.high, ..v}), Decision::NoOp),
        (s@State::Setup(_), Action::Quit) => (s, Decision::NoOp),

        (State::Entry(system_values), Action::MarketEvent(market_update)) => (State::AwaitingEntryConfirmation(system_values), Decision::Buy(OrderDetails::new(Direction::Long, 44))),
        (s@State::Entry(_), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::Entry(_), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::Entry(_), Action::TradeEvent(trade_update)) => (s, Decision::NoOp),
        (s@State::Entry(_), Action::Start) => (s, Decision::NoOp),
        (s@State::Entry(_), Action::Quit) => (s, Decision::NoOp),

        (State::Exit(system_values), Action::MarketEvent(market_update)) => (State::AwaitingExitConfirmation(system_values), Decision::Buy(OrderDetails::new(Direction::Short, 44))),
        (s@State::Exit(_), Action::TradeEvent(trade_update)) => (s, Decision::NoOp),
        (s@State::Exit(_), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::Exit(_), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::Exit(_), Action::Start) => (s, Decision::NoOp),
        (s@State::Exit(_), Action::Quit) => (s, Decision::NoOp),

        (State::AwaitingEntryConfirmation(system_values), Action::TradeEvent(account_update)) => (State::Exit(system_values), Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(_), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(_), Action::MarketEvent(market_update)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(_), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(_), Action::Start) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(_), Action::Quit) => (s, Decision::NoOp),

        (s@State::AwaitingExitConfirmation(_), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::AwaitingExitConfirmation(_), Action::MarketEvent(market_update)) => (s, Decision::NoOp),
        (s@State::AwaitingExitConfirmation(_), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::TradeEvent(account_update)) => (State::Setup(system_values), Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::Start) => (State::Setup(system_values), Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::Quit) => (State::Setup(system_values), Decision::NoOp),
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn create_session_in_init_state_return_session() {
//         let (state, decision) = do_action(
//             State::Setup(SystemValues::new(3, 4)),
//             Action::Start,
//         );
//         assert_eq!(state, State::Setup(SystemValues::new(3, 4)));
//         assert_eq!(decision, Decision::SetupOr);
//     }
// }
