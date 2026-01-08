//! Error types for the web server.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Server error: {0}")]
    Server(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(String),

    #[error("View not found: {0}")]
    ViewNotFound(String),

    #[error("Documentation not found: {0}")]
    DocNotFound(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] structurizr_core::Error),

    #[error("DSL error: {0}")]
    Dsl(#[from] structurizr_dsl::Error),

    #[error("Render error: {0}")]
    Render(#[from] structurizr_render::Error),

    #[error("Export error: {0}")]
    Export(#[from] structurizr_export::Error),

    #[error("File watcher error: {0}")]
    Watcher(#[from] notify::Error),

    #[error("GitHub error: {0}")]
    GitHub(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            Error::NotFound(_) | Error::WorkspaceNotFound(_) | Error::ViewNotFound(_) | Error::DocNotFound(_) => {
                (StatusCode::NOT_FOUND, self.to_string())
            }
            Error::BadRequest(_) | Error::Parse(_) | Error::Dsl(_) | Error::GitHub(_) => {
                (StatusCode::BAD_REQUEST, self.to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, message).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
