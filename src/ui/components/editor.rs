use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::themes::get_theme;

/// Render the text input editor inside `area`.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let is_busy = app.is_session_busy();

    let border_color = if is_busy { theme.warning } else { theme.border };
    let title = if is_busy { " Abort (Ctrl+C) " } else { " > " };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(if is_busy { theme.warning } else { theme.accent }))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));

    if is_busy {
        let hint = Paragraph::new(Span::styled(
            "Session is busy...",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        ))
        .block(block);
        frame.render_widget(hint, area);
        return;
    }

    // Build the text with a visible cursor.
    let input = &app.input_text;
    let cursor_pos = app.input_cursor.min(input.len());

    let before_cursor = &input[..cursor_pos];
    let cursor_char = if cursor_pos < input.len() {
        &input[cursor_pos..cursor_pos + 1]
    } else {
        " "
    };
    let after_cursor = if cursor_pos < input.len() {
        &input[cursor_pos + 1..]
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::styled(before_cursor, Style::default().fg(theme.fg)),
        Span::styled(
            cursor_char,
            Style::default()
                .fg(theme.bg)
                .bg(theme.fg)
                .add_modifier(Modifier::SLOW_BLINK),
        ),
        Span::styled(after_cursor, Style::default().fg(theme.fg)),
    ]);

    let paragraph = Paragraph::new(line).block(block);
    frame.render_widget(paragraph, area);

    // Set the real terminal cursor position for IME support.
    let inner = area.inner(Margin { vertical: 1, horizontal: 1 });
    let cx = inner.x + cursor_pos as u16;
    let cy = inner.y;
    if cx < inner.x + inner.width {
        frame.set_cursor_position(Position::new(cx, cy));
    }
}
