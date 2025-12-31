//! Hot-reload WebSocket functionality for live workspace updates.
//!
//! This module provides a WebSocket endpoint that clients can connect to
//! for receiving real-time notifications when workspace files change.
//!
//! ## Features
//!
//! - Automatic reconnection with exponential backoff
//! - Workspace-scoped subscriptions
//! - Granular change notifications (DSL, positions, documentation)
//! - Support for new/deleted workspace detection

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    response::IntoResponse,
};
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::editor::EditorMessage;
use crate::state::AppState;

/// Query parameters for hot-reload WebSocket connection.
#[derive(Debug, Deserialize)]
pub struct HotReloadParams {
    /// Optional workspace ID to subscribe to specific workspace changes.
    /// If not provided, subscribes to all workspace changes.
    pub workspace_id: Option<String>,
}

/// WebSocket handler for hot-reload subscriptions.
///
/// Clients connect to `/ws/reload` optionally with `?workspace_id={id}` to
/// receive notifications when workspace files change.
pub async fn hot_reload_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Query(params): Query<HotReloadParams>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_hot_reload_socket(socket, state, params))
}

/// Handle the hot-reload WebSocket connection.
async fn handle_hot_reload_socket(
    mut socket: WebSocket,
    state: AppState,
    params: HotReloadParams,
) {
    let workspace_filter = params.workspace_id.clone();

    info!(
        "Hot-reload WebSocket connected{}",
        workspace_filter
            .as_ref()
            .map(|id| format!(" for workspace: {}", id))
            .unwrap_or_default()
    );

    let editor = state.editor.clone();
    let mut receiver = editor.broadcast.subscribe();

    // Send initial connected acknowledgment
    let connected_msg = serde_json::json!({
        "type": "connected",
        "workspace_id": workspace_filter,
        "timestamp": chrono::Utc::now().timestamp()
    });
    if let Ok(json) = serde_json::to_string(&connected_msg) {
        let _ = socket.send(Message::Text(json.into())).await;
    }

    loop {
        tokio::select! {
            // Handle incoming messages from client (ping/pong, close)
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        // Handle ping messages
                        if text.contains("\"type\":\"ping\"") {
                            let pong = serde_json::json!({
                                "type": "pong",
                                "timestamp": chrono::Utc::now().timestamp()
                            });
                            if let Ok(json) = serde_json::to_string(&pong) {
                                let _ = socket.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Ok(Message::Ping(data)) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    Ok(Message::Close(_)) => {
                        info!(
                            "Hot-reload WebSocket closed{}",
                            workspace_filter
                                .as_ref()
                                .map(|id| format!(" for workspace: {}", id))
                                .unwrap_or_default()
                        );
                        break;
                    }
                    Err(e) => {
                        warn!("Hot-reload WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            // Broadcast reload events to this client
            Ok(broadcast_msg) = receiver.recv() => {
                // Filter messages based on workspace subscription
                let should_send = should_send_message(&broadcast_msg, &workspace_filter);

                if should_send {
                    if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                        debug!("Sending hot-reload message: {}", json);
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Determine if a message should be sent to a client based on their workspace filter.
fn should_send_message(msg: &EditorMessage, workspace_filter: &Option<String>) -> bool {
    match msg {
        // Always send global workspace list updates
        EditorMessage::WorkspaceListUpdated { .. } => true,

        // Always send workspace added/deleted (client can filter client-side if needed)
        EditorMessage::WorkspaceAdded { .. } => true,
        EditorMessage::WorkspaceDeleted { .. } => true,

        // Filter workspace-specific messages
        EditorMessage::WorkspaceReloaded { .. } => {
            // WorkspaceReloaded doesn't have workspace_id, so send to all
            // This is for backward compatibility
            true
        }

        EditorMessage::DiagramUpdated { workspace_id, .. } => {
            matches_workspace(workspace_id, workspace_filter)
        }

        EditorMessage::PositionsUpdated { workspace_id, .. } => {
            matches_workspace(workspace_id, workspace_filter)
        }

        EditorMessage::DocumentationUpdated { workspace_id, .. } => {
            matches_workspace(workspace_id, workspace_filter)
        }

        // Don't send editor-specific messages through hot-reload
        EditorMessage::ElementMoved { .. } => false,
        EditorMessage::RelationshipVertexMoved { .. } => false,
        EditorMessage::SelectionChanged { .. } => false,
        EditorMessage::RequestState { .. } => false,
        EditorMessage::State { .. } => false,
        EditorMessage::Undo { .. } => false,
        EditorMessage::Redo { .. } => false,
        EditorMessage::AutoLayout { .. } => false,
        EditorMessage::Save => false,
        EditorMessage::Error { .. } => false,
        EditorMessage::Ack { .. } => false,
    }
}

/// Check if a workspace ID matches the filter.
fn matches_workspace(workspace_id: &str, filter: &Option<String>) -> bool {
    match filter {
        None => true, // No filter means subscribe to all
        Some(filter_id) => workspace_id == filter_id,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_workspace_no_filter() {
        assert!(matches_workspace("any-workspace", &None));
    }

    #[test]
    fn test_matches_workspace_with_filter() {
        let filter = Some("my-workspace".to_string());
        assert!(matches_workspace("my-workspace", &filter));
        assert!(!matches_workspace("other-workspace", &filter));
    }

    #[test]
    fn test_should_send_workspace_list_updated() {
        let msg = EditorMessage::WorkspaceListUpdated {
            timestamp: 12345,
        };
        assert!(should_send_message(&msg, &None));
        assert!(should_send_message(&msg, &Some("any".to_string())));
    }

    #[test]
    fn test_should_send_diagram_updated_filtered() {
        let msg = EditorMessage::DiagramUpdated {
            workspace_id: "my-workspace".to_string(),
            view_key: None,
            timestamp: 12345,
        };

        // No filter - should send
        assert!(should_send_message(&msg, &None));

        // Matching filter - should send
        assert!(should_send_message(&msg, &Some("my-workspace".to_string())));

        // Non-matching filter - should not send
        assert!(!should_send_message(&msg, &Some("other-workspace".to_string())));
    }

    #[test]
    fn test_should_not_send_editor_messages() {
        let msg = EditorMessage::ElementMoved {
            view_key: "test".to_string(),
            element_id: "elem1".to_string(),
            x: 100,
            y: 200,
        };
        assert!(!should_send_message(&msg, &None));
    }
}
