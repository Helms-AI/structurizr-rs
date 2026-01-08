//! Background job processing for GitHub analysis.

use crate::error::{Error, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for an analysis job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
    /// Create a new random job ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Get the UUID.
    pub fn uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for JobId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::str::FromStr for JobId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Self(Uuid::parse_str(s)?))
    }
}

/// Status of an analysis job.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum JobStatus {
    /// Job is queued and waiting to be processed.
    Queued,

    /// Job is currently running.
    Running {
        /// Progress from 0.0 to 1.0.
        progress: f32,
        /// Current operation message.
        message: Option<String>,
    },

    /// Job completed successfully.
    Completed {
        /// ID of the created workspace.
        workspace_id: String,
        /// Timestamp when completed.
        completed_at: DateTime<Utc>,
    },

    /// Job failed with an error.
    Failed {
        /// Error message.
        error: String,
        /// Timestamp when failed.
        failed_at: DateTime<Utc>,
    },

    /// Job was cancelled.
    Cancelled {
        /// Timestamp when cancelled.
        cancelled_at: DateTime<Utc>,
    },
}

impl JobStatus {
    /// Check if the job is still active (queued or running).
    pub fn is_active(&self) -> bool {
        matches!(self, JobStatus::Queued | JobStatus::Running { .. })
    }

    /// Check if the job is complete (success, failure, or cancelled).
    pub fn is_complete(&self) -> bool {
        !self.is_active()
    }

    /// Get the status name as a string.
    pub fn name(&self) -> &'static str {
        match self {
            JobStatus::Queued => "queued",
            JobStatus::Running { .. } => "running",
            JobStatus::Completed { .. } => "completed",
            JobStatus::Failed { .. } => "failed",
            JobStatus::Cancelled { .. } => "cancelled",
        }
    }
}

/// Options for repository analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisOptions {
    /// Branch to analyze (default: repository's default branch).
    pub branch: Option<String>,

    /// Include test files in analysis.
    #[serde(default)]
    pub include_tests: bool,

    /// Maximum directory depth to analyze.
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,

    /// Languages to analyze (empty = all detected).
    #[serde(default)]
    pub languages: Vec<String>,

    /// Paths to include (glob patterns).
    #[serde(default)]
    pub include_paths: Vec<String>,

    /// Paths to exclude (glob patterns).
    #[serde(default)]
    pub exclude_paths: Vec<String>,
}

fn default_max_depth() -> usize {
    10
}

impl Default for AnalysisOptions {
    fn default() -> Self {
        Self {
            branch: None,
            include_tests: false,
            max_depth: default_max_depth(),
            languages: Vec::new(),
            include_paths: Vec::new(),
            exclude_paths: vec![
                "node_modules/**".to_string(),
                "target/**".to_string(),
                "vendor/**".to_string(),
                ".git/**".to_string(),
                "dist/**".to_string(),
                "build/**".to_string(),
            ],
        }
    }
}

/// An analysis job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisJob {
    /// Unique job ID.
    pub id: JobId,

    /// Repository owner.
    pub owner: String,

    /// Repository name.
    pub repo: String,

    /// Target workspace ID (will be created at workspaces/{owner}/{repo}).
    pub workspace_id: String,

    /// Analysis options.
    pub options: AnalysisOptions,

    /// Current status.
    pub status: JobStatus,

    /// When the job was created.
    pub created_at: DateTime<Utc>,

    /// When the job was last updated.
    pub updated_at: DateTime<Utc>,
}

impl AnalysisJob {
    /// Create a new analysis job.
    pub fn new(owner: impl Into<String>, repo: impl Into<String>, options: AnalysisOptions) -> Self {
        let owner = owner.into();
        let repo = repo.into();
        let workspace_id = format!("{}/{}", owner, repo);
        let now = Utc::now();

        Self {
            id: JobId::new(),
            owner,
            repo,
            workspace_id,
            options,
            status: JobStatus::Queued,
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the job status.
    pub fn update_status(&mut self, status: JobStatus) {
        self.status = status;
        self.updated_at = Utc::now();
    }

    /// Set progress for a running job.
    pub fn set_progress(&mut self, progress: f32, message: Option<String>) {
        self.status = JobStatus::Running { progress, message };
        self.updated_at = Utc::now();
    }

    /// Mark the job as completed.
    pub fn complete(&mut self) {
        self.status = JobStatus::Completed {
            workspace_id: self.workspace_id.clone(),
            completed_at: Utc::now(),
        };
        self.updated_at = Utc::now();
    }

    /// Mark the job as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = JobStatus::Failed {
            error: error.into(),
            failed_at: Utc::now(),
        };
        self.updated_at = Utc::now();
    }

    /// Mark the job as cancelled.
    pub fn cancel(&mut self) {
        self.status = JobStatus::Cancelled {
            cancelled_at: Utc::now(),
        };
        self.updated_at = Utc::now();
    }
}

/// Queue for managing analysis jobs.
#[derive(Clone)]
pub struct JobQueue {
    /// Pending jobs.
    queue: Arc<RwLock<VecDeque<AnalysisJob>>>,

    /// All jobs (including completed).
    jobs: Arc<RwLock<HashMap<JobId, AnalysisJob>>>,

    /// Maximum concurrent jobs (for future use).
    #[allow(dead_code)]
    max_concurrent: usize,
}

impl JobQueue {
    /// Create a new job queue.
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(RwLock::new(VecDeque::new())),
            jobs: Arc::new(RwLock::new(HashMap::new())),
            max_concurrent,
        }
    }

    /// Enqueue a new job.
    pub async fn enqueue(&self, job: AnalysisJob) -> JobId {
        let id = job.id;

        {
            let mut jobs = self.jobs.write().await;
            jobs.insert(id, job.clone());
        }

        {
            let mut queue = self.queue.write().await;
            queue.push_back(job);
        }

        id
    }

    /// Dequeue the next pending job.
    pub async fn dequeue(&self) -> Option<AnalysisJob> {
        let mut queue = self.queue.write().await;
        queue.pop_front()
    }

    /// Get a job by ID.
    pub async fn get(&self, id: JobId) -> Option<AnalysisJob> {
        let jobs = self.jobs.read().await;
        jobs.get(&id).cloned()
    }

    /// Update a job's status.
    pub async fn update(&self, id: JobId, status: JobStatus) -> Result<()> {
        let mut jobs = self.jobs.write().await;
        if let Some(job) = jobs.get_mut(&id) {
            job.update_status(status);
            Ok(())
        } else {
            Err(Error::Job(format!("Job not found: {}", id)))
        }
    }

    /// List all jobs, optionally filtered by status.
    pub async fn list(&self, status_filter: Option<&str>) -> Vec<AnalysisJob> {
        let jobs = self.jobs.read().await;
        jobs.values()
            .filter(|job| {
                status_filter
                    .map(|s| job.status.name() == s)
                    .unwrap_or(true)
            })
            .cloned()
            .collect()
    }

    /// Get the number of pending jobs.
    pub async fn pending_count(&self) -> usize {
        let queue = self.queue.read().await;
        queue.len()
    }

    /// Get the number of active jobs (queued + running).
    pub async fn active_count(&self) -> usize {
        let jobs = self.jobs.read().await;
        jobs.values().filter(|j| j.status.is_active()).count()
    }

    /// Cancel a job.
    pub async fn cancel(&self, id: JobId) -> Result<()> {
        // Remove from queue if pending
        {
            let mut queue = self.queue.write().await;
            queue.retain(|j| j.id != id);
        }

        // Update status
        self.update(id, JobStatus::Cancelled {
            cancelled_at: Utc::now(),
        })
        .await
    }

    /// Clean up old completed jobs (older than the given duration).
    pub async fn cleanup(&self, max_age: chrono::Duration) {
        let cutoff = Utc::now() - max_age;
        let mut jobs = self.jobs.write().await;
        jobs.retain(|_, job| {
            job.status.is_active() || job.updated_at > cutoff
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_job_queue_basic() {
        let queue = JobQueue::new(3);

        let job = AnalysisJob::new("rust-lang", "cargo", AnalysisOptions::default());
        let id = queue.enqueue(job).await;

        assert_eq!(queue.pending_count().await, 1);

        let dequeued = queue.dequeue().await.expect("Should have a job");
        assert_eq!(dequeued.id, id);

        assert_eq!(queue.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_job_status_update() {
        let queue = JobQueue::new(3);

        let job = AnalysisJob::new("owner", "repo", AnalysisOptions::default());
        let id = queue.enqueue(job).await;

        queue.update(id, JobStatus::Running {
            progress: 0.5,
            message: Some("Processing...".to_string()),
        }).await.expect("Should update");

        let updated = queue.get(id).await.expect("Should find job");
        assert!(matches!(updated.status, JobStatus::Running { .. }));
    }
}
