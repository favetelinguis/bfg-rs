//! Main struct is App which holds all GUI state in the application.
use bfg_core::account::AccountService;
use log::{debug, error, info, trace, warn};

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
}

pub struct App<'a> {
    account_service: AccountService,
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, account_service: AccountService) -> App<'a> {
        App {
            account_service,
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Account (a)", "Tab1"]),
        }
    }

    pub fn on_key(&mut self, c: char) {
        debug!("Key pressed: {}", c);
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_tick(&mut self) {
        return;
    }
}
