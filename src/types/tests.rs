use super::*;
use serde_json::json;

// ============================================================================
// Session tests
// ============================================================================

#[test]
fn test_session_deserialize_full() {
    let json = json!({
        "id": "sess_abc123",
        "slug": "my-session",
        "projectID": "proj_xyz",
        "directory": "/home/user/project",
        "parentID": "sess_parent",
        "summary": {
            "title": "Implement feature X",
            "description": "Working on the new feature",
            "additions": 10,
            "deletions": 3,
            "files": 2
        },
        "share": true,
        "title": "Feature X Session",
        "version": "3",
        "time": {
            "created": 1705312200000i64,
            "updated": 1705314000000i64
        },
        "permission": {
            "file:write": true
        },
        "revert": {
            "sessionID": "sess_old",
            "messageID": "msg_old"
        }
    });

    let session: Session = serde_json::from_value(json).unwrap();
    assert_eq!(session.id, "sess_abc123");
    assert_eq!(session.slug.as_deref(), Some("my-session"));
    assert_eq!(session.project_id.as_deref(), Some("proj_xyz"));
    assert_eq!(session.directory.as_deref(), Some("/home/user/project"));
    assert_eq!(session.parent_id.as_deref(), Some("sess_parent"));
    assert!(session.summary.is_some());
    let summary = session.summary.unwrap();
    assert_eq!(summary.title.as_deref(), Some("Implement feature X"));
    assert_eq!(
        summary.description.as_deref(),
        Some("Working on the new feature")
    );
    assert_eq!(summary.additions, Some(10));
    assert_eq!(session.share, Some(true));
    assert_eq!(session.title.as_deref(), Some("Feature X Session"));
    assert_eq!(session.version.as_deref(), Some("3"));
    assert_eq!(session.time.created, json!(1705312200000i64));
    assert_eq!(session.time.updated, json!(1705314000000i64));
    assert!(session.permission.is_some());
    let revert = session.revert.unwrap();
    assert_eq!(revert.session_id.as_deref(), Some("sess_old"));
    assert_eq!(revert.message_id.as_deref(), Some("msg_old"));
}

#[test]
fn test_session_deserialize_minimal() {
    let json = json!({
        "id": "sess_min",
        "projectID": "proj_1",
        "time": {
            "created": 1704067200000i64,
            "updated": 1704067200000i64
        }
    });

    let session: Session = serde_json::from_value(json).unwrap();
    assert_eq!(session.id, "sess_min");
    assert_eq!(session.project_id.as_deref(), Some("proj_1"));
    assert!(session.slug.is_none());
    assert!(session.directory.is_none());
    assert!(session.parent_id.is_none());
    assert!(session.summary.is_none());
    assert!(session.share.is_none());
    assert!(session.title.is_none());
    assert!(session.version.is_none());
    assert!(session.permission.is_none());
    assert!(session.revert.is_none());
}

#[test]
fn test_session_roundtrip() {
    let json = json!({
        "id": "sess_rt",
        "projectID": "proj_rt",
        "time": {
            "created": "2025-06-01T12:00:00Z",
            "updated": "2025-06-01T12:30:00Z"
        },
        "title": "Roundtrip test"
    });

    let session: Session = serde_json::from_value(json).unwrap();
    let serialized = serde_json::to_value(&session).unwrap();
    let deserialized: Session = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.id, "sess_rt");
    assert_eq!(deserialized.title.as_deref(), Some("Roundtrip test"));
}

#[test]
fn test_session_status_idle() {
    let json = json!({ "type": "idle" });
    let status: SessionStatus = serde_json::from_value(json).unwrap();
    assert!(matches!(status, SessionStatus::Idle));
}

#[test]
fn test_session_status_busy() {
    let json = json!({ "type": "busy" });
    let status: SessionStatus = serde_json::from_value(json).unwrap();
    assert!(matches!(status, SessionStatus::Busy));
}

#[test]
fn test_session_status_retry_full() {
    let json = json!({
        "type": "retry",
        "attempt": 2,
        "message": "Rate limited",
        "next": "2025-01-15T10:35:00Z"
    });

    let status: SessionStatus = serde_json::from_value(json).unwrap();
    match status {
        SessionStatus::Retry {
            attempt,
            message,
            next,
        } => {
            assert_eq!(attempt, Some(2));
            assert_eq!(message.as_deref(), Some("Rate limited"));
            assert_eq!(next.as_deref(), Some("2025-01-15T10:35:00Z"));
        }
        _ => panic!("Expected SessionStatus::Retry"),
    }
}

#[test]
fn test_session_status_retry_minimal() {
    let json = json!({ "type": "retry" });
    let status: SessionStatus = serde_json::from_value(json).unwrap();
    match status {
        SessionStatus::Retry {
            attempt,
            message,
            next,
        } => {
            assert!(attempt.is_none());
            assert!(message.is_none());
            assert!(next.is_none());
        }
        _ => panic!("Expected SessionStatus::Retry"),
    }
}

// ============================================================================
// Message tests
// ============================================================================

#[test]
fn test_message_user_deserialize() {
    let json = json!({
        "role": "user",
        "id": "msg_user1",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "format": {
            "style": "markdown"
        },
        "summary": "Please help me fix this bug",
        "agent": "default",
        "model": {
            "providerID": "anthropic",
            "modelID": "claude-sonnet-4-20250514"
        },
        "system": false,
        "tools": ["read_file", "write_file"],
        "variant": "primary"
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::User(user) => {
            assert_eq!(user.id, "msg_user1");
            assert_eq!(user.session_id, "sess_1");
            assert_eq!(
                user.summary.as_ref().and_then(|v| v.as_str()),
                Some("Please help me fix this bug")
            );
            assert_eq!(user.agent.as_deref(), Some("default"));
            assert_eq!(user.system, Some(false));
            let tools = user.tools.unwrap();
            assert_eq!(tools.len(), 2);
            assert_eq!(tools[0], "read_file");
            let model = user.model.unwrap();
            assert_eq!(model.provider_id, "anthropic");
            assert_eq!(model.model_id, "claude-sonnet-4-20250514");
            let format = user.format.unwrap();
            assert_eq!(format.style.as_deref(), Some("markdown"));
        }
        _ => panic!("Expected Message::User"),
    }
}

#[test]
fn test_message_user_minimal() {
    let json = json!({
        "role": "user",
        "id": "msg_u_min",
        "sessionID": "sess_2",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        }
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::User(user) => {
            assert_eq!(user.id, "msg_u_min");
            assert!(user.format.is_none());
            assert!(user.summary.is_none());
            assert!(user.agent.is_none());
            assert!(user.model.is_none());
            assert!(user.system.is_none());
            assert!(user.tools.is_none());
            assert!(user.variant.is_none());
        }
        _ => panic!("Expected Message::User"),
    }
}

#[test]
fn test_message_assistant_deserialize() {
    let json = json!({
        "role": "assistant",
        "id": "msg_asst1",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:05Z",
            "updated": "2025-01-15T10:30:45Z",
            "completed": "2025-01-15T10:30:45Z"
        },
        "parentID": "msg_user1",
        "modelID": "claude-sonnet-4-20250514",
        "providerID": "anthropic",
        "mode": "normal",
        "agent": "default",
        "path": ["step1", "step2"],
        "cost": 0.0035,
        "tokens": {
            "input": 1500,
            "output": 800,
            "reasoning": 200,
            "cache": {
                "read": 500,
                "write": 100
            }
        },
        "system": false
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::Assistant(asst) => {
            assert_eq!(asst.id, "msg_asst1");
            assert_eq!(asst.session_id, "sess_1");
            assert_eq!(
                asst.time.completed.as_ref().and_then(|v| v.as_str()),
                Some("2025-01-15T10:30:45Z")
            );
            assert_eq!(asst.parent_id.as_deref(), Some("msg_user1"));
            assert_eq!(
                asst.model_id.as_deref(),
                Some("claude-sonnet-4-20250514")
            );
            assert_eq!(asst.provider_id.as_deref(), Some("anthropic"));
            assert_eq!(asst.mode.as_deref(), Some("normal"));
            assert_eq!(asst.agent.as_deref(), Some("default"));
            let path = asst.path.unwrap();
            assert_eq!(path, json!(["step1", "step2"]));
            assert!((asst.cost.unwrap() - 0.0035).abs() < f64::EPSILON);
            let tokens = asst.tokens.unwrap();
            assert_eq!(tokens.input, Some(1500));
            assert_eq!(tokens.output, Some(800));
            assert_eq!(tokens.reasoning, Some(200));
            let cache = tokens.cache.unwrap();
            assert_eq!(cache.read, Some(500));
            assert_eq!(cache.write, Some(100));
        }
        _ => panic!("Expected Message::Assistant"),
    }
}

#[test]
fn test_message_assistant_with_error() {
    let json = json!({
        "role": "assistant",
        "id": "msg_err",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:05Z",
            "updated": "2025-01-15T10:30:06Z"
        },
        "error": "Context length exceeded"
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::Assistant(asst) => {
            assert_eq!(
                asst.error.as_ref().and_then(|v| v.as_str()),
                Some("Context length exceeded")
            );
            assert!(asst.tokens.is_none());
            assert!(asst.cost.is_none());
        }
        _ => panic!("Expected Message::Assistant"),
    }
}

#[test]
fn test_message_user_empty_tools_vec() {
    let json = json!({
        "role": "user",
        "id": "msg_et",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "tools": []
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::User(user) => {
            let tools = user.tools.unwrap();
            assert!(tools.is_empty());
        }
        _ => panic!("Expected Message::User"),
    }
}

#[test]
fn test_message_with_parts() {
    let json = json!({
        "info": {
            "role": "assistant",
            "id": "msg_wp",
            "sessionID": "sess_1",
            "time": {
                "created": "2025-01-15T10:30:00Z",
                "updated": "2025-01-15T10:30:05Z"
            }
        },
        "parts": [
            {
                "type": "text",
                "id": "part_1",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:05Z"
                },
                "text": "Hello, world!"
            }
        ]
    });

    let mwp: MessageWithParts = serde_json::from_value(json).unwrap();
    assert!(matches!(mwp.info, Message::Assistant(_)));
    assert_eq!(mwp.parts.len(), 1);
}

#[test]
fn test_message_with_parts_empty_parts() {
    let json = json!({
        "info": {
            "role": "user",
            "id": "msg_ep",
            "sessionID": "sess_1",
            "time": {
                "created": "2025-01-15T10:30:00Z",
                "updated": "2025-01-15T10:30:00Z"
            }
        },
        "parts": []
    });

    let mwp: MessageWithParts = serde_json::from_value(json).unwrap();
    assert!(mwp.parts.is_empty());
}

// ============================================================================
// Part tests
// ============================================================================

#[test]
fn test_part_text() {
    let json = json!({
        "type": "text",
        "id": "part_text1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "start": 1705312200000_i64,
            "end": 1705312205000_i64
        },
        "text": "Here is the analysis of the code."
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Text(text) => {
            assert_eq!(text.id, "part_text1");
            assert_eq!(text.session_id.as_deref(), Some("sess_1"));
            assert_eq!(text.message_id.as_deref(), Some("msg_1"));
            assert_eq!(
                text.text.as_deref(),
                Some("Here is the analysis of the code.")
            );
        }
        _ => panic!("Expected Part::Text"),
    }
}

#[test]
fn test_part_text_null_content() {
    let json = json!({
        "type": "text",
        "id": "part_text_null"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Text(text) => {
            assert!(text.text.is_none());
            assert!(text.session_id.is_none());
            assert!(text.message_id.is_none());
        }
        _ => panic!("Expected Part::Text"),
    }
}

#[test]
fn test_part_tool_pending() {
    let json = json!({
        "type": "tool",
        "id": "part_tool1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "tool": "read_file",
        "input": {
            "path": "/src/main.rs"
        },
        "state": {
            "status": "pending"
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Tool(tool) => {
            assert_eq!(tool.id, "part_tool1");
            assert_eq!(tool.tool.as_deref(), Some("read_file"));
            let input = tool.input.unwrap();
            assert_eq!(input["path"], "/src/main.rs");
            assert!(matches!(tool.state, ToolState::Pending { .. }));
        }
        _ => panic!("Expected Part::Tool"),
    }
}

#[test]
fn test_part_tool_running() {
    let json = json!({
        "type": "tool",
        "id": "part_tool_run",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:01Z"
        },
        "tool": "bash",
        "state": {
            "status": "running",
            "input": { "command": "ls -la" }
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Tool(tool) => {
            if let ToolState::Running { input } = &tool.state {
                let cmd = input.as_ref().and_then(|v| v.get("command")).and_then(|v| v.as_str());
                assert_eq!(cmd, Some("ls -la"));
            } else {
                panic!("Expected ToolState::Running");
            }
        }
        _ => panic!("Expected Part::Tool"),
    }
}

#[test]
fn test_part_tool_completed() {
    let json = json!({
        "type": "tool",
        "id": "part_tool_done",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:02Z"
        },
        "tool": "write_file",
        "state": {
            "status": "completed",
            "input": {
                "path": "/src/lib.rs",
                "content": "fn main() {}"
            },
            "output": "File written successfully",
            "title": "Wrote /src/lib.rs"
        },
        "metadata": {
            "bytes_written": 12
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Tool(tool) => {
            match &tool.state {
                ToolState::Completed { output, title, .. } => {
                    assert_eq!(output.as_deref(), Some("File written successfully"));
                    assert_eq!(title.as_deref(), Some("Wrote /src/lib.rs"));
                }
                _ => panic!("Expected ToolState::Completed"),
            }
            let metadata = tool.metadata.unwrap();
            assert_eq!(metadata["bytes_written"], 12);
        }
        _ => panic!("Expected Part::Tool"),
    }
}

#[test]
fn test_part_tool_error() {
    let json = json!({
        "type": "tool",
        "id": "part_tool_err",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:01Z"
        },
        "tool": "bash",
        "state": {
            "status": "error",
            "error": "Permission denied"
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Tool(tool) => match &tool.state {
            ToolState::Error { error, .. } => {
                assert_eq!(error.as_deref(), Some("Permission denied"));
            }
            _ => panic!("Expected ToolState::Error"),
        },
        _ => panic!("Expected Part::Tool"),
    }
}

#[test]
fn test_part_reasoning() {
    let json = json!({
        "type": "reasoning",
        "id": "part_reason1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:01Z"
        },
        "content": "Let me think about this step by step...",
        "redacted": false
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Reasoning(reasoning) => {
            assert_eq!(reasoning.id, "part_reason1");
            assert_eq!(
                reasoning.content.as_deref(),
                Some("Let me think about this step by step...")
            );
            assert_eq!(reasoning.redacted, Some(false));
        }
        _ => panic!("Expected Part::Reasoning"),
    }
}

#[test]
fn test_part_reasoning_redacted() {
    let json = json!({
        "type": "reasoning",
        "id": "part_reason_redacted",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:01Z"
        },
        "redacted": true
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Reasoning(reasoning) => {
            assert!(reasoning.content.is_none());
            assert_eq!(reasoning.redacted, Some(true));
        }
        _ => panic!("Expected Part::Reasoning"),
    }
}

#[test]
fn test_part_step_start() {
    let json = json!({
        "type": "step-start",
        "id": "part_ss1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "stepID": "step_abc",
        "title": "Analyzing codebase"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::StepStart(step) => {
            assert_eq!(step.id, "part_ss1");
            assert_eq!(step.step_id.as_deref(), Some("step_abc"));
            assert_eq!(step.title.as_deref(), Some("Analyzing codebase"));
        }
        _ => panic!("Expected Part::StepStart"),
    }
}

#[test]
fn test_part_step_finish() {
    let json = json!({
        "type": "step-finish",
        "id": "part_sf1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:05Z"
        },
        "stepID": "step_abc"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::StepFinish(step) => {
            assert_eq!(step.id, "part_sf1");
            assert_eq!(step.step_id.as_deref(), Some("step_abc"));
        }
        _ => panic!("Expected Part::StepFinish"),
    }
}

#[test]
fn test_part_subtask() {
    let json = json!({
        "type": "subtask",
        "id": "part_sub1",
        "sessionID": "sess_1",
        "messageID": "msg_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:10Z"
        },
        "input": "Investigate the test failures",
        "summary": "Found 3 failing tests in module X",
        "modelID": "claude-sonnet-4-20250514",
        "providerID": "anthropic"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Subtask(subtask) => {
            assert_eq!(subtask.id, "part_sub1");
            assert_eq!(
                subtask.input.as_deref(),
                Some("Investigate the test failures")
            );
            assert_eq!(
                subtask.summary.as_deref(),
                Some("Found 3 failing tests in module X")
            );
            assert_eq!(
                subtask.model_id.as_deref(),
                Some("claude-sonnet-4-20250514")
            );
            assert_eq!(subtask.provider_id.as_deref(), Some("anthropic"));
        }
        _ => panic!("Expected Part::Subtask"),
    }
}

#[test]
fn test_part_file() {
    let json = json!({
        "type": "file",
        "id": "part_file1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "filePath": "/home/user/image.png",
        "mediaType": "image/png",
        "url": "https://example.com/image.png"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::File(file) => {
            assert_eq!(file.id, "part_file1");
            assert_eq!(file.file_path.as_deref(), Some("/home/user/image.png"));
            assert_eq!(file.media_type.as_deref(), Some("image/png"));
            assert_eq!(file.url.as_deref(), Some("https://example.com/image.png"));
        }
        _ => panic!("Expected Part::File"),
    }
}

#[test]
fn test_part_snapshot() {
    let json = json!({
        "type": "snapshot",
        "id": "part_snap1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    assert!(matches!(part, Part::Snapshot(_)));
}

#[test]
fn test_part_patch() {
    let json = json!({
        "type": "patch",
        "id": "part_patch1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "path": "/src/main.rs",
        "content": "@@ -1,3 +1,4 @@\n+use std::io;\n fn main() {"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Patch(patch) => {
            assert_eq!(patch.path.as_deref(), Some("/src/main.rs"));
            assert!(patch.content.is_some());
        }
        _ => panic!("Expected Part::Patch"),
    }
}

#[test]
fn test_part_agent() {
    let json = json!({
        "type": "agent",
        "id": "part_agent1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "agent": "coder"
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Agent(agent) => {
            assert_eq!(agent.agent.as_deref(), Some("coder"));
        }
        _ => panic!("Expected Part::Agent"),
    }
}

#[test]
fn test_part_retry() {
    let json = json!({
        "type": "retry",
        "id": "part_retry1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "error": "Rate limit exceeded, retrying..."
    });

    let part: Part = serde_json::from_value(json).unwrap();
    match part {
        Part::Retry(retry) => {
            assert_eq!(
                retry.error.as_deref(),
                Some("Rate limit exceeded, retrying...")
            );
        }
        _ => panic!("Expected Part::Retry"),
    }
}

#[test]
fn test_part_compaction() {
    let json = json!({
        "type": "compaction",
        "id": "part_compact1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    assert!(matches!(part, Part::Compaction(_)));
}

#[test]
fn test_part_roundtrip_tool() {
    let json = json!({
        "type": "tool",
        "id": "part_rt",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:02Z"
        },
        "tool": "bash",
        "input": { "command": "echo hello" },
        "state": {
            "status": "completed",
            "output": "hello\n",
            "title": "Ran bash command"
        }
    });

    let part: Part = serde_json::from_value(json).unwrap();
    let serialized = serde_json::to_value(&part).unwrap();
    assert_eq!(serialized["type"], "tool");
    assert_eq!(serialized["state"]["status"], "completed");
    assert_eq!(serialized["state"]["output"], "hello\n");

    let deserialized: Part = serde_json::from_value(serialized).unwrap();
    match deserialized {
        Part::Tool(tool) => {
            assert_eq!(tool.id, "part_rt");
            assert!(matches!(tool.state, ToolState::Completed { .. }));
        }
        _ => panic!("Expected Part::Tool"),
    }
}

// ============================================================================
// Event tests
// ============================================================================

#[test]
fn test_event_session_created() {
    let json = json!({
        "type": "session.created",
        "properties": {
            "info": {
                "id": "sess_new",
                "projectID": "proj_1",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:00Z"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionCreated { properties } => {
            assert_eq!(properties.info.id, "sess_new");
            assert_eq!(properties.info.project_id.as_deref(), Some("proj_1"));
        }
        _ => panic!("Expected Event::SessionCreated"),
    }
}

#[test]
fn test_event_session_updated() {
    let json = json!({
        "type": "session.updated",
        "properties": {
            "info": {
                "id": "sess_upd",
                "projectID": "proj_1",
                "title": "Updated title",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:35:00Z"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionUpdated { properties } => {
            assert_eq!(properties.info.id, "sess_upd");
            assert_eq!(properties.info.title.as_deref(), Some("Updated title"));
        }
        _ => panic!("Expected Event::SessionUpdated"),
    }
}

#[test]
fn test_event_session_deleted() {
    let json = json!({
        "type": "session.deleted",
        "properties": {
            "info": {
                "id": "sess_del",
                "projectID": "proj_1",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:00Z"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    assert!(matches!(event, Event::SessionDeleted { .. }));
}

#[test]
fn test_event_session_status() {
    let json = json!({
        "type": "session.status",
        "properties": {
            "sessionID": "sess_1",
            "status": {
                "type": "busy"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionStatus { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert!(matches!(properties.status, SessionStatus::Busy));
        }
        _ => panic!("Expected Event::SessionStatus"),
    }
}

#[test]
fn test_event_session_idle() {
    let json = json!({
        "type": "session.idle",
        "properties": {
            "sessionID": "sess_1"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionIdle { properties } => {
            assert_eq!(properties.session_id, "sess_1");
        }
        _ => panic!("Expected Event::SessionIdle"),
    }
}

#[test]
fn test_event_session_error() {
    let json = json!({
        "type": "session.error",
        "properties": {
            "sessionID": "sess_1",
            "error": "Something went wrong"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionError { properties } => {
            assert_eq!(properties.session_id.as_deref(), Some("sess_1"));
            assert!(properties.error.is_some());
        }
        _ => panic!("Expected Event::SessionError"),
    }
}

#[test]
fn test_event_session_compacted() {
    let json = json!({
        "type": "session.compacted",
        "properties": {
            "sessionID": "sess_1"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    assert!(matches!(event, Event::SessionCompacted { .. }));
}

#[test]
fn test_event_message_updated_user() {
    let json = json!({
        "type": "message.updated",
        "properties": {
            "sessionID": "sess_1",
            "info": {
                "role": "user",
                "id": "msg_u1",
                "sessionID": "sess_1",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:00Z"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessageUpdated { properties } => {
            assert_eq!(properties.session_id.as_deref(), Some("sess_1"));
            assert!(matches!(properties.info, Message::User(_)));
        }
        _ => panic!("Expected Event::MessageUpdated"),
    }
}

#[test]
fn test_event_message_updated_assistant() {
    let json = json!({
        "type": "message.updated",
        "properties": {
            "sessionID": "sess_1",
            "info": {
                "role": "assistant",
                "id": "msg_a1",
                "sessionID": "sess_1",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:05Z"
                },
                "modelID": "claude-sonnet-4-20250514",
                "providerID": "anthropic"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessageUpdated { properties } => {
            match properties.info {
                Message::Assistant(asst) => {
                    assert_eq!(asst.id, "msg_a1");
                    assert_eq!(
                        asst.model_id.as_deref(),
                        Some("claude-sonnet-4-20250514")
                    );
                }
                _ => panic!("Expected Message::Assistant"),
            }
        }
        _ => panic!("Expected Event::MessageUpdated"),
    }
}

#[test]
fn test_event_message_removed() {
    let json = json!({
        "type": "message.removed",
        "properties": {
            "sessionID": "sess_1",
            "messageID": "msg_del"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessageRemoved { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert_eq!(properties.message_id, "msg_del");
        }
        _ => panic!("Expected Event::MessageRemoved"),
    }
}

#[test]
fn test_event_message_part_updated() {
    let json = json!({
        "type": "message.part.updated",
        "properties": {
            "sessionID": "sess_1",
            "messageID": "msg_1",
            "part": {
                "type": "text",
                "id": "part_upd",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:02Z"
                },
                "text": "Updated text content"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessagePartUpdated { properties } => {
            assert_eq!(properties.session_id.as_deref(), Some("sess_1"));
            assert_eq!(properties.message_id.as_deref(), Some("msg_1"));
            assert!(matches!(properties.part, Part::Text(_)));
        }
        _ => panic!("Expected Event::MessagePartUpdated"),
    }
}

#[test]
fn test_event_message_part_delta() {
    let json = json!({
        "type": "message.part.delta",
        "properties": {
            "sessionID": "sess_1",
            "messageID": "msg_1",
            "partID": "part_1",
            "field": "content",
            "delta": " more text"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessagePartDelta { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert_eq!(properties.message_id, "msg_1");
            assert_eq!(properties.part_id, "part_1");
            assert_eq!(properties.field, "content");
            assert_eq!(properties.delta.as_str().unwrap(), " more text");
        }
        _ => panic!("Expected Event::MessagePartDelta"),
    }
}

#[test]
fn test_event_message_part_removed() {
    let json = json!({
        "type": "message.part.removed",
        "properties": {
            "sessionID": "sess_1",
            "messageID": "msg_1",
            "partID": "part_del"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessagePartRemoved { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert_eq!(properties.part_id, "part_del");
        }
        _ => panic!("Expected Event::MessagePartRemoved"),
    }
}

#[test]
fn test_event_permission_asked() {
    // Matches the real API format: flat properties, tool is an object
    let json = json!({
        "type": "permission.asked",
        "properties": {
            "id": "per_abc123",
            "sessionID": "sess_1",
            "permission": "bash",
            "patterns": ["git log --oneline -5"],
            "metadata": {},
            "always": ["git log *"],
            "tool": {
                "messageID": "msg_xyz",
                "callID": "toolu_123"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::PermissionAsked { properties } => {
            let req = &properties.request;
            assert_eq!(req.id, "per_abc123");
            assert_eq!(req.session_id, "sess_1");
            assert_eq!(req.permission.as_deref(), Some("bash"));
            let patterns = req.patterns.as_ref().unwrap();
            assert_eq!(patterns, &["git log --oneline -5"]);
            assert_eq!(req.always.as_ref().unwrap(), &["git log *"]);
            assert!(req.tool.is_some());
            assert_eq!(req.description, None);
        }
        _ => panic!("Expected Event::PermissionAsked"),
    }
}

#[test]
fn test_event_permission_replied() {
    let json = json!({
        "type": "permission.replied",
        "properties": {
            "sessionID": "sess_1",
            "requestID": "perm_1",
            "reply": "always"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::PermissionReplied { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert_eq!(properties.request_id, "perm_1");
            assert!(matches!(properties.reply, PermissionReply::Always));
        }
        _ => panic!("Expected Event::PermissionReplied"),
    }
}

#[test]
fn test_event_question_asked() {
    let json = json!({
        "type": "question.asked",
        "properties": {
            "question": {
                "id": "q_1",
                "sessionID": "sess_1",
                "question": "Which framework?",
                "options": [
                    { "label": "React", "value": "react" },
                    { "label": "Vue", "value": "vue", "selected": true }
                ],
                "multiSelect": false
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::QuestionAsked { properties } => {
            let q = &properties.question;
            assert_eq!(q.id, "q_1");
            assert_eq!(q.question, "Which framework?");
            let options = q.options.as_ref().unwrap();
            assert_eq!(options.len(), 2);
            assert_eq!(options[0].label, "React");
            assert_eq!(options[1].selected, Some(true));
            assert_eq!(q.multi_select, Some(false));
        }
        _ => panic!("Expected Event::QuestionAsked"),
    }
}

#[test]
fn test_event_question_replied() {
    let json = json!({
        "type": "question.replied",
        "properties": {
            "sessionID": "sess_1",
            "questionID": "q_1",
            "answer": {
                "values": ["react"]
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::QuestionReplied { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            assert_eq!(properties.question_id, "q_1");
            let vals = properties.answer.values.unwrap();
            assert_eq!(vals, vec!["react"]);
        }
        _ => panic!("Expected Event::QuestionReplied"),
    }
}

#[test]
fn test_event_question_rejected() {
    let json = json!({
        "type": "question.rejected",
        "properties": {
            "sessionID": "sess_1",
            "questionID": "q_1"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    assert!(matches!(event, Event::QuestionRejected { .. }));
}

#[test]
fn test_event_todo_updated() {
    let json = json!({
        "type": "todo.updated",
        "properties": {
            "sessionID": "sess_1",
            "todos": [
                {
                    "id": "todo_1",
                    "content": "Fix the bug",
                    "status": "in_progress"
                },
                {
                    "id": "todo_2",
                    "content": "Write tests",
                    "status": "pending"
                },
                {
                    "id": "todo_3",
                    "content": "Update docs",
                    "status": "completed"
                }
            ]
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::TodoUpdated { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            let todos = properties.todos.unwrap();
            assert_eq!(todos.len(), 3);
            assert!(matches!(todos[0].status, TodoStatus::InProgress));
            assert!(matches!(todos[1].status, TodoStatus::Pending));
            assert!(matches!(todos[2].status, TodoStatus::Completed));
        }
        _ => panic!("Expected Event::TodoUpdated"),
    }
}

#[test]
fn test_event_todo_updated_empty() {
    let json = json!({
        "type": "todo.updated",
        "properties": {
            "sessionID": "sess_1",
            "todos": []
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::TodoUpdated { properties } => {
            let todos = properties.todos.unwrap();
            assert!(todos.is_empty());
        }
        _ => panic!("Expected Event::TodoUpdated"),
    }
}

#[test]
fn test_event_project_updated() {
    let json = json!({
        "type": "project.updated",
        "properties": {
            "info": {
                "id": "proj_1",
                "name": "my-project",
                "path": "/home/user/my-project",
                "directory": "/home/user/my-project",
                "summary": {
                    "description": "A Rust CLI tool",
                    "languages": ["rust"],
                    "frameworks": ["tokio", "clap"]
                },
                "vcs": {
                    "branch": "main",
                    "commit": "abc123",
                    "dirty": false,
                    "root": "/home/user/my-project"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::ProjectUpdated { properties } => {
            let proj = &properties.info;
            assert_eq!(proj.id.as_deref(), Some("proj_1"));
            assert_eq!(proj.name.as_deref(), Some("my-project"));
            let summary = proj.summary.as_ref().unwrap();
            assert_eq!(summary.description.as_deref(), Some("A Rust CLI tool"));
            let vcs = proj.vcs.as_ref().unwrap();
            assert_eq!(vcs["branch"].as_str(), Some("main"));
            assert_eq!(vcs["dirty"].as_bool(), Some(false));
        }
        _ => panic!("Expected Event::ProjectUpdated"),
    }
}

#[test]
fn test_event_vcs_branch_updated() {
    let json = json!({
        "type": "vcs.branch.updated",
        "properties": {
            "branch": "feature/new-thing"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::VcsBranchUpdated { properties } => {
            assert_eq!(properties.branch.as_deref(), Some("feature/new-thing"));
        }
        _ => panic!("Expected Event::VcsBranchUpdated"),
    }
}

#[test]
fn test_event_server_connected() {
    let json = json!({
        "type": "server.connected",
        "properties": {}
    });

    let event: Event = serde_json::from_value(json).unwrap();
    assert!(matches!(event, Event::ServerConnected { .. }));
}

#[test]
fn test_event_file_edited() {
    let json = json!({
        "type": "file.edited",
        "properties": {
            "file": "/src/main.rs"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::FileEdited { properties } => {
            assert_eq!(properties.file.as_deref(), Some("/src/main.rs"));
        }
        _ => panic!("Expected Event::FileEdited"),
    }
}

#[test]
fn test_event_file_watcher_updated() {
    let json = json!({
        "type": "file.watcher.updated",
        "properties": {
            "files": ["/src/main.rs", "/src/lib.rs"]
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::FileWatcherUpdated { properties } => {
            let files = properties.files.unwrap();
            assert_eq!(files.len(), 2);
        }
        _ => panic!("Expected Event::FileWatcherUpdated"),
    }
}

#[test]
fn test_event_command_executed() {
    let json = json!({
        "type": "command.executed",
        "properties": {
            "command": "test",
            "args": ["--verbose"]
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::CommandExecuted { properties } => {
            assert_eq!(properties.command.as_deref(), Some("test"));
            let args = properties.args.unwrap();
            assert_eq!(args, vec!["--verbose"]);
        }
        _ => panic!("Expected Event::CommandExecuted"),
    }
}

#[test]
fn test_event_session_diff() {
    let json = json!({
        "type": "session.diff",
        "properties": {
            "sessionID": "sess_1",
            "diffs": [
                {
                    "path": "/src/main.rs",
                    "status": "modified",
                    "diff": "@@ -1,3 +1,4 @@\n+use std::io;\n fn main() {}"
                }
            ]
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::SessionDiff { properties } => {
            assert_eq!(properties.session_id, "sess_1");
            let diffs = properties.diffs.unwrap();
            assert_eq!(diffs.len(), 1);
            assert_eq!(diffs[0].path, "/src/main.rs");
            assert!(matches!(diffs[0].status, Some(FileStatus::Modified)));
        }
        _ => panic!("Expected Event::SessionDiff"),
    }
}

#[test]
fn test_event_tui_prompt_append() {
    let json = json!({
        "type": "tui.prompt.append",
        "properties": {
            "text": "Fix the tests"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::TuiPromptAppend { properties } => {
            assert_eq!(properties.text.as_deref(), Some("Fix the tests"));
        }
        _ => panic!("Expected Event::TuiPromptAppend"),
    }
}

#[test]
fn test_event_tui_toast_show() {
    let json = json!({
        "type": "tui.toast.show",
        "properties": {
            "title": "Error",
            "message": "Something failed",
            "variant": "error"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::TuiToastShow { properties } => {
            assert_eq!(properties.title.as_deref(), Some("Error"));
            assert_eq!(properties.message.as_deref(), Some("Something failed"));
            assert_eq!(properties.variant.as_deref(), Some("error"));
        }
        _ => panic!("Expected Event::TuiToastShow"),
    }
}

#[test]
fn test_event_installation_updated() {
    let json = json!({
        "type": "installation.updated",
        "properties": {
            "info": {
                "version": "1.2.3",
                "path": "/usr/local/bin/opencode"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::InstallationUpdated { properties } => {
            let info = properties.info.unwrap();
            assert_eq!(info.version.as_deref(), Some("1.2.3"));
            assert_eq!(info.path.as_deref(), Some("/usr/local/bin/opencode"));
        }
        _ => panic!("Expected Event::InstallationUpdated"),
    }
}

#[test]
fn test_event_installation_update_available() {
    let json = json!({
        "type": "installation.update-available",
        "properties": {
            "version": "2.0.0"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::InstallationUpdateAvailable { properties } => {
            assert_eq!(properties.version.as_deref(), Some("2.0.0"));
        }
        _ => panic!("Expected Event::InstallationUpdateAvailable"),
    }
}

#[test]
fn test_event_pty_created() {
    let json = json!({
        "type": "pty.created",
        "properties": {
            "info": {
                "id": "pty_1",
                "name": "shell",
                "command": "bash",
                "running": true
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::PtyCreated { properties } => {
            assert_eq!(properties.info.id, "pty_1");
            assert_eq!(properties.info.name.as_deref(), Some("shell"));
            assert_eq!(properties.info.running, Some(true));
        }
        _ => panic!("Expected Event::PtyCreated"),
    }
}

#[test]
fn test_event_pty_exited() {
    let json = json!({
        "type": "pty.exited",
        "properties": {
            "id": "pty_1",
            "exitCode": 0
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::PtyExited { properties } => {
            assert_eq!(properties.id, "pty_1");
            assert_eq!(properties.exit_code, Some(0));
        }
        _ => panic!("Expected Event::PtyExited"),
    }
}

#[test]
fn test_event_mcp_tools_changed() {
    let json = json!({
        "type": "mcp.tools.changed",
        "properties": {
            "tools": {
                "server": "my-mcp-server",
                "tools": [
                    {
                        "name": "search",
                        "description": "Search the codebase",
                        "server": "my-mcp-server"
                    }
                ]
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::McpToolsChanged { properties } => {
            let tools = properties.tools.unwrap();
            assert_eq!(tools.server.as_deref(), Some("my-mcp-server"));
            let tool_list = tools.tools.unwrap();
            assert_eq!(tool_list.len(), 1);
            assert_eq!(tool_list[0].name.as_deref(), Some("search"));
        }
        _ => panic!("Expected Event::McpToolsChanged"),
    }
}

#[test]
fn test_event_lsp_updated() {
    let json = json!({
        "type": "lsp.updated",
        "properties": {
            "statuses": {
                "rust-analyzer": {
                    "name": "rust-analyzer",
                    "status": "running",
                    "language": "rust"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::LspUpdated { properties } => {
            let statuses = properties.statuses.unwrap();
            let ra = statuses.get("rust-analyzer").unwrap();
            assert_eq!(ra.name.as_deref(), Some("rust-analyzer"));
            assert_eq!(ra.status.as_deref(), Some("running"));
        }
        _ => panic!("Expected Event::LspUpdated"),
    }
}

#[test]
fn test_event_worktree_ready() {
    let json = json!({
        "type": "worktree.ready",
        "properties": {
            "worktree": {
                "path": "/tmp/worktree-abc",
                "branch": "feature/worktree-branch"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::WorktreeReady { properties } => {
            assert_eq!(
                properties.worktree.path.as_deref(),
                Some("/tmp/worktree-abc")
            );
            assert_eq!(
                properties.worktree.branch.as_deref(),
                Some("feature/worktree-branch")
            );
        }
        _ => panic!("Expected Event::WorktreeReady"),
    }
}

#[test]
fn test_event_worktree_failed() {
    let json = json!({
        "type": "worktree.failed",
        "properties": {
            "error": "Git worktree creation failed"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::WorktreeFailed { properties } => {
            assert_eq!(properties.error, "Git worktree creation failed");
        }
        _ => panic!("Expected Event::WorktreeFailed"),
    }
}

#[test]
fn test_event_roundtrip_session_created() {
    let json = json!({
        "type": "session.created",
        "properties": {
            "info": {
                "id": "sess_rt",
                "projectID": "proj_rt",
                "time": {
                    "created": "2025-01-15T10:30:00Z",
                    "updated": "2025-01-15T10:30:00Z"
                }
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    let serialized = serde_json::to_value(&event).unwrap();
    assert_eq!(serialized["type"], "session.created");
    assert_eq!(serialized["properties"]["info"]["id"], "sess_rt");

    let deserialized: Event = serde_json::from_value(serialized).unwrap();
    assert!(matches!(deserialized, Event::SessionCreated { .. }));
}

#[test]
fn test_global_event_payload_parsing() {
    // GlobalEvent struct was removed; test that parsing the inner payload works
    let json = json!({
        "type": "session.idle",
        "properties": {
            "sessionID": "sess_1"
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    assert!(matches!(event, Event::SessionIdle { .. }));
}

// ============================================================================
// Provider / Model tests
// ============================================================================

#[test]
fn test_provider_full() {
    let json = json!({
        "id": "anthropic",
        "name": "Anthropic",
        "source": "env",
        "env": ["ANTHROPIC_API_KEY"],
        "key": "sk-ant-...",
        "options": {
            "base_url": "https://api.anthropic.com"
        },
        "models": {
            "claude-sonnet-4-20250514": {
                "id": "claude-sonnet-4-20250514",
                "providerID": "anthropic",
                "api": {
                    "id": "claude-sonnet-4-20250514",
                    "url": "https://api.anthropic.com/v1",
                    "npm": "@ai-sdk/anthropic"
                },
                "name": "Claude Sonnet",
                "family": "claude",
                "capabilities": {
                    "reasoning": true,
                    "input": ["text", "image"],
                    "output": ["text"]
                },
                "cost": {
                    "input": 3.0,
                    "output": 15.0,
                    "cacheRead": 0.3,
                    "cacheWrite": 3.75
                },
                "limit": {
                    "context": 200000,
                    "output": 8192
                }
            }
        }
    });

    let provider: Provider = serde_json::from_value(json).unwrap();
    assert_eq!(provider.id, "anthropic");
    assert_eq!(provider.name, "Anthropic");
    assert!(matches!(provider.source, Some(ProviderSource::Env)));
    let env = provider.env.unwrap();
    assert_eq!(env, vec!["ANTHROPIC_API_KEY"]);
    assert_eq!(provider.key.as_deref(), Some("sk-ant-..."));
    assert!(provider.options.is_some());
    assert_eq!(provider.models.len(), 1);

    let model = provider.models.get("claude-sonnet-4-20250514").unwrap();
    assert_eq!(model.id, "claude-sonnet-4-20250514");
    assert_eq!(model.provider_id.as_deref(), Some("anthropic"));
    assert!(model.api.is_some());
    assert_eq!(model.name, "Claude Sonnet");
    assert_eq!(model.family.as_deref(), Some("claude"));

    let caps = model.capabilities.as_ref().unwrap();
    assert_eq!(caps.reasoning, Some(true));
    let input_types = caps.input.as_ref().unwrap();
    assert_eq!(input_types, &json!(["text", "image"]));

    let cost = model.cost.as_ref().unwrap();
    assert!((cost.input.unwrap() - 3.0).abs() < f64::EPSILON);
    assert!((cost.output.unwrap() - 15.0).abs() < f64::EPSILON);

    let limit = model.limit.as_ref().unwrap();
    assert_eq!(limit.context, Some(200000));
    assert_eq!(limit.output, Some(8192));
}

#[test]
fn test_provider_minimal() {
    let json = json!({
        "id": "custom",
        "name": "Custom Provider",
        "models": {}
    });

    let provider: Provider = serde_json::from_value(json).unwrap();
    assert_eq!(provider.id, "custom");
    assert!(provider.source.is_none());
    assert!(provider.env.is_none());
    assert!(provider.key.is_none());
    assert!(provider.options.is_none());
    assert!(provider.models.is_empty());
}

#[test]
fn test_model_minimal() {
    let json = json!({
        "id": "gpt-4",
        "name": "GPT-4"
    });

    let model: Model = serde_json::from_value(json).unwrap();
    assert_eq!(model.id, "gpt-4");
    assert_eq!(model.name, "GPT-4");
    assert!(model.provider_id.is_none());
    assert!(model.api.is_none());
    assert!(model.family.is_none());
    assert!(model.capabilities.is_none());
    assert!(model.cost.is_none());
    assert!(model.limit.is_none());
}

#[test]
fn test_provider_source_variants() {
    for (input, expected) in [
        ("\"env\"", "Env"),
        ("\"config\"", "Config"),
        ("\"custom\"", "Custom"),
        ("\"api\"", "Api"),
    ] {
        let source: ProviderSource = serde_json::from_str(input).unwrap();
        let serialized = serde_json::to_string(&source).unwrap();
        assert_eq!(
            serialized,
            format!("\"{}\"", input.trim_matches('"')),
            "Failed roundtrip for {expected}"
        );
    }
}

#[test]
fn test_provider_roundtrip() {
    let json = json!({
        "id": "openai",
        "name": "OpenAI",
        "source": "config",
        "models": {
            "gpt-4o": {
                "id": "gpt-4o",
                "name": "GPT-4o",
                "cost": {
                    "input": 5.0,
                    "output": 15.0
                }
            }
        }
    });

    let provider: Provider = serde_json::from_value(json).unwrap();
    let serialized = serde_json::to_value(&provider).unwrap();
    let deserialized: Provider = serde_json::from_value(serialized).unwrap();
    assert_eq!(deserialized.id, "openai");
    assert_eq!(deserialized.models.len(), 1);
    assert_eq!(deserialized.models.get("gpt-4o").unwrap().id, "gpt-4o");
}

// ============================================================================
// ProviderListResponse tests
// ============================================================================

#[test]
fn test_provider_list_response_deserialize_with_defaults_and_connected() {
    let json = json!({
        "all": [{
            "id": "anthropic",
            "name": "Anthropic",
            "models": {
                "claude-opus-4-5": {
                    "id": "claude-opus-4-5",
                    "name": "Claude Opus 4.5"
                }
            }
        }],
        "default": {
            "anthropic": "claude-sonnet-4-6",
            "opencode": "big-pickle"
        },
        "connected": ["anthropic", "opencode"]
    });

    let resp: ProviderListResponse = serde_json::from_value(json).unwrap();
    assert_eq!(resp.all.len(), 1);
    assert_eq!(resp.all[0].id, "anthropic");
    assert_eq!(resp.default.len(), 2);
    assert_eq!(resp.default.get("anthropic").unwrap(), "claude-sonnet-4-6");
    assert_eq!(resp.default.get("opencode").unwrap(), "big-pickle");
    assert_eq!(resp.connected, vec!["anthropic", "opencode"]);
}

#[test]
fn test_provider_list_response_deserialize_missing_optional_fields() {
    // Server might omit default/connected; they should default to empty
    let json = json!({
        "all": []
    });

    let resp: ProviderListResponse = serde_json::from_value(json).unwrap();
    assert!(resp.all.is_empty());
    assert!(resp.default.is_empty());
    assert!(resp.connected.is_empty());
}

// ============================================================================
// Permission tests
// ============================================================================

#[test]
fn test_permission_request_full() {
    // Matches real API: always is Vec<String>, tool is an object
    let json = json!({
        "id": "perm_abc",
        "sessionID": "sess_1",
        "permission": "bash",
        "patterns": ["git log --oneline -5"],
        "metadata": {},
        "always": ["git log *"],
        "tool": {
            "messageID": "msg_xyz",
            "callID": "toolu_123"
        },
        "description": "Run git log"
    });

    let req: PermissionRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.id, "perm_abc");
    assert_eq!(req.session_id, "sess_1");
    assert_eq!(req.permission.as_deref(), Some("bash"));
    let patterns = req.patterns.unwrap();
    assert_eq!(patterns.len(), 1);
    assert_eq!(patterns[0], "git log --oneline -5");
    assert_eq!(req.always.as_ref().unwrap(), &["git log *"]);
    assert!(req.tool.is_some());
    assert_eq!(req.description.as_deref(), Some("Run git log"));
}

#[test]
fn test_permission_request_minimal() {
    let json = json!({
        "id": "perm_min",
        "sessionID": "sess_1"
    });

    let req: PermissionRequest = serde_json::from_value(json).unwrap();
    assert_eq!(req.id, "perm_min");
    assert!(req.permission.is_none());
    assert!(req.patterns.is_none());
    assert!(req.metadata.is_none());
    assert!(req.always.is_none());
    assert!(req.tool.is_none());
    assert!(req.description.is_none());
    assert!(req.input.is_none());
}

#[test]
fn test_permission_reply_variants() {
    let once: PermissionReply = serde_json::from_str("\"once\"").unwrap();
    assert!(matches!(once, PermissionReply::Once));

    let always: PermissionReply = serde_json::from_str("\"always\"").unwrap();
    assert!(matches!(always, PermissionReply::Always));

    let reject: PermissionReply = serde_json::from_str("\"reject\"").unwrap();
    assert!(matches!(reject, PermissionReply::Reject));
}

#[test]
fn test_permission_reply_roundtrip() {
    for reply in [
        PermissionReply::Once,
        PermissionReply::Always,
        PermissionReply::Reject,
    ] {
        let serialized = serde_json::to_string(&reply).unwrap();
        let deserialized: PermissionReply = serde_json::from_str(&serialized).unwrap();
        assert_eq!(
            std::mem::discriminant(&reply),
            std::mem::discriminant(&deserialized)
        );
    }
}

#[test]
fn test_permission_action_variants() {
    let allow: PermissionAction = serde_json::from_str("\"allow\"").unwrap();
    assert!(matches!(allow, PermissionAction::Allow));

    let deny: PermissionAction = serde_json::from_str("\"deny\"").unwrap();
    assert!(matches!(deny, PermissionAction::Deny));

    let ask: PermissionAction = serde_json::from_str("\"ask\"").unwrap();
    assert!(matches!(ask, PermissionAction::Ask));
}

#[test]
fn test_permission_config() {
    let json = json!({
        "default": "ask",
        "rulesets": {
            "project": {
                "rules": [
                    {
                        "tool": "write_file",
                        "pattern": "/src/**",
                        "action": "allow"
                    },
                    {
                        "tool": "bash",
                        "action": "deny"
                    }
                ]
            }
        }
    });

    let config: PermissionConfig = serde_json::from_value(json).unwrap();
    assert!(matches!(config.default, Some(PermissionAction::Ask)));
    let rulesets = config.rulesets.unwrap();
    let project_ruleset = rulesets.get("project").unwrap();
    let rules = project_ruleset.rules.as_ref().unwrap();
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].tool.as_deref(), Some("write_file"));
    assert!(matches!(rules[0].action, Some(PermissionAction::Allow)));
}

// ============================================================================
// Project tests
// ============================================================================

#[test]
fn test_project_full() {
    let json = json!({
        "id": "proj_1",
        "name": "my-project",
        "path": "/home/user/my-project",
        "directory": "/home/user/my-project",
        "config": { "theme": "dark" },
        "summary": {
            "description": "A CLI tool",
            "languages": ["rust", "typescript"],
            "frameworks": ["tokio"]
        },
        "vcs": {
            "branch": "main",
            "commit": "deadbeef",
            "dirty": true,
            "root": "/home/user/my-project"
        },
        "paths": {
            "root": "/home/user/my-project",
            "config": "/home/user/.config/opencode",
            "data": "/home/user/.local/share/opencode",
            "state": "/home/user/.local/state/opencode",
            "cache": "/home/user/.cache/opencode"
        }
    });

    let project: Project = serde_json::from_value(json).unwrap();
    assert_eq!(project.id.as_deref(), Some("proj_1"));
    assert_eq!(project.name.as_deref(), Some("my-project"));

    let summary = project.summary.unwrap();
    let langs = summary.languages.unwrap();
    assert_eq!(langs, vec!["rust", "typescript"]);
    let frameworks = summary.frameworks.unwrap();
    assert_eq!(frameworks, vec!["tokio"]);

    let vcs = project.vcs.unwrap();
    assert_eq!(vcs["branch"].as_str(), Some("main"));
    assert_eq!(vcs["commit"].as_str(), Some("deadbeef"));
    assert_eq!(vcs["dirty"].as_bool(), Some(true));

    let paths = project.paths.unwrap();
    assert_eq!(paths.root.as_deref(), Some("/home/user/my-project"));
    assert!(paths.extra.is_none());
}

#[test]
fn test_project_minimal() {
    let json = json!({});

    let project: Project = serde_json::from_value(json).unwrap();
    assert!(project.id.is_none());
    assert!(project.name.is_none());
    assert!(project.path.is_none());
    assert!(project.directory.is_none());
    assert!(project.config.is_none());
    assert!(project.summary.is_none());
    assert!(project.vcs.is_none());
    assert!(project.paths.is_none());
}

// ============================================================================
// Misc types tests
// ============================================================================

#[test]
fn test_todo_status_variants() {
    let pending: TodoStatus = serde_json::from_str("\"pending\"").unwrap();
    assert!(matches!(pending, TodoStatus::Pending));

    let in_progress: TodoStatus = serde_json::from_str("\"in_progress\"").unwrap();
    assert!(matches!(in_progress, TodoStatus::InProgress));

    let completed: TodoStatus = serde_json::from_str("\"completed\"").unwrap();
    assert!(matches!(completed, TodoStatus::Completed));
}

#[test]
fn test_todo() {
    let json = json!({
        "id": "todo_1",
        "content": "Fix the broken test",
        "status": "pending"
    });

    let todo: Todo = serde_json::from_value(json).unwrap();
    assert_eq!(todo.id.as_deref(), Some("todo_1"));
    assert_eq!(todo.content, "Fix the broken test");
    assert!(matches!(todo.status, TodoStatus::Pending));
}

#[test]
fn test_mcp_status_variants() {
    for (input, variant_name) in [
        ("\"connecting\"", "Connecting"),
        ("\"connected\"", "Connected"),
        ("\"disconnected\"", "Disconnected"),
        ("\"error\"", "Error"),
    ] {
        let status: MCPStatus = serde_json::from_str(input).unwrap();
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(
            serialized, input,
            "Roundtrip failed for MCPStatus::{variant_name}"
        );
    }
}

#[test]
fn test_log_level_variants() {
    for (input, variant_name) in [
        ("\"debug\"", "Debug"),
        ("\"info\"", "Info"),
        ("\"warn\"", "Warn"),
        ("\"error\"", "Error"),
    ] {
        let level: LogLevel = serde_json::from_str(input).unwrap();
        let serialized = serde_json::to_string(&level).unwrap();
        assert_eq!(
            serialized, input,
            "Roundtrip failed for LogLevel::{variant_name}"
        );
    }
}

#[test]
fn test_question_info() {
    let json = json!({
        "id": "q_1",
        "sessionID": "sess_1",
        "question": "Select your preferred language",
        "options": [
            { "label": "Rust", "value": "rust", "selected": true },
            { "label": "Go", "value": "go" }
        ],
        "multiSelect": true
    });

    let info: QuestionInfo = serde_json::from_value(json).unwrap();
    assert_eq!(info.id, "q_1");
    assert_eq!(info.question, "Select your preferred language");
    assert_eq!(info.multi_select, Some(true));
    let options = info.options.unwrap();
    assert_eq!(options.len(), 2);
    assert_eq!(options[0].label, "Rust");
    assert_eq!(options[0].value, "rust");
    assert_eq!(options[0].selected, Some(true));
    assert!(options[1].selected.is_none());
}

#[test]
fn test_question_answer() {
    let json = json!({ "values": ["rust", "go"] });
    let answer: QuestionAnswer = serde_json::from_value(json).unwrap();
    let vals = answer.values.unwrap();
    assert_eq!(vals, vec!["rust", "go"]);
}

#[test]
fn test_question_answer_empty() {
    let json = json!({ "values": [] });
    let answer: QuestionAnswer = serde_json::from_value(json).unwrap();
    assert!(answer.values.unwrap().is_empty());
}

#[test]
fn test_question_answer_none() {
    let json = json!({});
    let answer: QuestionAnswer = serde_json::from_value(json).unwrap();
    assert!(answer.values.is_none());
}

#[test]
fn test_pty() {
    let json = json!({
        "id": "pty_abc",
        "name": "main-shell",
        "command": "/bin/bash",
        "running": true
    });

    let pty: Pty = serde_json::from_value(json).unwrap();
    assert_eq!(pty.id, "pty_abc");
    assert_eq!(pty.name.as_deref(), Some("main-shell"));
    assert_eq!(pty.command.as_deref(), Some("/bin/bash"));
    assert_eq!(pty.running, Some(true));
}

#[test]
fn test_worktree() {
    let json = json!({
        "path": "/tmp/wt-abc",
        "branch": "feature/new"
    });

    let wt: Worktree = serde_json::from_value(json).unwrap();
    assert_eq!(wt.path.as_deref(), Some("/tmp/wt-abc"));
    assert_eq!(wt.branch.as_deref(), Some("feature/new"));
}

#[test]
fn test_installation_info() {
    let json = json!({
        "version": "1.0.0",
        "path": "/usr/local/bin/opencode"
    });

    let info: InstallationInfo = serde_json::from_value(json).unwrap();
    assert_eq!(info.version.as_deref(), Some("1.0.0"));
    assert_eq!(info.path.as_deref(), Some("/usr/local/bin/opencode"));
}

#[test]
fn test_skill() {
    let json = json!({
        "name": "code-review",
        "description": "Review code for issues",
        "path": "/skills/code-review.md"
    });

    let skill: Skill = serde_json::from_value(json).unwrap();
    assert_eq!(skill.name.as_deref(), Some("code-review"));
    assert_eq!(
        skill.description.as_deref(),
        Some("Review code for issues")
    );
}

#[test]
fn test_path_struct() {
    let json = json!({
        "cwd": "/home/user/project",
        "root": "/home/user/project",
        "data": "/home/user/.local/share/opencode",
        "config": "/home/user/.config/opencode",
        "state": "/home/user/.local/state/opencode",
        "cache": "/home/user/.cache/opencode",
        "customField": "custom_value"
    });

    let path: Path = serde_json::from_value(json).unwrap();
    assert_eq!(path.cwd.as_deref(), Some("/home/user/project"));
    assert_eq!(path.root.as_deref(), Some("/home/user/project"));
    // Extra fields captured by #[serde(flatten)]
    assert!(path.extra.contains_key("customField"));
}

// ============================================================================
// File types tests
// ============================================================================

#[test]
fn test_file_status_variants() {
    for (input, variant_name) in [
        ("\"added\"", "Added"),
        ("\"modified\"", "Modified"),
        ("\"deleted\"", "Deleted"),
        ("\"renamed\"", "Renamed"),
    ] {
        let status: FileStatus = serde_json::from_str(input).unwrap();
        let serialized = serde_json::to_string(&status).unwrap();
        assert_eq!(
            serialized, input,
            "Roundtrip failed for FileStatus::{variant_name}"
        );
    }
}

#[test]
fn test_file_diff() {
    let json = json!({
        "path": "/src/main.rs",
        "status": "modified",
        "diff": "@@ -1 +1 @@\n-old\n+new",
        "before": "old",
        "after": "new"
    });

    let diff: FileDiff = serde_json::from_value(json).unwrap();
    assert_eq!(diff.path, "/src/main.rs");
    assert!(matches!(diff.status, Some(FileStatus::Modified)));
    assert_eq!(diff.before.as_deref(), Some("old"));
    assert_eq!(diff.after.as_deref(), Some("new"));
}

#[test]
fn test_file_node() {
    let json = json!({
        "path": "/src",
        "name": "src",
        "size": 4096,
        "isDirectory": true,
        "children": [
            {
                "path": "/src/main.rs",
                "name": "main.rs",
                "size": 256,
                "isDirectory": false
            }
        ]
    });

    let node: FileNode = serde_json::from_value(json).unwrap();
    assert_eq!(node.path, "/src");
    assert_eq!(node.is_directory, Some(true));
    let children = node.children.unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name.as_deref(), Some("main.rs"));
    assert_eq!(children[0].is_directory, Some(false));
}

// ============================================================================
// Command / Agent types tests
// ============================================================================

#[test]
fn test_command() {
    let json = json!({
        "name": "test",
        "description": "Run tests",
        "agent": "default",
        "model": "claude-sonnet-4-20250514",
        "source": "command",
        "template": "Run all tests",
        "subtask": false,
        "hints": ["test", "verify"],
        "shortcut": "t",
        "category": "development"
    });

    let cmd: Command = serde_json::from_value(json).unwrap();
    assert_eq!(cmd.name, "test");
    assert_eq!(cmd.description.as_deref(), Some("Run tests"));
    assert!(matches!(cmd.source, Some(CommandSource::Command)));
    assert_eq!(cmd.shortcut.as_deref(), Some("t"));
}

#[test]
fn test_command_source_variants() {
    for (input, variant_name) in [
        ("\"command\"", "Command"),
        ("\"mcp\"", "Mcp"),
        ("\"skill\"", "Skill"),
    ] {
        let source: CommandSource = serde_json::from_str(input).unwrap();
        let serialized = serde_json::to_string(&source).unwrap();
        assert_eq!(
            serialized, input,
            "Roundtrip failed for CommandSource::{variant_name}"
        );
    }
}

#[test]
fn test_agent() {
    let json = json!({
        "name": "coder",
        "description": "A coding agent",
        "mode": "agent",
        "native": true,
        "hidden": false,
        "color": "#FF5733",
        "model": "claude-sonnet-4-20250514",
        "tools": ["read_file", "write_file", "bash"],
        "system": "You are a helpful coding assistant."
    });

    let agent: Agent = serde_json::from_value(json).unwrap();
    assert_eq!(agent.name, "coder");
    assert_eq!(agent.mode.as_deref(), Some("agent"));
    assert_eq!(agent.native, Some(true));
    assert_eq!(agent.hidden, Some(false));
    let tools = agent.tools.unwrap();
    assert_eq!(tools.len(), 3);
}

// ============================================================================
// Edge case and stress tests
// ============================================================================

#[test]
fn test_session_with_null_optional_fields() {
    let json = json!({
        "id": "sess_null",
        "projectID": "proj_1",
        "slug": null,
        "directory": null,
        "parentID": null,
        "summary": null,
        "share": null,
        "title": null,
        "version": null,
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "permission": null,
        "revert": null
    });

    let session: Session = serde_json::from_value(json).unwrap();
    assert!(session.slug.is_none());
    assert!(session.directory.is_none());
    assert!(session.parent_id.is_none());
    assert!(session.summary.is_none());
    assert!(session.share.is_none());
    assert!(session.title.is_none());
    assert!(session.version.is_none());
    assert!(session.permission.is_none());
    assert!(session.revert.is_none());
}

#[test]
fn test_assistant_message_empty_path() {
    let json = json!({
        "role": "assistant",
        "id": "msg_ep",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "path": []
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    match msg {
        Message::Assistant(asst) => {
            assert_eq!(asst.path.unwrap(), json!([]));
        }
        _ => panic!("Expected Message::Assistant"),
    }
}

#[test]
fn test_tool_state_completed_null_fields() {
    let json = json!({
        "status": "completed",
        "output": null,
        "title": null
    });

    let state: ToolState = serde_json::from_value(json).unwrap();
    match state {
        ToolState::Completed { output, title, .. } => {
            assert!(output.is_none());
            assert!(title.is_none());
        }
        _ => panic!("Expected ToolState::Completed"),
    }
}

#[test]
fn test_tool_state_error_null_error() {
    let json = json!({
        "status": "error",
        "error": null
    });

    let state: ToolState = serde_json::from_value(json).unwrap();
    match state {
        ToolState::Error { error, .. } => {
            assert!(error.is_none());
        }
        _ => panic!("Expected ToolState::Error"),
    }
}

#[test]
fn test_message_tokens_all_none() {
    let json = json!({});

    let tokens: MessageTokens = serde_json::from_value(json).unwrap();
    assert!(tokens.input.is_none());
    assert!(tokens.output.is_none());
    assert!(tokens.reasoning.is_none());
    assert!(tokens.cache.is_none());
}

#[test]
fn test_part_delta_with_object_delta() {
    let json = json!({
        "type": "message.part.delta",
        "properties": {
            "sessionID": "sess_1",
            "messageID": "msg_1",
            "partID": "part_1",
            "field": "state",
            "delta": {
                "status": "completed",
                "output": "done"
            }
        }
    });

    let event: Event = serde_json::from_value(json).unwrap();
    match event {
        Event::MessagePartDelta { properties } => {
            assert_eq!(properties.field, "state");
            assert!(properties.delta.is_object());
        }
        _ => panic!("Expected Event::MessagePartDelta"),
    }
}

#[test]
fn test_permission_request_empty_patterns() {
    let json = json!({
        "id": "perm_ep",
        "sessionID": "sess_1",
        "patterns": []
    });

    let req: PermissionRequest = serde_json::from_value(json).unwrap();
    assert!(req.patterns.unwrap().is_empty());
}

#[test]
fn test_project_paths_with_extra() {
    let json = json!({
        "root": "/project",
        "config": "/config",
        "data": "/data",
        "state": "/state",
        "cache": "/cache",
        "extra": {
            "logs": "/var/log/opencode"
        }
    });

    let paths: ProjectPaths = serde_json::from_value(json).unwrap();
    let extra = paths.extra.unwrap();
    assert_eq!(extra.get("logs").unwrap(), "/var/log/opencode");
}

#[test]
fn test_lsp_diagnostics() {
    let json = json!({
        "uri": "file:///src/main.rs",
        "diagnostics": [
            {
                "message": "unused variable `x`",
                "severity": 2,
                "range": {
                    "start": { "line": 5, "character": 8 },
                    "end": { "line": 5, "character": 9 }
                },
                "source": "rust-analyzer"
            }
        ]
    });

    let diag: LSPDiagnostics = serde_json::from_value(json).unwrap();
    assert_eq!(diag.uri.as_deref(), Some("file:///src/main.rs"));
    let diagnostics = diag.diagnostics.unwrap();
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(
        diagnostics[0].message.as_deref(),
        Some("unused variable `x`")
    );
    assert_eq!(diagnostics[0].severity, Some(2));
    assert_eq!(diagnostics[0].source.as_deref(), Some("rust-analyzer"));
}

#[test]
fn test_mcp_tools_info() {
    let json = json!({
        "server": "filesystem",
        "tools": [
            {
                "name": "read_file",
                "description": "Read a file from disk",
                "server": "filesystem",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": { "type": "string" }
                    }
                }
            }
        ]
    });

    let info: MCPToolsInfo = serde_json::from_value(json).unwrap();
    assert_eq!(info.server.as_deref(), Some("filesystem"));
    let tools = info.tools.unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name.as_deref(), Some("read_file"));
    assert!(tools[0].input_schema.is_some());
}

#[test]
fn test_event_serialization_preserves_type_tag() {
    let event = Event::SessionIdle {
        properties: SessionIdProps {
            session_id: "sess_test".to_string(),
        },
    };

    let serialized = serde_json::to_value(&event).unwrap();
    assert_eq!(serialized["type"], "session.idle");
    assert_eq!(serialized["properties"]["sessionID"], "sess_test");
}

#[test]
fn test_message_serialization_preserves_role_tag() {
    let json = json!({
        "role": "user",
        "id": "msg_ser",
        "sessionID": "sess_1",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        }
    });

    let msg: Message = serde_json::from_value(json).unwrap();
    // Verify deserialization produced the right variant
    assert!(matches!(msg, Message::User(_)));
    let serialized = serde_json::to_value(&msg).unwrap();
    assert_eq!(serialized["id"], "msg_ser");
}

#[test]
fn test_part_serialization_preserves_type_tag() {
    let json = json!({
        "type": "reasoning",
        "id": "part_ser",
        "time": {
            "created": "2025-01-15T10:30:00Z",
            "updated": "2025-01-15T10:30:00Z"
        },
        "content": "Thinking..."
    });

    let part: Part = serde_json::from_value(json).unwrap();
    let serialized = serde_json::to_value(&part).unwrap();
    assert_eq!(serialized["type"], "reasoning");
    assert_eq!(serialized["content"], "Thinking...");
}

#[test]
fn test_tool_state_serialization_preserves_status_tag() {
    let state = ToolState::Completed {
        input: None,
        output: Some("result".to_string()),
        title: Some("Done".to_string()),
    };

    let serialized = serde_json::to_value(&state).unwrap();
    assert_eq!(serialized["status"], "completed");
    assert_eq!(serialized["output"], "result");
    assert_eq!(serialized["title"], "Done");
}

#[test]
fn test_session_status_serialization_preserves_type_tag() {
    let status = SessionStatus::Retry {
        attempt: Some(3),
        message: Some("Retrying...".to_string()),
        next: None,
    };

    let serialized = serde_json::to_value(&status).unwrap();
    assert_eq!(serialized["type"], "retry");
    assert_eq!(serialized["attempt"], 3);
    assert_eq!(serialized["message"], "Retrying...");
}
