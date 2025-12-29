//! Collaborative notes for dynamic view steps.
//!
//! This module handles loading and saving timestamped notes to a sidecar file
//! (`.notes.json`) alongside the workspace DSL file.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tokio::fs;
use tracing::{debug, warn};
use uuid::Uuid;

/// Notes file format for storing notes per view and step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesFile {
    /// Version number for future compatibility.
    #[serde(default)]
    pub version: u32,
    /// Notes for each view, keyed by view key.
    #[serde(default)]
    pub views: HashMap<String, ViewNotes>,
}

/// Notes for a single view.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewNotes {
    /// Notes per step, keyed by step index (as string).
    #[serde(default)]
    pub steps: HashMap<String, Vec<Note>>,
}

/// A single note with author and timestamp.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    /// Unique identifier for the note.
    pub id: String,
    /// Author's first name.
    pub first_name: String,
    /// Author's last name.
    pub last_name: String,
    /// ISO 8601 timestamp when the note was created.
    pub timestamp: String,
    /// The note content.
    pub content: String,
}

/// Request body for adding a new note.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddNoteRequest {
    /// The step index this note belongs to.
    pub step_index: u32,
    /// Author's first name.
    pub first_name: String,
    /// Author's last name.
    pub last_name: String,
    /// The note content.
    pub content: String,
}

impl NotesFile {
    /// Current version of the notes file format.
    pub const CURRENT_VERSION: u32 = 1;

    /// Create a new empty notes file.
    pub fn new() -> Self {
        Self {
            version: Self::CURRENT_VERSION,
            views: HashMap::new(),
        }
    }

    /// Get the notes file path for a workspace directory.
    pub fn path_for_workspace(workspace_dir: &Path) -> std::path::PathBuf {
        workspace_dir.join(".notes.json")
    }

    /// Load notes from a workspace directory.
    pub async fn load(workspace_dir: &Path) -> Self {
        let path = Self::path_for_workspace(workspace_dir);

        if !path.exists() {
            debug!("No notes file found at {:?}, using defaults", path);
            return Self::new();
        }

        match fs::read_to_string(&path).await {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(notes) => {
                    debug!("Loaded notes from {:?}", path);
                    notes
                }
                Err(e) => {
                    warn!("Failed to parse notes file {:?}: {}", path, e);
                    Self::new()
                }
            },
            Err(e) => {
                warn!("Failed to read notes file {:?}: {}", path, e);
                Self::new()
            }
        }
    }

    /// Save notes to a workspace directory.
    pub async fn save(&self, workspace_dir: &Path) -> Result<(), std::io::Error> {
        let path = Self::path_for_workspace(workspace_dir);

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        fs::write(&path, content).await?;
        debug!("Saved notes to {:?}", path);

        Ok(())
    }

    /// Add a note to a specific view and step.
    pub fn add_note(
        &mut self,
        view_key: &str,
        step_index: u32,
        first_name: String,
        last_name: String,
        content: String,
    ) -> &Note {
        let view_notes = self
            .views
            .entry(view_key.to_string())
            .or_insert_with(ViewNotes::default);

        let step_key = step_index.to_string();
        let step_notes = view_notes
            .steps
            .entry(step_key.clone())
            .or_insert_with(Vec::new);

        let note = Note {
            id: Uuid::new_v4().to_string(),
            first_name,
            last_name,
            timestamp: chrono::Utc::now().to_rfc3339(),
            content,
        };

        // Insert at the beginning for reverse chronological order
        step_notes.insert(0, note);

        // Return a reference to the newly added note
        &step_notes[0]
    }

    /// Get notes for a specific view.
    pub fn get_view_notes(&self, view_key: &str) -> Option<&ViewNotes> {
        self.views.get(view_key)
    }

    /// Get notes for a specific step in a view.
    pub fn get_step_notes(&self, view_key: &str, step_index: u32) -> Vec<&Note> {
        self.views
            .get(view_key)
            .and_then(|v| v.steps.get(&step_index.to_string()))
            .map(|notes| notes.iter().collect())
            .unwrap_or_default()
    }

    /// Check if any notes exist for a view.
    pub fn has_notes(&self, view_key: &str) -> bool {
        self.views
            .get(view_key)
            .map(|v| v.steps.values().any(|notes| !notes.is_empty()))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_save_and_load() {
        let dir = tempdir().unwrap();

        let mut notes = NotesFile::new();
        notes.add_note(
            "OrderFlow",
            0,
            "John".to_string(),
            "Doe".to_string(),
            "This step needs more documentation.".to_string(),
        );
        notes.add_note(
            "OrderFlow",
            1,
            "Jane".to_string(),
            "Smith".to_string(),
            "Consider adding error handling here.".to_string(),
        );

        notes.save(dir.path()).await.unwrap();

        let loaded = NotesFile::load(dir.path()).await;

        assert!(loaded.has_notes("OrderFlow"));

        let step0_notes = loaded.get_step_notes("OrderFlow", 0);
        assert_eq!(step0_notes.len(), 1);
        assert_eq!(step0_notes[0].first_name, "John");
        assert_eq!(step0_notes[0].content, "This step needs more documentation.");
    }

    #[tokio::test]
    async fn test_load_nonexistent() {
        let dir = tempdir().unwrap();
        let loaded = NotesFile::load(dir.path()).await;
        assert!(!loaded.has_notes("AnyView"));
    }

    #[test]
    fn test_notes_reverse_chronological() {
        let mut notes = NotesFile::new();

        notes.add_note(
            "TestView",
            0,
            "First".to_string(),
            "Author".to_string(),
            "First note".to_string(),
        );
        notes.add_note(
            "TestView",
            0,
            "Second".to_string(),
            "Author".to_string(),
            "Second note".to_string(),
        );

        let step_notes = notes.get_step_notes("TestView", 0);
        assert_eq!(step_notes.len(), 2);
        // Second note should be first (newest at top)
        assert_eq!(step_notes[0].first_name, "Second");
        assert_eq!(step_notes[1].first_name, "First");
    }
}
