use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::state::App;
use crate::types::SessionStatus;
use crate::ui::themes::get_theme;

/// Render the session sidebar inside `area`.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);

    let block = Block::default()
        .title(" Sessions ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.border))
        .style(Style::default().bg(theme.bg));

    if app.sessions.is_empty() {
        let empty = List::new(vec![ListItem::new(Span::styled(
            "  No sessions",
            Style::default().fg(theme.muted),
        ))])
        .block(block);
        frame.render_widget(empty, area);
        return;
    }

    // Group sessions by recency buckets.
    // For simplicity we just list them in order with group headers.
    let mut items: Vec<ListItem> = Vec::new();
    let mut current_group: Option<&str> = None;

    for (i, session) in app.sessions.iter().enumerate() {
        let group = session_time_group(&session.time.updated.to_string());
        if current_group != Some(group) {
            current_group = Some(group);
            items.push(ListItem::new(Span::styled(
                format!(" -- {} --", group),
                Style::default()
                    .fg(theme.muted)
                    .add_modifier(Modifier::BOLD),
            )));
        }

        let title = session
            .title
            .as_deref()
            .or_else(|| session.summary.as_ref().and_then(|s| s.title.as_deref()))
            .unwrap_or("New Session");

        // Status indicator
        let status_icon = match app.session_statuses.get(&session.id) {
            Some(SessionStatus::Busy) => Span::styled("* ", Style::default().fg(theme.tool_running)),
            Some(SessionStatus::Retry { .. }) => {
                Span::styled("r ", Style::default().fg(theme.warning))
            }
            _ => Span::styled("  ", Style::default()),
        };

        let is_current = app
            .current_session
            .as_ref()
            .map(|cs| cs.id == session.id)
            .unwrap_or(false);

        let style = if is_current {
            Style::default()
                .fg(theme.accent)
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD)
        } else if i == app.sidebar_selected {
            Style::default().fg(theme.fg).bg(theme.selection)
        } else {
            Style::default().fg(theme.fg)
        };

        // Truncate title to fit sidebar
        let max_len = area.width.saturating_sub(6) as usize;
        let display_title: String = if title.len() > max_len {
            format!("{}...", &title[..max_len.saturating_sub(3)])
        } else {
            title.to_string()
        };

        items.push(ListItem::new(Line::from(vec![
            status_icon,
            Span::styled(display_title, style),
        ])));
    }

    let mut list_state = ListState::default();
    list_state.select(Some(app.sidebar_selected));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, area, &mut list_state);
}

/// Map an ISO timestamp string to a recency group label.
/// This is a simplified version that just checks the date prefix.
fn session_time_group(timestamp: &str) -> &'static str {
    // Very simple grouping based on the date portion.
    // In a real app you would parse the date and compare to now.
    let date_prefix = if timestamp.len() >= 10 {
        &timestamp[..10]
    } else {
        return "Older";
    };

    // We use a static approach: the caller would ideally pass "now",
    // but for rendering we just bucket by a rough heuristic.
    // This is a placeholder -- in production, compare with chrono::Utc::now().
    let _ = date_prefix;
    "Recent"
}
