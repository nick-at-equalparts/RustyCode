use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Session {
    pub id: String,
    pub slug: Option<String>,
    #[serde(rename = "projectID")]
    pub project_id: Option<String>,
    pub directory: Option<String>,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
    pub summary: Option<SessionSummary>,
    pub share: Option<bool>,
    pub title: Option<String>,
    pub version: Option<String>,
    pub time: SessionTime,
    pub permission: Option<serde_json::Value>,
    pub revert: Option<SessionRevert>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSummary {
    pub title: Option<String>,
    pub description: Option<String>,
    pub additions: Option<u64>,
    pub deletions: Option<u64>,
    pub files: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTime {
    pub created: serde_json::Value,
    pub updated: serde_json::Value,
    pub archived: Option<serde_json::Value>,
    pub initialized: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SessionStatus {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "retry")]
    Retry {
        attempt: Option<u32>,
        message: Option<String>,
        next: Option<String>,
    },
    #[serde(rename = "busy")]
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRevert {
    #[serde(rename = "sessionID")]
    pub session_id: Option<String>,
    #[serde(rename = "messageID")]
    pub message_id: Option<String>,
}
