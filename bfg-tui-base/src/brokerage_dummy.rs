use bfg_core::errors::BrokerageError;
use bfg_core::ports::{AccountUpdate, Action, BrokerageApi, MarketUpdate, Or, OrderDetails, TradeUpdate};
use std::sync::mpsc::{SyncSender};
use std::thread;
use std::time::Duration;
use rand::distributions::Uniform;
use rand::prelude::*;

pub struct DummyBrokerageApi {
    // Need to be kept around to prevent disposing the sender side
    _tx: SyncSender<Action>,
}

impl BrokerageApi for DummyBrokerageApi {
    fn get_or(&mut self) -> Option<Or> {
        todo!()
    }

    fn place_order(&mut self, order: OrderDetails) {
        todo!()
    }

    fn get_market_details(&mut self) -> Result<(), BrokerageError> {
        todo!()
    }
}

impl DummyBrokerageApi {
    pub fn new(tx: SyncSender<Action>) -> Self {
        let event_tx = tx.clone(); // the thread::spawn own event_tx

        thread::spawn(move || loop {
            let mut rng = thread_rng();
            let between = Uniform::from(0..3);
            loop {
                thread::sleep(Duration::from_secs(1));
                let index = between.sample(&mut rng);
                match index {
                    0 => event_tx.send(Action::MarketEvent(MarketUpdate {high: random()})).unwrap(),
                    1 => event_tx.send(Action::AccountEvent(AccountUpdate {money: random() })).unwrap(),
                    2 => event_tx.send(Action::TradeEvent(TradeUpdate {entry: random()})).unwrap(),
                    _ => unreachable!(),
                }
            }
        });
        Self { _tx: tx }
    }
}


