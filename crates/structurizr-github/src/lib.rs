//! # structurizr-github
//!
//! GitHub integration for Structurizr - analyze repositories and generate C4 models.
//!
//! This crate provides:
//! - GitHub API client with authentication and rate limiting
//! - Repository cloning and file fetching
//! - Background job processing for analysis
//! - SQLite-based caching for incremental updates
//!
//! ## Usage
//!
//! ```rust,ignore
//! use structurizr_github::{GitHubClient, GitHubConfig, AnalysisJob};
//!
//! // Create a client with a personal access token
//! let config = GitHubConfig::with_token("ghp_xxxxxxxxxxxx");
//! let client = GitHubClient::new(config).await?;
//!
//! // Start an analysis job
//! let job = client.analyze_repository("owner", "repo", "main").await?;
//! ```

pub mod analyzer;
pub mod client;
pub mod config;
pub mod error;
pub mod jobs;
pub mod storage;

pub use analyzer::RepositoryAnalyzer;
pub use client::GitHubClient;
pub use config::GitHubConfig;
pub use error::{Error, Result};
pub use jobs::{AnalysisJob, JobId, JobQueue, JobStatus};
pub use storage::{AnalysisStorage, TokenInfo};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::analyzer::RepositoryAnalyzer;
    pub use crate::client::GitHubClient;
    pub use crate::config::GitHubConfig;
    pub use crate::error::{Error, Result};
    pub use crate::jobs::{AnalysisJob, JobId, JobQueue, JobStatus};
}
