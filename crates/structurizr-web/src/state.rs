//! Application state for the web server.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use structurizr_core::Workspace;
use structurizr_core::workspace::{DocumentationSection, DocumentationFormat, Decision, DecisionStatus};

use crate::discovery::WorkspaceRegistry;
use crate::editor::EditorState;
use crate::watcher::FileWatcher;

/// Configuration for the web server.
#[derive(Debug, Clone)]
pub struct Config {
    /// Directory containing workspaces.
    pub workspaces_dir: PathBuf,
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
            workspaces_dir: PathBuf::from("workspaces"),
            port: 8080,
            host: "127.0.0.1".to_string(),
            auto_save_interval: 5000,
            auto_refresh_interval: 0,
        }
    }
}

impl Config {
    /// Create a new config with a workspaces directory.
    pub fn new(workspaces_dir: impl Into<PathBuf>) -> Self {
        Self {
            workspaces_dir: workspaces_dir.into(),
            ..Default::default()
        }
    }

    /// Get the workspaces directory.
    pub fn workspaces_dir(&self) -> &PathBuf {
        &self.workspaces_dir
    }

    pub fn with_workspaces_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspaces_dir = path.into();
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
    /// Workspace registry for discovering and managing workspaces.
    pub registry: Arc<RwLock<Option<WorkspaceRegistry>>>,
    pub editor: EditorState,
    pub watcher: Arc<RwLock<FileWatcher>>,
}

impl AppState {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            registry: Arc::new(RwLock::new(None)),
            editor: EditorState::new(),
            watcher: Arc::new(RwLock::new(FileWatcher::new())),
        }
    }

    /// Initialize the state by discovering workspaces.
    pub async fn initialize(&self) -> crate::Result<()> {
        let mut registry = WorkspaceRegistry::new(&self.config.workspaces_dir);
        registry.discover().await?;
        *self.registry.write().await = Some(registry);
        Ok(())
    }

    /// Get a workspace by ID.
    pub async fn get_workspace_by_id(&self, id: &str) -> Option<Workspace> {
        let registry = self.registry.read().await;
        if let Some(reg) = registry.as_ref() {
            reg.get_workspace(id).await.ok().flatten()
        } else {
            None
        }
    }

    /// Get workspace info by ID.
    pub async fn get_workspace_info(&self, id: &str) -> Option<crate::discovery::WorkspaceInfo> {
        let registry = self.registry.read().await;
        registry.as_ref()?.get_info(id).cloned()
    }

    /// List all workspaces.
    pub async fn list_workspaces(&self) -> Vec<crate::discovery::WorkspaceInfo> {
        let registry = self.registry.read().await;
        if let Some(reg) = registry.as_ref() {
            reg.list().into_iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    /// Invalidate a workspace from the cache.
    pub async fn invalidate_workspace(&self, id: &str) {
        let registry = self.registry.read().await;
        if let Some(reg) = registry.as_ref() {
            reg.invalidate(id).await;
        }
    }

    /// Rediscover workspaces.
    pub async fn rediscover_workspaces(&self) -> crate::Result<()> {
        let mut registry = self.registry.write().await;
        if let Some(reg) = registry.as_mut() {
            reg.discover().await?;
        }
        Ok(())
    }

    /// Start the file watcher for auto-reload.
    pub async fn start_multi_watcher(&self) -> crate::Result<()> {
        let mut watcher = self.watcher.write().await;
        watcher.start_multi(self.config.workspaces_dir.clone(), self.clone())?;
        Ok(())
    }

    /// Stop the file watcher.
    pub async fn stop_watcher(&self) {
        let mut watcher = self.watcher.write().await;
        watcher.stop();
    }
}

/// Convert a string to title case.
fn titlecase(s: &str) -> String {
    s.split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parse ADR status from content (look for ## Status section).
fn parse_adr_status(content: &str) -> DecisionStatus {
    // Look for ## Status section and extract the status
    let lines: Vec<&str> = content.lines().collect();
    for (i, line) in lines.iter().enumerate() {
        if line.trim().to_lowercase().starts_with("## status") {
            // Get the next non-empty line
            for next_line in lines.iter().skip(i + 1) {
                let trimmed = next_line.trim().to_lowercase();
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed.contains("accepted") {
                    return DecisionStatus::Accepted;
                } else if trimmed.contains("proposed") {
                    return DecisionStatus::Proposed;
                } else if trimmed.contains("superseded") {
                    return DecisionStatus::Superseded;
                } else if trimmed.contains("deprecated") {
                    return DecisionStatus::Deprecated;
                } else if trimmed.contains("rejected") {
                    return DecisionStatus::Rejected;
                }
                break;
            }
        }
    }
    DecisionStatus::Proposed // Default
}

/// Extract title from ADR filename or content.
fn extract_adr_title(filename: &str, content: &str) -> String {
    // First try to get title from first # heading
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            return trimmed[2..].trim().to_string();
        }
    }
    // Fallback to filename
    filename
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '-')
        .replace("-", " ")
        .replace("_", " ")
        .trim()
        .to_string()
}

/// Extract ADR ID from filename (e.g., "001-title.md" -> "1").
fn extract_adr_id(filename: &str) -> String {
    // Extract leading digits
    let digits: String = filename.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        filename.to_string()
    } else {
        // Remove leading zeros
        digits.trim_start_matches('0').to_string()
    }
}

/// Load documentation files from the directory specified by !docs and !adrs directives.
pub async fn load_documentation(workspace: &mut Workspace, data_dir: &std::path::Path) -> crate::Result<()> {
    // Load docs if !docs directive was specified
    if let Some(docs_path) = workspace.get_property("structurizr.docs").cloned() {
        let docs_dir = data_dir.join(&docs_path);
        if docs_dir.exists() && docs_dir.is_dir() {
            let mut entries: Vec<_> = Vec::new();
            let mut read_dir = tokio::fs::read_dir(&docs_dir).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                entries.push(entry);
            }

            // Sort entries by filename for consistent ordering
            entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            let mut order = 1u32;
            for entry in entries {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let filename = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Untitled");

                    // Use "index" as a special case for the main page
                    // Strip numeric prefix (e.g., "001_" or "002-") from filename
                    let title = if filename.to_lowercase() == "index" {
                        Some("Overview".to_string())
                    } else {
                        let clean_name = filename
                            .trim_start_matches(|c: char| c.is_ascii_digit())
                            .trim_start_matches(|c: char| c == '_' || c == '-');
                        Some(titlecase(&clean_name.replace("-", " ").replace("_", " ")))
                    };

                    workspace.documentation.sections.push(DocumentationSection {
                        title,
                        content,
                        format: DocumentationFormat::Markdown,
                        order,
                    });
                    order += 1;
                }
            }
        }
    }

    // Load ADRs if !adrs directive was specified
    if let Some(adrs_path) = workspace.get_property("structurizr.adrs").cloned() {
        let adrs_dir = data_dir.join(&adrs_path);
        if adrs_dir.exists() && adrs_dir.is_dir() {
            let mut entries: Vec<_> = Vec::new();
            let mut read_dir = tokio::fs::read_dir(&adrs_dir).await?;
            while let Some(entry) = read_dir.next_entry().await? {
                entries.push(entry);
            }

            // Sort entries by filename for consistent ordering
            entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            for entry in entries {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let filename = path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("untitled");

                    let id = extract_adr_id(filename);
                    let title = extract_adr_title(filename, &content);
                    let status = parse_adr_status(&content);

                    // Try to extract date from content or use current date
                    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

                    workspace.documentation.decisions.push(Decision {
                        id,
                        title,
                        content,
                        format: DocumentationFormat::Markdown,
                        status,
                        date,
                        links: Vec::new(),
                    });
                }
            }
        }
    }

    Ok(())
}
