use bfg_core::decider::{dax_system, Command, Event, OrderEvent, TradeResult, MarketInfo};
use bfg_core::models::{OhlcPrice, OrderReference, Price};
use chrono::NaiveTime;
use ig_brokerage_adapter::realtime::models::{AccountUpdate, DealStatus, MarketState, MarketUpdate, OpenPositionUpdate, OpuStatus, PositionStatus, TradeConfirmationUpdate};
use ig_brokerage_adapter::rest::models::FetchDataResponse;
use ig_brokerage_adapter::{ConnectionDetails, IgBrokerageApi, RealtimeEvent};
use log::info;
use std::borrow::{Borrow};
use std::collections::{HashMap, LinkedList};
use std::str::FromStr;
use tokio::sync::mpsc::Sender;
use bfg_core::decider::order::WorkingOrder;
use bfg_core::decider::system::{System};
use ig_brokerage_adapter::errors::BrokerageError;
use crate::models::{AccountView, MarketView, ConnectionInformationView, TradeResultView};

pub mod models;

#[derive(Debug)]
pub enum IgEvent {
    MarketView(MarketView),
    ConnectionView(ConnectionInformationView),
    TradesResultsView(Vec<TradeResultView>),
    AccountView(AccountView),
    SystemView(SystemView)
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
struct TradeResultsCache {
    trades: Vec<TradeResult>,
}

impl TradeResultsCache {
    fn update(&mut self, update: TradeResult) {
        self.trades.push(update);
    }

    fn get_current_view(&self) -> IgEvent {
        let views = self.trades.iter()
            .map(|tr| TradeResultView {
                wanted_entry_level: tr.wanted_entry_level,
                actual_entry_level: tr.actual_entry_level,
                entry_time: tr.entry_time.to_string(),
                exit_time: tr.exit_time.to_string(),
                exit_level: tr.exit_level,
                reference: format!("{:?}", tr.reference),
                epic: tr.epic.clone(),
            }).collect();
        IgEvent::TradesResultsView(views)
    }
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

pub struct BfgIg {} // TODO why use a struct just spawn in a function

impl BfgIg {
    /// Spawns a new system and handles all the events from brokerage, system generates Commands that are
    /// executed hare also, ig_tx send back updates for the GUI to render
    pub fn new(connection_details: ConnectionDetails, market_infos: Vec<MarketInfo>, ig_tx: Sender<IgEvent>) -> Self {
        let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(10);
        tokio::spawn(async move {
            let brokerage = IgBrokerageApi::new(connection_details, market_infos, brokerage_tx).await;
            // TODO create SystemManager that uses market_infos then remove dax_system
            let mut system_cache = dax_system();
            let mut market_cache = MarketCache::default();
            let mut trade_confirmation_cache = TradeConfirmationCache::default();
            let mut open_position_cache = OpenPositionCache::default();
            let mut trade_results_cache = TradeResultsCache::default();
            let mut account_cache = AccountCache::default();
            while let Some(event) = ig_rx.recv().await {
                let core_event = match event {
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
                    let mut events: LinkedList<Event> = LinkedList::new();
                    events.push_back(bfg_e);
                    // Event -> Command -> Maybe Event -> Maybe Command
                    // This looping is so that commands can generate more events
                    while let Some(e) = events.pop_front() {
                        let (new_system, commands) = system_cache.step(e.borrow());
                        system_cache = new_system;
                        for c in commands {
                            let more_events = match c {
                                Command::FetchData {epic, start, duration } => {
                                    info!("Executing: FetchData");
                                    match brokerage
                                        .rest
                                        .fetch_data(epic.as_str(), start, duration)
                                        .await
                                    {
                                        Ok(result) =>
                                        vec![Event::Data {
                                            prices: extract_prices(result),
                                        }],
                                        Err(BrokerageError(error)) =>
                                        vec![Event::Error(error)]
                                    }
                                }
                                Command::CreateWorkingOrder {
                                    direction,
                                    price,
                                    reference,
                                    market_info,
                                    target_price,
                                } => {
                                    info!("Executing: CreateWorkingOrder");
                                    if let Err(BrokerageError(error)) = brokerage
                                        .rest
                                        .open_working_order(direction, price, format!("{:?}", reference).as_str(), market_info, target_price)
                                        .await
                                    {
                                        vec![Event::Error(error
                                        )]
                                    } else {
                                        vec![]
                                    }
                                }
                                Command::UpdatePosition { deal_id, level, trailing_stop_distance, target_level } => {
                                    info!("Executing: UpdatePosition");
                                    if let Err(BrokerageError(error)) = brokerage
                                        .rest
                                        .edit_position(deal_id.as_str(), level, trailing_stop_distance, target_level)
                                        .await
                                    {
                                        vec![Event::Error(error
                                        )]
                                    } else {
                                        vec![]
                                    }
                                }
                                /// Use the provided reference and the order cache to find order id to cancel
                                Command::CancelWorkingOrder {
                                    ref reference_to_cancel,
                                } => {
                                    info!("Executing: CancelWorkingOrder");
                                    if let Some(deal_id) =
                                        trade_confirmation_cache.get_deal_id(reference_to_cancel)
                                    {
                                        if let Err(BrokerageError(error)) = brokerage
                                            .rest
                                            .delete_working_order(deal_id.as_str())
                                            .await
                                        {
                                            vec![Event::Error(error
                                            )]
                                        } else {
                                            vec![]
                                        }
                                    } else { vec![] }
                                }
                                Command::PublishTradeResults(update) => {
                                    info!("Executing: PublishTradeResults");
                                    trade_results_cache.update(update);
                                    ig_tx.send(trade_results_cache.get_current_view()).await.expect("Failed sending message");
                                    vec![Event::PositionExit]
                                }
                                Command::FatalFailure(reason) => {
                                    info!("Executing: FatalFailure with reason {}", reason);
                                    vec![]
                                }
                            };
                            events.extend(more_events);
                        }
                    }
                }
                // Send updated system view on each event, might not always have a changed state so could optimize
                ig_tx.send(get_current_system_view(system_cache.borrow())).await.expect("Failed sending message");
            }
        });
        return BfgIg {};
    }
}

fn get_current_system_view(system: &System) -> IgEvent {
    let view = match system {
        System::Setup(val) => SystemView {
            state: String::from("Setup"),
            epic: val.market_info.epic.clone(),
            ..Default::default()
        },
        System::AwaitData(val) => SystemView {
            state: String::from("AwaitData"),
            epic: val.market_info.epic.clone(),
            ..Default::default()
        },
        System::DecideOrderPlacement(val) =>
            SystemView {
                state: String::from("DecideOrderPlacement"),
                opening_range_high_ask: Some(val.state.opening_range.high_ask),
                opening_range_high_bid: Some(val.state.opening_range.high_bid),
                opening_range_low_ask: Some(val.state.opening_range.low_ask),
                opening_range_low_bid: Some(val.state.opening_range.low_bid),
                epic: val.market_info.epic.clone(),
                ..Default::default()
            },
        System::ManageOrders(val) => SystemView {
            state: String::from("ManageOrders"),
            opening_range_high_ask: Some(val.state.opening_range.high_ask),
            opening_range_high_bid: Some(val.state.opening_range.high_bid),
            opening_range_low_ask: Some(val.state.opening_range.low_ask),
            opening_range_low_bid: Some(val.state.opening_range.low_bid),
            orders: create_order_view(val.state.order_manager.get_orders()),
            epic: val.market_info.epic.clone(),
            ..Default::default()
        },
        System::Error(val) => SystemView {
            state: String::from("Error"),
            epic: val.market_info.epic.clone(),
            ..Default::default()
        },
    };
    IgEvent::SystemView(view)
}

fn create_order_view(orders: &HashMap<OrderReference, WorkingOrder>) -> Vec<OrderView> {
    orders.iter()
        .map(|(k, v)| OrderView {
            reference: format!("{:?}", k),
            state: v.to_string(),
        }).collect()
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
    pub update_time: Option<NaiveTime>,
}

impl MarketCache {
    fn get_current_view(&self) -> IgEvent {
        IgEvent::MarketView(MarketView {
            epic: self.epic.clone(),
            bid: self.bid,
            ask: self.ask,
            market_delay: self.market_delay,
            market_state: self.market_state.clone().map(|s| format!("{:?}", s)),
            update_time: self.update_time.map(|t| t.to_string())
        })
    }
    /// Only update fields that has new values
    /// Returns the latest copy of the market
    fn update(&mut self, update: MarketUpdate) -> Option<Event> {
        if update.update_time.is_some() {
            self.update_time =
                Some(NaiveTime::from_str(update.update_time.unwrap().as_str()).unwrap());
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
    fn get_current_market_event(&self) -> Option<Event> {
        if self.is_filled_for_event() {
            if let Self {
                market_delay: Some(0),
                market_state: Some(MarketState::TRADEABLE),
                ..
            } = self
            {
                return Some(Event::Market {
                    update_time: self
                        .update_time
                        .expect("we know update_time always has value"),
                    bid: self.bid.expect("we know bid always has value"),
                    ask: self.ask.expect("we know ask always has value"),
                });
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

#[derive(Default, Debug, Clone)]
struct TradeConfirmationCache {
    confirms: HashMap<OrderReference, TradeConfirmationUpdate>,
}

impl TradeConfirmationCache {
    fn update(&mut self, update: TradeConfirmationUpdate) -> Option<Event> {
        let deal_reference: OrderReference = FromStr::from_str(update.deal_reference.as_str())
            .expect("Only supported deal references should be possible");
        self.confirms.insert(deal_reference.clone(), update);
        self.get_current_event(deal_reference.borrow())
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<Event> {
        let confirmation = self.confirms.get(deal_reference);
        if let Some(confirmation) = confirmation {
            return match confirmation {
                TradeConfirmationUpdate {
                    level: Some(level),
                    status: Some(PositionStatus::OPEN),
                    deal_status: DealStatus::ACCEPTED,
                    deal_id,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationOpenAccepted { level: *level, deal_id: deal_id.clone() },
                    deal_reference.clone(),
                )),
                TradeConfirmationUpdate {
                    status: Some(PositionStatus::AMENDED),
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationAmendedAccepted,
                    deal_reference.clone(),
                )),
                TradeConfirmationUpdate {
                    deal_status: DealStatus::REJECTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationRejection,
                    deal_reference.clone(),
                )),
                TradeConfirmationUpdate {
                    status: Some(PositionStatus::DELETED),
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationDeleteAccepted,
                    deal_reference.clone(),
                )),
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
    fn update(&mut self, update: OpenPositionUpdate) -> Option<Event> {
        let deal_reference: OrderReference = FromStr::from_str(update.deal_reference.as_str())
            .expect("Only supported deal references should be possible");
        self.positions.insert(deal_reference.clone(), update);
        self.get_current_event(deal_reference.borrow())
    }

    fn get_current_event(&self, deal_reference: &OrderReference) -> Option<Event> {
        let position = self.positions.get(deal_reference);
        if let Some(position) = position {
            return match position {
                OpenPositionUpdate {
                    level,
                    status: OpuStatus::OPEN,
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::PositionEntry {
                        entry_level: *level,
                    },
                    deal_reference.clone(),
                )),
                OpenPositionUpdate {
                    level,
                    status: OpuStatus::DELETED,
                    deal_status: DealStatus::ACCEPTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::PositionExit { exit_level: *level },
                    deal_reference.clone(),
                )),
                _ => None,
            }
        }
        None
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn it_works_factory() {}
}
