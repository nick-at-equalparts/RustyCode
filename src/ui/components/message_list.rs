use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::app::state::App;
use crate::types::{Message, Part, ToolState};
use crate::ui::themes::{get_theme, Theme};

/// Render the scrollable message history inside `area`.
pub fn render(frame: &mut Frame, app: &App, area: Rect) {
    let theme = get_theme(&app.theme_name);

    let outer = Block::default()
        .borders(Borders::NONE)
        .style(Style::default().bg(theme.bg));
    frame.render_widget(outer, area);

    if app.messages.is_empty() {
        render_welcome(frame, app, area, &theme);
        return;
    }

    // Build all lines for every message, then apply scroll.
    let mut all_lines: Vec<Line> = Vec::new();

    for msg_with_parts in &app.messages {
        match &msg_with_parts.info {
            Message::User(user_msg) => {
                // Separator
                all_lines.push(Line::default());

                // Header
                all_lines.push(Line::from(vec![
                    Span::styled(
                        " > You ",
                        Style::default()
                            .fg(theme.user_msg)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        user_msg.time.created.to_string(),
                        Style::default().fg(theme.muted),
                    ),
                ]));

                // User message content comes from the parts
                for part in &msg_with_parts.parts {
                    if let Part::Text(tp) = part {
                        if let Some(ref text) = tp.text {
                            for line in text.lines() {
                                all_lines.push(Line::from(Span::styled(
                                    format!("   {}", line),
                                    Style::default().fg(theme.user_msg),
                                )));
                            }
                        }
                    }
                }
            }
            Message::Assistant(asst_msg) => {
                // Separator
                all_lines.push(Line::default());

                // Header
                let model_label = asst_msg
                    .model_id
                    .as_deref()
                    .unwrap_or("assistant");
                all_lines.push(Line::from(vec![
                    Span::styled(
                        format!(" {} ", model_label),
                        Style::default()
                            .fg(theme.accent)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        asst_msg.time.created.to_string(),
                        Style::default().fg(theme.muted),
                    ),
                ]));

                // Render each part
                for part in &msg_with_parts.parts {
                    render_part(part, &mut all_lines, theme);
                }

                // Cost / tokens footer
                if let (Some(tokens), Some(cost)) = (&asst_msg.tokens, asst_msg.cost) {
                    let input_t = tokens.input.unwrap_or(0);
                    let output_t = tokens.output.unwrap_or(0);
                    all_lines.push(Line::from(Span::styled(
                        format!(
                            "   tokens: {}in / {}out | cost: ${:.4}",
                            input_t, output_t, cost
                        ),
                        Style::default().fg(theme.muted),
                    )));
                }

                // Error
                if let Some(ref err) = asst_msg.error {
                    let err_text = err.as_str().map(|s| s.to_string())
                        .unwrap_or_else(|| err.to_string());
                    all_lines.push(Line::from(Span::styled(
                        format!("   Error: {}", err_text),
                        Style::default()
                            .fg(theme.error)
                            .add_modifier(Modifier::BOLD),
                    )));
                }
            }
        }
    }

    // Busy indicator
    if app.is_session_busy() {
        all_lines.push(Line::default());
        all_lines.push(Line::from(Span::styled(
            "   Thinking...",
            Style::default()
                .fg(theme.accent)
                .add_modifier(Modifier::ITALIC),
        )));
    }

    // ---- Scrolling ----
    // Compute the total wrapped height so we can pin to the bottom.
    // Each Line may wrap to ceil(line_width / area_width) visual rows.
    let view_width = area.width.max(1) as usize;
    let total_visual_rows: usize = all_lines
        .iter()
        .map(|line| {
            let w = line.width();
            if w == 0 { 1 } else { (w + view_width - 1) / view_width }
        })
        .sum();
    let visible_height = area.height as usize;

    // message_scroll is an offset from the bottom (0 = fully scrolled down).
    // Convert to a top-down row offset for Paragraph::scroll().
    let max_scroll = total_visual_rows.saturating_sub(visible_height);
    let scroll_from_top = max_scroll.saturating_sub(app.message_scroll);

    let paragraph = Paragraph::new(Text::from(all_lines))
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .wrap(Wrap { trim: false })
        .scroll((scroll_from_top as u16, 0));

    frame.render_widget(paragraph, area);

    // Scroll indicator when not at bottom
    if app.message_scroll > 0 {
        let indicator = Paragraph::new(Span::styled(
            format!(" +{} lines below ", app.message_scroll),
            Style::default()
                .fg(theme.warning)
                .add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Right);
        let indicator_area = Rect::new(area.x, area.y + area.height.saturating_sub(1), area.width, 1);
        frame.render_widget(indicator, indicator_area);
    }
}

/// Convert a single Part into styled lines.
fn render_part<'a>(part: &'a Part, lines: &mut Vec<Line<'a>>, theme: &'static crate::ui::themes::Theme) {
    match part {
        Part::Text(tp) => {
            if let Some(ref text) = tp.text {
                render_text_content(text, lines, theme);
            }
        }
        Part::Tool(tp) => {
            let name = tp.tool.as_deref().unwrap_or("tool");

            // Extract input from any state variant
            let input = match &tp.state {
                ToolState::Pending { input, .. }
                | ToolState::Running { input, .. }
                | ToolState::Completed { input, .. }
                | ToolState::Error { input, .. } => input.as_ref(),
            };

            // Get command string from input (e.g. bash command, file path, etc.)
            let command = input
                .and_then(|v| {
                    v.get("command")
                        .or_else(|| v.get("file_path"))
                        .or_else(|| v.get("path"))
                        .or_else(|| v.get("url"))
                        .or_else(|| v.get("pattern"))
                })
                .and_then(|v| v.as_str());

            let (icon, color) = match &tp.state {
                ToolState::Pending { .. } => ("...", theme.tool_pending),
                ToolState::Running { .. } => ("=>", theme.tool_running),
                ToolState::Completed { .. } => ("ok", theme.tool_complete),
                ToolState::Error { .. } => ("!!", theme.tool_error),
            };

            // First line: [icon] tool_name command_preview
            let mut spans = vec![
                Span::styled(
                    format!("   [{}] ", icon),
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    name.to_string(),
                    Style::default().fg(color),
                ),
            ];
            if let Some(cmd) = command {
                // Truncate long commands to first line, max ~60 chars
                let first_line = cmd.lines().next().unwrap_or(cmd);
                let preview = if first_line.len() > 60 {
                    format!(" {}...", &first_line[..57])
                } else {
                    format!(" {}", first_line)
                };
                spans.push(Span::styled(preview, Style::default().fg(theme.muted)));
            }
            lines.push(Line::from(spans));

            // Show tool output/title on completion
            if let ToolState::Completed { title, output, .. } = &tp.state {
                if let Some(title) = title {
                    lines.push(Line::from(Span::styled(
                        format!("       {}", title),
                        Style::default().fg(theme.muted),
                    )));
                }
                if let Some(output) = output {
                    // Show first few lines of output
                    for (i, line) in output.lines().enumerate() {
                        if i >= 3 {
                            lines.push(Line::from(Span::styled(
                                "       ... (truncated)".to_string(),
                                Style::default().fg(theme.muted),
                            )));
                            break;
                        }
                        lines.push(Line::from(Span::styled(
                            format!("       {}", line),
                            Style::default().fg(theme.muted),
                        )));
                    }
                }
            }

            // Show error on failure
            if let ToolState::Error { error, .. } = &tp.state {
                if let Some(err) = error {
                    lines.push(Line::from(Span::styled(
                        format!("       {}", err),
                        Style::default().fg(theme.tool_error),
                    )));
                }
            }
        }
        Part::Reasoning(rp) => {
            if let Some(ref content) = rp.content {
                if !content.is_empty() {
                    lines.push(Line::from(Span::styled(
                        "   (reasoning)",
                        Style::default()
                            .fg(theme.muted)
                            .add_modifier(Modifier::ITALIC),
                    )));
                    // Show first 2 lines, collapsed
                    for (i, line) in content.lines().enumerate() {
                        if i >= 2 {
                            lines.push(Line::from(Span::styled(
                                "   ...".to_string(),
                                Style::default()
                                    .fg(theme.muted)
                                    .add_modifier(Modifier::ITALIC),
                            )));
                            break;
                        }
                        lines.push(Line::from(Span::styled(
                            format!("   {}", line),
                            Style::default()
                                .fg(theme.muted)
                                .add_modifier(Modifier::ITALIC),
                        )));
                    }
                }
            }
        }
        Part::File(fp) => {
            let label = fp
                .file_path
                .as_deref()
                .unwrap_or("(unknown file)");
            lines.push(Line::from(vec![
                Span::styled(
                    "   [file] ",
                    Style::default().fg(theme.accent),
                ),
                Span::styled(
                    label.to_string(),
                    Style::default()
                        .fg(theme.fg)
                        .add_modifier(Modifier::UNDERLINED),
                ),
            ]));
        }
        Part::StepStart(sp) => {
            if let Some(ref title) = sp.title {
                lines.push(Line::from(Span::styled(
                    format!("   -- {} --", title),
                    Style::default().fg(theme.muted),
                )));
            }
        }
        Part::StepFinish(_) => {
            // StepFinish cost/tokens are rolled into the assistant message footer
        }
        Part::Agent(ap) => {
            let agent_name = ap.agent.as_deref().unwrap_or("subagent");
            lines.push(Line::from(vec![
                Span::styled(
                    "   [..] ",
                    Style::default()
                        .fg(theme.tool_running)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("Running agent: {}", agent_name),
                    Style::default()
                        .fg(theme.tool_running)
                        .add_modifier(Modifier::ITALIC),
                ),
            ]));
        }
        // Other part variants are rendered minimally
        Part::Snapshot(_) | Part::Patch(_) | Part::Retry(_) | Part::Compaction(_) | Part::Subtask(_) => {}
    }
}

/// Very basic markdown-ish rendering for text content.
/// Handles **bold**, `inline code`, and ```code blocks```.
fn render_text_content<'a>(content: &'a str, lines: &mut Vec<Line<'a>>, theme: &'static crate::ui::themes::Theme) {
    let mut in_code_block = false;

    for raw_line in content.lines() {
        if raw_line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            if in_code_block {
                lines.push(Line::from(Span::styled(
                    "   --------".to_string(),
                    Style::default().fg(theme.muted),
                )));
            } else {
                lines.push(Line::from(Span::styled(
                    "   --------".to_string(),
                    Style::default().fg(theme.muted),
                )));
            }
            continue;
        }

        if in_code_block {
            lines.push(Line::from(Span::styled(
                format!("   {}", raw_line),
                Style::default()
                    .fg(theme.warning)
                    .add_modifier(Modifier::DIM),
            )));
            continue;
        }

        // Simple inline formatting
        let spans = parse_inline_spans(raw_line, theme);
        let mut prefixed = vec![Span::raw("   ")];
        prefixed.extend(spans);
        lines.push(Line::from(prefixed));
    }
}

/// Parse a single line for **bold** and `code` spans.
fn parse_inline_spans<'a>(line: &'a str, theme: &'static crate::ui::themes::Theme) -> Vec<Span<'a>> {
    let mut spans: Vec<Span<'a>> = Vec::new();
    let mut remaining = line;

    while !remaining.is_empty() {
        // Find the next special marker
        if let Some(pos) = remaining.find("**") {
            if pos > 0 {
                spans.push(Span::styled(
                    &remaining[..pos],
                    Style::default().fg(theme.assistant_msg),
                ));
            }
            let after = &remaining[pos + 2..];
            if let Some(end) = after.find("**") {
                spans.push(Span::styled(
                    &after[..end],
                    Style::default()
                        .fg(theme.assistant_msg)
                        .add_modifier(Modifier::BOLD),
                ));
                remaining = &after[end + 2..];
            } else {
                // No closing **, render as-is
                spans.push(Span::styled(
                    &remaining[pos..],
                    Style::default().fg(theme.assistant_msg),
                ));
                break;
            }
        } else if let Some(pos) = remaining.find('`') {
            if pos > 0 {
                spans.push(Span::styled(
                    &remaining[..pos],
                    Style::default().fg(theme.assistant_msg),
                ));
            }
            let after = &remaining[pos + 1..];
            if let Some(end) = after.find('`') {
                spans.push(Span::styled(
                    &after[..end],
                    Style::default().fg(theme.warning).bg(theme.selection),
                ));
                remaining = &after[end + 1..];
            } else {
                spans.push(Span::styled(
                    &remaining[pos..],
                    Style::default().fg(theme.assistant_msg),
                ));
                break;
            }
        } else {
            spans.push(Span::styled(
                remaining,
                Style::default().fg(theme.assistant_msg),
            ));
            break;
        }
    }

    spans
}

/// Render a centered welcome / landing screen when no conversation is open.
fn render_welcome(frame: &mut Frame, app: &App, area: Rect, theme: &Theme) {
    let project_name = app
        .project
        .as_ref()
        .and_then(|p| p.path.as_deref())
        .and_then(|p| p.rsplit('/').next())
        .unwrap_or("rustycode");

    let key_style = Style::default().fg(theme.accent).bold();
    let desc_style = Style::default().fg(theme.muted);

    let mut lines: Vec<Line> = Vec::new();

    // Title
    lines.push(Line::from(Span::styled(
        project_name,
        Style::default().fg(theme.fg).bold(),
    )));
    lines.push(Line::default());

    // Shortcuts (must match actual bindings in input/mod.rs)
    let shortcuts: &[(&str, &str)] = &[
        ("Enter",       " Send message"),
        ("Shift+Enter", " New line"),
        ("Ctrl+O",      " Open sessions"),
        ("Ctrl+N",      " New session"),
        ("Ctrl+B",      " Toggle sidebar"),
        ("Ctrl+K",      " Switch model"),
        ("Ctrl+P",      " Command palette"),
        ("F1",          " Help"),
        ("Esc",         " Quit"),
    ];

    for &(k, d) in shortcuts {
        lines.push(Line::from(vec![
            Span::styled(k.to_string(), key_style),
            Span::styled(d.to_string(), desc_style),
        ]));
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "Type below to start a conversation.",
        desc_style,
    )));

    // Center vertically
    let content_height = lines.len() as u16;
    let top_pad = area.height.saturating_sub(content_height) / 2;

    let mut padded: Vec<Line> = vec![Line::default(); top_pad as usize];
    padded.extend(lines);

    let paragraph = Paragraph::new(padded)
        .alignment(Alignment::Center)
        .style(Style::default().bg(theme.bg));

    frame.render_widget(paragraph, area);
}
