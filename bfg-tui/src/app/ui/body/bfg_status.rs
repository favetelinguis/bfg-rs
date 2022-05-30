use crate::app::state::AppState;
use crate::App;
use std::borrow::Borrow;
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
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunk);

    let market_results_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(body_chunks[0]);

    let system_account_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(body_chunks[1]);

    let market= draw_market_view(app.market.borrow());
    let results = draw_results_view(app.results.borrow());
    let system = draw_system_view(app.system.borrow());
    let account = draw_account_view(app.account.borrow());
    rect.render_widget(market, market_results_chunks[0]);
    rect.render_widget(results, market_results_chunks[1]);
    rect.render_widget(system, system_account_chunks[0]);
    rect.render_widget(account, system_account_chunks[1]);
}

pub fn draw_market_view<'a>(state: &MarketView) -> Paragraph<'a> {
    let bid = state.bid.unwrap_or_default();
    let ask = state.ask.unwrap_or_default();
    let real_spread = ask - bid;
    let spread = format!("Spread: {:.1}", real_spread);
    let bid = format!(
        "Bid: {}",
        state.bid.map(|n| n.to_string()).unwrap_or_default()
    );
    let offer = format!(
        "Ask: {}",
        state.ask.map(|n| n.to_string()).unwrap_or_default()
    );
    let market_state = format!("Market state: {:?}", state.market_state);
    let market_delay = format!(
        "Market delay: {}",
        state
            .market_delay
            .map(|n| n.to_string())
            .unwrap_or_default()
    );
    let update_time = format!(
        "Update time: {}",
        state.update_time.clone().unwrap_or_default()
    );
    Paragraph::new(vec![
        Spans::from(Span::raw(spread)),
        Spans::from(Span::raw(bid)),
        Spans::from(Span::raw(offer)),
        Spans::from(Span::raw(market_state)),
        Spans::from(Span::raw(market_delay)),
        Spans::from(Span::raw(update_time)),
    ])
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Market")
            .border_type(BorderType::Plain),
    )
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
            Cell::from(Span::styled(view.epic.clone(), row_style)),
            Cell::from(Span::styled(entry_slippage.to_string(), row_style)),
            Cell::from(Span::styled(pnl_pips.to_string(), row_style)),
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
        .widths(&[Constraint::Length(20), Constraint::Min(20)])
        .column_spacing(1)
}

pub fn draw_account_view<'a>(state: &AccountView) -> Paragraph<'a> {
    let account = format!("Account: {}", state.account.clone());
    let pnl = format!(
        "PNL: {}",
        state.pnl.map(|n| n.to_string()).unwrap_or_default()
    );
    let pnl_lr = format!(
        "PNL LR: {}",
        state.pnl_lr.map(|n| n.to_string()).unwrap_or_default()
    );
    let pnl_nlr = format!(
        "PNL NLR: {}",
        state.pnl_nlr.map(|n| n.to_string()).unwrap_or_default()
    );
    let deposit = format!(
        "Deposit: {}",
        state.deposit.map(|n| n.to_string()).unwrap_or_default()
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
    let margin_lr = format!(
        "Margin LR: {}",
        state.margin_lr.map(|n| n.to_string()).unwrap_or_default()
    );
    let margin_nlr = format!(
        "Margin NLR: {}",
        state.margin_nlr.map(|n| n.to_string()).unwrap_or_default()
    );
    let available_to_deal = format!(
        "Available To Deal: {}",
        state
            .available_to_deal
            .map(|n| n.to_string())
            .unwrap_or_default()
    );
    let equity = format!(
        "Equity: {}",
        state.equity.map(|n| n.to_string()).unwrap_or_default()
    );
    let equity_used = format!(
        "Equity Used: {}",
        state.equity_used.map(|n| n.to_string()).unwrap_or_default()
    );
    Paragraph::new(vec![
        Spans::from(Span::raw(account)),
        Spans::from(Span::raw(pnl)),
        Spans::from(Span::raw(pnl_lr)),
        Spans::from(Span::raw(pnl_nlr)),
        Spans::from(Span::raw(deposit)),
        Spans::from(Span::raw(available_cash)),
        Spans::from(Span::raw(funds)),
        Spans::from(Span::raw(margin)),
        Spans::from(Span::raw(margin_lr)),
        Spans::from(Span::raw(margin_nlr)),
        Spans::from(Span::raw(available_to_deal)),
        Spans::from(Span::raw(equity)),
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
pub fn draw_system_view<'a>(view: &SystemView) -> Paragraph<'a> {
    let status = format!("System Status: {}", view.state);
    let mut spans = vec![];
    for order in view.orders.iter() {
        let text = format!(" - {}: {}", order.reference, order.state);
        let span = Spans::from(Span::raw(text));
        spans.push(span);
    }
    let or_high_ask = format!(
        "Opening Range High Ask: {}",
        view.opening_range_high_ask.unwrap_or_default()
    );
    let or_low_ask = format!(
        "Opening Range Low Ask: {}",
        view.opening_range_low_ask.unwrap_or_default()
    );
    let or_high_bid = format!(
        "Opening Range High Bid: {}",
        view.opening_range_high_bid.unwrap_or_default()
    );
    let or_low_bid = format!(
        "Opening Range Low Bid: {}",
        view.opening_range_low_bid.unwrap_or_default()
    );
    let epic = format!("Epic: {}", view.epic);
    spans.extend(vec![
        Spans::from(Span::raw(status)),
        Spans::from(Span::raw(or_high_ask)),
        Spans::from(Span::raw(or_high_bid)),
        Spans::from(Span::raw(or_low_ask)),
        Spans::from(Span::raw(or_low_bid)),
        Spans::from(Span::raw(epic)),
    ]);

    Paragraph::new(spans)
    .style(Style::default().fg(Color::LightCyan))
    .alignment(Alignment::Left)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("System")
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
