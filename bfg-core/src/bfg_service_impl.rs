use super::ports::*;
use crate::domain::{do_action, Action, State};

pub struct BfgServiceImpl<A: BrokerageApi> {
    pub brokerage: A,
    pub state: State, // Should this be Arc<RwLock<State>>
}

impl<A> BfgService for BfgServiceImpl<A>
where
    A: BrokerageApi,
{
    fn publish_market_update_event(&mut self, update: MarketUpdate) {
        self.state = do_action(self.state, Action::MarketEvent(update))
        // TODO this is the place where I have access to the brokerage api, all interaction with brokerage is done here
    }

    fn publish_account_update_event(&mut self, update: AccountUpdate) {
        todo!()
    }

    fn publish_trade_update_event(&mut self, update: TradeUpdate) {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_bfg_with_broker_mock() {
        let mut mock = MockBrokerageApi::new();
        mock.expect_publish_markte_update_event()
            .with(eq(4))
            .times(1)
            .returning(|state, action| State::Init);

        let mut sut = BfgServiceImpl {
            brokerage: mock,
            state: State::Init,
        };
        let actual = sut.publish_market_update_event(MarketUpdate { high: 4 });
        let expected = State::Init;
        assert_eq!(expected, sut.state);
    }
}
