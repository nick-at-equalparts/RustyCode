use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::Rect;
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::components;

/// Draw the main chat page.
///
/// Layout:
/// - If sidebar visible: horizontal split  sidebar(25%) | main(75%)
/// - Main area: vertical split  messages(stretch) | editor(4 lines) | status(1 line)
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

    let vert = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(1),       // messages (stretches)
            Constraint::Length(4),     // editor
            Constraint::Length(1),     // status bar
        ])
        .split(main_area);

    components::message_list::render(frame, app, vert[0]);
    components::editor::render(frame, app, vert[1]);
    components::status_bar::render(frame, app, vert[2]);
}
