//! File watching functionality for auto-reloading workspace files.

use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::editor::EditorMessage;
use crate::state::AppState;

/// Debounce duration to avoid reloading too frequently.
const DEBOUNCE_DURATION: Duration = Duration::from_secs(1);

/// File watcher for workspace files.
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    last_reload: Arc<RwLock<Instant>>,
    /// Last reload time per workspace (for multi-workspace mode).
    last_reload_per_workspace: Arc<RwLock<HashMap<String, Instant>>>,
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new() -> Self {
        Self {
            watcher: None,
            last_reload: Arc::new(RwLock::new(Instant::now())),
            last_reload_per_workspace: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Start watching the workspace file (single-workspace mode).
    pub fn start(&mut self, workspace_path: PathBuf, state: AppState) -> crate::Result<()> {
        let last_reload = self.last_reload.clone();

        // Get a handle to the current Tokio runtime so we can spawn tasks from the notify thread
        let runtime_handle = tokio::runtime::Handle::current();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Only process modify events
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        // Check if event is for a .dsl or .json file
                        let is_workspace_file = event.paths.iter().any(|p| {
                            p.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e == "dsl" || e == "json")
                                .unwrap_or(false)
                        });

                        if is_workspace_file {
                            debug!("File change detected: {:?}", event.paths);

                            // Spawn reload task using the runtime handle (not tokio::spawn)
                            // because we're in the notify thread, not the Tokio runtime
                            let state_clone = state.clone();
                            let last_reload_clone = last_reload.clone();
                            runtime_handle.spawn(async move {
                                handle_file_change(state_clone, last_reload_clone).await;
                            });
                        }
                    }
                }
                Err(e) => {
                    error!("File watcher error: {}", e);
                }
            }
        })?;

        // Watch the directory containing the workspace file
        let watch_dir = if workspace_path.is_dir() {
            workspace_path.clone()
        } else {
            workspace_path
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| PathBuf::from("."))
        };

        info!("Starting file watcher for: {:?}", watch_dir);
        watcher.watch(&watch_dir, RecursiveMode::NonRecursive)?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Start watching the workspaces directory (multi-workspace mode).
    pub fn start_multi(&mut self, workspaces_dir: PathBuf, state: AppState) -> crate::Result<()> {
        let last_reload_per_workspace = self.last_reload_per_workspace.clone();
        let workspaces_root = workspaces_dir.clone();

        // Get a handle to the current Tokio runtime
        let runtime_handle = tokio::runtime::Handle::current();

        // Create the watcher
        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    // Only process modify events
                    if matches!(
                        event.kind,
                        EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                    ) {
                        // Check if event is for a .dsl, .json, or .md file
                        let relevant_paths: Vec<_> = event.paths.iter().filter(|p| {
                            p.extension()
                                .and_then(|e| e.to_str())
                                .map(|e| e == "dsl" || e == "json" || e == "md")
                                .unwrap_or(false)
                        }).cloned().collect();

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

/// Handle a file change event with debouncing (single-workspace mode).
async fn handle_file_change(state: AppState, last_reload: Arc<RwLock<Instant>>) {
    // Debounce: check if we recently reloaded
    {
        let last = last_reload.read().await;
        if last.elapsed() < DEBOUNCE_DURATION {
            debug!("Debouncing file change (last reload was {:?} ago)", last.elapsed());
            return;
        }
    }

    // Update last reload time
    *last_reload.write().await = Instant::now();

    // Reload the workspace
    info!("Reloading workspace due to file change");
    match state.load_workspace().await {
        Ok(()) => {
            info!("Workspace reloaded successfully");

            // Notify connected WebSocket clients to refresh
            state.editor.broadcast(EditorMessage::WorkspaceReloaded {
                timestamp: chrono::Utc::now().timestamp(),
            });
        }
        Err(e) => {
            warn!("Failed to reload workspace: {}", e);
            state.editor.broadcast(EditorMessage::Error {
                message: format!("Failed to reload workspace: {}", e),
            });
        }
    }
}

/// Handle a file change event in multi-workspace mode.
///
/// This function determines which workspace was affected by the file change
/// and invalidates only that workspace's cache.
async fn handle_multi_workspace_change(
    state: AppState,
    last_reload_per_workspace: Arc<RwLock<HashMap<String, Instant>>>,
    workspaces_root: PathBuf,
    changed_paths: Vec<PathBuf>,
) {
    // Determine which workspaces were affected
    let mut affected_workspaces = std::collections::HashSet::new();

    for path in &changed_paths {
        // Find the workspace ID by looking at the path relative to workspaces root
        if let Ok(relative) = path.strip_prefix(&workspaces_root) {
            // The workspace ID is the directory path up to (but not including) the file
            // For example: small/startup-saas/workspace.dsl -> small/startup-saas
            if let Some(parent) = relative.parent() {
                // Find the workspace directory (directory containing workspace.dsl)
                let mut current = workspaces_root.join(parent);
                while current.starts_with(&workspaces_root) && current != workspaces_root {
                    let dsl_path = current.join("workspace.dsl");
                    if dsl_path.exists() {
                        if let Ok(ws_relative) = current.strip_prefix(&workspaces_root) {
                            let workspace_id = ws_relative.to_string_lossy().replace('\\', "/");
                            affected_workspaces.insert(workspace_id);
                            break;
                        }
                    }
                    current = current.parent().map(|p| p.to_path_buf()).unwrap_or(workspaces_root.clone());
                }
            }
        }
    }

    // Process each affected workspace
    for workspace_id in affected_workspaces {
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

        // Invalidate the workspace cache
        info!("Invalidating workspace cache for: {}", workspace_id);
        state.invalidate_workspace(&workspace_id).await;

        // Notify connected WebSocket clients (broadcast to all for now)
        // In the future, we could scope this to clients viewing this specific workspace
        state.editor.broadcast(EditorMessage::WorkspaceReloaded {
            timestamp: chrono::Utc::now().timestamp(),
        });
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
