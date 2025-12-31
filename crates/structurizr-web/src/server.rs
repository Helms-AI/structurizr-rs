//! Web server implementation.

use axum::{
    routing::{get, post, put},
    Router,
};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::editor;
use crate::error::Result;
use crate::handlers;
use crate::hot_reload;
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
    /// Uses wildcard routing to support workspace IDs of any depth.
    /// For example: /w/my-workspace, /w/small/startup-saas, /w/team/project/app
    fn build_multi_router(state: AppState) -> Router {
        Router::new()
            // Workspaces index
            .route("/", get(handlers::workspaces_index))
            .route("/health", get(handlers::health))

            // Hot-reload WebSocket endpoint
            .route("/ws/reload", get(hot_reload::hot_reload_ws_handler))

            // Unified wildcard routes for all workspace operations
            // The dispatcher parses the path to extract workspace_id and remaining segments
            .route("/w/*path", get(handlers::workspace_dispatch))
            .route("/w/*path", put(handlers::workspace_dispatch_put))
            .route("/w/*path", post(handlers::workspace_dispatch_post))

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
