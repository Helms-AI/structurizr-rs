//! Simplified MCP server implementation for testing

use std::path::PathBuf;
use std::sync::Arc;
// use serde_json::{json, Value};  // Will be used in full implementation
use tracing::info;

use crate::state::McpServerState;
use crate::error::Result;

/// The main MCP server for structurizr-rs
pub struct StructurizrMcpServer {
    state: Arc<McpServerState>,
}

impl StructurizrMcpServer {
    /// Create a new MCP server
    pub fn new(workspaces_dir: PathBuf) -> Result<Self> {
        let state = Arc::new(McpServerState::new(workspaces_dir));
        Ok(Self { state })
    }

    /// Initialize the server (load workspace registry)
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing MCP server");
        self.state.initialize_registry().await?;
        let workspaces = self.state.list_workspaces().await?;
        info!("Found {} workspaces", workspaces.len());
        Ok(())
    }

    /// Run the server with stdio transport (simplified for now)
    pub async fn run_stdio(self) -> Result<()> {
        info!("Starting MCP server on stdio");
        self.initialize().await?;

        // For now, just log and wait
        info!("MCP server ready (simplified mode)");

        // Keep server running
        tokio::signal::ctrl_c().await?;
        info!("Shutting down MCP server");

        Ok(())
    }
}