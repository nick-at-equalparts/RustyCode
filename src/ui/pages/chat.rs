use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::components;

/// Draw the main chat page.
///
/// Layout:
/// - If sidebar visible: horizontal split  sidebar(25%) | main(75%)
/// - Main area: vertical split  messages(stretch) | [thinking(1)] | editor | status(1)
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let main_area = if app.sidebar_visible {
        let horiz = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);

        components::sidebar::render(frame, app, horiz[0]);
        horiz[1]
    } else {
        area
    };

    let is_busy = app.is_session_busy();
    let thinking_h: u16 = if is_busy { 1 } else { 0 };
    let editor_h = app.editor_height();

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),                  // messages (stretches)
            Constraint::Length(thinking_h),       // thinking indicator (0 when idle)
            Constraint::Length(editor_h),         // editor
            Constraint::Length(1),                // status bar
        ])
        .split(main_area);

    components::message_list::render(frame, app, vert[0]);
    if is_busy {
        components::thinking_bar::render(frame, &app.theme_name, vert[1]);
    }
    components::editor::render(frame, app, vert[2]);
    components::status_bar::render(frame, app, vert[3]);
}
