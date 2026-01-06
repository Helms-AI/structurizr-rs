//! Workspace management for Structurizr.
//!
//! A workspace is the top-level container that holds the model, views,
//! documentation, and styling configuration.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::error::Result;
use crate::model::Model;
use crate::style::{Styles, ViewConfiguration};
use crate::view::Views;

/// A perspective represents a stakeholder view of the architecture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Perspective {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl Perspective {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Documentation section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentationSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub content: String,
    #[serde(default)]
    pub format: DocumentationFormat,
    #[serde(default)]
    pub order: u32,
}

/// Format for documentation content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DocumentationFormat {
    #[default]
    Markdown,
    AsciiDoc,
}

/// An Architecture Decision Record (ADR).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub format: DocumentationFormat,
    pub status: DecisionStatus,
    pub date: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<DecisionLink>,
}

/// Status of an architecture decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum DecisionStatus {
    #[default]
    Proposed,
    Accepted,
    Superseded,
    Deprecated,
    Rejected,
}

/// A link between decisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLink {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Documentation attached to a workspace or element.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Documentation {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<DocumentationSection>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<Decision>,
}

impl Documentation {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_section(&mut self, title: impl Into<String>, content: impl Into<String>) {
        let order = self.sections.len() as u32 + 1;
        self.sections.push(DocumentationSection {
            title: Some(title.into()),
            content: content.into(),
            format: DocumentationFormat::Markdown,
            order,
        });
    }

    pub fn add_decision(&mut self, decision: Decision) {
        self.decisions.push(decision);
    }
}

/// Scope of the workspace - determines what level of architecture is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WorkspaceScope {
    /// Landscape scope - shows all software systems.
    Landscape,
    /// Software system scope - focuses on a single software system.
    SoftwareSystem,
    /// No specific scope.
    None,
}

/// Visibility of the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum WorkspaceVisibility {
    /// Private - only accessible to authorized users.
    #[default]
    Private,
    /// Public - accessible to anyone.
    Public,
}

/// Role a user has in the workspace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum UserRole {
    /// Read-only access.
    ReadOnly,
    /// Read-write access.
    ReadWrite,
}

/// A user with access to the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceUser {
    pub username: String,
    pub role: UserRole,
}

impl WorkspaceUser {
    pub fn new(username: impl Into<String>, role: UserRole) -> Self {
        Self {
            username: username.into(),
            role,
        }
    }
}

/// Workspace-level configuration (scope, visibility, users).
/// This is separate from ViewConfiguration which handles styles/branding.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceConfiguration {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<WorkspaceScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<WorkspaceVisibility>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub users: Vec<WorkspaceUser>,
}

/// Metadata about a workspace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceMetadata {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_secret: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_agent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

/// A Structurizr workspace containing the complete architecture model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revision: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_date: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_user: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_modified_agent: Option<String>,
    #[serde(default)]
    pub model: Model,
    #[serde(default)]
    pub views: Views,
    #[serde(default)]
    pub documentation: Documentation,
    /// View configuration (styles, branding, terminology) - serialized in JSON.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub configuration: Option<ViewConfiguration>,
    /// Workspace-level configuration (scope, visibility, users) - used for cloud deployment.
    #[serde(default, rename = "workspaceConfiguration", skip_serializing_if = "Option::is_none")]
    pub workspace_configuration: Option<WorkspaceConfiguration>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perspectives: Vec<Perspective>,
}

impl Workspace {
    /// Create a new workspace with the given name.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            description: Some(description.into()),
            version: None,
            revision: None,
            last_modified_date: Some(Utc::now()),
            last_modified_user: None,
            last_modified_agent: Some("structurizr-rs".to_string()),
            model: Model::new(),
            views: Views::new(),
            documentation: Documentation::new(),
            configuration: Some(ViewConfiguration::default()),
            workspace_configuration: None,
            properties: HashMap::new(),
            perspectives: Vec::new(),
        }
    }

    /// Get a mutable reference to the model.
    pub fn model_mut(&mut self) -> &mut Model {
        &mut self.model
    }

    /// Get a reference to the model.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// Get a mutable reference to the views.
    pub fn views_mut(&mut self) -> &mut Views {
        &mut self.views
    }

    /// Get a reference to the views.
    pub fn views(&self) -> &Views {
        &self.views
    }

    /// Get a mutable reference to the styles.
    pub fn styles_mut(&mut self) -> &mut Styles {
        if self.configuration.is_none() {
            self.configuration = Some(ViewConfiguration::default());
        }
        &mut self.configuration.as_mut().unwrap().styles
    }

    /// Get a reference to the styles.
    pub fn styles(&self) -> Option<&Styles> {
        self.configuration.as_ref().map(|c| &c.styles)
    }

    /// Load a workspace from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let workspace = serde_json::from_str(&content)?;
        Ok(workspace)
    }

    /// Save the workspace to a JSON file.
    pub fn to_json_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Serialize the workspace to JSON.
    pub fn to_json(&self) -> Result<String> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    /// Deserialize a workspace from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        let workspace = serde_json::from_str(json)?;
        Ok(workspace)
    }

    /// Update the last modified timestamp.
    pub fn touch(&mut self) {
        self.last_modified_date = Some(Utc::now());
    }

    /// Set a property on the workspace.
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.properties.insert(key.into(), value.into());
    }

    /// Get a property from the workspace.
    pub fn get_property(&self, key: &str) -> Option<&String> {
        self.properties.get(key)
    }

    /// Add a perspective to the workspace.
    pub fn add_perspective(&mut self, perspective: Perspective) {
        self.perspectives.push(perspective);
    }

    /// Get all perspectives.
    pub fn get_perspectives(&self) -> &[Perspective] {
        &self.perspectives
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Self::new("Untitled", "")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workspace() {
        let workspace = Workspace::new("Test System", "A test workspace");
        assert_eq!(workspace.name, "Test System");
        assert_eq!(workspace.description, Some("A test workspace".to_string()));
    }

    #[test]
    fn test_add_elements() {
        let mut workspace = Workspace::new("Test", "Test");
        let user_id = workspace.model_mut().add_person("User", "A user");
        let system_id = workspace.model_mut().add_software_system("System", "A system");

        assert_eq!(workspace.model().people.len(), 1);
        assert_eq!(workspace.model().software_systems.len(), 1);

        workspace.model_mut().add_relationship(user_id, system_id, "Uses", None);
        assert_eq!(workspace.model().relationships.len(), 1);
    }

    #[test]
    fn test_json_serialization() {
        let workspace = Workspace::new("Test", "A test");
        let json = workspace.to_json().unwrap();
        let restored = Workspace::from_json(&json).unwrap();
        assert_eq!(restored.name, workspace.name);
    }
}
