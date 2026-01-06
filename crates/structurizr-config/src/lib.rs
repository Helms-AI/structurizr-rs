//! Configuration management for structurizr-rs
//!
//! Provides TOML-based configuration with:
//! - Multiple config file locations
//! - Environment variable interpolation
//! - Profile support
//! - Workspace scoping
//! - CRDT-based collaborative editing

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub mod crdt;
pub mod discovery;
pub mod scope;
pub mod validation;

pub use discovery::find_config_file;
pub use scope::{WorkspaceScope, ScopeMode};

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Environment variable error: {0}")]
    EnvVar(String),
}

/// Root configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Metadata about the config file
    #[serde(default)]
    pub meta: MetaConfig,

    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,

    /// MCP configuration
    #[serde(default)]
    pub mcp: McpConfig,

    /// Collaboration configuration
    #[serde(default)]
    pub collaboration: CollaborationConfig,

    /// Profile-specific overrides
    #[serde(default)]
    pub profiles: HashMap<String, ProfileOverride>,
}

/// Config file metadata
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MetaConfig {
    /// Config schema version
    #[serde(default = "default_version")]
    pub version: String,

    /// Active profile name
    #[serde(default = "default_profile")]
    pub profile: String,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_profile() -> String {
    std::env::var("STRUCTURIZR_PROFILE").unwrap_or_else(|_| "default".to_string())
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    /// Server port
    #[serde(default = "default_server_port")]
    pub port: u16,

    /// Server host
    #[serde(default = "default_server_host")]
    pub host: String,

    /// Workspaces directory
    #[serde(default = "default_workspaces_dir")]
    pub workspaces_dir: PathBuf,
}

fn default_server_port() -> u16 { 8080 }
fn default_server_host() -> String { "127.0.0.1".to_string() }
fn default_workspaces_dir() -> PathBuf { PathBuf::from("workspaces") }

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: default_server_port(),
            host: default_server_host(),
            workspaces_dir: default_workspaces_dir(),
        }
    }
}

/// MCP configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpConfig {
    /// Enable MCP server
    #[serde(default = "bool_true")]
    pub enabled: bool,

    /// Auto-start MCP server
    #[serde(default = "bool_true")]
    pub auto_start: bool,

    /// MCP server settings
    #[serde(default)]
    pub server: McpServerConfig,

    /// MCP proxy settings
    #[serde(default)]
    pub proxy: McpProxyConfig,

    /// Workspace scoping
    #[serde(default)]
    pub workspace_scope: WorkspaceScope,
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            auto_start: true,
            server: McpServerConfig::default(),
            proxy: McpProxyConfig::default(),
            workspace_scope: WorkspaceScope::default(),
        }
    }
}

/// MCP server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpServerConfig {
    /// MCP server port
    #[serde(default = "default_mcp_port")]
    pub port: u16,

    /// Transport type
    #[serde(default = "default_mcp_transport")]
    pub transport: String,

    /// Health check interval in milliseconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_ms: u64,
}

fn default_mcp_port() -> u16 { 8586 }
fn default_mcp_transport() -> String { "websocket".to_string() }
fn default_health_check_interval() -> u64 { 30000 }

impl Default for McpServerConfig {
    fn default() -> Self {
        Self {
            port: default_mcp_port(),
            transport: default_mcp_transport(),
            health_check_interval_ms: default_health_check_interval(),
        }
    }
}

/// MCP proxy configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct McpProxyConfig {
    /// Enable MCP proxy
    #[serde(default = "bool_true")]
    pub enabled: bool,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Retry attempts
    #[serde(default = "default_retry_attempts")]
    pub retry_attempts: u32,
}

fn default_timeout() -> u64 { 30000 }
fn default_retry_attempts() -> u32 { 3 }

impl Default for McpProxyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            timeout_ms: default_timeout(),
            retry_attempts: default_retry_attempts(),
        }
    }
}

/// Collaboration configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollaborationConfig {
    /// Enable collaboration features
    #[serde(default = "bool_true")]
    pub enable_notifications: bool,

    /// Notification transport
    #[serde(default = "default_notification_transport")]
    pub notification_transport: String,

    /// CRDT configuration for collaborative editing
    #[serde(default)]
    pub crdt: CrdtConfig,

    /// File watch configuration
    #[serde(default)]
    pub file_watch: FileWatchConfig,
}

fn default_notification_transport() -> String { "sse".to_string() }

impl Default for CollaborationConfig {
    fn default() -> Self {
        Self {
            enable_notifications: true,
            notification_transport: default_notification_transport(),
            crdt: CrdtConfig::default(),
            file_watch: FileWatchConfig::default(),
        }
    }
}

/// CRDT configuration for collaborative editing
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrdtConfig {
    /// Enable CRDT-based collaborative editing
    #[serde(default = "bool_false")]
    pub enabled: bool,

    /// CRDT algorithm (yjs, automerge, or custom)
    #[serde(default = "default_crdt_algorithm")]
    pub algorithm: String,

    /// Sync interval in milliseconds
    #[serde(default = "default_sync_interval")]
    pub sync_interval_ms: u64,

    /// Conflict resolution strategy
    #[serde(default = "default_conflict_strategy")]
    pub conflict_strategy: String,
}

fn bool_false() -> bool { false }
fn bool_true() -> bool { true }
fn default_crdt_algorithm() -> String { "yjs".to_string() }
fn default_sync_interval() -> u64 { 1000 }
fn default_conflict_strategy() -> String { "last-write-wins".to_string() }

impl Default for CrdtConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            algorithm: default_crdt_algorithm(),
            sync_interval_ms: default_sync_interval(),
            conflict_strategy: default_conflict_strategy(),
        }
    }
}

/// File watch configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileWatchConfig {
    /// Enable file watching
    #[serde(default = "default_file_watch_enabled")]
    pub enabled: bool,

    /// Debounce milliseconds
    #[serde(default = "default_debounce")]
    pub debounce_ms: u64,

    /// Patterns to watch
    #[serde(default = "default_watch_patterns")]
    pub patterns: Vec<String>,
}

fn default_file_watch_enabled() -> bool {
    std::env::var("STRUCTURIZR_ENV")
        .map(|v| v != "production")
        .unwrap_or(true)
}

fn default_debounce() -> u64 { 500 }
fn default_watch_patterns() -> Vec<String> {
    vec!["*.dsl".to_string(), "docs/*.md".to_string(), "adrs/*.md".to_string()]
}

impl Default for FileWatchConfig {
    fn default() -> Self {
        Self {
            enabled: default_file_watch_enabled(),
            debounce_ms: default_debounce(),
            patterns: default_watch_patterns(),
        }
    }
}

/// Profile-specific configuration override
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProfileOverride {
    #[serde(flatten)]
    pub config: PartialConfig,
}

/// Partial config for profile overrides
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PartialConfig {
    pub server: Option<ServerConfig>,
    pub mcp: Option<McpConfig>,
    pub collaboration: Option<CollaborationConfig>,
}

impl Config {
    /// Load configuration from the filesystem
    pub fn load() -> Result<Self, ConfigError> {
        // Find config file
        let path = find_config_file();

        // Load from file or use defaults
        let mut config = if let Some(path) = path {
            tracing::info!("Loading config from: {:?}", path);
            Self::from_file(&path)?
        } else {
            tracing::info!("No config file found, using defaults");
            Self::default()
        };

        // Apply profile overrides
        config.apply_profile()?;

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        // Validate
        config.validate()?;

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn from_file(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Apply profile overrides
    fn apply_profile(&mut self) -> Result<(), ConfigError> {
        let profile = &self.meta.profile;
        if let Some(override_config) = self.profiles.get(profile) {
            tracing::info!("Applying profile: {}", profile);
            // Apply overrides (simplified - in real implementation would merge)
            if let Some(ref server) = override_config.config.server {
                self.server = server.clone();
            }
            if let Some(ref mcp) = override_config.config.mcp {
                self.mcp = mcp.clone();
            }
            if let Some(ref collab) = override_config.config.collaboration {
                self.collaboration = collab.clone();
            }
        }
        Ok(())
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<(), ConfigError> {
        // Check for specific env var overrides
        if let Ok(port) = std::env::var("STRUCTURIZR_PORT") {
            self.server.port = port.parse()
                .map_err(|_| ConfigError::EnvVar("Invalid port".to_string()))?;
        }

        if let Ok(workspaces) = std::env::var("STRUCTURIZR_WORKSPACES_DIR") {
            self.server.workspaces_dir = PathBuf::from(workspaces);
        }

        Ok(())
    }

    /// Validate configuration
    fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_config(self)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            meta: MetaConfig {
                version: default_version(),
                profile: default_profile(),
            },
            server: ServerConfig::default(),
            mcp: McpConfig::default(),
            collaboration: CollaborationConfig::default(),
            profiles: HashMap::new(),
        }
    }
}