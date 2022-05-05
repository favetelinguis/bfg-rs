use super::ports::*;
use crate::domain::*;

pub struct BfgServiceImpl<A: BrokerageApi> {
    pub brokerage: A,
    pub state: State,
}

impl<A> BfgService for BfgServiceImpl<A>
    where
        A: BrokerageApi,
{
    fn market_details(&self) -> MarketValues {
        match self.state {
            _ => MarketValues::new()
        }
    }
    fn setup_market(&mut self, market: MarketValues) {
        todo!()
    }

    fn publish_update_event(&mut self, update: Action) {
        let mut actions = vec![update]; // actions resulting from external actions
        for action in actions.clone() {
            let (next_state, decision) = do_action(self.state.clone(), action);
            // Execute decisions against brokerage api
            // match decision {
            //     Decision::NoOp => (),
            //     Decision::Buy(orderDetails) => self.brokerage.place_order(orderDetails),
            //     Decision::Sell(orderDetails) => self.brokerage.place_order(orderDetails),
            //     Decision::SetupOr => actions.push(Action::OrSetup(self.brokerage.get_or())),
            // }
            self.state = next_state;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_bfg_with_broker_mock() {
        let mut mock = MockBrokerageApi::new();
        mock.expect_get_or()
            .with()
            .times(1)
            .returning(|| Option::Some(Or::new(3,2)));

        let mut sut = BfgServiceImpl {
            brokerage: mock,
            state: State::Setup(SystemValues::new(33, 4)),
        };
        sut.publish_update_event(Action::Start);
        let expected = State::Setup(SystemValues::new(3, 2));
        assert_eq!(expected, sut.state);
    }
}
