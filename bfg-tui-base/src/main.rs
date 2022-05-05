use bfg_core::bfg_service_impl::BfgServiceImpl;
use bfg_core::domain::{State, SystemValues};
use bfg_tui_base::app::App;
use bfg_tui_base::io::handler::IoHandler;
use bfg_tui_base::io::IoEvent;
use bfg_tui_base::start_ui;
use eyre::Result;
use log::{debug, LevelFilter};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use bfg_core::ports::{Action, BfgService};
use bfg_tui_base::brokerage_dummy::DummyBrokerageApi;


fn main() -> Result<()> {
    // Channel for BFG
    let (sync_bfg_tx, sync_bfg_rx) = mpsc::sync_channel::<Action>(100);
    // Channel for IoEvent
    let (sync_io_tx, sync_io_rx) = mpsc::sync_channel::<IoEvent>(100);

    let brokerage = DummyBrokerageApi::new(sync_bfg_tx);
    let service = BfgServiceImpl {
        brokerage,
        state: State::new(SystemValues{system: 0, market: 0, trade: 0, account: 0}),
    };
    let service = Arc::new(RwLock::new(service));
    let service_app = Arc::clone(&service);

    // Create app
    let app = Arc::new(RwLock::new(App::new(sync_io_tx, service_app)));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Handle I/O - Nice to have here if we want right now only used for loading
    thread::spawn(move || {
        let mut handler = IoHandler::new(app);
        while let Ok(io_event) = sync_io_rx.recv() {
            handler.handle_io_event(io_event);
        }
    });

    // Run BFG
    thread::spawn(move || {
        while let Ok(event) = sync_bfg_rx.recv() {
            service.write().unwrap().publish_update_event(event);
        }
    });

    // Start UI
    start_ui(&app_ui)?;
    Ok(())
}
