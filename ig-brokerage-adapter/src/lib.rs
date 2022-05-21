use crate::errors::BrokerageError;
use crate::realtime::models::{
    OpenPositionUpdate, RestDetails, TradeConfirmationUpdate, WorkingOrderUpdate,
};
use crate::realtime::IgStreamClient;
use crate::rest::models::{AccessTokenResponse, FetchDataResponse};
use bfg_core::models::{AccountUpdate, DataUpdate, Decision, FetchDataDetails, LimitOrderDetails, MarketOrderDetails, MarketUpdate, OhlcPrice, Price, TradeConfirmation, WorkingOrderDetails};
use bfg_core::BfgEvent;
use reqwest::Client;
use std::borrow::Borrow;
use std::env;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

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
    RefreshToken,
}

#[derive(Debug)]
pub enum SessionState {
    NoSession,
    HasSession(AccessTokenResponse),
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

pub struct IgBrokerageApi {
    connection_details: ConnectionDetails,
    http_client: Client,
    session: SessionState,
    tx_out: Sender<RealtimeEvent>,
}

impl IgBrokerageApi {
    pub fn new(connection_details: ConnectionDetails, tx_out: Sender<RealtimeEvent>) -> Self {
        Self {
            connection_details,
            http_client: Client::builder().build().unwrap(),
            session: SessionState::NoSession,
            tx_out,
        }
    }

    pub async fn connect(&mut self) -> Result<(), BrokerageError> {
        if let SessionState::NoSession = self.session {
            // POST /session
            let session =
                rest::create_session(self.http_client.clone(), self.connection_details.borrow())
                    .await?;

            // Start a token refresh task
            let refresh_tx = self.tx_out.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(Duration::from_secs(40));
                loop {
                    interval.tick().await;
                    refresh_tx.send(RealtimeEvent::RefreshToken).await.unwrap();
                }
            });
            // GET /session
            let (xst, cst) = rest::get_session(
                self.http_client.clone(),
                self.connection_details.borrow(),
                session.oauth_token.access_token.borrow(),
            )
            .await?;

            // Setup details needed for stream
            let rest_details = RestDetails {
                xst,
                cst,
                url: session.lightstreamer_endpoint,
                account: self.connection_details.account.clone(),
            };

            IgStreamClient::start(rest_details, self.tx_out.clone()).await?;

            self.session = SessionState::HasSession(session.oauth_token);
        }

        Ok(())
    }

    pub async fn update_session(&mut self) {
        if let SessionState::HasSession(AccessTokenResponse {
            ref refresh_token, ..
        }) = self.session
        {
            let update = rest::refresh_token(
                self.http_client.clone(),
                self.connection_details.borrow(),
                refresh_token,
            )
            .await
            .unwrap();
            self.session = SessionState::HasSession(update);
        }
    }

    pub async fn execute_decision(
        &self,
        decision: Decision,
    ) -> Result<Option<BfgEvent>, BrokerageError> {
        if let SessionState::HasSession(AccessTokenResponse {
            ref access_token, ..
        }) = self.session
        {
            match decision {
                Decision::FetchData(FetchDataDetails { start, end }) => {
                    let res = rest::fetch_data(
                        self.http_client.clone(),
                        self.connection_details.borrow(),
                        access_token,
                        start.as_str(),
                        end.as_str(),
                    )
                    .await
                    .expect("Fetch data should never fails doo");
                    return Ok(Some(BfgEvent::Data(create_data_update(res))));
                }
                Decision::CreateWorkingOrder(WorkingOrderDetails {
                                                 direction, price, reference
                                             }) => {
                    rest::open_working_order(
                        self.http_client.clone(),
                        self.connection_details.borrow(),
                        access_token,
                        direction,
                        price,
                        format!("{:?}", reference).as_str(),
                    )
                    .await;
                }
                Decision::CancelWorkingOrder(deal_id) => {
                    rest::delete_working_order(
                        self.http_client.clone(),
                        self.connection_details.borrow(),
                        access_token,
                        deal_id.as_str(),
                    )
                        .await;
                }
                Decision::UpdateWithTrailingStop(deal_id, stop_level) => {
                    rest::edit_position(
                        self.http_client.clone(),
                        self.connection_details.borrow(),
                        access_token,
                        deal_id.as_str(),
                        stop_level
                    )
                        .await;
                }
                Decision::NoOp => return Ok(None),
            }
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
