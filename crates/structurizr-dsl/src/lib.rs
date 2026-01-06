//! # structurizr-dsl
//!
//! Parser for the Structurizr DSL (Domain Specific Language).
//!
//! This crate provides a parser that reads `.dsl` files and produces
//! a `Workspace` from `structurizr-core`.
//!
//! ## Example
//!
//! ```rust,ignore
//! use structurizr_dsl::parse;
//!
//! let dsl = r#"
//! workspace "My System" {
//!     model {
//!         user = person "User"
//!         system = softwareSystem "My System"
//!         user -> system "Uses"
//!     }
//! }
//! "#;
//!
//! let workspace = parse(dsl)?;
//! ```

pub mod error;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod validation;
pub mod serializer;

pub use error::{Error, Result, ParseError};
pub use parser::{parse, parse_with_base_path};
pub use validation::{
    validate_workspace, validate_workspace_with_config, Severity, ValidationConfig,
    ValidationIssue, ValidationResult,
};
pub use serializer::{serialize, serialize_with_options, DslSerializer, SerializerOptions};
