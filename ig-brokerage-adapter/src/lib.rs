use std::borrow::Borrow;
use crate::errors::BrokerageError;
use crate::realtime::models::{AccountUpdate, MarketUpdate, OpenPositionUpdate, RestDetails, TradeConfirmationUpdate, WorkingOrderUpdate};
use crate::realtime::IgStreamClient;
use crate::rest::models::FetchDataResponse;
use crate::rest::{HasSession, IgRestClient};
use bfg_core::models::{
    DataUpdate, Decision, FetchDataDetails,
    OhlcPrice, Price, TradeConfirmation, WorkingOrderDetails,
};
use log::{error, info, log, warn};
use std::env;
use std::ops::{Add, Sub};
use std::sync::Arc;
use chrono::{Duration, Utc};
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use bfg_core::decider::MarketInfo;
use chrono_tz::Europe::{Stockholm};
use tokio::select;
use tokio::time::{Instant, interval_at, sleep_until};

pub mod errors;
pub mod realtime;
pub mod rest;

#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    MarketEvent(MarketUpdate),
    AccountEvent(AccountUpdate),
    TradeConfirmation(TradeConfirmationUpdate),
    AccountPositionUpdate(OpenPositionUpdate),
    WorkingOrderUpdate(WorkingOrderUpdate),
    StreamStatus(String),
    AtrEvent(String),
    QuitSystem(String),
}

#[derive(Debug, Default)]
pub struct SessionState {
    xst: String,
    cst: String,
    lightstreamer_endpoint: String,
    account: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionDetails {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub account: String,
    pub base_url: String,
}

// TODO improve with type sessions so that session is avaliable only in the correct states.
// Try to always have session in top IgBrokerageApi and then control access with getter setter in traits
pub struct IgBrokerageApi {
    _session: Arc<Mutex<SessionState>>,
    pub rest: IgRestClient<HasSession>,
    _stream: IgStreamClient,
    tx_out: Sender<RealtimeEvent>,
}

impl IgBrokerageApi {
    pub async fn new(connection_details: ConnectionDetails, epics: Vec<String>, tx_out: Sender<RealtimeEvent>) -> Self {
        let session = Arc::new(Mutex::new(SessionState::default()));

        // Setup a session with rest client and make sure stream has proper connection details
        let disconnected_rest = IgRestClient::new(Arc::clone(&session), connection_details.clone());
        let connected_rest = disconnected_rest.create_session().await.unwrap();

        // Connect to stream and setup subscriptions
        let stream = IgStreamClient::new(Arc::clone(&session), tx_out.clone());
        stream.start(connection_details, epics).await.unwrap();

        Self {
            _session: session,
            rest: connected_rest,
            _stream: stream,
            tx_out,
        }
    }
    /// Schedule an ATR update 15 min after market open and then every 15 minutes, also schedule a
    /// stop event and kills the ATR update 5 min before market close
    pub fn schedule_atr_update(&self, markets: &[MarketInfo]) {
        for m in markets {
            // If the market is closed this will fail
            let five_min_before_close = m.utc_close_time.sub(Duration::minutes(5));
            if let Ok(five_min_before_close) = five_min_before_close.signed_duration_since(Utc::now()).to_std() {
                let five_min_before_close = Instant::now() + five_min_before_close;
                let epic = m.epic.clone();
                let epic_clone = m.epic.clone();
                let tx_out_atr = self.tx_out.clone();
                let tx_out_close = self.tx_out.clone();
                let now = Utc::now();
                let fifteen_min_after_open = m.utc_open_time.add(Duration::minutes(15));
                let mut start_instant= Instant::now();
                if now < fifteen_min_after_open {
                    start_instant += fifteen_min_after_open.signed_duration_since(Utc::now()).to_std().unwrap();
                }
                let mut interval = interval_at(start_instant, core::time::Duration::from_secs(60 * 30)); // Every 30 minutes since there is a api limit of 10k history/week so for 6 markets 30 min update is 8100 datapoints/week
                tokio::spawn(async move {

                    let atr_ticker = tokio::spawn(async move {
                        while let tick = interval.tick().await {
                            tx_out_atr.send(RealtimeEvent::AtrEvent(epic_clone.clone())).await.unwrap();
                        }
                    });
                    let end = sleep_until(five_min_before_close);
                    select! {
                _ = atr_ticker => {}
                _ = end => tx_out_close.send(RealtimeEvent::QuitSystem(epic)).await.unwrap(),
            }
                });
            } else {
                warn!("Market is closed so no ATR updates will be activated for {}", m.epic.clone());
            }
        };
    }
}

fn create_data_update(res: FetchDataResponse) -> DataUpdate {
    let ohlc = res
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
        .collect();

    DataUpdate { prices: ohlc }
}

fn create_data_update_noargs() -> DataUpdate {
    let middle_high = 14050.6;
    let middle_low = 14017.1;

    let ohlc = OhlcPrice {
        open: Price { ask: 0., bid: 0. },
        high: Price {
            ask: middle_high + 0.7,
            bid: middle_high - 0.7,
        },
        low: Price {
            ask: middle_low + 0.7,
            bid: middle_low - 0.7,
        },
        close: Price { ask: 0., bid: 0. },
    };

    DataUpdate { prices: vec![ohlc] }
}
