//! Application state for the web server.

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use structurizr_core::Workspace;
use structurizr_core::workspace::{DocumentationSection, DocumentationFormat, Decision, DecisionStatus};

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

        let (mut workspace, path) = if dsl_path.exists() {
            let content = tokio::fs::read_to_string(&dsl_path).await?;
            let ws = structurizr_dsl::parse_with_base_path(&content, Some(data_dir))?;
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

        // Load documentation and ADRs from disk if !docs or !adrs directives were specified
        load_documentation(&mut workspace, data_dir).await?;

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
async fn load_documentation(workspace: &mut Workspace, data_dir: &std::path::Path) -> crate::Result<()> {
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
