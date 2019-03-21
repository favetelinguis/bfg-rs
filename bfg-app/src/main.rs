//!
//! This crate covers information about how things is.
//!
use log::{debug, error, info, trace, warn};
use std::io::{stdin, stdout, Error, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, Widget};
use tui::Terminal;

fn main() {
    stderrlog::new().quiet(false).verbosity(4).init().unwrap(); // Init logger impl
    let stdout = stdout().into_raw_mode().unwrap();
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();

    // TODO check events.rs and implement my own event handler for tick events and input events
    // TODO make more stuff from the termion_demo.rs

    trace!("trace message");
    debug!("debug message");
    info!("info message");
    warn!("warn message");
    error!("error message");

    terminal.draw(|mut f| {
        let size = f.size();
        Block::default()
            .title("Block")
            .borders(Borders::ALL)
            .render(&mut f, size);
    });
}
