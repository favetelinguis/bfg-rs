use std::collections::LinkedList;
use tokio::sync::mpsc::Sender;
use bfg_core::decider::{Command, dax_system};
use ig_brokerage_adapter::{ConnectionDetails, IgBrokerageApi, RealtimeEvent};

#[derive(Debug)]
pub enum IgEvent {}

pub struct BfgIg {
}

impl BfgIg {
    pub fn new(connection_details: ConnectionDetails, ig_tx: Sender<IgEvent>) -> Self {
        let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(100);
        tokio::spawn(async move {
            let brokerage = IgBrokerageApi::new(connection_details, brokerage_tx).await;
            let mut system = dax_system();
            while let Some(event) = ig_rx.recv().await {
                let mut events: LinkedList<RealtimeEvent> = LinkedList::new();
                events.push_back(event);
                // Event -> Command -> Maybe Event -> Maybe Command
                // This looping is so that commands can generate more events
                while let Some(e) = events.pop_front() {
                    let (new_system, commands) = system.step(e.into()); // AsRef maybe to transform events
                    system = new_system;
                    for c in commands {
                        let more_events = match c {
                            Command::FetchData {..} => {
                                // TODO remove execute decision
                                let data = brokerage.rest.fetch_data().await;
                                ig_tx.send().await.expect("Cant send event to TUI");
                            }
                            Command::CreateWorkingOrder{..} => {}
                            Command::UpdatePosition{..} => {}
                            Command::CancelWorkingOrder{..} => {
                                // TODO Need to find deal_id from reference store some cache here
                            }
                            Command::PublishTradeResults{..} => {}
                        };
                        events.extend(more_events);
                    }
                }
            }
        });
        return BfgIg {};
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works_factory() {
    }

}
