use crate::app::state::AppState;
use crate::App;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;

pub fn draw_bfg_status<B>(rect: &mut Frame<B>, chunk: Rect, app: &App)
where
    B: Backend,
{
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    let market_trade_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(body_chunks[0]);

    let system_account_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(body_chunks[1]);

    let market_info = draw_tick(false, app.state());
    let trade_info = draw_tick(false, app.state());
    let system_info = draw_tick(false, app.state());
    let account_info = draw_tick(false, app.state());
    rect.render_widget(market_info, market_trade_chunks[0]);
    rect.render_widget(trade_info, market_trade_chunks[1]);
    rect.render_widget(system_info, system_account_chunks[0]);
    rect.render_widget(account_info, system_account_chunks[1]);
}

pub fn draw_tick<'a>(loading: bool, state: &AppState) -> Paragraph<'a> {
    let loading_text = if loading { "Loading..." } else { "" };
    let tick_text = if let Some(ticks) = state.count_tick() {
        format!("Tick count: {}", ticks)
    } else {
        String::default()
    };
    Paragraph::new(vec![
        Spans::from(Span::raw(loading_text)),
        Spans::from(Span::raw(tick_text)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .border_type(BorderType::Plain),
    )
}
