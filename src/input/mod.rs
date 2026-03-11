use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind};

use crate::app::state::{App, Dialog, Page};
use crate::config::Config;
use crate::ui::dialogs::models::filtered_models;
use crate::ui::themes::list_themes;

/// Actions that require async work in the event loop.
#[derive(Debug)]
pub enum Action {
    /// Send the current input text as a message.
    SendMessage,
    /// Abort the running operation on the current session.
    AbortSession,
    /// Create a new session and select it.
    CreateSession,
    /// Select a session by ID (from the session picker dialog).
    SelectSession(String),
    /// Reply to a pending permission request: (perm_id, session_id, response).
    /// Response must be "once", "always", or "reject".
    ReplyPermission(String, String, String),
    /// Reply to a pending question.
    ReplyQuestion(String, serde_json::Value),
    /// Quit the application.
    Quit,
    /// No action needed.
    None,
}

/// Process a key event in the context of the current application state.
///
/// Returns an [`Action`] that the event loop should execute asynchronously.
pub fn handle_key_event(app: &mut App, key: KeyEvent) -> Action {
    // Only handle key-press events, not release or repeat
    if key.kind != crossterm::event::KeyEventKind::Press {
        return Action::None;
    }

    // If a dialog is open, delegate to dialog-specific handling first
    if let Some(ref dialog) = app.active_dialog {
        return handle_dialog_key(app, key, dialog.clone());
    }

    // ── Global keybindings ──────────────────────────────────────────
    match (key.modifiers, key.code) {
        // Ctrl+C  => abort if busy and input is empty, otherwise clear input
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
            if app.is_session_busy() && app.input_text.is_empty() {
                return Action::AbortSession;
            }
            app.input_text.clear();
            app.input_cursor = 0;
            app.typed_visual_lines = 1;
            app.paste_line_count = None;
            return Action::None;
        }
        // Ctrl+N  => new session
        (KeyModifiers::CONTROL, KeyCode::Char('n')) => {
            return Action::CreateSession;
        }
        // Ctrl+O  => open session picker
        (KeyModifiers::CONTROL, KeyCode::Char('o')) => {
            app.open_dialog(Dialog::Sessions);
            return Action::None;
        }
        // Ctrl+K  => open model picker
        (KeyModifiers::CONTROL, KeyCode::Char('k')) => {
            app.open_dialog(Dialog::Models);
            return Action::None;
        }
        // Ctrl+P  => open command palette
        (KeyModifiers::CONTROL, KeyCode::Char('p')) => {
            app.open_dialog(Dialog::Commands);
            return Action::None;
        }
        // Ctrl+T  => open theme picker
        (KeyModifiers::CONTROL, KeyCode::Char('t')) => {
            app.open_dialog(Dialog::Themes);
            return Action::None;
        }
        // Ctrl+B  => toggle sidebar
        (KeyModifiers::CONTROL, KeyCode::Char('b')) => {
            app.toggle_sidebar();
            return Action::None;
        }
        // F1 or Ctrl+?  => help
        (_, KeyCode::F(1)) => {
            app.open_dialog(Dialog::Help);
            return Action::None;
        }
        // Ctrl+L  => toggle page between Chat and Logs
        (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
            app.active_page = match app.active_page {
                Page::Chat => Page::Logs,
                Page::Logs => Page::Chat,
            };
            return Action::None;
        }
        _ => {}
    }

    // ── Page-specific keybindings (no dialog open) ──────────────────
    match app.active_page {
        Page::Chat => handle_chat_key(app, key),
        Page::Logs => handle_logs_key(app, key),
    }
}

/// Handle pasted text (bracketed paste).
/// Multi-line paste enters "paste mode" — the editor shows "[Pasted X lines]"
/// and Enter sends the full text. Any other input clears the paste.
/// Single-line paste inserts inline as normal typed text.
pub fn handle_paste(app: &mut App, text: &str) {
    // Terminal paste uses \r for line breaks — normalize to \n.
    // Handle \r\n (Windows), \r (terminal/old Mac), and \n (Unix).
    let clean: String = text.replace("\r\n", "\n").replace('\r', "\n");
    let line_count = clean.lines().count().max(1);
    tracing::debug!(
        "handle_paste: {} chars, {} lines, first 80: {:?}",
        clean.len(),
        line_count,
        &clean[..clean.len().min(80)]
    );

    for c in clean.chars() {
        app.insert_char(c);
    }

    if line_count > 1 {
        app.paste_line_count = Some(line_count);
    } else {
        // Single-line paste: expand editor normally
        app.recalc_visual_lines();
    }
}

/// Process a mouse event (scroll wheel).
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            if app.active_dialog.is_some() {
                if app.dialog_selected > 0 {
                    app.dialog_selected -= 1;
                }
            } else {
                app.scroll_up();
            }
        }
        MouseEventKind::ScrollDown => {
            if app.active_dialog.is_some() {
                app.dialog_selected += 1;
            } else {
                app.scroll_down();
            }
        }
        _ => {}
    }
}

// ── Chat page input handling ────────────────────────────────────────────

fn handle_chat_key(app: &mut App, key: KeyEvent) -> Action {
    tracing::debug!(
        "chat key: code={:?} modifiers={:?} kind={:?}",
        key.code,
        key.modifiers,
        key.kind
    );

    // If agent autocomplete is visible, handle it first
    if app.agent_autocomplete_visible {
        return handle_agent_autocomplete_key(app, key);
    }

    // Paste mode: "[Pasted X lines]" is shown. Enter sends, any edit clears.
    if app.paste_line_count.is_some() {
        return match key.code {
            KeyCode::Enter => {
                app.paste_line_count = None;
                if !app.input_text.trim().is_empty() {
                    Action::SendMessage
                } else {
                    Action::None
                }
            }
            KeyCode::Esc => {
                app.open_dialog(Dialog::Quit);
                Action::None
            }
            _ => {
                // Any other key clears the paste
                app.input_text.clear();
                app.input_cursor = 0;
                app.typed_visual_lines = 1;
                app.paste_line_count = None;
                // If it was a printable char, insert it to start fresh
                if let KeyCode::Char(c) = key.code {
                    app.insert_char(c);
                    app.recalc_visual_lines();
                }
                Action::None
            }
        };
    }

    // Shift+Enter / Alt+Enter => insert newline.
    // iTerm2 sends 0x0A (\n) for Shift+Enter, which crossterm reports as Ctrl+J.
    if (key.code == KeyCode::Enter
        && (key.modifiers.contains(KeyModifiers::SHIFT)
            || key.modifiers.contains(KeyModifiers::ALT)))
        || (key.code == KeyCode::Char('j') && key.modifiers == KeyModifiers::CONTROL)
        || (key.code == KeyCode::Char('\n'))
    {
        app.insert_char('\n');
        app.recalc_visual_lines();
        return Action::None;
    }

    match key.code {
        // Enter => send message
        KeyCode::Enter => {
            if !app.input_text.trim().is_empty() {
                return Action::SendMessage;
            }
            Action::None
        }

        // Escape => quit
        KeyCode::Esc => {
            app.open_dialog(Dialog::Quit);
            Action::None
        }

        // Text input
        KeyCode::Char(c) => {
            app.insert_char(c);
            app.recalc_visual_lines();
            // Open agent autocomplete when '@' is typed at a word boundary
            if c == '@' && is_at_word_start(app) {
                app.agent_autocomplete_visible = true;
                app.agent_autocomplete_filter.clear();
                app.agent_autocomplete_selected = 0;
            }
            Action::None
        }

        // Editing keys
        KeyCode::Backspace => {
            app.delete_char();
            Action::None
        }
        KeyCode::Delete => {
            app.delete_char_forward();
            Action::None
        }
        KeyCode::Left => {
            app.move_cursor_left();
            Action::None
        }
        KeyCode::Right => {
            app.move_cursor_right();
            Action::None
        }
        KeyCode::Home => {
            app.move_cursor_home();
            Action::None
        }
        KeyCode::End => {
            app.move_cursor_end();
            Action::None
        }

        // Tab => toggle between Build and Plan mode
        KeyCode::Tab | KeyCode::BackTab => {
            app.chat_mode = app.chat_mode.toggle();
            Action::None
        }

        // Scrolling through messages
        KeyCode::Up | KeyCode::PageUp => {
            app.scroll_up();
            Action::None
        }
        KeyCode::Down | KeyCode::PageDown => {
            app.scroll_down();
            Action::None
        }

        _ => Action::None,
    }
}

// ── Logs page input handling ────────────────────────────────────────────

fn handle_logs_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.open_dialog(Dialog::Quit);
            Action::None
        }
        _ => Action::None,
    }
}

// ── Dialog input handling ───────────────────────────────────────────────

fn handle_dialog_key(app: &mut App, key: KeyEvent, dialog: Dialog) -> Action {
    match key.code {
        // Escape always closes the dialog
        KeyCode::Esc => {
            app.close_dialog();
            Action::None
        }

        _ => match dialog {
            Dialog::Sessions => handle_session_dialog_key(app, key),
            Dialog::Models => handle_model_dialog_key(app, key),
            Dialog::Commands => handle_filter_dialog_key(app, key),
            Dialog::Help => handle_simple_dialog_key(app, key),
            Dialog::Themes => handle_theme_dialog_key(app, key),
            Dialog::Permission => handle_permission_dialog_key(app, key),
            Dialog::Question => handle_question_dialog_key(app, key),
            Dialog::Quit => handle_quit_dialog_key(app, key),
        },
    }
}

// ── Session dialog ──────────────────────────────────────────────────────

fn handle_session_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            if app.dialog_selected > 0 {
                app.dialog_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            let max = app.sessions.len().saturating_sub(1);
            if app.dialog_selected < max {
                app.dialog_selected += 1;
            }
            Action::None
        }
        KeyCode::Enter => {
            if let Some(session) = app.sessions.get(app.dialog_selected) {
                let id = session.id.clone();
                app.close_dialog();
                Action::SelectSession(id)
            } else {
                Action::None
            }
        }
        KeyCode::Char(c) => {
            app.dialog_filter.push(c);
            app.dialog_selected = 0;
            Action::None
        }
        KeyCode::Backspace => {
            app.dialog_filter.pop();
            app.dialog_selected = 0;
            Action::None
        }
        _ => Action::None,
    }
}

// ── Filterable dialogs (Models, Commands, Themes, etc.) ─────────────────

fn handle_filter_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            if app.dialog_selected > 0 {
                app.dialog_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            app.dialog_selected += 1;
            Action::None
        }
        KeyCode::Enter => {
            // The specific dialog handler in the UI layer will read
            // dialog_selected to determine what was chosen.
            app.close_dialog();
            Action::None
        }
        KeyCode::Char(c) => {
            app.dialog_filter.push(c);
            app.dialog_selected = 0;
            Action::None
        }
        KeyCode::Backspace => {
            app.dialog_filter.pop();
            app.dialog_selected = 0;
            Action::None
        }
        _ => Action::None,
    }
}

// ── Theme dialog ────────────────────────────────────────────────────────

fn handle_theme_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            if app.dialog_selected > 0 {
                app.dialog_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            let max = list_themes().len().saturating_sub(1);
            if app.dialog_selected < max {
                app.dialog_selected += 1;
            }
            Action::None
        }
        KeyCode::Enter => {
            let themes = list_themes();
            if let Some(&name) = themes.get(app.dialog_selected) {
                app.theme_name = name.to_lowercase();
                // Persist the choice
                let mut config = Config::load();
                config.theme = Some(app.theme_name.clone());
                config.save();
            }
            app.close_dialog();
            Action::None
        }
        KeyCode::Char(c) => {
            app.dialog_filter.push(c);
            app.dialog_selected = 0;
            Action::None
        }
        KeyCode::Backspace => {
            app.dialog_filter.pop();
            app.dialog_selected = 0;
            Action::None
        }
        _ => Action::None,
    }
}

// ── Model dialog ────────────────────────────────────────────────────────

fn handle_model_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            if app.dialog_selected > 0 {
                app.dialog_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            let entries = filtered_models(app);
            let max = entries.len().saturating_sub(1);
            if app.dialog_selected < max {
                app.dialog_selected += 1;
            }
            Action::None
        }
        KeyCode::Enter => {
            let entries = filtered_models(app);
            if let Some(entry) = entries.get(app.dialog_selected) {
                app.current_model = Some((entry.provider_id.clone(), entry.model_id.clone()));
            }
            app.close_dialog();
            Action::None
        }
        KeyCode::Char(c) => {
            app.dialog_filter.push(c);
            app.dialog_selected = 0;
            Action::None
        }
        KeyCode::Backspace => {
            app.dialog_filter.pop();
            app.dialog_selected = 0;
            Action::None
        }
        _ => Action::None,
    }
}

// ── Simple dialog (Help, etc.) ──────────────────────────────────────────

fn handle_simple_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Enter | KeyCode::Char('q') => {
            app.close_dialog();
            Action::None
        }
        _ => Action::None,
    }
}

// ── Permission dialog ───────────────────────────────────────────────────

fn handle_permission_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    let response = match key.code {
        KeyCode::Char('y') => Some("once"),
        KeyCode::Char('a') => Some("always"),
        KeyCode::Char('n') | KeyCode::Char('r') => Some("reject"),
        _ => None,
    };

    if let Some(resp) = response {
        if let Some(perm) = app.pending_permissions.first() {
            let perm_id = perm.id.clone();
            let session_id = perm.session_id.clone();
            app.pending_permissions.remove(0);
            if app.pending_permissions.is_empty() {
                app.close_dialog();
            }
            Action::ReplyPermission(perm_id, session_id, resp.to_string())
        } else {
            app.close_dialog();
            Action::None
        }
    } else {
        Action::None
    }
}

// ── Question dialog ─────────────────────────────────────────────────────

fn handle_question_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Up => {
            if app.dialog_selected > 0 {
                app.dialog_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            app.dialog_selected += 1;
            Action::None
        }
        // Enter => submit the selected answer
        KeyCode::Enter => {
            if let Some(question) = app.pending_questions.first() {
                let q_id = question.id.clone();
                // Build the answer based on the selected option
                let answer = if let Some(ref options) = question.options {
                    if let Some(opt) = options.get(app.dialog_selected) {
                        serde_json::json!([opt.value])
                    } else {
                        serde_json::json!([])
                    }
                } else {
                    // Free-form: use the dialog filter as the answer
                    serde_json::json!([app.dialog_filter.clone()])
                };

                app.pending_questions.remove(0);
                if app.pending_questions.is_empty() {
                    app.close_dialog();
                }
                Action::ReplyQuestion(q_id, answer)
            } else {
                app.close_dialog();
                Action::None
            }
        }
        KeyCode::Char(c) => {
            app.dialog_filter.push(c);
            Action::None
        }
        KeyCode::Backspace => {
            app.dialog_filter.pop();
            Action::None
        }
        _ => Action::None,
    }
}

// ── Quit confirmation dialog ────────────────────────────────────────────

fn handle_quit_dialog_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.close_dialog();
            Action::Quit
        }
        KeyCode::Char('n') => {
            app.close_dialog();
            Action::None
        }
        _ => Action::None,
    }
}

// ── Agent autocomplete ─────────────────────────────────────────────────

/// Check if the '@' just inserted is at a word boundary.
fn is_at_word_start(app: &App) -> bool {
    let cursor = app.input_cursor;
    if cursor <= 1 {
        return true;
    }
    let byte_idx = app.cursor_byte_index();
    let text_before_at = &app.input_text[..byte_idx.saturating_sub(1)];
    matches!(text_before_at.chars().last(), Some(' ') | Some('\n') | None)
}

fn handle_agent_autocomplete_key(app: &mut App, key: KeyEvent) -> Action {
    match key.code {
        KeyCode::Esc => {
            app.agent_autocomplete_visible = false;
            Action::None
        }
        KeyCode::Up => {
            if app.agent_autocomplete_selected > 0 {
                app.agent_autocomplete_selected -= 1;
            }
            Action::None
        }
        KeyCode::Down => {
            let max = app.filtered_agents().len().saturating_sub(1);
            if app.agent_autocomplete_selected < max {
                app.agent_autocomplete_selected += 1;
            }
            Action::None
        }
        KeyCode::Enter | KeyCode::Tab => {
            let agents = app.filtered_agents();
            if let Some(agent) = agents.get(app.agent_autocomplete_selected) {
                let name = agent.name.clone();
                // Replace "@filter" with "@agentname "
                let byte_end = app.cursor_byte_index();
                let text_before = &app.input_text[..byte_end];
                if let Some(at_byte) = text_before.rfind('@') {
                    let replacement = format!("@{} ", name);
                    let new_cursor = app.input_text[..at_byte].chars().count() + name.len() + 2;
                    app.input_text.replace_range(at_byte..byte_end, &replacement);
                    app.input_cursor = new_cursor;
                }
            }
            app.agent_autocomplete_visible = false;
            Action::None
        }
        KeyCode::Backspace => {
            app.delete_char();
            update_autocomplete_filter(app);
            Action::None
        }
        KeyCode::Char(c) => {
            app.insert_char(c);
            app.recalc_visual_lines();
            update_autocomplete_filter(app);
            Action::None
        }
        _ => {
            app.agent_autocomplete_visible = false;
            Action::None
        }
    }
}

/// Update the autocomplete filter from the text between '@' and the cursor.
fn update_autocomplete_filter(app: &mut App) {
    let byte_idx = app.cursor_byte_index();
    let text_before = &app.input_text[..byte_idx];
    if let Some(at_pos) = text_before.rfind('@') {
        let filter = &text_before[at_pos + 1..];
        if filter.contains(' ') || filter.contains('\n') {
            app.agent_autocomplete_visible = false;
        } else {
            app.agent_autocomplete_filter = filter.to_string();
            app.agent_autocomplete_selected = 0;
        }
    } else {
        app.agent_autocomplete_visible = false;
    }
}
