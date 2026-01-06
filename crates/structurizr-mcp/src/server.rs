//! MCP server implementation for structurizr-rs using rmcp

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::*,
    tool, tool_handler, tool_router,
    transport::stdio,
    ErrorData as RmcpError,
    ServiceExt,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info};

use structurizr_core::model::{Component, Container, ElementId, Person, Relationship, SoftwareSystem};
use structurizr_core::view::{
    AutoLayout, AutoLayoutDirection, ComponentView, ContainerView, DeploymentView,
    DynamicView, ElementView, SystemContextView, SystemLandscapeView,
};
use structurizr_core::workspace::{Decision, DecisionStatus, DocumentationFormat, DocumentationSection};
use structurizr_core::Workspace;
use structurizr_dsl::serialize as dsl_serialize;
use structurizr_export::{
    D2Exporter, DotExporter, JsonExporter, MermaidExporter, PlantUmlExporter,
};
use structurizr_render::SvgRenderer;
use structurizr_web::discovery::WorkspaceRegistry;

// ============================================================================
// Parameter types for tools (derive JsonSchema for automatic schema generation)
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceIdParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceSearchParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
    #[schemars(description = "The search query to find elements by name or description")]
    pub query: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RenderViewParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
    #[schemars(description = "The key of the view to render")]
    pub view_key: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ExportFormatParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
    #[schemars(description = "Export format: json, plantuml, mermaid, dot, d2")]
    pub format: String,
}

// ============================================================================
// Parameter types for model manipulation tools
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddPersonParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The name of the person to add")]
    pub name: String,
    #[schemars(description = "Optional description of the person")]
    pub description: Option<String>,
    #[schemars(description = "Whether the person is external to the organization")]
    #[serde(default)]
    pub external: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddSoftwareSystemParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The name of the software system to add")]
    pub name: String,
    #[schemars(description = "Optional description of the software system")]
    pub description: Option<String>,
    #[schemars(description = "Whether the system is external to the organization")]
    #[serde(default)]
    pub external: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddContainerParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The name of the software system to add the container to")]
    pub system_name: String,
    #[schemars(description = "The name of the container to add")]
    pub name: String,
    #[schemars(description = "Optional description of the container")]
    pub description: Option<String>,
    #[schemars(description = "Optional technology/framework used (e.g., 'Spring Boot', 'PostgreSQL')")]
    pub technology: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddRelationshipParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Name of the source element (person, system, or container)")]
    pub source_name: String,
    #[schemars(description = "Name of the destination element (person, system, or container)")]
    pub destination_name: String,
    #[schemars(description = "Description of the relationship (e.g., 'Uses', 'Sends data to')")]
    pub description: Option<String>,
    #[schemars(description = "Technology used for the interaction (e.g., 'REST/HTTP', 'gRPC')")]
    pub technology: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SaveWorkspaceParam {
    #[schemars(description = "The unique identifier of the workspace to save")]
    pub workspace_id: String,
    #[schemars(description = "Optional filename for the output JSON file (defaults to workspace_id.json)")]
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SaveDslParam {
    #[schemars(description = "The unique identifier of the workspace to save")]
    pub workspace_id: String,
    #[schemars(description = "Optional filename for the output DSL file (defaults to workspace.dsl in workspace directory)")]
    pub filename: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AddComponentParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The name of the software system containing the container")]
    pub system_name: String,
    #[schemars(description = "The name of the container to add the component to")]
    pub container_name: String,
    #[schemars(description = "The name of the component to add")]
    pub name: String,
    #[schemars(description = "Optional description of the component")]
    pub description: Option<String>,
    #[schemars(description = "Optional technology/framework used (e.g., 'Spring MVC Controller', 'React Component')")]
    pub technology: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateElementParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The current name of the element to update")]
    pub element_name: String,
    #[schemars(description = "New name for the element (optional, keeps current if not provided)")]
    pub new_name: Option<String>,
    #[schemars(description = "New description for the element (optional)")]
    pub new_description: Option<String>,
    #[schemars(description = "New technology for containers/components (optional)")]
    pub new_technology: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RemoveElementParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The name of the element to remove")]
    pub element_name: String,
    #[schemars(description = "If true, also removes all relationships involving this element")]
    #[serde(default = "default_true")]
    pub cascade_relationships: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListModifiedElementsParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
}

// ============================================================================
// Parameter types for view management tools
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSystemLandscapeViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Unique key for the view (e.g., 'SystemLandscape', 'Overview')")]
    pub view_key: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateSystemContextViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Name of the software system to create a context view for")]
    pub system_name: String,
    #[schemars(description = "Unique key for the view (e.g., 'SystemContext', 'MySystemContext')")]
    pub view_key: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateContainerViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Name of the software system to create a container view for")]
    pub system_name: String,
    #[schemars(description = "Unique key for the view (e.g., 'Containers', 'MySystemContainers')")]
    pub view_key: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateComponentViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Name of the software system containing the container")]
    pub system_name: String,
    #[schemars(description = "Name of the container to create a component view for")]
    pub container_name: String,
    #[schemars(description = "Unique key for the view (e.g., 'Components', 'APIComponents')")]
    pub view_key: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDynamicViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Unique key for the view (e.g., 'Dynamic', 'UserFlow')")]
    pub view_key: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Optional name of the element to scope the dynamic view to")]
    pub scope_element: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDeploymentViewParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Unique key for the view (e.g., 'Deployment', 'Production')")]
    pub view_key: String,
    #[schemars(description = "Deployment environment name (e.g., 'Production', 'Development', 'Staging')")]
    pub environment: String,
    #[schemars(description = "Optional title for the view")]
    pub title: Option<String>,
    #[schemars(description = "Optional description of the view")]
    pub description: Option<String>,
    #[schemars(description = "Optional name of the software system to scope the deployment view to")]
    pub system_name: Option<String>,
    #[schemars(description = "Whether to enable auto-layout (default: true)")]
    #[serde(default = "default_true")]
    pub auto_layout: bool,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewAddElementParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The key of the view to add the element to")]
    pub view_key: String,
    #[schemars(description = "The name of the element to add to the view")]
    pub element_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewAddAllElementsParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The key of the view to add all elements to")]
    pub view_key: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewRemoveElementParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The key of the view to remove the element from")]
    pub view_key: String,
    #[schemars(description = "The name of the element to remove from the view")]
    pub element_name: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewSetAutoLayoutParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "The key of the view to configure")]
    pub view_key: String,
    #[schemars(description = "Whether to enable auto-layout")]
    pub enabled: bool,
    #[schemars(description = "Layout direction: 'TopBottom', 'BottomTop', 'LeftRight', 'RightLeft' (default: TopBottom)")]
    pub direction: Option<String>,
}

// ============================================================================
// Parameter types for documentation and ADR tools
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsAddSectionParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Title of the documentation section")]
    pub title: String,
    #[schemars(description = "Content of the section (Markdown format)")]
    pub content: String,
    #[schemars(description = "Order/position of the section (optional, defaults to end)")]
    pub order: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsUpdateSectionParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Title of the section to update")]
    pub section_title: String,
    #[schemars(description = "New title for the section (optional)")]
    pub new_title: Option<String>,
    #[schemars(description = "New content for the section (optional)")]
    pub new_content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsRemoveSectionParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Title of the section to remove")]
    pub section_title: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsListSectionsParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdrCreateParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "Unique ID for the ADR (e.g., 'ADR-001', '1')")]
    pub adr_id: String,
    #[schemars(description = "Title of the architecture decision")]
    pub title: String,
    #[schemars(description = "Content/body of the ADR (Markdown format)")]
    pub content: String,
    #[schemars(description = "Status: 'Proposed', 'Accepted', 'Superseded', 'Deprecated', 'Rejected' (default: Proposed)")]
    pub status: Option<String>,
    #[schemars(description = "Date of the decision (YYYY-MM-DD format, defaults to today)")]
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdrUpdateParam {
    #[schemars(description = "The unique identifier of the workspace to modify")]
    pub workspace_id: String,
    #[schemars(description = "ID of the ADR to update")]
    pub adr_id: String,
    #[schemars(description = "New title (optional)")]
    pub new_title: Option<String>,
    #[schemars(description = "New content (optional)")]
    pub new_content: Option<String>,
    #[schemars(description = "New status: 'Proposed', 'Accepted', 'Superseded', 'Deprecated', 'Rejected' (optional)")]
    pub new_status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdrListParam {
    #[schemars(description = "The unique identifier of the workspace")]
    pub workspace_id: String,
}

// ============================================================================
// Response types for tools
// ============================================================================

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceListResponse {
    pub workspaces: Vec<WorkspaceInfoResponse>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceInfoResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub view_count: usize,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceLoadResponse {
    pub name: String,
    pub description: String,
    pub people_count: usize,
    pub systems_count: usize,
    pub relationships_count: usize,
    pub views_count: usize,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResponse {
    pub valid: bool,
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ElementAddedResponse {
    pub element_id: String,
    pub element_type: String,
    pub name: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationshipAddedResponse {
    pub relationship_id: String,
    pub source: String,
    pub destination: String,
    pub description: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct WorkspaceSavedResponse {
    pub workspace_id: String,
    pub filename: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ElementUpdatedResponse {
    pub element_name: String,
    pub element_type: String,
    pub changes: Vec<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ElementRemovedResponse {
    pub element_name: String,
    pub element_type: String,
    pub relationships_removed: usize,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewCreatedResponse {
    pub view_key: String,
    pub view_type: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ViewModifiedResponse {
    pub view_key: String,
    pub action: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct DocsSectionResponse {
    pub title: String,
    pub order: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct AdrResponse {
    pub adr_id: String,
    pub title: String,
    pub status: String,
    pub message: String,
}

// ============================================================================
// MCP Server Implementation
// ============================================================================

/// The main MCP server for structurizr-rs
#[derive(Clone)]
pub struct StructurizrMcpServer {
    /// Workspace registry (read-only source)
    registry: Arc<RwLock<Option<WorkspaceRegistry>>>,
    /// Modified workspaces cache (for write operations)
    /// These are cloned from registry when first modified
    modified_workspaces: Arc<RwLock<HashMap<String, Workspace>>>,
    /// Active workspace ID
    active_workspace: Arc<RwLock<Option<String>>>,
    /// Root directory for workspaces
    workspaces_dir: PathBuf,
    /// Tool router
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl StructurizrMcpServer {
    /// Create a new MCP server
    pub fn new(workspaces_dir: PathBuf) -> Self {
        Self {
            registry: Arc::new(RwLock::new(None)),
            modified_workspaces: Arc::new(RwLock::new(HashMap::new())),
            active_workspace: Arc::new(RwLock::new(None)),
            workspaces_dir,
            tool_router: Self::tool_router(),
        }
    }

    /// Initialize the workspace registry
    pub async fn initialize(&self) -> crate::error::Result<()> {
        info!("Initializing MCP server");
        let mut registry = WorkspaceRegistry::new(self.workspaces_dir.clone());
        registry.discover().await?;
        *self.registry.write().await = Some(registry);

        let count = self.list_workspace_infos().await.len();
        info!("Found {} workspaces", count);
        Ok(())
    }

    /// Get list of workspace infos
    async fn list_workspace_infos(&self) -> Vec<WorkspaceInfoResponse> {
        let registry = self.registry.read().await;
        match registry.as_ref() {
            Some(reg) => reg
                .list()
                .into_iter()
                .map(|w| WorkspaceInfoResponse {
                    id: w.id.clone(),
                    name: w.name.clone(),
                    description: w.description.clone(),
                    view_count: w.view_count,
                })
                .collect(),
            None => vec![],
        }
    }

    /// Get a workspace by ID (prefers modified version if available)
    async fn get_workspace(&self, workspace_id: &str) -> Option<Workspace> {
        // First check if we have a modified version
        {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(workspace_id) {
                return Some(workspace.clone());
            }
        }

        // Otherwise get from registry
        let registry = self.registry.read().await;
        let registry = registry.as_ref()?;
        match registry.get_workspace(workspace_id).await {
            Ok(workspace) => workspace,
            Err(_) => None,
        }
    }

    /// Get or create a mutable workspace for modification
    /// Clones from registry if not already in modified cache
    /// Uses entry() API to avoid race conditions with concurrent requests
    async fn get_or_create_mutable_workspace(&self, workspace_id: &str) -> Option<()> {
        // Fast path: check if already in modified cache
        {
            let modified = self.modified_workspaces.read().await;
            if modified.contains_key(workspace_id) {
                return Some(());
            }
        }

        // Clone from registry (outside the write lock to avoid deadlocks)
        let workspace = {
            let registry = self.registry.read().await;
            let registry = registry.as_ref()?;
            match registry.get_workspace(workspace_id).await {
                Ok(Some(ws)) => ws,
                _ => return None,
            }
        };

        // Insert only if not already present (handles race condition)
        let mut modified = self.modified_workspaces.write().await;
        if !modified.contains_key(workspace_id) {
            modified.insert(workspace_id.to_string(), workspace);
            info!("Created mutable workspace copy for: {}", workspace_id);
        }
        Some(())
    }

    /// Check if a workspace has unsaved modifications
    async fn has_modifications(&self, workspace_id: &str) -> bool {
        let modified = self.modified_workspaces.read().await;
        modified.contains_key(workspace_id)
    }

    /// Find an element by name across people, systems, and containers
    /// Returns (element_id, element_type, parent_system_name for containers)
    async fn find_element_by_name(
        &self,
        workspace_id: &str,
        name: &str,
    ) -> Option<(ElementId, String, Option<String>)> {
        let modified = self.modified_workspaces.read().await;
        let workspace = modified.get(workspace_id)?;

        // Search people
        for person in &workspace.model.people {
            if person.name() == name {
                return Some((person.id(), "Person".to_string(), None));
            }
        }

        // Search software systems and their containers
        for system in &workspace.model.software_systems {
            if system.name() == name {
                return Some((system.id(), "SoftwareSystem".to_string(), None));
            }
            for container in &system.containers {
                if container.name() == name {
                    return Some((
                        container.id(),
                        "Container".to_string(),
                        Some(system.name().to_string()),
                    ));
                }
            }
        }

        None
    }

    /// Find an element ID by name in a workspace (synchronous version for use inside locks)
    fn find_element_id_in_workspace(&self, workspace: &Workspace, name: &str) -> Option<ElementId> {
        // Search people
        for person in &workspace.model.people {
            if person.name() == name {
                return Some(person.id());
            }
        }

        // Search software systems and their children
        for system in &workspace.model.software_systems {
            if system.name() == name {
                return Some(system.id());
            }
            for container in &system.containers {
                if container.name() == name {
                    return Some(container.id());
                }
                for component in &container.components {
                    if component.name() == name {
                        return Some(component.id());
                    }
                }
            }
        }

        None
    }

    // ========================================================================
    // MCP Tools
    // ========================================================================

    #[tool(description = "List all available workspaces in the system. Returns workspace IDs, names, descriptions, and view counts.")]
    async fn workspace_list(&self) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_list");

        let workspaces = self.list_workspace_infos().await;
        let response = WorkspaceListResponse { workspaces };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Load a workspace by its ID and get detailed information including element counts and statistics.")]
    async fn workspace_load(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_load with id={}", params.workspace_id);

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        // Set as active workspace
        *self.active_workspace.write().await = Some(params.workspace_id.clone());

        let response = WorkspaceLoadResponse {
            name: workspace.name.clone(),
            description: workspace.description.clone().unwrap_or_default(),
            people_count: workspace.model.people.len(),
            systems_count: workspace.model.software_systems.len(),
            relationships_count: workspace.model.relationships.len(),
            views_count: workspace.views.all_keys().len(),
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Validate a workspace and check for errors, warnings, and issues in the architecture model.")]
    async fn workspace_validate(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_validate with id={}", params.workspace_id);

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let result = structurizr_dsl::validate_workspace(&workspace);

        let issues: Vec<String> = result
            .issues
            .iter()
            .map(|i| format!("[{}] {}", i.severity, i.message))
            .collect();

        let response = ValidationResponse {
            valid: result.error_count == 0,
            error_count: result.error_count,
            warning_count: result.warning_count,
            info_count: result.info_count,
            issues,
        };

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Export a workspace to JSON format (Structurizr-compatible JSON).")]
    async fn workspace_export_json(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_export_json with id={}", params.workspace_id);

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        match JsonExporter::export(&workspace) {
            Ok(json) => Ok(CallToolResult::success(vec![Content::text(json)])),
            Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
                "Export failed: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Get details about the model elements in a workspace including people, software systems, containers, and relationships.")]
    async fn workspace_get_model(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_get_model with id={}", params.workspace_id);

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let mut output = String::new();
        output.push_str("# Model Elements\n\n");

        // People
        output.push_str("## People\n");
        for person in &workspace.model.people {
            output.push_str(&format!(
                "- **{}**: {}\n",
                person.name(),
                person.properties.description.as_deref().unwrap_or("(no description)")
            ));
        }

        // Software Systems
        output.push_str("\n## Software Systems\n");
        for system in &workspace.model.software_systems {
            output.push_str(&format!(
                "- **{}**: {}\n",
                system.name(),
                system.properties.description.as_deref().unwrap_or("(no description)")
            ));

            // Containers within systems
            for container in &system.containers {
                output.push_str(&format!(
                    "  - Container: **{}** [{}]: {}\n",
                    container.name(),
                    container.technology.as_deref().unwrap_or("?"),
                    container.properties.description.as_deref().unwrap_or("(no description)")
                ));
            }
        }

        // Relationships
        output.push_str("\n## Relationships\n");
        for rel in &workspace.model.relationships {
            output.push_str(&format!(
                "- {} -> {}: {}\n",
                rel.source_id,
                rel.destination_id,
                rel.description.as_deref().unwrap_or("(uses)")
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Get the list of views defined in a workspace including system landscape, system context, container, component, dynamic, and deployment views.")]
    async fn workspace_get_views(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!("Tool called: workspace_get_views with id={}", params.workspace_id);

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let mut output = String::new();
        output.push_str("# Views\n\n");

        // System Landscape Views
        if !workspace.views.system_landscape_views.is_empty() {
            output.push_str("## System Landscape Views\n");
            for view in &workspace.views.system_landscape_views {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)")
                ));
            }
        }

        // System Context Views
        if !workspace.views.system_context_views.is_empty() {
            output.push_str("\n## System Context Views\n");
            for view in &workspace.views.system_context_views {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)")
                ));
            }
        }

        // Container Views
        if !workspace.views.container_views.is_empty() {
            output.push_str("\n## Container Views\n");
            for view in &workspace.views.container_views {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)")
                ));
            }
        }

        // Component Views
        if !workspace.views.component_views.is_empty() {
            output.push_str("\n## Component Views\n");
            for view in &workspace.views.component_views {
                output.push_str(&format!(
                    "- **{}**: {}\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)")
                ));
            }
        }

        // Dynamic Views
        if !workspace.views.dynamic_views.is_empty() {
            output.push_str("\n## Dynamic Views\n");
            for view in &workspace.views.dynamic_views {
                output.push_str(&format!(
                    "- **{}**: {} ({} steps)\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)"),
                    view.steps.len()
                ));
            }
        }

        // Deployment Views
        if !workspace.views.deployment_views.is_empty() {
            output.push_str("\n## Deployment Views\n");
            for view in &workspace.views.deployment_views {
                output.push_str(&format!(
                    "- **{}**: {} (env: {})\n",
                    view.properties.key,
                    view.properties.title.as_deref().unwrap_or("(no title)"),
                    &view.environment
                ));
            }
        }

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    #[tool(description = "Search for elements in a workspace by name or description. Returns matching people, software systems, and containers.")]
    async fn workspace_search(
        &self,
        Parameters(params): Parameters<WorkspaceSearchParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: workspace_search with id={}, query={}",
            params.workspace_id, params.query
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let query_lower = params.query.to_lowercase();
        let mut results = Vec::new();

        // Search people
        for person in &workspace.model.people {
            if person.name().to_lowercase().contains(&query_lower)
                || person
                    .properties.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            {
                results.push(format!(
                    "Person: {} - {}",
                    person.name(),
                    person.properties.description.as_deref().unwrap_or("")
                ));
            }
        }

        // Search systems
        for system in &workspace.model.software_systems {
            if system.name().to_lowercase().contains(&query_lower)
                || system
                    .properties.description
                    .as_ref()
                    .map(|d| d.to_lowercase().contains(&query_lower))
                    .unwrap_or(false)
            {
                results.push(format!(
                    "Software System: {} - {}",
                    system.name(),
                    system.properties.description.as_deref().unwrap_or("")
                ));
            }

            // Search containers
            for container in &system.containers {
                if container.name().to_lowercase().contains(&query_lower)
                    || container
                        .properties.description
                        .as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                {
                    results.push(format!(
                        "Container: {}/{} - {}",
                        system.name(),
                        container.name(),
                        container.properties.description.as_deref().unwrap_or("")
                    ));
                }
            }
        }

        if results.is_empty() {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "No results found for query: {}",
                params.query
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "# Search Results for '{}'\n\n{}",
                params.query,
                results.join("\n")
            ))]))
        }
    }

    // ========================================================================
    // Render and Export Tools
    // ========================================================================

    #[tool(description = "Render a specific view to SVG format. Returns the SVG markup for the diagram.")]
    async fn render_svg(
        &self,
        Parameters(params): Parameters<RenderViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: render_svg with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let renderer = SvgRenderer::default();
        let view_key = &params.view_key;

        // Try each view type to find the matching key
        // System Landscape Views
        if let Some(view) = workspace.views.system_landscape_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_system_landscape(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        // System Context Views
        if let Some(view) = workspace.views.system_context_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_system_context(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        // Container Views
        if let Some(view) = workspace.views.container_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_container(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        // Component Views
        if let Some(view) = workspace.views.component_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_component(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        // Dynamic Views
        if let Some(view) = workspace.views.dynamic_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_dynamic(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        // Deployment Views
        if let Some(view) = workspace.views.deployment_views.iter().find(|v| v.properties.key == *view_key) {
            match renderer.render_deployment(&workspace, view) {
                Ok(svg) => return Ok(CallToolResult::success(vec![Content::text(svg)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Render error: {}", e))])),
            }
        }

        Ok(CallToolResult::error(vec![Content::text(format!(
            "View not found: {}. Use workspace_get_views to see available view keys.",
            view_key
        ))]))
    }

    #[tool(description = "Export a specific view to PlantUML format for diagram generation.")]
    async fn export_plantuml(
        &self,
        Parameters(params): Parameters<RenderViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: export_plantuml with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let view_key = &params.view_key;

        // Try each view type
        if let Some(view) = workspace.views.system_landscape_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_system_landscape(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.system_context_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_system_context(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.container_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_container(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.component_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_component(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.dynamic_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_dynamic(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.deployment_views.iter().find(|v| v.properties.key == *view_key) {
            match PlantUmlExporter::export_deployment(&workspace, view) {
                Ok(puml) => return Ok(CallToolResult::success(vec![Content::text(puml)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        Ok(CallToolResult::error(vec![Content::text(format!(
            "View not found: {}. Use workspace_get_views to see available view keys.",
            view_key
        ))]))
    }

    #[tool(description = "Export a specific view to Mermaid format for diagram generation.")]
    async fn export_mermaid(
        &self,
        Parameters(params): Parameters<RenderViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: export_mermaid with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let view_key = &params.view_key;

        // Try each view type
        if let Some(view) = workspace.views.system_landscape_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_system_landscape(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.system_context_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_system_context(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.container_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_container(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.component_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_component(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.dynamic_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_dynamic(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.deployment_views.iter().find(|v| v.properties.key == *view_key) {
            match MermaidExporter::export_deployment(&workspace, view) {
                Ok(mmd) => return Ok(CallToolResult::success(vec![Content::text(mmd)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        Ok(CallToolResult::error(vec![Content::text(format!(
            "View not found: {}. Use workspace_get_views to see available view keys.",
            view_key
        ))]))
    }

    #[tool(description = "Export a specific view to D2 format for diagram generation.")]
    async fn export_d2(
        &self,
        Parameters(params): Parameters<RenderViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: export_d2 with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let view_key = &params.view_key;

        // Try each view type
        if let Some(view) = workspace.views.system_landscape_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_system_landscape(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.system_context_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_system_context(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.container_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_container(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.component_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_component(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.dynamic_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_dynamic(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.deployment_views.iter().find(|v| v.properties.key == *view_key) {
            match D2Exporter::export_deployment(&workspace, view) {
                Ok(d2) => return Ok(CallToolResult::success(vec![Content::text(d2)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        Ok(CallToolResult::error(vec![Content::text(format!(
            "View not found: {}. Use workspace_get_views to see available view keys.",
            view_key
        ))]))
    }

    #[tool(description = "Export a specific view to DOT/Graphviz format for diagram generation.")]
    async fn export_dot(
        &self,
        Parameters(params): Parameters<RenderViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: export_dot with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        let workspace = match self.get_workspace(&params.workspace_id).await {
            Some(ws) => ws,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace not found: {}",
                    params.workspace_id
                ))]));
            }
        };

        let view_key = &params.view_key;

        // Try each view type (DOT exporter supports fewer view types)
        if let Some(view) = workspace.views.system_landscape_views.iter().find(|v| v.properties.key == *view_key) {
            match DotExporter::export_system_landscape(&workspace, view) {
                Ok(dot) => return Ok(CallToolResult::success(vec![Content::text(dot)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.system_context_views.iter().find(|v| v.properties.key == *view_key) {
            match DotExporter::export_system_context(&workspace, view) {
                Ok(dot) => return Ok(CallToolResult::success(vec![Content::text(dot)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.container_views.iter().find(|v| v.properties.key == *view_key) {
            match DotExporter::export_container(&workspace, view) {
                Ok(dot) => return Ok(CallToolResult::success(vec![Content::text(dot)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        if let Some(view) = workspace.views.component_views.iter().find(|v| v.properties.key == *view_key) {
            match DotExporter::export_component(&workspace, view) {
                Ok(dot) => return Ok(CallToolResult::success(vec![Content::text(dot)])),
                Err(e) => return Ok(CallToolResult::error(vec![Content::text(format!("Export error: {}", e))])),
            }
        }

        Ok(CallToolResult::error(vec![Content::text(format!(
            "View not found or not supported for DOT export: {}. DOT exporter supports system landscape, system context, container, and component views.",
            view_key
        ))]))
    }

    // ========================================================================
    // Model Manipulation Tools (Write Operations)
    // ========================================================================

    #[tool(description = "Add a new person to the workspace model. Returns the element ID of the created person. Note: Changes are held in memory until workspace_save_json is called.")]
    async fn model_add_person(
        &self,
        Parameters(params): Parameters<AddPersonParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_add_person with workspace={}, name={}",
            params.workspace_id, params.name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        // Create and add the person
        let mut person = Person::new(&params.name);
        if let Some(desc) = &params.description {
            person = person.with_description(desc);
        }
        if params.external {
            person = person.external();
        }
        let person_id = person.id();

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.model.people.push(person);
            }
        }

        let response = ElementAddedResponse {
            element_id: person_id.to_string(),
            element_type: "Person".to_string(),
            name: params.name.clone(),
            message: format!(
                "Added person '{}' to workspace '{}'. Use workspace_save_json to persist changes.",
                params.name, params.workspace_id
            ),
        };

        info!("Added person '{}' to workspace '{}'", params.name, params.workspace_id);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Add a new software system to the workspace model. Returns the element ID of the created system. Note: Changes are held in memory until workspace_save_json is called.")]
    async fn model_add_system(
        &self,
        Parameters(params): Parameters<AddSoftwareSystemParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_add_system with workspace={}, name={}",
            params.workspace_id, params.name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        // Create and add the software system
        let mut system = SoftwareSystem::new(&params.name);
        if let Some(desc) = &params.description {
            system = system.with_description(desc);
        }
        if params.external {
            system = system.external();
        }
        let system_id = system.id();

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.model.software_systems.push(system);
            }
        }

        let response = ElementAddedResponse {
            element_id: system_id.to_string(),
            element_type: "SoftwareSystem".to_string(),
            name: params.name.clone(),
            message: format!(
                "Added software system '{}' to workspace '{}'. Use workspace_save_json to persist changes.",
                params.name, params.workspace_id
            ),
        };

        info!("Added software system '{}' to workspace '{}'", params.name, params.workspace_id);
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Add a new container to an existing software system. Returns the element ID of the created container. Note: Changes are held in memory until workspace_save_json is called.")]
    async fn model_add_container(
        &self,
        Parameters(params): Parameters<AddContainerParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_add_container with workspace={}, system={}, name={}",
            params.workspace_id, params.system_name, params.name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        // Create the container
        let mut container = Container::new(&params.name);
        if let Some(desc) = &params.description {
            container = container.with_description(desc);
        }
        if let Some(tech) = &params.technology {
            container = container.with_technology(tech);
        }
        let container_id = container.id();

        // Find the system and add the container
        let system_found = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                if let Some(system) = workspace
                    .model
                    .software_systems
                    .iter_mut()
                    .find(|s| s.name() == params.system_name)
                {
                    system.containers.push(container);
                    true
                } else {
                    false
                }
            } else {
                false
            }
        };

        if !system_found {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Software system '{}' not found in workspace '{}'. Use model_add_system to create it first.",
                params.system_name, params.workspace_id
            ))]));
        }

        let response = ElementAddedResponse {
            element_id: container_id.to_string(),
            element_type: "Container".to_string(),
            name: params.name.clone(),
            message: format!(
                "Added container '{}' to system '{}' in workspace '{}'. Use workspace_save_json to persist changes.",
                params.name, params.system_name, params.workspace_id
            ),
        };

        info!(
            "Added container '{}' to system '{}' in workspace '{}'",
            params.name, params.system_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Add a relationship between two elements (person, software system, or container). Elements are referenced by name. Note: Changes are held in memory until workspace_save_json is called.")]
    async fn model_add_relationship(
        &self,
        Parameters(params): Parameters<AddRelationshipParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_add_relationship with workspace={}, source={}, dest={}",
            params.workspace_id, params.source_name, params.destination_name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        // Find source element
        let source = self.find_element_by_name(&params.workspace_id, &params.source_name).await;
        let (source_id, source_type, _) = match source {
            Some(s) => s,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Source element '{}' not found in workspace '{}'. Add the element first using model_add_person, model_add_system, or model_add_container.",
                    params.source_name, params.workspace_id
                ))]));
            }
        };

        // Find destination element
        let dest = self.find_element_by_name(&params.workspace_id, &params.destination_name).await;
        let (dest_id, dest_type, _) = match dest {
            Some(d) => d,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Destination element '{}' not found in workspace '{}'. Add the element first using model_add_person, model_add_system, or model_add_container.",
                    params.destination_name, params.workspace_id
                ))]));
            }
        };

        // Create the relationship
        let mut relationship = Relationship::new(source_id, dest_id);
        if let Some(desc) = &params.description {
            relationship = relationship.with_description(desc);
        }
        if let Some(tech) = &params.technology {
            relationship = relationship.with_technology(tech);
        }
        let relationship_id = relationship.id;

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.model.relationships.push(relationship);
            }
        }

        let response = RelationshipAddedResponse {
            relationship_id: relationship_id.to_string(),
            source: format!("{} ({})", params.source_name, source_type),
            destination: format!("{} ({})", params.destination_name, dest_type),
            description: params.description.clone(),
            message: format!(
                "Added relationship from '{}' to '{}' in workspace '{}'. Use workspace_save_json to persist changes.",
                params.source_name, params.destination_name, params.workspace_id
            ),
        };

        info!(
            "Added relationship from '{}' to '{}' in workspace '{}'",
            params.source_name, params.destination_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Save a modified workspace to JSON format (Structurizr-compatible). This persists any changes made with model_add_* tools. The JSON file can be imported into Structurizr.")]
    async fn workspace_save_json(
        &self,
        Parameters(params): Parameters<SaveWorkspaceParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: workspace_save_json with workspace={}",
            params.workspace_id
        );

        // Check if workspace has modifications
        if !self.has_modifications(&params.workspace_id).await {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' has no pending modifications to save. Use model_add_* tools to modify the workspace first.",
                params.workspace_id
            ))]));
        }

        // Get the modified workspace
        let workspace = {
            let modified = self.modified_workspaces.read().await;
            match modified.get(&params.workspace_id) {
                Some(ws) => ws.clone(),
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Workspace '{}' not found in modified cache.",
                        params.workspace_id
                    ))]));
                }
            }
        };

        // Export to JSON
        let json = match JsonExporter::export(&workspace) {
            Ok(j) => j,
            Err(e) => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Failed to export workspace to JSON: {}",
                    e
                ))]));
            }
        };

        // Determine output filename
        let filename = params.filename.unwrap_or_else(|| {
            format!("{}.json", params.workspace_id.replace('/', "_"))
        });

        // Write to file in workspaces directory
        let output_path = self.workspaces_dir.join(&filename);
        if let Err(e) = std::fs::write(&output_path, &json) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to write JSON file: {}",
                e
            ))]));
        }

        let response = WorkspaceSavedResponse {
            workspace_id: params.workspace_id.clone(),
            filename: output_path.display().to_string(),
            message: format!(
                "Workspace '{}' saved to '{}'. The JSON file is Structurizr-compatible and can be imported.",
                params.workspace_id,
                output_path.display()
            ),
        };

        info!(
            "Saved workspace '{}' to '{}'",
            params.workspace_id,
            output_path.display()
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Save a modified workspace to DSL format (Structurizr DSL). This creates a .dsl file compatible with Structurizr Java tooling.")]
    async fn workspace_save_dsl(
        &self,
        Parameters(params): Parameters<SaveDslParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: workspace_save_dsl with workspace={}",
            params.workspace_id
        );

        // Check if workspace has modifications
        if !self.has_modifications(&params.workspace_id).await {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' has no pending modifications to save. Use model_add_* tools to modify the workspace first.",
                params.workspace_id
            ))]));
        }

        // Get the modified workspace
        let workspace = {
            let modified = self.modified_workspaces.read().await;
            match modified.get(&params.workspace_id) {
                Some(ws) => ws.clone(),
                None => {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Workspace '{}' not found in modified cache.",
                        params.workspace_id
                    ))]));
                }
            }
        };

        // Serialize to DSL
        let dsl_content = dsl_serialize(&workspace);

        // Determine output filename
        let filename = params.filename.unwrap_or_else(|| {
            format!("{}_modified.dsl", params.workspace_id.replace('/', "_"))
        });

        // Write to file in workspaces directory
        let output_path = self.workspaces_dir.join(&filename);
        if let Err(e) = std::fs::write(&output_path, &dsl_content) {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Failed to write DSL file: {}",
                e
            ))]));
        }

        let response = WorkspaceSavedResponse {
            workspace_id: params.workspace_id.clone(),
            filename: output_path.display().to_string(),
            message: format!(
                "Workspace '{}' saved to '{}'. The DSL file is compatible with Structurizr Java tooling.",
                params.workspace_id,
                output_path.display()
            ),
        };

        info!(
            "Saved workspace '{}' to DSL file '{}'",
            params.workspace_id,
            output_path.display()
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Discard any pending modifications to a workspace, reverting to the original state from the DSL file.")]
    async fn workspace_discard_changes(
        &self,
        Parameters(params): Parameters<WorkspaceIdParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: workspace_discard_changes with workspace={}",
            params.workspace_id
        );

        let had_changes = {
            let mut modified = self.modified_workspaces.write().await;
            modified.remove(&params.workspace_id).is_some()
        };

        if had_changes {
            info!("Discarded changes for workspace '{}'", params.workspace_id);
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Discarded pending changes for workspace '{}'. The workspace is now back to its original state.",
                params.workspace_id
            ))]))
        } else {
            Ok(CallToolResult::success(vec![Content::text(format!(
                "Workspace '{}' had no pending changes to discard.",
                params.workspace_id
            ))]))
        }
    }

    // ========================================================================
    // Phase 4: Extended Model Operations
    // ========================================================================

    #[tool(description = "Add a new component to an existing container within a software system. Returns the element ID of the created component.")]
    async fn model_add_component(
        &self,
        Parameters(params): Parameters<AddComponentParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_add_component with workspace={}, system={}, container={}, name={}",
            params.workspace_id, params.system_name, params.container_name, params.name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        // Create the component
        let mut component = Component::new(&params.name);
        if let Some(desc) = &params.description {
            component = component.with_description(desc);
        }
        if let Some(tech) = &params.technology {
            component = component.with_technology(tech);
        }
        let component_id = component.id();

        // Find the system and container, then add the component
        let container_found = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                let mut found = false;
                for system in workspace.model.software_systems.iter_mut() {
                    if system.name() == params.system_name {
                        for container in system.containers.iter_mut() {
                            if container.name() == params.container_name {
                                container.components.push(component);
                                found = true;
                                break;
                            }
                        }
                        break;
                    }
                }
                found
            } else {
                false
            }
        };

        if !container_found {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Container '{}' not found in system '{}' in workspace '{}'. Use model_add_container to create it first.",
                params.container_name, params.system_name, params.workspace_id
            ))]));
        }

        let response = ElementAddedResponse {
            element_id: component_id.to_string(),
            element_type: "Component".to_string(),
            name: params.name.clone(),
            message: format!(
                "Added component '{}' to container '{}' in system '{}'. Use workspace_save_json to persist changes.",
                params.name, params.container_name, params.system_name
            ),
        };

        info!(
            "Added component '{}' to container '{}' in workspace '{}'",
            params.name, params.container_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Update properties of an existing element (person, software system, container, or component). Can update name, description, and technology.")]
    async fn model_update_element(
        &self,
        Parameters(params): Parameters<UpdateElementParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_update_element with workspace={}, element={}",
            params.workspace_id, params.element_name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        let mut changes = Vec::new();
        let mut element_type = String::new();
        let mut found = false;

        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Try to find and update in people
                for person in workspace.model.people.iter_mut() {
                    if person.name() == params.element_name {
                        element_type = "Person".to_string();
                        found = true;
                        if let Some(new_name) = &params.new_name {
                            person.properties.name = new_name.to_string();
                            changes.push(format!("name → '{}'", new_name));
                        }
                        if let Some(new_desc) = &params.new_description {
                            person.properties.description = Some(new_desc.to_string());
                            changes.push(format!("description → '{}'", new_desc));
                        }
                        break;
                    }
                }

                // Try to find and update in software systems
                if !found {
                    for system in workspace.model.software_systems.iter_mut() {
                        if system.name() == params.element_name {
                            element_type = "SoftwareSystem".to_string();
                            found = true;
                            if let Some(new_name) = &params.new_name {
                                system.properties.name = new_name.to_string();
                                changes.push(format!("name → '{}'", new_name));
                            }
                            if let Some(new_desc) = &params.new_description {
                                system.properties.description = Some(new_desc.to_string());
                                changes.push(format!("description → '{}'", new_desc));
                            }
                            break;
                        }

                        // Check containers within this system
                        for container in system.containers.iter_mut() {
                            if container.name() == params.element_name {
                                element_type = "Container".to_string();
                                found = true;
                                if let Some(new_name) = &params.new_name {
                                    container.properties.name = new_name.to_string();
                                    changes.push(format!("name → '{}'", new_name));
                                }
                                if let Some(new_desc) = &params.new_description {
                                    container.properties.description = Some(new_desc.to_string());
                                    changes.push(format!("description → '{}'", new_desc));
                                }
                                if let Some(new_tech) = &params.new_technology {
                                    container.technology = Some(new_tech.to_string());
                                    changes.push(format!("technology → '{}'", new_tech));
                                }
                                break;
                            }

                            // Check components within this container
                            for component in container.components.iter_mut() {
                                if component.name() == params.element_name {
                                    element_type = "Component".to_string();
                                    found = true;
                                    if let Some(new_name) = &params.new_name {
                                        component.properties.name = new_name.to_string();
                                        changes.push(format!("name → '{}'", new_name));
                                    }
                                    if let Some(new_desc) = &params.new_description {
                                        component.properties.description = Some(new_desc.to_string());
                                        changes.push(format!("description → '{}'", new_desc));
                                    }
                                    if let Some(new_tech) = &params.new_technology {
                                        component.technology = Some(new_tech.to_string());
                                        changes.push(format!("technology → '{}'", new_tech));
                                    }
                                    break;
                                }
                            }
                            if found {
                                break;
                            }
                        }
                        if found {
                            break;
                        }
                    }
                }
            }
        }

        if !found {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Element '{}' not found in workspace '{}'. Use workspace_get_model to see available elements.",
                params.element_name, params.workspace_id
            ))]));
        }

        if changes.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No changes specified for element '{}'. Provide new_name, new_description, or new_technology.",
                params.element_name
            ))]));
        }

        let display_name = params.new_name.as_ref().unwrap_or(&params.element_name);
        let response = ElementUpdatedResponse {
            element_name: display_name.clone(),
            element_type,
            changes: changes.clone(),
            message: format!(
                "Updated element '{}': {}. Use workspace_save_json to persist changes.",
                params.element_name,
                changes.join(", ")
            ),
        };

        info!(
            "Updated element '{}' in workspace '{}': {}",
            params.element_name,
            params.workspace_id,
            changes.join(", ")
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Remove an element from the workspace model. By default, also removes all relationships involving that element.")]
    async fn model_remove_element(
        &self,
        Parameters(params): Parameters<RemoveElementParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_remove_element with workspace={}, element={}",
            params.workspace_id, params.element_name
        );

        // Ensure workspace exists and create mutable copy if needed
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace not found: {}",
                params.workspace_id
            ))]));
        }

        let mut element_type = String::new();
        let mut element_id: Option<ElementId> = None;
        let mut found = false;

        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Try to remove from people
                if let Some(pos) = workspace.model.people.iter().position(|p| p.name() == params.element_name) {
                    element_id = Some(workspace.model.people[pos].id());
                    element_type = "Person".to_string();
                    workspace.model.people.remove(pos);
                    found = true;
                }

                // Try to remove from software systems
                if !found {
                    if let Some(pos) = workspace.model.software_systems.iter().position(|s| s.name() == params.element_name) {
                        element_id = Some(workspace.model.software_systems[pos].id());
                        element_type = "SoftwareSystem".to_string();
                        workspace.model.software_systems.remove(pos);
                        found = true;
                    }
                }

                // Try to remove from containers within systems
                if !found {
                    'outer: for system in workspace.model.software_systems.iter_mut() {
                        if let Some(pos) = system.containers.iter().position(|c| c.name() == params.element_name) {
                            element_id = Some(system.containers[pos].id());
                            element_type = "Container".to_string();
                            system.containers.remove(pos);
                            found = true;
                            break 'outer;
                        }

                        // Try to remove from components within containers
                        for container in system.containers.iter_mut() {
                            if let Some(pos) = container.components.iter().position(|c| c.name() == params.element_name) {
                                element_id = Some(container.components[pos].id());
                                element_type = "Component".to_string();
                                container.components.remove(pos);
                                found = true;
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }

        if !found {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Element '{}' not found in workspace '{}'. Use workspace_get_model to see available elements.",
                params.element_name, params.workspace_id
            ))]));
        }

        // Remove relationships involving this element if cascade is enabled
        let mut relationships_removed = 0;
        if params.cascade_relationships {
            if let Some(eid) = element_id {
                let mut modified = self.modified_workspaces.write().await;
                if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                    let original_count = workspace.model.relationships.len();
                    workspace.model.relationships.retain(|r| {
                        r.source_id != eid && r.destination_id != eid
                    });
                    relationships_removed = original_count - workspace.model.relationships.len();
                }
            }
        }

        let response = ElementRemovedResponse {
            element_name: params.element_name.clone(),
            element_type,
            relationships_removed,
            message: format!(
                "Removed element '{}' from workspace '{}'. {} relationship(s) also removed. Use workspace_save_json to persist changes.",
                params.element_name, params.workspace_id, relationships_removed
            ),
        };

        info!(
            "Removed element '{}' from workspace '{}' ({} relationships removed)",
            params.element_name, params.workspace_id, relationships_removed
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "List all elements that have been added or modified in a workspace. Shows pending changes that will be saved with workspace_save_json.")]
    async fn model_list_pending_changes(
        &self,
        Parameters(params): Parameters<ListModifiedElementsParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: model_list_pending_changes with workspace={}",
            params.workspace_id
        );

        // Check if workspace has modifications
        if !self.has_modifications(&params.workspace_id).await {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "Workspace '{}' has no pending modifications.",
                params.workspace_id
            ))]));
        }

        // Get element counts from modified workspace
        let (people_count, systems_count, containers_count, components_count, relationships_count) = {
            let modified = self.modified_workspaces.read().await;
            match modified.get(&params.workspace_id) {
                Some(workspace) => {
                    let containers: usize = workspace.model.software_systems.iter()
                        .map(|s| s.containers.len())
                        .sum();
                    let components: usize = workspace.model.software_systems.iter()
                        .flat_map(|s| s.containers.iter())
                        .map(|c| c.components.len())
                        .sum();
                    (
                        workspace.model.people.len(),
                        workspace.model.software_systems.len(),
                        containers,
                        components,
                        workspace.model.relationships.len(),
                    )
                }
                None => (0, 0, 0, 0, 0),
            }
        };

        let output = format!(
            "# Pending Changes for '{}'\n\n\
             The workspace has been modified. Current element counts:\n\n\
             - **People**: {}\n\
             - **Software Systems**: {}\n\
             - **Containers**: {}\n\
             - **Components**: {}\n\
             - **Relationships**: {}\n\n\
             Use `workspace_save_json` to persist changes or `workspace_discard_changes` to revert.",
            params.workspace_id,
            people_count,
            systems_count,
            containers_count,
            components_count,
            relationships_count
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    // ========================================================================
    // Phase 6: View Creation Tools
    // ========================================================================

    #[tool(description = "Create a system landscape view showing all people and software systems in the workspace.")]
    async fn view_create_system_landscape(
        &self,
        Parameters(params): Parameters<CreateSystemLandscapeViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_system_landscape with workspace={}, key={}",
            params.workspace_id, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Check if view key already exists
        {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                if workspace.views.system_landscape_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }
            }
        }

        // Create the view
        let mut view = SystemLandscapeView::new(&params.view_key);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.system_landscape_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "SystemLandscape".to_string(),
            message: format!(
                "Created system landscape view '{}' in workspace '{}'.",
                params.view_key, params.workspace_id
            ),
        };

        info!(
            "Created system landscape view '{}' in workspace '{}'",
            params.view_key, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a system context view showing a software system and its relationships with people and other systems.")]
    async fn view_create_system_context(
        &self,
        Parameters(params): Parameters<CreateSystemContextViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_system_context with workspace={}, system={}, key={}",
            params.workspace_id, params.system_name, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Find the software system and check view key
        let system_id = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                // Check if view key already exists
                if workspace.views.system_context_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }

                // Find the system
                workspace.model.software_systems.iter()
                    .find(|s| s.name() == params.system_name)
                    .map(|s| s.id())
            } else {
                None
            }
        };

        let system_id = match system_id {
            Some(id) => id,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Software system '{}' not found in workspace '{}'.",
                    params.system_name, params.workspace_id
                ))]));
            }
        };

        // Create the view
        let mut view = SystemContextView::new(&params.view_key, system_id);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.system_context_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "SystemContext".to_string(),
            message: format!(
                "Created system context view '{}' for system '{}' in workspace '{}'.",
                params.view_key, params.system_name, params.workspace_id
            ),
        };

        info!(
            "Created system context view '{}' for '{}' in workspace '{}'",
            params.view_key, params.system_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a container view showing the containers within a software system.")]
    async fn view_create_container(
        &self,
        Parameters(params): Parameters<CreateContainerViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_container with workspace={}, system={}, key={}",
            params.workspace_id, params.system_name, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Find the software system and check view key
        let system_id = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                // Check if view key already exists
                if workspace.views.container_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }

                // Find the system
                workspace.model.software_systems.iter()
                    .find(|s| s.name() == params.system_name)
                    .map(|s| s.id())
            } else {
                None
            }
        };

        let system_id = match system_id {
            Some(id) => id,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Software system '{}' not found in workspace '{}'.",
                    params.system_name, params.workspace_id
                ))]));
            }
        };

        // Create the view
        let mut view = ContainerView::new(&params.view_key, system_id);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.container_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "Container".to_string(),
            message: format!(
                "Created container view '{}' for system '{}' in workspace '{}'.",
                params.view_key, params.system_name, params.workspace_id
            ),
        };

        info!(
            "Created container view '{}' for '{}' in workspace '{}'",
            params.view_key, params.system_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a component view showing the components within a container.")]
    async fn view_create_component(
        &self,
        Parameters(params): Parameters<CreateComponentViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_component with workspace={}, system={}, container={}, key={}",
            params.workspace_id, params.system_name, params.container_name, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Find the container and check view key
        let container_id = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                // Check if view key already exists
                if workspace.views.component_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }

                // Find the container
                workspace.model.software_systems.iter()
                    .find(|s| s.name() == params.system_name)
                    .and_then(|s| s.containers.iter().find(|c| c.name() == params.container_name))
                    .map(|c| c.id())
            } else {
                None
            }
        };

        let container_id = match container_id {
            Some(id) => id,
            None => {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Container '{}' not found in system '{}' in workspace '{}'.",
                    params.container_name, params.system_name, params.workspace_id
                ))]));
            }
        };

        // Create the view
        let mut view = ComponentView::new(&params.view_key, container_id);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.component_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "Component".to_string(),
            message: format!(
                "Created component view '{}' for container '{}' in workspace '{}'.",
                params.view_key, params.container_name, params.workspace_id
            ),
        };

        info!(
            "Created component view '{}' for '{}' in workspace '{}'",
            params.view_key, params.container_name, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a dynamic view showing interactions between elements over time.")]
    async fn view_create_dynamic(
        &self,
        Parameters(params): Parameters<CreateDynamicViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_dynamic with workspace={}, key={}",
            params.workspace_id, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Check if view key already exists and find scope element if provided
        let scope_element_id = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                // Check if view key already exists
                if workspace.views.dynamic_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }

                // Find scope element if provided
                if let Some(scope_name) = &params.scope_element {
                    let id = self.find_element_id_in_workspace(workspace, scope_name);
                    if id.is_none() {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Scope element '{}' not found in workspace '{}'.",
                            scope_name, params.workspace_id
                        ))]));
                    }
                    id
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Create the view
        let mut view = DynamicView::new(&params.view_key);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }
        view.element_id = scope_element_id;

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.dynamic_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "Dynamic".to_string(),
            message: format!(
                "Created dynamic view '{}' in workspace '{}'.",
                params.view_key, params.workspace_id
            ),
        };

        info!(
            "Created dynamic view '{}' in workspace '{}'",
            params.view_key, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Create a deployment view showing how software systems are deployed to infrastructure.")]
    async fn view_create_deployment(
        &self,
        Parameters(params): Parameters<CreateDeploymentViewParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_create_deployment with workspace={}, key={}, env={}",
            params.workspace_id, params.view_key, params.environment
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Check if view key already exists and find system if provided
        let system_id = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                // Check if view key already exists
                if workspace.views.deployment_views.iter().any(|v| v.properties.key == params.view_key) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "View with key '{}' already exists.",
                        params.view_key
                    ))]));
                }

                // Find system if provided
                if let Some(ref system_name) = params.system_name {
                    let id = workspace.model.software_systems.iter()
                        .find(|s| s.name() == system_name)
                        .map(|s| s.id());
                    if id.is_none() {
                        return Ok(CallToolResult::error(vec![Content::text(format!(
                            "Software system '{}' not found in workspace '{}'.",
                            system_name, params.workspace_id
                        ))]));
                    }
                    id
                } else {
                    None
                }
            } else {
                None
            }
        };

        // Create the view
        let mut view = DeploymentView::new(&params.view_key, &params.environment);
        if let Some(title) = params.title {
            view.properties.title = Some(title);
        }
        if let Some(desc) = params.description {
            view.properties.description = Some(desc);
        }
        if params.auto_layout {
            view.properties.auto_layout = Some(AutoLayout::default());
        }
        view.software_system_id = system_id;

        // Add to workspace
        {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                workspace.views.deployment_views.push(view);
            }
        }

        let response = ViewCreatedResponse {
            view_key: params.view_key.clone(),
            view_type: "Deployment".to_string(),
            message: format!(
                "Created deployment view '{}' for environment '{}' in workspace '{}'.",
                params.view_key, params.environment, params.workspace_id
            ),
        };

        info!(
            "Created deployment view '{}' for '{}' in workspace '{}'",
            params.view_key, params.environment, params.workspace_id
        );
        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    // ========================================================================
    // Phase 6: View Element Management Tools
    // ========================================================================

    #[tool(description = "Add an element to an existing view. The element must exist in the workspace model.")]
    async fn view_add_element(
        &self,
        Parameters(params): Parameters<ViewAddElementParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_add_element with workspace={}, view={}, element={}",
            params.workspace_id, params.view_key, params.element_name
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Find the element and add to view
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Find the element
                let element_id = self.find_element_id_in_workspace(workspace, &params.element_name);

                if element_id.is_none() {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Element '{}' not found in workspace '{}'.",
                        params.element_name, params.workspace_id
                    ))]));
                }
                let element_id = element_id.unwrap();

                // Find the view and add the element
                let element_view = ElementView::new(element_id);

                // Try each view type
                if let Some(view) = workspace.views.system_landscape_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("SystemLandscape")
                } else if let Some(view) = workspace.views.system_context_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("SystemContext")
                } else if let Some(view) = workspace.views.container_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("Container")
                } else if let Some(view) = workspace.views.component_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("Component")
                } else if let Some(view) = workspace.views.dynamic_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("Dynamic")
                } else if let Some(view) = workspace.views.deployment_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    if !view.properties.elements.iter().any(|e| e.id == element_id) {
                        view.properties.elements.push(element_view);
                    }
                    Some("Deployment")
                } else {
                    None
                }
            } else {
                None
            }
        };

        match result {
            Some(view_type) => {
                let response = ViewModifiedResponse {
                    view_key: params.view_key.clone(),
                    action: format!("Added element '{}'", params.element_name),
                    message: format!(
                        "Added element '{}' to {} view '{}' in workspace '{}'.",
                        params.element_name, view_type, params.view_key, params.workspace_id
                    ),
                };
                info!(
                    "Added element '{}' to view '{}' in workspace '{}'",
                    params.element_name, params.view_key, params.workspace_id
                );
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "View '{}' not found in workspace '{}'.",
                params.view_key, params.workspace_id
            ))])),
        }
    }

    #[tool(description = "Add all relevant elements to a view (equivalent to 'include *' in DSL).")]
    async fn view_add_all_elements(
        &self,
        Parameters(params): Parameters<ViewAddAllElementsParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_add_all_elements with workspace={}, view={}",
            params.workspace_id, params.view_key
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Add all elements to the view
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Collect all element IDs
                let mut all_ids: Vec<ElementId> = Vec::new();
                for person in &workspace.model.people {
                    all_ids.push(person.id());
                }
                for system in &workspace.model.software_systems {
                    all_ids.push(system.id());
                    for container in &system.containers {
                        all_ids.push(container.id());
                        for component in &container.components {
                            all_ids.push(component.id());
                        }
                    }
                }

                // Find the view and add all elements
                let added_count: usize;

                if let Some(view) = workspace.views.system_landscape_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("SystemLandscape", added_count))
                } else if let Some(view) = workspace.views.system_context_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("SystemContext", added_count))
                } else if let Some(view) = workspace.views.container_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("Container", added_count))
                } else if let Some(view) = workspace.views.component_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("Component", added_count))
                } else if let Some(view) = workspace.views.dynamic_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("Dynamic", added_count))
                } else if let Some(view) = workspace.views.deployment_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    let before = view.properties.elements.len();
                    for id in &all_ids {
                        if !view.properties.elements.iter().any(|e| e.id == *id) {
                            view.properties.elements.push(ElementView::new(*id));
                        }
                    }
                    added_count = view.properties.elements.len() - before;
                    Some(("Deployment", added_count))
                } else {
                    None
                }
            } else {
                None
            }
        };

        match result {
            Some((view_type, count)) => {
                let response = ViewModifiedResponse {
                    view_key: params.view_key.clone(),
                    action: format!("Added {} elements", count),
                    message: format!(
                        "Added {} elements to {} view '{}' in workspace '{}'.",
                        count, view_type, params.view_key, params.workspace_id
                    ),
                };
                info!(
                    "Added {} elements to view '{}' in workspace '{}'",
                    count, params.view_key, params.workspace_id
                );
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "View '{}' not found in workspace '{}'.",
                params.view_key, params.workspace_id
            ))])),
        }
    }

    #[tool(description = "Remove an element from a view.")]
    async fn view_remove_element(
        &self,
        Parameters(params): Parameters<ViewRemoveElementParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_remove_element with workspace={}, view={}, element={}",
            params.workspace_id, params.view_key, params.element_name
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Find the element and remove from view
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Find the element
                let element_id = self.find_element_id_in_workspace(workspace, &params.element_name);

                if element_id.is_none() {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Element '{}' not found in workspace '{}'.",
                        params.element_name, params.workspace_id
                    ))]));
                }
                let element_id = element_id.unwrap();

                // Find the view and remove the element
                if let Some(view) = workspace.views.system_landscape_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("SystemLandscape")
                } else if let Some(view) = workspace.views.system_context_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("SystemContext")
                } else if let Some(view) = workspace.views.container_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("Container")
                } else if let Some(view) = workspace.views.component_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("Component")
                } else if let Some(view) = workspace.views.dynamic_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("Dynamic")
                } else if let Some(view) = workspace.views.deployment_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.elements.retain(|e| e.id != element_id);
                    Some("Deployment")
                } else {
                    None
                }
            } else {
                None
            }
        };

        match result {
            Some(view_type) => {
                let response = ViewModifiedResponse {
                    view_key: params.view_key.clone(),
                    action: format!("Removed element '{}'", params.element_name),
                    message: format!(
                        "Removed element '{}' from {} view '{}' in workspace '{}'.",
                        params.element_name, view_type, params.view_key, params.workspace_id
                    ),
                };
                info!(
                    "Removed element '{}' from view '{}' in workspace '{}'",
                    params.element_name, params.view_key, params.workspace_id
                );
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "View '{}' not found in workspace '{}'.",
                params.view_key, params.workspace_id
            ))])),
        }
    }

    #[tool(description = "Configure auto-layout settings for a view.")]
    async fn view_set_auto_layout(
        &self,
        Parameters(params): Parameters<ViewSetAutoLayoutParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: view_set_auto_layout with workspace={}, view={}, enabled={}",
            params.workspace_id, params.view_key, params.enabled
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Parse direction if provided
        let direction = params.direction.as_deref().map(|d: &str| match d.to_lowercase().as_str() {
            "bottomtop" | "bt" => AutoLayoutDirection::BottomTop,
            "leftright" | "lr" => AutoLayoutDirection::LeftRight,
            "rightleft" | "rl" => AutoLayoutDirection::RightLeft,
            _ => AutoLayoutDirection::TopBottom,
        }).unwrap_or(AutoLayoutDirection::TopBottom);

        // Set auto-layout on the view
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                let auto_layout = if params.enabled {
                    Some(AutoLayout { direction, ..Default::default() })
                } else {
                    None
                };

                // Find the view and set auto-layout
                if let Some(view) = workspace.views.system_landscape_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("SystemLandscape")
                } else if let Some(view) = workspace.views.system_context_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("SystemContext")
                } else if let Some(view) = workspace.views.container_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("Container")
                } else if let Some(view) = workspace.views.component_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("Component")
                } else if let Some(view) = workspace.views.dynamic_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("Dynamic")
                } else if let Some(view) = workspace.views.deployment_views.iter_mut()
                    .find(|v| v.properties.key == params.view_key) {
                    view.properties.auto_layout = auto_layout;
                    Some("Deployment")
                } else {
                    None
                }
            } else {
                None
            }
        };

        match result {
            Some(view_type) => {
                let action = if params.enabled {
                    format!("Enabled auto-layout ({:?})", direction)
                } else {
                    "Disabled auto-layout".to_string()
                };
                let response = ViewModifiedResponse {
                    view_key: params.view_key.clone(),
                    action: action.clone(),
                    message: format!(
                        "{} for {} view '{}' in workspace '{}'.",
                        action, view_type, params.view_key, params.workspace_id
                    ),
                };
                info!(
                    "Set auto-layout for view '{}' in workspace '{}'",
                    params.view_key, params.workspace_id
                );
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "View '{}' not found in workspace '{}'.",
                params.view_key, params.workspace_id
            ))])),
        }
    }

    // ========================================================================
    // Documentation Management Tools
    // ========================================================================

    #[tool(description = "Add a documentation section to a workspace. Sections are Markdown-formatted and can be used for technical documentation, guides, or any explanatory content.")]
    async fn docs_add_section(
        &self,
        Parameters(params): Parameters<DocsAddSectionParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: docs_add_section with workspace={}, title={}",
            params.workspace_id, params.title
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Add the documentation section
        let order = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Check if section with this title already exists
                if workspace.documentation.sections.iter().any(|s| s.title.as_deref() == Some(params.title.as_str())) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "Documentation section '{}' already exists in workspace '{}'.",
                        params.title, params.workspace_id
                    ))]));
                }

                // Calculate order (either provided or next available)
                let order = params.order.unwrap_or_else(|| {
                    workspace.documentation.sections.iter()
                        .map(|s| s.order)
                        .max()
                        .map(|max| max + 1)
                        .unwrap_or(1)
                });

                // Create the section
                let section = DocumentationSection {
                    title: Some(params.title.clone()),
                    content: params.content.clone(),
                    format: DocumentationFormat::Markdown,
                    order,
                };

                workspace.documentation.sections.push(section);
                order
            } else {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace '{}' not found.",
                    params.workspace_id
                ))]));
            }
        };

        let response = DocsSectionResponse {
            title: params.title.clone(),
            order,
            message: format!(
                "Added documentation section '{}' (order: {}) to workspace '{}'.",
                params.title, order, params.workspace_id
            ),
        };

        info!(
            "Added documentation section '{}' to workspace '{}'",
            params.title, params.workspace_id
        );

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }

    #[tool(description = "Update an existing documentation section in a workspace. Can update the title and/or content.")]
    async fn docs_update_section(
        &self,
        Parameters(params): Parameters<DocsUpdateSectionParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: docs_update_section with workspace={}, section={}",
            params.workspace_id, params.section_title
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Update the documentation section
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                if let Some(section) = workspace.documentation.sections.iter_mut()
                    .find(|s| s.title.as_deref() == Some(params.section_title.as_str()))
                {
                    let mut changes = Vec::new();

                    if let Some(new_title) = &params.new_title {
                        section.title = Some(new_title.to_string());
                        changes.push(format!("title -> '{}'", new_title));
                    }

                    if let Some(new_content) = &params.new_content {
                        section.content = new_content.to_string();
                        changes.push("content updated".to_string());
                    }

                    if changes.is_empty() {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "No changes specified. Provide new_title and/or new_content."
                        )]));
                    }

                    Some((section.title.clone().unwrap_or_default(), section.order, changes))
                } else {
                    None
                }
            } else {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace '{}' not found.",
                    params.workspace_id
                ))]));
            }
        };

        match result {
            Some((title, order, changes)) => {
                let response = DocsSectionResponse {
                    title,
                    order,
                    message: format!(
                        "Updated documentation section '{}': {}.",
                        params.section_title, changes.join(", ")
                    ),
                };

                info!(
                    "Updated documentation section '{}' in workspace '{}'",
                    params.section_title, params.workspace_id
                );

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "Documentation section '{}' not found in workspace '{}'.",
                params.section_title, params.workspace_id
            ))])),
        }
    }

    #[tool(description = "Remove a documentation section from a workspace.")]
    async fn docs_remove_section(
        &self,
        Parameters(params): Parameters<DocsRemoveSectionParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: docs_remove_section with workspace={}, section={}",
            params.workspace_id, params.section_title
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Remove the documentation section
        let removed = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                let initial_len = workspace.documentation.sections.len();
                workspace.documentation.sections.retain(|s| s.title.as_deref() != Some(params.section_title.as_str()));
                workspace.documentation.sections.len() < initial_len
            } else {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace '{}' not found.",
                    params.workspace_id
                ))]));
            }
        };

        if removed {
            let response = DocsSectionResponse {
                title: params.section_title.clone(),
                order: 0,
                message: format!(
                    "Removed documentation section '{}' from workspace '{}'.",
                    params.section_title, params.workspace_id
                ),
            };

            info!(
                "Removed documentation section '{}' from workspace '{}'",
                params.section_title, params.workspace_id
            );

            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&response).unwrap_or_default(),
            )]))
        } else {
            Ok(CallToolResult::error(vec![Content::text(format!(
                "Documentation section '{}' not found in workspace '{}'.",
                params.section_title, params.workspace_id
            ))]))
        }
    }

    #[tool(description = "List all documentation sections in a workspace.")]
    async fn docs_list_sections(
        &self,
        Parameters(params): Parameters<DocsListSectionsParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: docs_list_sections with workspace={}",
            params.workspace_id
        );

        // Try modified workspace first, then original
        let sections = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                workspace.documentation.sections.clone()
            } else {
                drop(modified);
                // Try to load from registry
                let registry = self.registry.read().await;
                if let Some(ref registry) = *registry {
                    match registry.get_workspace(&params.workspace_id).await {
                        Ok(Some(workspace)) => workspace.documentation.sections.clone(),
                        Ok(None) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Workspace '{}' not found.",
                                params.workspace_id
                            ))]));
                        }
                        Err(e) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Failed to load workspace '{}': {}",
                                params.workspace_id, e
                            ))]));
                        }
                    }
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "No workspace registry available."
                    )]));
                }
            }
        };

        if sections.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No documentation sections found in workspace '{}'.",
                params.workspace_id
            ))]));
        }

        let mut output = format!("# Documentation Sections in '{}'\n\n", params.workspace_id);
        let mut sorted_sections = sections.clone();
        sorted_sections.sort_by_key(|s| s.order);

        for section in &sorted_sections {
            let title = section.title.as_deref().unwrap_or("Untitled");
            output.push_str(&format!(
                "## {} (Order: {})\n\n{}\n\n---\n\n",
                title, section.order,
                if section.content.len() > 200 {
                    format!("{}...", &section.content[..200])
                } else {
                    section.content.clone()
                }
            ));
        }

        output.push_str(&format!("\n**Total sections: {}**", sections.len()));

        info!(
            "Listed {} documentation sections for workspace '{}'",
            sections.len(), params.workspace_id
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }

    // ========================================================================
    // ADR (Architecture Decision Record) Management Tools
    // ========================================================================

    #[tool(description = "Create a new Architecture Decision Record (ADR) in a workspace. ADRs document important architectural decisions with their context, status, and consequences.")]
    async fn adr_create(
        &self,
        Parameters(params): Parameters<AdrCreateParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: adr_create with workspace={}, adr_id={}, title={}",
            params.workspace_id, params.adr_id, params.title
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Parse status
        let status = params.status.as_deref().map(|s: &str| match s.to_lowercase().as_str() {
            "accepted" => DecisionStatus::Accepted,
            "superseded" => DecisionStatus::Superseded,
            "deprecated" => DecisionStatus::Deprecated,
            "rejected" => DecisionStatus::Rejected,
            _ => DecisionStatus::Proposed,
        }).unwrap_or(DecisionStatus::Proposed);

        // Parse date or use today
        let date = params.date.clone().unwrap_or_else(|| {
            chrono::Local::now().format("%Y-%m-%d").to_string()
        });

        // Add the ADR
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                // Check if ADR with this ID already exists
                if workspace.documentation.decisions.iter().any(|d| d.id == params.adr_id) {
                    return Ok(CallToolResult::error(vec![Content::text(format!(
                        "ADR '{}' already exists in workspace '{}'.",
                        params.adr_id, params.workspace_id
                    ))]));
                }

                let decision = Decision {
                    id: params.adr_id.clone(),
                    title: params.title.clone(),
                    content: params.content.clone(),
                    format: DocumentationFormat::Markdown,
                    status: status.clone(),
                    date: date.clone(),
                    links: Vec::new(),
                };

                workspace.documentation.decisions.push(decision);
                Some(status)
            } else {
                None
            }
        };

        match result {
            Some(status) => {
                let response = AdrResponse {
                    adr_id: params.adr_id.clone(),
                    title: params.title.clone(),
                    status: format!("{:?}", status),
                    message: format!(
                        "Created ADR '{}' ({}) with status {:?} in workspace '{}'.",
                        params.adr_id, params.title, status, params.workspace_id
                    ),
                };

                info!(
                    "Created ADR '{}' in workspace '{}'",
                    params.adr_id, params.workspace_id
                );

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))])),
        }
    }

    #[tool(description = "Update an existing Architecture Decision Record (ADR) in a workspace. Can update title, content, or status.")]
    async fn adr_update(
        &self,
        Parameters(params): Parameters<AdrUpdateParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: adr_update with workspace={}, adr_id={}",
            params.workspace_id, params.adr_id
        );

        // Ensure we have a mutable copy
        if self.get_or_create_mutable_workspace(&params.workspace_id).await.is_none() {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Workspace '{}' not found.",
                params.workspace_id
            ))]));
        }

        // Update the ADR
        let result = {
            let mut modified = self.modified_workspaces.write().await;
            if let Some(workspace) = modified.get_mut(&params.workspace_id) {
                if let Some(decision) = workspace.documentation.decisions.iter_mut()
                    .find(|d| d.id == params.adr_id)
                {
                    let mut changes = Vec::new();

                    if let Some(new_title) = &params.new_title {
                        decision.title = new_title.clone();
                        changes.push(format!("title -> '{}'", new_title));
                    }

                    if let Some(new_content) = &params.new_content {
                        decision.content = new_content.clone();
                        changes.push("content updated".to_string());
                    }

                    if let Some(new_status) = &params.new_status {
                        let status = match new_status.to_lowercase().as_str() {
                            "accepted" => DecisionStatus::Accepted,
                            "superseded" => DecisionStatus::Superseded,
                            "deprecated" => DecisionStatus::Deprecated,
                            "rejected" => DecisionStatus::Rejected,
                            _ => DecisionStatus::Proposed,
                        };
                        decision.status = status.clone();
                        changes.push(format!("status -> {:?}", status));
                    }

                    if changes.is_empty() {
                        return Ok(CallToolResult::error(vec![Content::text(
                            "No changes specified. Provide new_title, new_content, and/or new_status."
                        )]));
                    }

                    Some((decision.title.clone(), format!("{:?}", decision.status), changes))
                } else {
                    None
                }
            } else {
                return Ok(CallToolResult::error(vec![Content::text(format!(
                    "Workspace '{}' not found.",
                    params.workspace_id
                ))]));
            }
        };

        match result {
            Some((title, status, changes)) => {
                let response = AdrResponse {
                    adr_id: params.adr_id.clone(),
                    title,
                    status,
                    message: format!(
                        "Updated ADR '{}': {}.",
                        params.adr_id, changes.join(", ")
                    ),
                };

                info!(
                    "Updated ADR '{}' in workspace '{}'",
                    params.adr_id, params.workspace_id
                );

                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            None => Ok(CallToolResult::error(vec![Content::text(format!(
                "ADR '{}' not found in workspace '{}'.",
                params.adr_id, params.workspace_id
            ))])),
        }
    }

    #[tool(description = "List all Architecture Decision Records (ADRs) in a workspace.")]
    async fn adr_list(
        &self,
        Parameters(params): Parameters<AdrListParam>,
    ) -> std::result::Result<CallToolResult, RmcpError> {
        debug!(
            "Tool called: adr_list with workspace={}",
            params.workspace_id
        );

        // Try modified workspace first, then original
        let decisions = {
            let modified = self.modified_workspaces.read().await;
            if let Some(workspace) = modified.get(&params.workspace_id) {
                workspace.documentation.decisions.clone()
            } else {
                drop(modified);
                // Try to load from registry
                let registry = self.registry.read().await;
                if let Some(ref registry) = *registry {
                    match registry.get_workspace(&params.workspace_id).await {
                        Ok(Some(workspace)) => workspace.documentation.decisions.clone(),
                        Ok(None) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Workspace '{}' not found.",
                                params.workspace_id
                            ))]));
                        }
                        Err(e) => {
                            return Ok(CallToolResult::error(vec![Content::text(format!(
                                "Failed to load workspace '{}': {}",
                                params.workspace_id, e
                            ))]));
                        }
                    }
                } else {
                    return Ok(CallToolResult::error(vec![Content::text(
                        "No workspace registry available."
                    )]));
                }
            }
        };

        if decisions.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(format!(
                "No ADRs found in workspace '{}'.",
                params.workspace_id
            ))]));
        }

        let mut output = format!("# Architecture Decision Records in '{}'\n\n", params.workspace_id);
        output.push_str("| ID | Title | Status | Date |\n");
        output.push_str("|---|---|---|---|\n");

        for decision in &decisions {
            output.push_str(&format!(
                "| {} | {} | {:?} | {} |\n",
                decision.id, decision.title, decision.status, decision.date
            ));
        }

        output.push_str(&format!("\n**Total ADRs: {}**", decisions.len()));

        info!(
            "Listed {} ADRs for workspace '{}'",
            decisions.len(), params.workspace_id
        );

        Ok(CallToolResult::success(vec![Content::text(output)]))
    }
}

// Implement ServerHandler for MCP protocol
#[tool_handler]
impl rmcp::ServerHandler for StructurizrMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Structurizr-rs MCP server for C4 architecture modeling. \
                 Read tools: \
                 workspace_list, workspace_load, workspace_get_model, workspace_get_views, \
                 workspace_validate, workspace_export_json, workspace_search. \
                 Render/Export tools: \
                 render_svg, export_plantuml, export_mermaid, export_d2, export_dot. \
                 Model manipulation (create): \
                 model_add_person, model_add_system, model_add_container, model_add_component, model_add_relationship. \
                 Model manipulation (update/delete): \
                 model_update_element, model_remove_element, model_list_pending_changes. \
                 View management: \
                 view_create_system_landscape, view_create_system_context, view_create_container, \
                 view_create_component, view_create_dynamic, view_create_deployment, \
                 view_add_element, view_add_all_elements, view_remove_element, view_set_auto_layout. \
                 Documentation: \
                 docs_add_section, docs_update_section, docs_remove_section, docs_list_sections. \
                 ADR (Architecture Decision Records): \
                 adr_create, adr_update, adr_list. \
                 Persistence: \
                 workspace_save_json (save to JSON), workspace_save_dsl (save to DSL), workspace_discard_changes (revert)."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

impl StructurizrMcpServer {
    #[cfg(feature = "sse")]
    fn get_tool_list(&self) -> Vec<serde_json::Value> {
        // Get all tools from the tool router
        self.tool_router.map.values().map(|route| {
            serde_json::json!({
                "name": route.attr.name,
                "description": route.attr.description,
                "inputSchema": route.attr.input_schema
            })
        }).collect()
    }

    #[cfg(feature = "sse")]
    async fn call_tool(&self, tool_name: &str, _params: serde_json::Value) -> Result<serde_json::Value, String> {
        use std::borrow::Cow;

        // Verify the tool exists in the router
        let _tool_route = self.tool_router.map.get(&Cow::Borrowed(tool_name))
            .ok_or_else(|| format!("Unknown tool: {}", tool_name))?;

        // TODO: Full integration would invoke the tool through rmcp context
        // For now, return a placeholder indicating the tool exists
        Ok(serde_json::json!({
            "content": [{
                "type": "text",
                "text": format!("Tool '{}' found but SSE call routing needs full integration with rmcp context", tool_name)
            }]
        }))
    }

    /// Run the server with stdio transport
    pub async fn run_stdio(self) -> crate::error::Result<()> {
        info!("Starting MCP server on stdio");
        self.initialize().await?;

        info!("MCP server ready, starting stdio transport");

        let service = self.serve(stdio()).await.map_err(|e| {
            crate::error::McpError::InternalError(format!("Failed to start MCP service: {}", e))
        })?;

        service.waiting().await.map_err(|e| {
            crate::error::McpError::InternalError(format!("MCP service error: {}", e))
        })?;

        info!("MCP server shutting down");
        Ok(())
    }

    /// Run the server with WebSocket transport
    #[cfg(feature = "websocket")]
    pub async fn run_websocket(self, addr: std::net::SocketAddr) -> crate::error::Result<()> {
        use futures_util::{SinkExt, StreamExt};
        use tokio::net::TcpListener;
        use tokio_tungstenite::tungstenite::Message;

        info!("Starting MCP server on WebSocket at {}", addr);
        self.initialize().await?;

        let listener = TcpListener::bind(addr).await.map_err(|e| {
            crate::error::McpError::InternalError(format!("Failed to bind WebSocket: {}", e))
        })?;

        info!("MCP WebSocket server listening on ws://{}", addr);

        loop {
            let (stream, peer_addr) = listener.accept().await.map_err(|e| {
                crate::error::McpError::InternalError(format!("Failed to accept connection: {}", e))
            })?;

            let _server = self.clone();
            tokio::spawn(async move {
                info!("New WebSocket connection from {}", peer_addr);

                let ws_stream = match tokio_tungstenite::accept_async(stream).await {
                    Ok(ws) => ws,
                    Err(e) => {
                        tracing::error!("WebSocket handshake failed: {}", e);
                        return;
                    }
                };

                let (write, read) = ws_stream.split();

                // Convert WebSocket to async read/write
                let read_stream = read.filter_map(|msg| async {
                    match msg {
                        Ok(Message::Text(text)) => Some(Ok(text.into_bytes())),
                        Ok(Message::Binary(data)) => Some(Ok(data)),
                        Ok(Message::Close(_)) => None,
                        Ok(_) => None, // Ignore ping/pong
                        Err(e) => Some(Err(std::io::Error::new(std::io::ErrorKind::Other, e))),
                    }
                });

                let write_sink = write.with(|data: Vec<u8>| async move {
                    Ok::<_, tokio_tungstenite::tungstenite::Error>(Message::Text(
                        String::from_utf8_lossy(&data).to_string(),
                    ))
                });

                // Create a channel-based transport
                let (tx, rx) = tokio::sync::mpsc::channel::<String>(100);
                let (response_tx, mut response_rx) = tokio::sync::mpsc::channel::<String>(100);

                // Handle incoming messages
                let read_handle = tokio::spawn(async move {
                    tokio::pin!(read_stream);
                    while let Some(result) = read_stream.next().await {
                        match result {
                            Ok(data) => {
                                if let Ok(text) = String::from_utf8(data) {
                                    if tx.send(text).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("WebSocket read error: {}", e);
                                break;
                            }
                        }
                    }
                });

                // Handle outgoing messages
                let write_handle = tokio::spawn(async move {
                    tokio::pin!(write_sink);
                    while let Some(response) = response_rx.recv().await {
                        if write_sink.send(response.into_bytes()).await.is_err() {
                            break;
                        }
                    }
                });

                // Process MCP messages
                let mut rx = rx;
                while let Some(request) = rx.recv().await {
                    // Parse and handle MCP request
                    debug!("Received MCP request: {}", &request[..request.len().min(100)]);

                    // For now, echo back - full MCP handling would require more integration
                    if response_tx.send(request).await.is_err() {
                        break;
                    }
                }

                let _ = read_handle.await;
                let _ = write_handle.await;
                info!("WebSocket connection closed for {}", peer_addr);
            });
        }
    }

    /// Run the server with HTTP/SSE transport (Streamable HTTP)
    #[cfg(feature = "sse")]
    pub async fn run_http(self, addr: std::net::SocketAddr) -> crate::error::Result<()> {
        use axum::{routing::{get, post}, Router};
        use tower_http::cors::CorsLayer;

        info!("Starting MCP server on HTTP at {}", addr);
        self.initialize().await?;

        // Shared state for HTTP handlers
        let shared_state = Arc::new(self);

        let app = Router::new()
            .route("/mcp", post(handle_mcp_post))
            .route("/mcp/sse", get(handle_mcp_sse))
            .route("/health", get(|| async { "OK" }))
            .layer(CorsLayer::permissive())
            .with_state(shared_state);

        info!("MCP HTTP server listening on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.map_err(|e| {
            crate::error::McpError::InternalError(format!("Failed to bind HTTP: {}", e))
        })?;

        axum::serve(listener, app).await.map_err(|e| {
            crate::error::McpError::InternalError(format!("HTTP server error: {}", e))
        })?;

        Ok(())
    }
}

// Session management structures for future SSE enhancements
// These will be used for multi-client session tracking and request correlation
#[cfg(feature = "sse")]
#[derive(Clone)]
#[allow(dead_code)]
struct SseSession {
    id: String,
    tx: tokio::sync::mpsc::UnboundedSender<Result<axum::response::sse::Event, axum::Error>>,
}

#[cfg(feature = "sse")]
#[allow(dead_code)]
struct SseState {
    sessions: Arc<RwLock<HashMap<String, SseSession>>>,
    request_handlers: Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<serde_json::Value>>>>,
}

#[cfg(feature = "sse")]
#[allow(dead_code)]
impl SseState {
    fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            request_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(feature = "sse")]
async fn handle_mcp_post(
    axum::extract::State(server): axum::extract::State<Arc<StructurizrMcpServer>>,
    jar: axum_extra::extract::CookieJar,
    axum::Json(request): axum::Json<serde_json::Value>,
) -> impl axum::response::IntoResponse {
    use axum_extra::extract::cookie::Cookie;

    debug!("Received MCP POST: {:?}", request);

    // Extract session ID from cookie
    let session_id = jar.get("mcp_session_id")
        .map(|c: &axum_extra::extract::cookie::Cookie| c.value().to_string())
        .unwrap_or_else(|| {
            // Generate new session if not present
            uuid::Uuid::new_v4().to_string()
        });

    // Extract request ID - preserve the original value type (can be string, number, or null)
    let request_id = request.get("id")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    // Extract method
    let method = request.get("method")
        .and_then(|m| m.as_str())
        .unwrap_or("");

    // Process the request based on method
    let response = match method {
        "initialize" => {
            // MCP protocol initialization - return server capabilities
            info!("Handling MCP initialize request");
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        },
                        "resources": {
                            "subscribe": false,
                            "listChanged": false
                        },
                        "prompts": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "structurizr-mcp",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                }
            })
        }
        "notifications/initialized" | "initialized" => {
            // Client acknowledges initialization - no response needed for notifications
            // But if it has an ID, respond with success
            if !request_id.is_null() {
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": request_id,
                    "result": {}
                })
            } else {
                // Notification - return empty but don't error
                serde_json::json!({})
            }
        }
        "tools/list" => {
            // Return tool list from the tool router
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "tools": server.get_tool_list()
                }
            })
        }
        "tools/call" => {
            // Extract tool name and parameters from MCP protocol
            let params = request.get("params").cloned().unwrap_or(serde_json::json!({}));
            let tool_name = params.get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let tool_args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

            // Call the tool through the router
            match server.call_tool(tool_name, tool_args).await {
                Ok(result) => {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "result": result
                    })
                }
                Err(e) => {
                    serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": request_id,
                        "error": {
                            "code": -32603,
                            "message": format!("Tool execution failed: {}", e)
                        }
                    })
                }
            }
        }
        "resources/list" => {
            // Return empty resources list for now
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "resources": []
                }
            })
        }
        "prompts/list" => {
            // Return empty prompts list for now
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "prompts": []
                }
            })
        }
        _ => {
            // Unknown method - log it for debugging
            debug!("Unknown MCP method: {}", method);
            serde_json::json!({
                "jsonrpc": "2.0",
                "id": request_id,
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                }
            })
        }
    };

    // Set session cookie if new
    let mut jar = jar;
    if !jar.get("mcp_session_id").is_some() {
        jar = jar.add(Cookie::build(("mcp_session_id", session_id))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .path("/")
            .build());
    }

    (jar, axum::Json(response))
}

#[cfg(feature = "sse")]
async fn handle_mcp_sse(
    axum::extract::State(_server): axum::extract::State<Arc<StructurizrMcpServer>>,
    axum::extract::Host(host): axum::extract::Host,
    jar: axum_extra::extract::CookieJar,
) -> impl axum::response::IntoResponse {
    use axum::response::sse::{Event, KeepAlive, Sse};
    use axum_extra::extract::cookie::Cookie;
    use futures_util::stream::{self, StreamExt};
    use std::time::Duration;

    // Extract or create session ID
    let session_id = jar.get("mcp_session_id")
        .map(|c: &axum_extra::extract::cookie::Cookie| c.value().to_string())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

    info!("SSE connection established for session: {}", session_id);

    // Build the POST endpoint URL - MCP protocol requires sending this to the client
    let endpoint_url = format!("http://{}/mcp", host);

    // Create the initial endpoint event
    let initial_event = Event::default()
        .event("endpoint")
        .data(endpoint_url);

    // Create a stream that sends the initial event, then stays open forever
    // The keep-alive will send periodic pings to maintain the connection
    let initial_stream = stream::once(async move {
        Ok::<_, axum::Error>(initial_event)
    });

    // Create an infinite pending stream that never yields but keeps the connection alive
    let pending_stream = stream::pending::<Result<Event, axum::Error>>();

    // Chain them together: initial event followed by infinite pending
    let combined_stream = initial_stream.chain(pending_stream);

    // Set session cookie
    let mut jar = jar;
    if !jar.get("mcp_session_id").is_some() {
        jar = jar.add(Cookie::build(("mcp_session_id", session_id.clone()))
            .http_only(true)
            .same_site(axum_extra::extract::cookie::SameSite::Lax)
            .path("/")
            .build());
    }

    // Return SSE stream with cookie jar and keep-alive
    // The keep-alive will send periodic comments to maintain the connection
    (
        jar,
        Sse::new(combined_stream).keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(15))
                .text(":"),  // SSE comment format for keep-alive
        ),
    )
}
