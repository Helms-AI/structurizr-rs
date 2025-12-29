//! Position persistence for drag-and-drop element placement.
//!
//! This module handles loading and saving element positions to a sidecar file
//! (`.positions.json`) alongside the workspace DSL file.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, warn};

/// Positions file format for storing element positions per view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PositionsFile {
    /// Version number for future compatibility.
    #[serde(default)]
    pub version: u32,
    /// Positions for each view, keyed by view key.
    #[serde(default)]
    pub views: HashMap<String, ViewPositions>,
}

/// Positions for a single view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewPositions {
    /// Element positions, keyed by element ID.
    #[serde(default)]
    pub elements: HashMap<String, ElementPosition>,
    /// Relationship routing vertices, keyed by relationship ID.
    #[serde(default)]
    pub relationships: HashMap<String, Vec<Vertex>>,
}

/// Element position in a view.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementPosition {
    pub x: i32,
    pub y: i32,
}

/// Vertex in a relationship route.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

impl PositionsFile {
    /// Current version of the positions file format.
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty positions file.
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            views: HashMap::new(),
        }
    }

    /// Get the positions file path for a workspace directory.
    pub fn path_for_workspace(workspace_dir: &Path) -> std::path::PathBuf {
        workspace_dir.join(".positions.json")
    }

    /// Load positions from a workspace directory.
    pub async fn load(workspace_dir: &Path) -> Self {
        let path = Self::path_for_workspace(workspace_dir);

        if !path.exists() {
            debug!("No positions file found at {:?}, using defaults", path);
            return Self::new();
        }

        match fs::read_to_string(&path).await {
            Ok(content) => {
                match serde_json::from_str(&content) {
                    Ok(positions) => {
                        debug!("Loaded positions from {:?}", path);
                        positions
                    }
                    Err(e) => {
                        warn!("Failed to parse positions file {:?}: {}", path, e);
                        Self::new()
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read positions file {:?}: {}", path, e);
                Self::new()
            }
        }
    }

    /// Save positions to a workspace directory.
    pub async fn save(&self, workspace_dir: &Path) -> Result<(), std::io::Error> {
        let path = Self::path_for_workspace(workspace_dir);

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&path, content).await?;
        debug!("Saved positions to {:?}", path);

        Ok(())
    }

    /// Update a single element position.
    pub fn update_element_position(
        &mut self,
        view_key: &str,
        element_id: &str,
        x: i32,
        y: i32,
    ) {
        let view_positions = self.views
            .entry(view_key.to_string())
            .or_insert_with(ViewPositions::default);

        view_positions.elements.insert(
            element_id.to_string(),
            ElementPosition { x, y },
        );
    }

    /// Get element positions for a view.
    pub fn get_view_positions(&self, view_key: &str) -> Option<&ViewPositions> {
        self.views.get(view_key)
    }

    /// Check if positions have been set for a view.
    pub fn has_positions(&self, view_key: &str) -> bool {
        self.views.get(view_key)
            .map(|v| !v.elements.is_empty())
            .unwrap_or(false)
    }

    /// Merge loaded positions into a workspace's view elements.
    pub fn apply_to_workspace(&self, workspace: &mut structurizr_core::Workspace) {
        use structurizr_core::view::ElementView;

        // Helper to apply positions to a list of element views
        fn apply_to_elements(
            elements: &mut Vec<ElementView>,
            view_positions: Option<&ViewPositions>,
        ) {
            if let Some(positions) = view_positions {
                for elem in elements.iter_mut() {
                    let elem_id = elem.id.to_string();
                    if let Some(pos) = positions.elements.get(&elem_id) {
                        elem.x = Some(pos.x);
                        elem.y = Some(pos.y);
                    }
                }
            }
        }

        // Apply to all view types
        for view in &mut workspace.views_mut().system_landscape_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }

        for view in &mut workspace.views_mut().system_context_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }

        for view in &mut workspace.views_mut().container_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }

        for view in &mut workspace.views_mut().component_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }

        for view in &mut workspace.views_mut().dynamic_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }

        for view in &mut workspace.views_mut().deployment_views {
            let positions = self.get_view_positions(&view.properties.key);
            apply_to_elements(&mut view.properties.elements, positions);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();

        let mut positions = PositionsFile::new();
        positions.update_element_position("SystemContext", "abc-123", 100, 200);
        positions.update_element_position("SystemContext", "def-456", 300, 400);
        positions.update_element_position("Container", "ghi-789", 500, 600);

        positions.save(dir.path()).await.unwrap();

        let loaded = PositionsFile::load(dir.path()).await;

        assert!(loaded.has_positions("SystemContext"));
        assert!(loaded.has_positions("Container"));

        let sc = loaded.get_view_positions("SystemContext").unwrap();
        assert_eq!(sc.elements.get("abc-123").unwrap().x, 100);
        assert_eq!(sc.elements.get("abc-123").unwrap().y, 200);
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let loaded = PositionsFile::load(dir.path()).await;
        assert!(!loaded.has_positions("AnyView"));
    }
}
