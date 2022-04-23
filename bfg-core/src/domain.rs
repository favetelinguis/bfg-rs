use super::ports::*;

struct ConnectionDetails {
    username: String,
    password: String,
}

pub enum Action {
    CreateSession(ConnectionDetails),
    DestroySession,
    MarketEvent(MarketUpdate),
    AccountEvent(AccountUpdate),
    TradeEvent(TradeUpdate),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MarketValues {
    id: usize,
    open_time: usize,
    close_time: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct SystemValues {
    market: MarketValues,
    or_high: usize,
    or_low: usize,
}

impl SystemValues {
    fn new(or_high: usize, or_low: usize) -> SystemValues {
        SystemValues {
            or_high,
            or_low,
            market: MarketValues {
                id: 33,
                open_time: 5,
                close_time: 8,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SystemState {
    Init,                // Await OR to be created
    Setup(SystemValues), // Await LTP to go over or_high or below or_low
    Entry(SystemValues), // LTP touches or_high or or_low
    AwaitingEntryConfirmation(SystemValues),
    Exit(SystemValues), // After 10 seconds or if LTP is over or_low och below or_high
    AwaitingExitConfirmation(SystemValues),
}

// TODO this will be handled by adapter Remove here and only make this about system state
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum State {
    Init,
    SessionCreated(SystemState),
    NoSession,
}

pub fn do_action(state: State, action: Action) -> State {
    match (state, action) {
        (State::Init, Action::CreateSession(connection_details)) => {
            setup_session(connection_details)
        }
        (nop @ State::Init, _) => nop,

        (State::SessionCreated(system_state), Action::MarketEvent(market_update)) => {
            State::SessionCreated(handle_market_event(system_state, market_update))
        }
        (State::SessionCreated(system_state), Action::AccountEvent(account_update)) => {
            State::SessionCreated(handle_account_event(system_state, account_update))
        }
        (State::SessionCreated(system_state), Action::TradeEvent(trade_update)) => {
            State::SessionCreated(handle_trade_event(system_state, trade_update))
        }
        (State::SessionCreated(_), Action::DestroySession) => destroy_session(),
        (nop @ State::SessionCreated(_), _) => nop,

        (State::NoSession, Action::CreateSession(connection_details)) => {
            setup_session(connection_details)
        }
        (nop @ State::NoSession, _) => nop,
    }
}

fn destroy_session() -> State {
    //DO destroy action
    State::NoSession
}

fn handle_trade_event(system_state: SystemState, update: TradeUpdate) -> SystemState {
    match system_state {
        SystemState::AwaitingEntryConfirmation(state) => {
            SystemState::Exit(SystemValues { ..state })
        }
        SystemState::AwaitingExitConfirmation(state) => {
            SystemState::Setup(SystemValues { ..state })
        }
        _ => todo!(), // Log error this should not happen
    }
}

fn handle_account_event(system_state: SystemState, update: AccountUpdate) -> SystemState {
    match system_state {
        _ => todo!(), // Will do later
    }
}

fn handle_market_event(system_state: SystemState, update: MarketUpdate) -> SystemState {
    match system_state {
        SystemState::Init => build_or(update),
        SystemState::Setup(state) => await_setup(state, update),
        SystemState::Entry(state) => await_entry(state, update),
        nop @ SystemState::AwaitingEntryConfirmation(_) => nop,
        SystemState::Exit(state) => await_exit(state, update),
        nop @ SystemState::AwaitingExitConfirmation(_) => nop,
    }
}

fn await_exit(state: SystemValues, update: MarketUpdate) -> SystemState {
    // After 10 seconds or if LTP is over or_low och below or_high
    SystemState::AwaitingExitConfirmation(SystemValues { ..state })
}

fn await_entry(state: SystemValues, update: MarketUpdate) -> SystemState {
    // LTP touches or_high or or_low
    SystemState::AwaitingEntryConfirmation(SystemValues { ..state })
}

fn await_setup(state: SystemValues, update: MarketUpdate) -> SystemState {
    // Await LTP to go over or_high or below or_low
    SystemState::Entry(SystemValues { ..state })
}

fn build_or(update: MarketUpdate) -> SystemState {
    // Cases for creating
    // Time before open -> do nothing
    // Time after open but less then 1min after -> do nothing
    // Time after open and more then 1min -> get highest high and lowest low
    // Time after open but close is in 120min -> do nothing
    SystemState::Setup(SystemValues::new(10, 5))
}

fn setup_session(connection_details: ConnectionDetails) -> State {
    if let Ok(status) = get_session() {
        State::SessionCreated(SystemState::Init)
    } else {
        State::NoSession
    }
}

fn get_session() -> Result<String, &'static str> {
    Ok(String::from("aaa")) // Should take a client and I want to control results in tests
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_session_in_init_state_return_session() {
        let result = do_action(
            State::Init,
            Action::CreateSession(ConnectionDetails {
                username: String::from(""),
                password: String::from(""),
            }),
        );
        assert_eq!(result, State::SessionCreated(SystemState::Init))
    }

    #[test]
    fn initial_connection() {
        let mut result = State::Init;
        for step in vec![
            Action::CreateSession(ConnectionDetails {
                username: String::from(""),
                password: String::from(""),
            }),
            Action::MarketEvent(MarketUpdate { high: 2 }),
            Action::MarketEvent(MarketUpdate { high: 2 }),
            Action::MarketEvent(MarketUpdate { high: 2 }),
            Action::TradeEvent(TradeUpdate {
                entry: String::from(""),
            }),
            Action::MarketEvent(MarketUpdate { high: 2 }),
            Action::TradeEvent(TradeUpdate {
                entry: String::from(""),
            }),
        ] {
            result = do_action(result, step);
        }
        assert_eq!(result, State::SessionCreated(SystemState::Init))
    }
}
