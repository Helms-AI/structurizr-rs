//! Configuration for GitHub integration.

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for GitHub integration.
#[derive(Clone)]
pub struct GitHubConfig {
    /// Authentication method
    pub auth: AuthConfig,

    /// Base URL for GitHub API (for GitHub Enterprise)
    pub api_base_url: Option<String>,

    /// Directory for caching cloned repositories
    pub cache_dir: PathBuf,

    /// Webhook secret for verifying incoming webhooks
    pub webhook_secret: Option<SecretString>,

    /// Maximum concurrent analysis jobs
    pub max_concurrent_jobs: usize,
}

impl Default for GitHubConfig {
    fn default() -> Self {
        Self {
            auth: AuthConfig::None,
            api_base_url: None,
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("structurizr-github"),
            webhook_secret: None,
            max_concurrent_jobs: 3,
        }
    }
}

impl GitHubConfig {
    /// Create a new configuration with a personal access token.
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            auth: AuthConfig::Token(SecretString::new(token.into().into())),
            ..Default::default()
        }
    }

    /// Create a new configuration for GitHub App authentication.
    pub fn with_app(app_id: u64, installation_id: u64, private_key: impl Into<String>) -> Self {
        Self {
            auth: AuthConfig::App {
                app_id,
                installation_id,
                private_key: SecretString::new(private_key.into().into()),
            },
            ..Default::default()
        }
    }

    /// Set the cache directory.
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.cache_dir = path.into();
        self
    }

    /// Set the webhook secret.
    pub fn webhook_secret(mut self, secret: impl Into<String>) -> Self {
        self.webhook_secret = Some(SecretString::new(secret.into().into()));
        self
    }

    /// Set the maximum concurrent jobs.
    pub fn max_concurrent_jobs(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max;
        self
    }

    /// Set a custom API base URL (for GitHub Enterprise).
    pub fn api_base_url(mut self, url: impl Into<String>) -> Self {
        self.api_base_url = Some(url.into());
        self
    }
}

/// Authentication configuration.
#[derive(Clone)]
pub enum AuthConfig {
    /// No authentication (public repositories only)
    None,

    /// Personal access token
    Token(SecretString),

    /// GitHub App authentication
    App {
        app_id: u64,
        installation_id: u64,
        private_key: SecretString,
    },

    /// OAuth token
    OAuth(SecretString),
}

impl AuthConfig {
    /// Check if authentication is configured.
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, AuthConfig::None)
    }

    /// Get the token for API requests (if using token auth).
    pub fn token(&self) -> Option<&str> {
        match self {
            AuthConfig::Token(token) => Some(token.expose_secret()),
            AuthConfig::OAuth(token) => Some(token.expose_secret()),
            _ => None,
        }
    }
}

/// Serializable version of config for storage (without secrets).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredConfig {
    /// Whether authentication is configured
    pub has_auth: bool,

    /// API base URL
    pub api_base_url: Option<String>,

    /// Cache directory path
    pub cache_dir: PathBuf,

    /// Maximum concurrent jobs
    pub max_concurrent_jobs: usize,
}

impl From<&GitHubConfig> for StoredConfig {
    fn from(config: &GitHubConfig) -> Self {
        Self {
            has_auth: config.auth.is_authenticated(),
            api_base_url: config.api_base_url.clone(),
            cache_dir: config.cache_dir.clone(),
            max_concurrent_jobs: config.max_concurrent_jobs,
        }
    }
}

// Helper for dirs crate
mod dirs {
    use std::path::PathBuf;

    pub fn cache_dir() -> Option<PathBuf> {
        #[cfg(target_os = "macos")]
        {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Caches"))
        }
        #[cfg(target_os = "linux")]
        {
            std::env::var_os("XDG_CACHE_HOME")
                .map(PathBuf::from)
                .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        }
        #[cfg(target_os = "windows")]
        {
            std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            None
        }
    }
}
