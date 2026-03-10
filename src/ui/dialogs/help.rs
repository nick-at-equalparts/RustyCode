use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Row, Table};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Help dialog -- centered overlay showing keybindings in a two-column table.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(60, 70, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Help - Keybindings (Esc to close) ")
        .title_style(Style::default().fg(theme.accent).add_modifier(Modifier::BOLD))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    let key_style = Style::default()
        .fg(theme.accent)
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(theme.fg);
    let section_style = Style::default()
        .fg(theme.warning)
        .add_modifier(Modifier::BOLD | Modifier::UNDERLINED);

    let rows = vec![
        Row::new(vec![
            Span::styled("-- Navigation --", section_style),
            Span::raw(""),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+C / Esc", key_style),
            Span::styled("Quit / Close dialog / Abort", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Tab", key_style),
            Span::styled("Switch between Chat and Logs pages", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+B", key_style),
            Span::styled("Toggle sidebar", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Up / Down", key_style),
            Span::styled("Navigate sidebar / dialog items", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Enter", key_style),
            Span::styled("Send message / Select item", desc_style),
        ]),
        Row::new(vec![Span::raw(""), Span::raw("")]),
        Row::new(vec![
            Span::styled("-- Dialogs --", section_style),
            Span::raw(""),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+O", key_style),
            Span::styled("Open session picker", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+K", key_style),
            Span::styled("Open model picker", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+P", key_style),
            Span::styled("Open command palette", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+T", key_style),
            Span::styled("Open theme picker", desc_style),
        ]),
        Row::new(vec![
            Span::styled("?", key_style),
            Span::styled("Open this help dialog", desc_style),
        ]),
        Row::new(vec![Span::raw(""), Span::raw("")]),
        Row::new(vec![
            Span::styled("-- Messages --", section_style),
            Span::raw(""),
        ]),
        Row::new(vec![
            Span::styled("Page Up / Page Down", key_style),
            Span::styled("Scroll message history", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Home / End", key_style),
            Span::styled("Jump to top / bottom of messages", desc_style),
        ]),
        Row::new(vec![Span::raw(""), Span::raw("")]),
        Row::new(vec![
            Span::styled("-- Editor --", section_style),
            Span::raw(""),
        ]),
        Row::new(vec![
            Span::styled("Left / Right", key_style),
            Span::styled("Move cursor in input", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+A / Ctrl+E", key_style),
            Span::styled("Jump to start / end of input", desc_style),
        ]),
        Row::new(vec![
            Span::styled("Ctrl+U", key_style),
            Span::styled("Clear input line", desc_style),
        ]),
    ];

    let widths = [Constraint::Length(22), Constraint::Min(1)];

    let table = Table::new(rows, widths)
        .block(block)
        .column_spacing(2);

    frame.render_widget(table, popup);
}
