use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, LinkedList};
use std::str::FromStr;
use chrono::NaiveTime;
use log::info;
use tokio::sync::mpsc::Sender;
use bfg_core::BfgEvent::TradeConfirmation;
use bfg_core::decider::{Command, dax_system, Event, OrderEvent, OrderReference};
use bfg_core::decider::system::System;
use bfg_core::models::{MarketState, MarketUpdate, OhlcPrice, OrderReference, Price};
use ig_brokerage_adapter::{ConnectionDetails, IgBrokerageApi, RealtimeEvent};
use ig_brokerage_adapter::realtime::models::{DealStatus, OpenPositionUpdate, PositionStatus, TradeConfirmationUpdate};
use ig_brokerage_adapter::RealtimeEvent::MarketEvent;
use ig_brokerage_adapter::rest::models::FetchDataResponse;

#[derive(Debug)]
pub enum IgEvent {}

pub struct BfgIg {
}

#[derive(Clone, Debug, Default)]
pub struct MarketCache {
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub market_delay: Option<usize>,
    pub market_state: Option<MarketState>,
    pub update_time: Option<NaiveTime>,
}

impl MarketCache {
    /// Only update fields that has new values
    /// Returns the latest copy of the market
    fn update(&mut self, update: MarketUpdate) -> Option<Event> {
        if update.update_time.is_some() {
            self.update_time = Some(NaiveTime::from_str(update.update_time.unwrap().as_str()).unwrap());
        }
        if update.market_state.is_some() {
            self.market_state = update.market_state.clone();
        }
        if update.market_delay.is_some() {
            self.market_delay = update.market_delay;
        }
        if update.bid.is_some() {
            self.bid = update.bid;
        }
        if update.offer.is_some() {
            self.ask = update.offer;
        }
        self.get_current_market_event()
    }

    /// If there is a full market, that is all field are filled and the state
    /// of the market is Tradable and the market is not delayed
    fn get_current_market_event(&self) -> Option<Event >{
        if self.is_filled_for_event() {
            if let Self {market_delay: Some(0), market_state: Some(MarketState::TRADEABLE), ..} = self {
                return Some(Event::Market {
                    update_time: self.update_time.expect("we know update_time always has value"),
                    bid: self.bid.expect("we know bid always has value"),
                    ask: self.ask.expect("we know ask always has value"),
                })
            }
        }
        None
    }

    fn is_filled_for_event(&self) -> bool {
        self.bid.is_some() && self.market_state.is_some() && self.ask.is_some() && self.market_delay.is_some() && self.update_time.is_some()
    }
}

#[derive(Default, Debug, Clone)]
struct TradeConfirmationCache {
    confirms: HashMap<OrderReference, TradeConfirmationUpdate>,
}

impl TradeConfirmationCache {
    fn update(&mut self, update: TradeConfirmationUpdate) -> Option<Event> {
        let deal_reference: OrderReference = FromStr::from_str(update.deal_reference.as_str()).expect("Only supported deal references should be possible");
        self.confirms.insert(deal_reference.clone(), update);
        self.get_current_event(deal_reference.borrow())
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<Event >{
        let confirmation = self.confirms.get(deal_reference);
        if let Some(confirmation) = confirmation {
            match confirmation {
                TradeConfirmationUpdate {level: Some(level), status: Some(PositionStatus::OPEN), deal_status: DealStatus::ACCEPTED, ..} => Some(Event::Order(OrderEvent::ConfirmationOpenAccepted {level: *level}, deal_reference.clone())),
                TradeConfirmationUpdate {status: Some(PositionStatus::OPEN), deal_status: DealStatus::REJECTED, ..} => Some(Event::Order(OrderEvent::ConfirmationOpenRejected, deal_reference.clone())),
                TradeConfirmationUpdate {status: Some(PositionStatus::AMENDED), deal_status: DealStatus::ACCEPTED, ..} => Some(Event::Order(OrderEvent::ConfirmationAmendedAccepted, deal_reference.clone())),
                TradeConfirmationUpdate {status: Some(PositionStatus::AMENDED), deal_status: DealStatus::REJECTED, ..} => Some(Event::Order(OrderEvent::ConfirmationAmendedRejected, deal_reference.clone())),
                TradeConfirmationUpdate {status: Some(PositionStatus::DELETED), deal_status: DealStatus::ACCEPTED, ..} => Some(Event::Order(OrderEvent::ConfirmationDeleteAccepted, deal_reference.clone())),
                TradeConfirmationUpdate {status: Some(PositionStatus::DELETED), deal_status: DealStatus::REJECTED, ..} => Some(Event::Order(OrderEvent::ConfirmationDeleteRejected, deal_reference.clone())),
                _ => None
            }
        }
        None
    }

    fn get_deal_id(&self, deal_reference: &OrderReference) -> Option<String> {
        if let Some(confirm) = self.confirms.get(deal_reference) {
            Some(confirm.deal_id.clone())
        }
        None
    }
}

#[derive(Default, Debug, Clone)]
struct OpenPositionCache {
    positions: HashMap<OrderReference, OpenPositionUpdate>,
}

impl OpenPositionCache {
    fn update(&mut self, update: OpenPositionUpdate) -> Option<Event> {
        let deal_reference: OrderReference = FromStr::from_str(update.deal_reference.as_str()).expect("Only supported deal references should be possible");
        self.positions.insert(deal_reference.clone(), update);
        self.get_current_event(deal_reference.borrow())
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<Event >{
        let position = self.positions.get(deal_reference);
        if let Some(position) = position {
            match position {
                OpenPositionUpdate {level, status: Some(PositionStatus::OPEN), deal_status: DealStatus::ACCEPTED, ..} => Some(Event::Order(OrderEvent::PositionEntry {entry_level: *level}, deal_reference.clone())),
                OpenPositionUpdate {level, status: Some(PositionStatus::DELETED), deal_status: DealStatus::ACCEPTED, ..} => Some(Event::Order(OrderEvent::PositionExit {exit_level: *level}, deal_reference.clone())),
                _ => None
            }
        }
        None
    }
}
impl BfgIg {
    pub fn new(connection_details: ConnectionDetails, ig_tx: Sender<IgEvent>) -> Self {
        let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(100);
        tokio::spawn(async move {
            let brokerage = IgBrokerageApi::new(connection_details, brokerage_tx).await;
            let mut system_cache = dax_system();
            let mut market_cache = MarketCache::default();
            let mut trade_confirmation_cache = TadeConfirmationCache::default();
            let mut open_position_cache = OpenPositionCache::default();
            while let Some(event) = ig_rx.recv().await {
                let core_event = match event {
                    RealtimeEvent::MarketEvent(update) => market_cache.update(update),
                    RealtimeEvent::TradeConfirmation(update) => trade_confirmation_cache.update(update),
                    RealtimeEvent::AccountPositionUpdate(update) => open_position_cache.update(update),
                    // TODO Imp sending status to TUI
                    RealtimeEvent::AccountEvent(_) => None,
                    RealtimeEvent::WorkingOrderUpdate(_) => None,
                    RealtimeEvent::StreamStatus(_) => None,
                };

                // If there was an event that will effect the trade system execure it
                if let Some(bfg_e) = core_event {
                    let mut events: LinkedList<Event> = LinkedList::new();
                    events.push_back(bfg_e);
                    // Event -> Command -> Maybe Event -> Maybe Command
                    // This looping is so that commands can generate more events
                    while let Some(e) = events.pop_front() {
                        let (new_system, commands) = system_cache.step(e.borrow());
                        system_cache = new_system;
                        for c in commands {
                            let more_events = match c {
                                Command::FetchData {start, end} => {
                                    info!("Executing: FetchData");
                                    if let Ok(result) = brokerage.rest.fetch_data(start.as_str(), end.as_str()).await {
                                        vec![Event::Data {prices: extract_prices(result)}]
                                    } else {
                                        // Empty prices is failure
                                        vec![Event::Data {prices: vec![]}]
                                    }
                                }
                                Command::CreateWorkingOrder{direction, price, reference} => {
                                    info!("Executing: CreateWorkingOrder");
                                    brokerage.rest.open_working_order(direction, price, reference.into()).await.expect("Open working order never fail doo");
                                    vec![]
                                },
                                Command::UpdatePosition{deal_id, level} => {
                                    info!("Executing: UpdatePosition");
                                    brokerage.rest.edit_position(deal_id.as_str(), level).await.expect("Edit position never fail doo");
                                    vec![]
                                },
                                Command::CancelWorkingOrder{ref reference_to_cancel} => {
                                    info!("Executing: CancelWorkingOrder");
                                    if let Some(deal_id) = trade_confirmation_cache.get_deal_id(reference_to_cancel) {
                                        brokerage.rest.delete_working_order(deal_id).await.expect("Cancel working order never fail doo");
                                    }
                                    vec![]
                                }
                                Command::PublishTradeResults{..} => {
                                    info!("Executing: PublishTradeResults");
                                    vec![]
                                },
                            };
                            events.extend(more_events);
                        }
                    }
                }



            }
        });
        return BfgIg {};
    }
}

fn extract_prices(res: FetchDataResponse) -> Vec<OhlcPrice>{
    res
        .prices
        .iter()
        .map(|p| OhlcPrice {
            open: Price {
                bid: p.open_price.bid,
                ask: p.open_price.ask,
            },
            high: Price {
                bid: p.high_price.bid,
                ask: p.high_price.ask,
            },
            low: Price {
                bid: p.low_price.bid,
                ask: p.low_price.ask,
            },
            close: Price {
                bid: p.close_price.bid,
                ask: p.close_price.ask,
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works_factory() {
    }

}
