use serde::{Deserialize, Serialize};

use super::part::Part;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role")]
pub enum Message {
    #[serde(rename = "user")]
    User(UserMessage),
    #[serde(rename = "assistant")]
    Assistant(AssistantMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserMessage {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub role: Option<String>,
    pub time: MessageTime,
    pub format: Option<MessageFormat>,
    pub summary: Option<serde_json::Value>,
    pub agent: Option<String>,
    pub model: Option<MessageModel>,
    pub system: Option<bool>,
    pub tools: Option<Vec<String>>,
    pub variant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssistantMessage {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub role: Option<String>,
    pub time: MessageTime,
    pub error: Option<serde_json::Value>,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
    #[serde(rename = "modelID")]
    pub model_id: Option<String>,
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
    pub mode: Option<String>,
    pub agent: Option<String>,
    pub path: Option<serde_json::Value>,
    pub cost: Option<f64>,
    pub tokens: Option<MessageTokens>,
    pub system: Option<bool>,
    pub finish: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageModel {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageFormat {
    pub style: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTime {
    pub created: serde_json::Value,
    #[serde(default)]
    pub updated: Option<serde_json::Value>,
    pub completed: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageTokens {
    pub total: Option<u64>,
    pub input: Option<u64>,
    pub output: Option<u64>,
    pub reasoning: Option<u64>,
    pub cache: Option<TokenCache>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCache {
    pub read: Option<u64>,
    pub write: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageWithParts {
    pub info: Message,
    pub parts: Vec<Part>,
}
