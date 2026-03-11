use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::App;
use crate::ui::dialogs::centered_rect;
use crate::ui::themes::get_theme;

/// Question dialog -- centered overlay showing a question and selectable options.
pub fn draw(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let popup = centered_rect(55, 50, area);

    frame.render_widget(Clear, popup);

    let block = Block::default()
        .title(" Question ")
        .title_style(
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.accent))
        .style(Style::default().bg(theme.bg).fg(theme.fg));

    if let Some(question) = app.pending_questions.first() {
        let inner = popup.inner(Margin {
            vertical: 1,
            horizontal: 1,
        });

        frame.render_widget(block, popup);

        // Split into question text area and options list
        let question_height = (question.question.len() as u16 / inner.width.max(1) + 2).min(6);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(question_height), Constraint::Min(1)])
            .split(inner);

        // ---- Question text ----
        let question_para = Paragraph::new(Span::styled(
            &question.question,
            Style::default().fg(theme.fg),
        ))
        .wrap(Wrap { trim: true });
        frame.render_widget(question_para, chunks[0]);

        // ---- Options list ----
        let is_multi = question.multi_select.unwrap_or(false);

        if let Some(ref options) = question.options {
            let items: Vec<ListItem> = options
                .iter()
                .map(|opt| {
                    let selected = opt.selected.unwrap_or(false);
                    let checkbox = if is_multi {
                        if selected {
                            "[x] "
                        } else {
                            "[ ] "
                        }
                    } else {
                        if selected {
                            "(*) "
                        } else {
                            "( ) "
                        }
                    };

                    ListItem::new(Line::from(vec![
                        Span::styled(
                            checkbox,
                            Style::default().fg(if selected { theme.success } else { theme.muted }),
                        ),
                        Span::styled(&opt.label, Style::default().fg(theme.fg)),
                    ]))
                })
                .collect();

            let mut state = ListState::default();
            let sel = app.dialog_selected.min(items.len().saturating_sub(1));
            state.select(Some(sel));

            let hint = if is_multi {
                " (Space: toggle, Enter: confirm, Esc: cancel) "
            } else {
                " (Enter: select, Esc: cancel) "
            };

            let list = List::new(items)
                .block(
                    Block::default()
                        .title(hint)
                        .title_style(Style::default().fg(theme.muted))
                        .borders(Borders::TOP)
                        .border_style(Style::default().fg(theme.border)),
                )
                .highlight_style(
                    Style::default()
                        .bg(theme.selection)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol("> ");

            frame.render_stateful_widget(list, chunks[1], &mut state);
        } else {
            let para = Paragraph::new(Span::styled(
                "  (No options provided)",
                Style::default().fg(theme.muted),
            ));
            frame.render_widget(para, chunks[1]);
        }
    } else {
        let para = Paragraph::new("  No pending questions.")
            .block(block)
            .wrap(Wrap { trim: true });
        frame.render_widget(para, popup);
    }
}
