//! Scripting and plugin support for structurizr-rs.
//!
//! This crate provides Lua scripting and WASM plugin capabilities for extending
//! Structurizr workspaces programmatically. It maintains backwards compatibility
//! with Structurizr DSL's `!script` directive while using Lua instead of Groovy/Kotlin.
//!
//! # Features
//!
//! - **Lua Scripting**: Native Lua 5.4 runtime for inline scripts
//! - **Groovy Compatibility**: Automatic transpilation of Groovy scripts to Lua
//! - **WASM Plugins**: (Optional) Load and execute WASM plugins for advanced extensibility
//!
//! # Example
//!
//! ```ignore
//! use structurizr_scripting::{ScriptEngine, ScriptConfig};
//! use structurizr_core::Workspace;
//!
//! let mut workspace = Workspace::new("Test", "Test workspace");
//! let engine = ScriptEngine::new(ScriptConfig::default())?;
//!
//! // Execute inline Lua script
//! engine.execute_lua(&mut workspace, r#"
//!     workspace.name = workspace.name .. " (Modified)"
//!     local sys = workspace:addSoftwareSystem("Dynamic", "Created by script")
//! "#)?;
//! ```

mod error;
mod engine;
mod api;
mod sandbox;
mod transpiler;
mod plugin;

pub use error::{ScriptError, Result};
pub use engine::{ScriptEngine, ScriptConfig, ScriptLanguage};
pub use sandbox::SandboxConfig;
pub use transpiler::GroovyTranspiler;
pub use plugin::{PluginManifest, PluginMetadata, PluginCapabilities, PluginInfo};

#[cfg(feature = "wasm")]
pub use plugin::{PluginEngine, PluginEngineConfig};

/// Re-export mlua for advanced use cases
pub use mlua;
