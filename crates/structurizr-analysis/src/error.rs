//! Error types for the analysis engine.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse { file: String, message: String },

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Analysis failed: {0}")]
    Analysis(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Tree-sitter error: {0}")]
    TreeSitter(String),

    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),

    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
