use crate::app::state::AppState;
use crate::App;
use bfg_core::models::{AccountUpdate, MarketUpdate, SystemState, SystemValues};
use ig_brokerage_adapter::realtime::models::{OpenPositionUpdate, OpuStatus};
use std::borrow::Borrow;
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, BorderType, Borders, Cell, Paragraph, Row, Table};
use tui::Frame;

pub fn draw_bfg_status<'a, B>(rect: &mut Frame<'a, B>, chunk: Rect, app: &App)
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

    let market_info = draw_market_info(app.market.borrow());
    let trade_info = draw_trade_info(app.trade.borrow());
    let system_info = draw_system_info(app.system.borrow());
    let account_info = draw_account_info(app.account.borrow());
    rect.render_widget(market_info, market_trade_chunks[0]);
    rect.render_widget(trade_info, market_trade_chunks[1]);
    rect.render_widget(system_info, system_account_chunks[0]);
    rect.render_widget(account_info, system_account_chunks[1]);
}

pub fn draw_market_info<'a>(state: &MarketUpdate) -> Paragraph<'a> {
    let bid = state.bid.unwrap_or_default();
    let offer = state.offer.unwrap_or_default();
    let real_spread = offer - bid;
    let spread = format!("Spread: {:.1}", real_spread);
    let bid = format!(
        "Bid: {}",
        state.bid.map(|n| n.to_string()).unwrap_or_default()
    );
    let offer = format!(
        "Offer: {}",
        state.offer.map(|n| n.to_string()).unwrap_or_default()
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
            .title("Market Info")
            .border_type(BorderType::Plain),
    )
}

pub fn draw_trade_info<'a>(state: &Option<OpenPositionUpdate>) -> Table<'a> {
    let key_style = Style::default().fg(Color::LightCyan);
    let help_style = Style::default().fg(Color::Gray);

    let mut rows = vec![];

    if let Some(s) = state {
        if s.status != OpuStatus::DELETED {
            let row = Row::new(vec![
                Cell::from(Span::styled("Direction", key_style)),
                Cell::from(Span::styled(s.direction.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Level", key_style)),
                Cell::from(Span::styled(s.level.to_string(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Limit Level", key_style)),
                Cell::from(Span::styled(format!("{:?}", s.limit_level), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Stop Level", key_style)),
                Cell::from(Span::styled(format!("{:?}", s.stop_level), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Deal Id", key_style)),
                Cell::from(Span::styled(s.deal_id.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Deal Id Origin", key_style)),
                Cell::from(Span::styled(s.deal_id_origin.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Expiry", key_style)),
                Cell::from(Span::styled(s.expiry.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Timestamp", key_style)),
                Cell::from(Span::styled(s.timestamp.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Size", key_style)),
                Cell::from(Span::styled(s.size.to_string(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Status", key_style)),
                Cell::from(Span::styled(format!("{:?}", s.status.clone()), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Deal Status", key_style)),
                Cell::from(Span::styled(format!("{:?}", s.deal_status), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Epic", key_style)),
                Cell::from(Span::styled(s.epic.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Guaranteed Stop", key_style)),
                Cell::from(Span::styled(s.guaranteed_stop.to_string(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Deal Reference", key_style)),
                Cell::from(Span::styled(s.deal_reference.clone(), help_style)),
            ]);
            rows.push(row);

            let row = Row::new(vec![
                Cell::from(Span::styled("Channel", key_style)),
                Cell::from(Span::styled(s.channel.clone(), help_style)),
            ]);
            rows.push(row);
        }
    }

    Table::new(rows)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Plain)
                .title("Trade status"),
        )
        .widths(&[Constraint::Length(20), Constraint::Min(20)])
        .column_spacing(1)
}
pub fn draw_account_info<'a>(state: &AccountUpdate) -> Paragraph<'a> {
    let account = format!("Account: {}", state.account.clone().unwrap_or_default());
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
            .title("Account Info")
            .border_type(BorderType::Plain),
    )
}
pub fn draw_system_info<'a>(state: &SystemState) -> Paragraph<'a> {
    let (state, system_values) = match state {
        SystemState::Setup => ("Setup", None),
        SystemState::SetupWorkingOrder(v) => ("SetupWorkingOrder", Some(v)),
        SystemState::ManageOrder(v) => ("ManageOrder", Some(v)),
    };

    let status = format!("System Status: {}", state);
    let mut order_state_long = "None".to_string();
    let mut order_state_short = "None".to_string();
    if let Some(SystemValues {
        working_orders: (long, short),
        ..
    }) = system_values
    {
        if let Some(l) = long {
            order_state_long = format!("{:?}", l);
        }
        if let Some(s) = short {
            order_state_short = format!("{:?}", s);
        }
    }
    let order_status_long = format!(" - Status Long: {:?}", order_state_long);
    let order_status_short = format!(" - Status Short: {:?}", order_state_short);
    let or_high_ask = format!(
        "OR High Ask: {}",
        system_values
            .map(|a| a.or_high_ask.to_string())
            .unwrap_or_default()
    );
    let or_low_ask = format!(
        "OR Low Ask: {}",
        system_values
            .map(|a| a.or_low_ask.to_string())
            .unwrap_or_default()
    );
    let or_high_bid = format!(
        "OR High Bid: {}",
        system_values
            .map(|a| a.or_high_bid.to_string())
            .unwrap_or_default()
    );
    let or_low_bid = format!(
        "OR Low Bid: {}",
        system_values
            .map(|a| a.or_low_bid.to_string())
            .unwrap_or_default()
    );
    Paragraph::new(vec![
        Spans::from(Span::raw(status)),
        Spans::from(Span::raw(order_status_long)),
        Spans::from(Span::raw(order_status_short)),
        Spans::from(Span::raw(or_high_ask)),
        Spans::from(Span::raw(or_low_ask)),
        Spans::from(Span::raw(or_high_bid)),
        Spans::from(Span::raw(or_low_bid)),
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
