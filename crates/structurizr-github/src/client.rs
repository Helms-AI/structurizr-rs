//! GitHub API client with rate limiting and authentication.

use crate::config::{AuthConfig, GitHubConfig};
use crate::error::{Error, Result};
use octocrab::Octocrab;
use secrecy::ExposeSecret;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// GitHub API client with rate limiting support.
pub struct GitHubClient {
    /// Octocrab instance for API calls
    octocrab: Octocrab,

    /// Configuration
    config: GitHubConfig,

    /// Rate limiter state
    rate_limiter: RateLimiter,
}

/// Rate limiter for GitHub API.
#[derive(Clone)]
pub struct RateLimiter {
    /// Remaining API calls
    remaining: Arc<AtomicU32>,

    /// Reset timestamp (Unix epoch seconds)
    reset_at: Arc<RwLock<u64>>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self {
            remaining: Arc::new(AtomicU32::new(5000)), // Default limit
            reset_at: Arc::new(RwLock::new(0)),
        }
    }
}

impl RateLimiter {
    /// Get remaining API calls.
    pub fn remaining(&self) -> u32 {
        self.remaining.load(Ordering::SeqCst)
    }

    /// Update rate limit from API response headers.
    pub async fn update(&self, remaining: u32, reset_at: u64) {
        self.remaining.store(remaining, Ordering::SeqCst);
        *self.reset_at.write().await = reset_at;
    }

    /// Check if we should wait before making a request.
    pub fn should_wait(&self) -> bool {
        self.remaining.load(Ordering::SeqCst) < 10
    }

    /// Get the reset timestamp.
    pub async fn reset_at(&self) -> u64 {
        *self.reset_at.read().await
    }
}

impl GitHubClient {
    /// Create a new GitHub client with the given configuration.
    pub async fn new(config: GitHubConfig) -> Result<Self> {
        let mut builder = Octocrab::builder();

        // Configure authentication
        match &config.auth {
            AuthConfig::None => {
                debug!("Creating unauthenticated GitHub client");
            }
            AuthConfig::Token(token) => {
                debug!("Creating GitHub client with personal access token");
                builder = builder.personal_token(token.expose_secret().to_string());
            }
            AuthConfig::OAuth(token) => {
                debug!("Creating GitHub client with OAuth token");
                builder = builder.personal_token(token.expose_secret().to_string());
            }
            AuthConfig::App {
                app_id,
                installation_id: _,
                private_key: _,
            } => {
                debug!(
                    "Creating GitHub client with App authentication (app_id: {})",
                    app_id
                );
                // Note: Full GitHub App auth requires additional setup with JWT
                // For now, we'll use the simpler installation token approach
                return Err(Error::Auth(
                    "GitHub App authentication not yet fully implemented".to_string(),
                ));
            }
        }

        // Configure base URL for GitHub Enterprise
        if let Some(base_url) = &config.api_base_url {
            builder = builder.base_uri(base_url).map_err(|e| Error::Config(e.to_string()))?;
        }

        let octocrab = builder.build().map_err(|e| Error::Config(e.to_string()))?;

        Ok(Self {
            octocrab,
            config,
            rate_limiter: RateLimiter::default(),
        })
    }

    /// Get the rate limiter for monitoring API usage.
    pub fn rate_limiter(&self) -> &RateLimiter {
        &self.rate_limiter
    }

    /// Get repository information.
    pub async fn get_repository(&self, owner: &str, repo: &str) -> Result<RepositoryInfo> {
        info!("Fetching repository info: {}/{}", owner, repo);

        let repo_data = self
            .octocrab
            .repos(owner, repo)
            .get()
            .await
            .map_err(|e| match e {
                octocrab::Error::GitHub { source, .. } if source.message.contains("Not Found") => {
                    Error::RepoNotFound {
                        owner: owner.to_string(),
                        repo: repo.to_string(),
                    }
                }
                _ => Error::Api(e.to_string()),
            })?;

        Ok(RepositoryInfo {
            owner: owner.to_string(),
            name: repo.to_string(),
            full_name: format!("{}/{}", owner, repo),
            description: repo_data.description,
            default_branch: repo_data.default_branch.unwrap_or_else(|| "main".to_string()),
            private: repo_data.private.unwrap_or(false),
            clone_url: repo_data
                .clone_url
                .map(|u| u.to_string())
                .unwrap_or_default(),
            html_url: repo_data.html_url.map(|u| u.to_string()),
            language: repo_data.language.and_then(|v| v.as_str().map(String::from)),
        })
    }

    /// Get the file tree for a repository at a specific ref (branch/tag/commit).
    /// This recursively fetches all files in the repository.
    pub async fn get_tree(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
    ) -> Result<Vec<TreeEntry>> {
        info!(
            "Fetching repository tree: {}/{} @ {}",
            owner, repo, ref_name
        );

        // First, get the commit SHA for the ref
        let commit = self
            .octocrab
            .repos(owner, repo)
            .get_ref(&octocrab::params::repos::Reference::Branch(
                ref_name.to_string(),
            ))
            .await
            .map_err(|e| Error::Api(format!("Failed to get ref {}: {}", ref_name, e)))?;

        let sha = match commit.object {
            octocrab::models::repos::Object::Commit { sha, .. } => sha,
            octocrab::models::repos::Object::Tag { sha, .. } => sha,
            _ => return Err(Error::Api("Unexpected ref type".to_string())),
        };

        // Recursively fetch all files starting from root
        let mut all_entries = Vec::new();
        self.fetch_directory_recursive(owner, repo, &sha, "", &mut all_entries).await?;

        info!("Found {} files in repository", all_entries.len());
        Ok(all_entries)
    }

    /// Recursively fetch directory contents.
    fn fetch_directory_recursive<'a>(
        &'a self,
        owner: &'a str,
        repo: &'a str,
        sha: &'a str,
        path: &'a str,
        entries: &'a mut Vec<TreeEntry>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
        Box::pin(async move {
            let contents = self
                .octocrab
                .repos(owner, repo)
                .get_content()
                .path(path)
                .r#ref(sha)
                .send()
                .await
                .map_err(|e| Error::Api(format!("Failed to get contents at '{}': {}", path, e)))?;

            for item in contents.items {
                if item.r#type == "dir" {
                    // Recursively fetch subdirectory
                    self.fetch_directory_recursive(owner, repo, sha, &item.path, entries).await?;
                } else {
                    // Add file entry
                    entries.push(TreeEntry {
                        path: item.path,
                        entry_type: EntryType::File,
                        size: item.size as u64,
                        sha: item.sha,
                    });
                }
            }

            Ok(())
        })
    }

    /// Get the contents of a file.
    pub async fn get_file_content(
        &self,
        owner: &str,
        repo: &str,
        path: &str,
        ref_name: &str,
    ) -> Result<String> {
        debug!("Fetching file: {}/{}/{} @ {}", owner, repo, path, ref_name);

        let content = self
            .octocrab
            .repos(owner, repo)
            .get_content()
            .path(path)
            .r#ref(ref_name)
            .send()
            .await
            .map_err(|e| Error::Api(format!("Failed to get file {}: {}", path, e)))?;

        // For a single file, we get one item in the items vec
        let item = content
            .items
            .into_iter()
            .next()
            .ok_or_else(|| Error::Api(format!("No content returned for: {}", path)))?;

        if item.r#type == "dir" {
            return Err(Error::Api(format!("Expected file, got directory: {}", path)));
        }

        if let Some(file_content) = item.content {
            // Content is base64 encoded
            let decoded = base64_decode(&file_content)?;
            String::from_utf8(decoded)
                .map_err(|e| Error::Api(format!("Invalid UTF-8 in {}: {}", path, e)))
        } else {
            Err(Error::Api(format!("No content in file: {}", path)))
        }
    }

    /// Parse a GitHub URL and extract owner and repo.
    pub fn parse_github_url(url: &str) -> Result<(String, String)> {
        // Handle various GitHub URL formats:
        // - https://github.com/owner/repo
        // - https://github.com/owner/repo.git
        // - git@github.com:owner/repo.git
        // - owner/repo

        let url = url.trim();

        // HTTPS format - check first as it's most common
        if url.starts_with("https://github.com/") {
            let path = url.strip_prefix("https://github.com/").unwrap();
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 2 {
                let repo = parts[1].trim_end_matches(".git");
                return Ok((parts[0].to_string(), repo.to_string()));
            }
        }

        // SSH format - check before simple format
        if url.starts_with("git@github.com:") {
            let path = url.strip_prefix("git@github.com:").unwrap();
            if let Some((owner, repo)) = path.split_once('/') {
                let repo = repo.trim_end_matches(".git");
                return Ok((owner.to_string(), repo.to_string()));
            }
        }

        // Simple owner/repo format - fallback for simple "owner/repo" strings
        if url.chars().filter(|c| *c == '/').count() == 1 && !url.contains(':') {
            if let Some((owner, repo)) = url.split_once('/') {
                let repo = repo.trim_end_matches(".git");
                if !owner.is_empty() && !repo.is_empty() {
                    return Ok((owner.to_string(), repo.to_string()));
                }
            }
        }

        Err(Error::InvalidUrl(format!(
            "Cannot parse GitHub URL: {}",
            url
        )))
    }

    /// Get the cache directory for a repository.
    pub fn cache_path(&self, owner: &str, repo: &str) -> PathBuf {
        self.config.cache_dir.join(owner).join(repo)
    }

    /// Get the configuration.
    pub fn config(&self) -> &GitHubConfig {
        &self.config
    }
}

/// Information about a GitHub repository.
#[derive(Debug, Clone)]
pub struct RepositoryInfo {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub private: bool,
    pub clone_url: String,
    pub html_url: Option<String>,
    pub language: Option<String>,
}

/// Entry in a repository tree.
#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub path: String,
    pub entry_type: EntryType,
    pub size: u64,
    pub sha: String,
}

/// Type of tree entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryType {
    File,
    Directory,
}

/// Decode base64 content (handles GitHub's line-wrapped format).
fn base64_decode(content: &str) -> Result<Vec<u8>> {
    // Remove newlines and whitespace
    let clean: String = content.chars().filter(|c| !c.is_whitespace()).collect();

    // Use a simple base64 decoder
    let mut result = Vec::new();
    let mut buffer = 0u32;
    let mut bits = 0;

    for c in clean.chars() {
        let value = match c {
            'A'..='Z' => c as u32 - 'A' as u32,
            'a'..='z' => c as u32 - 'a' as u32 + 26,
            '0'..='9' => c as u32 - '0' as u32 + 52,
            '+' => 62,
            '/' => 63,
            '=' => continue, // Padding
            _ => {
                return Err(Error::Api(format!(
                    "Invalid base64 character: {}",
                    c
                )))
            }
        };

        buffer = (buffer << 6) | value;
        bits += 6;

        if bits >= 8 {
            bits -= 8;
            result.push((buffer >> bits) as u8);
            buffer &= (1 << bits) - 1;
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url_https() {
        let (owner, repo) = GitHubClient::parse_github_url("https://github.com/rust-lang/cargo")
            .expect("Should parse HTTPS URL");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "cargo");
    }

    #[test]
    fn test_parse_github_url_https_with_git() {
        let (owner, repo) =
            GitHubClient::parse_github_url("https://github.com/rust-lang/cargo.git")
                .expect("Should parse HTTPS URL with .git");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "cargo");
    }

    #[test]
    fn test_parse_github_url_ssh() {
        let (owner, repo) = GitHubClient::parse_github_url("git@github.com:rust-lang/cargo.git")
            .expect("Should parse SSH URL");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "cargo");
    }

    #[test]
    fn test_parse_github_url_simple() {
        let (owner, repo) =
            GitHubClient::parse_github_url("rust-lang/cargo").expect("Should parse simple format");
        assert_eq!(owner, "rust-lang");
        assert_eq!(repo, "cargo");
    }

    #[test]
    fn test_parse_github_url_invalid() {
        let result = GitHubClient::parse_github_url("not-a-valid-url");
        assert!(result.is_err());
    }

    #[test]
    fn test_base64_decode() {
        let encoded = "SGVsbG8gV29ybGQ="; // "Hello World"
        let decoded = base64_decode(encoded).expect("Should decode base64");
        assert_eq!(String::from_utf8(decoded).unwrap(), "Hello World");
    }
}
