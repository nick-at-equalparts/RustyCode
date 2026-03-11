use anyhow::{Context, Result};
use crossterm::{
    event::{
        DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event as CrosstermEvent, EventStream, KeyCode, KeyEventKind, KeyModifiers,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::io;
use tokio::time::{interval, Duration, Instant};

use crate::api::events::EventStream as SseEventStream;
use crate::input;

use super::state::App;

/// Minimum number of rapid-fire key events to treat as a paste operation.
/// When bracketed paste isn't supported by the terminal (e.g. iTerm2), pasted
/// text arrives as individual key events in a rapid burst. A burst of this
/// many keys within a few milliseconds is definitely paste, not human typing.
const PASTE_BURST_THRESHOLD: usize = 5;

/// Sliding window: after each rapid key event, wait this long for the next one.
/// Single keypresses only see this delay once (~5ms, imperceptible).
/// Paste keeps extending the deadline as long as events keep arriving.
const PASTE_BURST_WINDOW: Duration = Duration::from_millis(5);

/// Absolute maximum time to spend collecting a burst, to prevent pathological
/// cases from blocking the event loop indefinitely.
const PASTE_BURST_MAX: Duration = Duration::from_millis(200);

/// Execute an action returned by the input handler.
/// Returns `true` if the application should quit.
async fn execute_action(app: &mut App, action: input::Action) -> Result<bool> {
    match action {
        input::Action::SendMessage => {
            if let Err(e) = app.send_message().await {
                app.status_message = format!("Send failed: {}", e);
                tracing::error!("Failed to send message: {}", e);
            }
        }
        input::Action::AbortSession => {
            if let Err(e) = app.abort_current().await {
                app.status_message = format!("Abort failed: {}", e);
                tracing::error!("Failed to abort: {}", e);
            }
        }
        input::Action::CreateSession => {
            if let Err(e) = app.create_new_session().await {
                app.status_message = format!("Create session failed: {}", e);
                tracing::error!("Failed to create session: {}", e);
            }
        }
        input::Action::SelectSession(id) => {
            app.select_session(&id);
        }
        input::Action::ReplyPermission(perm_id, session_id, response) => {
            if let Err(e) = app
                .client
                .reply_permission(&session_id, &perm_id, &response, None)
                .await
            {
                app.status_message = format!("Permission reply failed: {}", e);
                tracing::error!("Failed to reply to permission: {}", e);
            }
        }
        input::Action::ReplyQuestion(q_id, answer) => {
            if let Err(e) = app.client.reply_question(&q_id, &answer).await {
                app.status_message = format!("Question reply failed: {}", e);
                tracing::error!("Failed to reply to question: {}", e);
            }
        }
        input::Action::Quit => {
            return Ok(true);
        }
        input::Action::None => {}
    }
    Ok(false)
}

/// Build a paste string from a burst of key events.
///
/// Maps printable characters directly, and various newline representations
/// (Enter, Ctrl+J, literal \r and \n) to `\n`. Ignores everything else.
pub(crate) fn key_burst_to_paste(keys: &[crossterm::event::KeyEvent]) -> String {
    let mut text = String::with_capacity(keys.len());
    for k in keys {
        match (k.modifiers, k.code) {
            // Ctrl+J is how iTerm2 encodes \n (0x0A)
            (m, KeyCode::Char('j')) if m.contains(KeyModifiers::CONTROL) => text.push('\n'),
            // Ctrl+M is \r (0x0D) — also a newline in paste context
            (m, KeyCode::Char('m')) if m.contains(KeyModifiers::CONTROL) => text.push('\n'),
            (_, KeyCode::Char('\n')) => text.push('\n'),
            (_, KeyCode::Char('\r')) => text.push('\n'),
            (_, KeyCode::Char(c)) => text.push(c),
            (_, KeyCode::Enter) => text.push('\n'),
            _ => {
                tracing::trace!(
                    "key_burst_to_paste: skipping {:?} {:?}",
                    k.modifiers,
                    k.code
                );
            }
        }
    }
    text
}

/// Run the main TUI event loop.
///
/// This sets up the terminal, connects to the SSE stream, and runs a
/// `tokio::select!` loop that handles keyboard input, server events,
/// and periodic UI refreshes.
pub async fn run(app: &mut App) -> Result<()> {
    // ── Terminal setup ──────────────────────────────────────────────
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )
    .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    terminal.clear()?;

    // ── Crossterm event stream ──────────────────────────────────────
    let mut crossterm_events = EventStream::new();

    // ── Tick timer for periodic redraws ─────────────────────────────
    let mut tick = interval(Duration::from_millis(250));

    // ── Initial render (UI appears instantly, data loads after) ─────
    app.status_message = "Loading...".to_string();
    app.last_terminal_width = terminal.size()?.width;
    terminal.draw(|frame| crate::ui::draw(frame, app))?;

    // ── Load initial data + connect SSE (after first render) ─────────
    if let Err(e) = app.load_initial_data().await {
        tracing::warn!("Failed to load initial data: {}", e);
        app.status_message = format!("Partial load: {}", e);
    }
    terminal.draw(|frame| crate::ui::draw(frame, app))?;

    let sse_result = SseEventStream::connect(&app.base_url).await;
    let mut sse_stream = match sse_result {
        Ok(stream) => Some(stream),
        Err(e) => {
            tracing::warn!(
                "Failed to connect to SSE stream: {}. Continuing without live events.",
                e
            );
            app.status_message = "No live event connection".to_string();
            None
        }
    };

    // ── Main loop ───────────────────────────────────────────────────
    loop {
        let sse_next = async {
            match &mut sse_stream {
                Some(stream) => stream.next().await,
                None => std::future::pending().await,
            }
        };

        let session_load = async {
            match &mut app.session_load_rx {
                Some(rx) => rx.await.ok(),
                None => std::future::pending().await,
            }
        };

        let backfill = async {
            match &mut app.backfill_rx {
                Some(rx) => rx.await.ok(),
                None => std::future::pending().await,
            }
        };

        tokio::select! {
            // Crossterm keyboard/mouse events
            maybe_event = crossterm_events.next() => {
                if let Some(Ok(event)) = maybe_event {
                    match event {
                        CrosstermEvent::Paste(text) => {
                            tracing::debug!("Paste event: {} bytes", text.len());
                            input::handle_paste(app, &text);
                            terminal.draw(|frame| crate::ui::draw(frame, app))?;
                        }
                        CrosstermEvent::Mouse(mouse_event) => {
                            input::handle_mouse_event(app, mouse_event);
                            terminal.draw(|frame| crate::ui::draw(frame, app))?;
                        }
                        CrosstermEvent::Resize(w, _) => {
                            app.last_terminal_width = w;
                            terminal.draw(|frame| crate::ui::draw(frame, app))?;
                        }
                        CrosstermEvent::Key(first_key) => {
                            app.last_terminal_width = terminal.size()?.width;

                            // ── Paste burst detection ──────────────────────
                            // When bracketed paste is unsupported (e.g. iTerm2),
                            // pasted text arrives as individual key events in a
                            // rapid burst. Use a sliding window: after each event
                            // extend the deadline, so long pastes are fully
                            // captured while single keypresses get ~5ms delay.
                            let mut key_burst = vec![first_key];
                            let max_deadline = Instant::now() + PASTE_BURST_MAX;
                            let mut deadline = Instant::now() + PASTE_BURST_WINDOW;

                            loop {
                                let effective = deadline.min(max_deadline);
                                match tokio::time::timeout_at(effective, crossterm_events.next()).await {
                                    Ok(Some(Ok(CrosstermEvent::Key(k)))) if k.kind == KeyEventKind::Press => {
                                        key_burst.push(k);
                                        // Slide the window forward with each event
                                        deadline = Instant::now() + PASTE_BURST_WINDOW;
                                    }
                                    Ok(Some(Ok(CrosstermEvent::Key(_)))) => {
                                        // Non-press (release/repeat) — skip but extend
                                        deadline = Instant::now() + PASTE_BURST_WINDOW;
                                    }
                                    _ => break, // Timeout, non-key event, error, or stream end
                                }
                            }

                            if key_burst.len() >= PASTE_BURST_THRESHOLD {
                                // Treat the burst as a paste operation
                                let paste_text = key_burst_to_paste(&key_burst);
                                let newline_count = paste_text.chars().filter(|&c| c == '\n').count();
                                tracing::debug!(
                                    "Burst paste detected: {} keys → {} chars, {} newlines, first 80: {:?}",
                                    key_burst.len(),
                                    paste_text.len(),
                                    newline_count,
                                    &paste_text[..paste_text.len().min(80)]
                                );
                                // Log the first few raw key events for debugging
                                for (i, k) in key_burst.iter().take(10).enumerate() {
                                    tracing::debug!(
                                        "  burst key[{}]: code={:?} mods={:?}",
                                        i, k.code, k.modifiers
                                    );
                                }
                                input::handle_paste(app, &paste_text);
                            } else {
                                // Normal key-by-key processing
                                for k in key_burst {
                                    let action = input::handle_key_event(app, k);
                                    if execute_action(app, action).await? {
                                        app.should_quit = true;
                                        break;
                                    }
                                }
                            }

                            // Redraw after input
                            terminal.draw(|frame| crate::ui::draw(frame, app))?;
                        }
                        _ => {}
                    }
                }
            }

            // SSE events from the server
            maybe_sse = sse_next => {
                match maybe_sse {
                    Some(Ok(event)) => {
                        app.handle_event(event);
                        // Redraw after server event
                        terminal.draw(|frame| crate::ui::draw(frame, app))?;
                    }
                    Some(Err(e)) => {
                        tracing::warn!("SSE event error: {}", e);
                        // The stream may have disconnected; try to continue
                    }
                    None => {
                        // SSE stream ended — attempt reconnection
                        tracing::warn!("SSE stream ended, attempting reconnect...");
                        app.status_message = "Reconnecting...".to_string();
                        match SseEventStream::connect(&app.base_url).await {
                            Ok(new_stream) => {
                                sse_stream = Some(new_stream);
                                app.status_message = String::new();
                                app.connected = true;
                            }
                            Err(e) => {
                                tracing::error!("SSE reconnect failed: {}", e);
                                app.status_message = "Disconnected from server".to_string();
                                app.connected = false;
                                sse_stream = None;
                            }
                        }
                    }
                }
            }

            // Background session load completed (fast initial batch)
            Some(result) = session_load => {
                app.session_load_rx = None;
                app.apply_session_load(result);
                terminal.draw(|frame| crate::ui::draw(frame, app))?;
            }

            // Background full-history backfill completed
            Some(result) = backfill => {
                app.backfill_rx = None;
                app.apply_backfill(result);
                // No redraw needed — user is already viewing recent messages.
                // The extra history is silently available when they scroll up.
            }

            // Tick timer for periodic UI updates
            _ = tick.tick() => {
                // Clear stale toasts
                app.tick_toast();
                // Periodic redraw
                terminal.draw(|frame| crate::ui::draw(frame, app))?;
            }
        }

        // Check quit flag (set by event handler or elsewhere)
        if app.should_quit {
            break;
        }
    }

    // ── Terminal teardown ────────────────────────────────────────────
    disable_raw_mode().context("Failed to disable raw mode")?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    Ok(())
}
