use super::Msg;
use tui_realm_stdlib::Container;
use tuirealm::{command::CmdResult, props::Alignment, Component, MockComponent, NoUserEvent};

#[derive(MockComponent)]
pub struct LadderComponent {
    component: Container,
}

impl Default for LadderComponent {
    fn default() -> Self {
        Self {
            component: Container::default()
                .background(tuirealm::props::Color::Green)
                .foreground(tuirealm::props::Color::Yellow)
                .title("Ladder", Alignment::Center),
        }
    }
}

impl Component<Msg, NoUserEvent> for LadderComponent {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
