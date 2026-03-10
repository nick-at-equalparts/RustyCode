use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Part {
    #[serde(rename = "text")]
    Text(TextPart),
    #[serde(rename = "subtask")]
    Subtask(SubtaskPart),
    #[serde(rename = "reasoning")]
    Reasoning(ReasoningPart),
    #[serde(rename = "file")]
    File(FilePart),
    #[serde(rename = "tool")]
    Tool(ToolPart),
    #[serde(rename = "step-start")]
    StepStart(StepStartPart),
    #[serde(rename = "step-finish")]
    StepFinish(StepFinishPart),
    #[serde(rename = "snapshot")]
    Snapshot(SnapshotPart),
    #[serde(rename = "patch")]
    Patch(PatchPart),
    #[serde(rename = "agent")]
    Agent(AgentPart),
    #[serde(rename = "retry")]
    Retry(RetryPart),
    #[serde(rename = "compaction")]
    Compaction(CompactionPart),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtaskPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub input: Option<String>,
    pub summary: Option<String>,
    #[serde(rename = "modelID")]
    pub model_id: Option<String>,
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReasoningPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub content: Option<String>,
    pub redacted: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilePart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
    pub media_type: Option<String>,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub tool: Option<String>,
    pub input: Option<serde_json::Value>,
    pub state: ToolState,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ToolState {
    #[serde(rename = "pending")]
    Pending {
        #[serde(default)]
        input: Option<serde_json::Value>,
    },
    #[serde(rename = "running")]
    Running {
        #[serde(default)]
        input: Option<serde_json::Value>,
    },
    #[serde(rename = "completed")]
    Completed {
        #[serde(default)]
        input: Option<serde_json::Value>,
        output: Option<String>,
        title: Option<String>,
    },
    #[serde(rename = "error")]
    Error {
        #[serde(default)]
        input: Option<serde_json::Value>,
        error: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepStartPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    #[serde(rename = "stepID")]
    pub step_id: Option<String>,
    pub title: Option<String>,
    pub snapshot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StepFinishPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    #[serde(rename = "stepID")]
    pub step_id: Option<String>,
    pub reason: Option<String>,
    pub snapshot: Option<String>,
    pub cost: Option<f64>,
    pub tokens: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub path: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompactionPart {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
    #[serde(default)]
    pub time: Option<serde_json::Value>,
}
