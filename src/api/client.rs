use std::collections::HashMap;

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::{
    Agent, Command, FileContent, FileDiff, FileNode, FormatterStatus, LSPStatus, MessageWithParts,
    Path, PermissionRequest, Project, ProviderListResponse, Pty, QuestionRequest, Session,
    SessionStatus, Todo,
};

// ---------------------------------------------------------------------------
// Request / response helper structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

/// Model selector used in prompt requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelSelector {
    #[serde(rename = "providerID")]
    pub provider_id: String,
    #[serde(rename = "modelID")]
    pub model_id: String,
}

/// A single part that can be included in a prompt message.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PartInput {
    #[serde(rename = "text")]
    Text {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        synthetic: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        ignored: Option<bool>,
    },
    #[serde(rename = "file")]
    File {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        mime: String,
        url: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        filename: Option<String>,
    },
    #[serde(rename = "agent")]
    Agent {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        name: String,
    },
    #[serde(rename = "subtask")]
    Subtask {
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        prompt: String,
        description: String,
        agent: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        model: Option<ModelSelector>,
        #[serde(skip_serializing_if = "Option::is_none")]
        command: Option<String>,
    },
}

/// Body for `POST /session/{id}/message` and `POST /session/{id}/prompt_async`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub parts: Vec<PartInput>,
    #[serde(rename = "messageID", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<ModelSelector>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(rename = "noReply", skip_serializing_if = "Option::is_none")]
    pub no_reply: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<serde_json::Value>,
}

/// Body for `POST /session/{id}/command`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCommandRequest {
    pub command: String,
    pub arguments: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    #[serde(rename = "messageID", skip_serializing_if = "Option::is_none")]
    pub message_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parts: Option<Vec<serde_json::Value>>,
}

/// TUI control request returned by `GET /tui/control/next`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TuiControlRequest {
    pub path: String,
    pub body: serde_json::Value,
}

// ---------------------------------------------------------------------------
// ApiClient
// ---------------------------------------------------------------------------

/// HTTP client for the OpenCode API.
///
/// All methods return `anyhow::Result` and assume JSON transport.
/// The struct is `Clone`-able (reqwest::Client uses an inner Arc).
#[derive(Debug, Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    /// Create a new client pointing at the given base URL
    /// (e.g. `http://127.0.0.1:3000`).
    pub fn new(base_url: String) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");
        Self {
            client,
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Return the configured base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full URL from a path segment.
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // =======================================================================
    // Health & Global
    // =======================================================================

    /// `GET /global/health`
    pub async fn health(&self) -> Result<HealthResponse> {
        let resp = self
            .client
            .get(self.url("/global/health"))
            .send()
            .await
            .context("GET /global/health")?;
        resp.error_for_status_ref()
            .context("GET /global/health status")?;
        resp.json().await.context("parse health response")
    }

    /// `POST /global/dispose`
    pub async fn dispose_global(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/global/dispose"))
            .send()
            .await
            .context("POST /global/dispose")?;
        resp.error_for_status_ref()
            .context("POST /global/dispose status")?;
        resp.json().await.context("parse dispose_global response")
    }

    /// `GET /global/config`
    pub async fn get_global_config(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/global/config"))
            .send()
            .await
            .context("GET /global/config")?;
        resp.error_for_status_ref()
            .context("GET /global/config status")?;
        resp.json().await.context("parse global config")
    }

    /// `PATCH /global/config`
    pub async fn update_global_config(
        &self,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .patch(self.url("/global/config"))
            .json(config)
            .send()
            .await
            .context("PATCH /global/config")?;
        resp.error_for_status_ref()
            .context("PATCH /global/config status")?;
        resp.json()
            .await
            .context("parse update_global_config response")
    }

    // =======================================================================
    // Auth
    // =======================================================================

    /// `PUT /auth/{providerID}`
    pub async fn set_auth(
        &self,
        provider_id: &str,
        credentials: &serde_json::Value,
    ) -> Result<bool> {
        let resp = self
            .client
            .put(self.url(&format!("/auth/{}", provider_id)))
            .json(credentials)
            .send()
            .await
            .context("PUT /auth/{providerID}")?;
        resp.error_for_status_ref()
            .context("PUT /auth/{providerID} status")?;
        resp.json().await.context("parse set_auth response")
    }

    /// `DELETE /auth/{providerID}`
    pub async fn delete_auth(&self, provider_id: &str) -> Result<bool> {
        let resp = self
            .client
            .delete(self.url(&format!("/auth/{}", provider_id)))
            .send()
            .await
            .context("DELETE /auth/{providerID}")?;
        resp.error_for_status_ref()
            .context("DELETE /auth/{providerID} status")?;
        resp.json().await.context("parse delete_auth response")
    }

    // =======================================================================
    // Projects
    // =======================================================================

    /// `GET /project`
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        let resp = self
            .client
            .get(self.url("/project"))
            .send()
            .await
            .context("GET /project")?;
        resp.error_for_status_ref().context("GET /project status")?;
        resp.json().await.context("parse list_projects response")
    }

    /// `GET /project/current`
    pub async fn get_current_project(&self) -> Result<Project> {
        let resp = self
            .client
            .get(self.url("/project/current"))
            .send()
            .await
            .context("GET /project/current")?;
        resp.error_for_status_ref()
            .context("GET /project/current status")?;
        resp.json()
            .await
            .context("parse get_current_project response")
    }

    /// `PATCH /project/{projectID}`
    pub async fn update_project(
        &self,
        project_id: &str,
        updates: &serde_json::Value,
    ) -> Result<Project> {
        let resp = self
            .client
            .patch(self.url(&format!("/project/{}", project_id)))
            .json(updates)
            .send()
            .await
            .context("PATCH /project/{projectID}")?;
        resp.error_for_status_ref()
            .context("PATCH /project/{projectID} status")?;
        resp.json().await.context("parse update_project response")
    }

    // =======================================================================
    // Config
    // =======================================================================

    /// `GET /config`
    pub async fn get_config(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/config"))
            .send()
            .await
            .context("GET /config")?;
        resp.error_for_status_ref().context("GET /config status")?;
        resp.json().await.context("parse get_config response")
    }

    /// `PATCH /config`
    pub async fn update_config(&self, config: &serde_json::Value) -> Result<serde_json::Value> {
        let resp = self
            .client
            .patch(self.url("/config"))
            .json(config)
            .send()
            .await
            .context("PATCH /config")?;
        resp.error_for_status_ref()
            .context("PATCH /config status")?;
        resp.json().await.context("parse update_config response")
    }

    /// `GET /config/providers`
    pub async fn get_providers_config(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/config/providers"))
            .send()
            .await
            .context("GET /config/providers")?;
        resp.error_for_status_ref()
            .context("GET /config/providers status")?;
        resp.json()
            .await
            .context("parse get_providers_config response")
    }

    // =======================================================================
    // Providers
    // =======================================================================

    /// `GET /provider`
    pub async fn list_providers(&self) -> Result<ProviderListResponse> {
        let resp = self
            .client
            .get(self.url("/provider"))
            .send()
            .await
            .context("GET /provider")?;
        resp.error_for_status_ref()
            .context("GET /provider status")?;
        resp.json().await.context("parse list_providers response")
    }

    /// `GET /provider/auth`
    pub async fn get_provider_auth(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/provider/auth"))
            .send()
            .await
            .context("GET /provider/auth")?;
        resp.error_for_status_ref()
            .context("GET /provider/auth status")?;
        resp.json()
            .await
            .context("parse get_provider_auth response")
    }

    /// `POST /provider/{providerID}/oauth/authorize`
    pub async fn oauth_authorize(&self, provider_id: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/provider/{}/oauth/authorize", provider_id)))
            .send()
            .await
            .context("POST /provider/{id}/oauth/authorize")?;
        resp.error_for_status_ref()
            .context("POST /provider/{id}/oauth/authorize status")?;
        resp.json().await.context("parse oauth_authorize response")
    }

    /// `POST /provider/{providerID}/oauth/callback`
    pub async fn oauth_callback(&self, provider_id: &str) -> Result<bool> {
        let resp = self
            .client
            .post(self.url(&format!("/provider/{}/oauth/callback", provider_id)))
            .send()
            .await
            .context("POST /provider/{id}/oauth/callback")?;
        resp.error_for_status_ref()
            .context("POST /provider/{id}/oauth/callback status")?;
        resp.json().await.context("parse oauth_callback response")
    }

    // =======================================================================
    // Sessions
    // =======================================================================

    /// `GET /session`
    pub async fn list_sessions(&self, project_id: Option<&str>) -> Result<Vec<Session>> {
        let mut req = self.client.get(self.url("/session"));
        if let Some(pid) = project_id {
            req = req.query(&[("projectID", pid)]);
        }
        let resp = req.send().await.context("GET /session")?;
        resp.error_for_status_ref().context("GET /session status")?;
        resp.json().await.context("parse list_sessions response")
    }

    /// `POST /session`
    pub async fn create_session(
        &self,
        parent_id: Option<&str>,
        title: Option<&str>,
    ) -> Result<Session> {
        let mut body = serde_json::Map::new();
        if let Some(pid) = parent_id {
            body.insert("parentID".into(), serde_json::Value::String(pid.into()));
        }
        if let Some(t) = title {
            body.insert("title".into(), serde_json::Value::String(t.into()));
        }
        let resp = self
            .client
            .post(self.url("/session"))
            .json(&body)
            .send()
            .await
            .context("POST /session")?;
        resp.error_for_status_ref()
            .context("POST /session status")?;
        resp.json().await.context("parse create_session response")
    }

    /// `GET /session/status`
    pub async fn get_session_statuses(&self) -> Result<HashMap<String, SessionStatus>> {
        let resp = self
            .client
            .get(self.url("/session/status"))
            .send()
            .await
            .context("GET /session/status")?;
        resp.error_for_status_ref()
            .context("GET /session/status status")?;
        resp.json()
            .await
            .context("parse get_session_statuses response")
    }

    /// `GET /session/{sessionID}`
    pub async fn get_session(&self, id: &str) -> Result<Session> {
        let resp = self
            .client
            .get(self.url(&format!("/session/{}", id)))
            .send()
            .await
            .context("GET /session/{sessionID}")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID} status")?;
        resp.json().await.context("parse get_session response")
    }

    /// `DELETE /session/{sessionID}`
    pub async fn delete_session(&self, id: &str) -> Result<bool> {
        let resp = self
            .client
            .delete(self.url(&format!("/session/{}", id)))
            .send()
            .await
            .context("DELETE /session/{sessionID}")?;
        resp.error_for_status_ref()
            .context("DELETE /session/{sessionID} status")?;
        resp.json().await.context("parse delete_session response")
    }

    /// `PATCH /session/{sessionID}`
    pub async fn update_session(&self, id: &str, title: Option<&str>) -> Result<Session> {
        let mut body = serde_json::Map::new();
        if let Some(t) = title {
            body.insert("title".into(), serde_json::Value::String(t.into()));
        }
        let resp = self
            .client
            .patch(self.url(&format!("/session/{}", id)))
            .json(&body)
            .send()
            .await
            .context("PATCH /session/{sessionID}")?;
        resp.error_for_status_ref()
            .context("PATCH /session/{sessionID} status")?;
        resp.json().await.context("parse update_session response")
    }

    /// `GET /session/{sessionID}/children`
    pub async fn get_session_children(&self, id: &str) -> Result<Vec<Session>> {
        let resp = self
            .client
            .get(self.url(&format!("/session/{}/children", id)))
            .send()
            .await
            .context("GET /session/{sessionID}/children")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID}/children status")?;
        resp.json()
            .await
            .context("parse get_session_children response")
    }

    /// `GET /session/{sessionID}/todo`
    pub async fn get_session_todos(&self, id: &str) -> Result<Vec<Todo>> {
        let resp = self
            .client
            .get(self.url(&format!("/session/{}/todo", id)))
            .send()
            .await
            .context("GET /session/{sessionID}/todo")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID}/todo status")?;
        resp.json()
            .await
            .context("parse get_session_todos response")
    }

    /// `POST /session/{sessionID}/init`
    pub async fn init_session(
        &self,
        id: &str,
        message_id: &str,
        provider_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        let body = serde_json::json!({
            "messageID": message_id,
            "providerID": provider_id,
            "modelID": model_id,
        });
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/init", id)))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/init")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/init status")?;
        resp.json().await.context("parse init_session response")
    }

    /// `POST /session/{sessionID}/fork`
    pub async fn fork_session(&self, id: &str, message_id: Option<&str>) -> Result<Session> {
        let mut body = serde_json::Map::new();
        if let Some(mid) = message_id {
            body.insert("messageID".into(), serde_json::Value::String(mid.into()));
        }
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/fork", id)))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/fork")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/fork status")?;
        resp.json().await.context("parse fork_session response")
    }

    /// `POST /session/{sessionID}/abort`
    pub async fn abort_session(&self, id: &str) -> Result<bool> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/abort", id)))
            .send()
            .await
            .context("POST /session/{sessionID}/abort")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/abort status")?;
        resp.json().await.context("parse abort_session response")
    }

    /// `POST /session/{sessionID}/share`
    pub async fn share_session(&self, id: &str) -> Result<Session> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/share", id)))
            .send()
            .await
            .context("POST /session/{sessionID}/share")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/share status")?;
        resp.json().await.context("parse share_session response")
    }

    /// `DELETE /session/{sessionID}/share`
    pub async fn unshare_session(&self, id: &str) -> Result<Session> {
        let resp = self
            .client
            .delete(self.url(&format!("/session/{}/share", id)))
            .send()
            .await
            .context("DELETE /session/{sessionID}/share")?;
        resp.error_for_status_ref()
            .context("DELETE /session/{sessionID}/share status")?;
        resp.json().await.context("parse unshare_session response")
    }

    /// `GET /session/{sessionID}/diff`
    pub async fn get_session_diff(
        &self,
        id: &str,
        message_id: Option<&str>,
    ) -> Result<Vec<FileDiff>> {
        let mut req = self.client.get(self.url(&format!("/session/{}/diff", id)));
        if let Some(mid) = message_id {
            req = req.query(&[("messageID", mid)]);
        }
        let resp = req.send().await.context("GET /session/{sessionID}/diff")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID}/diff status")?;
        resp.json().await.context("parse get_session_diff response")
    }

    /// `POST /session/{sessionID}/summarize`
    pub async fn summarize_session(
        &self,
        id: &str,
        provider_id: &str,
        model_id: &str,
    ) -> Result<bool> {
        let body = serde_json::json!({
            "providerID": provider_id,
            "modelID": model_id,
        });
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/summarize", id)))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/summarize")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/summarize status")?;
        resp.json()
            .await
            .context("parse summarize_session response")
    }

    /// `POST /session/{sessionID}/revert`
    pub async fn revert_session(
        &self,
        id: &str,
        message_id: &str,
        part_id: Option<&str>,
    ) -> Result<bool> {
        let mut body = serde_json::Map::new();
        body.insert(
            "messageID".into(),
            serde_json::Value::String(message_id.into()),
        );
        if let Some(pid) = part_id {
            body.insert("partID".into(), serde_json::Value::String(pid.into()));
        }
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/revert", id)))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/revert")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/revert status")?;
        // Server returns Session on success; we convert to bool.
        let _: serde_json::Value = resp.json().await.context("parse revert response")?;
        Ok(true)
    }

    /// `POST /session/{sessionID}/unrevert`
    pub async fn unrevert_session(&self, id: &str) -> Result<bool> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/unrevert", id)))
            .send()
            .await
            .context("POST /session/{sessionID}/unrevert")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/unrevert status")?;
        let _: serde_json::Value = resp.json().await.context("parse unrevert response")?;
        Ok(true)
    }

    /// `POST /session/{sessionID}/permissions/{permissionID}` (deprecated)
    pub async fn reply_permission(
        &self,
        session_id: &str,
        permission_id: &str,
        response: &str,
        _remember: Option<bool>,
    ) -> Result<bool> {
        let body = serde_json::json!({ "response": response });
        let resp = self
            .client
            .post(self.url(&format!(
                "/session/{}/permissions/{}",
                session_id, permission_id
            )))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/permissions/{permissionID}")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/permissions/{permissionID} status")?;
        resp.json().await.context("parse reply_permission response")
    }

    // =======================================================================
    // Messages
    // =======================================================================

    /// `GET /session/{sessionID}/message`
    pub async fn list_messages(
        &self,
        session_id: &str,
        limit: Option<u32>,
    ) -> Result<Vec<MessageWithParts>> {
        let mut req = self
            .client
            .get(self.url(&format!("/session/{}/message", session_id)));
        if let Some(l) = limit {
            req = req.query(&[("limit", l)]);
        }
        let resp = req
            .send()
            .await
            .context("GET /session/{sessionID}/message")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID}/message status")?;
        resp.json().await.context("parse list_messages response")
    }

    /// `POST /session/{sessionID}/message` -- send a prompt and stream/get the
    /// assistant response.
    pub async fn send_message(
        &self,
        session_id: &str,
        body: &SendMessageRequest,
    ) -> Result<MessageWithParts> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/message", session_id)))
            .json(body)
            .send()
            .await
            .context("POST /session/{sessionID}/message")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/message status")?;
        resp.json().await.context("parse send_message response")
    }

    /// `GET /session/{sessionID}/message/{messageID}`
    pub async fn get_message(
        &self,
        session_id: &str,
        message_id: &str,
    ) -> Result<MessageWithParts> {
        let resp = self
            .client
            .get(self.url(&format!("/session/{}/message/{}", session_id, message_id)))
            .send()
            .await
            .context("GET /session/{sessionID}/message/{messageID}")?;
        resp.error_for_status_ref()
            .context("GET /session/{sessionID}/message/{messageID} status")?;
        resp.json().await.context("parse get_message response")
    }

    /// `DELETE /session/{sessionID}/message/{messageID}`
    pub async fn delete_message(&self, session_id: &str, message_id: &str) -> Result<bool> {
        let resp = self
            .client
            .delete(self.url(&format!("/session/{}/message/{}", session_id, message_id)))
            .send()
            .await
            .context("DELETE /session/{sessionID}/message/{messageID}")?;
        resp.error_for_status_ref()
            .context("DELETE /session/{sessionID}/message/{messageID} status")?;
        resp.json().await.context("parse delete_message response")
    }

    /// `DELETE /session/{sessionID}/message/{messageID}/part/{partID}`
    pub async fn delete_message_part(
        &self,
        session_id: &str,
        message_id: &str,
        part_id: &str,
    ) -> Result<bool> {
        let resp = self
            .client
            .delete(self.url(&format!(
                "/session/{}/message/{}/part/{}",
                session_id, message_id, part_id
            )))
            .send()
            .await
            .context("DELETE .../message/{messageID}/part/{partID}")?;
        resp.error_for_status_ref()
            .context("DELETE .../part/{partID} status")?;
        resp.json()
            .await
            .context("parse delete_message_part response")
    }

    /// `PATCH /session/{sessionID}/message/{messageID}/part/{partID}`
    pub async fn update_message_part(
        &self,
        session_id: &str,
        message_id: &str,
        part_id: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .patch(self.url(&format!(
                "/session/{}/message/{}/part/{}",
                session_id, message_id, part_id
            )))
            .json(body)
            .send()
            .await
            .context("PATCH .../part/{partID}")?;
        resp.error_for_status_ref()
            .context("PATCH .../part/{partID} status")?;
        resp.json()
            .await
            .context("parse update_message_part response")
    }

    /// `POST /session/{sessionID}/prompt_async` -- fire-and-forget (returns 204).
    pub async fn send_prompt_async(
        &self,
        session_id: &str,
        body: &SendMessageRequest,
    ) -> Result<()> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/prompt_async", session_id)))
            .json(body)
            .send()
            .await
            .context("POST /session/{sessionID}/prompt_async")?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            anyhow::bail!("prompt_async failed ({}): {}", status, text);
        }
        Ok(())
    }

    /// `POST /session/{sessionID}/command`
    pub async fn send_command(
        &self,
        session_id: &str,
        body: &SendCommandRequest,
    ) -> Result<MessageWithParts> {
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/command", session_id)))
            .json(body)
            .send()
            .await
            .context("POST /session/{sessionID}/command")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/command status")?;
        resp.json().await.context("parse send_command response")
    }

    /// `POST /session/{sessionID}/shell`
    pub async fn send_shell(
        &self,
        session_id: &str,
        agent: &str,
        model: Option<&str>,
        command: &str,
    ) -> Result<MessageWithParts> {
        let mut body = serde_json::json!({
            "agent": agent,
            "command": command,
        });
        if let Some(m) = model {
            body.as_object_mut().unwrap().insert(
                "model".into(),
                serde_json::json!({
                    "providerID": m,
                    "modelID": m,
                }),
            );
        }
        let resp = self
            .client
            .post(self.url(&format!("/session/{}/shell", session_id)))
            .json(&body)
            .send()
            .await
            .context("POST /session/{sessionID}/shell")?;
        resp.error_for_status_ref()
            .context("POST /session/{sessionID}/shell status")?;
        resp.json().await.context("parse send_shell response")
    }

    // =======================================================================
    // Permissions & Questions
    // =======================================================================

    /// `POST /permission/{requestID}/reply`
    pub async fn reply_permission_request(
        &self,
        request_id: &str,
        body: &serde_json::Value,
    ) -> Result<bool> {
        let resp = self
            .client
            .post(self.url(&format!("/permission/{}/reply", request_id)))
            .json(body)
            .send()
            .await
            .context("POST /permission/{requestID}/reply")?;
        resp.error_for_status_ref()
            .context("POST /permission/{requestID}/reply status")?;
        resp.json()
            .await
            .context("parse reply_permission_request response")
    }

    /// `GET /permission`
    pub async fn list_permissions(&self) -> Result<Vec<PermissionRequest>> {
        let resp = self
            .client
            .get(self.url("/permission"))
            .send()
            .await
            .context("GET /permission")?;
        resp.error_for_status_ref()
            .context("GET /permission status")?;
        resp.json().await.context("parse list_permissions response")
    }

    /// `GET /question`
    pub async fn list_questions(&self) -> Result<Vec<QuestionRequest>> {
        let resp = self
            .client
            .get(self.url("/question"))
            .send()
            .await
            .context("GET /question")?;
        resp.error_for_status_ref()
            .context("GET /question status")?;
        resp.json().await.context("parse list_questions response")
    }

    /// `POST /question/{requestID}/reply`
    pub async fn reply_question(
        &self,
        request_id: &str,
        answers: &serde_json::Value,
    ) -> Result<bool> {
        let body = serde_json::json!({ "answers": answers });
        let resp = self
            .client
            .post(self.url(&format!("/question/{}/reply", request_id)))
            .json(&body)
            .send()
            .await
            .context("POST /question/{requestID}/reply")?;
        resp.error_for_status_ref()
            .context("POST /question/{requestID}/reply status")?;
        resp.json().await.context("parse reply_question response")
    }

    /// `POST /question/{requestID}/reject`
    pub async fn reject_question(&self, request_id: &str) -> Result<bool> {
        let resp = self
            .client
            .post(self.url(&format!("/question/{}/reject", request_id)))
            .send()
            .await
            .context("POST /question/{requestID}/reject")?;
        resp.error_for_status_ref()
            .context("POST /question/{requestID}/reject status")?;
        resp.json().await.context("parse reject_question response")
    }

    // =======================================================================
    // Files & Search
    // =======================================================================

    /// `GET /find?pattern=`
    pub async fn find(&self, pattern: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/find"))
            .query(&[("pattern", pattern)])
            .send()
            .await
            .context("GET /find")?;
        resp.error_for_status_ref().context("GET /find status")?;
        resp.json().await.context("parse find response")
    }

    /// `GET /find/file`
    pub async fn find_file(
        &self,
        query: &str,
        file_type: Option<&str>,
        directory: Option<&str>,
        limit: Option<u32>,
        dirs: Option<bool>,
    ) -> Result<Vec<String>> {
        let mut params: Vec<(&str, String)> = vec![("query", query.to_string())];
        if let Some(ft) = file_type {
            params.push(("type", ft.to_string()));
        }
        if let Some(dir) = directory {
            params.push(("directory", dir.to_string()));
        }
        if let Some(l) = limit {
            params.push(("limit", l.to_string()));
        }
        if let Some(d) = dirs {
            params.push(("dirs", d.to_string()));
        }
        let resp = self
            .client
            .get(self.url("/find/file"))
            .query(&params)
            .send()
            .await
            .context("GET /find/file")?;
        resp.error_for_status_ref()
            .context("GET /find/file status")?;
        resp.json().await.context("parse find_file response")
    }

    /// `GET /find/symbol`
    pub async fn find_symbol(&self, query: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/find/symbol"))
            .query(&[("query", query)])
            .send()
            .await
            .context("GET /find/symbol")?;
        resp.error_for_status_ref()
            .context("GET /find/symbol status")?;
        resp.json().await.context("parse find_symbol response")
    }

    /// `GET /file?path=`
    pub async fn list_files(&self, path: &str) -> Result<Vec<FileNode>> {
        let resp = self
            .client
            .get(self.url("/file"))
            .query(&[("path", path)])
            .send()
            .await
            .context("GET /file")?;
        resp.error_for_status_ref().context("GET /file status")?;
        resp.json().await.context("parse list_files response")
    }

    /// `GET /file/content?path=`
    pub async fn get_file_content(&self, path: &str) -> Result<FileContent> {
        let resp = self
            .client
            .get(self.url("/file/content"))
            .query(&[("path", path)])
            .send()
            .await
            .context("GET /file/content")?;
        resp.error_for_status_ref()
            .context("GET /file/content status")?;
        resp.json().await.context("parse get_file_content response")
    }

    /// `GET /file/status`
    pub async fn get_file_status(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/file/status"))
            .send()
            .await
            .context("GET /file/status")?;
        resp.error_for_status_ref()
            .context("GET /file/status status")?;
        resp.json().await.context("parse get_file_status response")
    }

    // =======================================================================
    // Commands, Agents, Skills
    // =======================================================================

    /// `GET /command`
    pub async fn list_commands(&self) -> Result<Vec<Command>> {
        let resp = self
            .client
            .get(self.url("/command"))
            .send()
            .await
            .context("GET /command")?;
        resp.error_for_status_ref().context("GET /command status")?;
        resp.json().await.context("parse list_commands response")
    }

    /// `GET /agent`
    pub async fn list_agents(&self) -> Result<Vec<Agent>> {
        let resp = self
            .client
            .get(self.url("/agent"))
            .send()
            .await
            .context("GET /agent")?;
        resp.error_for_status_ref().context("GET /agent status")?;
        resp.json().await.context("parse list_agents response")
    }

    /// `GET /skill`
    pub async fn list_skills(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/skill"))
            .send()
            .await
            .context("GET /skill")?;
        resp.error_for_status_ref().context("GET /skill status")?;
        resp.json().await.context("parse list_skills response")
    }

    // =======================================================================
    // MCP
    // =======================================================================

    /// `GET /mcp`
    pub async fn list_mcp(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/mcp"))
            .send()
            .await
            .context("GET /mcp")?;
        resp.error_for_status_ref().context("GET /mcp status")?;
        resp.json().await.context("parse list_mcp response")
    }

    /// `POST /mcp`
    pub async fn create_mcp(
        &self,
        name: &str,
        config: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "name": name,
            "config": config,
        });
        let resp = self
            .client
            .post(self.url("/mcp"))
            .json(&body)
            .send()
            .await
            .context("POST /mcp")?;
        resp.error_for_status_ref().context("POST /mcp status")?;
        resp.json().await.context("parse create_mcp response")
    }

    /// `POST /mcp/{name}/connect`
    pub async fn connect_mcp(&self, name: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/mcp/{}/connect", name)))
            .send()
            .await
            .context("POST /mcp/{name}/connect")?;
        resp.error_for_status_ref()
            .context("POST /mcp/{name}/connect status")?;
        resp.json().await.context("parse connect_mcp response")
    }

    /// `POST /mcp/{name}/disconnect`
    pub async fn disconnect_mcp(&self, name: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/mcp/{}/disconnect", name)))
            .send()
            .await
            .context("POST /mcp/{name}/disconnect")?;
        resp.error_for_status_ref()
            .context("POST /mcp/{name}/disconnect status")?;
        resp.json().await.context("parse disconnect_mcp response")
    }

    /// `POST /mcp/{name}/auth` -- Start MCP OAuth flow.
    pub async fn mcp_auth_start(&self, name: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/mcp/{}/auth", name)))
            .send()
            .await
            .context("POST /mcp/{name}/auth")?;
        resp.error_for_status_ref()
            .context("POST /mcp/{name}/auth status")?;
        resp.json().await.context("parse mcp_auth_start response")
    }

    /// `DELETE /mcp/{name}/auth` -- Remove MCP OAuth credentials.
    pub async fn mcp_auth_remove(&self, name: &str) -> Result<serde_json::Value> {
        let resp = self
            .client
            .delete(self.url(&format!("/mcp/{}/auth", name)))
            .send()
            .await
            .context("DELETE /mcp/{name}/auth")?;
        resp.error_for_status_ref()
            .context("DELETE /mcp/{name}/auth status")?;
        resp.json().await.context("parse mcp_auth_remove response")
    }

    /// `POST /mcp/{name}/auth/authenticate`
    pub async fn mcp_auth_authenticate(
        &self,
        name: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/mcp/{}/auth/authenticate", name)))
            .json(body)
            .send()
            .await
            .context("POST /mcp/{name}/auth/authenticate")?;
        resp.error_for_status_ref()
            .context("POST /mcp/{name}/auth/authenticate status")?;
        resp.json()
            .await
            .context("parse mcp_auth_authenticate response")
    }

    /// `POST /mcp/{name}/auth/callback`
    pub async fn mcp_auth_callback(
        &self,
        name: &str,
        body: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let resp = self
            .client
            .post(self.url(&format!("/mcp/{}/auth/callback", name)))
            .json(body)
            .send()
            .await
            .context("POST /mcp/{name}/auth/callback")?;
        resp.error_for_status_ref()
            .context("POST /mcp/{name}/auth/callback status")?;
        resp.json()
            .await
            .context("parse mcp_auth_callback response")
    }

    // =======================================================================
    // PTY
    // =======================================================================

    /// `GET /pty`
    pub async fn list_ptys(&self) -> Result<Vec<Pty>> {
        let resp = self
            .client
            .get(self.url("/pty"))
            .send()
            .await
            .context("GET /pty")?;
        resp.error_for_status_ref().context("GET /pty status")?;
        resp.json().await.context("parse list_ptys response")
    }

    /// `POST /pty`
    pub async fn create_pty(&self) -> Result<Pty> {
        let resp = self
            .client
            .post(self.url("/pty"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /pty")?;
        resp.error_for_status_ref().context("POST /pty status")?;
        resp.json().await.context("parse create_pty response")
    }

    /// `GET /pty/{ptyID}`
    pub async fn get_pty(&self, id: &str) -> Result<Pty> {
        let resp = self
            .client
            .get(self.url(&format!("/pty/{}", id)))
            .send()
            .await
            .context("GET /pty/{ptyID}")?;
        resp.error_for_status_ref()
            .context("GET /pty/{ptyID} status")?;
        resp.json().await.context("parse get_pty response")
    }

    /// `PUT /pty/{ptyID}`
    pub async fn update_pty(&self, id: &str, body: &serde_json::Value) -> Result<Pty> {
        let resp = self
            .client
            .put(self.url(&format!("/pty/{}", id)))
            .json(body)
            .send()
            .await
            .context("PUT /pty/{ptyID}")?;
        resp.error_for_status_ref()
            .context("PUT /pty/{ptyID} status")?;
        resp.json().await.context("parse update_pty response")
    }

    /// `DELETE /pty/{ptyID}`
    pub async fn delete_pty(&self, id: &str) -> Result<bool> {
        let resp = self
            .client
            .delete(self.url(&format!("/pty/{}", id)))
            .send()
            .await
            .context("DELETE /pty/{ptyID}")?;
        resp.error_for_status_ref()
            .context("DELETE /pty/{ptyID} status")?;
        resp.json().await.context("parse delete_pty response")
    }

    // =======================================================================
    // LSP, Formatters
    // =======================================================================

    /// `GET /lsp`
    pub async fn list_lsp(&self) -> Result<Vec<LSPStatus>> {
        let resp = self
            .client
            .get(self.url("/lsp"))
            .send()
            .await
            .context("GET /lsp")?;
        resp.error_for_status_ref().context("GET /lsp status")?;
        resp.json().await.context("parse list_lsp response")
    }

    /// `GET /formatter`
    pub async fn list_formatters(&self) -> Result<Vec<FormatterStatus>> {
        let resp = self
            .client
            .get(self.url("/formatter"))
            .send()
            .await
            .context("GET /formatter")?;
        resp.error_for_status_ref()
            .context("GET /formatter status")?;
        resp.json().await.context("parse list_formatters response")
    }

    // =======================================================================
    // Misc
    // =======================================================================

    /// `GET /path`
    pub async fn get_path(&self) -> Result<Path> {
        let resp = self
            .client
            .get(self.url("/path"))
            .send()
            .await
            .context("GET /path")?;
        resp.error_for_status_ref().context("GET /path status")?;
        resp.json().await.context("parse get_path response")
    }

    /// `GET /vcs`
    pub async fn get_vcs(&self) -> Result<serde_json::Value> {
        let resp = self
            .client
            .get(self.url("/vcs"))
            .send()
            .await
            .context("GET /vcs")?;
        resp.error_for_status_ref().context("GET /vcs status")?;
        resp.json().await.context("parse get_vcs response")
    }

    /// `POST /log`
    pub async fn log(
        &self,
        service: &str,
        level: &str,
        message: &str,
        extra: Option<&serde_json::Value>,
    ) -> Result<bool> {
        let mut body = serde_json::json!({
            "service": service,
            "level": level,
            "message": message,
        });
        if let Some(e) = extra {
            body.as_object_mut()
                .unwrap()
                .insert("extra".into(), e.clone());
        }
        let resp = self
            .client
            .post(self.url("/log"))
            .json(&body)
            .send()
            .await
            .context("POST /log")?;
        resp.error_for_status_ref().context("POST /log status")?;
        resp.json().await.context("parse log response")
    }

    /// `POST /instance/dispose`
    pub async fn dispose_instance(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/instance/dispose"))
            .send()
            .await
            .context("POST /instance/dispose")?;
        resp.error_for_status_ref()
            .context("POST /instance/dispose status")?;
        resp.json().await.context("parse dispose_instance response")
    }

    // =======================================================================
    // TUI Control (for remote control of the TUI)
    // =======================================================================

    /// `POST /tui/append-prompt`
    pub async fn tui_append_prompt(&self, text: &str) -> Result<bool> {
        let body = serde_json::json!({ "text": text });
        let resp = self
            .client
            .post(self.url("/tui/append-prompt"))
            .json(&body)
            .send()
            .await
            .context("POST /tui/append-prompt")?;
        resp.error_for_status_ref()
            .context("POST /tui/append-prompt status")?;
        resp.json()
            .await
            .context("parse tui_append_prompt response")
    }

    /// `POST /tui/clear-prompt`
    pub async fn tui_clear_prompt(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/clear-prompt"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/clear-prompt")?;
        resp.error_for_status_ref()
            .context("POST /tui/clear-prompt status")?;
        resp.json().await.context("parse tui_clear_prompt response")
    }

    /// `POST /tui/submit-prompt`
    pub async fn tui_submit_prompt(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/submit-prompt"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/submit-prompt")?;
        resp.error_for_status_ref()
            .context("POST /tui/submit-prompt status")?;
        resp.json()
            .await
            .context("parse tui_submit_prompt response")
    }

    /// `POST /tui/execute-command`
    pub async fn tui_execute_command(&self, command: &str) -> Result<bool> {
        let body = serde_json::json!({ "command": command });
        let resp = self
            .client
            .post(self.url("/tui/execute-command"))
            .json(&body)
            .send()
            .await
            .context("POST /tui/execute-command")?;
        resp.error_for_status_ref()
            .context("POST /tui/execute-command status")?;
        resp.json()
            .await
            .context("parse tui_execute_command response")
    }

    /// `POST /tui/show-toast`
    pub async fn tui_show_toast(
        &self,
        message: &str,
        variant: &str,
        title: Option<&str>,
        duration: Option<u64>,
    ) -> Result<bool> {
        let mut body = serde_json::json!({
            "message": message,
            "variant": variant,
        });
        let obj = body.as_object_mut().unwrap();
        if let Some(t) = title {
            obj.insert("title".into(), serde_json::Value::String(t.into()));
        }
        if let Some(d) = duration {
            obj.insert("duration".into(), serde_json::json!(d));
        }
        let resp = self
            .client
            .post(self.url("/tui/show-toast"))
            .json(&body)
            .send()
            .await
            .context("POST /tui/show-toast")?;
        resp.error_for_status_ref()
            .context("POST /tui/show-toast status")?;
        resp.json().await.context("parse tui_show_toast response")
    }

    /// `POST /tui/open-help`
    pub async fn tui_open_help(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/open-help"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/open-help")?;
        resp.error_for_status_ref()
            .context("POST /tui/open-help status")?;
        resp.json().await.context("parse tui_open_help response")
    }

    /// `POST /tui/open-sessions`
    pub async fn tui_open_sessions(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/open-sessions"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/open-sessions")?;
        resp.error_for_status_ref()
            .context("POST /tui/open-sessions status")?;
        resp.json()
            .await
            .context("parse tui_open_sessions response")
    }

    /// `POST /tui/open-themes`
    pub async fn tui_open_themes(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/open-themes"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/open-themes")?;
        resp.error_for_status_ref()
            .context("POST /tui/open-themes status")?;
        resp.json().await.context("parse tui_open_themes response")
    }

    /// `POST /tui/open-models`
    pub async fn tui_open_models(&self) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/open-models"))
            .json(&serde_json::json!({}))
            .send()
            .await
            .context("POST /tui/open-models")?;
        resp.error_for_status_ref()
            .context("POST /tui/open-models status")?;
        resp.json().await.context("parse tui_open_models response")
    }

    /// `POST /tui/select-session`
    pub async fn tui_select_session(&self, session_id: &str) -> Result<bool> {
        let body = serde_json::json!({ "sessionID": session_id });
        let resp = self
            .client
            .post(self.url("/tui/select-session"))
            .json(&body)
            .send()
            .await
            .context("POST /tui/select-session")?;
        resp.error_for_status_ref()
            .context("POST /tui/select-session status")?;
        resp.json()
            .await
            .context("parse tui_select_session response")
    }

    /// `POST /tui/publish` -- Publish a TUI event to connected clients.
    pub async fn tui_publish(&self, event: &serde_json::Value) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/publish"))
            .json(event)
            .send()
            .await
            .context("POST /tui/publish")?;
        resp.error_for_status_ref()
            .context("POST /tui/publish status")?;
        resp.json().await.context("parse tui_publish response")
    }

    /// `GET /tui/control/next` -- Long-poll for the next TUI control request.
    pub async fn tui_control_next(&self) -> Result<TuiControlRequest> {
        let resp = self
            .client
            .get(self.url("/tui/control/next"))
            .send()
            .await
            .context("GET /tui/control/next")?;
        resp.error_for_status_ref()
            .context("GET /tui/control/next status")?;
        resp.json().await.context("parse tui_control_next response")
    }

    /// `POST /tui/control/response` -- Send a response to a TUI control request.
    pub async fn tui_control_response(&self, body: &serde_json::Value) -> Result<bool> {
        let resp = self
            .client
            .post(self.url("/tui/control/response"))
            .json(body)
            .send()
            .await
            .context("POST /tui/control/response")?;
        resp.error_for_status_ref()
            .context("POST /tui/control/response status")?;
        resp.json()
            .await
            .context("parse tui_control_response response")
    }
}
