//! State management for the MCP server

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use structurizr_core::Workspace;
use structurizr_web::discovery::WorkspaceRegistry;

use crate::error::Result;

/// Session data for MCP clients
#[derive(Debug, Clone)]
pub struct SessionData {
    pub session_id: Uuid,
    pub active_workspace: Option<String>,
    pub created_at: std::time::Instant,
}

/// MCP server state
#[derive(Clone)]
pub struct McpServerState {
    /// Shared workspace registry with web server
    pub registry: Arc<RwLock<Option<WorkspaceRegistry>>>,

    /// Current active workspace for the session
    pub active_workspace: Arc<RwLock<Option<String>>>,

    /// Session data for connected clients
    pub sessions: Arc<RwLock<HashMap<Uuid, SessionData>>>,

    /// Root directory for workspaces
    pub workspaces_dir: PathBuf,
}

impl McpServerState {
    /// Create a new MCP server state
    pub fn new(workspaces_dir: PathBuf) -> Self {
        Self {
            registry: Arc::new(RwLock::new(None)),
            active_workspace: Arc::new(RwLock::new(None)),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            workspaces_dir,
        }
    }

    /// Initialize the workspace registry
    pub async fn initialize_registry(&self) -> Result<()> {
        let mut registry = WorkspaceRegistry::new(self.workspaces_dir.clone());
        registry.discover().await?;
        *self.registry.write().await = Some(registry);
        Ok(())
    }

    /// List all available workspaces
    pub async fn list_workspaces(&self) -> Result<Vec<WorkspaceInfo>> {
        let registry = self.registry.read().await;
        let registry = registry.as_ref().ok_or_else(|| {
            crate::error::McpError::InternalError("Registry not initialized".to_string())
        })?;

        // list() returns Vec<&WorkspaceInfo>
        let web_workspaces = registry.list();

        let workspaces = web_workspaces
            .into_iter()
            .map(|w| WorkspaceInfo {
                id: w.id.clone(),
                name: w.name.clone(),
                description: w.description.clone(),
                path: w.path.clone(),
                view_count: w.view_count,
            })
            .collect();

        Ok(workspaces)
    }

    /// Get a workspace by ID
    pub async fn get_workspace_by_id(&self, workspace_id: &str) -> Option<Workspace> {
        let registry = self.registry.read().await;
        let registry = registry.as_ref()?;
        match registry.get_workspace(workspace_id).await {
            Ok(workspace) => workspace,
            Err(_) => None,
        }
    }

    /// Set the active workspace for the session
    pub async fn set_active_workspace(&self, workspace_id: String) {
        *self.active_workspace.write().await = Some(workspace_id);
    }

    /// Get the active workspace
    pub async fn get_active_workspace(&self) -> Option<Workspace> {
        let active_id = self.active_workspace.read().await.clone()?;
        self.get_workspace_by_id(&active_id).await
    }

    /// Create a new session
    pub async fn create_session(&self) -> Uuid {
        let session_id = Uuid::new_v4();
        let session_data = SessionData {
            session_id,
            active_workspace: None,
            created_at: std::time::Instant::now(),
        };
        self.sessions.write().await.insert(session_id, session_data);
        session_id
    }

    /// Remove a session
    pub async fn remove_session(&self, session_id: Uuid) {
        self.sessions.write().await.remove(&session_id);
    }
}

/// Information about a workspace
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub path: PathBuf,
    pub view_count: usize,
}