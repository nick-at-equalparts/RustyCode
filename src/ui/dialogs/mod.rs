pub mod commands;
pub mod help;
pub mod models;
pub mod permission;
pub mod question;
pub mod quit;
pub mod session;
pub mod theme;

use ratatui::layout::Rect;

/// Create a centered rectangle of the given percentage size within `area`.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_width = area.width * percent_x / 100;
    let popup_height = area.height * percent_y / 100;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    Rect::new(x, y, popup_width, popup_height)
}
