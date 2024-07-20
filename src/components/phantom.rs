use super::Msg;
use tui_realm_stdlib::Phantom;
use tuirealm::{
    command::CmdResult,
    event::{Key, KeyEvent},
    Component, Event, MockComponent, NoUserEvent,
};

#[derive(MockComponent)]
pub struct PhantomComponent {
    component: Phantom,
}

impl Default for PhantomComponent {
    fn default() -> Self {
        Self {
            component: Phantom::default(),
        }
    }
}

impl Component<Msg, NoUserEvent> for PhantomComponent {
    fn on(&mut self, ev: tuirealm::Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
