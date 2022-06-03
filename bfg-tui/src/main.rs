use std::env::home_dir;
use bfg_tui::app::App;
use bfg_tui::io::handler::IoAsyncHandler;
use bfg_tui::io::IoEvent;
use bfg_tui::start_ui;
use dotenvy::dotenv;
use eyre::Result;
use std::sync::Arc;
use chrono::Utc;
use log::LevelFilter;
use tokio::select;
use bfg_ig::{IgEvent, spawn_bfg};
use bfg_ig::models::ConnectionDetails;
use crate::config::BfgConfig;

mod config;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Configure log
    let todays_file = Utc::now().date().to_string();
    let mut path = home_dir().expect("always have a home");
    path.push(format!("bfg/demo/{}.log", todays_file));
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(LevelFilter::Debug);
    tui_logger::set_log_file(path.to_str().expect("The path to log file in bfg/demo fail")).unwrap();

    // Channel for IgEvent
    let (ig_tx, mut tui_rx) = tokio::sync::mpsc::channel::<IgEvent>(10);
    // Channel for IoEvent
    let (io_tx, mut io_rx) = tokio::sync::mpsc::channel::<IoEvent>(10);

    // Create shared state
    let gui_state = Arc::new(tokio::sync::RwLock::new(App::new(io_tx)));
    let copy_gui_state_io = Arc::clone(&gui_state);
    let copy_gui_state_system= Arc::clone(&gui_state);

    // Handle I/O from GUI
    let io = tokio::spawn(async move {
        let mut handler = IoAsyncHandler::new(copy_gui_state_io);
        while let Some(io_event) = io_rx.recv().await {
            handler.handle_io_event(io_event).await;
        }
    });

    // Run Trading system
    let trade_system = tokio::spawn(async move {
        let config = BfgConfig::new().await;
        spawn_bfg(config.connection_details, config.epics, ig_tx);
        while let Some(event) = tui_rx.recv().await {
            let mut gui = copy_gui_state_system.write().await;
            match event {
                IgEvent::MarketView(epic, current_market) => gui.markets.update(epic, current_market),
                IgEvent::SystemView(epic, current_system) => gui.systems.update(epic, current_system),
                IgEvent::AccountView(current_account) => gui.account = current_account,
                IgEvent::TradesResultsView(result) => gui.add_trade_result(result),
                IgEvent::ConnectionView(current_connection) => gui.connection_information = current_connection,
            }
        }
    });

    // Start UI
    let ui = start_ui(&gui_state);

    select! {
        Ok(_) = trade_system => println!("COMPLETED Trade System"),
        Ok(_) = io => println!("COMPLETED IO"),
        Ok(_) = ui => println!("COMPLETED UI"),
    }

    Ok(())
}
