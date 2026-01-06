//! Workspace management tools for MCP

use serde::{Deserialize, Serialize};
use structurizr_core::Workspace;

use crate::error::Result;
use crate::state::McpServerState;

/// Input for listing workspaces
#[derive(Debug, Deserialize)]
pub struct ListWorkspacesInput {
    pub recursive: Option<bool>,
}

/// Output for listing workspaces
#[derive(Debug, Serialize)]
pub struct ListWorkspacesOutput {
    pub workspaces: Vec<WorkspaceInfo>,
}

/// Information about a workspace
#[derive(Debug, Serialize)]
pub struct WorkspaceInfo {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub view_count: usize,
}

/// List all available workspaces
pub async fn list_workspaces(
    state: &McpServerState,
    _input: ListWorkspacesInput,
) -> Result<ListWorkspacesOutput> {
    let workspaces = state.list_workspaces().await?;

    let workspace_infos = workspaces
        .into_iter()
        .map(|w| WorkspaceInfo {
            id: w.id,
            name: w.name,
            description: w.description,
            path: w.path.to_string_lossy().to_string(),
            view_count: w.view_count,
        })
        .collect();

    Ok(ListWorkspacesOutput {
        workspaces: workspace_infos,
    })
}

/// Input for loading a workspace
#[derive(Debug, Deserialize)]
pub struct LoadWorkspaceInput {
    pub workspace_id: String,
}

/// Output for loading a workspace
#[derive(Debug, Serialize)]
pub struct LoadWorkspaceOutput {
    pub workspace: Workspace,
    pub message: String,
}

/// Load a workspace by ID
pub async fn load_workspace(
    state: &McpServerState,
    input: LoadWorkspaceInput,
) -> Result<LoadWorkspaceOutput> {
    let workspace = state
        .get_workspace_by_id(&input.workspace_id)
        .await
        .ok_or_else(|| {
            crate::error::McpError::WorkspaceNotFound(input.workspace_id.clone())
        })?;

    state.set_active_workspace(input.workspace_id.clone()).await;

    Ok(LoadWorkspaceOutput {
        message: format!("Loaded workspace: {}", workspace.name),
        workspace,
    })
}

/// Input for validating a workspace
#[derive(Debug, Deserialize)]
pub struct ValidateWorkspaceInput {
    pub workspace_id: String,
}

/// Output for validating a workspace
#[derive(Debug, Serialize)]
pub struct ValidateWorkspaceOutput {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub hints: Vec<String>,
}

/// Validate a workspace
pub async fn validate_workspace(
    state: &McpServerState,
    input: ValidateWorkspaceInput,
) -> Result<ValidateWorkspaceOutput> {
    let workspace = state
        .get_workspace_by_id(&input.workspace_id)
        .await
        .ok_or_else(|| {
            crate::error::McpError::WorkspaceNotFound(input.workspace_id.clone())
        })?;

    let validation_result = structurizr_dsl::validate_workspace(&workspace);

    // Extract issues by severity
    let errors: Vec<String> = validation_result.issues
        .iter()
        .filter(|i| matches!(i.severity, structurizr_dsl::Severity::Error))
        .map(|i| i.message.clone())
        .collect();

    let warnings: Vec<String> = validation_result.issues
        .iter()
        .filter(|i| matches!(i.severity, structurizr_dsl::Severity::Warning))
        .map(|i| i.message.clone())
        .collect();

    let hints: Vec<String> = validation_result.issues
        .iter()
        .filter(|i| matches!(i.severity, structurizr_dsl::Severity::Info))
        .map(|i| i.message.clone())
        .collect();

    Ok(ValidateWorkspaceOutput {
        valid: validation_result.error_count == 0,
        errors,
        warnings,
        hints,
    })
}