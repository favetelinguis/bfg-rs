use crate::ui::body::bfg_status::draw_bfg_status;
use crate::ui::body::help::draw_help;
use crate::ui::body::logs::draw_logs;
use crate::ui::menu::MenuItem;
use crate::App;
use tui::backend::Backend;
use tui::layout::Rect;
use tui::Frame;

pub mod bfg_status;
pub mod help;
pub mod logs;

pub fn draw_body<'a, B>(rect: &mut Frame<'a, B>, chunk: Rect, app: &App)
where
    B: Backend,
{
    match app.active_menu_item() {
        MenuItem::Home => draw_bfg_status(rect, chunk, app),
        MenuItem::Logs => draw_logs(rect, chunk),
        MenuItem::Help => draw_help(rect, chunk, app),
    };
}
