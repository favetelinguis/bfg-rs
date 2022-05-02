use std::sync::{Arc, mpsc, RwLock};
use std::thread;
use eyre::Result;
use log::LevelFilter;
use bfg_tui_base::app::App;
use bfg_tui_base::io::handler::IoHandler;
use bfg_tui_base::io::IoEvent;
use bfg_tui_base::start_ui;

fn main() -> Result<()> {
    // Channel for IoEvent
    let (sync_io_tx, sync_io_rx) = mpsc::sync_channel::<IoEvent>(100);
    // Create app
    let app = Arc::new(RwLock::new(App::new(sync_io_tx)));
    let app_ui = Arc::clone(&app);

    // Configure log
    tui_logger::init_logger(LevelFilter::Debug).unwrap();
    tui_logger::set_default_level(log::LevelFilter::Debug);

    // Handle I/O
    thread::spawn(move || {
        let mut handler = IoHandler::new(app);
        while let Ok(io_event) = sync_io_rx.recv() {
            handler.handle_io_event(io_event);
        }
    });

    // Start UI
    start_ui(&app_ui)?;
    Ok(())
}
