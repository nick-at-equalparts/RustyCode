use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Wrapper for the provider list API response: `{"all": [...], "default": ..., "connected": ...}`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderListResponse {
    pub all: Vec<Provider>,
    #[serde(default)]
    pub default: Option<serde_json::Value>,
    #[serde(default)]
    pub connected: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Provider {
    pub id: String,
    pub name: String,
    pub source: Option<ProviderSource>,
    pub env: Option<Vec<String>>,
    pub key: Option<String>,
    pub options: Option<HashMap<String, serde_json::Value>>,
    #[serde(default)]
    pub models: HashMap<String, Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    pub id: String,
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
    pub api: Option<ModelApi>,
    pub name: String,
    pub family: Option<String>,
    pub status: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub capabilities: Option<ModelCapabilities>,
    pub cost: Option<ModelCost>,
    pub limit: Option<ModelLimit>,
    pub release_date: Option<String>,
    pub variants: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelApi {
    pub id: Option<String>,
    pub url: Option<String>,
    pub npm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ProviderSource {
    #[serde(rename = "env")]
    Env,
    #[serde(rename = "config")]
    Config,
    #[serde(rename = "custom")]
    Custom,
    #[serde(rename = "api")]
    Api,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub reasoning: Option<bool>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCost {
    pub input: Option<f64>,
    pub output: Option<f64>,
    pub cache_read: Option<f64>,
    pub cache_write: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelLimit {
    pub context: Option<u64>,
    pub output: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderAuthMethod {
    #[serde(rename = "type")]
    pub auth_type: Option<String>,
    pub env: Option<String>,
    pub header: Option<String>,
}
