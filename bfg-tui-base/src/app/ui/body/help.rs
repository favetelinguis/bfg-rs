use crate::App;
use tui::backend::Backend;
use tui::layout::{Constraint, Rect};
use tui::style::{Color, Style};
use tui::text::Span;
use tui::widgets::{Block, BorderType, Borders, Cell, Row, Table};
use tui::Frame;

pub fn draw_help<B>(rect: &mut Frame<B>, chunk: Rect, app: &App)
where
    B: Backend,
{
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    for action in app.actions().actions().iter() {
        let mut first = true;
        for key in action.keys() {
            let help = if first {
                first = false;
                action.to_string()
            } else {
                String::from("")
            };
            let row = Row::new(vec![
                Cell::from(Span::styled(key.to_string(), key_style)),
                Cell::from(Span::styled(help, help_style)),
            ]);
            rows.push(row)
        }
    }

    let table = Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Help"),
        )
        .widths(&[Constraint::Length(11), Constraint::Min(20)])
        .column_spacing(1);

    rect.render_widget(table, chunk);
}
