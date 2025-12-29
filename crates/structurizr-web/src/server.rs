//! Web server implementation.

use axum::{
    routing::{get, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::editor;
use crate::error::Result;
use crate::handlers;
use crate::state::{AppState, Config};

/// Structurizr web server.
pub struct Server {
    config: Config,
}

impl Server {
    /// Create a new server with the given configuration.
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Create a server with default configuration for a workspaces directory.
    pub fn with_workspaces_dir(workspaces_dir: impl Into<std::path::PathBuf>) -> Self {
        Self::new(Config::default().with_workspaces_dir(workspaces_dir))
    }

    /// Build the router for multi-workspace mode.
    ///
    /// Workspace IDs can contain slashes (e.g., "small/startup-saas"), so we use
    /// a path parameter approach. Single-level workspace IDs use :id, while nested
    /// ones use :category/:id pattern.
    fn build_multi_router(state: AppState) -> Router {
        Router::new()
            // Workspaces index
            .route("/", get(handlers::workspaces_index))
            .route("/health", get(handlers::health))

            // Single-level workspace routes (e.g., /w/my-workspace)
            .route("/w/:workspace_id", get(handlers::workspace_home))
            .route("/w/:workspace_id/view/:key", get(handlers::workspace_view_diagram))
            .route("/w/:workspace_id/view/:key/animate", get(handlers::workspace_view_animated))
            .route("/w/:workspace_id/view/:key/svg", get(handlers::workspace_render_svg))
            .route("/w/:workspace_id/view/:key/plantuml", get(handlers::workspace_export_plantuml))
            .route("/w/:workspace_id/view/:key/mermaid", get(handlers::workspace_export_mermaid))
            .route("/w/:workspace_id/view/:key/dot", get(handlers::workspace_export_dot))
            .route("/w/:workspace_id/view/:key/d2", get(handlers::workspace_export_d2))
            .route("/w/:workspace_id/edit/:key", get(handlers::workspace_edit_diagram))
            .route("/w/:workspace_id/ws/edit/:key", get(editor::workspace_editor_ws))
            .route("/w/:workspace_id/docs", get(handlers::workspace_documentation))
            .route("/w/:workspace_id/search", get(handlers::workspace_search_page))
            .route("/w/:workspace_id/tree", get(handlers::workspace_tree_view))
            .route("/w/:workspace_id/presentation", get(handlers::workspace_presentation))
            .route("/w/:workspace_id/explore", get(handlers::workspace_explore))
            .route("/w/:workspace_id/api/workspace", get(handlers::workspace_get_json))
            .route("/w/:workspace_id/api/validate", get(handlers::workspace_validate))
            .route("/w/:workspace_id/api/search", get(handlers::workspace_search_api))
            .route("/w/:workspace_id/api/view/:view_key/positions", get(editor::workspace_get_positions))
            .route("/w/:workspace_id/api/view/:view_key/positions", put(editor::workspace_batch_update_positions))
            .route("/w/:workspace_id/export/json", get(handlers::workspace_export_json))

            // Two-level workspace routes (e.g., /w/small/startup-saas)
            .route("/w/:category/:workspace_id", get(handlers::workspace_home_nested))
            .route("/w/:category/:workspace_id/view/:key", get(handlers::workspace_view_diagram_nested))
            .route("/w/:category/:workspace_id/view/:key/animate", get(handlers::workspace_view_animated_nested))
            .route("/w/:category/:workspace_id/view/:key/svg", get(handlers::workspace_render_svg_nested))
            .route("/w/:category/:workspace_id/view/:key/plantuml", get(handlers::workspace_export_plantuml_nested))
            .route("/w/:category/:workspace_id/view/:key/mermaid", get(handlers::workspace_export_mermaid_nested))
            .route("/w/:category/:workspace_id/view/:key/dot", get(handlers::workspace_export_dot_nested))
            .route("/w/:category/:workspace_id/view/:key/d2", get(handlers::workspace_export_d2_nested))
            .route("/w/:category/:workspace_id/edit/:key", get(handlers::workspace_edit_diagram_nested))
            .route("/w/:category/:workspace_id/ws/edit/:key", get(editor::workspace_editor_ws_nested))
            .route("/w/:category/:workspace_id/docs", get(handlers::workspace_documentation_nested))
            .route("/w/:category/:workspace_id/search", get(handlers::workspace_search_page_nested))
            .route("/w/:category/:workspace_id/tree", get(handlers::workspace_tree_view_nested))
            .route("/w/:category/:workspace_id/presentation", get(handlers::workspace_presentation_nested))
            .route("/w/:category/:workspace_id/explore", get(handlers::workspace_explore_nested))
            .route("/w/:category/:workspace_id/api/workspace", get(handlers::workspace_get_json_nested))
            .route("/w/:category/:workspace_id/api/validate", get(handlers::workspace_validate_nested))
            .route("/w/:category/:workspace_id/api/search", get(handlers::workspace_search_api_nested))
            .route("/w/:category/:workspace_id/api/view/:view_key/positions", get(editor::workspace_get_positions_nested))
            .route("/w/:category/:workspace_id/api/view/:view_key/positions", put(editor::workspace_batch_update_positions_nested))
            .route("/w/:category/:workspace_id/export/json", get(handlers::workspace_export_json_nested))

            // Middleware
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .with_state(state)
    }

    /// Run the server.
    pub async fn run(self) -> Result<()> {
        let state = AppState::new(self.config.clone());

        // Initialize multi-workspace mode
        let workspaces_dir = self.config.workspaces_dir();
        info!("Starting in multi-workspace mode");
        info!("Discovering workspaces in {:?}", workspaces_dir);
        if let Err(e) = state.initialize().await {
            tracing::error!("Failed to discover workspaces: {}", e);
        } else {
            let count = state.list_workspaces().await.len();
            info!("Discovered {} workspace(s)", count);
        }

        // Start file watcher for auto-reload
        if let Err(e) = state.start_multi_watcher().await {
            tracing::warn!("Failed to start file watcher: {}. Auto-reload disabled.", e);
        } else {
            info!("File watcher started - workspaces will auto-reload on changes");
        }

        // Start auto-save background task
        let _auto_save_handle = editor::start_auto_save_task(state.clone());
        info!("Auto-save task started - positions will be saved automatically");

        let app = Self::build_multi_router(state);

        let addr = self.config.address();
        info!("Starting Structurizr server at http://{}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    /// Run the server on a specific port.
    pub async fn run_on_port(workspaces_dir: impl Into<std::path::PathBuf>, port: u16) -> Result<()> {
        let config = Config::default()
            .with_workspaces_dir(workspaces_dir)
            .with_port(port);

        Server::new(config).run().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.port, 8080);
        assert_eq!(config.host, "127.0.0.1");
    }

    #[test]
    fn test_config_builder() {
        let config = Config::default()
            .with_port(3000)
            .with_host("0.0.0.0");

        assert_eq!(config.port, 3000);
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.address(), "0.0.0.0:3000");
    }
}
