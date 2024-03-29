use crate::inputs::key::Key;

pub mod events;
pub mod key;

pub enum InputEvent {
    Input(Key),
    Tick,
}
