//!
//! The main entrypoint for the tui of BFG
//!
use log::{debug, error, info, trace, warn};
use std::env;
use std::error::Error;
use std::io::stdout;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::Terminal;

// TODO I need to isolate the use of bfg-ig specific stuff in a App, then have an app impl for each
// Broker, use a Broker trait in bfg-app module to hold all services and then send that App into
// the gui?
#[cfg(feature = "bfg-ig")]
pub use bfg_ig::{IGAccountProvider, IGConfig};

use bfg_core::account::AccountService;

mod event;
use event::{Event, Events};
mod app;
use app::App;
mod ui;

fn main() -> Result<(), Box<Error>> {
    stderrlog::new().quiet(false).verbosity(4).init().unwrap(); // Init logger impl

    let usr = env::var("IG_USER")?;
    let pwd = env::var("IG_PWD")?;
    let api_key = env::var("IG_API_KEY")?;
    let account = env::var("IG_ACCOUNT")?;
    let config = IGConfig::new(usr, pwd, api_key, account);
    let ig_account_provider = IGAccountProvider::new(config);
    let account_service = AccountService::new(Box::new(ig_account_provider));

    let events = Events::new();

    // Setup the terminal
    let stdout = stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor().unwrap();

    // TODO I should make up an app
    // TODO i should call a run metod in lib that has all program logic to test
    let mut app = App::new("BFG the app", account_service);
    loop {
        ui::draw(&mut terminal, &app)?;
        match events.next().unwrap() {
            Event::Input(key) => match key {
                Key::Char(c) => {
                    app.on_key(c);
                }
                // TODO use other from Key enum
                _ => {}
            },
            Event::Tick => {
                app.on_tick();
            }
        }
        if app.should_quit {
            break;
        }
    }
    Ok(())
}
