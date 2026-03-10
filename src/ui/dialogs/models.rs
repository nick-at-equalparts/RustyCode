use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// A single entry in the filtered model list.
pub struct ModelEntry {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
}

/// Build the filtered list of models based on the current dialog filter.
pub fn filtered_models(app: &App) -> Vec<ModelEntry> {
    let filter = app.dialog_filter.to_lowercase();
    let mut entries = Vec::new();

    for provider in &app.providers {
        for (_key, model) in &provider.models {
            if !filter.is_empty() {
                let haystack = format!(
                    "{} {} {} {}",
                    provider.name, provider.id, model.name, model.id
                )
                .to_lowercase();
                if !filter.split_whitespace().all(|word| haystack.contains(word)) {
                    continue;
                }
            }
            entries.push(ModelEntry {
                provider_id: provider.id.clone(),
                provider_name: provider.name.clone(),
                model_id: model.id.clone(),
                model_name: model.name.clone(),
            });
        }
    }
    entries
}

/// Model picker dialog -- centered overlay with search filtering.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(60, 70, area);

    frame.render_widget(Clear, popup);

    let entries = filtered_models(app);

    // Split popup into search bar + list
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(1)])
        .split(popup);

    // Search input
    let search_text = if app.dialog_filter.is_empty() {
        " Type to search models...".to_string()
    } else {
        format!(" {}", app.dialog_filter)
    };
    let search_style = if app.dialog_filter.is_empty() {
        Style::default().fg(theme.muted)
    } else {
        Style::default().fg(theme.fg)
    };
    let search = Paragraph::new(search_text)
        .style(search_style)
        .block(
            Block::default()
                .title(format!(" Models ({}) ", entries.len()))
                .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.accent))
                .style(Style::default().bg(theme.bg)),
        );
    frame.render_widget(search, chunks[0]);

    // Model list
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    if entries.is_empty() {
        let items = vec![ListItem::new(Span::styled(
            "  No matching models",
            Style::default().fg(theme.muted),
        ))];
        let list = List::new(items).block(list_block);
        frame.render_widget(list, chunks[1]);
        return;
    }

    let items: Vec<ListItem> = entries
        .iter()
        .map(|entry| {
            let is_current = app
                .current_model
                .as_ref()
                .map(|(pid, mid)| pid == &entry.provider_id && mid == &entry.model_id)
                .unwrap_or(false);

            let marker = if is_current { " *" } else { "" };
            let style = if is_current {
                Style::default()
                    .fg(theme.success)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.fg)
            };

            ListItem::new(Line::from(vec![
                Span::styled(format!("  {}", entry.model_name), style),
                Span::styled(
                    format!("  ({})", entry.provider_name),
                    Style::default().fg(theme.muted),
                ),
                Span::styled(marker, Style::default().fg(theme.success)),
            ]))
        })
        .collect();

    let sel = app.dialog_selected.min(items.len().saturating_sub(1));
    let mut state = ListState::default();
    state.select(Some(sel));

    let list = List::new(items)
        .block(list_block)
        .highlight_style(
            Style::default()
                .bg(theme.selection)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    frame.render_stateful_widget(list, chunks[1], &mut state);
}
