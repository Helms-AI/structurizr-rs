//! Model Context Protocol (MCP) server for structurizr-rs
//!
//! This crate provides an MCP server that exposes structurizr-rs functionality
//! to AI assistants and other MCP clients. It enables natural language interaction
//! with C4 architecture diagrams and workspace management.

pub mod error;
pub mod server;
pub mod server_simple;  // Fallback simplified implementation
pub mod state;
pub mod tools;
pub mod transport;

pub use error::{McpError, Result};
pub use server::StructurizrMcpServer;
pub use state::{McpServerState, WorkspaceInfo};