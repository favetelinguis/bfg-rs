use crate::domain::State::Setup;
use crate::ports::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SystemValues {
    market: MarketValues,
    or_high: usize,
    or_low: usize,
}

impl SystemValues {
    pub fn new(or_high: usize, or_low: usize) -> SystemValues {
        SystemValues {
            or_high,
            or_low,
            market: MarketValues::new()
        }
    }
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
        (s@State::Setup(system_values), Action::Start) => (s, Decision::SetupOr),
        (State::Setup(system_values), Action::OrSetup(or)) => (State::Entry(system_values), Decision::NoOp),
        (s@State::Setup(system_values), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::Setup(system_values), Action::TradeEvent(trade_update)) => (s, Decision::NoOp),
        (s@State::Setup(system_values), Action::MarketEvent(market_update)) => (s, Decision::NoOp),
        (s@State::Setup(system_values), Action::Quit) => (s, Decision::NoOp),

        (State::Entry(system_values), Action::MarketEvent(market_update)) => (State::AwaitingEntryConfirmation(system_values), Decision::Buy(OrderDetails::new(Direction::Long, 44))),
        (s@State::Entry(system_values), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::Entry(system_values), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::Entry(system_values), Action::TradeEvent(trade_update)) => (s, Decision::NoOp),
        (s@State::Entry(system_values), Action::Start) => (s, Decision::NoOp),
        (s@State::Entry(system_values), Action::Quit) => (s, Decision::NoOp),

        (State::Exit(system_values), Action::MarketEvent(market_update)) => (State::AwaitingExitConfirmation(system_values), Decision::Buy(OrderDetails::new(Direction::Short, 44))),
        (s@State::Exit(system_values), Action::TradeEvent(trade_update)) => (s, Decision::NoOp),
        (s@State::Exit(system_values), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::Exit(system_values), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::Exit(system_values), Action::Start) => (s, Decision::NoOp),
        (s@State::Exit(system_values), Action::Quit) => (s, Decision::NoOp),

        (State::AwaitingEntryConfirmation(system_values), Action::TradeEvent(account_update)) => (State::Exit(system_values), Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(system_values), a@Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(system_values), Action::MarketEvent(market_update)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(system_values), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(system_values), Action::Start) => (s, Decision::NoOp),
        (s@State::AwaitingEntryConfirmation(system_values), Action::Quit) => (s, Decision::NoOp),

        (s@State::AwaitingExitConfirmation(system_values), Action::OrSetup(maybe_or)) => (s, Decision::NoOp),
        (s@State::AwaitingExitConfirmation(system_values), Action::MarketEvent(market_update)) => (s, Decision::NoOp),
        (s@State::AwaitingExitConfirmation(system_values), Action::AccountEvent(account_update)) => (s, Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::TradeEvent(account_update)) => (State::Setup(system_values), Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::Start) => (State::Setup(system_values), Decision::NoOp),
        (State::AwaitingExitConfirmation(system_values), Action::Quit) => (State::Setup(system_values), Decision::NoOp),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_session_in_init_state_return_session() {
        let (state, decision) = do_action(
            State::Setup(SystemValues::new(3, 4)),
            Action::Start,
        );
        assert_eq!(state, State::Setup(SystemValues::new(3, 4)));
        assert_eq!(decision, Decision::SetupOr);
    }
}
