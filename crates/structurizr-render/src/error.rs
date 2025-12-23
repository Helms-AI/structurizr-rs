//! Error types for rendering.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Rendering error: {0}")]
    Render(String),

    #[error("Layout error: {0}")]
    Layout(String),

    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("View not found: {0}")]
    ViewNotFound(String),

    #[error("Core error: {0}")]
    Core(#[from] structurizr_core::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
