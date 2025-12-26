//! Diagram editor functionality with WebSocket support.
//!
//! This module provides real-time collaborative diagram editing capabilities:
//! - WebSocket-based element position updates
//! - Auto-layout algorithms (hierarchical and grid)
//! - Undo/redo support
//! - Multi-user synchronization
//!
//! ## Auto-Layout
//!
//! The auto-layout feature supports multiple algorithms:
//!
//! ### Hierarchical Layout
//! Uses a topological sort (Kahn's algorithm) to arrange elements in layers based on
//! their relationships. Elements with no incoming relationships appear in the first layer,
//! and subsequent layers are determined by the dependency graph.
//!
//! Supports four directions:
//! - TopBottom: Sources at top, targets below (default)
//! - BottomTop: Sources at bottom, targets above
//! - LeftRight: Sources on left, targets on right
//! - RightLeft: Sources on right, targets on left
//!
//! ### Grid Layout
//! Arranges elements in a simple grid pattern (roughly square).
//! Useful for diagrams with no clear hierarchical structure.

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path, State,
    },
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};

use crate::state::AppState;
use crate::error::Result;

/// Message types for WebSocket communication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EditorMessage {
    /// Client requests initial state
    RequestState {
        view_key: String,
    },
    /// Server sends full state
    State {
        view_key: String,
        elements: Vec<ElementPosition>,
        relationships: Vec<RelationshipPosition>,
    },
    /// Element position update
    ElementMoved {
        view_key: String,
        element_id: String,
        x: i32,
        y: i32,
    },
    /// Relationship vertex update
    RelationshipVertexMoved {
        view_key: String,
        relationship_id: String,
        vertices: Vec<Vertex>,
    },
    /// Selection changed
    SelectionChanged {
        view_key: String,
        element_ids: Vec<String>,
    },
    /// Undo/redo action
    Undo {
        view_key: String,
    },
    Redo {
        view_key: String,
    },
    /// Auto-layout request
    AutoLayout {
        view_key: String,
    },
    /// Save request
    Save,
    /// Workspace reloaded notification
    WorkspaceReloaded {
        timestamp: i64,
    },
    /// Error response
    Error {
        message: String,
    },
    /// Acknowledgment
    Ack {
        message_id: Option<String>,
    },
}

/// Element position in a view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementPosition {
    pub id: String,
    pub x: i32,
    pub y: i32,
}

/// Relationship vertex position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

/// Relationship position data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipPosition {
    pub id: String,
    pub vertices: Vec<Vertex>,
}

/// Request to update element position.
#[derive(Debug, Deserialize)]
pub struct UpdatePositionRequest {
    pub x: i32,
    pub y: i32,
}

/// Request to update multiple element positions.
#[derive(Debug, Deserialize)]
pub struct BatchUpdatePositionsRequest {
    pub positions: Vec<ElementPosition>,
}

/// Shared state for editor broadcasting.
#[derive(Clone)]
pub struct EditorState {
    /// Broadcast channel for editor updates
    pub broadcast: broadcast::Sender<EditorMessage>,
    /// Current element positions per view
    pub positions: Arc<RwLock<HashMap<String, HashMap<String, ElementPosition>>>>,
    /// Undo history per view
    pub undo_stack: Arc<RwLock<HashMap<String, Vec<EditorMessage>>>>,
    /// Redo history per view
    pub redo_stack: Arc<RwLock<HashMap<String, Vec<EditorMessage>>>>,
}

impl Default for EditorState {
    fn default() -> Self {
        Self::new()
    }
}

impl EditorState {
    pub fn new() -> Self {
        let (broadcast, _) = broadcast::channel(256);
        Self {
            broadcast,
            positions: Arc::new(RwLock::new(HashMap::new())),
            undo_stack: Arc::new(RwLock::new(HashMap::new())),
            redo_stack: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get element positions for a view.
    pub async fn get_positions(&self, view_key: &str) -> HashMap<String, ElementPosition> {
        let positions = self.positions.read().await;
        positions.get(view_key).cloned().unwrap_or_default()
    }

    /// Update element position.
    pub async fn update_position(&self, view_key: &str, element_id: &str, x: i32, y: i32) {
        let mut positions = self.positions.write().await;
        let view_positions = positions.entry(view_key.to_string()).or_default();

        // Save to undo stack
        if let Some(old_pos) = view_positions.get(element_id) {
            let mut undo = self.undo_stack.write().await;
            let view_undo = undo.entry(view_key.to_string()).or_default();
            view_undo.push(EditorMessage::ElementMoved {
                view_key: view_key.to_string(),
                element_id: element_id.to_string(),
                x: old_pos.x,
                y: old_pos.y,
            });

            // Clear redo stack on new action
            let mut redo = self.redo_stack.write().await;
            redo.remove(view_key);
        }

        view_positions.insert(element_id.to_string(), ElementPosition {
            id: element_id.to_string(),
            x,
            y,
        });
    }

    /// Broadcast a message to all connected clients.
    pub fn broadcast(&self, message: EditorMessage) {
        let _ = self.broadcast.send(message);
    }
}

/// WebSocket handler for editor.
pub async fn editor_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_editor_socket(socket, state, view_key))
}

/// Handle WebSocket connection.
async fn handle_editor_socket(mut socket: WebSocket, state: AppState, view_key: String) {
    info!("Editor WebSocket connected for view: {}", view_key);

    let editor = state.editor.clone();
    let mut receiver = editor.broadcast.subscribe();

    // Send initial state
    if let Some(_workspace) = state.get_workspace().await {
        let positions = editor.get_positions(&view_key).await;
        let elements: Vec<ElementPosition> = positions.values().cloned().collect();

        let initial = EditorMessage::State {
            view_key: view_key.clone(),
            elements,
            relationships: vec![],
        };

        if let Ok(json) = serde_json::to_string(&initial) {
            let _ = socket.send(Message::Text(json.into())).await;
        }
    }

    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(editor_msg) = serde_json::from_str::<EditorMessage>(&text) {
                            handle_editor_message(&state, &editor_msg).await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Editor WebSocket closed for view: {}", view_key);
                        break;
                    }
                    Err(e) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            // Broadcast updates to this client
            Ok(broadcast_msg) = receiver.recv() => {
                // Only send messages for this view
                let should_send = match &broadcast_msg {
                    EditorMessage::ElementMoved { view_key: vk, .. } => vk == &view_key,
                    EditorMessage::State { view_key: vk, .. } => vk == &view_key,
                    EditorMessage::SelectionChanged { view_key: vk, .. } => vk == &view_key,
                    _ => true,
                };

                if should_send {
                    if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Handle an editor message.
async fn handle_editor_message(state: &AppState, message: &EditorMessage) {
    let editor = &state.editor;

    match message {
        EditorMessage::ElementMoved { view_key, element_id, x, y } => {
            debug!("Element {} moved to ({}, {})", element_id, x, y);
            editor.update_position(view_key, element_id, *x, *y).await;

            // Update workspace
            if let Some(mut workspace) = state.get_workspace().await {
                update_element_position_in_workspace(&mut workspace, view_key, element_id, *x, *y);
                *state.workspace.write().await = Some(workspace);
            }

            // Broadcast to other clients
            editor.broadcast(message.clone());
        }
        EditorMessage::Save => {
            debug!("Save requested");
            if let Err(e) = state.save_workspace().await {
                warn!("Failed to save workspace: {}", e);
                editor.broadcast(EditorMessage::Error {
                    message: format!("Failed to save: {}", e),
                });
            } else {
                editor.broadcast(EditorMessage::Ack { message_id: None });
            }
        }
        EditorMessage::AutoLayout { view_key } => {
            debug!("Auto-layout requested for view: {}", view_key);
            if let Some(workspace) = state.get_workspace().await {
                if let Some(positions) = handle_auto_layout(&workspace, view_key).await {
                    // Update all positions in the editor state
                    for pos in &positions {
                        editor.update_position(view_key, &pos.id, pos.x, pos.y).await;
                    }

                    // Update workspace
                    if let Some(mut workspace_mut) = state.get_workspace().await {
                        for pos in &positions {
                            update_element_position_in_workspace(&mut workspace_mut, view_key, &pos.id, pos.x, pos.y);
                        }
                        *state.workspace.write().await = Some(workspace_mut);
                    }

                    // Broadcast all position updates
                    for pos in positions {
                        editor.broadcast(EditorMessage::ElementMoved {
                            view_key: view_key.to_string(),
                            element_id: pos.id,
                            x: pos.x,
                            y: pos.y,
                        });
                    }
                } else {
                    warn!("Failed to apply auto-layout for view: {}", view_key);
                    editor.broadcast(EditorMessage::Error {
                        message: format!("Failed to apply auto-layout for view: {}", view_key),
                    });
                }
            }
        }
        _ => {}
    }
}

/// Update element position in workspace views.
fn update_element_position_in_workspace(
    workspace: &mut structurizr_core::Workspace,
    view_key: &str,
    element_id: &str,
    x: i32,
    y: i32,
) {
    // Parse element ID
    let id = if let Ok(uuid) = element_id.parse::<uuid::Uuid>() {
        structurizr_core::ElementId::from(uuid)
    } else {
        return;
    };

    // Update in all view types
    for view in &mut workspace.views_mut().system_landscape_views {
        if view.properties.key == view_key {
            update_element_in_view(&mut view.properties.elements, id, x, y);
            return;
        }
    }

    for view in &mut workspace.views_mut().system_context_views {
        if view.properties.key == view_key {
            update_element_in_view(&mut view.properties.elements, id, x, y);
            return;
        }
    }

    for view in &mut workspace.views_mut().container_views {
        if view.properties.key == view_key {
            update_element_in_view(&mut view.properties.elements, id, x, y);
            return;
        }
    }

    for view in &mut workspace.views_mut().component_views {
        if view.properties.key == view_key {
            update_element_in_view(&mut view.properties.elements, id, x, y);
            return;
        }
    }
}

/// Update element position in view elements list.
fn update_element_in_view(
    elements: &mut Vec<structurizr_core::view::ElementView>,
    element_id: structurizr_core::ElementId,
    x: i32,
    y: i32,
) {
    if let Some(elem) = elements.iter_mut().find(|e| e.id == element_id) {
        elem.x = Some(x);
        elem.y = Some(y);
    }
}

/// REST API: Get element positions for a view.
pub async fn get_positions(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<Json<Vec<ElementPosition>>> {
    let positions = state.editor.get_positions(&view_key).await;
    Ok(Json(positions.values().cloned().collect()))
}

/// REST API: Update element position.
pub async fn update_position(
    State(state): State<AppState>,
    Path((view_key, element_id)): Path<(String, String)>,
    Json(req): Json<UpdatePositionRequest>,
) -> Result<Json<ElementPosition>> {
    state.editor.update_position(&view_key, &element_id, req.x, req.y).await;

    // Update workspace
    if let Some(mut workspace) = state.get_workspace().await {
        update_element_position_in_workspace(&mut workspace, &view_key, &element_id, req.x, req.y);
        *state.workspace.write().await = Some(workspace);
    }

    // Broadcast update
    state.editor.broadcast(EditorMessage::ElementMoved {
        view_key: view_key.clone(),
        element_id: element_id.clone(),
        x: req.x,
        y: req.y,
    });

    let position = ElementPosition {
        id: element_id,
        x: req.x,
        y: req.y,
    };

    Ok(Json(position))
}

/// REST API: Batch update element positions.
pub async fn batch_update_positions(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
    Json(req): Json<BatchUpdatePositionsRequest>,
) -> Result<Json<Vec<ElementPosition>>> {
    for pos in &req.positions {
        state.editor.update_position(&view_key, &pos.id, pos.x, pos.y).await;

        // Update workspace
        if let Some(mut workspace) = state.get_workspace().await {
            update_element_position_in_workspace(&mut workspace, &view_key, &pos.id, pos.x, pos.y);
            *state.workspace.write().await = Some(workspace);
        }

        state.editor.broadcast(EditorMessage::ElementMoved {
            view_key: view_key.clone(),
            element_id: pos.id.clone(),
            x: pos.x,
            y: pos.y,
        });
    }

    Ok(Json(req.positions))
}

/// REST API: Trigger save.
pub async fn save(State(state): State<AppState>) -> Result<&'static str> {
    state.save_workspace().await?;
    Ok("Saved")
}

/// Apply auto-layout to a view based on its auto-layout settings.
async fn handle_auto_layout(
    workspace: &structurizr_core::Workspace,
    view_key: &str,
) -> Option<Vec<ElementPosition>> {
    use structurizr_core::view::AutoLayoutDirection;

    // Find the view and get its properties
    let (elements, relationships, auto_layout) = get_view_data(workspace, view_key)?;

    // If no auto-layout config is set, use default (hierarchical top-bottom)
    let auto_layout = auto_layout.unwrap_or_default();

    // Get element IDs that need positioning
    let element_ids: Vec<String> = elements.iter().map(|e| e.id.to_string()).collect();

    if element_ids.is_empty() {
        return Some(vec![]);
    }

    // Build relationship map for hierarchical layout using the workspace model
    let relationship_map = build_relationship_map_from_model(
        workspace,
        &relationships,
        &element_ids,
    );

    // Apply layout algorithm based on direction
    let positions = match auto_layout.direction {
        AutoLayoutDirection::TopBottom => {
            apply_hierarchical_layout(
                &element_ids,
                &relationship_map,
                auto_layout.rank_separation,
                auto_layout.node_separation,
                true, // vertical
                false, // not reversed
            )
        }
        AutoLayoutDirection::BottomTop => {
            apply_hierarchical_layout(
                &element_ids,
                &relationship_map,
                auto_layout.rank_separation,
                auto_layout.node_separation,
                true, // vertical
                true,  // reversed
            )
        }
        AutoLayoutDirection::LeftRight => {
            apply_hierarchical_layout(
                &element_ids,
                &relationship_map,
                auto_layout.rank_separation,
                auto_layout.node_separation,
                false, // horizontal
                false, // not reversed
            )
        }
        AutoLayoutDirection::RightLeft => {
            apply_hierarchical_layout(
                &element_ids,
                &relationship_map,
                auto_layout.rank_separation,
                auto_layout.node_separation,
                false, // horizontal
                true,  // reversed
            )
        }
    };

    Some(positions)
}

/// Get view data (elements, relationships, auto-layout config) for a given view key.
fn get_view_data(
    workspace: &structurizr_core::Workspace,
    view_key: &str,
) -> Option<(
    Vec<structurizr_core::view::ElementView>,
    Vec<structurizr_core::view::RelationshipView>,
    Option<structurizr_core::view::AutoLayout>,
)> {
    let views = workspace.views();

    // Check system landscape views
    for view in &views.system_landscape_views {
        if view.properties.key == view_key {
            return Some((
                view.properties.elements.clone(),
                view.properties.relationships.clone(),
                view.properties.auto_layout.clone(),
            ));
        }
    }

    // Check system context views
    for view in &views.system_context_views {
        if view.properties.key == view_key {
            return Some((
                view.properties.elements.clone(),
                view.properties.relationships.clone(),
                view.properties.auto_layout.clone(),
            ));
        }
    }

    // Check container views
    for view in &views.container_views {
        if view.properties.key == view_key {
            return Some((
                view.properties.elements.clone(),
                view.properties.relationships.clone(),
                view.properties.auto_layout.clone(),
            ));
        }
    }

    // Check component views
    for view in &views.component_views {
        if view.properties.key == view_key {
            return Some((
                view.properties.elements.clone(),
                view.properties.relationships.clone(),
                view.properties.auto_layout.clone(),
            ));
        }
    }

    None
}

/// Build a map of relationships from the workspace model.
///
/// Maps source element ID -> list of destination element IDs.
fn build_relationship_map_from_model(
    workspace: &structurizr_core::Workspace,
    view_relationships: &[structurizr_core::view::RelationshipView],
    element_ids: &[String],
) -> HashMap<String, Vec<String>> {
    let mut map: HashMap<String, Vec<String>> = HashMap::new();
    let element_set: std::collections::HashSet<String> = element_ids.iter().cloned().collect();

    // Get relationships from the model
    let model_relationships = &workspace.model.relationships;

    // Create a set of relationship IDs that are in the view
    let view_rel_ids: std::collections::HashSet<String> = view_relationships
        .iter()
        .map(|r| r.id.to_string())
        .collect();

    // Iterate through model relationships and add them to the map if they're in the view
    for rel in model_relationships {
        let rel_id = rel.id.to_string();

        // Only include relationships that are in the view
        if view_rel_ids.contains(&rel_id) {
            let source_id = rel.source_id.to_string();
            let dest_id = rel.destination_id.to_string();

            // Only include if both source and destination are in the element set
            if element_set.contains(&source_id) && element_set.contains(&dest_id) {
                map.entry(source_id).or_default().push(dest_id);
            }
        }
    }

    map
}

/// Apply hierarchical layout algorithm.
///
/// This implements a layered graph layout similar to Sugiyama's algorithm:
/// 1. Assign nodes to layers based on their depth in the dependency graph
/// 2. Position nodes within each layer with even spacing
/// 3. Support different directions (TB, BT, LR, RL)
fn apply_hierarchical_layout(
    element_ids: &[String],
    relationship_map: &HashMap<String, Vec<String>>,
    rank_separation: u32,
    node_separation: u32,
    vertical: bool,
    reversed: bool,
) -> Vec<ElementPosition> {
    if element_ids.is_empty() {
        return vec![];
    }

    // Build adjacency lists for the graph
    let mut outgoing: HashMap<String, Vec<String>> = HashMap::new();
    let mut incoming: HashMap<String, Vec<String>> = HashMap::new();

    for id in element_ids {
        outgoing.insert(id.clone(), vec![]);
        incoming.insert(id.clone(), vec![]);
    }

    // Populate adjacency lists from relationships
    for (from, tos) in relationship_map {
        for to in tos {
            if outgoing.contains_key(from) && outgoing.contains_key(to) {
                outgoing.get_mut(from).unwrap().push(to.clone());
                incoming.get_mut(to).unwrap().push(from.clone());
            }
        }
    }

    // Compute layers using topological sort (Kahn's algorithm)
    let mut layers: Vec<Vec<String>> = Vec::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Initialize in-degrees
    for id in element_ids {
        in_degree.insert(id.clone(), incoming.get(id).map(|v| v.len()).unwrap_or(0));
    }

    // Process nodes layer by layer
    loop {
        // Find nodes with in-degree 0 that haven't been processed
        let current_layer: Vec<String> = element_ids
            .iter()
            .filter(|id| !processed.contains(*id) && in_degree.get(*id).copied().unwrap_or(0) == 0)
            .cloned()
            .collect();

        if current_layer.is_empty() {
            break;
        }

        // Mark nodes as processed
        for id in &current_layer {
            processed.insert(id.clone());
            // Decrease in-degree of neighbors
            if let Some(neighbors) = outgoing.get(id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree = degree.saturating_sub(1);
                    }
                }
            }
        }

        layers.push(current_layer);
    }

    // Add any remaining nodes (cycles or disconnected components) to the last layer
    let remaining: Vec<String> = element_ids
        .iter()
        .filter(|id| !processed.contains(*id))
        .cloned()
        .collect();

    if !remaining.is_empty() {
        layers.push(remaining);
    }

    // If no layers were created (all nodes were in cycles), create a single layer
    if layers.is_empty() {
        layers.push(element_ids.to_vec());
    }

    // Position elements
    let mut positions = Vec::new();
    let start_offset = 100i32;

    for (layer_idx, layer) in layers.iter().enumerate() {
        let layer_pos = start_offset + (layer_idx as i32 * rank_separation as i32);

        for (node_idx, element_id) in layer.iter().enumerate() {
            let node_pos = start_offset + (node_idx as i32 * node_separation as i32);

            let (x, y) = if vertical {
                if reversed {
                    // Bottom to top: reverse layer index
                    let reversed_layer = (layers.len() - 1 - layer_idx) as i32 * rank_separation as i32;
                    (node_pos, start_offset + reversed_layer)
                } else {
                    // Top to bottom
                    (node_pos, layer_pos)
                }
            } else {
                if reversed {
                    // Right to left: reverse layer index
                    let reversed_layer = (layers.len() - 1 - layer_idx) as i32 * rank_separation as i32;
                    (start_offset + reversed_layer, node_pos)
                } else {
                    // Left to right
                    (layer_pos, node_pos)
                }
            };

            positions.push(ElementPosition {
                id: element_id.clone(),
                x,
                y,
            });
        }
    }

    positions
}

/// Apply grid layout algorithm.
///
/// This arranges elements in a simple grid pattern.
#[allow(dead_code)]
fn apply_grid_layout(
    element_ids: &[String],
    node_separation: u32,
) -> Vec<ElementPosition> {
    if element_ids.is_empty() {
        return vec![];
    }

    let mut positions = Vec::new();
    let start_offset = 100i32;

    // Calculate grid dimensions (roughly square)
    let total = element_ids.len();
    let cols = (total as f64).sqrt().ceil() as usize;

    for (idx, element_id) in element_ids.iter().enumerate() {
        let row = idx / cols;
        let col = idx % cols;

        let x = start_offset + (col as i32 * node_separation as i32);
        let y = start_offset + (row as i32 * node_separation as i32);

        positions.push(ElementPosition {
            id: element_id.clone(),
            x,
            y,
        });
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_hierarchical_layout_simple() {
        let element_ids = vec![
            "elem1".to_string(),
            "elem2".to_string(),
            "elem3".to_string(),
        ];

        let mut relationship_map = HashMap::new();
        // elem1 -> elem2 -> elem3 (linear chain)
        relationship_map.insert("elem1".to_string(), vec!["elem2".to_string()]);
        relationship_map.insert("elem2".to_string(), vec!["elem3".to_string()]);

        let positions = apply_hierarchical_layout(
            &element_ids,
            &relationship_map,
            300,
            300,
            true,  // vertical
            false, // not reversed
        );

        assert_eq!(positions.len(), 3);

        // Should be arranged in layers top to bottom
        // elem1 should be at the top (smallest y)
        // elem3 should be at the bottom (largest y)
        let elem1_pos = positions.iter().find(|p| p.id == "elem1").unwrap();
        let elem2_pos = positions.iter().find(|p| p.id == "elem2").unwrap();
        let elem3_pos = positions.iter().find(|p| p.id == "elem3").unwrap();

        assert!(elem1_pos.y < elem2_pos.y);
        assert!(elem2_pos.y < elem3_pos.y);
    }

    #[test]
    fn test_hierarchical_layout_no_relationships() {
        let element_ids = vec![
            "elem1".to_string(),
            "elem2".to_string(),
        ];

        let relationship_map = HashMap::new();

        let positions = apply_hierarchical_layout(
            &element_ids,
            &relationship_map,
            300,
            300,
            true,
            false,
        );

        assert_eq!(positions.len(), 2);
        // All elements should be in the same layer
        let elem1_pos = positions.iter().find(|p| p.id == "elem1").unwrap();
        let elem2_pos = positions.iter().find(|p| p.id == "elem2").unwrap();
        assert_eq!(elem1_pos.y, elem2_pos.y);
    }

    #[test]
    fn test_grid_layout() {
        let element_ids = vec![
            "elem1".to_string(),
            "elem2".to_string(),
            "elem3".to_string(),
            "elem4".to_string(),
        ];

        let positions = apply_grid_layout(&element_ids, 300);

        assert_eq!(positions.len(), 4);

        // Grid should be 2x2 for 4 elements
        let unique_x: std::collections::HashSet<_> = positions.iter().map(|p| p.x).collect();
        let unique_y: std::collections::HashSet<_> = positions.iter().map(|p| p.y).collect();

        assert_eq!(unique_x.len(), 2);
        assert_eq!(unique_y.len(), 2);
    }

    #[test]
    fn test_hierarchical_layout_reversed_vertical() {
        let element_ids = vec![
            "elem1".to_string(),
            "elem2".to_string(),
        ];

        let mut relationship_map = HashMap::new();
        relationship_map.insert("elem1".to_string(), vec!["elem2".to_string()]);

        let positions = apply_hierarchical_layout(
            &element_ids,
            &relationship_map,
            300,
            300,
            true, // vertical
            true, // reversed
        );

        assert_eq!(positions.len(), 2);

        // elem1 should be at the bottom (larger y) when reversed
        let elem1_pos = positions.iter().find(|p| p.id == "elem1").unwrap();
        let elem2_pos = positions.iter().find(|p| p.id == "elem2").unwrap();

        assert!(elem1_pos.y > elem2_pos.y);
    }

    #[test]
    fn test_hierarchical_layout_horizontal() {
        let element_ids = vec![
            "elem1".to_string(),
            "elem2".to_string(),
        ];

        let mut relationship_map = HashMap::new();
        relationship_map.insert("elem1".to_string(), vec!["elem2".to_string()]);

        let positions = apply_hierarchical_layout(
            &element_ids,
            &relationship_map,
            300,
            300,
            false, // horizontal
            false, // not reversed
        );

        assert_eq!(positions.len(), 2);

        // elem1 should be to the left (smaller x)
        let elem1_pos = positions.iter().find(|p| p.id == "elem1").unwrap();
        let elem2_pos = positions.iter().find(|p| p.id == "elem2").unwrap();

        assert!(elem1_pos.x < elem2_pos.x);
    }
}

// ============================================================================
// Multi-Workspace Mode Editor Handlers
// ============================================================================

/// Helper function to extract workspace_id from wildcard path.
fn extract_workspace_id(path: &str) -> String {
    path.trim_start_matches('/').to_string()
}

/// Workspace-scoped: Get element positions for a view.
pub async fn workspace_get_positions(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> Result<Json<Vec<ElementPosition>>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let _workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| crate::error::Error::WorkspaceNotFound(workspace_id.clone()))?;

    let positions = state.editor.get_positions(&view_key).await;
    Ok(Json(positions.values().cloned().collect()))
}

/// Workspace-scoped: Batch update element positions.
pub async fn workspace_batch_update_positions(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
    Json(req): Json<BatchUpdatePositionsRequest>,
) -> Result<Json<Vec<ElementPosition>>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let _workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| crate::error::Error::WorkspaceNotFound(workspace_id.clone()))?;

    for pos in &req.positions {
        state.editor.update_position(&view_key, &pos.id, pos.x, pos.y).await;

        state.editor.broadcast(EditorMessage::ElementMoved {
            view_key: view_key.clone(),
            element_id: pos.id.clone(),
            x: pos.x,
            y: pos.y,
        });
    }

    Ok(Json(req.positions))
}

/// Nested workspace: Get element positions.
pub async fn workspace_get_positions_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<Json<Vec<ElementPosition>>> {
    let full_id = format!("{}/{}", category, workspace_id);
    let _workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| crate::error::Error::WorkspaceNotFound(full_id.clone()))?;
    let positions = state.editor.get_positions(&view_key).await;
    Ok(Json(positions.values().cloned().collect()))
}

/// Nested workspace: Batch update positions.
pub async fn workspace_batch_update_positions_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Json(req): Json<BatchUpdatePositionsRequest>,
) -> Result<Json<Vec<ElementPosition>>> {
    let full_id = format!("{}/{}", category, workspace_id);
    let _workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| crate::error::Error::WorkspaceNotFound(full_id.clone()))?;
    for pos in &req.positions {
        state.editor.update_position(&view_key, &pos.id, pos.x, pos.y).await;
        state.editor.broadcast(EditorMessage::ElementMoved {
            view_key: view_key.clone(),
            element_id: pos.id.clone(),
            x: pos.x,
            y: pos.y,
        });
    }
    Ok(Json(req.positions))
}

// ============================================================================
// Multi-Workspace WebSocket Handlers
// ============================================================================

/// WebSocket handler for multi-workspace editor (single-level: /w/:workspace_id/ws/edit/:key).
pub async fn workspace_editor_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_workspace_editor_socket(socket, state, workspace_id, view_key))
}

/// WebSocket handler for multi-workspace editor (nested: /w/:category/:workspace_id/ws/edit/:key).
pub async fn workspace_editor_ws_nested(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let full_id = format!("{}/{}", category, workspace_id);
    ws.on_upgrade(move |socket| handle_workspace_editor_socket(socket, state, full_id, view_key))
}

/// Handle WebSocket connection for multi-workspace mode.
async fn handle_workspace_editor_socket(
    mut socket: WebSocket,
    state: AppState,
    workspace_id: String,
    view_key: String,
) {
    info!("Editor WebSocket connected for workspace: {}, view: {}", workspace_id, view_key);

    let editor = state.editor.clone();
    let mut receiver = editor.broadcast.subscribe();

    // Send initial state
    if let Some(_workspace) = state.get_workspace_by_id(&workspace_id).await {
        let positions = editor.get_positions(&view_key).await;
        let elements: Vec<ElementPosition> = positions.values().cloned().collect();

        let initial = EditorMessage::State {
            view_key: view_key.clone(),
            elements,
            relationships: vec![],
        };

        if let Ok(json) = serde_json::to_string(&initial) {
            let _ = socket.send(Message::Text(json.into())).await;
        }
    }

    loop {
        tokio::select! {
            // Handle incoming messages from client
            Some(msg) = socket.recv() => {
                match msg {
                    Ok(Message::Text(text)) => {
                        if let Ok(editor_msg) = serde_json::from_str::<EditorMessage>(&text) {
                            handle_workspace_editor_message(&state, &workspace_id, &editor_msg).await;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("Editor WebSocket closed for workspace: {}, view: {}", workspace_id, view_key);
                        break;
                    }
                    Err(e) => {
                        warn!("WebSocket error: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
            // Broadcast updates to this client
            Ok(broadcast_msg) = receiver.recv() => {
                // Only send messages for this view
                let should_send = match &broadcast_msg {
                    EditorMessage::ElementMoved { view_key: vk, .. } => vk == &view_key,
                    EditorMessage::State { view_key: vk, .. } => vk == &view_key,
                    EditorMessage::SelectionChanged { view_key: vk, .. } => vk == &view_key,
                    _ => true,
                };

                if should_send {
                    if let Ok(json) = serde_json::to_string(&broadcast_msg) {
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// Handle an editor message for multi-workspace mode.
async fn handle_workspace_editor_message(state: &AppState, workspace_id: &str, message: &EditorMessage) {
    let editor = &state.editor;

    match message {
        EditorMessage::ElementMoved { view_key, element_id, x, y } => {
            debug!("Element {} moved to ({}, {}) in workspace {}", element_id, x, y, workspace_id);
            editor.update_position(view_key, element_id, *x, *y).await;

            // Update workspace
            if let Some(mut workspace) = state.get_workspace_by_id(workspace_id).await {
                update_element_position_in_workspace(&mut workspace, view_key, element_id, *x, *y);
                // Note: Multi-workspace mode doesn't persist changes back to registry
                // This would require additional registry support
            }

            // Broadcast to other clients
            editor.broadcast(message.clone());
        }
        EditorMessage::Save => {
            debug!("Save requested for workspace {}", workspace_id);
            // Save not yet implemented for multi-workspace mode
            // Would need to update the workspace file in the discovered directory
        }
        EditorMessage::AutoLayout { view_key } => {
            debug!("Auto-layout requested for view {} in workspace {}", view_key, workspace_id);
            if let Some(workspace) = state.get_workspace_by_id(workspace_id).await {
                if let Some(positions) = handle_auto_layout(&workspace, view_key).await {
                    // Update all positions in the editor state
                    for pos in &positions {
                        editor.update_position(view_key, &pos.id, pos.x, pos.y).await;
                    }
                    // Broadcast the layout update
                    editor.broadcast(EditorMessage::State {
                        view_key: view_key.clone(),
                        elements: positions,
                        relationships: vec![],
                    });
                }
            }
        }
        EditorMessage::Undo { view_key } => {
            debug!("Undo requested for view {} in workspace {}", view_key, workspace_id);
            // Undo not yet implemented for multi-workspace mode
        }
        EditorMessage::Redo { view_key } => {
            debug!("Redo requested for view {} in workspace {}", view_key, workspace_id);
            // Redo not yet implemented for multi-workspace mode
        }
        _ => {}
    }
}
