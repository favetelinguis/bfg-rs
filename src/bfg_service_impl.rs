use super::ports::*;
use crate::domain::{do_action, Action, State};

pub struct BfgServiceImpl<A: BrokerageApi> {
    pub brokerage: A,
    pub state: State,
}

impl<A> BfgService for BfgServiceImpl<A>
where
    A: BrokerageApi + Sync + Send,
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
        let mut brokerage_api = MockBrokerageApi::new();
        brokerage_api
            .expect_publish_markte_update_event()
            .returning(|state, action| State::Init);

        let mut sut = BfgServiceImpl {
            brokerage: brokerage_api,
            state: State::Init,
        };
        let actual = sut.publish_market_update_event(MarketUpdate { high: 4 });
        let expected = State::Init;
        assert_eq!(expected, sut.state);
    }
}
