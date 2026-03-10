use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Command palette dialog -- centered overlay with filter input and command list.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(60, 60, area);

    frame.render_widget(Clear, popup);

    let outer_block = Block::default()
        .title(" Commands (type to filter, Enter: run, Esc: close) ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    frame.render_widget(outer_block, popup);

    // Inner area (inside the border)
    let inner = popup.inner(Margin { vertical: 1, horizontal: 1 });

    // Split inner into filter input (1 line) + list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1)])
        .split(inner);

    // ---- Filter input ----
    let filter_display = if app.dialog_filter.is_empty() {
        Span::styled("Type to filter...", Style::default().fg(theme.muted))
    } else {
        Span::styled(&app.dialog_filter, Style::default().fg(theme.fg))
    };
    let filter_line = Paragraph::new(Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.accent)),
        filter_display,
    ]));
    frame.render_widget(filter_line, chunks[0]);

    // ---- Filtered command list ----
    let filter_lower = app.dialog_filter.to_lowercase();
    let filtered: Vec<&crate::types::Command> = app
        .commands
        .iter()
        .filter(|cmd| {
            if filter_lower.is_empty() {
                return true;
            }
            cmd.name.to_lowercase().contains(&filter_lower)
                || cmd
                    .description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&filter_lower))
                    .unwrap_or(false)
        })
        .collect();

    if filtered.is_empty() {
        let items = vec![ListItem::new(Span::styled(
            "  No matching commands",
            Style::default().fg(theme.muted),
        ))];
        let list = List::new(items);
        frame.render_widget(list, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = filtered
        .iter()
        .map(|cmd| {
            let desc = cmd.description.as_deref().unwrap_or("");
            let shortcut = cmd
                .shortcut
                .as_ref()
                .map(|s| format!("  [{}]", s))
                .unwrap_or_default();

            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {}", cmd.name),
                    Style::default().fg(theme.fg),
                ),
                Span::styled(
                    shortcut,
                    Style::default()
                        .fg(theme.accent)
                        .add_modifier(Modifier::DIM),
                ),
                Span::styled(
                    format!("  {}", desc),
                    Style::default().fg(theme.muted),
                ),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    let sel = app.dialog_selected.min(items.len().saturating_sub(1));
    state.select(Some(sel));

    let list = List::new(items)
        .highlight_style(
            Style::default()
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut state);
}
