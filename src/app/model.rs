use std::time::Duration;

use crate::components::{LadderComponent, MarketsComponent, PhantomComponent, StatusComponent};

use super::{Id, Msg};
use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    terminal::TerminalBridge,
    tui::layout::{Constraint, Direction, Layout},
    Application, EventListenerCfg, NoUserEvent, Sub, Update,
};

pub struct Model {
    /// Application
    pub app: Application<Id, Msg, NoUserEvent>,
    /// Indicates that the application must quit
    pub quit: bool,
    /// Tells whether to redraw interface
    pub redraw: bool,
    /// Used to draw to terminal
    pub terminal: TerminalBridge,
}

impl Model {
    pub fn view(&mut self) {
        assert!(self
            .terminal
            .raw_mut()
            .draw(|f| {
                let outer_layout = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(vec![Constraint::Percentage(99), Constraint::Length(3)])
                    .split(f.size());

                let inner_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints(vec![Constraint::Percentage(99), Constraint::Length(20)])
                    .split(outer_layout[0]);

                self.app.view(&Id::Ladder, f, inner_layout[0]);
                self.app.view(&Id::Markets, f, inner_layout[1]);
                self.app.view(&Id::Status, f, outer_layout[1]);
            })
            .is_ok());
    }

    fn init_app() -> Application<Id, Msg, NoUserEvent> {
        // TODO what is events and what is different from Msg?
        // Msg are handled in update where are events handled?
        // App with event listener, what is event listener?
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(20))
                .poll_timeout(Duration::from_millis(10))
                .tick_interval(Duration::from_secs(1)),
        );

        // Mount components
        assert!(app
            .mount(
                Id::Markets,
                Box::new(MarketsComponent::default()),
                Vec::default(),
            )
            .is_ok());

        assert!(app
            .mount(
                Id::Ladder,
                Box::new(LadderComponent::default()),
                Vec::default(),
            )
            .is_ok());

        assert!(app
            .mount(
                Id::Status,
                Box::new(StatusComponent::default()),
                Vec::default(),
            )
            .is_ok());

        assert!(app
            .mount(
                Id::Phantom,
                Box::new(PhantomComponent::default()),
                vec![Sub::new(
                    tuirealm::SubEventClause::Keyboard(KeyEvent {
                        code: Key::Esc,
                        modifiers: KeyModifiers::NONE,
                    }),
                    tuirealm::SubClause::Always
                )],
            )
            .is_ok());
        // Set active component
        assert!(app.active(&Id::Markets).is_ok());

        return app;
    }
}

impl Default for Model {
    fn default() -> Self {
        Self {
            app: Self::init_app(),
            quit: false,
            redraw: true,
            terminal: TerminalBridge::new().expect("Cannot initialize terminal"),
        }
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        if let Some(msg) = msg {
            // Set redraw
            self.redraw = true;
            match msg {
                Msg::AppClose => {
                    self.quit = true;
                    None
                }
                _ => None,
            }
        } else {
            None
        }
    }
}
