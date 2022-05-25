use bfg_core::models::{AccountUpdate, MarketUpdate, SystemState, TradeConfirmation, TradeUpdate};
use bfg_core::{step_system, BfgEvent};
use bfg_tui_base::app::App;
use bfg_tui_base::io::handler::IoAsyncHandler;
use bfg_tui_base::io::IoEvent;
use bfg_tui_base::start_ui;
use dotenvy::dotenv;
use eyre::Result;
use ig_brokerage_adapter::realtime::models::{PositionStatus, TradeConfirmationUpdate};
use ig_brokerage_adapter::{ConnectionDetails, IgBrokerageApi, RealtimeEvent};
use log::{error, info, warn, LevelFilter};
use std::borrow::Borrow;
use std::collections::LinkedList;
use std::str::FromStr;
use std::sync::Arc;
use tokio::select;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    // Channel for BFG
    let (sync_bfg_tx, mut sync_bfg_rx) = tokio::sync::mpsc::channel::<RealtimeEvent>(100);
    // Channel for IoEvent
    let (sync_io_tx, mut sync_io_rx) = tokio::sync::mpsc::channel::<IoEvent>(100);

    // Create app
    let gui_state = Arc::new(tokio::sync::RwLock::new(App::new(sync_io_tx)));
    let app_ui = Arc::clone(&gui_state);
    let app_bfg = Arc::clone(&gui_state);

    // Configure log
    tui_logger::init_logger(LevelFilter::Info).unwrap();
    tui_logger::set_default_level(LevelFilter::Info);
    tui_logger::set_log_file("event_log.log").unwrap();

    // Handle I/O from GUI
    let io = tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(gui_state);
        while let Some(io_event) = sync_io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    // Run Trading system
    let trade_system = tokio::spawn(async move {
        let handler = IgBrokerageApi::new(ConnectionDetails::from_env(), sync_bfg_tx).await;
        let mut bfg_state = SystemState::Setup;
        while let Some(action) = sync_bfg_rx.recv().await {
            match action {
                RealtimeEvent::MarketEvent(data) => {
                    let mut a = app_bfg.write().await;
                    let next_market = MarketUpdate {
                        update_time: data.update_time.or_else(|| a.market.update_time.clone()),
                        market_delay: data.market_delay.or(a.market.market_delay),
                        market_state: data.market_state.or_else(|| a.market.market_state.clone()),
                        offer: data.offer.or(a.market.offer),
                        bid: data.bid.or(a.market.bid),
                    };

                    let (next_state, decisions) =
                        step_system(bfg_state.clone(), BfgEvent::Market(next_market.clone()));
                    bfg_state = next_state;

                    let mut events = LinkedList::new();
                    for d in decisions {
                        events.push_back(handler.execute_decision(d).await.unwrap());
                    }

                    // // A system step could result in more events
                    while let Some(maybe_event) = events.pop_front() {
                        if let Some(event) = maybe_event {
                            let (next_state, more_decisions) =
                                step_system(bfg_state.clone(), event.clone());
                            bfg_state = next_state;

                            for d in more_decisions {
                                events.push_back(handler.execute_decision(d).await.unwrap());
                            }
                        }
                    }

                    a.market = next_market;
                    a.system = bfg_state.clone();
                }
                RealtimeEvent::AccountEvent(data) => {
                    let mut a = app_bfg.write().await;
                    a.account = AccountUpdate {
                        account: data.account.or_else(|| a.account.account.clone()),
                        pnl: data.pnl.or(a.account.pnl),
                        deposit: data.deposit.or(a.account.deposit),
                        available_cash: data.available_cash.or(a.account.available_cash),
                        pnl_lr: data.pnl_lr.or(a.account.pnl_lr),
                        pnl_nlr: data.pnl_nlr.or(a.account.pnl_nlr),
                        funds: data.funds.or(a.account.funds),
                        margin: data.margin.or(a.account.margin),
                        margin_lr: data.margin_lr.or(a.account.margin_lr),
                        margin_nlr: data.margin_lr.or(a.account.margin_nlr),
                        available_to_deal: data.available_to_deal.or(a.account.available_to_deal),
                        equity: data.equity.or(a.account.equity),
                        equity_used: data.equity_used.or(a.account.equity_used),
                    };
                    a.system = bfg_state.clone();
                }
                RealtimeEvent::TradeConfirmation(data) => {
                    let (next_state, decisions) = step_system(
                        bfg_state.clone(),
                        BfgEvent::TradeConfirmation(TradeConfirmation {
                            deal_status: data.deal_status.into(),
                            status: data.status.map(|i: PositionStatus| i.into()),
                            deal_id: data.deal_id,
                            deal_reference: FromStr::from_str(data.deal_reference.as_str())
                                .expect("We should only use deal references that are expected"),
                            reason: data.reason,
                        }),
                    );
                    bfg_state = next_state;

                    let mut events = LinkedList::new();
                    for d in decisions {
                        events.push_back(handler.execute_decision(d).await.unwrap());
                    }
                    // // A system step could result in more events
                    while let Some(maybe_event) = events.pop_front() {
                        if let Some(event) = maybe_event {
                            let (next_state, more_decisions) =
                                step_system(bfg_state.clone(), event.clone());
                            bfg_state = next_state;

                            for d in more_decisions {
                                events.push_back(handler.execute_decision(d).await.unwrap());
                            }
                        }
                    }

                    let mut a = app_bfg.write().await;
                    a.system = bfg_state.clone();
                }
                RealtimeEvent::AccountPositionUpdate(data) => {
                    let (next_state, decisions) = step_system(
                        bfg_state.clone(),
                        BfgEvent::Trade(TradeUpdate {
                            deal_status: data.deal_status.clone().into(),
                            status: data.status.clone().into(),
                            deal_id: data.deal_id.clone(),
                            deal_reference: FromStr::from_str(data.deal_reference.as_str())
                                .expect("We should only use deal references that are expected"),
                        }),
                    );
                    bfg_state = next_state;

                    let mut events = LinkedList::new();
                    for d in decisions {
                        events.push_back(handler.execute_decision(d).await.unwrap());
                    }
                    // // A system step could result in more events
                    while let Some(maybe_event) = events.pop_front() {
                        if let Some(event) = maybe_event {
                            let (next_state, more_decisions) =
                                step_system(bfg_state.clone(), event.clone());
                            bfg_state = next_state;

                            for d in more_decisions {
                                events.push_back(handler.execute_decision(d).await.unwrap());
                            }
                        }
                    }

                    let mut a = app_bfg.write().await;
                    a.trade = Some(data); // Also update UI
                    a.system = bfg_state.clone();
                }
                RealtimeEvent::WorkingOrderUpdate(data) => {
                    error!("WOU never seen this before: {:?}", data);
                }
                RealtimeEvent::StreamStatus(data) => {
                    let mut a = app_bfg.write().await;
                    a.stream_status = data;
                }
                msg => info!("{:?}", msg),
            }
        }
    });

    // Start UI
    let ui = start_ui(&app_ui);

    select! {
        Ok(_) = trade_system => println!("COMPLETED Trade System"),
        Ok(_) = io => println!("COMPLETED IO"),
        Ok(_) = ui => println!("COMPLETED UI"),
    }

    Ok(())
}
