use crate::api::ApiClient;
use crate::app::state::{App, Dialog, Page};
use crate::types::{
    Event, MessagePartDeltaProps, MessageUpdatedProps, MessageWithParts, PartUpdatedProps,
    PermissionAskedProps, PermissionRepliedProps, SessionIdProps, SessionInfoProps,
    SessionStatusProps, TodoUpdatedProps,
};
use crate::types::{
    AssistantMessage, Message, MessageTime, Part, PermissionReply, PermissionRequest,
    Session, SessionStatus, SessionTime, TextPart, Todo, TodoStatus, UserMessage,
};

// ── Helpers ─────────────────────────────────────────────────────────────

fn test_app() -> App {
    let client = ApiClient::new("http://localhost:4000".to_string());
    App::new(client, "http://localhost:4000".to_string())
}

fn make_session(id: &str) -> Session {
    Session {
        id: id.to_string(),
        slug: None,
        project_id: Some("proj-1".to_string()),
        directory: None,
        parent_id: None,
        summary: None,
        share: None,
        title: Some(format!("Session {}", id)),
        version: None,
        time: SessionTime {
            created: serde_json::json!(1704067200000i64),
            updated: serde_json::json!(1704067200000i64),
            archived: None,
            initialized: None,
        },
        permission: None,
        revert: None,
    }
}

fn make_user_message(id: &str, session_id: &str) -> Message {
    Message::User(UserMessage {
        id: id.to_string(),
        session_id: session_id.to_string(),
        role: Some("user".to_string()),
        time: MessageTime {
            created: serde_json::json!(1704067200000_i64),
            updated: Some(serde_json::json!(1704067200000_i64)),
            completed: None,
        },
        format: None,
        summary: None,
        agent: None,
        model: None,
        system: None,
        tools: None,
        variant: None,
    })
}

fn make_assistant_message(id: &str, session_id: &str) -> Message {
    Message::Assistant(AssistantMessage {
        id: id.to_string(),
        session_id: session_id.to_string(),
        role: Some("assistant".to_string()),
        time: MessageTime {
            created: serde_json::json!(1704067200000_i64),
            updated: Some(serde_json::json!(1704067200000_i64)),
            completed: None,
        },
        error: None,
        parent_id: None,
        model_id: None,
        provider_id: None,
        mode: None,
        agent: None,
        path: None,
        cost: None,
        tokens: None,
        system: None,
        finish: None,
    })
}

fn make_text_part(id: &str, session_id: &str, message_id: &str) -> Part {
    Part::Text(TextPart {
        id: id.to_string(),
        session_id: Some(session_id.to_string()),
        message_id: Some(message_id.to_string()),
        time: None,
        text: Some("Hello world".to_string()),
    })
}

fn make_permission_request(id: &str, session_id: &str) -> PermissionRequest {
    PermissionRequest {
        id: id.to_string(),
        session_id: session_id.to_string(),
        permission: Some("bash".to_string()),
        patterns: Some(vec!["echo hello".to_string()]),
        metadata: None,
        always: None,
        tool: Some(serde_json::json!({"messageID": "msg_1", "callID": "call_1"})),
        description: Some("Print hello".to_string()),
        input: None,
    }
}

/// Set a current session on the app so that session-scoped event handlers
/// can match against it.
fn set_current_session(app: &mut App, session_id: &str) {
    let session = make_session(session_id);
    app.current_session = Some(session.clone());
    if !app.sessions.iter().any(|s| s.id == session.id) {
        app.sessions.push(session);
    }
}

// ── App::new tests ──────────────────────────────────────────────────────

#[test]
fn test_new_initializes_fields_correctly() {
    let app = test_app();

    assert_eq!(app.base_url, "http://localhost:4000");
    assert!(!app.connected);

    assert!(app.sessions.is_empty());
    assert!(app.current_session.is_none());
    assert!(app.session_statuses.is_empty());

    assert!(app.messages.is_empty());
    assert_eq!(app.message_scroll, 0);

    assert!(app.providers.is_empty());
    assert!(app.current_model.is_none());

    assert!(app.agents.is_empty());
    assert!(app.commands.is_empty());

    assert!(app.project.is_none());
    assert!(app.todos.is_empty());

    assert!(app.pending_permissions.is_empty());
    assert!(app.pending_questions.is_empty());

    assert_eq!(app.active_page, Page::Chat);
    assert!(app.active_dialog.is_none());
    assert!(app.input_text.is_empty());
    assert_eq!(app.input_cursor, 0);
    assert!(!app.sidebar_visible);
    assert_eq!(app.sidebar_selected, 0);

    assert!(app.dialog_filter.is_empty());
    assert_eq!(app.dialog_selected, 0);

    assert!(!app.should_quit);
    assert!(app.toast.is_none());
    assert!(app.toast_time.is_none());
    assert!(app.status_message.is_empty());
    assert!(!app.is_busy);

    assert_eq!(app.theme_name, "default");
}

// ── SessionCreated ──────────────────────────────────────────────────────

#[test]
fn test_handle_session_created_adds_session() {
    let mut app = test_app();
    let session = make_session("s-1");

    app.handle_event(Event::SessionCreated {
        properties: SessionInfoProps {
            info: session.clone(),
        },
    });

    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.sessions[0].id, "s-1");
}

#[test]
fn test_handle_session_created_does_not_duplicate() {
    let mut app = test_app();
    let session = make_session("s-1");

    app.handle_event(Event::SessionCreated {
        properties: SessionInfoProps {
            info: session.clone(),
        },
    });
    app.handle_event(Event::SessionCreated {
        properties: SessionInfoProps {
            info: session.clone(),
        },
    });

    assert_eq!(app.sessions.len(), 1);
}

// ── SessionUpdated ──────────────────────────────────────────────────────

#[test]
fn test_handle_session_updated_updates_existing() {
    let mut app = test_app();
    let session = make_session("s-1");
    app.sessions.push(session);

    let mut updated = make_session("s-1");
    updated.title = Some("Updated Title".to_string());

    app.handle_event(Event::SessionUpdated {
        properties: SessionInfoProps {
            info: updated.clone(),
        },
    });

    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.sessions[0].title.as_deref(), Some("Updated Title"));
}

#[test]
fn test_handle_session_updated_updates_current_session() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let mut updated = make_session("s-1");
    updated.title = Some("New Title".to_string());

    app.handle_event(Event::SessionUpdated {
        properties: SessionInfoProps {
            info: updated.clone(),
        },
    });

    assert_eq!(
        app.current_session.as_ref().unwrap().title.as_deref(),
        Some("New Title")
    );
}

// ── SessionDeleted ──────────────────────────────────────────────────────

#[test]
fn test_handle_session_deleted_removes_session() {
    let mut app = test_app();
    app.sessions.push(make_session("s-1"));
    app.sessions.push(make_session("s-2"));

    app.handle_event(Event::SessionDeleted {
        properties: SessionInfoProps {
            info: make_session("s-1"),
        },
    });

    assert_eq!(app.sessions.len(), 1);
    assert_eq!(app.sessions[0].id, "s-2");
}

#[test]
fn test_handle_session_deleted_clears_current_if_matching() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.messages.push(MessageWithParts {
        info: make_user_message("m-1", "s-1"),
        parts: vec![],
    });

    app.handle_event(Event::SessionDeleted {
        properties: SessionInfoProps {
            info: make_session("s-1"),
        },
    });

    assert!(app.current_session.is_none());
    assert!(app.messages.is_empty());
}

#[test]
fn test_handle_session_deleted_does_not_clear_current_if_different() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.sessions.push(make_session("s-2"));

    app.handle_event(Event::SessionDeleted {
        properties: SessionInfoProps {
            info: make_session("s-2"),
        },
    });

    assert!(app.current_session.is_some());
    assert_eq!(app.current_session.as_ref().unwrap().id, "s-1");
}

// ── SessionStatus ───────────────────────────────────────────────────────

#[test]
fn test_handle_session_status_updates_status_and_busy() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    app.handle_event(Event::SessionStatus {
        properties: SessionStatusProps {
            session_id: "s-1".to_string(),
            status: SessionStatus::Busy,
        },
    });

    assert!(matches!(
        app.session_statuses.get("s-1"),
        Some(SessionStatus::Busy)
    ));
    assert!(app.is_busy);
}

#[test]
fn test_handle_session_status_idle_not_busy() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.is_busy = true;

    app.handle_event(Event::SessionStatus {
        properties: SessionStatusProps {
            session_id: "s-1".to_string(),
            status: SessionStatus::Idle,
        },
    });

    assert!(!app.is_busy);
}

#[test]
fn test_handle_session_status_different_session_does_not_change_busy() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.is_busy = false;

    app.handle_event(Event::SessionStatus {
        properties: SessionStatusProps {
            session_id: "s-other".to_string(),
            status: SessionStatus::Busy,
        },
    });

    assert!(!app.is_busy);
}

// ── SessionIdle ─────────────────────────────────────────────────────────

#[test]
fn test_handle_session_idle_marks_not_busy() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.is_busy = true;

    app.handle_event(Event::SessionIdle {
        properties: SessionIdProps {
            session_id: "s-1".to_string(),
        },
    });

    assert!(!app.is_busy);
    assert!(matches!(
        app.session_statuses.get("s-1"),
        Some(SessionStatus::Idle)
    ));
}

#[test]
fn test_handle_session_idle_different_session_no_change() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.is_busy = true;

    app.handle_event(Event::SessionIdle {
        properties: SessionIdProps {
            session_id: "s-other".to_string(),
        },
    });

    assert!(app.is_busy);
}

// ── MessageUpdated ──────────────────────────────────────────────────────

#[test]
fn test_handle_message_updated_adds_new_message() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let msg = make_user_message("m-1", "s-1");
    app.handle_event(Event::MessageUpdated {
        properties: MessageUpdatedProps {
            session_id: Some("s-1".to_string()),
            info: msg.clone(),
        },
    });

    assert_eq!(app.messages.len(), 1);
}

#[test]
fn test_handle_message_updated_updates_existing_message() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let msg = make_user_message("m-1", "s-1");
    app.messages.push(MessageWithParts {
        info: msg.clone(),
        parts: vec![],
    });

    // Update the same message
    let updated_msg = make_assistant_message("m-1", "s-1");
    app.handle_event(Event::MessageUpdated {
        properties: MessageUpdatedProps {
            session_id: Some("s-1".to_string()),
            info: updated_msg.clone(),
        },
    });

    // Should still be 1 message (updated in place), not 2
    assert_eq!(app.messages.len(), 1);
}

#[test]
fn test_handle_message_updated_ignores_other_session() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let msg = make_user_message("m-1", "s-other");
    app.handle_event(Event::MessageUpdated {
        properties: MessageUpdatedProps {
            session_id: Some("s-other".to_string()),
            info: msg,
        },
    });

    assert!(app.messages.is_empty());
}

// ── MessagePartUpdated ──────────────────────────────────────────────────

#[test]
fn test_handle_message_part_updated_adds_part() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let msg = make_user_message("m-1", "s-1");
    app.messages.push(MessageWithParts {
        info: msg,
        parts: vec![],
    });

    let part = make_text_part("p-1", "s-1", "m-1");
    app.handle_event(Event::MessagePartUpdated {
        properties: PartUpdatedProps {
            session_id: Some("s-1".to_string()),
            message_id: Some("m-1".to_string()),
            part: part.clone(),
        },
    });

    assert_eq!(app.messages[0].parts.len(), 1);
}

#[test]
fn test_handle_message_part_updated_updates_existing_part() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let part = make_text_part("p-1", "s-1", "m-1");
    app.messages.push(MessageWithParts {
        info: make_user_message("m-1", "s-1"),
        parts: vec![part],
    });

    // Create an updated part with new content
    let updated_part = Part::Text(TextPart {
        id: "p-1".to_string(),
        session_id: Some("s-1".to_string()),
        message_id: Some("m-1".to_string()),
        time: None,
        text: Some("Updated content".to_string()),
    });

    app.handle_event(Event::MessagePartUpdated {
        properties: PartUpdatedProps {
            session_id: Some("s-1".to_string()),
            message_id: Some("m-1".to_string()),
            part: updated_part,
        },
    });

    assert_eq!(app.messages[0].parts.len(), 1);
    if let Part::Text(ref tp) = app.messages[0].parts[0] {
        assert_eq!(tp.text.as_deref(), Some("Updated content"));
    } else {
        panic!("Expected Text part");
    }
}

// ── PermissionAsked ─────────────────────────────────────────────────────

#[test]
fn test_handle_permission_asked_adds_to_pending_and_opens_dialog() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let request = make_permission_request("perm-1", "s-1");
    app.handle_event(Event::PermissionAsked {
        properties: PermissionAskedProps {
            request: request.clone(),
        },
    });

    assert_eq!(app.pending_permissions.len(), 1);
    assert_eq!(app.pending_permissions[0].id, "perm-1");
    assert_eq!(app.active_dialog, Some(Dialog::Permission));
}

#[test]
fn test_handle_permission_asked_does_not_override_existing_dialog() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.active_dialog = Some(Dialog::Help);

    let request = make_permission_request("perm-1", "s-1");
    app.handle_event(Event::PermissionAsked {
        properties: PermissionAskedProps { request },
    });

    assert_eq!(app.pending_permissions.len(), 1);
    // Dialog should stay as Help, not be overridden
    assert_eq!(app.active_dialog, Some(Dialog::Help));
}

#[test]
fn test_handle_permission_asked_ignores_other_session() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let request = make_permission_request("perm-1", "s-other");
    app.handle_event(Event::PermissionAsked {
        properties: PermissionAskedProps { request },
    });

    assert!(app.pending_permissions.is_empty());
    assert!(app.active_dialog.is_none());
}

// ── PermissionReplied ───────────────────────────────────────────────────

#[test]
fn test_handle_permission_replied_removes_from_pending() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.pending_permissions
        .push(make_permission_request("perm-1", "s-1"));
    app.active_dialog = Some(Dialog::Permission);

    app.handle_event(Event::PermissionReplied {
        properties: PermissionRepliedProps {
            session_id: "s-1".to_string(),
            request_id: "perm-1".to_string(),
            reply: PermissionReply::Once,
        },
    });

    assert!(app.pending_permissions.is_empty());
    assert!(app.active_dialog.is_none());
}

#[test]
fn test_handle_permission_replied_keeps_dialog_if_more_pending() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.pending_permissions
        .push(make_permission_request("perm-1", "s-1"));
    app.pending_permissions
        .push(make_permission_request("perm-2", "s-1"));
    app.active_dialog = Some(Dialog::Permission);

    app.handle_event(Event::PermissionReplied {
        properties: PermissionRepliedProps {
            session_id: "s-1".to_string(),
            request_id: "perm-1".to_string(),
            reply: PermissionReply::Once,
        },
    });

    assert_eq!(app.pending_permissions.len(), 1);
    assert_eq!(app.active_dialog, Some(Dialog::Permission));
}

// ── TodoUpdated ─────────────────────────────────────────────────────────

#[test]
fn test_handle_todo_updated() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    let todos = vec![
        Todo {
            id: Some("t-1".to_string()),
            content: "First task".to_string(),
            status: TodoStatus::Pending,
        },
        Todo {
            id: Some("t-2".to_string()),
            content: "Second task".to_string(),
            status: TodoStatus::Completed,
        },
    ];

    app.handle_event(Event::TodoUpdated {
        properties: TodoUpdatedProps {
            session_id: "s-1".to_string(),
            todos: Some(todos),
        },
    });

    assert_eq!(app.todos.len(), 2);
    assert_eq!(app.todos[0].content, "First task");
}

#[test]
fn test_handle_todo_updated_with_none_clears_todos() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.todos.push(Todo {
        id: Some("t-1".to_string()),
        content: "Existing".to_string(),
        status: TodoStatus::Pending,
    });

    app.handle_event(Event::TodoUpdated {
        properties: TodoUpdatedProps {
            session_id: "s-1".to_string(),
            todos: None,
        },
    });

    assert!(app.todos.is_empty());
}

#[test]
fn test_handle_todo_updated_ignores_other_session() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    app.handle_event(Event::TodoUpdated {
        properties: TodoUpdatedProps {
            session_id: "s-other".to_string(),
            todos: Some(vec![Todo {
                id: Some("t-1".to_string()),
                content: "Should not appear".to_string(),
                status: TodoStatus::Pending,
            }]),
        },
    });

    assert!(app.todos.is_empty());
}

// ── Input editing ───────────────────────────────────────────────────────

#[test]
fn test_insert_char() {
    let mut app = test_app();
    app.insert_char('H');
    app.insert_char('i');

    assert_eq!(app.input_text, "Hi");
    assert_eq!(app.input_cursor, 2);
}

#[test]
fn test_insert_char_at_middle() {
    let mut app = test_app();
    app.input_text = "Hllo".to_string();
    app.input_cursor = 1;

    app.insert_char('e');

    assert_eq!(app.input_text, "Hello");
    assert_eq!(app.input_cursor, 2);
}

#[test]
fn test_insert_char_multibyte() {
    let mut app = test_app();
    app.insert_char('a');
    app.insert_char('\u{00e9}'); // e with accent
    app.insert_char('b');

    assert_eq!(app.input_text, "a\u{00e9}b");
    assert_eq!(app.input_cursor, 3);
}

#[test]
fn test_delete_char_backspace() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 5;

    app.delete_char();

    assert_eq!(app.input_text, "Hell");
    assert_eq!(app.input_cursor, 4);
}

#[test]
fn test_delete_char_at_beginning_does_nothing() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 0;

    app.delete_char();

    assert_eq!(app.input_text, "Hello");
    assert_eq!(app.input_cursor, 0);
}

#[test]
fn test_delete_char_from_middle() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 3;

    app.delete_char();

    assert_eq!(app.input_text, "Helo");
    assert_eq!(app.input_cursor, 2);
}

#[test]
fn test_move_cursor_left() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 3;

    app.move_cursor_left();
    assert_eq!(app.input_cursor, 2);

    app.move_cursor_left();
    assert_eq!(app.input_cursor, 1);
}

#[test]
fn test_move_cursor_left_at_zero_stays() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 0;

    app.move_cursor_left();
    assert_eq!(app.input_cursor, 0);
}

#[test]
fn test_move_cursor_right() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 2;

    app.move_cursor_right();
    assert_eq!(app.input_cursor, 3);
}

#[test]
fn test_move_cursor_right_at_end_stays() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 5;

    app.move_cursor_right();
    assert_eq!(app.input_cursor, 5);
}

#[test]
fn test_move_cursor_home() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 3;

    app.move_cursor_home();
    assert_eq!(app.input_cursor, 0);
}

#[test]
fn test_move_cursor_end() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 1;

    app.move_cursor_end();
    assert_eq!(app.input_cursor, 5);
}

// ── Scrolling ───────────────────────────────────────────────────────────

#[test]
fn test_scroll_up() {
    let mut app = test_app();
    assert_eq!(app.message_scroll, 0);

    app.scroll_up();
    assert_eq!(app.message_scroll, 1);

    app.scroll_up();
    assert_eq!(app.message_scroll, 2);
}

#[test]
fn test_scroll_down() {
    let mut app = test_app();
    app.message_scroll = 5;

    app.scroll_down();
    assert_eq!(app.message_scroll, 4);
}

#[test]
fn test_scroll_down_saturates_at_zero() {
    let mut app = test_app();
    app.message_scroll = 0;

    app.scroll_down();
    assert_eq!(app.message_scroll, 0);
}

// ── UI helpers ──────────────────────────────────────────────────────────

#[test]
fn test_toggle_sidebar() {
    let mut app = test_app();
    assert!(!app.sidebar_visible);

    app.toggle_sidebar();
    assert!(app.sidebar_visible);

    app.toggle_sidebar();
    assert!(!app.sidebar_visible);
}

#[test]
fn test_open_dialog() {
    let mut app = test_app();
    app.dialog_filter = "leftover".to_string();
    app.dialog_selected = 5;

    app.open_dialog(Dialog::Sessions);

    assert_eq!(app.active_dialog, Some(Dialog::Sessions));
    assert!(app.dialog_filter.is_empty());
    assert_eq!(app.dialog_selected, 0);
}

#[test]
fn test_close_dialog() {
    let mut app = test_app();
    app.active_dialog = Some(Dialog::Help);
    app.dialog_filter = "filter text".to_string();
    app.dialog_selected = 3;

    app.close_dialog();

    assert!(app.active_dialog.is_none());
    assert!(app.dialog_filter.is_empty());
    assert_eq!(app.dialog_selected, 0);
}

// ── Additional edge cases ───────────────────────────────────────────────

#[test]
fn test_current_session_title_with_session() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    assert_eq!(app.current_session_title(), "Session s-1");
}

#[test]
fn test_current_session_title_without_session() {
    let app = test_app();
    assert_eq!(app.current_session_title(), "New Session");
}

#[test]
fn test_is_session_busy_from_status_map() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    app.session_statuses
        .insert("s-1".to_string(), SessionStatus::Busy);

    assert!(app.is_session_busy());
}

#[test]
fn test_is_session_busy_fallback_to_is_busy() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");
    // No entry in session_statuses, falls back to is_busy field
    app.is_busy = true;

    assert!(app.is_session_busy());
}

#[test]
fn test_model_display_name_no_model() {
    let app = test_app();
    assert_eq!(app.model_display_name(), "No model");
}

#[test]
fn test_project_name_no_project() {
    let app = test_app();
    assert_eq!(app.project_name(), "opencode");
}

#[test]
fn test_delete_char_forward() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 0;

    app.delete_char_forward();

    assert_eq!(app.input_text, "ello");
    assert_eq!(app.input_cursor, 0);
}

#[test]
fn test_delete_char_forward_at_end_does_nothing() {
    let mut app = test_app();
    app.input_text = "Hello".to_string();
    app.input_cursor = 5;

    app.delete_char_forward();

    assert_eq!(app.input_text, "Hello");
    assert_eq!(app.input_cursor, 5);
}

#[test]
fn test_message_part_delta_appends_text() {
    let mut app = test_app();
    set_current_session(&mut app, "s-1");

    // Set up a message with a text part that has empty content
    let part = Part::Text(TextPart {
        id: "p-1".to_string(),
        session_id: Some("s-1".to_string()),
        message_id: Some("m-1".to_string()),
        time: None,
        text: Some("Hello".to_string()),
    });
    app.messages.push(MessageWithParts {
        info: make_assistant_message("m-1", "s-1"),
        parts: vec![part],
    });

    app.handle_event(Event::MessagePartDelta {
        properties: MessagePartDeltaProps {
            session_id: "s-1".to_string(),
            message_id: "m-1".to_string(),
            part_id: "p-1".to_string(),
            field: "text".to_string(),
            delta: serde_json::Value::String(" World".to_string()),
        },
    });

    if let Part::Text(ref tp) = app.messages[0].parts[0] {
        assert_eq!(tp.text.as_deref(), Some("Hello World"));
    } else {
        panic!("Expected Text part");
    }
}

#[test]
fn test_server_connected_sets_connected() {
    let mut app = test_app();
    assert!(!app.connected);

    app.handle_event(Event::ServerConnected {
        properties: serde_json::Value::Null,
    });

    assert!(app.connected);
}

#[test]
fn test_server_disposed_sets_disconnected() {
    let mut app = test_app();
    app.connected = true;

    app.handle_event(Event::GlobalDisposed {
        properties: serde_json::Value::Null,
    });

    assert!(!app.connected);
    assert_eq!(app.status_message, "Server disconnected");
}
