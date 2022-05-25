use crate::inputs::key::Key;
use crate::ui::menu::MenuItem;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Display;
use std::slice::Iter;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Action {
    Quit,
    Sleep,
    MenuChange(MenuItem),
}

impl Action {
    /// All available actions
    pub fn iterator() -> Iter<'static, Action> {
        static ACTIONS: [Action; 5] = [
            Action::Quit,
            Action::Sleep,
            Action::MenuChange(MenuItem::Home),
            Action::MenuChange(MenuItem::Logs),
            Action::MenuChange(MenuItem::Help),
        ];
        ACTIONS.iter()
    }

    // List of key associated to action
    pub fn keys(&self) -> &[Key] {
        match self {
            Action::Quit => &[Key::Ctrl('c'), Key::Char('q')],
            Action::Sleep => &[Key::Char('s')],
            Action::MenuChange(MenuItem::Home) => &[Key::Char('h')],
            Action::MenuChange(MenuItem::Logs) => &[Key::Char('l')],
            Action::MenuChange(MenuItem::Help) => &[Key::Char('?')],
        }
    }
}

/// The application should have some contextual actions
#[derive(Default, Debug, Clone)]
pub struct Actions(Vec<Action>);

impl Actions {
    /// Given a key, find the corresponding action
    pub fn find(&self, key: Key) -> Option<&Action> {
        Action::iterator()
            .filter(|action| self.0.contains(action))
            .find(|action| action.keys().contains(&key))
    }

    /// Get contextual actions.
    /// (just for building a help view)
    pub fn actions(&self) -> &[Action] {
        self.0.as_slice()
    }
}

/// Could display a user friendly short description of action
impl Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let str = match self {
            Action::Quit => "Quit",
            Action::Sleep => "Sleep",
            Action::MenuChange(MenuItem::Home) => "Menu|Home",
            Action::MenuChange(MenuItem::Logs) => "Menu|Logs",
            Action::MenuChange(MenuItem::Help) => "Menu|Help",
        };
        write!(f, "{}", str)
    }
}

impl From<Vec<Action>> for Actions {
    /// Build contextual action
    ///
    /// # Panics
    ///
    /// If two actions have same key
    fn from(actions: Vec<Action>) -> Self {
        let mut map: HashMap<Key, Vec<Action>> = HashMap::new();
        for action in actions.iter() {
            for key in action.keys().iter() {
                match map.get_mut(key) {
                    Some(vec) => vec.push(*action),
                    None => {
                        map.insert(*key, vec![*action]);
                    }
                }
            }
        }
        let errors = map
            .iter()
            .filter(|(_, actions)| actions.len() > 1) // if at least two actions share same shortcut
            .map(|(key, actions)| {
                let actions = actions
                    .iter()
                    .map(Action::to_string)
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("Conflict key {:?} with actions {}", key, actions)
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            panic!("{}", errors.join("; "))
        }

        Self(actions)
    }
}
