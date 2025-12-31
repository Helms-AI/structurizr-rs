//! File watching functionality for auto-reloading workspace files.
//!
//! This module provides file system monitoring for workspace directories,
//! detecting changes to DSL files, position files, documentation, and more.
//!
//! ## Change Types
//!
//! The watcher distinguishes between different types of changes:
//! - **DSL changes**: Updates to workspace.dsl files
//! - **Position changes**: Updates to .positions.json files
//! - **Documentation changes**: Updates to markdown files in docs/ directories
//! - **Structural changes**: New or deleted workspaces

use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::editor::EditorMessage;
use crate::state::AppState;

/// Type of change detected in a workspace.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChangeType {
    /// DSL file changed (workspace.dsl)
    Dsl,
    /// Position file changed (.positions.json)
    Positions,
    /// Documentation file changed (*.md in docs/)
    Documentation,
    /// Notes file changed (.notes.json)
    Notes,
}

/// Debounce duration to avoid reloading too frequently.
const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

/// File watcher for workspace files.
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    /// Last reload time per workspace.
    last_reload_per_workspace: Arc<RwLock<HashMap<String, Instant>>>,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new() -> Self {
        Self {
            watcher: None,
            last_reload_per_workspace: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching the workspaces directory.
    pub fn start_multi(&mut self, workspaces_dir: PathBuf, state: AppState) -> crate::Result<()> {
        let last_reload_per_workspace = self.last_reload_per_workspace.clone();
        // Canonicalize the path to match the absolute paths from notify
        let workspaces_root = workspaces_dir.canonicalize()
            .unwrap_or_else(|_| workspaces_dir.clone());

        // Get a handle to the current Tokio runtime
        let runtime_handle = tokio::runtime::Handle::current();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Process create, modify, and remove events for all files in workspaces
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        // Watch all files in workspace directories - the handler will
                        // categorize changes and decide what action to take
                        let relevant_paths: Vec<_> = event.paths.to_vec();

                        if !relevant_paths.is_empty() {
                            debug!("File change detected in multi-workspace mode: {:?}", relevant_paths);

                            // Spawn reload task
                            let state_clone = state.clone();
                            let last_reload_clone = last_reload_per_workspace.clone();
                            let root_clone = workspaces_root.clone();
                            runtime_handle.spawn(async move {
                                handle_multi_workspace_change(
                                    state_clone,
                                    last_reload_clone,
                                    root_clone,
                                    relevant_paths,
                                ).await;
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        })?;

        info!("Starting multi-workspace file watcher for: {:?}", workspaces_dir);
        watcher.watch(&workspaces_dir, RecursiveMode::Recursive)?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Stop the file watcher.
    pub fn stop(&mut self) {
        if let Some(_watcher) = self.watcher.take() {
            // Watcher is automatically stopped when dropped
            info!("Stopping file watcher");
        }
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Categorize a file change based on the file path.
fn categorize_change(path: &PathBuf) -> Option<ChangeType> {
    let filename = path.file_name().and_then(OsStr::to_str)?;

    // Check for specific file types
    if filename == "workspace.dsl" {
        return Some(ChangeType::Dsl);
    }
    if filename == ".positions.json" {
        return Some(ChangeType::Positions);
    }
    if filename == ".notes.json" {
        return Some(ChangeType::Notes);
    }

    // Check extension for documentation
    if let Some(ext) = path.extension().and_then(OsStr::to_str) {
        if ext == "md" {
            // Check if it's in a docs or adrs directory
            let path_str = path.to_string_lossy();
            if path_str.contains("/docs/") || path_str.contains("/adrs/") ||
               path_str.contains("\\docs\\") || path_str.contains("\\adrs\\") {
                return Some(ChangeType::Documentation);
            }
            // Generic markdown file in workspace
            return Some(ChangeType::Documentation);
        }
    }

    None
}

/// Handle a file change event.
///
/// This function determines which workspace was affected by the file change
/// and invalidates only that workspace's cache. It also broadcasts granular
/// change notifications to connected WebSocket clients.
async fn handle_multi_workspace_change(
    state: AppState,
    last_reload_per_workspace: Arc<RwLock<HashMap<String, Instant>>>,
    workspaces_root: PathBuf,
    changed_paths: Vec<PathBuf>,
) {
    // Canonicalize changed paths to match the canonicalized workspaces_root
    // This ensures consistent path comparison across platforms
    let changed_paths: Vec<PathBuf> = changed_paths
        .into_iter()
        .map(|p| p.canonicalize().unwrap_or(p))
        .collect();

    // Get current list of known workspaces for detecting new/deleted workspaces
    let known_workspaces: HashSet<String> = state
        .list_workspaces()
        .await
        .into_iter()
        .map(|w| w.id)
        .collect();

    // Track workspace changes with their change types
    let mut workspace_changes: HashMap<String, HashSet<ChangeType>> = HashMap::new();
    let mut potential_new_workspaces: HashSet<PathBuf> = HashSet::new();
    let mut potential_deleted_workspaces: HashSet<String> = HashSet::new();

    for path in &changed_paths {
        let filename = path.file_name().and_then(OsStr::to_str);

        // Check for new/deleted workspace.dsl files
        if filename == Some("workspace.dsl") {
            if path.exists() {
                // Workspace might be new - canonicalize parent for consistent comparison
                if let Some(parent) = path.parent() {
                    let canonical_parent = parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf());
                    potential_new_workspaces.insert(canonical_parent);
                }
            } else {
                // Workspace might be deleted - try to derive ID from path
                // Since the file is deleted, we can't canonicalize it, so try multiple approaches
                if let Some(parent) = path.parent() {
                    // Try canonicalizing parent first (might work if directory still exists)
                    let workspace_id = parent.canonicalize()
                        .ok()
                        .and_then(|p| p.strip_prefix(&workspaces_root).ok().map(|r| r.to_path_buf()))
                        .or_else(|| parent.strip_prefix(&workspaces_root).ok().map(|r| r.to_path_buf()))
                        .map(|r| r.to_string_lossy().replace('\\', "/"));

                    if let Some(id) = workspace_id {
                        if known_workspaces.contains(&id) {
                            potential_deleted_workspaces.insert(id);
                        }
                    }
                }
            }
        }

        // Check for directory renames (path is a directory, not workspace.dsl)
        // This handles cases where the notify event is for the directory itself
        if path.extension().is_none() {
            let dsl_path = path.join("workspace.dsl");
            if dsl_path.exists() {
                // Directory exists and contains workspace.dsl - this is a new/renamed workspace
                let canonical_path = path.canonicalize().unwrap_or_else(|_| path.clone());
                potential_new_workspaces.insert(canonical_path);
            } else if !path.exists() {
                // Directory was deleted/renamed away - check if it was a known workspace
                let workspace_id = path.strip_prefix(&workspaces_root)
                    .ok()
                    .map(|r| r.to_string_lossy().replace('\\', "/"));

                if let Some(id) = workspace_id {
                    if known_workspaces.contains(&id) {
                        potential_deleted_workspaces.insert(id);
                    }
                }
            }
        }

        // Find the workspace ID by looking at the path relative to workspaces root
        // Path is already canonicalized at this point
        if let Ok(relative) = path.strip_prefix(&workspaces_root) {
            // The workspace ID is the directory path up to (but not including) the file
            if let Some(parent) = relative.parent() {
                // Find the workspace directory (directory containing workspace.dsl)
                let mut current = workspaces_root.join(parent);
                while current.starts_with(&workspaces_root) && current != workspaces_root {
                    let dsl_path = current.join("workspace.dsl");
                    if dsl_path.exists() {
                        if let Ok(ws_relative) = current.strip_prefix(&workspaces_root) {
                            let workspace_id = ws_relative.to_string_lossy().replace('\\', "/");

                            // Categorize the change
                            if let Some(change_type) = categorize_change(path) {
                                workspace_changes
                                    .entry(workspace_id)
                                    .or_default()
                                    .insert(change_type);
                            }
                            break;
                        }
                    }
                    current = current.parent().map(|p| p.to_path_buf()).unwrap_or(workspaces_root.clone());
                }
            }
        }
    }

    // Handle potential new workspaces
    let mut structural_change = false;
    for ws_path in &potential_new_workspaces {
        if let Ok(relative) = ws_path.strip_prefix(&workspaces_root) {
            let workspace_id = relative.to_string_lossy().replace('\\', "/");
            if !known_workspaces.contains(&workspace_id) {
                info!("New workspace detected: {}", workspace_id);
                structural_change = true;
            }
        }
    }

    // Handle deleted workspaces
    for workspace_id in &potential_deleted_workspaces {
        info!("Deleted workspace detected: {}", workspace_id);
        structural_change = true;
    }

    // If structural changes occurred, rediscover workspaces
    if structural_change {
        info!("Structural workspace changes detected, rediscovering workspaces...");
        if let Err(e) = state.rediscover_workspaces().await {
            warn!("Failed to rediscover workspaces: {}", e);
        }

        // Broadcast workspace list update
        state.editor.broadcast(EditorMessage::WorkspaceListUpdated {
            timestamp: chrono::Utc::now().timestamp(),
        });

        // Broadcast individual workspace added/deleted events
        for ws_path in &potential_new_workspaces {
            if let Ok(relative) = ws_path.strip_prefix(&workspaces_root) {
                let workspace_id = relative.to_string_lossy().replace('\\', "/");
                if !known_workspaces.contains(&workspace_id) {
                    state.editor.broadcast(EditorMessage::WorkspaceAdded {
                        workspace_id,
                        timestamp: chrono::Utc::now().timestamp(),
                    });
                }
            }
        }

        for workspace_id in &potential_deleted_workspaces {
            state.editor.broadcast(EditorMessage::WorkspaceDeleted {
                workspace_id: workspace_id.clone(),
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
    }

    // Process each affected workspace with its specific changes
    for (workspace_id, changes) in workspace_changes {
        // Debounce per workspace
        {
            let last_reloads = last_reload_per_workspace.read().await;
            if let Some(last) = last_reloads.get(&workspace_id) {
                if last.elapsed() < DEBOUNCE_DURATION {
                    debug!("Debouncing workspace {} (last reload was {:?} ago)", workspace_id, last.elapsed());
                    continue;
                }
            }
        }

        // Update last reload time for this workspace
        {
            let mut last_reloads = last_reload_per_workspace.write().await;
            last_reloads.insert(workspace_id.clone(), Instant::now());
        }

        // Invalidate the workspace cache (except for position-only changes)
        let needs_cache_invalidation = changes.contains(&ChangeType::Dsl)
            || changes.contains(&ChangeType::Documentation);

        if needs_cache_invalidation {
            info!("Invalidating workspace cache for: {}", workspace_id);
            state.invalidate_workspace(&workspace_id).await;
        }

        // Broadcast granular change notifications
        let timestamp = chrono::Utc::now().timestamp();

        for change in &changes {
            match change {
                ChangeType::Dsl => {
                    info!("DSL changed for workspace: {}", workspace_id);
                    state.editor.broadcast(EditorMessage::DiagramUpdated {
                        workspace_id: workspace_id.clone(),
                        view_key: None,
                        timestamp,
                    });
                }
                ChangeType::Positions => {
                    debug!("Positions changed for workspace: {}", workspace_id);
                    state.editor.broadcast(EditorMessage::PositionsUpdated {
                        workspace_id: workspace_id.clone(),
                        view_key: "all".to_string(), // Position file affects all views
                        timestamp,
                    });
                }
                ChangeType::Documentation => {
                    info!("Documentation changed for workspace: {}", workspace_id);
                    state.editor.broadcast(EditorMessage::DocumentationUpdated {
                        workspace_id: workspace_id.clone(),
                        timestamp,
                    });
                }
                ChangeType::Notes => {
                    debug!("Notes changed for workspace: {}", workspace_id);
                    // Notes changes are less critical, just send a generic reload
                    state.editor.broadcast(EditorMessage::WorkspaceReloaded {
                        timestamp,
                    });
                }
            }
        }

        // Also send a general WorkspaceReloaded for backward compatibility
        // if there were any significant changes
        if changes.contains(&ChangeType::Dsl) {
            state.editor.broadcast(EditorMessage::WorkspaceReloaded {
                timestamp,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounce_duration() {
        // Ensure debounce is at least 500ms
        assert!(DEBOUNCE_DURATION.as_millis() >= 500);
    }
}
