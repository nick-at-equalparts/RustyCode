use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::{get_theme, list_themes};

/// Theme picker dialog -- centered overlay listing available themes.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(40, 50, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Themes (Enter: select, Esc: close) ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let themes = list_themes();
    let items: Vec<ListItem> = themes
        .iter()
        .map(|&name| {
            let is_current = name.to_lowercase() == app.theme_name.to_lowercase();
            let marker = if is_current { " *" } else { "" };
            let style = if is_current {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", name), style),
                Span::styled(marker, Style::default().fg(theme.success)),
            ]))
        })
        .collect();

    let mut state = ListState::default();
    let sel = app.dialog_selected.min(items.len().saturating_sub(1));
    state.select(Some(sel));

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
