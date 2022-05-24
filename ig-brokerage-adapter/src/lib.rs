use crate::errors::BrokerageError;
use crate::realtime::models::{
    OpenPositionUpdate, RestDetails, TradeConfirmationUpdate, WorkingOrderUpdate,
};
use crate::realtime::IgStreamClient;
use crate::rest::models::FetchDataResponse;
use crate::rest::{HasSession, IgRestClient};
use bfg_core::models::{
    AccountUpdate, DataUpdate, Decision, FetchDataDetails, MarketOrderDetails, MarketUpdate,
    OhlcPrice, Price, TradeConfirmation, WorkingOrderDetails,
};
use bfg_core::BfgEvent;
use std::env;
use std::sync::Arc;
use log::error;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;

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
}

#[derive(Debug, Default)]
pub struct SessionState {
    xst: String,
    cst: String,
    lightstreamer_endpoint: String,
    account: String,
}

#[derive(Clone)]
pub struct ConnectionDetails {
    pub username: String,
    pub password: String,
    pub api_key: String,
    pub account: String,
    pub base_url: String,
    pub epic: String,
}

impl ConnectionDetails {
    pub fn from_env() -> Self {
        Self {
            username: env::var("IG_USER").expect("IG_USER not set"),
            password: env::var("IG_PASSWORD").expect("IG_PASSWORD not set"),
            api_key: env::var("IG_APIKEY").expect("IG_API_KEY not set"),
            base_url: env::var("IG_BASEURL").expect("IG_BASEURL not set"),
            account: env::var("IG_ACCOUNT").expect("IG_ACCOUNT not set"),
            epic: env::var("EPIC").expect("EPIC not set"),
        }
    }
}

// TODO session should be an Option<Arc...> then use .expect in the states where it should always be set
pub struct IgBrokerageApi {
    _session: Arc<Mutex<SessionState>>,
    rest: IgRestClient<HasSession>,
    _stream: IgStreamClient,
}

impl IgBrokerageApi {
    pub async fn new(connection_details: ConnectionDetails, tx_out: Sender<RealtimeEvent>) -> Self {
        let session = Arc::new(Mutex::new(SessionState::default()));

        // Setup a session with rest client and make sure stream has proper connection details
        let disconnected_rest = IgRestClient::new(Arc::clone(&session), connection_details);
        let connected_rest = disconnected_rest.create_session().await.unwrap();

        // Connect to stream and setup subscriptions
        let stream = IgStreamClient::new(Arc::clone(&session), tx_out);
        stream.start().await.unwrap();

        Self {
            _session: session,
            rest: connected_rest,
            _stream: stream,
        }
    }

    pub async fn execute_decision(
        &self,
        decision: Decision,
    ) -> Result<Option<BfgEvent>, BrokerageError> {
        match decision {
            Decision::FetchData(FetchDataDetails { start, end }) => {
                error!("Fetching data");
                let res = self
                    .rest
                    .fetch_data(start.as_str(), end.as_str())
                    .await
                    .expect("Fetch data should never fail doo");
                return Ok(Some(BfgEvent::Data(create_data_update(res))));
                // return Ok(Some(BfgEvent::Data(create_data_update_noargs())));
            }
            Decision::CreateWorkingOrder(WorkingOrderDetails {
                direction,
                price,
                reference,
            }) => {
                error!("Creating WO");
                self.rest
                    .open_working_order(direction, price, format!("{:?}", reference).as_str())
                    .await?;
            }
            Decision::CancelWorkingOrder(deal_id) => {
                error!("Cancel WO");
                self.rest.delete_working_order(deal_id.as_str()).await?;
            }
            Decision::UpdateWithTrailingStop(deal_id, stop_level) => {
                error!("Update with Trailing stop");
                self.rest
                    .edit_position(deal_id.as_str(), stop_level)
                    .await?;
            }
            Decision::NoOp => return Ok(None),
        }
        Ok(None)
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
