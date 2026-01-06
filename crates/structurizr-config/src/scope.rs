//! Workspace scoping for MCP sessions
//!
//! Provides pattern-based access control for workspaces
//! with support for auto-including newly created workspaces.

use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Workspace scope mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ScopeMode {
    /// All workspaces are accessible
    All,
    /// Only workspaces matching patterns are accessible
    Allow,
    /// All workspaces except those matching patterns are accessible
    Deny,
}

impl Default for ScopeMode {
    fn default() -> Self {
        // Default to all in development, deny in production
        if super::discovery::is_production() {
            ScopeMode::Deny
        } else {
            ScopeMode::All
        }
    }
}

/// Workspace scope configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceScope {
    /// Scope mode
    #[serde(default)]
    pub mode: ScopeMode,

    /// Glob patterns for workspace matching
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Auto-include workspaces created by this session
    #[serde(default = "bool_true")]
    pub auto_include_created: bool,

    /// Audit workspace access attempts
    #[serde(default = "bool_false")]
    pub audit_access: bool,

    /// Runtime state (not serialized)
    #[serde(skip)]
    runtime_state: Option<Arc<RwLock<RuntimeScopeState>>>,
}

fn bool_true() -> bool { true }
fn bool_false() -> bool { false }

impl Default for WorkspaceScope {
    fn default() -> Self {
        Self {
            mode: ScopeMode::default(),
            patterns: Vec::new(),
            auto_include_created: true,
            audit_access: false,
            runtime_state: None,
        }
    }
}

/// Runtime state for workspace scoping
#[derive(Debug, Default)]
struct RuntimeScopeState {
    /// Workspaces created by this session (auto-included)
    created_workspaces: HashSet<String>,

    /// Compiled glob patterns for efficient matching
    glob_set: Option<GlobSet>,
}

impl WorkspaceScope {
    /// Initialize runtime state
    pub fn init_runtime(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Compile glob patterns
        let mut builder = GlobSetBuilder::new();
        for pattern in &self.patterns {
            // Handle negation patterns
            if let Some(_pattern) = pattern.strip_prefix('!') {
                // Negation patterns handled separately
                continue;
            }
            builder.add(Glob::new(pattern)?);
        }
        let glob_set = builder.build()?;

        self.runtime_state = Some(Arc::new(RwLock::new(RuntimeScopeState {
            created_workspaces: HashSet::new(),
            glob_set: Some(glob_set),
        })));

        Ok(())
    }

    /// Check if a workspace is accessible
    pub async fn is_accessible(&self, workspace_id: &str) -> bool {
        // Check mode
        match self.mode {
            ScopeMode::All => return true,
            ScopeMode::Allow | ScopeMode::Deny => {}
        }

        // Check if this was created by our session
        if self.auto_include_created {
            if let Some(ref state) = self.runtime_state {
                let state = state.read().await;
                if state.created_workspaces.contains(workspace_id) {
                    if self.audit_access {
                        tracing::info!("Workspace access granted (created): {}", workspace_id);
                    }
                    return true;
                }
            }
        }

        // Check patterns
        let matches = self.matches_patterns(workspace_id).await;

        let accessible = match self.mode {
            ScopeMode::All => true,
            ScopeMode::Allow => matches,
            ScopeMode::Deny => !matches,
        };

        if self.audit_access {
            tracing::info!(
                "Workspace access {}: {} (mode: {:?}, matches: {})",
                if accessible { "granted" } else { "denied" },
                workspace_id,
                self.mode,
                matches
            );
        }

        accessible
    }

    /// Check if a workspace matches the configured patterns
    async fn matches_patterns(&self, workspace_id: &str) -> bool {
        // First check positive patterns
        if let Some(ref state) = self.runtime_state {
            let state = state.read().await;
            if let Some(ref glob_set) = state.glob_set {
                if glob_set.is_match(workspace_id) {
                    // Check for negation patterns
                    for pattern in &self.patterns {
                        if let Some(negation) = pattern.strip_prefix('!') {
                            if let Ok(glob) = Glob::new(negation) {
                                if glob.compile_matcher().is_match(workspace_id) {
                                    return false;  // Negation pattern matches, exclude
                                }
                            }
                        }
                    }
                    return true;
                }
            }
        }
        false
    }

    /// Register a newly created workspace for auto-inclusion
    pub async fn register_created(&self, workspace_id: String) {
        if !self.auto_include_created {
            return;
        }

        if let Some(ref state) = self.runtime_state {
            let mut state = state.write().await;
            state.created_workspaces.insert(workspace_id.clone());
            tracing::info!("Auto-included created workspace: {}", workspace_id);
        }
    }

    /// Get list of accessible workspaces from a set
    pub async fn filter_accessible(&self, workspace_ids: Vec<String>) -> Vec<String> {
        let mut accessible = Vec::new();
        for id in workspace_ids {
            if self.is_accessible(&id).await {
                accessible.push(id);
            }
        }
        accessible
    }

    /// Create a scope that allows specific workspaces
    pub fn allow_list(workspaces: Vec<String>) -> Self {
        Self {
            mode: ScopeMode::Allow,
            patterns: workspaces,
            auto_include_created: true,
            audit_access: false,
            runtime_state: None,
        }
    }

    /// Create a scope that denies specific workspaces
    pub fn deny_list(workspaces: Vec<String>) -> Self {
        Self {
            mode: ScopeMode::Deny,
            patterns: workspaces,
            auto_include_created: true,
            audit_access: false,
            runtime_state: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scope_all_mode() {
        let scope = WorkspaceScope {
            mode: ScopeMode::All,
            ..Default::default()
        };

        assert!(scope.is_accessible("any/workspace").await);
        assert!(scope.is_accessible("another/workspace").await);
    }

    #[tokio::test]
    async fn test_scope_allow_mode() {
        let mut scope = WorkspaceScope {
            mode: ScopeMode::Allow,
            patterns: vec![
                "team-a/*".to_string(),
                "shared/**".to_string(),
            ],
            ..Default::default()
        };
        scope.init_runtime().unwrap();

        assert!(scope.is_accessible("team-a/project1").await);
        assert!(scope.is_accessible("shared/common/lib").await);
        assert!(!scope.is_accessible("team-b/project1").await);
    }

    #[tokio::test]
    async fn test_scope_deny_mode() {
        let mut scope = WorkspaceScope {
            mode: ScopeMode::Deny,
            patterns: vec![
                "test/*".to_string(),
                "*.tmp".to_string(),
            ],
            ..Default::default()
        };
        scope.init_runtime().unwrap();

        assert!(!scope.is_accessible("test/workspace").await);
        assert!(!scope.is_accessible("workspace.tmp").await);
        assert!(scope.is_accessible("production/app").await);
    }

    #[tokio::test]
    async fn test_negation_patterns() {
        let mut scope = WorkspaceScope {
            mode: ScopeMode::Allow,
            patterns: vec![
                "team-a/*".to_string(),
                "!team-a/secret".to_string(),
            ],
            ..Default::default()
        };
        scope.init_runtime().unwrap();

        assert!(scope.is_accessible("team-a/public").await);
        assert!(!scope.is_accessible("team-a/secret").await);
    }

    #[tokio::test]
    async fn test_auto_include_created() {
        let mut scope = WorkspaceScope {
            mode: ScopeMode::Deny,
            patterns: vec!["*".to_string()],  // Deny all
            auto_include_created: true,
            ..Default::default()
        };
        scope.init_runtime().unwrap();

        // Initially not accessible
        assert!(!scope.is_accessible("new/workspace").await);

        // Register as created
        scope.register_created("new/workspace".to_string()).await;

        // Now accessible
        assert!(scope.is_accessible("new/workspace").await);
    }
}