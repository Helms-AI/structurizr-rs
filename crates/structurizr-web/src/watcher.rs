//! File watching functionality for auto-reloading workspace files.

use notify::{
    Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher as NotifyWatcher,
};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
}

impl FileWatcher {
    /// Create a new file watcher.
    pub fn new() -> Self {
        Self {
            watcher: None,
            last_reload: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Start watching the workspace file.
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

/// Handle a file change event with debouncing.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debounce_duration() {
        // Ensure debounce is at least 500ms
        assert!(DEBOUNCE_DURATION.as_millis() >= 500);
    }
}
