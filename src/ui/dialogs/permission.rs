use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Permission request dialog -- prominent centered overlay.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(70, 60, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Permission Required ")
        .title_style(
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.warning))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    if let Some(perm) = app.pending_permissions.first() {
        // The permission type doubles as the tool name (e.g. "bash", "read", "write")
        let tool_name = perm.permission.as_deref().unwrap_or("unknown");
        let description = perm
            .description
            .as_deref()
            .unwrap_or("Tool requires permission to proceed.");

        let mut lines: Vec<Line> = Vec::new();

        lines.push(Line::default());
        lines.push(Line::from(vec![
            Span::styled(
                "  Tool: ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(tool_name, Style::default().fg(theme.fg)),
        ]));

        // Show the command / patterns (e.g. the actual bash command)
        if let Some(ref patterns) = perm.patterns {
            if !patterns.is_empty() {
                lines.push(Line::default());
                lines.push(Line::from(Span::styled(
                    "  Command:",
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::BOLD),
                )));
                for pattern in patterns {
                    for cmd_line in pattern.lines() {
                        lines.push(Line::from(Span::styled(
                            format!("    {}", cmd_line),
                            Style::default().fg(theme.warning),
                        )));
                    }
                }
            }
        }

        // Show description if we have one
        if perm.description.is_some() {
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                format!("  {}", description),
                Style::default().fg(theme.muted),
            )));
        }

        // Show "always allow" pattern hint if available
        if let Some(ref always) = perm.always {
            if !always.is_empty() {
                lines.push(Line::default());
                lines.push(Line::from(vec![
                    Span::styled("  Always pattern: ", Style::default().fg(theme.muted)),
                    Span::styled(always.join(", "), Style::default().fg(theme.muted)),
                ]));
            }
        }

        lines.push(Line::default());
        lines.push(Line::default());

        // Action hints
        lines.push(Line::from(vec![
            Span::styled(
                "  [y] ",
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Allow once", Style::default().fg(theme.fg)),
            Span::styled("    ", Style::default()),
            Span::styled(
                "[a] ",
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Always allow", Style::default().fg(theme.fg)),
            Span::styled("    ", Style::default()),
            Span::styled(
                "[n] ",
                Style::default()
                    .fg(theme.error)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Reject", Style::default().fg(theme.fg)),
        ]));

        let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: true });
        frame.render_widget(para, popup);
    } else {
        let para = Paragraph::new("  No pending permission requests.")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(para, popup);
    }
}
