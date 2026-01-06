//! # structurizr-web
//!
//! Web server for Structurizr Lite.
//!
//! Provides a web interface for viewing and editing architecture diagrams.

pub mod discovery;
pub mod editor;
pub mod error;
pub mod explore;
pub mod handlers;
pub mod hot_reload;
pub mod layout;
pub mod markdown;
pub mod notes;
pub mod positions;
pub mod server;
pub mod state;
pub mod watcher;

#[cfg(feature = "mcp-proxy")]
pub mod mcp_proxy;

pub use discovery::{WorkspaceInfo, WorkspaceRegistry};
pub use editor::EditorState;
pub use error::{Error, Result};
pub use layout::{ContentType, LayoutConfig, NavItem, generate_page_layout};
pub use notes::{AddNoteRequest, Note, NotesFile, ViewNotes};
pub use server::Server;
pub use state::{AppState, Config};
