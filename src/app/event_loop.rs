use anyhow::{Context, Result};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, EventStream, Event as CrosstermEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::io;
use tokio::time::{interval, Duration};

use crate::api::events::EventStream as SseEventStream;
use crate::input;

use super::state::App;

/// Run the main TUI event loop.
///
/// This sets up the terminal, connects to the SSE stream, and runs a
/// `tokio::select!` loop that handles keyboard input, server events,
/// and periodic UI refreshes.
pub async fn run(app: &mut App) -> Result<()> {
    // ── Terminal setup ──────────────────────────────────────────────
    enable_raw_mode().context("Failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
        .context("Failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;
    terminal.clear()?;

    // ── SSE event stream ────────────────────────────────────────────
    let sse_result = SseEventStream::connect(&app.base_url).await;
    let mut sse_stream = match sse_result {
        Ok(stream) => Some(stream),
        Err(e) => {
            tracing::warn!("Failed to connect to SSE stream: {}. Continuing without live events.", e);
            app.status_message = "No live event connection".to_string();
            None
        }
    };

    // ── Crossterm event stream ──────────────────────────────────────
    let mut crossterm_events = EventStream::new();

    // ── Tick timer for periodic redraws ─────────────────────────────
    let mut tick = interval(Duration::from_millis(250));

    // ── Initial render ──────────────────────────────────────────────
    terminal.draw(|frame| crate::ui::draw(frame, app))?;

    // ── Main loop ───────────────────────────────────────────────────
    loop {
        let sse_next = async {
            match &mut sse_stream {
                Some(stream) => stream.next().await,
                None => {
                    // If no SSE stream, never resolve this branch
                    std::future::pending().await
                }
            }
        };

        tokio::select! {
            // Crossterm keyboard/mouse events
            maybe_event = crossterm_events.next() => {
                if let Some(Ok(event)) = maybe_event {
                    if let CrosstermEvent::Mouse(mouse_event) = event {
                        input::handle_mouse_event(app, mouse_event);
                    }
                    if let CrosstermEvent::Key(key_event) = event {
                        let action = input::handle_key_event(app, key_event);

                        // Execute any async actions returned by the input handler
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
                                if let Err(e) = app.select_session(&id).await {
                                    app.status_message = format!("Select session failed: {}", e);
                                    tracing::error!("Failed to select session: {}", e);
                                }
                            }
                            input::Action::ReplyPermission(perm_id, session_id, response) => {
                                if let Err(e) = app.client.reply_permission(
                                    &session_id,
                                    &perm_id,
                                    &response,
                                    None,
                                ).await {
                                    app.status_message = format!("Permission reply failed: {}", e);
                                    tracing::error!("Failed to reply to permission: {}", e);
                                }
                            }
                            input::Action::ReplyQuestion(q_id, answer) => {
                                // reply_question(request_id, &answers_value)
                                if let Err(e) = app.client.reply_question(&q_id, &answer).await {
                                    app.status_message = format!("Question reply failed: {}", e);
                                    tracing::error!("Failed to reply to question: {}", e);
                                }
                            }
                            input::Action::Quit => {
                                break;
                            }
                            input::Action::None => {}
                        }

                        // Redraw after input
                        terminal.draw(|frame| crate::ui::draw(frame, app))?;
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
        DisableMouseCapture
    )
    .context("Failed to leave alternate screen")?;
    terminal.show_cursor().context("Failed to show cursor")?;

    Ok(())
}
