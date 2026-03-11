use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::ui::themes::get_theme;

/// Render a single-line "Thinking..." indicator inside `area`.
pub fn render(frame: &mut Frame, theme_name: &str, area: Rect) {
    let theme = get_theme(theme_name);

    let bar = Paragraph::new(Line::from(vec![
        Span::styled(
            " ● ",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Thinking...",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::ITALIC | Modifier::BOLD),
        ),
    ]))
    .style(Style::default().bg(theme.bg));

    frame.render_widget(bar, area);
}
