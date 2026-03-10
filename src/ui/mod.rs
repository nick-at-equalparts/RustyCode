pub mod components;
pub mod dialogs;
pub mod pages;
pub mod themes;

use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::{App, Dialog, Page};
use crate::types::ToastLevel;

use self::themes::get_theme;

/// Main entry point: renders the active page, optional dialog overlay, and toast.
pub fn draw(frame: &mut Frame, app: &App) {
    let area = frame.area();
    let theme = get_theme(&app.theme_name);

    // Fill background
    let bg_block = Block::default().style(Style::default().bg(theme.bg));
    frame.render_widget(bg_block, area);

    // ---- Active page --------------------------------------------------
    match app.active_page {
        Page::Chat => pages::chat::draw(frame, app, area),
        Page::Logs => pages::logs::draw(frame, app, area),
    }

    // ---- Dialog overlay -----------------------------------------------
    if let Some(ref dialog) = app.active_dialog {
        match dialog {
            Dialog::Sessions => dialogs::session::draw(frame, app, area),
            Dialog::Models => dialogs::models::draw(frame, app, area),
            Dialog::Commands => dialogs::commands::draw(frame, app, area),
            Dialog::Help => dialogs::help::draw(frame, app, area),
            Dialog::Themes => dialogs::theme::draw(frame, app, area),
            Dialog::Permission => dialogs::permission::draw(frame, app, area),
            Dialog::Question => dialogs::question::draw(frame, app, area),
            Dialog::Quit => dialogs::quit::draw(frame, app, area),
        }
    }

    // ---- Toast notification -------------------------------------------
    if let Some(ref toast) = app.toast {
        render_toast(frame, app, area, toast);
    }
}

/// Render a toast notification at the bottom-right of the screen.
fn render_toast(frame: &mut Frame, app: &App, area: Rect, toast: &crate::types::Toast) {
    let theme = get_theme(&app.theme_name);

    let toast_width = (toast.message.len() as u16 + 4).min(area.width.saturating_sub(4));
    let toast_height = 3u16;
    let x = area.width.saturating_sub(toast_width + 2);
    let y = area.height.saturating_sub(toast_height + 1);
    let toast_area = Rect::new(x, y, toast_width, toast_height);

    frame.render_widget(Clear, toast_area);

    let (border_color, icon) = match toast.level {
        ToastLevel::Info => (theme.accent, "i"),
        ToastLevel::Success => (theme.success, "+"),
        ToastLevel::Warning => (theme.warning, "!"),
        ToastLevel::Error => (theme.error, "x"),
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let content = Paragraph::new(Line::from(vec![
        Span::styled(
            format!(" [{}] ", icon),
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&toast.message),
    ]))
    .block(block)
    .wrap(Wrap { trim: true });

    frame.render_widget(content, toast_area);
}
