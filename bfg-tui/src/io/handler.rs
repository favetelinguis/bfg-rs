use crate::io::IoEvent;
use crate::App;
use eyre::Result;
use log::{error, info};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

pub struct IoAsyncHandler {
    app: Arc<RwLock<App>>,
}

impl IoAsyncHandler {
    pub fn new(app: Arc<RwLock<App>>) -> Self {
        Self { app }
    }

    pub async fn handle_io_event(&mut self, io_event: IoEvent) {
        let result = match io_event {
            IoEvent::Initialize => self.do_initialize().await,
            IoEvent::Sleep(duration) => self.do_sleep(duration).await,
        };

        if let Err(err) = result {
            error!("Oops, something bad happened {:?}", err);
        }

        let mut app = self.app.write().await;
        app.loaded();
    }

    async fn do_initialize(&mut self) -> Result<()> {
        info!("Initialize the application");
        let mut app = self.app.write().await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        app.initialized();
        info!("Application initialized");

        Ok(())
    }

    async fn do_sleep(&mut self, duration: Duration) -> Result<()> {
        info!("Go sleeping for {:?}", duration);
        tokio::time::sleep(duration).await;
        info!("Wake up!");
        let mut app = self.app.write().await;
        app.sleeped();

        Ok(())
    }
}
