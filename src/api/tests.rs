use serde_json::json;

use super::client::{
    ApiClient, HealthResponse, ModelSelector, PartInput, SendCommandRequest, SendMessageRequest,
    TuiControlRequest,
};
use super::events::parse_global_event_data;
use crate::types::Event;

// ===========================================================================
// ApiClient construction
// ===========================================================================

#[test]
fn api_client_new_stores_base_url() {
    let client = ApiClient::new("http://127.0.0.1:3000".to_string());
    assert_eq!(client.base_url(), "http://127.0.0.1:3000");
}

#[test]
fn api_client_new_strips_trailing_slash() {
    let client = ApiClient::new("http://127.0.0.1:3000/".to_string());
    assert_eq!(client.base_url(), "http://127.0.0.1:3000");
}

#[test]
fn api_client_new_strips_multiple_trailing_slashes() {
    let client = ApiClient::new("http://127.0.0.1:3000///".to_string());
    assert_eq!(client.base_url(), "http://127.0.0.1:3000");
}

#[test]
fn api_client_clone_preserves_base_url() {
    let client = ApiClient::new("http://localhost:8080".to_string());
    let cloned = client.clone();
    assert_eq!(cloned.base_url(), "http://localhost:8080");
}

// ===========================================================================
// HealthResponse deserialization
// ===========================================================================

#[test]
fn health_response_deserialize_valid() {
    let json_str = r#"{"healthy":true,"version":"1.2.3"}"#;
    let resp: HealthResponse = serde_json::from_str(json_str).unwrap();
    assert!(resp.healthy);
    assert_eq!(resp.version, "1.2.3");
}

#[test]
fn health_response_deserialize_unhealthy() {
    let json_str = r#"{"healthy":false,"version":"0.0.1"}"#;
    let resp: HealthResponse = serde_json::from_str(json_str).unwrap();
    assert!(!resp.healthy);
    assert_eq!(resp.version, "0.0.1");
}

#[test]
fn health_response_roundtrip() {
    let original = HealthResponse {
        healthy: true,
        version: "2.0.0".to_string(),
    };
    let serialized = serde_json::to_string(&original).unwrap();
    let deserialized: HealthResponse = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.healthy, original.healthy);
    assert_eq!(deserialized.version, original.version);
}

// ===========================================================================
// ModelSelector serialization
// ===========================================================================

#[test]
fn model_selector_serializes_with_correct_field_names() {
    let selector = ModelSelector {
        provider_id: "anthropic".to_string(),
        model_id: "claude-3-opus".to_string(),
    };
    let value = serde_json::to_value(&selector).unwrap();
    // Verify the JSON uses providerID and modelID (not camelCase defaults).
    assert_eq!(value["providerID"], "anthropic");
    assert_eq!(value["modelID"], "claude-3-opus");
    // Ensure snake_case fields are NOT present.
    assert!(value.get("provider_id").is_none());
    assert!(value.get("model_id").is_none());
}

#[test]
fn model_selector_deserialize_from_wire_format() {
    let json_str = r#"{"providerID":"openai","modelID":"gpt-4"}"#;
    let selector: ModelSelector = serde_json::from_str(json_str).unwrap();
    assert_eq!(selector.provider_id, "openai");
    assert_eq!(selector.model_id, "gpt-4");
}

#[test]
fn model_selector_roundtrip() {
    let original = ModelSelector {
        provider_id: "google".to_string(),
        model_id: "gemini-pro".to_string(),
    };
    let json_str = serde_json::to_string(&original).unwrap();
    let restored: ModelSelector = serde_json::from_str(&json_str).unwrap();
    assert_eq!(restored.provider_id, original.provider_id);
    assert_eq!(restored.model_id, original.model_id);
}

// ===========================================================================
// PartInput serialization
// ===========================================================================

#[test]
fn part_input_text_serializes_with_type_tag() {
    let part = PartInput::Text {
        id: None,
        text: "hello world".to_string(),
        synthetic: None,
        ignored: None,
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "text");
    assert_eq!(value["text"], "hello world");
    // Optional fields with None should be absent.
    assert!(value.get("id").is_none());
    assert!(value.get("synthetic").is_none());
    assert!(value.get("ignored").is_none());
}

#[test]
fn part_input_text_with_optional_fields() {
    let part = PartInput::Text {
        id: Some("part-1".to_string()),
        text: "some text".to_string(),
        synthetic: Some(true),
        ignored: Some(false),
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "text");
    assert_eq!(value["id"], "part-1");
    assert_eq!(value["text"], "some text");
    assert_eq!(value["synthetic"], true);
    assert_eq!(value["ignored"], false);
}

#[test]
fn part_input_file_serializes_with_type_tag() {
    let part = PartInput::File {
        id: None,
        mime: "image/png".to_string(),
        url: "https://example.com/image.png".to_string(),
        filename: Some("image.png".to_string()),
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "file");
    assert_eq!(value["mime"], "image/png");
    assert_eq!(value["url"], "https://example.com/image.png");
    assert_eq!(value["filename"], "image.png");
    assert!(value.get("id").is_none());
}

#[test]
fn part_input_file_without_filename() {
    let part = PartInput::File {
        id: Some("file-1".to_string()),
        mime: "application/pdf".to_string(),
        url: "https://example.com/doc.pdf".to_string(),
        filename: None,
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "file");
    assert_eq!(value["id"], "file-1");
    assert!(value.get("filename").is_none());
}

#[test]
fn part_input_agent_serializes_with_type_tag() {
    let part = PartInput::Agent {
        id: None,
        name: "coder".to_string(),
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "agent");
    assert_eq!(value["name"], "coder");
}

#[test]
fn part_input_subtask_serializes_with_type_tag() {
    let part = PartInput::Subtask {
        id: None,
        prompt: "Fix the bug".to_string(),
        description: "Debug segfault".to_string(),
        agent: "coder".to_string(),
        model: None,
        command: Some("/fix".to_string()),
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "subtask");
    assert_eq!(value["prompt"], "Fix the bug");
    assert_eq!(value["description"], "Debug segfault");
    assert_eq!(value["agent"], "coder");
    assert!(value.get("model").is_none());
    assert_eq!(value["command"], "/fix");
}

#[test]
fn part_input_text_deserialize() {
    let json_str = r#"{"type":"text","text":"hello"}"#;
    let part: PartInput = serde_json::from_str(json_str).unwrap();
    match part {
        PartInput::Text { text, id, .. } => {
            assert_eq!(text, "hello");
            assert!(id.is_none());
        }
        _ => panic!("expected PartInput::Text"),
    }
}

#[test]
fn part_input_file_deserialize() {
    let json_str = r#"{"type":"file","mime":"text/plain","url":"file:///tmp/f.txt"}"#;
    let part: PartInput = serde_json::from_str(json_str).unwrap();
    match part {
        PartInput::File { mime, url, .. } => {
            assert_eq!(mime, "text/plain");
            assert_eq!(url, "file:///tmp/f.txt");
        }
        _ => panic!("expected PartInput::File"),
    }
}

// ===========================================================================
// SendMessageRequest serialization
// ===========================================================================

#[test]
fn send_message_request_minimal() {
    let req = SendMessageRequest {
        parts: vec![PartInput::Text {
            id: None,
            text: "Hello".to_string(),
            synthetic: None,
            ignored: None,
        }],
        message_id: None,
        model: None,
        agent: None,
        no_reply: None,
        system: None,
        variant: None,
        format: None,
    };
    let value = serde_json::to_value(&req).unwrap();
    assert!(value["parts"].is_array());
    assert_eq!(value["parts"][0]["type"], "text");
    assert_eq!(value["parts"][0]["text"], "Hello");
    // Optional None fields should be absent.
    assert!(value.get("messageID").is_none());
    assert!(value.get("model").is_none());
    assert!(value.get("agent").is_none());
    assert!(value.get("noReply").is_none());
    assert!(value.get("system").is_none());
    assert!(value.get("variant").is_none());
    assert!(value.get("format").is_none());
}

#[test]
fn send_message_request_with_model_and_message_id() {
    let req = SendMessageRequest {
        parts: vec![PartInput::Text {
            id: None,
            text: "What is Rust?".to_string(),
            synthetic: None,
            ignored: None,
        }],
        message_id: Some("msg-123".to_string()),
        model: Some(ModelSelector {
            provider_id: "anthropic".to_string(),
            model_id: "claude-3-opus".to_string(),
        }),
        agent: Some("coder".to_string()),
        no_reply: Some(false),
        system: None,
        variant: None,
        format: None,
    };
    let value = serde_json::to_value(&req).unwrap();
    assert_eq!(value["messageID"], "msg-123");
    assert_eq!(value["model"]["providerID"], "anthropic");
    assert_eq!(value["model"]["modelID"], "claude-3-opus");
    assert_eq!(value["agent"], "coder");
    assert_eq!(value["noReply"], false);
}

#[test]
fn send_message_request_roundtrip() {
    let req = SendMessageRequest {
        parts: vec![
            PartInput::Text {
                id: Some("p1".to_string()),
                text: "first".to_string(),
                synthetic: None,
                ignored: None,
            },
            PartInput::File {
                id: None,
                mime: "image/jpeg".to_string(),
                url: "https://example.com/img.jpg".to_string(),
                filename: None,
            },
        ],
        message_id: Some("m-42".to_string()),
        model: Some(ModelSelector {
            provider_id: "openai".to_string(),
            model_id: "gpt-4".to_string(),
        }),
        agent: None,
        no_reply: None,
        system: Some("You are helpful.".to_string()),
        variant: Some("v2".to_string()),
        format: Some(json!({"type": "json"})),
    };
    let json_str = serde_json::to_string(&req).unwrap();
    let restored: SendMessageRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(restored.parts.len(), 2);
    assert_eq!(restored.message_id.as_deref(), Some("m-42"));
    assert_eq!(restored.system.as_deref(), Some("You are helpful."));
    assert_eq!(restored.variant.as_deref(), Some("v2"));
}

// ===========================================================================
// SendCommandRequest serialization
// ===========================================================================

#[test]
fn send_command_request_minimal() {
    let req = SendCommandRequest {
        command: "/help".to_string(),
        arguments: "".to_string(),
        agent: None,
        model: None,
        variant: None,
        message_id: None,
        parts: None,
    };
    let value = serde_json::to_value(&req).unwrap();
    assert_eq!(value["command"], "/help");
    assert_eq!(value["arguments"], "");
    assert!(value.get("agent").is_none());
    assert!(value.get("model").is_none());
    assert!(value.get("messageID").is_none());
}

#[test]
fn send_command_request_with_all_fields() {
    let req = SendCommandRequest {
        command: "/run".to_string(),
        arguments: "test --release".to_string(),
        agent: Some("builder".to_string()),
        model: Some("gpt-4".to_string()),
        variant: Some("fast".to_string()),
        message_id: Some("cmd-99".to_string()),
        parts: Some(vec![json!({"type": "text", "text": "extra"})]),
    };
    let value = serde_json::to_value(&req).unwrap();
    assert_eq!(value["command"], "/run");
    assert_eq!(value["arguments"], "test --release");
    assert_eq!(value["agent"], "builder");
    assert_eq!(value["model"], "gpt-4");
    assert_eq!(value["variant"], "fast");
    assert_eq!(value["messageID"], "cmd-99");
    assert!(value["parts"].is_array());
}

// ===========================================================================
// TuiControlRequest serialization
// ===========================================================================

#[test]
fn tui_control_request_deserialize() {
    let json_str = r#"{"path":"/tui/append-prompt","body":{"text":"hello"}}"#;
    let req: TuiControlRequest = serde_json::from_str(json_str).unwrap();
    assert_eq!(req.path, "/tui/append-prompt");
    assert_eq!(req.body["text"], "hello");
}

#[test]
fn tui_control_request_roundtrip() {
    let original = TuiControlRequest {
        path: "/tui/execute-command".to_string(),
        body: json!({"command": "/quit"}),
    };
    let json_str = serde_json::to_string(&original).unwrap();
    let restored: TuiControlRequest = serde_json::from_str(&json_str).unwrap();
    assert_eq!(restored.path, original.path);
    assert_eq!(restored.body, original.body);
}

// ===========================================================================
// Event deserialization
// ===========================================================================

#[test]
fn event_session_created_deserialize() {
    let json_str = json!({
        "type": "session.created",
        "properties": {
            "info": {
                "id": "sess-1",
                "slug": null,
                "projectID": "proj-1",
                "directory": null,
                "parentID": null,
                "summary": null,
                "share": null,
                "title": "Test Session",
                "version": "1.0.0",
                "time": {
                    "created": 1704067200000_i64,
                    "updated": 1704067200000_i64
                },
                "permission": null,
                "revert": null
            }
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::SessionCreated { properties } => {
            assert_eq!(properties.info.id, "sess-1");
            assert_eq!(properties.info.title.as_deref(), Some("Test Session"));
        }
        _ => panic!("expected Event::SessionCreated"),
    }
}

#[test]
fn event_session_idle_deserialize() {
    let json_str = json!({
        "type": "session.idle",
        "properties": {
            "sessionID": "sess-42"
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::SessionIdle { properties } => {
            assert_eq!(properties.session_id, "sess-42");
        }
        _ => panic!("expected Event::SessionIdle"),
    }
}

#[test]
fn event_session_status_busy_deserialize() {
    let json_str = json!({
        "type": "session.status",
        "properties": {
            "sessionID": "sess-7",
            "status": {
                "type": "busy"
            }
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::SessionStatus { properties } => {
            assert_eq!(properties.session_id, "sess-7");
        }
        _ => panic!("expected Event::SessionStatus"),
    }
}

#[test]
fn event_vcs_branch_updated_deserialize() {
    let json_str = json!({
        "type": "vcs.branch.updated",
        "properties": {
            "branch": "feature/cool-stuff"
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::VcsBranchUpdated { properties } => {
            assert_eq!(properties.branch.as_deref(), Some("feature/cool-stuff"));
        }
        _ => panic!("expected Event::VcsBranchUpdated"),
    }
}

#[test]
fn event_server_connected_deserialize() {
    let json_str = json!({
        "type": "server.connected",
        "properties": {}
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::ServerConnected { .. } => {}
        _ => panic!("expected Event::ServerConnected"),
    }
}

#[test]
fn event_tui_toast_show_deserialize() {
    let json_str = json!({
        "type": "tui.toast.show",
        "properties": {
            "title": "Error",
            "message": "Something went wrong",
            "variant": "error"
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::TuiToastShow { properties } => {
            assert_eq!(properties.title.as_deref(), Some("Error"));
            assert_eq!(properties.message.as_deref(), Some("Something went wrong"));
            assert_eq!(properties.variant.as_deref(), Some("error"));
        }
        _ => panic!("expected Event::TuiToastShow"),
    }
}

// ===========================================================================
// SSE global event parsing (parse_global_event_data)
// ===========================================================================

#[test]
fn parse_global_event_data_extracts_payload() {
    let data = json!({
        "directory": "/home/user/project",
        "payload": {
            "type": "session.idle",
            "properties": {
                "sessionID": "sess-99"
            }
        }
    })
    .to_string();

    let event = parse_global_event_data(&data).unwrap();
    match event {
        Event::SessionIdle { properties } => {
            assert_eq!(properties.session_id, "sess-99");
        }
        _ => panic!("expected Event::SessionIdle from global payload"),
    }
}

#[test]
fn parse_global_event_data_extracts_session_created_payload() {
    let data = json!({
        "directory": "/tmp/workspace",
        "payload": {
            "type": "session.created",
            "properties": {
                "info": {
                    "id": "s-abc",
                    "slug": null,
                    "projectID": "p-1",
                    "directory": null,
                    "parentID": null,
                    "summary": null,
                    "share": null,
                    "title": "New Session",
                    "version": "1.0.0",
                    "time": {
                        "created": 1717243200000_i64,
                        "updated": 1717243200000_i64
                    },
                    "permission": null,
                    "revert": null
                }
            }
        }
    })
    .to_string();

    let event = parse_global_event_data(&data).unwrap();
    match event {
        Event::SessionCreated { properties } => {
            assert_eq!(properties.info.id, "s-abc");
            assert_eq!(properties.info.title.as_deref(), Some("New Session"));
        }
        _ => panic!("expected Event::SessionCreated from global payload"),
    }
}

#[test]
fn parse_global_event_data_fallback_without_payload_key() {
    // When "payload" key is absent, the function falls back to parsing the
    // entire object as an Event. This should work if the object is shaped
    // like a valid Event.
    let data = json!({
        "type": "vcs.branch.updated",
        "properties": {
            "branch": "main"
        }
    })
    .to_string();

    let event = parse_global_event_data(&data).unwrap();
    match event {
        Event::VcsBranchUpdated { properties } => {
            assert_eq!(properties.branch.as_deref(), Some("main"));
        }
        _ => panic!("expected Event::VcsBranchUpdated from fallback"),
    }
}

#[test]
fn parse_global_event_data_rejects_invalid_json() {
    let result = parse_global_event_data("not json at all");
    assert!(result.is_err());
}

#[test]
fn parse_global_event_data_rejects_invalid_payload() {
    let data = json!({
        "directory": "/home/user",
        "payload": {
            "type": "unknown.event.type.that.does.not.exist",
            "properties": {}
        }
    })
    .to_string();

    let result = parse_global_event_data(&data);
    assert!(result.is_err());
}

// ===========================================================================
// Global event payload parsing
// ===========================================================================

#[test]
fn global_event_payload_parsing() {
    // GlobalEvent struct was removed; test that parse_global_event_data extracts the payload
    let data = r#"{"directory":"/workspace","payload":{"type":"server.connected","properties":{}}}"#;
    let event = parse_global_event_data(data).unwrap();
    match event {
        Event::ServerConnected { .. } => {}
        _ => panic!("expected Event::ServerConnected"),
    }
}

// ===========================================================================
// Edge cases and regression tests
// ===========================================================================

#[test]
fn send_message_request_empty_parts() {
    let req = SendMessageRequest {
        parts: vec![],
        message_id: None,
        model: None,
        agent: None,
        no_reply: None,
        system: None,
        variant: None,
        format: None,
    };
    let value = serde_json::to_value(&req).unwrap();
    assert!(value["parts"].as_array().unwrap().is_empty());
}

#[test]
fn part_input_subtask_with_model() {
    let part = PartInput::Subtask {
        id: Some("st-1".to_string()),
        prompt: "Do something".to_string(),
        description: "A subtask".to_string(),
        agent: "helper".to_string(),
        model: Some(ModelSelector {
            provider_id: "anthropic".to_string(),
            model_id: "claude-3-haiku".to_string(),
        }),
        command: None,
    };
    let value = serde_json::to_value(&part).unwrap();
    assert_eq!(value["type"], "subtask");
    assert_eq!(value["id"], "st-1");
    assert_eq!(value["model"]["providerID"], "anthropic");
    assert_eq!(value["model"]["modelID"], "claude-3-haiku");
    assert!(value.get("command").is_none());
}

#[test]
fn health_response_rejects_missing_fields() {
    let result = serde_json::from_str::<HealthResponse>(r#"{"healthy":true}"#);
    assert!(result.is_err(), "missing 'version' field should fail");
}

#[test]
fn model_selector_rejects_wrong_field_names() {
    // camelCase field names should NOT work because we use explicit renames.
    let result =
        serde_json::from_str::<ModelSelector>(r#"{"provider_id":"x","model_id":"y"}"#);
    assert!(result.is_err(), "snake_case field names should not deserialize");
}

#[test]
fn event_session_error_with_null_fields() {
    let json_str = json!({
        "type": "session.error",
        "properties": {
            "sessionID": null,
            "error": null
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::SessionError { properties } => {
            assert!(properties.session_id.is_none());
            assert!(properties.error.is_none());
        }
        _ => panic!("expected Event::SessionError"),
    }
}

#[test]
fn event_file_watcher_updated_deserialize() {
    let json_str = json!({
        "type": "file.watcher.updated",
        "properties": {
            "files": ["src/main.rs", "Cargo.toml"]
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::FileWatcherUpdated { properties } => {
            let files = properties.files.unwrap();
            assert_eq!(files.len(), 2);
            assert_eq!(files[0], "src/main.rs");
            assert_eq!(files[1], "Cargo.toml");
        }
        _ => panic!("expected Event::FileWatcherUpdated"),
    }
}

#[test]
fn event_command_executed_deserialize() {
    let json_str = json!({
        "type": "command.executed",
        "properties": {
            "command": "cargo",
            "args": ["build", "--release"]
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::CommandExecuted { properties } => {
            assert_eq!(properties.command.as_deref(), Some("cargo"));
            let args = properties.args.unwrap();
            assert_eq!(args, vec!["build", "--release"]);
        }
        _ => panic!("expected Event::CommandExecuted"),
    }
}

#[test]
fn event_message_part_delta_deserialize() {
    let json_str = json!({
        "type": "message.part.delta",
        "properties": {
            "sessionID": "s-1",
            "messageID": "m-1",
            "partID": "p-1",
            "field": "content",
            "delta": "some new text"
        }
    });
    let event: Event = serde_json::from_value(json_str).unwrap();
    match event {
        Event::MessagePartDelta { properties } => {
            assert_eq!(properties.session_id, "s-1");
            assert_eq!(properties.message_id, "m-1");
            assert_eq!(properties.part_id, "p-1");
            assert_eq!(properties.field, "content");
            assert_eq!(properties.delta, "some new text");
        }
        _ => panic!("expected Event::MessagePartDelta"),
    }
}
