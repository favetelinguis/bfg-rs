use crate::realtime::models::{Mode, TlcpRequest, TlcpResponse};
use crate::realtime::notifications::{
    parse_account_update, parse_market_update, parse_trade_update,
};
use crate::{BrokerageError, RealtimeEvent, RestDetails};
use futures_util::{SinkExt, StreamExt};
use http::HeaderValue;
use log::{error, info, warn};
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc::{channel, Sender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use url::Url;

pub mod models;
pub mod notifications;

#[derive(Debug)]
struct Tlcp(TlcpRequest);

pub struct IgStreamClient {}

impl IgStreamClient {
    pub async fn start(
        rest_details: RestDetails,
        tx: Sender<RealtimeEvent>,
    ) -> Result<(), BrokerageError> {
        let url_raw = rest_details.url.clone();
        let stream_details = Arc::new(Mutex::new(None));
        let mut url = Url::parse(&format!("{}/lightstreamer", url_raw)).unwrap();
        url.set_scheme("wss").unwrap();
        let mut request = url.into_client_request().unwrap();
        request.headers_mut().insert(
            "Sec-WebSocket-Protocol",
            HeaderValue::from_str("TLCP-2.1.0.lightstreamer.com").unwrap(),
        );
        let (ws_stream, _) = connect_async(request).await.expect("Can't connect");
        let (mut write, read) = ws_stream.split();
        let details = Arc::clone(&stream_details);
        let (control_tx, mut control_rx) = channel(22);
        let cloned_tx = control_tx.clone();

        tokio::spawn(async move {
            info!("Starting IG Control");
            while let Some(Tlcp(request)) = control_rx.recv().await {
                write.send(request.into()).await.unwrap();
            }
        });

        tokio::spawn(async move {
            read.for_each(|m| async {
                let data = m.unwrap().into_text().unwrap();
                let messages = data.split_terminator("\r\n");
                for m in messages {
                    // info!("WS: {:?}", m);
                    match TlcpResponse::from_str(m).unwrap() {
                        TlcpResponse::SYNC { .. } => {} // TODO check dock for how to use
                        TlcpResponse::PROBE => tx
                            .send(RealtimeEvent::StreamStatus("ALIVE".to_string()))
                            .await
                            .unwrap(), // Save now() time in state and spawn a new task that will post a message 2 seconds after last probe should come then check
                        TlcpResponse::CONOK { session_id, .. } => {
                            // TODO if this is a session rebind we should not re subscribe
                            tx.send(RealtimeEvent::StreamStatus("CONNECTED".to_string()))
                                .await
                                .unwrap();
                            cloned_tx
                                .send(Tlcp(TlcpRequest::Subscribe {
                                    item: "MARKET:IX.D.DAX.IFMM.IP".to_string(),
                                    fields: vec![
                                        "BID".to_string(),
                                        "OFFER".to_string(),
                                        "MARKET_DELAY".to_string(),
                                        "MARKET_STATE".to_string(),
                                        "UPDATE_TIME".to_string(),
                                    ],
                                    mode: Mode::Merge,
                                    req_id: 1,
                                    sub_id: 1,
                                    session_id: session_id.clone(),
                                }))
                                .await
                                .unwrap(); // MARKET
                            cloned_tx
                                .send(Tlcp(TlcpRequest::Subscribe {
                                    item: "ACCOUNT:ZQVBB".to_string(),
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
                                    req_id: 2,
                                    sub_id: 2,
                                    session_id: session_id.clone(),
                                }))
                                .await
                                .unwrap(); // ACCOUNT
                            cloned_tx
                                .send(Tlcp(TlcpRequest::Subscribe {
                                    item: "TRADE:ZQVBB".to_string(),
                                    fields: vec![
                                        "CONFIRMS".to_string(),
                                        "OPU".to_string(),
                                        "WOU".to_string(),
                                    ],
                                    mode: Mode::Distinct,
                                    req_id: 3,
                                    sub_id: 3,
                                    session_id: session_id.clone(),
                                }))
                                .await
                                .unwrap(); // TRADE

                            let mut d = details.lock().unwrap();
                            *d = Some(session_id);
                        }
                        TlcpResponse::U {
                            ref fields_values,
                            subscription_id,
                            ..
                        } => match subscription_id {
                            1 => tx
                                .send(RealtimeEvent::MarketEvent(parse_market_update(
                                    fields_values,
                                )))
                                .await
                                .unwrap(),
                            2 => tx
                                .send(RealtimeEvent::AccountEvent(parse_account_update(
                                    fields_values,
                                )))
                                .await
                                .unwrap(),
                            3 => {
                                // Simpler since distinct mode and only one value so no need to split on |
                                let (confirms, open_position_updates, working_orders_updates) =
                                    parse_trade_update(fields_values);
                                // Should be used by trading system
                                if let Some(confs) = confirms {
                                    tx.send(RealtimeEvent::TradeConfirmation(confs))
                                        .await
                                        .unwrap()
                                }
                                // Should be used by GUI
                                if let Some(opu) = open_position_updates {
                                    tx.send(RealtimeEvent::AccountPositionUpdate(opu))
                                        .await
                                        .unwrap()
                                }
                                if let Some(wou) = working_orders_updates {
                                    tx.send(RealtimeEvent::WorkingOrderUpdate(wou))
                                        .await
                                        .unwrap()
                                }
                            }
                            msg => error!("Unsupported update: {:?}", msg),
                        },
                        TlcpResponse::LOOP { .. } => panic!("Session rebinding not supported"),
                        msg => error!("Unhandled WS event: {:?}", msg),
                    }
                }
            })
            .await;
        });

        // Setup session
        let user = rest_details.account.clone();
        let account_token = rest_details.xst.clone();
        let client_token = rest_details.cst.clone();
        control_tx
            .send(Tlcp(TlcpRequest::CreateSession {
                user,
                account_token,
                client_token,
            }))
            .await
            .map_err(|_| BrokerageError::CoreBrokerageError)?;
        Ok(())
    }
}
