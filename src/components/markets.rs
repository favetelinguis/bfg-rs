use super::Msg;
use tui_realm_stdlib::Container;
use tuirealm::{command::CmdResult, props::Alignment, Component, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct MarketsComponent {
    component: Container,
}

impl Default for MarketsComponent {
    fn default() -> Self {
        Self {
            component: Container::default()
                .background(tuirealm::props::Color::Green)
                .foreground(tuirealm::props::Color::Yellow)
                .title("All markets", Alignment::Center),
        }
    }
}

impl Component<Msg, NoUserEvent> for MarketsComponent {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
