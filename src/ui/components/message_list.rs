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
        render_welcome(frame, app, area, theme);
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
                let model_label = asst_msg.model_id.as_deref().unwrap_or("assistant");
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
                    render_part(part, &mut all_lines, theme, app.tick_count);
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
                    let err_text = err
                        .as_str()
                        .map(|s| s.to_string())
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

    // ---- Scrolling ----
    let view_width = area.width.max(1);
    let visible_height = area.height as usize;

    // Performance: for large sessions, only keep lines near the viewport.
    // Ratatui layouts ALL lines even with .scroll(), so trimming from the top
    // avoids expensive word-wrap computation on thousands of off-screen lines.
    // Use a generous budget so scrolling up still works smoothly.
    let lines_budget = visible_height * 5 + app.message_scroll * 3 + 100;
    let skip = all_lines.len().saturating_sub(lines_budget);
    if skip > 0 {
        all_lines.drain(..skip);
    }

    // Use Paragraph::line_count() for the exact wrapped height.
    // Our old ceil(width/view_width) estimate was wrong because Ratatui's
    // word-wrapper breaks at word boundaries, producing more visual rows
    // than a simple character-width estimate. The error accumulated across
    // hundreds of lines and left the user stranded at the top.
    let paragraph = Paragraph::new(Text::from(all_lines))
        .style(Style::default().bg(theme.bg).fg(theme.fg))
        .wrap(Wrap { trim: false });

    let total_visual_rows = paragraph.line_count(view_width);
    let max_scroll = total_visual_rows.saturating_sub(visible_height);
    let scroll_from_top = max_scroll.saturating_sub(app.message_scroll);

    let paragraph = paragraph.scroll((scroll_from_top as u16, 0));
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
        let indicator_area = Rect::new(
            area.x,
            area.y + area.height.saturating_sub(1),
            area.width,
            1,
        );
        frame.render_widget(indicator, indicator_area);
    }
}

/// Spinner frames used for running tasks (cycles every 250ms tick).
const SPINNER_FRAMES: &[char] = &['◐', '◑', '◒', '◓'];

/// Build a box top-border line: `   ┌ ◐ title ─────────────`
fn box_top<'a>(icon: &str, title: &str, color: Color) -> Line<'a> {
    let fill_len = 40usize.saturating_sub(title.len() + icon.len() + 5);
    let fill: String = "─".repeat(fill_len);
    Line::from(vec![
        Span::styled("   ┌ ", Style::default().fg(color)),
        Span::styled(
            format!("{} ", icon),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            title.to_string(),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", fill), Style::default().fg(color)),
    ])
}

/// Build a box content line: `   │  text`
fn box_content<'a>(text: String, color: Color, text_color: Color) -> Line<'a> {
    Line::from(vec![
        Span::styled("   │  ", Style::default().fg(color)),
        Span::styled(text, Style::default().fg(text_color)),
    ])
}

/// Build a box bottom-border line: `   └──────────────────────`
fn box_bottom<'a>(color: Color) -> Line<'a> {
    Line::from(Span::styled(
        format!("   └{}", "─".repeat(40)),
        Style::default().fg(color),
    ))
}

/// Render a "task" tool call (sub-agent invocation) with a bordered box.
fn render_task_tool<'a>(
    tp: &'a crate::types::ToolPart,
    input: Option<&'a serde_json::Value>,
    lines: &mut Vec<Line<'a>>,
    theme: &'static Theme,
    tick_count: usize,
) {
    let is_done = matches!(tp.state, ToolState::Completed { .. });
    let is_error = matches!(tp.state, ToolState::Error { .. });

    let (icon, color) = if is_error {
        ("✗".to_string(), theme.tool_error)
    } else if is_done {
        ("✓".to_string(), theme.tool_complete)
    } else {
        let spinner = SPINNER_FRAMES[tick_count % SPINNER_FRAMES.len()];
        (format!("{}", spinner), theme.accent)
    };

    // Try to extract a human-readable description from the input
    let description = input
        .and_then(|v| {
            v.get("description")
                .or_else(|| v.get("input"))
                .or_else(|| v.get("command"))
                .or_else(|| v.get("prompt"))
        })
        .and_then(|v| v.as_str());

    let title = description.unwrap_or("Task");
    let title_preview = if title.len() > 50 {
        format!("{}...", &title[..47])
    } else {
        title.to_string()
    };

    lines.push(box_top(&icon, &title_preview, color));

    // Show task_id on the content line if available
    let task_id = input
        .and_then(|v| v.get("task_id"))
        .and_then(|v| v.as_str());
    if let Some(tid) = task_id {
        let id_preview = if tid.len() > 60 {
            format!("{}...", &tid[..57])
        } else {
            tid.to_string()
        };
        lines.push(box_content(id_preview, color, theme.muted));
    }

    // Show summary/output on completion
    if let ToolState::Completed { output, title, .. } = &tp.state {
        if let Some(t) = title {
            let preview = if t.len() > 60 {
                format!("{}...", &t[..57])
            } else {
                t.clone()
            };
            lines.push(box_content(preview, color, theme.muted));
        }
        if let Some(out) = output {
            // Show first 3 lines of output
            for (i, line) in out.lines().enumerate() {
                if i >= 3 {
                    lines.push(box_content(
                        "... (truncated)".to_string(),
                        color,
                        theme.muted,
                    ));
                    break;
                }
                let preview = if line.len() > 60 {
                    format!("{}...", &line[..57])
                } else {
                    line.to_string()
                };
                lines.push(box_content(preview, color, theme.muted));
            }
        }
    }

    // Show error
    if let ToolState::Error {
        error: Some(err), ..
    } = &tp.state
    {
        let preview = if err.len() > 60 {
            format!("{}...", &err[..57])
        } else {
            err.clone()
        };
        lines.push(box_content(preview, color, theme.tool_error));
    }

    lines.push(box_bottom(color));
}

/// Convert a single Part into styled lines.
fn render_part<'a>(
    part: &'a Part,
    lines: &mut Vec<Line<'a>>,
    theme: &'static crate::ui::themes::Theme,
    tick_count: usize,
) {
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

            // Sub-agent tasks (tool == "task") get special box rendering
            if name == "task" {
                render_task_tool(tp, input, lines, theme, tick_count);
                return;
            }

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
                Span::styled(name.to_string(), Style::default().fg(color)),
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
            if let ToolState::Error {
                error: Some(err), ..
            } = &tp.state
            {
                lines.push(Line::from(Span::styled(
                    format!("       {}", err),
                    Style::default().fg(theme.tool_error),
                )));
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
            let label = fp.file_path.as_deref().unwrap_or("(unknown file)");
            lines.push(Line::from(vec![
                Span::styled("   [file] ", Style::default().fg(theme.accent)),
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
                let fill_len = 40usize.saturating_sub(title.len() + 5);
                let fill: String = "─".repeat(fill_len);
                lines.push(Line::from(vec![
                    Span::styled("   ── ", Style::default().fg(theme.muted)),
                    Span::styled(
                        title.to_string(),
                        Style::default()
                            .fg(theme.muted)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(format!(" {}", fill), Style::default().fg(theme.muted)),
                ]));
            }
        }
        Part::StepFinish(_) => {
            // StepFinish cost/tokens are rolled into the assistant message footer
        }
        Part::Agent(ap) => {
            let agent_name = ap.agent.as_deref().unwrap_or("subagent");
            let spinner = SPINNER_FRAMES[tick_count % SPINNER_FRAMES.len()];
            let icon = format!("{}", spinner);
            let color = theme.accent;
            lines.push(box_top(&icon, agent_name, color));
            lines.push(box_bottom(color));
        }
        Part::Subtask(sp) => {
            let has_summary = sp.summary.is_some();
            let (icon, color) = if has_summary {
                ("✓".to_string(), theme.tool_complete)
            } else {
                let spinner = SPINNER_FRAMES[tick_count % SPINNER_FRAMES.len()];
                (format!("{}", spinner), theme.accent)
            };

            lines.push(box_top(&icon, "Task", color));

            // Show input description if available
            if let Some(ref input) = sp.input {
                let preview = if input.len() > 70 {
                    format!("{}...", &input[..67])
                } else {
                    input.clone()
                };
                lines.push(box_content(preview, color, theme.fg));
            }

            // Show summary on completion
            if let Some(ref summary) = sp.summary {
                let preview = if summary.len() > 70 {
                    format!("{}...", &summary[..67])
                } else {
                    summary.clone()
                };
                lines.push(box_content(preview, color, theme.muted));
            }

            lines.push(box_bottom(color));
        }
        // Other part variants are rendered minimally
        Part::Snapshot(_) | Part::Patch(_) | Part::Retry(_) | Part::Compaction(_) => {}
    }
}

/// Very basic markdown-ish rendering for text content.
/// Handles **bold**, `inline code`, and ```code blocks```.
fn render_text_content<'a>(
    content: &'a str,
    lines: &mut Vec<Line<'a>>,
    theme: &'static crate::ui::themes::Theme,
) {
    let mut in_code_block = false;

    for raw_line in content.lines() {
        if raw_line.trim_start().starts_with("```") {
            in_code_block = !in_code_block;
            lines.push(Line::from(Span::styled(
                "   --------".to_string(),
                Style::default().fg(theme.muted),
            )));
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
fn parse_inline_spans<'a>(
    line: &'a str,
    theme: &'static crate::ui::themes::Theme,
) -> Vec<Span<'a>> {
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
        ("Enter", " Send message"),
        ("Shift+Enter", " New line"),
        ("Ctrl+O", " Open sessions"),
        ("Ctrl+N", " New session"),
        ("Ctrl+B", " Toggle sidebar"),
        ("Ctrl+K", " Switch model"),
        ("Ctrl+P", " Command palette"),
        ("F1", " Help"),
        ("Esc", " Quit"),
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
