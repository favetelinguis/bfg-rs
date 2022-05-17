use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, Tabs};

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum MenuItem {
    Home,
    Logs,
    Help,
}

const MENU_TITLES: [&str; 3] = ["Home", "Logs", "?HELP"];

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> Self {
        match input {
            MenuItem::Home => 0,
            MenuItem::Logs => 1,
            MenuItem::Help => 2,
        }
    }
}

pub fn draw_menu<'a>(selected: usize) -> Tabs<'a> {
    let menu = MENU_TITLES
        .iter()
        .map(|t| {
            let (first, rest) = t.split_at(1);
            Spans::from(vec![
                Span::styled(
                    first,
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::UNDERLINED),
                ),
                Span::styled(rest, Style::default().fg(Color::White)),
            ])
        })
        .collect();
    Tabs::new(menu)
        .select(selected)
        .block(Block::default().title("Menu").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow))
        .divider(Span::raw("|"))
}
