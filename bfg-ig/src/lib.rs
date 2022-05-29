use bfg_core::decider::{dax_system, Command, Event, OrderEvent, TradeResult};
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
use bfg_core::decider::system::{Long, Short, System};
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
    pub opening_range_high_ask: Option<f64>,
    pub opening_range_high_bid: Option<f64>,
    pub opening_range_low_ask: Option<f64>,
    pub opening_range_low_bid: Option<f64>,
    pub short_state: Option<String>,
    pub long_state: Option<String>,
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
            }).collect();
        IgEvent::TradesResultsView(views)
    }
}

#[derive(Debug, Default)]
struct AccountCache {
    account: Option<String>,
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
        if update.account.is_some() {
            self.account = update.account;
        }
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
    pub fn new(connection_details: ConnectionDetails, ig_tx: Sender<IgEvent>) -> Self {
        let (brokerage_tx, mut ig_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(100);
        tokio::spawn(async move {
            let brokerage = IgBrokerageApi::new(connection_details, brokerage_tx).await;
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
                    RealtimeEvent::WorkingOrderUpdate(_) => None, // Waiting for reply if WOU is depricated
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
                                Command::FetchData { start, end } => {
                                    info!("Executing: FetchData");
                                    if let Ok(result) = brokerage
                                        .rest
                                        .fetch_data(start.as_str(), end.as_str())
                                        .await
                                    {
                                        vec![Event::Data {
                                            prices: extract_prices(result),
                                        }]
                                    } else {
                                        vec![Event::Error(
                                            "Rest failure when fetching data".to_string(),
                                        )]
                                    }
                                }
                                Command::CreateWorkingOrder {
                                    direction,
                                    price,
                                    reference,
                                } => {
                                    info!("Executing: CreateWorkingOrder");
                                    if let Err(response) = brokerage
                                        .rest
                                        .open_working_order(direction, price, format!("{:?}", reference).as_str())
                                        .await
                                    {
                                        vec![Event::Error(
                                            "Rest failure when creating order".to_string(),
                                        )]
                                    } else {
                                        vec![]
                                    }
                                }
                                Command::UpdatePosition { deal_id, level } => {
                                    info!("Executing: UpdatePosition");
                                    if let Err(err) = brokerage
                                        .rest
                                        .edit_position(deal_id.as_str(), level)
                                        .await
                                    {
                                        vec![Event::Error(
                                            "Rest failure when updating position".to_string(),
                                        )]
                                    } else {
                                        vec![]
                                    }
                                }
                                Command::CancelWorkingOrder {
                                    ref reference_to_cancel,
                                } => {
                                    info!("Executing: CancelWorkingOrder");
                                    if let Some(deal_id) =
                                        trade_confirmation_cache.get_deal_id(reference_to_cancel)
                                    {
                                        if let Err(err) = brokerage
                                            .rest
                                            .delete_working_order(deal_id.as_str())
                                            .await
                                        {
                                            vec![Event::Error(
                                                "Rest failure when canceling working order"
                                                    .to_string(),
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
                                    vec![] // TODO i could send an event here to step system instead of waiting for next market event to get from PublishTradeResult
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
        System::Setup(_) => SystemView {
            state: String::from("Setup"),
            ..Default::default()
        },
        System::AwaitData(_) => SystemView {
            state: String::from("AwaitData"),
            ..Default::default()
        },
        System::DecideOrderPlacement(system) =>
            SystemView {
                state: String::from("DecideOrderPlacement"),
                opening_range_high_ask: Some(system.state.opening_range.high_ask),
                opening_range_high_bid: Some(system.state.opening_range.high_bid),
                opening_range_low_ask: Some(system.state.opening_range.low_ask),
                opening_range_low_bid: Some(system.state.opening_range.low_bid),
                ..Default::default()
            },
        System::ManageShort(system, Short(short)) => SystemView {
            state: String::from("ManageShort"),
            opening_range_high_ask: Some(system.state.opening_range.high_ask),
            opening_range_high_bid: Some(system.state.opening_range.high_bid),
            opening_range_low_ask: Some(system.state.opening_range.low_ask),
            opening_range_low_bid: Some(system.state.opening_range.low_bid),
            short_state: Some(short.to_string()),
            ..Default::default()
        },
        System::ManageLongAndShort(system, Long(long), Short(short)) => SystemView {
            state: String::from("ManageLongAndShort"),
            opening_range_high_ask: Some(system.state.opening_range.high_ask),
            opening_range_high_bid: Some(system.state.opening_range.high_bid),
            opening_range_low_ask: Some(system.state.opening_range.low_ask),
            opening_range_low_bid: Some(system.state.opening_range.low_bid),
            short_state: Some(short.to_string()),
            long_state: Some(long.to_string()),
            ..Default::default()
        },
        System::ManageLong(system, Long(long)) => SystemView {
            state: String::from("ManageLong"),
            opening_range_high_ask: Some(system.state.opening_range.high_ask),
            opening_range_high_bid: Some(system.state.opening_range.high_bid),
            opening_range_low_ask: Some(system.state.opening_range.low_ask),
            opening_range_low_bid: Some(system.state.opening_range.low_bid),
            long_state: Some(long.to_string()),
            ..Default::default()
        },
        System::Error(_) => SystemView {
            state: String::from("Error"),
            ..Default::default()
        },
    };
    IgEvent::SystemView(view)
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
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub market_delay: Option<usize>,
    pub market_state: Option<MarketState>,
    pub update_time: Option<NaiveTime>,
}

impl MarketCache {
    fn get_current_view(&self) -> IgEvent {
        IgEvent::MarketView(MarketView {
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
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationOpenAccepted { level: *level },
                    deal_reference.clone(),
                )),
                TradeConfirmationUpdate {
                    status: Some(PositionStatus::OPEN),
                    deal_status: DealStatus::REJECTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationOpenRejected,
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
                    status: Some(PositionStatus::AMENDED),
                    deal_status: DealStatus::REJECTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationAmendedRejected,
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
                TradeConfirmationUpdate {
                    status: Some(PositionStatus::DELETED),
                    deal_status: DealStatus::REJECTED,
                    ..
                } => Some(Event::Order(
                    OrderEvent::ConfirmationDeleteRejected,
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
