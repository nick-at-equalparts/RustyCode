use std::collections::HashMap;
use std::time::Instant;

use anyhow::Result;
use tokio::sync::oneshot;

use crate::api::client::{ModelSelector, PartInput, SendMessageRequest};
use crate::api::ApiClient;
use crate::types::{
    Agent, Command, Event, MessagePartDeltaProps, MessageWithParts, Part, PermissionRequest,
    Project, Provider, QuestionRequest, Session, SessionStatus, Toast, Todo, TuiToastProps,
};

/// Result of a background session load (messages + todos).
pub type SessionLoadResult = (
    Result<Vec<MessageWithParts>>,
    Result<Vec<Todo>>,
);

/// Number of recent messages to fetch on session open (fast initial load).
/// The server supports `?limit=N` and returns the N most recent messages.
const INITIAL_MESSAGE_LIMIT: u32 = 100;

/// Result of a background full-history backfill.
pub type BackfillResult = Result<Vec<MessageWithParts>>;

// ── UI enums ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Page {
    Chat,
    Logs,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    Sessions,
    Models,
    Commands,
    Help,
    Themes,
    Permission,
    Question,
    Quit,
}

// ── App ─────────────────────────────────────────────────────────────────

pub struct App {
    // Connection
    pub client: ApiClient,
    pub base_url: String,
    pub connected: bool,

    // Sessions
    pub sessions: Vec<Session>,
    pub current_session: Option<Session>,
    pub session_statuses: HashMap<String, SessionStatus>,

    // Messages
    pub messages: Vec<MessageWithParts>,
    pub message_scroll: usize,

    // Providers / Models
    pub providers: Vec<Provider>,
    pub current_model: Option<(String, String)>, // (providerID, modelID)

    // Agents / Commands
    pub agents: Vec<Agent>,
    pub commands: Vec<Command>,

    // Project
    pub project: Option<Project>,
    pub todos: Vec<Todo>,

    // Pending interactions
    pub pending_permissions: Vec<PermissionRequest>,
    pub pending_questions: Vec<QuestionRequest>,

    // UI State
    pub active_page: Page,
    pub active_dialog: Option<Dialog>,
    pub input_text: String,
    pub input_cursor: usize,
    pub sidebar_visible: bool,
    pub sidebar_selected: usize,

    // Dialog filter state
    pub dialog_filter: String,
    pub dialog_selected: usize,

    // App status
    pub should_quit: bool,
    pub toast: Option<Toast>,
    pub toast_time: Option<Instant>,
    pub status_message: String,
    pub is_busy: bool,

    // Theme
    pub theme_name: String,

    // Editor sizing — visual line count from typed input (not paste)
    pub typed_visual_lines: u16,
    pub last_terminal_width: u16,

    // Paste mode: when Some, editor shows "[Pasted X lines]" instead of raw text.
    // Enter sends, any other input clears.
    pub paste_line_count: Option<usize>,

    // Agent autocomplete
    pub agent_autocomplete_visible: bool,
    pub agent_autocomplete_selected: usize,
    pub agent_autocomplete_filter: String,

    // Background session load (messages + todos arrive asynchronously)
    pub session_load_rx: Option<oneshot::Receiver<SessionLoadResult>>,
    // Background full-history backfill (fires after initial fast load)
    pub backfill_rx: Option<oneshot::Receiver<BackfillResult>>,
}

impl App {
    /// Create a new App with the given API client and base URL.
    pub fn new(client: ApiClient, base_url: String) -> Self {
        Self {
            client,
            base_url,
            connected: false,

            sessions: Vec::new(),
            current_session: None,
            session_statuses: HashMap::new(),
            messages: Vec::new(),
            message_scroll: 0,
            providers: Vec::new(),
            current_model: None,
            agents: Vec::new(),
            commands: Vec::new(),
            project: None,
            todos: Vec::new(),

            pending_permissions: Vec::new(),
            pending_questions: Vec::new(),

            active_page: Page::Chat,
            active_dialog: None,
            input_text: String::new(),
            input_cursor: 0,
            sidebar_visible: false,
            sidebar_selected: 0,

            dialog_filter: String::new(),
            dialog_selected: 0,

            should_quit: false,
            toast: None,
            toast_time: None,
            status_message: String::new(),
            is_busy: false,

            theme_name: "default".to_string(),

            typed_visual_lines: 1,
            last_terminal_width: 80,

            paste_line_count: None,

            agent_autocomplete_visible: false,
            agent_autocomplete_selected: 0,
            agent_autocomplete_filter: String::new(),

            session_load_rx: None,
            backfill_rx: None,
        }
    }

    // ── Data loading ────────────────────────────────────────────────

    /// Fetch initial data from the server: sessions, providers, agents,
    /// commands, project, and VCS info.
    ///
    /// All requests fire in a single parallel wave. No conversation is
    /// auto-selected — the UI shows a welcome screen until the user picks one.
    pub async fn load_initial_data(&mut self) -> Result<()> {
        self.status_message = "Loading...".to_string();

        // Everything in one parallel wave (sessions listing works without project ID)
        let (project_res, sessions_res, providers_res, agents_res, commands_res) = tokio::join!(
            self.client.get_current_project(),
            self.client.list_sessions(None),
            self.client.list_providers(),
            self.client.list_agents(),
            self.client.list_commands(),
        );

        match project_res {
            Ok(project) => self.project = Some(project),
            Err(e) => tracing::warn!("Failed to load project: {}", e),
        }
        match sessions_res {
            Ok(sessions) => self.sessions = sessions,
            Err(e) => tracing::warn!("Failed to load sessions: {}", e),
        }
        match providers_res {
            Ok(response) => self.providers = response.all,
            Err(e) => tracing::warn!("Failed to load providers: {}", e),
        }
        match agents_res {
            Ok(agents) => self.agents = agents,
            Err(e) => tracing::warn!("Failed to load agents: {}", e),
        }
        match commands_res {
            Ok(commands) => self.commands = commands,
            Err(e) => tracing::warn!("Failed to load commands: {}", e),
        }

        self.connected = true;
        self.status_message = String::new();
        Ok(())
    }

    /// Select a session by ID.
    ///
    /// Updates the UI immediately (clears messages, shows "Loading…") and
    /// spawns a background task to fetch messages + todos. The event loop
    /// polls `session_load_rx` and applies the results when they arrive.
    pub fn select_session(&mut self, session_id: &str) {
        // Set current session from the list (or just store the ID)
        if let Some(s) = self.sessions.iter().find(|s| s.id == session_id) {
            self.current_session = Some(s.clone());
        }

        // Clear stale data so the UI shows a clean loading state
        self.messages.clear();
        self.message_scroll = 0;
        self.todos.clear();
        self.status_message = "Loading session…".to_string();

        // Spawn background fetch: grab the last INITIAL_MESSAGE_LIMIT messages
        // (fast), then backfill the full history.
        let (tx, rx) = oneshot::channel();
        let client = self.client.clone();
        let sid = session_id.to_string();
        tokio::spawn(async move {
            let t0 = Instant::now();
            let result = tokio::join!(
                client.list_messages(&sid, Some(INITIAL_MESSAGE_LIMIT)),
                client.get_session_todos(&sid),
            );
            tracing::info!(
                "Session load took {:.1}s (messages: {}, todos: {})",
                t0.elapsed().as_secs_f64(),
                match &result.0 {
                    Ok(m) => format!("{} msgs", m.len()),
                    Err(e) => format!("err: {}", e),
                },
                match &result.1 {
                    Ok(t) => format!("{} todos", t.len()),
                    Err(e) => format!("err: {}", e),
                },
            );
            let _ = tx.send(result);
        });
        self.session_load_rx = Some(rx);
    }

    /// Apply the result of a background session load.
    ///
    /// If the initial load returned exactly INITIAL_MESSAGE_LIMIT messages,
    /// there are probably more — kick off a background full-history fetch.
    pub fn apply_session_load(&mut self, result: SessionLoadResult) {
        let (messages_res, todos_res) = result;
        let mut need_backfill = false;
        match messages_res {
            Ok(messages) => {
                need_backfill = messages.len() as u32 >= INITIAL_MESSAGE_LIMIT;
                self.messages = messages;
                self.message_scroll = 0;
            }
            Err(e) => {
                tracing::warn!("Failed to load messages: {}", e);
            }
        }
        match todos_res {
            Ok(todos) => self.todos = todos,
            Err(e) => {
                tracing::debug!("Failed to load todos: {}", e);
            }
        }
        self.status_message = String::new();

        // Kick off full-history backfill if we likely have more messages
        if need_backfill {
            if let Some(session) = &self.current_session {
                let (tx, rx) = oneshot::channel();
                let client = self.client.clone();
                let sid = session.id.clone();
                tokio::spawn(async move {
                    let t0 = Instant::now();
                    let result = client.list_messages(&sid, None).await;
                    tracing::info!(
                        "Backfill took {:.1}s ({})",
                        t0.elapsed().as_secs_f64(),
                        match &result {
                            Ok(m) => format!("{} msgs", m.len()),
                            Err(e) => format!("err: {}", e),
                        },
                    );
                    let _ = tx.send(result);
                });
                self.backfill_rx = Some(rx);
            }
        }
    }

    /// Apply the full-history backfill, preserving the user's scroll position.
    pub fn apply_backfill(&mut self, result: BackfillResult) {
        match result {
            Ok(all_messages) => {
                // The user is looking at the newest messages (from the initial load).
                // Prepend the older history so the scroll position stays stable.
                let old_len = self.messages.len();
                self.messages = all_messages;
                let new_len = self.messages.len();
                // Shift scroll to compensate for prepended messages
                let prepended = new_len.saturating_sub(old_len);
                self.message_scroll += prepended;
                tracing::debug!(
                    "Backfill applied: {} → {} messages, scroll adjusted by {}",
                    old_len, new_len, prepended
                );
            }
            Err(e) => {
                tracing::warn!("Backfill failed: {}", e);
            }
        }
    }

    /// Create a new session and select it.
    pub async fn create_new_session(&mut self) -> Result<()> {
        let session = self.client.create_session(None, None).await?;
        let session_id = session.id.clone();
        self.sessions.insert(0, session);
        self.select_session(&session_id);
        Ok(())
    }

    /// Send the current input text as a message to the current session.
    pub async fn send_message(&mut self) -> Result<()> {
        let text = self.input_text.trim().to_string();
        if text.is_empty() {
            return Ok(());
        }

        // Extract @agent prefix if present
        let (agent, message_text) = if text.starts_with('@') {
            if let Some(space_idx) = text.find(' ') {
                let name = &text[1..space_idx];
                if self.agents.iter().any(|a| a.name == name) {
                    (Some(name.to_string()), text[space_idx..].trim().to_string())
                } else {
                    (None, text)
                }
            } else {
                (None, text)
            }
        } else {
            (None, text)
        };

        if message_text.is_empty() {
            return Ok(());
        }

        let session_id = match &self.current_session {
            Some(s) => s.id.clone(),
            None => {
                // Create a new session if none exists
                self.create_new_session().await?;
                match &self.current_session {
                    Some(s) => s.id.clone(),
                    None => anyhow::bail!("Failed to create session"),
                }
            }
        };

        let model_selector = self.current_model.as_ref().map(|(p, m)| ModelSelector {
            provider_id: p.clone(),
            model_id: m.clone(),
        });

        let request = SendMessageRequest {
            parts: vec![PartInput::Text {
                id: None,
                text: message_text,
                synthetic: None,
                ignored: None,
            }],
            message_id: None,
            model: model_selector,
            agent,
            no_reply: None,
            system: None,
            variant: None,
            format: None,
        };

        self.client
            .send_prompt_async(&session_id, &request)
            .await?;

        // Clear input after successful send
        self.input_text.clear();
        self.input_cursor = 0;
        self.typed_visual_lines = 1;
        self.paste_line_count = None;
        self.is_busy = true;

        Ok(())
    }

    /// Abort the current session's running operation.
    pub async fn abort_current(&mut self) -> Result<()> {
        if let Some(session) = &self.current_session {
            let session_id = session.id.clone();
            let _aborted = self.client.abort_session(&session_id).await?;
            self.is_busy = false;
        }
        Ok(())
    }

    // ── SSE event handling ──────────────────────────────────────────

    /// Process an SSE event from the server, updating application state.
    pub fn handle_event(&mut self, event: Event) {
        match event {
            // ── Session lifecycle ────────────────────────────────────
            Event::SessionCreated { properties } => {
                let session = properties.info;
                if !self.sessions.iter().any(|s| s.id == session.id) {
                    self.sessions.insert(0, session);
                }
            }
            Event::SessionUpdated { properties } => {
                let session = properties.info;
                if let Some(existing) = self.sessions.iter_mut().find(|s| s.id == session.id) {
                    *existing = session.clone();
                }
                if let Some(current) = &self.current_session {
                    if current.id == session.id {
                        self.current_session = Some(session);
                    }
                }
            }
            Event::SessionDeleted { properties } => {
                let session = properties.info;
                self.sessions.retain(|s| s.id != session.id);
                if let Some(current) = &self.current_session {
                    if current.id == session.id {
                        self.current_session = None;
                        self.messages.clear();
                    }
                }
            }
            Event::SessionCompacted { properties } => {
                let session_id = properties.session_id;
                if let Some(current) = &self.current_session {
                    if current.id == session_id {
                        // Messages may have been compacted; a reload may be needed
                        tracing::debug!("Session {} compacted", session_id);
                    }
                }
            }

            // ── Session status ──────────────────────────────────────
            Event::SessionStatus { properties } => {
                self.session_statuses
                    .insert(properties.session_id.clone(), properties.status);
                if let Some(current) = &self.current_session {
                    if current.id == properties.session_id {
                        self.is_busy = matches!(
                            self.session_statuses.get(&properties.session_id),
                            Some(SessionStatus::Busy)
                        );
                    }
                }
            }
            Event::SessionIdle { properties } => {
                self.session_statuses
                    .insert(properties.session_id.clone(), SessionStatus::Idle);
                if let Some(current) = &self.current_session {
                    if current.id == properties.session_id {
                        self.is_busy = false;
                    }
                }
            }
            Event::SessionError { properties } => {
                if let Some(session_id) = &properties.session_id {
                    self.session_statuses
                        .insert(session_id.clone(), SessionStatus::Idle);
                    if let Some(current) = &self.current_session {
                        if &current.id == session_id {
                            self.is_busy = false;
                            if let Some(err) = &properties.error {
                                self.status_message = format!(
                                    "Error: {}",
                                    serde_json::to_string(err).unwrap_or_default()
                                );
                            }
                        }
                    }
                }
            }

            // ── Message updates ─────────────────────────────────────
            Event::MessageUpdated { properties } => {
                self.apply_message_update(&properties);
            }
            Event::MessageRemoved { properties } => {
                if let Some(current) = &self.current_session {
                    if current.id == properties.session_id {
                        self.messages
                            .retain(|m| message_id(&m.info) != properties.message_id);
                    }
                }
            }

            // ── Part updates ────────────────────────────────────────
            Event::MessagePartUpdated { properties } => {
                self.apply_part_update(&properties.part);
            }
            Event::MessagePartDelta { properties } => {
                self.apply_message_delta(&properties);
            }
            Event::MessagePartRemoved { properties } => {
                if let Some(current) = &self.current_session {
                    if current.id == properties.session_id {
                        for msg in &mut self.messages {
                            if message_id(&msg.info) == properties.message_id {
                                msg.parts.retain(|p| part_id(p) != properties.part_id);
                                break;
                            }
                        }
                    }
                }
            }

            // ── Permissions ─────────────────────────────────────────
            Event::PermissionAsked { properties } => {
                let request = properties.request;
                if let Some(current) = &self.current_session {
                    if current.id == request.session_id {
                        self.pending_permissions.push(request);
                        if self.active_dialog.is_none() {
                            self.active_dialog = Some(Dialog::Permission);
                        }
                    }
                }
            }
            Event::PermissionReplied { properties } => {
                self.pending_permissions
                    .retain(|p| p.id != properties.request_id);
                if self.pending_permissions.is_empty()
                    && self.active_dialog == Some(Dialog::Permission)
                {
                    self.active_dialog = None;
                }
            }

            // ── Questions ───────────────────────────────────────────
            Event::QuestionAsked { properties } => {
                let question = properties.question;
                if let Some(current) = &self.current_session {
                    if current.id == question.session_id {
                        // Wrap the QuestionInfo into a QuestionRequest
                        let qr = QuestionRequest {
                            id: question.id,
                            session_id: question.session_id,
                            question: question.question,
                            options: question.options,
                            multi_select: question.multi_select,
                        };
                        self.pending_questions.push(qr);
                        if self.active_dialog.is_none() {
                            self.active_dialog = Some(Dialog::Question);
                        }
                    }
                }
            }
            Event::QuestionReplied { .. } | Event::QuestionRejected { .. } => {
                // Questions are resolved server-side
            }

            // ── Todos ───────────────────────────────────────────────
            Event::TodoUpdated { properties } => {
                if let Some(current) = &self.current_session {
                    if current.id == properties.session_id {
                        self.todos = properties.todos.unwrap_or_default();
                    }
                }
            }

            // ── Project / VCS ───────────────────────────────────────
            Event::ProjectUpdated { properties } => {
                self.project = Some(properties.info);
            }
            Event::VcsBranchUpdated { properties } => {
                // Store VCS branch on the project if available
                if let Some(project) = &mut self.project {
                    let branch_val = properties
                        .branch
                        .map(serde_json::Value::String)
                        .unwrap_or(serde_json::Value::Null);
                    project.vcs = Some(serde_json::json!({ "branch": branch_val }));
                }
            }

            // ── TUI control events ──────────────────────────────────
            Event::TuiToastShow { properties } => {
                self.show_toast(properties);
            }
            Event::TuiPromptAppend { properties } => {
                if let Some(text) = properties.text {
                    self.input_text.push_str(&text);
                    self.input_cursor = self.input_text.chars().count();
                }
            }
            Event::TuiSessionSelect { properties } => {
                if let Some(session_id) = properties.session_id {
                    // The caller should handle this async; store it for the event loop
                    tracing::debug!("TUI session select: {}", session_id);
                }
            }

            // ── Server lifecycle ────────────────────────────────────
            Event::ServerConnected { .. } => {
                self.connected = true;
            }
            Event::GlobalDisposed { .. } | Event::ServerInstanceDisposed { .. } => {
                self.connected = false;
                self.status_message = "Server disconnected".to_string();
            }

            // ── Other events — acknowledge but do not handle ────────
            Event::SessionDiff { .. }
            | Event::FileEdited { .. }
            | Event::FileWatcherUpdated { .. }
            | Event::CommandExecuted { .. }
            | Event::TuiCommandExecute { .. }
            | Event::InstallationUpdated { .. }
            | Event::InstallationUpdateAvailable { .. }
            | Event::PtyCreated { .. }
            | Event::PtyUpdated { .. }
            | Event::PtyExited { .. }
            | Event::PtyDeleted { .. }
            | Event::McpToolsChanged { .. }
            | Event::McpBrowserOpenFailed { .. }
            | Event::LspUpdated { .. }
            | Event::LspClientDiagnostics { .. }
            | Event::WorktreeReady { .. }
            | Event::WorktreeFailed { .. } => {
                tracing::trace!("Unhandled event type received");
            }
        }
    }

    /// Apply a streaming text delta to the appropriate part.
    pub fn apply_message_delta(&mut self, delta: &MessagePartDeltaProps) {
        if let Some(current) = &self.current_session {
            if current.id != delta.session_id {
                return;
            }
        } else {
            return;
        }

        for msg in &mut self.messages {
            if message_id(&msg.info) != delta.message_id {
                continue;
            }
            for part in &mut msg.parts {
                if part_id(part) != delta.part_id {
                    continue;
                }
                // Apply delta based on the field — delta is a serde_json::Value
                let delta_str = match &delta.delta {
                    serde_json::Value::String(s) => s.as_str(),
                    _ => return,
                };
                match delta.field.as_str() {
                    "text" | "content" => {
                        apply_text_delta(part, delta_str);
                    }
                    _ => {
                        tracing::debug!(
                            "Unhandled delta field '{}' for part {}",
                            delta.field,
                            delta.part_id
                        );
                    }
                }
                return;
            }
            break;
        }
    }

    /// Update or add a part in the messages list.
    pub fn apply_part_update(&mut self, part: &Part) {
        let msg_id = part_message_id(part);
        let p_id = part_id(part);
        let sess_id = part_session_id(part);

        if let Some(current) = &self.current_session {
            if current.id != sess_id {
                return;
            }
        } else {
            return;
        }

        for msg in &mut self.messages {
            if message_id(&msg.info) != msg_id {
                continue;
            }
            if let Some(existing) = msg.parts.iter_mut().find(|p| part_id(p) == p_id) {
                *existing = part.clone();
                return;
            }
            msg.parts.push(part.clone());
            return;
        }

        tracing::debug!(
            "Part {} arrived for unknown message {}; queuing",
            p_id,
            msg_id
        );
    }

    /// Update message metadata (tokens, cost, completion time, etc.).
    pub fn apply_message_update(&mut self, props: &crate::types::MessageUpdatedProps) {
        let msg = &props.info;
        let msg_id_val = message_id(msg);
        let sess_id = message_session_id(msg);

        if let Some(current) = &self.current_session {
            if current.id != sess_id {
                return;
            }
        } else {
            return;
        }

        if let Some(existing) = self.messages.iter_mut().find(|m| message_id(&m.info) == msg_id_val) {
            existing.info = msg.clone();
            return;
        }

        // New message
        self.messages.push(MessageWithParts {
            info: msg.clone(),
            parts: Vec::new(),
        });
    }

    // ── Input editing ───────────────────────────────────────────────

    /// Insert a character at the current cursor position.
    pub fn insert_char(&mut self, c: char) {
        let byte_idx = self.cursor_byte_index();
        self.input_text.insert(byte_idx, c);
        self.input_cursor += 1;
    }

    /// Delete the character before the cursor (backspace).
    pub fn delete_char(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
            let byte_idx = self.cursor_byte_index();
            self.input_text.remove(byte_idx);
            self.recalc_visual_lines();
        }
    }

    /// Delete the character at the cursor (delete key).
    pub fn delete_char_forward(&mut self) {
        let char_count = self.input_text.chars().count();
        if self.input_cursor < char_count {
            let byte_idx = self.cursor_byte_index();
            self.input_text.remove(byte_idx);
            self.recalc_visual_lines();
        }
    }

    /// Move the cursor one character to the left.
    pub fn move_cursor_left(&mut self) {
        if self.input_cursor > 0 {
            self.input_cursor -= 1;
        }
    }

    /// Move the cursor one character to the right.
    pub fn move_cursor_right(&mut self) {
        let char_count = self.input_text.chars().count();
        if self.input_cursor < char_count {
            self.input_cursor += 1;
        }
    }

    /// Move cursor to the beginning of the input.
    pub fn move_cursor_home(&mut self) {
        self.input_cursor = 0;
    }

    /// Move cursor to the end of the input.
    pub fn move_cursor_end(&mut self) {
        self.input_cursor = self.input_text.chars().count();
    }

    /// Returns agents matching the current autocomplete filter.
    pub fn filtered_agents(&self) -> Vec<&Agent> {
        let filter = self.agent_autocomplete_filter.to_lowercase();
        self.agents
            .iter()
            .filter(|a| {
                if a.hidden == Some(true) {
                    return false;
                }
                if filter.is_empty() {
                    return true;
                }
                a.name.to_lowercase().contains(&filter)
            })
            .collect()
    }

    /// Convert the character-based cursor position to a byte index.
    pub(crate) fn cursor_byte_index(&self) -> usize {
        self.input_text
            .char_indices()
            .nth(self.input_cursor)
            .map(|(i, _)| i)
            .unwrap_or(self.input_text.len())
    }

    // ── Scrolling ───────────────────────────────────────────────────

    pub fn scroll_up(&mut self) {
        self.message_scroll = self.message_scroll.saturating_add(1);
    }

    pub fn scroll_down(&mut self) {
        self.message_scroll = self.message_scroll.saturating_sub(1);
    }

    /// Desired editor height: visual lines + 2 for borders, capped at 12.
    pub fn editor_height(&self) -> u16 {
        (self.typed_visual_lines + 2).min(12)
    }

    /// Compute the number of visual lines the input text requires at the given inner width.
    pub fn compute_visual_lines(&self, inner_width: u16) -> u16 {
        if self.input_text.is_empty() {
            return 1;
        }
        let w = inner_width.max(1) as usize;
        let mut count: usize = 0;
        for line in self.input_text.split('\n') {
            let chars = line.chars().count();
            if chars == 0 {
                count += 1;
            } else {
                count += (chars + w - 1) / w;
            }
        }
        count as u16
    }

    /// Recalculate `typed_visual_lines` from the current input text and terminal width.
    /// Call this after typed edits (not paste) to auto-expand the editor.
    pub fn recalc_visual_lines(&mut self) {
        let editor_width = if self.sidebar_visible {
            self.last_terminal_width * 75 / 100
        } else {
            self.last_terminal_width
        };
        let inner_width = editor_width.saturating_sub(2).max(1);
        self.typed_visual_lines = self.compute_visual_lines(inner_width);
    }

    // ── UI helpers ──────────────────────────────────────────────────

    pub fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
    }

    pub fn open_dialog(&mut self, dialog: Dialog) {
        self.active_dialog = Some(dialog);
        self.dialog_filter.clear();
        self.dialog_selected = 0;
    }

    pub fn close_dialog(&mut self) {
        self.dialog_filter.clear();
        self.dialog_selected = 0;

        // Re-surface pending permission/question dialogs
        if !self.pending_permissions.is_empty() {
            self.active_dialog = Some(Dialog::Permission);
        } else if !self.pending_questions.is_empty() {
            self.active_dialog = Some(Dialog::Question);
        } else {
            self.active_dialog = None;
        }
    }

    /// Returns the title of the current session, or a default string.
    pub fn current_session_title(&self) -> &str {
        self.current_session
            .as_ref()
            .and_then(|s| s.title.as_deref())
            .unwrap_or("New Session")
    }

    /// Returns whether the current session is busy.
    pub fn is_session_busy(&self) -> bool {
        if let Some(session) = &self.current_session {
            if let Some(status) = self.session_statuses.get(&session.id) {
                return matches!(status, SessionStatus::Busy);
            }
        }
        self.is_busy
    }

    /// Returns the current model display name.
    pub fn model_display_name(&self) -> String {
        if let Some((provider_id, model_id)) = &self.current_model {
            for provider in &self.providers {
                if &provider.id == provider_id {
                    for (_key, model) in &provider.models {
                        if &model.id == model_id {
                            return format!("{}/{}", provider.name, model.name);
                        }
                    }
                    return format!("{}/{}", provider.name, model_id);
                }
            }
            format!("{}/{}", provider_id, model_id)
        } else {
            "No model".to_string()
        }
    }

    /// Returns the project display name.
    pub fn project_name(&self) -> &str {
        self.project
            .as_ref()
            .and_then(|p| p.name.as_deref())
            .unwrap_or("opencode")
    }

    /// Get VCS branch name from project data.
    pub fn vcs_branch(&self) -> Option<String> {
        self.project
            .as_ref()
            .and_then(|p| p.vcs.as_ref())
            .and_then(|v| v.get("branch"))
            .and_then(|b| b.as_str())
            .map(|s| s.to_string())
    }

    /// Show a toast notification.
    fn show_toast(&mut self, props: TuiToastProps) {
        self.toast = Some(Toast {
            message: props.message.unwrap_or_default(),
            level: match props.variant.as_deref() {
                Some("success") => crate::types::ToastLevel::Success,
                Some("warning") => crate::types::ToastLevel::Warning,
                Some("error") => crate::types::ToastLevel::Error,
                _ => crate::types::ToastLevel::Info,
            },
        });
        self.toast_time = Some(Instant::now());
    }

    /// Clear the toast after it has been displayed for a while.
    pub fn tick_toast(&mut self) {
        if let Some(time) = &self.toast_time {
            if time.elapsed().as_secs() >= 5 {
                self.toast = None;
                self.toast_time = None;
            }
        }
    }
}

// ── Helper functions for type-safe access to Message/Part fields ────────

use crate::types::Message;

fn message_id(msg: &Message) -> &str {
    match msg {
        Message::User(m) => &m.id,
        Message::Assistant(m) => &m.id,
    }
}

fn message_session_id(msg: &Message) -> &str {
    match msg {
        Message::User(m) => &m.session_id,
        Message::Assistant(m) => &m.session_id,
    }
}

fn part_id(part: &Part) -> &str {
    match part {
        Part::Text(p) => &p.id,
        Part::Tool(p) => &p.id,
        Part::Subtask(p) => &p.id,
        Part::Reasoning(p) => &p.id,
        Part::File(p) => &p.id,
        Part::StepStart(p) => &p.id,
        Part::StepFinish(p) => &p.id,
        Part::Snapshot(p) => &p.id,
        Part::Patch(p) => &p.id,
        Part::Agent(p) => &p.id,
        Part::Retry(p) => &p.id,
        Part::Compaction(p) => &p.id,
    }
}

fn part_session_id(part: &Part) -> &str {
    match part {
        Part::Text(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Tool(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Subtask(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Reasoning(p) => p.session_id.as_deref().unwrap_or(""),
        Part::File(p) => p.session_id.as_deref().unwrap_or(""),
        Part::StepStart(p) => p.session_id.as_deref().unwrap_or(""),
        Part::StepFinish(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Snapshot(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Patch(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Agent(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Retry(p) => p.session_id.as_deref().unwrap_or(""),
        Part::Compaction(p) => p.session_id.as_deref().unwrap_or(""),
    }
}

fn part_message_id(part: &Part) -> &str {
    match part {
        Part::Text(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Tool(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Subtask(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Reasoning(p) => p.message_id.as_deref().unwrap_or(""),
        Part::File(p) => p.message_id.as_deref().unwrap_or(""),
        Part::StepStart(p) => p.message_id.as_deref().unwrap_or(""),
        Part::StepFinish(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Snapshot(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Patch(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Agent(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Retry(p) => p.message_id.as_deref().unwrap_or(""),
        Part::Compaction(p) => p.message_id.as_deref().unwrap_or(""),
    }
}

/// Apply a text delta to a part's text/content field.
fn apply_text_delta(part: &mut Part, delta: &str) {
    match part {
        Part::Text(p) => {
            if let Some(ref mut text) = p.text {
                text.push_str(delta);
            } else {
                p.text = Some(delta.to_string());
            }
        }
        Part::Reasoning(p) => {
            if let Some(ref mut content) = p.content {
                content.push_str(delta);
            } else {
                p.content = Some(delta.to_string());
            }
        }
        _ => {
            tracing::debug!("Text delta applied to non-text part type");
        }
    }
}
