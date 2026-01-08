//! Error types for GitHub integration.

use thiserror::Error;

/// Result type alias for GitHub operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur during GitHub operations.
#[derive(Debug, Error)]
pub enum Error {
    /// GitHub API error
    #[error("GitHub API error: {0}")]
    Api(String),

    /// Authentication error
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Rate limit exceeded
    #[error("GitHub API rate limit exceeded. Resets at {reset_at}")]
    RateLimited { reset_at: String },

    /// Repository not found or inaccessible
    #[error("Repository not found: {owner}/{repo}")]
    RepoNotFound { owner: String, repo: String },

    /// Invalid repository URL
    #[error("Invalid GitHub URL: {0}")]
    InvalidUrl(String),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Storage/database error
    #[error("Storage error: {0}")]
    Storage(String),

    /// Job processing error
    #[error("Job error: {0}")]
    Job(String),

    /// File system error
    #[error("File system error: {0}")]
    FileSystem(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Analysis error
    #[error("Analysis error: {0}")]
    Analysis(String),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(err.to_string())
    }
}

impl From<octocrab::Error> for Error {
    fn from(err: octocrab::Error) -> Self {
        Error::Api(err.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Error::Storage(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::FileSystem(err.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Storage(format!("JSON error: {}", err))
    }
}
