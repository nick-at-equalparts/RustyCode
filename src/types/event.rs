use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::file::FileDiff;
use super::message::Message;
use super::misc::{
    InstallationInfo, LSPDiagnostics, LSPStatus, MCPToolsInfo, Pty, QuestionAnswer, QuestionInfo,
    Todo, Worktree,
};
use super::part::Part;
use super::permission::{PermissionReply, PermissionRequest};
use super::project::Project;
use super::session::{Session, SessionStatus};

/// All SSE event types emitted by the OpenCode server.
///
/// The server sends events as JSON objects with a `"type"` field discriminator
/// and a `"properties"` field containing event-specific data.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    // ── Session events ──────────────────────────────────────────────
    #[serde(rename = "session.created")]
    SessionCreated { properties: SessionInfoProps },
    #[serde(rename = "session.updated")]
    SessionUpdated { properties: SessionInfoProps },
    #[serde(rename = "session.deleted")]
    SessionDeleted { properties: SessionInfoProps },
    #[serde(rename = "session.compacted")]
    SessionCompacted { properties: SessionIdProps },

    #[serde(rename = "session.status")]
    SessionStatus { properties: SessionStatusProps },
    #[serde(rename = "session.idle")]
    SessionIdle { properties: SessionIdProps },
    #[serde(rename = "session.error")]
    SessionError { properties: SessionErrorProps },
    #[serde(rename = "session.diff")]
    SessionDiff { properties: SessionDiffProps },

    // ── Message events ──────────────────────────────────────────────
    #[serde(rename = "message.updated")]
    MessageUpdated { properties: MessageUpdatedProps },
    #[serde(rename = "message.removed")]
    MessageRemoved { properties: MessageRemovedProps },

    // ── Part events ─────────────────────────────────────────────────
    #[serde(rename = "message.part.updated")]
    MessagePartUpdated { properties: PartUpdatedProps },
    #[serde(rename = "message.part.delta")]
    MessagePartDelta { properties: MessagePartDeltaProps },
    #[serde(rename = "message.part.removed")]
    MessagePartRemoved { properties: PartRemovedProps },

    // ── Permission events ───────────────────────────────────────────
    #[serde(rename = "permission.asked")]
    PermissionAsked { properties: PermissionAskedProps },
    #[serde(rename = "permission.replied")]
    PermissionReplied { properties: PermissionRepliedProps },

    // ── Question events ─────────────────────────────────────────────
    #[serde(rename = "question.asked")]
    QuestionAsked { properties: QuestionAskedProps },
    #[serde(rename = "question.replied")]
    QuestionReplied { properties: QuestionRepliedProps },
    #[serde(rename = "question.rejected")]
    QuestionRejected { properties: QuestionRejectedProps },

    // ── Todo events ─────────────────────────────────────────────────
    #[serde(rename = "todo.updated")]
    TodoUpdated { properties: TodoUpdatedProps },

    // ── Project / VCS events ────────────────────────────────────────
    #[serde(rename = "project.updated")]
    ProjectUpdated { properties: ProjectUpdatedProps },
    #[serde(rename = "vcs.branch.updated")]
    VcsBranchUpdated { properties: VcsBranchProps },

    // ── Server lifecycle ────────────────────────────────────────────
    #[serde(rename = "server.connected")]
    ServerConnected { properties: serde_json::Value },
    #[serde(rename = "server.instance.disposed")]
    ServerInstanceDisposed { properties: serde_json::Value },
    #[serde(rename = "global.disposed")]
    GlobalDisposed { properties: serde_json::Value },

    // ── File events ─────────────────────────────────────────────────
    #[serde(rename = "file.edited")]
    FileEdited { properties: FileEditedProps },
    #[serde(rename = "file.watcher.updated")]
    FileWatcherUpdated { properties: FileWatcherUpdatedProps },

    // ── Command events ──────────────────────────────────────────────
    #[serde(rename = "command.executed")]
    CommandExecuted { properties: CommandExecutedProps },

    // ── TUI control events ──────────────────────────────────────────
    #[serde(rename = "tui.prompt.append")]
    TuiPromptAppend { properties: TuiPromptAppendProps },
    #[serde(rename = "tui.command.execute")]
    TuiCommandExecute { properties: TuiCommandExecuteProps },
    #[serde(rename = "tui.toast.show")]
    TuiToastShow { properties: TuiToastProps },
    #[serde(rename = "tui.session.select")]
    TuiSessionSelect { properties: TuiSessionSelectProps },

    // ── Installation events ─────────────────────────────────────────
    #[serde(rename = "installation.updated")]
    InstallationUpdated { properties: InstallationUpdatedProps },
    #[serde(rename = "installation.update-available")]
    InstallationUpdateAvailable {
        properties: InstallationUpdateAvailableProps,
    },

    // ── PTY events ──────────────────────────────────────────────────
    #[serde(rename = "pty.created")]
    PtyCreated { properties: PtyEventProps },
    #[serde(rename = "pty.updated")]
    PtyUpdated { properties: PtyEventProps },
    #[serde(rename = "pty.exited")]
    PtyExited { properties: PtyExitedProps },
    #[serde(rename = "pty.deleted")]
    PtyDeleted { properties: PtyEventProps },

    // ── MCP events ──────────────────────────────────────────────────
    #[serde(rename = "mcp.tools.changed")]
    McpToolsChanged { properties: McpToolsChangedProps },
    #[serde(rename = "mcp.browser.open.failed")]
    McpBrowserOpenFailed { properties: McpBrowserOpenFailedProps },

    // ── LSP events ──────────────────────────────────────────────────
    #[serde(rename = "lsp.updated")]
    LspUpdated { properties: LspUpdatedProps },
    #[serde(rename = "lsp.client.diagnostics")]
    LspClientDiagnostics { properties: LspClientDiagnosticsProps },

    // ── Worktree events ─────────────────────────────────────────────
    #[serde(rename = "worktree.ready")]
    WorktreeReady { properties: WorktreeReadyProps },
    #[serde(rename = "worktree.failed")]
    WorktreeFailed { properties: WorktreeFailedProps },
}

// ── Session property structs ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfoProps {
    pub info: Session,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatusProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionIdProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionErrorProps {
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    pub error: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionDiffProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub diffs: Option<Vec<FileDiff>>,
}

// ── Message property structs ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageUpdatedProps {
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    pub info: Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageRemovedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "messageID")]
    pub message_id: String,
}

// ── Part property structs ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartUpdatedProps {
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    pub part: Part,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessagePartDeltaProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "messageID")]
    pub message_id: String,
    #[serde(rename = "partID")]
    pub part_id: String,
    pub field: String,
    pub delta: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartRemovedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "messageID")]
    pub message_id: String,
    #[serde(rename = "partID")]
    pub part_id: String,
}

// ── Permission property structs ───────────────────────────────────────────

/// The `permission.asked` event sends the permission fields flat in properties
/// (not nested inside a `request` wrapper).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionAskedProps {
    #[serde(flatten)]
    pub request: PermissionRequest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRepliedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "requestID")]
    pub request_id: String,
    pub reply: PermissionReply,
}

// ── Question property structs ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionAskedProps {
    pub question: QuestionInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionRepliedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "questionID")]
    pub question_id: String,
    pub answer: QuestionAnswer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuestionRejectedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    #[serde(rename = "questionID")]
    pub question_id: String,
}

// ── Todo property structs ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodoUpdatedProps {
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub todos: Option<Vec<Todo>>,
}

// ── Project / VCS property structs ────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectUpdatedProps {
    pub info: Project,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VcsBranchProps {
    pub branch: Option<String>,
}

// ── File property structs ─────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEditedProps {
    pub file: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileWatcherUpdatedProps {
    pub files: Option<Vec<String>>,
}

// ── Command property structs ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandExecutedProps {
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

// ── TUI property structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuiPromptAppendProps {
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuiCommandExecuteProps {
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuiToastProps {
    pub title: Option<String>,
    pub message: Option<String>,
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TuiSessionSelectProps {
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
}

// ── Installation property structs ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationUpdatedProps {
    pub info: Option<InstallationInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallationUpdateAvailableProps {
    pub version: Option<String>,
}

// ── PTY property structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyEventProps {
    pub info: Pty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PtyExitedProps {
    pub id: String,
    pub exit_code: Option<i32>,
}

// ── MCP property structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpToolsChangedProps {
    pub tools: Option<MCPToolsInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpBrowserOpenFailedProps {
    pub url: Option<String>,
    pub error: Option<String>,
}

// ── LSP property structs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspUpdatedProps {
    pub statuses: Option<HashMap<String, LSPStatus>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LspClientDiagnosticsProps {
    pub diagnostics: Option<LSPDiagnostics>,
}

// ── Worktree property structs ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeReadyProps {
    pub worktree: Worktree,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeFailedProps {
    pub error: String,
}
