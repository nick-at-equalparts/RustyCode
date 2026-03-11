use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::app::state::{App, ChatMode};
use crate::ui::themes::get_theme;

/// Render the text input editor inside `area`.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);
    let is_busy = app.is_session_busy();

    let border_color = if is_busy { theme.warning } else { theme.border };
    let title = if is_busy {
        " Abort (Ctrl+C) ".to_string()
    } else if app.chat_mode == ChatMode::Build {
        " > ".to_string()
    } else {
        format!(" {} > ", app.chat_mode.label())
    };

    let block = Block::default()
        .title(title)
        .title_style(Style::default().fg(if is_busy { theme.warning } else { theme.accent }))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(theme.bg));

    if is_busy {
        let hint = Paragraph::new(Span::styled(
            "Session is busy...",
            Style::default()
                .fg(theme.muted)
                .add_modifier(Modifier::ITALIC),
        ))
        .block(block);
        frame.render_widget(hint, area);
        return;
    }

    let cursor_style = Style::default()
        .fg(theme.bg)
        .bg(theme.fg)
        .add_modifier(Modifier::SLOW_BLINK);
    let text_style = Style::default().fg(theme.fg);

    // Paste mode: show "[Pasted X lines]" instead of raw text.
    if let Some(line_count) = app.paste_line_count {
        let label = format!("[Pasted {} lines]", line_count);
        let rendered = Line::from(vec![
            Span::styled(label, Style::default().fg(theme.muted).add_modifier(Modifier::ITALIC)),
            Span::styled(" ", cursor_style),
        ]);
        let paragraph = Paragraph::new(rendered).block(block);
        frame.render_widget(paragraph, area);
        // Cursor at end of label
        let inner = area.inner(Margin { vertical: 1, horizontal: 1 });
        let label_len = format!("[Pasted {} lines]", line_count).chars().count() as u16;
        let cx = inner.x + label_len;
        let cy = inner.y;
        if cx < inner.x + inner.width && cy < inner.y + inner.height {
            frame.set_cursor_position(Position::new(cx, cy));
        }
        return;
    }

    let inner_width = area.width.saturating_sub(2).max(1) as usize;
    let input = &app.input_text;
    let byte_cursor = app.cursor_byte_index();

    // Phase 1: Build visual line chunks by wrapping each logical line at inner_width.
    struct VisualChunk<'a> {
        text: &'a str,
        abs_byte_start: usize,
    }

    let mut visual_chunks: Vec<VisualChunk> = Vec::new();
    let mut byte_offset: usize = 0;

    for line_text in input.split('\n') {
        let line_start = byte_offset;
        let chars: Vec<(usize, char)> = line_text.char_indices().collect();
        let total_chars = chars.len();

        if total_chars == 0 {
            visual_chunks.push(VisualChunk {
                text: "",
                abs_byte_start: line_start,
            });
        } else {
            let mut ci = 0;
            while ci < total_chars {
                let end_ci = (ci + inner_width).min(total_chars);
                let start_byte = chars[ci].0;
                let end_byte = if end_ci < chars.len() {
                    chars[end_ci].0
                } else {
                    line_text.len()
                };
                visual_chunks.push(VisualChunk {
                    text: &line_text[start_byte..end_byte],
                    abs_byte_start: line_start + start_byte,
                });
                ci = end_ci;
            }
            // If cursor could be at exact wrap boundary (end of full-width last chunk),
            // add an empty visual line so it has somewhere to render.
            if total_chars % inner_width == 0 {
                visual_chunks.push(VisualChunk {
                    text: "",
                    abs_byte_start: line_start + line_text.len(),
                });
            }
        }

        byte_offset += line_text.len() + 1; // +1 for '\n'
    }

    // Phase 2: Render each visual line, placing cursor on the correct one.
    let mut rendered_lines: Vec<Line> = Vec::new();
    let mut cursor_visual_row: usize = 0;
    let mut cursor_visual_col: usize = 0;
    let mut found_cursor = false;

    for (vi, chunk) in visual_chunks.iter().enumerate() {
        let chunk_byte_end = chunk.abs_byte_start + chunk.text.len();

        // Determine if this is the last chunk of its logical line.
        // Next chunk from the same logical line has abs_byte_start == chunk_byte_end;
        // next chunk from a new logical line has abs_byte_start == chunk_byte_end + 1 (past '\n').
        let is_last_of_logical = vi + 1 >= visual_chunks.len()
            || visual_chunks[vi + 1].abs_byte_start != chunk_byte_end;

        let cursor_here = !found_cursor
            && byte_cursor >= chunk.abs_byte_start
            && if is_last_of_logical {
                byte_cursor <= chunk_byte_end
            } else {
                byte_cursor < chunk_byte_end
            };

        if cursor_here {
            found_cursor = true;
            cursor_visual_row = vi;
            let local_byte = byte_cursor - chunk.abs_byte_start;
            cursor_visual_col = chunk.text[..local_byte].chars().count();

            let before = &chunk.text[..local_byte];
            if local_byte < chunk.text.len() {
                let char_end = chunk.text[local_byte..]
                    .char_indices()
                    .nth(1)
                    .map(|(i, _)| local_byte + i)
                    .unwrap_or(chunk.text.len());
                let cursor_ch = &chunk.text[local_byte..char_end];
                let after = &chunk.text[char_end..];
                rendered_lines.push(Line::from(vec![
                    Span::styled(before, text_style),
                    Span::styled(cursor_ch, cursor_style),
                    Span::styled(after, text_style),
                ]));
            } else {
                // Cursor at end of chunk
                rendered_lines.push(Line::from(vec![
                    Span::styled(before, text_style),
                    Span::styled(" ", cursor_style),
                ]));
            }
        } else {
            rendered_lines.push(Line::from(Span::styled(chunk.text, text_style)));
        }
    }

    // Empty input: show just a blinking cursor
    if input.is_empty() {
        rendered_lines.push(Line::from(Span::styled(" ", cursor_style)));
    }

    let paragraph = Paragraph::new(rendered_lines).block(block);
    frame.render_widget(paragraph, area);

    // Set the real terminal cursor position for IME support.
    let inner = area.inner(Margin {
        vertical: 1,
        horizontal: 1,
    });
    let cx = inner.x + cursor_visual_col as u16;
    let cy = inner.y + cursor_visual_row as u16;
    if cx < inner.x + inner.width && cy < inner.y + inner.height {
        frame.set_cursor_position(Position::new(cx, cy));
    }

    // Agent autocomplete popup
    if app.agent_autocomplete_visible {
        let agents = app.filtered_agents();
        if !agents.is_empty() {
            let visible_count = agents.len().min(6) as u16;
            let popup_height = visible_count + 2;
            let popup_width = 40u16.min(area.width.saturating_sub(2));

            let popup_x = area.x + 1;
            let popup_y = area.y.saturating_sub(popup_height);

            let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);
            frame.render_widget(Clear, popup_area);

            let ac_block = Block::default()
                .title(" Agents ")
                .title_style(Style::default().fg(theme.accent))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .style(Style::default().bg(theme.bg));

            let items: Vec<ListItem> = agents
                .iter()
                .take(6)
                .map(|agent| {
                    let desc = agent.description.as_deref().unwrap_or("");
                    let truncated: String = desc.chars().take(30).collect();
                    ListItem::new(Line::from(vec![
                        Span::styled(
                            format!(" @{} ", agent.name),
                            Style::default().fg(theme.accent),
                        ),
                        Span::styled(truncated, Style::default().fg(theme.muted)),
                    ]))
                })
                .collect();

            let sel = app
                .agent_autocomplete_selected
                .min(items.len().saturating_sub(1));
            let mut state = ListState::default();
            state.select(Some(sel));

            let list = List::new(items).block(ac_block).highlight_style(
                Style::default()
                    .bg(theme.selection)
                    .add_modifier(Modifier::BOLD),
            );

            frame.render_stateful_widget(list, popup_area, &mut state);
        }
    }
}
