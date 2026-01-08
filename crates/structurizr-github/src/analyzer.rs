//! Repository analyzer - integrates GitHub client with code analysis engine.
//!
//! This module coordinates:
//! 1. Downloading repository files from GitHub
//! 2. Running the analysis engine
//! 3. Generating DSL output
//! 4. Saving to the workspaces directory

use crate::client::GitHubClient;
use crate::error::{Error, Result};
use crate::jobs::{AnalysisJob, AnalysisOptions};
use std::fs;
use std::path::{Path, PathBuf};
use structurizr_analysis::{
    analyze_and_generate_dsl, AnalyzerConfig, GeneratorConfig,
};
use tracing::{debug, info, warn};

/// Analyzer that runs on repository contents.
pub struct RepositoryAnalyzer {
    /// GitHub client for fetching files
    client: GitHubClient,

    /// Base directory for workspaces output
    workspaces_dir: PathBuf,

    /// Temporary directory for cloned repos
    temp_dir: PathBuf,
}

impl RepositoryAnalyzer {
    /// Create a new repository analyzer.
    pub fn new(client: GitHubClient, workspaces_dir: impl Into<PathBuf>) -> Self {
        let workspaces_dir = workspaces_dir.into();
        let temp_dir = std::env::temp_dir().join("structurizr-github");

        Self {
            client,
            workspaces_dir,
            temp_dir,
        }
    }

    /// Run analysis on a job.
    ///
    /// This is the main entry point for analyzing a repository:
    /// 1. Downloads/clones the repository to a temp directory
    /// 2. Runs the analysis engine
    /// 3. Generates DSL
    /// 4. Saves to workspaces/{owner}/{repo}/workspace.dsl
    pub async fn analyze(&self, job: &mut AnalysisJob) -> Result<PathBuf> {
        // Clone owner and repo to avoid borrow conflicts with job updates
        let owner = job.owner.clone();
        let repo = job.repo.clone();
        let branch = job.options.branch.clone();
        let options = job.options.clone();

        info!("Starting analysis for {}/{}", owner, repo);
        job.set_progress(0.0, Some("Fetching repository information...".to_string()));

        // Get repository info
        let repo_info = self.client.get_repository(&owner, &repo).await?;
        let ref_name = branch.unwrap_or(repo_info.default_branch.clone());

        job.set_progress(0.1, Some(format!("Using branch: {}", ref_name)));

        // Create temp directory for downloading
        let download_dir = self.temp_dir.join(&owner).join(&repo);
        if download_dir.exists() {
            fs::remove_dir_all(&download_dir)?;
        }
        fs::create_dir_all(&download_dir)?;

        job.set_progress(0.2, Some("Downloading repository files...".to_string()));

        // Download the repository files
        self.download_repository(&owner, &repo, &ref_name, &download_dir)
            .await?;

        job.set_progress(0.5, Some("Analyzing code structure...".to_string()));

        // Run analysis
        let analyzer_config = self.create_analyzer_config(&options);
        let generator_config = self.create_generator_config(&owner, &repo, &repo_info.description);

        let dsl = analyze_and_generate_dsl(&download_dir, &analyzer_config, &generator_config)
            .map_err(|e| Error::Analysis(e.to_string()))?;

        job.set_progress(0.8, Some("Generating workspace files...".to_string()));

        // Save to workspaces directory
        let workspace_path = self.save_workspace(&owner, &repo, &dsl, &repo_info)?;

        job.set_progress(0.9, Some("Cleaning up...".to_string()));

        // Clean up temp directory
        if let Err(e) = fs::remove_dir_all(&download_dir) {
            warn!("Failed to clean up temp directory: {}", e);
        }

        job.complete();
        info!(
            "Analysis complete for {}/{}, workspace saved to: {}",
            owner,
            repo,
            workspace_path.display()
        );

        Ok(workspace_path)
    }

    /// Download repository files from GitHub.
    async fn download_repository(
        &self,
        owner: &str,
        repo: &str,
        ref_name: &str,
        target_dir: &Path,
    ) -> Result<()> {
        info!("Downloading {}/{} @ {} to {}", owner, repo, ref_name, target_dir.display());

        // Get the file tree
        let tree = self.client.get_tree(owner, repo, ref_name).await?;

        // Download each file
        for entry in tree {
            if entry.entry_type == crate::client::EntryType::File {
                let file_path = target_dir.join(&entry.path);

                // Create parent directories
                if let Some(parent) = file_path.parent() {
                    fs::create_dir_all(parent)?;
                }

                // Download file content
                match self
                    .client
                    .get_file_content(owner, repo, &entry.path, ref_name)
                    .await
                {
                    Ok(content) => {
                        fs::write(&file_path, content)?;
                        debug!("Downloaded: {}", entry.path);
                    }
                    Err(e) => {
                        // Some files might be binary or too large - skip them
                        debug!("Skipped {}: {}", entry.path, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Create analyzer configuration from job options.
    fn create_analyzer_config(&self, options: &AnalysisOptions) -> AnalyzerConfig {
        let mut config = AnalyzerConfig::default();

        config.max_depth = options.max_depth;
        config.include_tests = options.include_tests;

        if !options.include_paths.is_empty() {
            config.include_patterns = options.include_paths.clone();
        }

        if !options.exclude_paths.is_empty() {
            config.exclude_patterns = options.exclude_paths.clone();
        }

        config
    }

    /// Create generator configuration.
    fn create_generator_config(
        &self,
        owner: &str,
        repo: &str,
        description: &Option<String>,
    ) -> GeneratorConfig {
        let mut config = GeneratorConfig::default();

        config.workspace_name = Some(format!("{}/{}", owner, repo));
        config.workspace_description = description.clone();

        config
    }

    /// Save the generated workspace to the workspaces directory.
    fn save_workspace(
        &self,
        owner: &str,
        repo: &str,
        dsl: &str,
        repo_info: &crate::client::RepositoryInfo,
    ) -> Result<PathBuf> {
        // Create workspace directory: workspaces/{owner}/{repo}/
        let workspace_dir = self.workspaces_dir.join(owner).join(repo);
        fs::create_dir_all(&workspace_dir)?;

        // Write workspace.dsl
        let dsl_path = workspace_dir.join("workspace.dsl");
        fs::write(&dsl_path, dsl)?;
        info!("Saved workspace DSL to: {}", dsl_path.display());

        // Write .github.json metadata
        let metadata = serde_json::json!({
            "owner": owner,
            "repo": repo,
            "full_name": repo_info.full_name,
            "description": repo_info.description,
            "default_branch": repo_info.default_branch,
            "html_url": repo_info.html_url,
            "language": repo_info.language,
            "analyzed_at": chrono::Utc::now().to_rfc3339(),
        });
        let metadata_path = workspace_dir.join(".github.json");
        fs::write(&metadata_path, serde_json::to_string_pretty(&metadata)?)?;

        Ok(workspace_dir)
    }
}

/// Result of a repository analysis.
#[derive(Debug, Clone)]
pub struct AnalysisResult {
    /// Path to the generated workspace
    pub workspace_path: PathBuf,

    /// Workspace ID (owner/repo)
    pub workspace_id: String,

    /// Generated DSL content
    pub dsl: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_analyzer_config_defaults() {
        let options = AnalysisOptions::default();
        let analyzer = create_test_analyzer_config(&options);

        assert_eq!(analyzer.max_depth, 10);
        assert!(!analyzer.include_tests);
    }

    fn create_test_analyzer_config(options: &AnalysisOptions) -> AnalyzerConfig {
        let mut config = AnalyzerConfig::default();
        config.max_depth = options.max_depth;
        config.include_tests = options.include_tests;
        config
    }
}
