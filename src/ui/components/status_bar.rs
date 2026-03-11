use ratatui::prelude::*;
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::state::{App, ChatMode};
use crate::ui::themes::get_theme;

/// Render a single-line status bar inside `area`.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);

    // ---- Left section: mode pill + project name + git branch ----
    let mut left_parts: Vec<Span> = Vec::new();

    // Mode indicator
    let (mode_label, mode_fg, mode_bg) = match app.chat_mode {
        ChatMode::Build => ("Build", theme.bg, theme.accent),
        ChatMode::Plan => ("Plan", theme.bg, theme.warning),
    };
    left_parts.push(Span::styled(
        format!(" {} ", mode_label),
        Style::default()
            .fg(mode_fg)
            .bg(mode_bg)
            .add_modifier(Modifier::BOLD),
    ));

    let project_name = app.project_name();
    left_parts.push(Span::styled(
        format!(" {} ", project_name),
        Style::default()
            .fg(theme.accent)
            .add_modifier(Modifier::BOLD),
    ));

    if let Some(branch) = app.vcs_branch() {
        left_parts.push(Span::styled(
            format!(" {} ", branch),
            Style::default().fg(theme.muted),
        ));
    }

    // ---- Center: session title ----
    let session_title = app.current_session_title();
    let max_title_len = (area.width as usize).saturating_sub(50) / 2;
    let truncated_title: String = if session_title.len() > max_title_len && max_title_len > 3 {
        format!("{}...", &session_title[..max_title_len.saturating_sub(3)])
    } else {
        session_title.to_string()
    };

    // ---- Right section: model, connection ----
    let model_name = app.model_display_name();
    let conn_indicator = if app.connected {
        Span::styled(" ok ", Style::default().fg(theme.success))
    } else {
        Span::styled(" -- ", Style::default().fg(theme.error))
    };

    let right_parts = vec![
        Span::styled(format!(" {} ", model_name), Style::default().fg(theme.fg)),
        Span::styled(" | ", Style::default().fg(theme.border)),
        conn_indicator,
    ];

    // Calculate widths for layout
    let left_width = left_parts.iter().map(|s| s.width()).sum::<usize>();
    let right_width = right_parts.iter().map(|s| s.width()).sum::<usize>();
    let center_width = truncated_title.len() + 2;
    let total_width = area.width as usize;

    // Compose the full line with spacing
    let mut spans: Vec<Span> = Vec::new();
    spans.extend(left_parts);

    let left_pad = if total_width > left_width + center_width + right_width {
        (total_width - left_width - center_width - right_width) / 2
    } else {
        1
    };
    if left_pad > 0 {
        spans.push(Span::raw(" ".repeat(left_pad)));
    }

    spans.push(Span::styled(
        format!(" {} ", truncated_title),
        Style::default().fg(theme.muted),
    ));

    let used = left_width + left_pad + center_width;
    let right_pad = total_width.saturating_sub(used + right_width);
    if right_pad > 0 {
        spans.push(Span::raw(" ".repeat(right_pad)));
    }

    spans.extend(right_parts);

    let bar =
        Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.selection).fg(theme.fg));

    frame.render_widget(bar, area);
}
