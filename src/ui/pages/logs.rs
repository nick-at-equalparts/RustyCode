use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::themes::get_theme;

/// Draw the logs page (placeholder).
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);

    let block = Block::default()
        .title(" Logs ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let content = Paragraph::new(vec![
        Line::default(),
        Line::from(Span::styled(
            "  Logs - coming soon",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        )),
        Line::default(),
        Line::from(Span::styled(
            "  Press Tab to switch back to Chat.",
            Style::default().fg(theme.muted),
        )),
    ])
    .block(block)
    .wrap(Wrap { trim: true });

    frame.render_widget(content, area);
}
