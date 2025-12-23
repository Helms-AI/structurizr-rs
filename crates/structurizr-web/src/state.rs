//! Application state for the web server.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use structurizr_core::Workspace;

use crate::editor::EditorState;
use crate::watcher::FileWatcher;

/// Configuration for the web server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory containing workspace files.
    pub data_dir: PathBuf,
    /// Port to listen on.
    pub port: u16,
    /// Host to bind to.
    pub host: String,
    /// Auto-save interval in milliseconds (0 to disable).
    pub auto_save_interval: u64,
    /// Auto-refresh interval in milliseconds (0 to disable).
    pub auto_refresh_interval: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("."),
            port: 8080,
            host: "127.0.0.1".to_string(),
            auto_save_interval: 5000,
            auto_refresh_interval: 0,
        }
    }
}

impl Config {
    pub fn with_data_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.data_dir = path.into();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn with_host(mut self, host: impl Into<String>) -> Self {
        self.host = host.into();
        self
    }

    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

/// Shared application state.
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub workspace: Arc<RwLock<Option<Workspace>>>,
    pub workspace_path: Arc<RwLock<Option<PathBuf>>>,
    pub editor: EditorState,
    pub watcher: Arc<RwLock<FileWatcher>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            workspace: Arc::new(RwLock::new(None)),
            workspace_path: Arc::new(RwLock::new(None)),
            editor: EditorState::new(),
            watcher: Arc::new(RwLock::new(FileWatcher::new())),
        }
    }

    /// Load workspace from the data directory.
    pub async fn load_workspace(&self) -> crate::Result<()> {
        let data_dir = &self.config.data_dir;

        // Look for workspace.dsl first, then workspace.json
        let dsl_path = data_dir.join("workspace.dsl");
        let json_path = data_dir.join("workspace.json");

        let (workspace, path) = if dsl_path.exists() {
            let content = tokio::fs::read_to_string(&dsl_path).await?;
            let ws = structurizr_dsl::parse(&content)?;
            (ws, dsl_path)
        } else if json_path.exists() {
            let content = tokio::fs::read_to_string(&json_path).await?;
            let ws = Workspace::from_json(&content)?;
            (ws, json_path)
        } else {
            // Create a default workspace
            let ws = Workspace::new("Untitled", "A new workspace");
            (ws, dsl_path)
        };

        *self.workspace.write().await = Some(workspace);
        *self.workspace_path.write().await = Some(path);

        Ok(())
    }

    /// Save workspace to the data directory.
    pub async fn save_workspace(&self) -> crate::Result<()> {
        let workspace = self.workspace.read().await;
        let path = self.workspace_path.read().await;

        if let (Some(ws), Some(p)) = (workspace.as_ref(), path.as_ref()) {
            let json = ws.to_json()?;
            let json_path = p.with_extension("json");
            tokio::fs::write(json_path, json).await?;
        }

        Ok(())
    }

    /// Get a clone of the current workspace.
    pub async fn get_workspace(&self) -> Option<Workspace> {
        self.workspace.read().await.clone()
    }

    /// Start the file watcher for auto-reload.
    pub async fn start_watcher(&self) -> crate::Result<()> {
        let path = self.workspace_path.read().await;

        if let Some(p) = path.as_ref() {
            let watch_path = p.parent()
                .map(|parent| parent.to_path_buf())
                .unwrap_or_else(|| self.config.data_dir.clone());

            let mut watcher = self.watcher.write().await;
            watcher.start(watch_path, self.clone())?;
        }

        Ok(())
    }

    /// Stop the file watcher.
    pub async fn stop_watcher(&self) {
        let mut watcher = self.watcher.write().await;
        watcher.stop();
    }
}
