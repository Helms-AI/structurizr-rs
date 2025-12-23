//! Error types for the DSL parser.

use thiserror::Error;

/// Location in source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// A parsing error with location information.
#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
    pub location: Option<Location>,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "{}: {}", loc, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Error type for DSL parsing operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parse error: {0}")]
    Parse(ParseError),

    #[error("Lexer error at {location}: {message}")]
    Lexer { message: String, location: Location },

    #[error("Undefined identifier: {0}")]
    UndefinedIdentifier(String),

    #[error("Duplicate identifier: {0}")]
    DuplicateIdentifier(String),

    #[error("Invalid reference: {0}")]
    InvalidReference(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Core error: {0}")]
    Core(#[from] structurizr_core::Error),
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

/// Result type alias for DSL operations.
pub type Result<T> = std::result::Result<T, Error>;
