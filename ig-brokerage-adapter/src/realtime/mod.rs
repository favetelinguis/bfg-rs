use crate::realtime::models::{Mode, TlcpRequest, TlcpResponse};
use crate::realtime::notifications::{
    parse_account_update, parse_market_update, parse_trade_update,
};
use crate::{BrokerageError, ConnectionDetails, RealtimeEvent, RestDetails, SessionState};
use futures_util::{SinkExt, StreamExt};
use http::HeaderValue;
use log::{error, info, warn};
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::mpsc::{channel, Sender};
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use url::Url;
use bfg_core::decider::MarketInfo;
use crate::realtime::subscription_manager::SubscriptionManager;

pub mod models;
pub mod notifications;
mod subscription_manager;

#[derive(Debug)]
struct Tlcp(TlcpRequest);

pub struct IgStreamClient {
    session: Arc<Mutex<SessionState>>,
    tx: Sender<RealtimeEvent>,
}

impl IgStreamClient {
    pub fn new(session: Arc<Mutex<SessionState>>, tx: Sender<RealtimeEvent>) -> Self {
        Self { session, tx }
    }

    pub async fn start(&self, connection_details: ConnectionDetails, epics: Vec<String>) -> Result<(), BrokerageError> {
        let SessionState {
            ref xst,
            ref cst,
            ref account,
            ref lightstreamer_endpoint,
        } = &*self.session.lock().await;
        let mut url = Url::parse(&format!("{}/lightstreamer", lightstreamer_endpoint)).unwrap();
        url.set_scheme("wss").unwrap();
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("TLCP-2.1.0.lightstreamer.com").unwrap(),
        );
        let (ws_stream, _) = connect_async(request).await.expect("Can't connect");
        let (mut write, mut read) = ws_stream.split();
        let (control_tx, mut control_rx) = channel(22);
        let cloned_ws_tx = control_tx.clone();
        let cloned_event_tx = self.tx.clone();

        tokio::spawn(async move {
            while let Some(Tlcp(request)) = control_rx.recv().await {
                write.send(request.into()).await.unwrap();
            }
        });

        tokio::spawn(async move {
            let mut subscription_manager = SubscriptionManager::new(epics);
            while let Some(Ok(m)) = read.next().await {
                let data = m.into_text().unwrap();
                let messages = data.split_terminator("\r\n");
                for m in messages {
                    // info!("WS: {:?}", m);
                    match TlcpResponse::from_str(m).unwrap() {
                        TlcpResponse::SYNC { .. } => {} // TODO check dock for how to use
                        TlcpResponse::PROBE => cloned_event_tx
                            .send(RealtimeEvent::StreamStatus("ALIVE".to_string()))
                            .await
                            .unwrap(), // Save now() time in state and spawn a new task that will post a message 2 seconds after last probe should come then check
                        TlcpResponse::CONOK { session_id, .. } => {
                            // TODO if this is a session rebind we should not re subscribe
                            cloned_event_tx
                                .send(RealtimeEvent::StreamStatus("CONNECTED".to_string()))
                                .await
                                .unwrap();
                            cloned_ws_tx
                                .send(Tlcp(TlcpRequest::Subscribe {
                                    item: format!("ACCOUNT:{}", connection_details.account.clone()),
                                    fields: vec![
                                        "PNL".to_string(),
                                        "DEPOSIT".to_string(),
                                        "AVAILABLE_CASH".to_string(),
                                        "PNL_LR".to_string(),
                                        "PNL_NLR".to_string(),
                                        "FUNDS".to_string(),
                                        "MARGIN".to_string(),
                                        "MARGIN_LR".to_string(),
                                        "MARGIN_NLR".to_string(),
                                        "AVAILABLE_TO_DEAL".to_string(),
                                        "EQUITY".to_string(),
                                        "EQUITY_USED".to_string(),
                                    ],
                                    mode: Mode::Merge,
                                    req_id: 1,
                                    sub_id: 1,
                                    session_id: session_id.clone(),
                                    snapshot: true,
                                }))
                                .await
                                .unwrap(); // ACCOUNT
                            cloned_ws_tx
                                .send(Tlcp(TlcpRequest::Subscribe {
                                    item: format!("TRADE:{}", connection_details.account.clone()),
                                    fields: vec![
                                        "CONFIRMS".to_string(),
                                        "OPU".to_string(),
                                        "WOU".to_string(),
                                    ],
                                    mode: Mode::Distinct,
                                    req_id: 2,
                                    sub_id: 2,
                                    session_id: session_id.clone(),
                                    snapshot: false,
                                }))
                                .await
                                .unwrap(); // TRADE
                            let mut market_stream_id = 3;
                            for sub_id in subscription_manager.get_subscription_id_range() {
                                cloned_ws_tx
                                    .send(Tlcp(TlcpRequest::Subscribe {
                                        item: format!("MARKET:{}", subscription_manager.get_epic_from_subscription_id(sub_id)),
                                        fields: vec![
                                            "BID".to_string(),
                                            "OFFER".to_string(),
                                            "MARKET_DELAY".to_string(),
                                            "MARKET_STATE".to_string(),
                                            "UPDATE_TIME".to_string(),
                                        ],
                                        mode: Mode::Merge,
                                        req_id: market_stream_id,
                                        sub_id: market_stream_id,
                                        session_id: session_id.clone(),
                                        snapshot: true
                                    }))
                                    .await
                                    .unwrap(); // MARKET
                                market_stream_id += 1;
                            }
                        }
                        TlcpResponse::U {
                            ref fields_values,
                            subscription_id,
                            ..
                        } => match subscription_id {
                            1 => cloned_event_tx
                                .send(RealtimeEvent::AccountEvent(parse_account_update(
                                    fields_values, connection_details.account.clone()
                                )))
                                .await
                                .unwrap(),
                            2 => {
                                // Simpler since distinct mode and only one value so no need to split on |
                                let (confirms, open_position_updates, working_orders_updates) =
                                    parse_trade_update(fields_values);
                                // Should be used by trading system
                                if let Some(confs) = confirms {
                                    // info!("CONFS {:?}", confs,);
                                    cloned_event_tx
                                        .send(RealtimeEvent::TradeConfirmation(confs))
                                        .await
                                        .unwrap()
                                }
                                // Should be used by GUI
                                if let Some(opu) = open_position_updates {
                                    // info!("OPU {:?}",opu);
                                    cloned_event_tx
                                        .send(RealtimeEvent::AccountPositionUpdate(opu))
                                        .await
                                        .unwrap()
                                }
                                if let Some(wou) = working_orders_updates {
                                    // info!("WOU {:?}", wou);
                                    cloned_event_tx
                                        .send(RealtimeEvent::WorkingOrderUpdate(wou))
                                        .await
                                        .unwrap()
                                }
                            }
                            id if id > 2 => cloned_event_tx
                                .send(RealtimeEvent::MarketEvent(parse_market_update(
                                    fields_values, subscription_manager.get_epic_from_subscription_id(id as usize)
                                )))
                                .await
                                .unwrap(),
                            id => warn!("Unsupported update: {:?}", id),
                        },
                        TlcpResponse::LOOP { .. } => panic!("Session rebinding not supported"),
                        msg => warn!("Unhandled WS event: {:?}", msg),
                    }
                }
            }
        });
        // Setup session
        control_tx
            .send(Tlcp(TlcpRequest::CreateSession {
                user: account.clone(),
                account_token: xst.clone(),
                client_token: cst.clone(),
            }))
            .await
            .map_err(|e| BrokerageError(e.to_string()))?;
        return Ok(());
    }
}
