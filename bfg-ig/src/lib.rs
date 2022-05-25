use tokio::sync::mpsc::Sender;
use bfg_core::BfgEvent;
use bfg_core::models::SystemState;
use ig_brokerage_adapter::{BrokerageEvent, ConnectionDetails, IgBrokerageApi, RealtimeEvent};
use ig_brokerage_adapter::RealtimeEvent::TradeConfirmation;

pub enum IgEvent {}

pub struct BfgIg {
}

impl BfgIg {
    pub fn new(connection_details: ConnectionDetails, ig_tx: Sender<IgEvent>) -> Self {
        let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<BrokerageEvent>(100);
        // tokio::spawn(async move {
        //     let brokerage = IgBrokerageApi::new(connection_details, brokerage_tx);
        //     let mut bfg_system = SystemState::Setup;
        //     while let Some(brokerage_event) = ig_rx.recv().await {
        //         let maybe_event: Option<BfgEvent >= Some(brokerage_event.into());
        //         while let Some(event) = maybe_event {
        //             bfg_system.step_system(event); // TODO while there are unconsumed decissions a step_system does nothing
        //             while
        //         }
        //         ig_tx.send(bfg_system.current_state().into()).await.expect("failed sending current system state event to tui");
        //         ig_tx.send(brokerage_event.into()).await.expect("failed sending brokerage event to tui");
        //     }
        // });
        return BfgIg {};
    }
}

// SYSTEM

#[cfg(test)]
mod tests {
    use crate::{WorkingOrderMachine, Factory, Filling, transition_the_states, Waiting};

    #[test]
    fn it_works_factory() {
        let mut the_factory = Factory::new();
        the_factory.bottle_filling_machine = the_factory.bottle_filling_machine.step("");
    }

    #[test]
    fn it_works() {
        let in_waiting = WorkingOrderMachine::<Waiting>::new(0);
        let in_filling = WorkingOrderMachine::<Filling>::from(in_waiting);
        // let in_filling2 = transition_the_states(in_waiting);
    }
}
