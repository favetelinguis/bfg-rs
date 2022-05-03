use crate::app::actions::{Action, Actions};
use crate::app::state::AppState;
use crate::inputs::key::Key;
use crate::io::IoEvent;
use crate::ui::menu::MenuItem;
use crate::ui::menu::MenuItem::{Help, Logs};
use log::{debug, error, warn};
use std::sync::mpsc;

pub mod actions;
pub mod state;
pub mod ui;

#[derive(Debug, PartialEq, Eq)]
pub enum AppReturn {
    Exit,
    Continue,
}

pub struct App {
    io_tx: mpsc::SyncSender<IoEvent>, // App can send IO events as result of user input
    is_loading: bool,
    actions: Actions,
    state: AppState,
    active_menu_item: MenuItem,
}

impl App {
    #[allow(clippy::new_without_default)]
    pub fn new(io_tx: mpsc::SyncSender<IoEvent>) -> Self {
        let actions = vec![Action::Quit].into();
        let state = AppState::default();
        let is_loading = false;
        let active_menu_item = MenuItem::Home;
        Self {
            actions,
            state,
            is_loading,
            io_tx,
            active_menu_item,
        }
    }

    pub fn dispatch(&mut self, action: IoEvent) {
        // is_loading will be set to false again after the action has finised in io/handler.rs
        self.is_loading = true;
        if let Err(e) = self.io_tx.send(action) {
            self.is_loading = false;
            error!("Error from dispatch {}", e);
        };
    }

    /// Handle a user action
    pub fn do_action(&mut self, key: Key) -> AppReturn {
        if let Some(action) = self.actions.find(key) {
            debug!("Run action [{:?}]", action);
            match action {
                Action::Quit => AppReturn::Exit,
                Action::Sleep => {
                    if let Some(duration) = self.state.duration().cloned() {
                        // Sleep is an I/O action we dispatch on the IO channel that is run on another thread
                        self.dispatch(IoEvent::Sleep(duration));
                    }
                    AppReturn::Continue
                }
                Action::MenuChange(menu_item) => {
                    self.active_menu_item = menu_item.clone();
                    AppReturn::Continue
                }
            }
        } else {
            warn!("No action associated to {:?}", key);
            AppReturn::Continue
        }
    }

    /// We could update the app or dispatch event on tick
    pub fn update_on_tick(&mut self) -> AppReturn {
        self.state.incr_tick();
        AppReturn::Continue
    }

    pub fn state(&self) -> &AppState {
        &self.state
    }

    pub fn active_menu_item(&self) -> &MenuItem {
        &self.active_menu_item
    }

    pub fn actions(&self) -> &Actions {
        &self.actions
    }

    pub fn initialized(&mut self) {
        self.actions = vec![
            Action::Quit,
            Action::Sleep,
            Action::MenuChange(MenuItem::Home),
            Action::MenuChange(Logs),
            Action::MenuChange(Help),
        ]
        .into();
        self.state = AppState::initialized()
    }

    pub fn loaded(&mut self) {
        self.is_loading = false;
    }

    pub fn sleeped(&mut self) {
        self.state.incr_sleep();
    }
}
