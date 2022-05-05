use crate::app::state::AppState;
use crate::App;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Paragraph};
use tui::Frame;
use bfg_core::domain::State;
use bfg_core::ports::BfgService;

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

    let market_info = draw_market_info(&app.bfg.read().unwrap().state);
    let trade_info = draw_trade_info(&app.bfg.read().unwrap().state);
    let system_info = draw_system_info(&app.bfg.read().unwrap().state);
    let account_info = draw_account_info(&app.bfg.read().unwrap().state);
    rect.render_widget(market_info, market_trade_chunks[0]);
    rect.render_widget(trade_info, market_trade_chunks[1]);
    rect.render_widget(system_info, system_account_chunks[0]);
    rect.render_widget(account_info, system_account_chunks[1]);
}

pub fn draw_market_info<'a>(state: &State) -> Paragraph<'a> {
    let system_values = match state {
        State::Setup(v) => v,
        State::Entry(v) => v,
        State::Exit(v) => v,
        State::AwaitingEntryConfirmation(v) => v,
        State::AwaitingExitConfirmation(v) => v,
    };

    let text = format!("Value: {}", system_values.market);
    Paragraph::new(vec![
        Spans::from(Span::raw(text)),
    ])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Market Info")
                .border_type(BorderType::Plain),
        )
}

pub fn draw_trade_info<'a>(state: &State) -> Paragraph<'a> {
    let system_values = match state {
        State::Setup(v) => v,
        State::Entry(v) => v,
        State::Exit(v) => v,
        State::AwaitingEntryConfirmation(v) => v,
        State::AwaitingExitConfirmation(v) => v,
    };

    let text = format!("Value: {}", system_values.trade);
    Paragraph::new(vec![
        Spans::from(Span::raw(text)),
    ])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Trade Info")
                .border_type(BorderType::Plain),
        )
}
pub fn draw_account_info<'a>(state: &State) -> Paragraph<'a> {
    let system_values = match state {
        State::Setup(v) => v,
        State::Entry(v) => v,
        State::Exit(v) => v,
        State::AwaitingEntryConfirmation(v) => v,
        State::AwaitingExitConfirmation(v) => v,
    };

    let text = format!("Value: {}", system_values.account);
    Paragraph::new(vec![
        Spans::from(Span::raw(text)),
    ])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Account Info")
                .border_type(BorderType::Plain),
        )
}
pub fn draw_system_info<'a>(state: &State) -> Paragraph<'a> {
    let (state, system_values) = match state {
        State::Setup(v) => ("Setup", v),
        State::Entry(v) => ("Entry", v),
        State::Exit(v) => ("Exit", v),
        State::AwaitingEntryConfirmation(v) => ("AwaitingEntryConfirmation", v),
        State::AwaitingExitConfirmation(v) => ("AwaitingExitConfirmation", v),
    };

    let status = format!("Status: {}", state);
    let text = format!("Value: {}", system_values.system);
    Paragraph::new(vec![
        Spans::from(Span::raw(status)),
        Spans::from(Span::raw(text)),
    ])
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("System Info")
                .border_type(BorderType::Plain),
        )
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
