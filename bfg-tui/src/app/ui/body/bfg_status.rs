use crate::app::state::AppState;
use crate::App;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::path::Iter;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use tui::Frame;
use bfg_ig::models::{AccountView, MarketView, TradeResultView};
use bfg_ig::SystemView;

pub fn draw_bfg_status<'a, B>(rect: &mut Frame<'a, B>, chunk: Rect, app: &App)
where
    B: Backend,
{
    let body_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(chunk);

    let market_results_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(body_chunks[0]);

    let system_account_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(75), Constraint::Percentage(25)].as_ref())
        .split(body_chunks[1]);

    let market= draw_market_view(&app.markets.markets);
    let results = draw_results_view(app.results.borrow());
    let system = draw_system_view(&app.systems.systems);
    let account = draw_account_view(app.account.borrow());
    rect.render_widget(market, market_results_chunks[0]);
    rect.render_widget(results, market_results_chunks[1]);
    rect.render_widget(system, system_account_chunks[0]);
    rect.render_widget(account, system_account_chunks[1]);
}

pub fn draw_market_view<'a>(views: &HashMap<String, MarketView>) -> Table<'a> {
    let header_style = Style::default().fg(Color::LightCyan);
    let row_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    let headers = Row::new(vec![
        Cell::from(Span::styled("Epic", header_style)),
        Cell::from(Span::styled("Bid", header_style)),
        Cell::from(Span::styled("Ask", header_style)),
        Cell::from(Span::styled("Spread", header_style)),
        Cell::from(Span::styled("State", header_style)),
        Cell::from(Span::styled("Delay", header_style)),
        Cell::from(Span::styled("Time", header_style)),
    ]);
    rows.push(headers); // TODO is this how headers are set?

    for (epic, view) in views { // Fake now but shows when we have multiple markets
        let bid = view.bid.unwrap_or_default();
        let ask = view.ask.unwrap_or_default();
        let spread = ask - bid;
        let row = Row::new(vec![
            Cell::from(Span::styled(format!("{:.8}..", view.epic), row_style)),
            Cell::from(Span::styled(format!("{:.1}", bid), row_style)),
            Cell::from(Span::styled(format!("{:.1}", ask), row_style)),
            Cell::from(Span::styled(format!("{:.1}", spread), row_style)),
            Cell::from(Span::styled(view.market_state.clone().unwrap_or_default(), row_style)),
            Cell::from(Span::styled(format!("{}", view.market_delay.unwrap_or_default()), row_style)),
            Cell::from(Span::styled(view.update_time.clone().unwrap_or_default(), row_style)),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Market"),
        )
        .widths(&[
            Constraint::Percentage(15), // epic
            Constraint::Percentage(10), // bid
            Constraint::Percentage(10), // ask
            Constraint::Percentage(10), // spread
            Constraint::Percentage(15), // state
            Constraint::Percentage(10), // delay
            Constraint::Percentage(10), // time
        ])
        .column_spacing(1)
}

pub fn draw_results_view<'a>(views: &Vec<TradeResultView>) -> Table<'a> {
    let header_style = Style::default().fg(Color::LightCyan);
    let row_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    let headers = Row::new(vec![
        Cell::from(Span::styled("Epic", header_style)),
        Cell::from(Span::styled("Entry Slippage", header_style)),
        Cell::from(Span::styled("P/L Pips", header_style)),
    ]);
   rows.push(headers); // TODO is this how headers are set?

    for view in views {
        let entry_slippage = (view.wanted_entry_level - view.actual_entry_level).abs();
        let pnl_pips;
        if view.reference.contains("LONG") {
            pnl_pips = view.exit_level - view.actual_entry_level;
        } else {
            pnl_pips = view.actual_entry_level - view.exit_level;
        }
        let row = Row::new(vec![
            Cell::from(Span::styled(format!("{:.8}..", view.epic), row_style)),
            Cell::from(Span::styled(format!("{:.1}", entry_slippage), row_style)),
            Cell::from(Span::styled(format!("{:.1}", pnl_pips), row_style)),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Result"),
        )
        .widths(&[Constraint::Percentage(40), Constraint::Percentage(30), Constraint::Percentage(30)])
        .column_spacing(1)
}

pub fn draw_account_view<'a>(state: &AccountView) -> Paragraph<'a> {
    let account = format!("Account: {}", state.account.clone());
    let pnl = format!(
        "PNL: {}",
        state.pnl.map(|n| n.to_string()).unwrap_or_default()
    );
    let available_cash = format!(
        "Available Cash: {}",
        state
            .available_cash
            .map(|n| n.to_string())
            .unwrap_or_default()
    );
    let funds = format!(
        "Funds: {}",
        state.funds.map(|n| n.to_string()).unwrap_or_default()
    );
    let margin = format!(
        "Margin: {}",
        state.margin.map(|n| n.to_string()).unwrap_or_default()
    );
    let equity_used = format!(
        "Equity Used: {}%",
        state.equity_used.map(|n| n.to_string()).unwrap_or_default()
    );
    Paragraph::new(vec![
        Spans::from(Span::raw(account)),
        Spans::from(Span::raw(available_cash)),
        Spans::from(Span::raw(margin)),
        Spans::from(Span::raw(pnl)),
        Spans::from(Span::raw(funds)),
        Spans::from(Span::raw(equity_used)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Account")
            .border_type(BorderType::Plain),
    )
}

pub fn draw_system_view<'a>(views: &HashMap<String,SystemView>) -> Table<'a> {
    let header_style = Style::default().fg(Color::LightCyan);
    let row_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];
    let headers = Row::new(vec![
        Cell::from(Span::styled("Epic", header_style)),
        Cell::from(Span::styled("Status", header_style)),
        Cell::from(Span::styled("Long", header_style)),
        Cell::from(Span::styled("Short", header_style)),
        Cell::from(Span::styled("High Ask", header_style)),
        Cell::from(Span::styled("High Bid", header_style)),
        Cell::from(Span::styled("Low Ask", header_style)),
        Cell::from(Span::styled("Low Bid", header_style)),
    ]);
    rows.push(headers); // TODO is this how headers are set?

    for (epic, view) in views { // Fake now but shows when we have multiple system
        let mut long = "".to_string();
        let mut short = "".to_string();
        for order in view.orders.iter() {
            if order.reference.contains("LONG") {
                long = order.state.clone();
            } else {
                short = order.state.clone();
            }
        }
        let row = Row::new(vec![
            Cell::from(Span::styled(format!("{:.8}..", view.epic), row_style)),
            Cell::from(Span::styled(view.state.clone(), row_style)),
            Cell::from(Span::styled(long, row_style)),
            Cell::from(Span::styled(short, row_style)),
            Cell::from(Span::styled(format!("{:.1}", view.opening_range_high_ask.unwrap_or_default()), row_style)),
            Cell::from(Span::styled(format!("{:.1}", view.opening_range_high_bid.unwrap_or_default()), row_style)),
            Cell::from(Span::styled(format!("{:.1}", view.opening_range_low_ask.unwrap_or_default()), row_style)),
            Cell::from(Span::styled(format!("{:.1}", view.opening_range_low_bid.unwrap_or_default()), row_style)),
        ]);
        rows.push(row);
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("System"),
        )
        .widths(&[
            Constraint::Percentage(10), // epic
            Constraint::Percentage(10), // status
            Constraint::Percentage(20), // long
            Constraint::Percentage(20), // short
            Constraint::Percentage(10), // hi ask
            Constraint::Percentage(10), // hi bid
            Constraint::Percentage(10), // lo ask
            Constraint::Percentage(10), //lo bid
        ])
        .column_spacing(1)
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
