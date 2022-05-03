use tui::backend::Backend;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::Frame;

use crate::ui::body::draw_body;
use crate::ui::menu::draw_menu;
use crate::ui::status_bar::draw_title;
use crate::App;

pub mod body;
pub mod menu;
pub mod status_bar;

pub fn draw<B>(rect: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let size = rect.size();
    check_size(&size);

    // Vertical layout
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(5),
                Constraint::Percentage(90),
                Constraint::Length(5),
            ]
            .as_ref(),
        )
        .split(size);

    // Title block
    let title = draw_title();
    rect.render_widget(title, chunks[0]);

    // Body
    draw_body(rect, chunks[1], app);

    // Menu block
    let menu = draw_menu(app.active_menu_item().clone().into());
    rect.render_widget(menu, chunks[2]);
}

fn check_size(rect: &Rect) {
    if rect.width < 52 {
        panic!("Required width >= 52, (got {})", rect.width);
    }
    if rect.height < 28 {
        panic!("Required height >= 28, (got {})", rect.height);
    }
}
