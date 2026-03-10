use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Quit confirmation dialog -- small centered overlay.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(35, 20, area);

    // Ensure minimum usable size
    let popup = if popup.height < 5 {
        centered_rect(35, 30, area)
    } else {
        popup
    };

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Quit ")
        .title_style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let text = vec![
        Line::default(),
        Line::from(Span::styled(
            "  Are you sure you want to quit?",
            Style::default().fg(theme.fg),
        )),
        Line::default(),
        Line::from(vec![
            Span::styled(
                "  [y] ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Yes", Style::default().fg(theme.fg)),
            Span::styled("    ", Style::default()),
            Span::styled(
                "[n] ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("No", Style::default().fg(theme.fg)),
        ]),
    ];

    let para = Paragraph::new(text).block(block).wrap(Wrap { trim: true });
    frame.render_widget(para, popup);
}
