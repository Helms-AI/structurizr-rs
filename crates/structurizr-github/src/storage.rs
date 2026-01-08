//! SQLite-based storage for analysis results and caching.

use crate::error::Result;
use crate::jobs::{AnalysisJob, JobStatus};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, info};

/// Storage for analysis results and job history.
pub struct AnalysisStorage {
    /// Database connection (wrapped for async safety)
    conn: Arc<Mutex<Connection>>,

    /// Path to the database file (for future use, e.g., file-based operations)
    #[allow(dead_code)]
    db_path: PathBuf,
}

impl AnalysisStorage {
    /// Open or create a storage database at the given path.
    pub async fn open(path: impl AsRef<Path>) -> Result<Self> {
        let db_path = path.as_ref().to_path_buf();

        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Initialize schema
        conn.execute_batch(SCHEMA)?;

        info!("Opened analysis storage at: {}", db_path.display());

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        })
    }

    /// Open an in-memory database (for testing).
    pub async fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
        })
    }

    /// Save or update a repository connection.
    pub async fn save_repository(&self, repo: &RepositoryRecord) -> Result<i64> {
        let conn = self.conn.lock().await;

        conn.execute(
            r#"
            INSERT INTO repositories (url, owner, repo, workspace_id, branch, last_commit_sha, last_analyzed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(url) DO UPDATE SET
                workspace_id = excluded.workspace_id,
                branch = excluded.branch,
                last_commit_sha = excluded.last_commit_sha,
                last_analyzed_at = excluded.last_analyzed_at
            "#,
            params![
                repo.url,
                repo.owner,
                repo.repo,
                repo.workspace_id,
                repo.branch,
                repo.last_commit_sha,
                repo.last_analyzed_at.map(|dt| dt.to_rfc3339()),
            ],
        )?;

        let id = conn.last_insert_rowid();
        debug!("Saved repository: {} (id: {})", repo.url, id);

        Ok(id)
    }

    /// Get a repository by URL.
    pub async fn get_repository(&self, url: &str) -> Result<Option<RepositoryRecord>> {
        let conn = self.conn.lock().await;

        let result = conn
            .query_row(
                "SELECT id, url, owner, repo, workspace_id, branch, last_commit_sha, last_analyzed_at FROM repositories WHERE url = ?1",
                params![url],
                |row| {
                    Ok(RepositoryRecord {
                        id: Some(row.get(0)?),
                        url: row.get(1)?,
                        owner: row.get(2)?,
                        repo: row.get(3)?,
                        workspace_id: row.get(4)?,
                        branch: row.get(5)?,
                        last_commit_sha: row.get(6)?,
                        last_analyzed_at: row.get::<_, Option<String>>(7)?
                            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                            .map(|dt| dt.with_timezone(&Utc)),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Get a repository by workspace ID.
    pub async fn get_repository_by_workspace(&self, workspace_id: &str) -> Result<Option<RepositoryRecord>> {
        let conn = self.conn.lock().await;

        let result = conn
            .query_row(
                "SELECT id, url, owner, repo, workspace_id, branch, last_commit_sha, last_analyzed_at FROM repositories WHERE workspace_id = ?1",
                params![workspace_id],
                |row| {
                    Ok(RepositoryRecord {
                        id: Some(row.get(0)?),
                        url: row.get(1)?,
                        owner: row.get(2)?,
                        repo: row.get(3)?,
                        workspace_id: row.get(4)?,
                        branch: row.get(5)?,
                        last_commit_sha: row.get(6)?,
                        last_analyzed_at: row.get::<_, Option<String>>(7)?
                            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                            .map(|dt| dt.with_timezone(&Utc)),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// List all repositories.
    pub async fn list_repositories(&self) -> Result<Vec<RepositoryRecord>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT id, url, owner, repo, workspace_id, branch, last_commit_sha, last_analyzed_at FROM repositories ORDER BY last_analyzed_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(RepositoryRecord {
                id: Some(row.get(0)?),
                url: row.get(1)?,
                owner: row.get(2)?,
                repo: row.get(3)?,
                workspace_id: row.get(4)?,
                branch: row.get(5)?,
                last_commit_sha: row.get(6)?,
                last_analyzed_at: row.get::<_, Option<String>>(7)?
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
            })
        })?;

        let mut repos = Vec::new();
        for row in rows {
            repos.push(row?);
        }

        Ok(repos)
    }

    /// Save a file cache entry.
    pub async fn save_file_cache(&self, entry: &FileCacheEntry) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            r#"
            INSERT INTO file_cache (repo_id, file_path, file_hash, analysis_result, analyzed_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(repo_id, file_path) DO UPDATE SET
                file_hash = excluded.file_hash,
                analysis_result = excluded.analysis_result,
                analyzed_at = excluded.analyzed_at
            "#,
            params![
                entry.repo_id,
                entry.file_path,
                entry.file_hash,
                entry.analysis_result,
                entry.analyzed_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Get a cached file analysis.
    pub async fn get_file_cache(&self, repo_id: i64, file_path: &str) -> Result<Option<FileCacheEntry>> {
        let conn = self.conn.lock().await;

        let result = conn
            .query_row(
                "SELECT id, repo_id, file_path, file_hash, analysis_result, analyzed_at FROM file_cache WHERE repo_id = ?1 AND file_path = ?2",
                params![repo_id, file_path],
                |row| {
                    Ok(FileCacheEntry {
                        id: Some(row.get(0)?),
                        repo_id: row.get(1)?,
                        file_path: row.get(2)?,
                        file_hash: row.get(3)?,
                        analysis_result: row.get(4)?,
                        analyzed_at: row.get::<_, String>(5)?
                            .parse::<DateTime<Utc>>()
                            .unwrap_or_else(|_| Utc::now()),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Check if a file needs re-analysis based on hash.
    pub async fn needs_reanalysis(&self, repo_id: i64, file_path: &str, current_hash: &str) -> Result<bool> {
        if let Some(cached) = self.get_file_cache(repo_id, file_path).await? {
            Ok(cached.file_hash != current_hash)
        } else {
            Ok(true) // Not cached, needs analysis
        }
    }

    /// Delete all cache entries for a repository.
    pub async fn clear_cache(&self, repo_id: i64) -> Result<usize> {
        let conn = self.conn.lock().await;
        let count = conn.execute("DELETE FROM file_cache WHERE repo_id = ?1", params![repo_id])?;
        Ok(count)
    }

    /// Save a job record.
    pub async fn save_job(&self, job: &AnalysisJob) -> Result<()> {
        let conn = self.conn.lock().await;

        let status_json = serde_json::to_string(&job.status)?;
        let options_json = serde_json::to_string(&job.options)?;

        conn.execute(
            r#"
            INSERT INTO analysis_jobs (id, owner, repo, workspace_id, status, options, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                updated_at = excluded.updated_at
            "#,
            params![
                job.id.to_string(),
                job.owner,
                job.repo,
                job.workspace_id,
                status_json,
                options_json,
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
            ],
        )?;

        Ok(())
    }

    /// Get recent jobs.
    pub async fn get_recent_jobs(&self, limit: usize) -> Result<Vec<AnalysisJob>> {
        let conn = self.conn.lock().await;

        let mut stmt = conn.prepare(
            "SELECT id, owner, repo, workspace_id, status, options, created_at, updated_at FROM analysis_jobs ORDER BY created_at DESC LIMIT ?1",
        )?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let id_str: String = row.get(0)?;
            let status_json: String = row.get(4)?;
            let options_json: String = row.get(5)?;
            let created_str: String = row.get(6)?;
            let updated_str: String = row.get(7)?;

            Ok((
                id_str,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                status_json,
                options_json,
                created_str,
                updated_str,
            ))
        })?;

        let mut jobs = Vec::new();
        for row in rows {
            let (id_str, owner, repo, workspace_id, status_json, options_json, created_str, updated_str) = row?;

            jobs.push(AnalysisJob {
                id: id_str.parse().unwrap_or_default(),
                owner,
                repo,
                workspace_id,
                status: serde_json::from_str(&status_json).unwrap_or(JobStatus::Queued),
                options: serde_json::from_str(&options_json).unwrap_or_default(),
                created_at: DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
                updated_at: DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            });
        }

        Ok(jobs)
    }

    // ==================== Token Storage ====================

    /// Save or update a GitHub personal access token.
    /// The token is base64 encoded before storage for basic obfuscation.
    pub async fn save_github_token(&self, token: &SecretString, description: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().await;

        // Base64 encode the token for obfuscation
        let encoded = base64_encode(token.expose_secret());

        conn.execute(
            r#"
            INSERT INTO auth_tokens (provider, token_type, token_value, description, created_at, last_used_at)
            VALUES ('github', 'personal_access_token', ?1, ?2, ?3, ?3)
            ON CONFLICT(provider, token_type) DO UPDATE SET
                token_value = excluded.token_value,
                description = excluded.description,
                last_used_at = excluded.last_used_at
            "#,
            params![
                encoded,
                description,
                Utc::now().to_rfc3339(),
            ],
        )?;

        info!("Saved GitHub personal access token");
        Ok(())
    }

    /// Get the stored GitHub personal access token.
    /// Returns None if no token is stored.
    pub async fn get_github_token(&self) -> Result<Option<SecretString>> {
        let conn = self.conn.lock().await;

        let result = conn
            .query_row(
                "SELECT token_value FROM auth_tokens WHERE provider = 'github' AND token_type = 'personal_access_token'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        if let Some(encoded) = result {
            // Base64 decode the token
            let decoded = base64_decode_string(&encoded)?;
            Ok(Some(SecretString::from(decoded)))
        } else {
            Ok(None)
        }
    }

    /// Check if a GitHub token is stored.
    pub async fn has_github_token(&self) -> Result<bool> {
        let conn = self.conn.lock().await;

        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM auth_tokens WHERE provider = 'github' AND token_type = 'personal_access_token'",
            [],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    /// Get token metadata without the actual token value.
    pub async fn get_github_token_info(&self) -> Result<Option<TokenInfo>> {
        let conn = self.conn.lock().await;

        let result = conn
            .query_row(
                "SELECT description, created_at, last_used_at FROM auth_tokens WHERE provider = 'github' AND token_type = 'personal_access_token'",
                [],
                |row| {
                    Ok(TokenInfo {
                        provider: "github".to_string(),
                        token_type: "personal_access_token".to_string(),
                        description: row.get(0)?,
                        created_at: row.get::<_, String>(1)?
                            .parse::<DateTime<Utc>>()
                            .ok(),
                        last_used_at: row.get::<_, Option<String>>(2)?
                            .and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Update the last_used_at timestamp for the GitHub token.
    pub async fn touch_github_token(&self) -> Result<()> {
        let conn = self.conn.lock().await;

        conn.execute(
            "UPDATE auth_tokens SET last_used_at = ?1 WHERE provider = 'github' AND token_type = 'personal_access_token'",
            params![Utc::now().to_rfc3339()],
        )?;

        Ok(())
    }

    /// Delete the stored GitHub token.
    pub async fn delete_github_token(&self) -> Result<bool> {
        let conn = self.conn.lock().await;

        let deleted = conn.execute(
            "DELETE FROM auth_tokens WHERE provider = 'github' AND token_type = 'personal_access_token'",
            [],
        )?;

        if deleted > 0 {
            info!("Deleted GitHub personal access token");
        }

        Ok(deleted > 0)
    }
}

/// Metadata about a stored token (without the actual token value).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub provider: String,
    pub token_type: String,
    pub description: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Simple base64 encoding for token obfuscation.
fn base64_encode(input: &str) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let bytes = input.as_bytes();
    let mut result = String::new();

    for chunk in bytes.chunks(3) {
        let n = match chunk.len() {
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            1 => (chunk[0] as u32) << 16,
            _ => unreachable!(),
        };

        result.push(CHARS[((n >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((n >> 12) & 0x3F) as usize] as char);

        if chunk.len() > 1 {
            result.push(CHARS[((n >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(n & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

/// Simple base64 decoding for token deobfuscation.
fn base64_decode_string(input: &str) -> Result<String> {
    let bytes = base64_decode_bytes(input)?;
    String::from_utf8(bytes).map_err(|e| crate::error::Error::Api(format!("Invalid UTF-8 in token: {}", e)))
}

fn base64_decode_bytes(input: &str) -> Result<Vec<u8>> {
    let clean: String = input.chars().filter(|c| !c.is_whitespace()).collect();

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
            '=' => continue,
            _ => {
                return Err(crate::error::Error::Api(format!(
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

/// Database schema.
const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS repositories (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT UNIQUE NOT NULL,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    branch TEXT,
    last_commit_sha TEXT,
    last_analyzed_at TEXT
);

CREATE TABLE IF NOT EXISTS file_cache (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo_id INTEGER NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    analysis_result TEXT,
    analyzed_at TEXT NOT NULL,
    UNIQUE(repo_id, file_path)
);

CREATE INDEX IF NOT EXISTS idx_file_cache_repo ON file_cache(repo_id);

CREATE TABLE IF NOT EXISTS analysis_jobs (
    id TEXT PRIMARY KEY,
    owner TEXT NOT NULL,
    repo TEXT NOT NULL,
    workspace_id TEXT NOT NULL,
    status TEXT NOT NULL,
    options TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_jobs_created ON analysis_jobs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_jobs_workspace ON analysis_jobs(workspace_id);

-- Token storage for GitHub authentication
-- Tokens are base64 encoded for basic obfuscation
-- Note: For production use, consider additional encryption
CREATE TABLE IF NOT EXISTS auth_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL DEFAULT 'github',
    token_type TEXT NOT NULL DEFAULT 'personal_access_token',
    token_value TEXT NOT NULL,
    description TEXT,
    created_at TEXT NOT NULL,
    last_used_at TEXT,
    UNIQUE(provider, token_type)
);
"#;

/// Repository record in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRecord {
    pub id: Option<i64>,
    pub url: String,
    pub owner: String,
    pub repo: String,
    pub workspace_id: String,
    pub branch: Option<String>,
    pub last_commit_sha: Option<String>,
    pub last_analyzed_at: Option<DateTime<Utc>>,
}

/// File cache entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCacheEntry {
    pub id: Option<i64>,
    pub repo_id: i64,
    pub file_path: String,
    pub file_hash: String,
    pub analysis_result: Option<String>,
    pub analyzed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_storage_basic() {
        let storage = AnalysisStorage::open_in_memory()
            .await
            .expect("Should create in-memory storage");

        let repo = RepositoryRecord {
            id: None,
            url: "https://github.com/rust-lang/cargo".to_string(),
            owner: "rust-lang".to_string(),
            repo: "cargo".to_string(),
            workspace_id: "rust-lang/cargo".to_string(),
            branch: Some("main".to_string()),
            last_commit_sha: None,
            last_analyzed_at: None,
        };

        let id = storage.save_repository(&repo).await.expect("Should save");
        assert!(id > 0);

        let loaded = storage
            .get_repository(&repo.url)
            .await
            .expect("Should query")
            .expect("Should find");

        assert_eq!(loaded.owner, "rust-lang");
        assert_eq!(loaded.repo, "cargo");
    }

    #[tokio::test]
    async fn test_file_cache() {
        let storage = AnalysisStorage::open_in_memory()
            .await
            .expect("Should create storage");

        // First save a repository
        let repo = RepositoryRecord {
            id: None,
            url: "https://github.com/test/repo".to_string(),
            owner: "test".to_string(),
            repo: "repo".to_string(),
            workspace_id: "test/repo".to_string(),
            branch: Some("main".to_string()),
            last_commit_sha: None,
            last_analyzed_at: None,
        };

        let repo_id = storage.save_repository(&repo).await.expect("Should save repo");

        // Save a cache entry
        let entry = FileCacheEntry {
            id: None,
            repo_id,
            file_path: "src/main.rs".to_string(),
            file_hash: "abc123".to_string(),
            analysis_result: Some("{}".to_string()),
            analyzed_at: Utc::now(),
        };

        storage.save_file_cache(&entry).await.expect("Should save cache");

        // Check if reanalysis is needed
        assert!(!storage.needs_reanalysis(repo_id, "src/main.rs", "abc123").await.unwrap());
        assert!(storage.needs_reanalysis(repo_id, "src/main.rs", "different").await.unwrap());
        assert!(storage.needs_reanalysis(repo_id, "src/other.rs", "abc123").await.unwrap());
    }

    #[tokio::test]
    async fn test_github_token_storage() {
        let storage = AnalysisStorage::open_in_memory()
            .await
            .expect("Should create storage");

        // Initially no token should be stored
        assert!(!storage.has_github_token().await.unwrap());
        assert!(storage.get_github_token().await.unwrap().is_none());

        // Save a token
        let token = SecretString::from("ghp_test123456789");
        storage
            .save_github_token(&token, Some("Test token"))
            .await
            .expect("Should save token");

        // Token should now exist
        assert!(storage.has_github_token().await.unwrap());

        // Retrieve and verify token
        let retrieved = storage
            .get_github_token()
            .await
            .expect("Should query")
            .expect("Should find token");
        assert_eq!(retrieved.expose_secret(), "ghp_test123456789");

        // Check token info
        let info = storage
            .get_github_token_info()
            .await
            .expect("Should query info")
            .expect("Should find info");
        assert_eq!(info.provider, "github");
        assert_eq!(info.description, Some("Test token".to_string()));

        // Update token
        let new_token = SecretString::from("ghp_newtoken999");
        storage
            .save_github_token(&new_token, Some("Updated token"))
            .await
            .expect("Should update token");

        let retrieved = storage.get_github_token().await.unwrap().unwrap();
        assert_eq!(retrieved.expose_secret(), "ghp_newtoken999");

        // Delete token
        assert!(storage.delete_github_token().await.unwrap());
        assert!(!storage.has_github_token().await.unwrap());
    }

    #[test]
    fn test_base64_roundtrip() {
        let original = "ghp_test123456789ABCDEFxyz";
        let encoded = base64_encode(original);
        let decoded = base64_decode_string(&encoded).unwrap();
        assert_eq!(original, decoded);
    }
}
