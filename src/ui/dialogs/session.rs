use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::state::App;
use crate::types::SessionStatus;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Session picker dialog -- centered overlay listing all sessions.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(60, 70, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Sessions (Enter: select, Esc: close) ")
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    if app.sessions.is_empty() {
        let items = vec![ListItem::new(Span::styled(
            "  No sessions available",
            Style::default().fg(theme.muted),
        ))];
        let list = List::new(items).block(block);
        frame.render_widget(list, popup);
        return;
    }

    let items: Vec<ListItem> = app
        .sessions
        .iter()
        .map(|session| {
            let title = session
                .title
                .as_deref()
                .or_else(|| session.summary.as_ref().and_then(|s| s.title.as_deref()))
                .unwrap_or("New Session");

            let status_icon = match app.session_statuses.get(&session.id) {
                Some(SessionStatus::Busy) => "[*] ",
                Some(SessionStatus::Retry { .. }) => "[r] ",
                _ => "    ",
            };

            let is_current = app
                .current_session
                .as_ref()
                .map(|cs| cs.id == session.id)
                .unwrap_or(false);

            let marker = if is_current { " (current)" } else { "" };

            let style = if is_current {
                Style::default()
                    .fg(theme.accent)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::styled(status_icon, Style::default().fg(theme.muted)),
                Span::styled(title.to_string(), style),
                Span::styled(marker, Style::default().fg(theme.muted)),
                Span::styled(
                    format!("  {}", session.time.updated),
                    Style::default().fg(theme.muted),
                ),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    state.select(Some(app.dialog_selected));

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, popup, &mut state);
}
