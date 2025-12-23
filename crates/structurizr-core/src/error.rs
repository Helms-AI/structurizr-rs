//! Error types for structurizr-core.

use thiserror::Error;

/// Core error type for structurizr operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Invalid relationship: {0}")]
    InvalidRelationship(String),

    #[error("Duplicate identifier: {0}")]
    DuplicateIdentifier(String),

    #[error("Invalid view configuration: {0}")]
    InvalidView(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("Theme error: {0}")]
    Theme(String),
}

/// Result type alias for structurizr operations.
pub type Result<T> = std::result::Result<T, Error>;
