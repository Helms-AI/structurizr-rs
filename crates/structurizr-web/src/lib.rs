//! # structurizr-web
//!
//! Web server for Structurizr Lite.
//!
//! Provides a web interface for viewing and editing architecture diagrams.

pub mod editor;
pub mod error;
pub mod handlers;
pub mod markdown;
pub mod server;
pub mod state;
pub mod watcher;

pub use editor::EditorState;
pub use error::{Error, Result};
pub use server::Server;
pub use state::{AppState, Config};
