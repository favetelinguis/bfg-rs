use crate::app::{ui, App, AppReturn};
use crate::inputs::events::Events;
use crate::inputs::InputEvent;
use crate::io::IoEvent;
use eyre::Result;
use std::io::stdout;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tui::backend::CrosstermBackend;
use tui::Terminal;

pub mod app;
pub mod inputs;
pub mod io;

pub async fn start_ui(app: &Arc<RwLock<App>>) -> Result<()> {
    // Setup Crossterm backend
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let mut events = Events::new(tick_rate);

    // Trigger state change from Init to Initialized
    {
        let mut app = app.write().await;
        app.dispatch(IoEvent::Initialize).await;
    } // lock goes out of scope here

    // Render loop
    loop {
        // Draw UI
        {
            let app = app.read().await;
            terminal.draw(|rect| ui::draw(rect, &app))?;
        }
        // Handle user input
        let result = match events.next().await {
            InputEvent::Input(key) => {
                let mut app = app.write().await;
                app.do_action(key).await
            }
            InputEvent::Tick => {
                let mut app = app.write().await;
                app.update_on_tick()
            }
        };
        if result == AppReturn::Exit {
            events.close();
            break;
        }
    }

    // Restore terminal when terminating
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
