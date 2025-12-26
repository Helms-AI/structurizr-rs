//! Workspace discovery for multi-workspace mode.
//!
//! This module provides functionality to discover workspaces in a directory tree
//! and extract metadata for display in the workspace index.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use structurizr_core::Workspace;
use tokio::sync::RwLock;

use crate::error::Result;

/// Information about a discovered workspace.
#[derive(Debug, Clone)]
pub struct WorkspaceInfo {
    /// Unique identifier (relative path from workspaces root, e.g., "small/startup-saas").
    pub id: String,
    /// Workspace name (from workspace.dsl).
    pub name: String,
    /// Workspace description (from workspace.dsl).
    pub description: Option<String>,
    /// Absolute path to the workspace directory.
    pub path: PathBuf,
    /// Number of views defined in the workspace.
    pub view_count: usize,
    /// Last modification time of workspace.dsl.
    pub last_modified: SystemTime,
}

/// Registry for managing multiple workspaces.
pub struct WorkspaceRegistry {
    /// Root directory for workspaces.
    root: PathBuf,
    /// Discovered workspaces indexed by ID.
    workspaces: HashMap<String, WorkspaceInfo>,
    /// Loaded workspace cache.
    loaded: RwLock<HashMap<String, Workspace>>,
}

impl WorkspaceRegistry {
    /// Create a new workspace registry for the given root directory.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            workspaces: HashMap::new(),
            loaded: RwLock::new(HashMap::new()),
        }
    }

    /// Get the root directory.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Discover all workspaces in the root directory.
    pub async fn discover(&mut self) -> Result<()> {
        self.workspaces.clear();
        let workspaces = discover_workspaces(&self.root).await?;
        for ws in workspaces {
            self.workspaces.insert(ws.id.clone(), ws);
        }
        Ok(())
    }

    /// Get information about a workspace by ID.
    pub fn get_info(&self, id: &str) -> Option<&WorkspaceInfo> {
        self.workspaces.get(id)
    }

    /// List all discovered workspaces.
    pub fn list(&self) -> Vec<&WorkspaceInfo> {
        let mut workspaces: Vec<_> = self.workspaces.values().collect();
        workspaces.sort_by(|a, b| a.id.cmp(&b.id));
        workspaces
    }

    /// Get or load a workspace by ID.
    pub async fn get_workspace(&self, id: &str) -> Result<Option<Workspace>> {
        // Check cache first
        {
            let cache = self.loaded.read().await;
            if let Some(ws) = cache.get(id) {
                return Ok(Some(ws.clone()));
            }
        }

        // Load workspace if info exists
        let info = match self.workspaces.get(id) {
            Some(info) => info,
            None => return Ok(None),
        };

        let workspace = load_workspace(&info.path).await?;

        // Cache the loaded workspace
        {
            let mut cache = self.loaded.write().await;
            cache.insert(id.to_string(), workspace.clone());
        }

        Ok(Some(workspace))
    }

    /// Invalidate a workspace from the cache.
    pub async fn invalidate(&self, id: &str) {
        let mut cache = self.loaded.write().await;
        cache.remove(id);
    }

    /// Invalidate all workspaces from the cache.
    pub async fn invalidate_all(&self) {
        let mut cache = self.loaded.write().await;
        cache.clear();
    }

    /// Get workspace ID from a file path.
    pub fn path_to_workspace_id(&self, path: &Path) -> Option<String> {
        // Find which workspace directory contains this path
        for (id, info) in &self.workspaces {
            if path.starts_with(&info.path) {
                return Some(id.clone());
            }
        }
        None
    }

    /// Check if a workspace exists.
    pub fn contains(&self, id: &str) -> bool {
        self.workspaces.contains_key(id)
    }

    /// Get the number of discovered workspaces.
    pub fn len(&self) -> usize {
        self.workspaces.len()
    }

    /// Check if the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.workspaces.is_empty()
    }
}

/// Discover all workspaces in the given root directory.
///
/// Recursively scans for directories containing `workspace.dsl` files
/// and extracts metadata from each.
pub async fn discover_workspaces(root: &Path) -> Result<Vec<WorkspaceInfo>> {
    let mut workspaces = Vec::new();

    if !root.exists() {
        return Ok(workspaces);
    }

    discover_recursive(root, root, &mut workspaces).await?;
    workspaces.sort_by(|a, b| a.id.cmp(&b.id));

    Ok(workspaces)
}

/// Recursively discover workspaces in a directory.
async fn discover_recursive(
    root: &Path,
    current: &Path,
    workspaces: &mut Vec<WorkspaceInfo>,
) -> Result<()> {
    let mut entries = tokio::fs::read_dir(current).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        if path.is_dir() {
            // Check if this directory contains a workspace.dsl
            let dsl_path = path.join("workspace.dsl");
            if dsl_path.exists() {
                if let Ok(info) = extract_workspace_info(root, &path, &dsl_path).await {
                    workspaces.push(info);
                }
            }

            // Continue recursing into subdirectories
            if let Err(e) = Box::pin(discover_recursive(root, &path, workspaces)).await {
                tracing::warn!("Error scanning {:?}: {}", path, e);
            }
        }
    }

    Ok(())
}

/// Extract workspace information from a workspace.dsl file.
async fn extract_workspace_info(
    root: &Path,
    workspace_dir: &Path,
    dsl_path: &Path,
) -> Result<WorkspaceInfo> {
    // Get the workspace ID as the relative path from root
    let id = workspace_dir
        .strip_prefix(root)
        .unwrap_or(workspace_dir)
        .to_string_lossy()
        .replace('\\', "/"); // Normalize path separators

    // Get last modified time
    let metadata = tokio::fs::metadata(dsl_path).await?;
    let last_modified = metadata.modified().unwrap_or_else(|_| SystemTime::now());

    // Read and parse the DSL file to extract metadata
    let content = tokio::fs::read_to_string(dsl_path).await?;
    let (name, description, view_count) = extract_metadata_from_dsl(&content);

    Ok(WorkspaceInfo {
        id,
        name,
        description,
        path: workspace_dir.to_path_buf(),
        view_count,
        last_modified,
    })
}

/// Extract workspace metadata from DSL content without full parsing.
///
/// This is a lightweight extraction that reads the workspace name and description
/// from the first few lines without fully parsing the DSL.
fn extract_metadata_from_dsl(content: &str) -> (String, Option<String>, usize) {
    let mut name = "Untitled".to_string();
    let mut description = None;
    let mut view_count = 0;

    // Simple regex-like parsing for workspace declaration
    // Format: workspace "Name" "Description" { or workspace "Name" {
    for line in content.lines().take(50) {
        let trimmed = line.trim();

        // Look for workspace declaration
        if trimmed.starts_with("workspace") {
            let parts: Vec<&str> = trimmed.split('"').collect();
            if parts.len() >= 2 {
                name = parts[1].to_string();
            }
            if parts.len() >= 4 {
                let desc = parts[3].trim();
                if !desc.is_empty() {
                    description = Some(desc.to_string());
                }
            }
        }
    }

    // Count views by looking for view keywords
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("systemLandscape ")
            || trimmed.starts_with("systemContext ")
            || trimmed.starts_with("container ")
            || trimmed.starts_with("component ")
            || trimmed.starts_with("dynamic ")
            || trimmed.starts_with("deployment ")
            || trimmed.starts_with("filtered ")
        {
            // Check if this looks like a view declaration (has a key in quotes)
            if trimmed.contains('"') {
                view_count += 1;
            }
        }
    }

    (name, description, view_count)
}

/// Load a workspace from a directory.
async fn load_workspace(workspace_dir: &Path) -> Result<Workspace> {
    let dsl_path = workspace_dir.join("workspace.dsl");
    let json_path = workspace_dir.join("workspace.json");

    let mut workspace = if dsl_path.exists() {
        let content = tokio::fs::read_to_string(&dsl_path).await?;
        structurizr_dsl::parse_with_base_path(&content, Some(workspace_dir))?
    } else if json_path.exists() {
        let content = tokio::fs::read_to_string(&json_path).await?;
        Workspace::from_json(&content)?
    } else {
        return Err(crate::error::Error::WorkspaceNotFound(
            workspace_dir.to_string_lossy().to_string(),
        ));
    };

    // Load documentation and ADRs
    crate::state::load_documentation(&mut workspace, workspace_dir).await?;

    Ok(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_metadata_simple() {
        let content = r#"workspace "My System" "A description of the system" {
            model {
            }
            views {
                systemContext system "Context" {
                }
            }
        }"#;

        let (name, desc, views) = extract_metadata_from_dsl(content);
        assert_eq!(name, "My System");
        assert_eq!(desc, Some("A description of the system".to_string()));
        assert_eq!(views, 1);
    }

    #[test]
    fn test_extract_metadata_no_description() {
        let content = r#"workspace "My System" {
            model {
            }
        }"#;

        let (name, desc, views) = extract_metadata_from_dsl(content);
        assert_eq!(name, "My System");
        assert_eq!(desc, None);
        assert_eq!(views, 0);
    }

    #[test]
    fn test_extract_metadata_multiple_views() {
        let content = r#"workspace "Test" "Desc" {
            views {
                systemLandscape "Landscape" {}
                systemContext sys "Context" {}
                container sys "Containers" {}
                dynamic sys "Flow" {}
            }
        }"#;

        let (_, _, views) = extract_metadata_from_dsl(content);
        assert_eq!(views, 4);
    }
}
