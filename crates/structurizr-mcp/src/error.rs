//! Error types for the MCP server

use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("View not found: {0}")]
    ViewNotFound(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("DSL error: {0}")]
    DslError(#[from] structurizr_dsl::Error),

    #[error("Core error: {0}")]
    CoreError(#[from] structurizr_core::Error),

    #[error("Export error: {0}")]
    ExportError(#[from] structurizr_export::Error),

    #[error("Render error: {0}")]
    RenderError(#[from] structurizr_render::Error),

    #[error("Web error: {0}")]
    WebError(#[from] structurizr_web::Error),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, McpError>;