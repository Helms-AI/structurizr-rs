//! Error types for export operations.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Export error: {0}")]
    Export(String),

    #[error("View not found: {0}")]
    ViewNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Core error: {0}")]
    Core(#[from] structurizr_core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
