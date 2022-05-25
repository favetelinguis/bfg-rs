use crate::inputs::key::Key;
use crate::inputs::InputEvent;
use crossterm::event;
use log::error;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub struct Events {
    rx: tokio::sync::mpsc::Receiver<InputEvent>,
    // Need to be kept around to prevent disposing the sender side
    _tx: tokio::sync::mpsc::Sender<InputEvent>,
    stop_capture: Arc<AtomicBool>,
}

impl Events {
    pub fn new(tick_rate: Duration) -> Self {
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let stop_capture = Arc::new(AtomicBool::new(false));

        let event_tx = tx.clone(); // the thread::spawn own event_tx
        let event_stop_capture = stop_capture.clone();
        tokio::spawn(async move {
            loop {
                // poll for tick rate duration if no event send tick event
                if event::poll(tick_rate).unwrap() {
                    if let event::Event::Key(key) = event::read().unwrap() {
                        let key = Key::from(key);
                        if let Err(err) = event_tx.send(InputEvent::Input(key)).await {
                            error!("Ooops!, {}", err)
                        }
                    }
                }
                if let Err(err) = event_tx.send(InputEvent::Tick).await {
                    error!("Ooops!, {}", err)
                }
                if event_stop_capture.load(Ordering::Relaxed) {
                    break;
                }
            }
        });
        Events {
            rx,
            _tx: tx,
            stop_capture,
        }
    }

    /// Attempts to read an event.
    pub async fn next(&mut self) -> InputEvent {
        self.rx.recv().await.unwrap_or(InputEvent::Tick)
    }

    /// Close
    pub fn close(&mut self) {
        self.stop_capture.store(true, Ordering::Relaxed)
    }
}
