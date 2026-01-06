//! Real-time collaboration module
//!
//! Provides WebSocket-based real-time collaboration features:
//! - CRDT synchronization for concurrent editing
//! - File change notifications
//! - Presence awareness (who's viewing/editing)
//! - Collaborative session management

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension, Path, State,
    },
    response::IntoResponse,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Collaboration server state
pub struct CollaborationServer {
    /// Active sessions by workspace ID
    sessions: Arc<RwLock<HashMap<String, WorkspaceSession>>>,

    /// Broadcast channel for global events
    global_tx: broadcast::Sender<CollaborationEvent>,

    /// Configuration
    config: CollaborationConfig,
}

/// Collaboration configuration
#[derive(Debug, Clone)]
pub struct CollaborationConfig {
    /// Enable CRDT-based editing
    pub crdt_enabled: bool,

    /// Presence heartbeat interval in milliseconds
    pub presence_heartbeat_ms: u64,

    /// Presence timeout in milliseconds
    pub presence_timeout_ms: u64,

    /// Maximum clients per workspace
    pub max_clients_per_workspace: usize,
}

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            crdt_enabled: true,
            presence_heartbeat_ms: 5000,
            presence_timeout_ms: 30000,
            max_clients_per_workspace: 50,
        }
    }
}

/// A collaborative session for a workspace
pub struct WorkspaceSession {
    /// Workspace ID
    pub workspace_id: String,

    /// Connected clients
    pub clients: HashMap<Uuid, ClientConnection>,

    /// Broadcast channel for this workspace
    pub tx: broadcast::Sender<CollaborationEvent>,

    /// CRDT document state (if enabled)
    pub crdt_state: Option<CrdtState>,

    /// Last activity timestamp
    pub last_activity: Instant,
}

/// A connected client
pub struct ClientConnection {
    /// Client unique ID
    pub id: Uuid,

    /// Client display name (if provided)
    pub name: Option<String>,

    /// Current cursor position (line, column)
    pub cursor: Option<(usize, usize)>,

    /// Currently selected view
    pub active_view: Option<String>,

    /// Last heartbeat
    pub last_heartbeat: Instant,

    /// Client color for presence display
    pub color: String,
}

/// CRDT document state
pub struct CrdtState {
    /// Document content
    pub content: String,

    /// Operation log
    pub operations: Vec<CrdtOperation>,

    /// Vector clock
    pub clock: HashMap<Uuid, u64>,
}

/// CRDT operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrdtOperation {
    /// Operation ID
    pub id: Uuid,

    /// Client that made the operation
    pub client_id: Uuid,

    /// Operation type
    pub op_type: CrdtOpType,

    /// Timestamp
    pub timestamp: u64,
}

/// CRDT operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CrdtOpType {
    Insert { position: usize, text: String },
    Delete { position: usize, length: usize },
}

/// Collaboration events sent over WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CollaborationEvent {
    /// Client joined the workspace
    ClientJoined {
        client_id: String,
        name: Option<String>,
        color: String,
    },

    /// Client left the workspace
    ClientLeft {
        client_id: String,
    },

    /// Client cursor moved
    CursorMoved {
        client_id: String,
        line: usize,
        column: usize,
    },

    /// Client changed active view
    ViewChanged {
        client_id: String,
        view_key: String,
    },

    /// File changed on disk
    FileChanged {
        workspace_id: String,
        file_path: String,
        change_type: String,
    },

    /// Workspace reloaded
    WorkspaceReloaded {
        workspace_id: String,
    },

    /// CRDT operation
    CrdtOp {
        operation: CrdtOperation,
    },

    /// Full CRDT sync
    CrdtSync {
        content: String,
        operations: Vec<CrdtOperation>,
    },

    /// Presence update (list of all clients)
    PresenceUpdate {
        clients: Vec<PresenceInfo>,
    },

    /// Heartbeat response
    Heartbeat {
        server_time: u64,
    },

    /// Error
    Error {
        message: String,
    },
}

/// Client presence information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceInfo {
    pub client_id: String,
    pub name: Option<String>,
    pub color: String,
    pub cursor: Option<(usize, usize)>,
    pub active_view: Option<String>,
}

/// Messages received from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    /// Join a workspace
    Join {
        workspace_id: String,
        name: Option<String>,
    },

    /// Leave current workspace
    Leave,

    /// Update cursor position
    CursorMove {
        line: usize,
        column: usize,
    },

    /// Change active view
    ViewChange {
        view_key: String,
    },

    /// CRDT operation
    CrdtOp {
        op_type: CrdtOpType,
    },

    /// Request full sync
    SyncRequest,

    /// Heartbeat
    Heartbeat,
}

impl CollaborationServer {
    /// Create a new collaboration server
    pub fn new(config: CollaborationConfig) -> Self {
        let (global_tx, _) = broadcast::channel(1000);

        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            global_tx,
            config,
        }
    }

    /// Get or create a workspace session
    async fn get_or_create_session(&self, workspace_id: &str) -> broadcast::Sender<CollaborationEvent> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get(workspace_id) {
            return session.tx.clone();
        }

        // Create new session
        let (tx, _) = broadcast::channel(100);
        let session = WorkspaceSession {
            workspace_id: workspace_id.to_string(),
            clients: HashMap::new(),
            tx: tx.clone(),
            crdt_state: if self.config.crdt_enabled {
                Some(CrdtState {
                    content: String::new(),
                    operations: Vec::new(),
                    clock: HashMap::new(),
                })
            } else {
                None
            },
            last_activity: Instant::now(),
        };

        sessions.insert(workspace_id.to_string(), session);
        info!("Created new collaboration session for workspace: {}", workspace_id);

        tx
    }

    /// Add a client to a workspace session
    async fn add_client(&self, workspace_id: &str, client_id: Uuid, name: Option<String>) -> Option<String> {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(workspace_id) {
            // Check capacity
            if session.clients.len() >= self.config.max_clients_per_workspace {
                warn!("Workspace {} at capacity", workspace_id);
                return None;
            }

            // Generate a color for this client
            let color = generate_client_color(client_id);

            let client = ClientConnection {
                id: client_id,
                name: name.clone(),
                cursor: None,
                active_view: None,
                last_heartbeat: Instant::now(),
                color: color.clone(),
            };

            session.clients.insert(client_id, client);
            session.last_activity = Instant::now();

            // Broadcast join event
            let _ = session.tx.send(CollaborationEvent::ClientJoined {
                client_id: client_id.to_string(),
                name,
                color: color.clone(),
            });

            return Some(color);
        }

        None
    }

    /// Remove a client from a workspace session
    async fn remove_client(&self, workspace_id: &str, client_id: Uuid) {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(workspace_id) {
            session.clients.remove(&client_id);

            // Broadcast leave event
            let _ = session.tx.send(CollaborationEvent::ClientLeft {
                client_id: client_id.to_string(),
            });

            // Clean up empty sessions
            if session.clients.is_empty() {
                info!("Removing empty collaboration session: {}", workspace_id);
                sessions.remove(workspace_id);
            }
        }
    }

    /// Update client cursor position
    async fn update_cursor(&self, workspace_id: &str, client_id: Uuid, line: usize, column: usize) {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(workspace_id) {
            if let Some(client) = session.clients.get_mut(&client_id) {
                client.cursor = Some((line, column));
                client.last_heartbeat = Instant::now();
            }

            // Broadcast cursor update
            let _ = session.tx.send(CollaborationEvent::CursorMoved {
                client_id: client_id.to_string(),
                line,
                column,
            });
        }
    }

    /// Update client active view
    async fn update_view(&self, workspace_id: &str, client_id: Uuid, view_key: String) {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(workspace_id) {
            if let Some(client) = session.clients.get_mut(&client_id) {
                client.active_view = Some(view_key.clone());
                client.last_heartbeat = Instant::now();
            }

            // Broadcast view change
            let _ = session.tx.send(CollaborationEvent::ViewChanged {
                client_id: client_id.to_string(),
                view_key,
            });
        }
    }

    /// Apply a CRDT operation
    async fn apply_crdt_op(&self, workspace_id: &str, client_id: Uuid, op_type: CrdtOpType) {
        let mut sessions = self.sessions.write().await;

        if let Some(session) = sessions.get_mut(workspace_id) {
            if let Some(ref mut crdt) = session.crdt_state {
                // Update clock
                let client_clock = crdt.clock.entry(client_id).or_insert(0);
                *client_clock += 1;

                // Create operation
                let operation = CrdtOperation {
                    id: Uuid::new_v4(),
                    client_id,
                    op_type: op_type.clone(),
                    timestamp: *client_clock,
                };

                // Apply operation to content
                match &op_type {
                    CrdtOpType::Insert { position, text } => {
                        let pos = (*position).min(crdt.content.len());
                        crdt.content.insert_str(pos, text);
                    }
                    CrdtOpType::Delete { position, length } => {
                        let start = (*position).min(crdt.content.len());
                        let end = (start + length).min(crdt.content.len());
                        crdt.content.replace_range(start..end, "");
                    }
                }

                // Store operation
                crdt.operations.push(operation.clone());

                // Broadcast operation
                let _ = session.tx.send(CollaborationEvent::CrdtOp { operation });
            }
        }
    }

    /// Get presence information for a workspace
    async fn get_presence(&self, workspace_id: &str) -> Vec<PresenceInfo> {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(workspace_id) {
            session.clients.values()
                .map(|c| PresenceInfo {
                    client_id: c.id.to_string(),
                    name: c.name.clone(),
                    color: c.color.clone(),
                    cursor: c.cursor,
                    active_view: c.active_view.clone(),
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Broadcast a file change event
    pub async fn notify_file_change(&self, workspace_id: &str, file_path: &str, change_type: &str) {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(workspace_id) {
            let _ = session.tx.send(CollaborationEvent::FileChanged {
                workspace_id: workspace_id.to_string(),
                file_path: file_path.to_string(),
                change_type: change_type.to_string(),
            });
        }
    }

    /// Broadcast a workspace reload event
    pub async fn notify_workspace_reload(&self, workspace_id: &str) {
        let sessions = self.sessions.read().await;

        if let Some(session) = sessions.get(workspace_id) {
            let _ = session.tx.send(CollaborationEvent::WorkspaceReloaded {
                workspace_id: workspace_id.to_string(),
            });
        }
    }
}

/// Handle a WebSocket connection for collaboration
pub async fn collaboration_ws_handler(
    ws: WebSocketUpgrade,
    Extension(collab): Extension<Arc<CollaborationServer>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_collaboration_socket(socket, collab))
}

/// Handle the WebSocket connection
async fn handle_collaboration_socket(socket: WebSocket, collab: Arc<CollaborationServer>) {
    let (mut sender, mut receiver) = socket.split();

    let client_id = Uuid::new_v4();
    let mut current_workspace: Option<String> = None;
    let mut rx: Option<broadcast::Receiver<CollaborationEvent>> = None;

    info!("New collaboration client connected: {}", client_id);

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                match client_msg {
                                    ClientMessage::Join { workspace_id, name } => {
                                        // Leave current workspace if any
                                        if let Some(ref ws) = current_workspace {
                                            collab.remove_client(ws, client_id).await;
                                        }

                                        // Get or create session
                                        let session_tx = collab.get_or_create_session(&workspace_id).await;
                                        rx = Some(session_tx.subscribe());

                                        // Add client to session
                                        if let Some(color) = collab.add_client(&workspace_id, client_id, name.clone()).await {
                                            current_workspace = Some(workspace_id.clone());

                                            // Send presence update
                                            let presence = collab.get_presence(&workspace_id).await;
                                            let msg = serde_json::to_string(&CollaborationEvent::PresenceUpdate { clients: presence }).unwrap();
                                            let _ = sender.send(Message::Text(msg)).await;

                                            info!("Client {} joined workspace {}", client_id, workspace_id);
                                        } else {
                                            let msg = serde_json::to_string(&CollaborationEvent::Error {
                                                message: "Workspace at capacity".to_string()
                                            }).unwrap();
                                            let _ = sender.send(Message::Text(msg)).await;
                                        }
                                    }

                                    ClientMessage::Leave => {
                                        if let Some(ref ws) = current_workspace {
                                            collab.remove_client(ws, client_id).await;
                                            current_workspace = None;
                                            rx = None;
                                        }
                                    }

                                    ClientMessage::CursorMove { line, column } => {
                                        if let Some(ref ws) = current_workspace {
                                            collab.update_cursor(ws, client_id, line, column).await;
                                        }
                                    }

                                    ClientMessage::ViewChange { view_key } => {
                                        if let Some(ref ws) = current_workspace {
                                            collab.update_view(ws, client_id, view_key).await;
                                        }
                                    }

                                    ClientMessage::CrdtOp { op_type } => {
                                        if let Some(ref ws) = current_workspace {
                                            collab.apply_crdt_op(ws, client_id, op_type).await;
                                        }
                                    }

                                    ClientMessage::SyncRequest => {
                                        // TODO: Send full CRDT state
                                    }

                                    ClientMessage::Heartbeat => {
                                        let msg = serde_json::to_string(&CollaborationEvent::Heartbeat {
                                            server_time: std::time::SystemTime::now()
                                                .duration_since(std::time::UNIX_EPOCH)
                                                .unwrap()
                                                .as_millis() as u64,
                                        }).unwrap();
                                        let _ = sender.send(Message::Text(msg)).await;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse client message: {}", e);
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }

            // Forward events from the session to this client
            event = async {
                if let Some(ref mut receiver) = rx {
                    receiver.recv().await.ok()
                } else {
                    // No subscription, wait forever
                    std::future::pending::<Option<CollaborationEvent>>().await
                }
            } => {
                if let Some(event) = event {
                    // Don't send events back to the client that originated them
                    let should_send = match &event {
                        CollaborationEvent::CursorMoved { client_id: cid, .. } |
                        CollaborationEvent::ViewChanged { client_id: cid, .. } |
                        CollaborationEvent::ClientJoined { client_id: cid, .. } => {
                            cid != &client_id.to_string()
                        }
                        CollaborationEvent::CrdtOp { operation } => {
                            operation.client_id != client_id
                        }
                        _ => true,
                    };

                    if should_send {
                        let msg = serde_json::to_string(&event).unwrap();
                        if sender.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    // Clean up on disconnect
    if let Some(ref ws) = current_workspace {
        collab.remove_client(ws, client_id).await;
    }
    info!("Collaboration client disconnected: {}", client_id);
}

/// Generate a consistent color for a client based on their ID
fn generate_client_color(client_id: Uuid) -> String {
    let colors = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4",
        "#FFEAA7", "#DDA0DD", "#98D8C8", "#F7DC6F",
        "#BB8FCE", "#85C1E9", "#F8B500", "#00CED1",
    ];

    let index = (client_id.as_u128() % colors.len() as u128) as usize;
    colors[index].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let server = CollaborationServer::new(CollaborationConfig::default());
        let _tx = server.get_or_create_session("test-workspace").await;

        let sessions = server.sessions.read().await;
        assert!(sessions.contains_key("test-workspace"));
    }

    #[tokio::test]
    async fn test_add_remove_client() {
        let server = CollaborationServer::new(CollaborationConfig::default());
        let _tx = server.get_or_create_session("test-workspace").await;

        let client_id = Uuid::new_v4();
        let color = server.add_client("test-workspace", client_id, Some("Test User".to_string())).await;
        assert!(color.is_some());

        let presence = server.get_presence("test-workspace").await;
        assert_eq!(presence.len(), 1);

        server.remove_client("test-workspace", client_id).await;

        let sessions = server.sessions.read().await;
        assert!(!sessions.contains_key("test-workspace")); // Should be removed when empty
    }

    #[tokio::test]
    async fn test_cursor_update() {
        let server = CollaborationServer::new(CollaborationConfig::default());
        let _tx = server.get_or_create_session("test-workspace").await;

        let client_id = Uuid::new_v4();
        server.add_client("test-workspace", client_id, None).await;

        server.update_cursor("test-workspace", client_id, 10, 5).await;

        let presence = server.get_presence("test-workspace").await;
        assert_eq!(presence[0].cursor, Some((10, 5)));
    }
}