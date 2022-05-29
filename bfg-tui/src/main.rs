use bfg_tui_base::app::App;
use bfg_tui_base::io::handler::IoAsyncHandler;
use bfg_tui_base::io::IoEvent;
use bfg_tui_base::start_ui;
use dotenvy::dotenv;
use eyre::Result;
use std::sync::Arc;
use log::LevelFilter;
use tokio::select;
use bfg_ig::{BfgIg, IgEvent};
use bfg_ig::models::ConnectionDetails;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    // Configure log
    tui_logger::init_logger(LevelFilter::Info).unwrap();
    tui_logger::set_default_level(LevelFilter::Info);
    tui_logger::set_log_file("event_log.log").unwrap();

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
        // TODO BfgIg should prob just be a function
        let _ = BfgIg::new(ConnectionDetails::from_env(), ig_tx);
        while let Some(event) = tui_rx.recv().await {
            let mut gui = copy_gui_state_system.write().await;
            match event {
                IgEvent::MarketView(current_market) => gui.market = current_market,
                IgEvent::SystemView(current_system) => gui.system = current_system,
                IgEvent::AccountView(current_account) => gui.account = current_account,
                IgEvent::TradesResultsView(current_results) => gui.results = current_results,
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
