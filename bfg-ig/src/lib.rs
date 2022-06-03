use bfg_core::decider::{Command, Event, OrderEvent, TradeResult, MarketInfo};
use chrono_tz::Europe::{London, Stockholm};
use bfg_core::models::{OhlcPrice, OrderReference, Price};
use chrono::{DateTime, NaiveDateTime, NaiveTime, Timelike, Utc};
use ig_brokerage_adapter::realtime::models::{AccountUpdate, DealStatus, MarketState, MarketUpdate, OpenPositionUpdate, OpuStatus, PositionStatus, TradeConfirmationUpdate};
use ig_brokerage_adapter::rest::models::FetchDataResponse;
use ig_brokerage_adapter::{ConnectionDetails, IgBrokerageApi, RealtimeEvent};
use log::{debug, info};
use std::borrow::{Borrow};
use std::collections::{HashMap, LinkedList};
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use bfg_core::decider::order::WorkingOrder;
use bfg_core::decider::system::{System, SystemFactory};
use ig_brokerage_adapter::errors::BrokerageError;
use crate::file_writer::write_results_to_file;
use crate::models::{AccountView, MarketView, ConnectionInformationView, TradeResultView};
use crate::systems_manager::SystemsManager;

pub mod models;
mod file_writer;
mod systems_manager;

#[derive(Debug)]
pub enum IgEvent {
    MarketView(String, MarketView),
    ConnectionView(ConnectionInformationView),
    TradesResultsView(TradeResultView),
    AccountView(AccountView),
    SystemView(String, SystemView)
}

#[derive(Debug, Default)]
pub struct SystemView {
    pub state: String,
    pub epic: String,
    pub opening_range_high_ask: Option<f64>,
    pub opening_range_high_bid: Option<f64>,
    pub opening_range_low_ask: Option<f64>,
    pub opening_range_low_bid: Option<f64>,
    pub orders: Vec<OrderView>,
}

#[derive(Debug, Default)]
pub struct OrderView {
    pub reference: String,
    pub state: String,
}

#[derive(Debug, Default)]
struct AccountCache {
    account: String,
    pnl: Option<f64>,
    deposit: Option<f64>,
    available_cash: Option<f64>,
    pnl_lr: Option<f64>,
    pnl_nlr: Option<f64>,
    funds: Option<f64>,
    margin: Option<f64>,
    margin_lr: Option<f64>,
    margin_nlr: Option<f64>,
    available_to_deal: Option<f64>,
    equity: Option<f64>,
    equity_used: Option<f64>,
}

impl AccountCache {
    fn update(&mut self, update: AccountUpdate) {
        self.account = update.account;
        if update.pnl.is_some() {
            self.pnl= update.pnl;
        }
        if update.deposit.is_some() {
            self.deposit = update.deposit;
        }
        if update.available_cash.is_some() {
            self.available_cash = update.available_cash;
        }
        if update.pnl_lr.is_some() {
            self.pnl_lr = update.pnl_lr;
        }
        if update.pnl_nlr.is_some() {
            self.pnl_nlr = update.pnl_nlr;
        }
        if update.funds.is_some() {
            self.funds = update.funds;
        }
        if update.margin.is_some() {
            self.margin = update.margin;
        }
        if update.margin_lr.is_some() {
            self.margin_lr = update.margin_lr;
        }
        if update.margin_nlr.is_some() {
            self.margin_nlr = update.margin_nlr;
        }
        if update.available_to_deal.is_some() {
            self.available_to_deal= update.available_to_deal;
        }
        if update.equity.is_some() {
            self.equity= update.equity;
        }
        if update.equity_used.is_some() {
            self.equity_used= update.equity_used;
        }
    }

    fn get_current_view(&self) -> IgEvent {
        IgEvent::AccountView(
        AccountView {
            account: self.account.clone(),
            pnl: self.pnl,
            deposit: self.deposit,
            available_cash: self.available_cash,
            pnl_lr: self.pnl_lr,
            pnl_nlr: self.pnl_nlr,
            funds: self.funds,
            margin: self.margin,
            margin_lr: self.margin_lr,
            margin_nlr: self.margin_nlr,
            available_to_deal: self.available_to_deal,
            equity: self.equity,
            equity_used: self.equity_used
        })
    }
}

/// Spawns a new system and handles all the events from brokerage, system generates Commands that are
/// executed hare also, ig_tx send back updates for the GUI to render
pub fn spawn_bfg(connection_details: ConnectionDetails, market_infos: Vec<MarketInfo>, ig_tx: Sender<IgEvent>) {
    let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(10);
    tokio::spawn(async move {
        let mut systems_manager = SystemsManager::new(&market_infos[..]);
        let brokerage = IgBrokerageApi::new(connection_details, market_infos, brokerage_tx).await;
        let mut market_cache = MarketCache::default();
        let mut trade_confirmation_cache = TradeConfirmationCache::default();
        let mut open_position_cache = OpenPositionCache::default();
        let mut account_cache = AccountCache::default();
        while let Some(event) = ig_rx.recv().await {
            let core_event: Option<(String, Event)> = match event {
                RealtimeEvent::MarketEvent(update) => {
                    let system_event = market_cache.update(update);
                    ig_tx.send(market_cache.get_current_view()).await.expect("Sending message failure");
                    system_event
                },
                RealtimeEvent::TradeConfirmation(update) => {
                    trade_confirmation_cache.update(update)
                }
                RealtimeEvent::AccountPositionUpdate(update) => {
                    open_position_cache.update(update)
                }
                RealtimeEvent::AccountEvent(update) => {
                    account_cache.update(update);
                    ig_tx.send(account_cache.get_current_view()).await.expect("Sending message failure");
                    None
                },
                RealtimeEvent::WorkingOrderUpdate(_) => None, // Waiting for reply if WOU is deprecated
                RealtimeEvent::StreamStatus(status) => {
                    ig_tx.send(IgEvent::ConnectionView(ConnectionInformationView {
                        stream_status: status
                    })).await.expect("Sending message failure");
                    None
                }
            };

            // If there was an event that will effect the trade system then execute it
            if let Some(bfg_e) = core_event {
                let mut events: LinkedList<(String, Event)> = LinkedList::new();
                events.push_back(bfg_e);
                // Event -> Command -> Maybe Event -> Maybe Command
                // This looping is so that commands can generate more events
                while let Some((epic, event)) = events.pop_front() {
                    let commands = systems_manager.step_one(epic.clone(), &event) ;
                    if let Event::Market {..} = event {
                    } else {
                        debug!("Epic: {} Event: {:?}", epic, event);
                    }
                    for c in commands {
                        debug!("Epic: {} Command: {:?}", epic, c);
                        let more_events: Vec<(String, Event)> = match c {
                            Command::FetchData {epic, start, duration } => {
                                info!("Executing: FetchData for {}", epic);
                                match brokerage
                                    .rest
                                    .fetch_data(epic.as_str(), start, duration)
                                    .await
                                {
                                    Ok(result) =>
                                    vec![(epic, Event::Data {
                                        prices: extract_prices(result),
                                    })],
                                    Err(BrokerageError(error)) =>
                                    vec![(epic, Event::Error(error))]
                                }
                            }
                            Command::CreateWorkingOrder {
                                direction,
                                price,
                                reference,
                                market_info,
                                target_distance,
                                stop_distance
                            } => {
                                info!("Executing: CreateWorkingOrder for {}, target: {} stop: {}",epic, target_distance, stop_distance);
                                let epic = market_info.epic.clone();
                                if let Err(BrokerageError(error)) = brokerage
                                    .rest
                                    .open_working_order(direction, price, format!("{:?}", reference).as_str(), market_info, target_distance, stop_distance)
                                    .await
                                {
                                    vec![(epic, Event::Error(error
                                    ))]
                                } else {
                                    vec![]
                                }
                            }
                            Command::UpdatePosition {epic, deal_id, level, trailing_stop_distance, target_distance, reference} => {
                                info!("Executing: UpdatePosition for {}", epic);
                                if let Err(BrokerageError(error)) = brokerage
                                    .rest
                                    .edit_position(deal_id.as_str(), level, trailing_stop_distance, target_distance)
                                    .await
                                {
                                    // TODO rough to retry everything, probably only want this for som errors
                                    vec![(epic, Event::Order(OrderEvent::RejectedAtRestApi, reference))]
                                } else {
                                    vec![]
                                }
                            }
                            /// Use the provided reference and the order cache to find order id to cancel
                            Command::CancelWorkingOrder {
                                epic,
                                ref reference_to_cancel,
                            } => {
                                info!("Executing: CancelWorkingOrder for {}", epic);
                                if let Some(deal_id) =
                                    trade_confirmation_cache.get_deal_id(reference_to_cancel)
                                {
                                    if let Err(BrokerageError(error)) = brokerage
                                        .rest
                                        .delete_working_order(deal_id.as_str())
                                        .await
                                    {
                                        vec![(epic, Event::Error(error
                                        ))]
                                    } else {
                                        vec![]
                                    }
                                } else { vec![] }
                            }
                            Command::PublishTradeResults(tr) => {
                                info!("Executing: PublishTradeResults for {}", epic);
                                let epic = tr.epic.clone();
                                write_results_to_file(tr.clone());
                                let view = TradeResultView {
                                        wanted_entry_level: tr.wanted_entry_level,
                                        actual_entry_level: tr.actual_entry_level,
                                        entry_time: tr.entry_time.to_string(),
                                        exit_time: tr.exit_time.to_string(),
                                        exit_level: tr.exit_level,
                                        reference: format!("{:?}", tr.reference),
                                        epic: tr.epic.clone(),
                                    };
                                ig_tx.send(IgEvent::TradesResultsView(view)).await.expect("Failed sending message");
                                vec![(epic.clone(), Event::PositionExit(tr.reference.clone()))]
                            }
                            Command::Restart(reference) => {
                                vec![(epic.clone(), Event::WOCancel(reference))]
                            },
                            Command::FatalFailure(reason) => {
                                info!("Executing: FatalFailure for {} with reason {}",epic, reason);
                                vec![]
                            }
                        };
                        events.extend(more_events);
                    }
                    ig_tx.send(systems_manager.get_current_system_view(epic.clone())).await.expect("Failed sending message");
                }
            }
        }
    });
}


fn extract_prices(res: FetchDataResponse) -> Vec<OhlcPrice> {
    res.prices
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

#[derive(Clone, Debug, Default)]
pub struct MarketCache {
    pub epic: String,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub market_delay: Option<usize>,
    pub market_state: Option<MarketState>,
    pub update_time: Option<DateTime<Utc>>,
}

impl MarketCache {
    fn get_current_view(&self) -> IgEvent {
        IgEvent::MarketView(self.epic.clone(),MarketView {
            epic: self.epic.clone(),
            bid: self.bid,
            ask: self.ask,
            market_delay: self.market_delay,
            market_state: self.market_state.clone().map(|s| format!("{:?}", s)),
            update_time: self.update_time.map(|t| t.time().format("%H:%M:%S").to_string())
        })
    }
    /// Only update fields that has new values
    /// Returns the latest copy of the market
    fn update(&mut self, update: MarketUpdate) -> Option<(String, Event)> {
        self.epic = update.epic;
        if update.update_time.is_some() {
            self.update_time = get_utc_time_for_update(&update.update_time.unwrap());
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
    fn get_current_market_event(&self) -> Option<(String, Event)> {
        if self.is_filled_for_event() {
            if let Self {
                market_delay: Some(0),
                market_state: Some(MarketState::TRADEABLE),
                ..
            } = self
            {
                return Some((self.epic.clone(), Event::Market {
                    update_time: self
                        .update_time
                        .expect("we know update_time always has value"),
                    bid: self.bid.expect("we know bid always has value"),
                    ask: self.ask.expect("we know ask always has value"),
                    epic: self.epic.clone(),
                }));
            }
        }
        None
    }

    fn is_filled_for_event(&self) -> bool {
        self.bid.is_some()
            && self.market_state.is_some()
            && self.ask.is_some()
            && self.market_delay.is_some()
            && self.update_time.is_some()
    }
}

/// Stream update times are in GMT or BST depending on daylight savings
/// We want them to be converted to Utc
fn get_utc_time_for_update(update_time: &String) -> Option<DateTime<Utc>> {
    let update_time = NaiveTime::from_str(update_time).unwrap();
    // Here update_time is with London timezone
    let updae_time = Utc::now().with_timezone(&London)
        .with_hour(update_time.hour()).unwrap()
        .with_minute(update_time.minute()).unwrap()
        .with_second(update_time.second()).unwrap()
        .with_timezone(&Utc);
    Some(updae_time)
}

#[derive(Default, Debug, Clone)]
struct TradeConfirmationCache {
    confirms: HashMap<OrderReference, TradeConfirmationUpdate>,
}

impl TradeConfirmationCache {
    fn update(&mut self, update: TradeConfirmationUpdate) -> Option<(String, Event)> {
        let deal_reference: Option<OrderReference> = FromStr::from_str(update.deal_reference.as_str()).ok();
        // Only cara about updates for known OrderReferences, since i get an old snapshot if i have done orders in gui or api i can get a snapshot with
        if let Some(reference) = deal_reference {
            self.confirms.insert(reference.clone(), update);
            self.get_current_event(reference.borrow())
        } else { None }
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<(String, Event)> {
        let confirmation = self.confirms.get(deal_reference);
        if let Some(confirmation) = confirmation {
            return match confirmation {
                TradeConfirmationUpdate {
                    epic,
                    level: Some(level),
                    status: Some(PositionStatus::OPEN),
                    deal_status: DealStatus::ACCEPTED,
                    deal_id,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::ConfirmationOpenAccepted { level: *level, deal_id: deal_id.clone() },
                    deal_reference.clone(),
                ))),
                TradeConfirmationUpdate {
                    epic,
                    status: Some(PositionStatus::AMENDED),
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::ConfirmationAmendedAccepted,
                    deal_reference.clone(),
                ))),
                TradeConfirmationUpdate {
                    epic,
                    deal_status: DealStatus::REJECTED,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::ConfirmationRejection,
                    deal_reference.clone(),
                ))),
                TradeConfirmationUpdate {
                    epic,
                    status: Some(PositionStatus::DELETED),
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::ConfirmationDeleteAccepted,
                    deal_reference.clone(),
                ))),
                _ => None,
            }
        }
        None
    }

    fn get_deal_id(&self, deal_reference: &OrderReference) -> Option<String> {
        if let Some(confirm) = self.confirms.get(deal_reference) {
            Some(confirm.deal_id.clone())
        } else {
            None
        }
    }
}

#[derive(Default, Debug, Clone)]
struct OpenPositionCache {
    positions: HashMap<OrderReference, OpenPositionUpdate>,
}

impl OpenPositionCache {
    fn update(&mut self, update: OpenPositionUpdate) -> Option<(String, Event)> {
        let deal_reference: OrderReference = FromStr::from_str(update.deal_reference.as_str())
            .expect("Only supported deal references should be possible");
        self.positions.insert(deal_reference.clone(), update);
        self.get_current_event(deal_reference.borrow())
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<(String, Event)> {
        let position = self.positions.get(deal_reference);
        if let Some(position) = position {
            return match position {
                OpenPositionUpdate {
                    epic,
                    level,
                    status: OpuStatus::OPEN,
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::PositionEntry {
                        entry_level: *level,
                    },
                    deal_reference.clone(),
                ))),
                OpenPositionUpdate {
                    epic,
                    level,
                    status: OpuStatus::DELETED,
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some((epic.clone(), Event::Order(
                    OrderEvent::PositionExit { exit_level: *level },
                    deal_reference.clone(),
                ))),
                _ => None,
            }
        }
        None
    }
}
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use chrono::{NaiveTime, Timelike, Utc};
    use chrono_tz::Europe::London;

    #[test]
    fn it_works_factory() {
        let update_time = NaiveTime::from_str("12:0:0").unwrap();
        // Here update_time is with London timezone
        let updae_time = Utc::now().with_timezone(&London)
            .with_hour(update_time.hour()).unwrap()
            .with_minute(update_time.minute()).unwrap()
            .with_second(update_time.second()).unwrap()
            .with_timezone(&Utc);
        println!("{}", updae_time.to_string())
    }
}
