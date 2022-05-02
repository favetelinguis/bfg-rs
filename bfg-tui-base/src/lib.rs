use std::io::stdout;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use eyre::Result;
use tui::backend::CrosstermBackend;
use tui::Terminal;
use crate::app::{App, AppReturn, ui};
use crate::inputs::events::Events;
use crate::inputs::InputEvent;
use crate::io::IoEvent;

pub mod app;
pub mod inputs;
pub mod io;

#[allow(unreachable_code)]
pub fn start_ui(app: &Arc<RwLock<App>>) -> Result<()> {
    // Setup Crossterm backend
    let stdout = stdout();
    crossterm::terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;
    terminal.hide_cursor()?;

    let tick_rate = Duration::from_millis(200);
    let events = Events::new(tick_rate);

    // Trigger state change from Init to Initialized
    {
        let mut app = app.write().unwrap();
        app.dispatch(IoEvent::Initialize);
    } // lock goes out of scope here

    // Render loop
    loop {
        let mut app = app.write().unwrap();
        // Draw UI
        terminal.draw(|rect| ui::draw(rect, &app))?;

        // Handle user input
        let result = match events.next()? {
            InputEvent::Input(key) => app.do_action(key),
            InputEvent::Tick => app.update_on_tick(),
        };
        if result == AppReturn::Exit {
            break;
        }
    }

    // Restore terminal when terminating
    terminal.clear()?;
    terminal.show_cursor()?;
    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}