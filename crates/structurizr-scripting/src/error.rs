//! Error types for the scripting module.

use thiserror::Error;

/// Result type for scripting operations.
pub type Result<T> = std::result::Result<T, ScriptError>;

/// Errors that can occur during script execution.
#[derive(Error, Debug)]
pub enum ScriptError {
    /// Lua runtime error
    #[error("Lua error: {0}")]
    Lua(#[from] mlua::Error),

    /// Script execution timeout
    #[error("Script execution timed out after {0} seconds")]
    Timeout(u64),

    /// Script exceeded memory limit
    #[error("Script exceeded memory limit of {0} bytes")]
    MemoryLimit(usize),

    /// Script attempted forbidden operation
    #[error("Script attempted forbidden operation: {0}")]
    Forbidden(String),

    /// Script file not found
    #[error("Script file not found: {0}")]
    FileNotFound(String),

    /// Script syntax error
    #[error("Script syntax error: {0}")]
    Syntax(String),

    /// Transpilation error (e.g., Groovy to Lua)
    #[error("Transpilation error: {0}")]
    Transpilation(String),

    /// Unsupported script language
    #[error("Unsupported script language: {0}")]
    UnsupportedLanguage(String),

    /// Invalid script configuration
    #[error("Invalid script configuration: {0}")]
    Configuration(String),

    /// Workspace modification error
    #[error("Failed to modify workspace: {0}")]
    WorkspaceModification(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
