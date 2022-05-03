use crate::io::IoEvent;
use crate::App;
use eyre::Result;
use log::{error, info};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

pub struct IoHandler {
    app: Arc<RwLock<App>>,
}

impl IoHandler {
    pub fn new(app: Arc<RwLock<App>>) -> Self {
        Self { app }
    }

    pub fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize(),
            IoEvent::Sleep(duration) => self.do_sleep(duration),
        };

        if let Err(err) = result {
            error!("Oops, something bad happened {:?}", err);
        }

        let mut app = self.app.write().unwrap();
        app.loaded();
    }

    fn do_initialize(&mut self) -> Result<()> {
        info!("Initialize the application");
        let mut app = self.app.write().unwrap();
        thread::sleep(Duration::from_secs(1));
        app.initialized(); // we can update the app state
        info!("Application initialized");

        Ok(())
    }

    fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        info!("Go sleeping for {:?}", duration);
        thread::sleep(duration);
        info!("Wake up!");
        let mut app = self.app.write().unwrap();
        app.sleeped();

        Ok(())
    }
}
