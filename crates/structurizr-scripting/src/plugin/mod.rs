//! WASM Plugin System for structurizr-rs.
//!
//! This module provides support for loading and executing WASM plugins
//! that can extend workspace functionality.
//!
//! # Plugin Manifest
//!
//! Plugins are defined by a `plugin.toml` manifest file:
//!
//! ```toml
//! [plugin]
//! name = "my-plugin"
//! version = "1.0.0"
//! description = "A custom plugin"
//! wasm = "plugin.wasm"
//!
//! [capabilities]
//! read_workspace = true
//! modify_workspace = true
//! ```

#[cfg(feature = "wasm")]
mod wasm_runtime;

#[cfg(feature = "wasm")]
pub use wasm_runtime::*;

#[cfg(feature = "auto-build-plugins")]
pub mod builder;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Plugin manifest structure (plugin.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    /// Plugin metadata
    pub plugin: PluginMetadata,
    /// Plugin capabilities/permissions
    #[serde(default)]
    pub capabilities: PluginCapabilities,
}

/// Plugin metadata from manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin name
    pub name: String,
    /// Plugin version (semver)
    pub version: String,
    /// Plugin description
    #[serde(default)]
    pub description: Option<String>,
    /// Path to the WASM binary (relative to manifest)
    pub wasm: String,
    /// Plugin author
    #[serde(default)]
    pub author: Option<String>,
    /// Plugin homepage/repository
    #[serde(default)]
    pub homepage: Option<String>,
}

/// Plugin capabilities/permissions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    /// Can read workspace data
    #[serde(default = "default_true")]
    pub read_workspace: bool,
    /// Can modify workspace data
    #[serde(default)]
    pub modify_workspace: bool,
    /// Can access the filesystem (sandboxed)
    #[serde(default)]
    pub filesystem: bool,
    /// Can make network requests
    #[serde(default)]
    pub network: bool,
    /// Maximum memory in bytes (0 = default)
    #[serde(default)]
    pub max_memory: usize,
    /// Maximum execution time in milliseconds (0 = default)
    #[serde(default)]
    pub max_time: u64,
}

impl Default for PluginCapabilities {
    fn default() -> Self {
        Self {
            read_workspace: true, // Default to read-only access
            modify_workspace: false,
            filesystem: false,
            network: false,
            max_memory: 0,
            max_time: 0,
        }
    }
}

fn default_true() -> bool {
    true
}

impl PluginManifest {
    /// Load a plugin manifest from a TOML file.
    #[cfg(feature = "wasm")]
    pub fn load(path: &std::path::Path) -> Result<Self, crate::error::ScriptError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| crate::error::ScriptError::FileNotFound(format!("{}: {}", path.display(), e)))?;

        toml::from_str(&content)
            .map_err(|e| crate::error::ScriptError::Configuration(format!("Invalid plugin manifest: {}", e)))
    }

    /// Load manifest from string content.
    #[cfg(feature = "wasm")]
    pub fn from_str(content: &str) -> Result<Self, crate::error::ScriptError> {
        toml::from_str(content)
            .map_err(|e| crate::error::ScriptError::Configuration(format!("Invalid plugin manifest: {}", e)))
    }

    /// Get the WASM binary path relative to a base directory.
    pub fn wasm_path(&self, base_dir: &std::path::Path) -> PathBuf {
        base_dir.join(&self.plugin.wasm)
    }
}

/// Information about a loaded plugin.
#[derive(Debug, Clone)]
pub struct PluginInfo {
    /// Plugin name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: Option<String>,
    /// Path to the plugin directory
    pub path: PathBuf,
    /// Plugin capabilities
    pub capabilities: PluginCapabilities,
}

impl From<&PluginManifest> for PluginInfo {
    fn from(manifest: &PluginManifest) -> Self {
        Self {
            name: manifest.plugin.name.clone(),
            version: manifest.plugin.version.clone(),
            description: manifest.plugin.description.clone(),
            path: PathBuf::new(),
            capabilities: manifest.capabilities.clone(),
        }
    }
}

#[cfg(all(test, feature = "wasm"))]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manifest_defaults() {
        let toml_str = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
wasm = "plugin.wasm"
"#;
        let manifest: PluginManifest = toml::from_str(toml_str).unwrap();

        assert_eq!(manifest.plugin.name, "test-plugin");
        assert_eq!(manifest.plugin.version, "1.0.0");
        assert_eq!(manifest.plugin.wasm, "plugin.wasm");
        assert!(manifest.capabilities.read_workspace);
        assert!(!manifest.capabilities.modify_workspace);
        assert!(!manifest.capabilities.filesystem);
        assert!(!manifest.capabilities.network);
    }

    #[test]
    fn test_plugin_manifest_full() {
        let toml_str = r#"
[plugin]
name = "advanced-plugin"
version = "2.1.0"
description = "An advanced plugin"
wasm = "build/plugin.wasm"
author = "Test Author"
homepage = "https://example.com"

[capabilities]
read_workspace = true
modify_workspace = true
filesystem = true
max_memory = 104857600
max_time = 5000
"#;
        let manifest: PluginManifest = toml::from_str(toml_str).unwrap();

        assert_eq!(manifest.plugin.name, "advanced-plugin");
        assert_eq!(manifest.plugin.description, Some("An advanced plugin".to_string()));
        assert!(manifest.capabilities.modify_workspace);
        assert!(manifest.capabilities.filesystem);
        assert_eq!(manifest.capabilities.max_memory, 104857600);
        assert_eq!(manifest.capabilities.max_time, 5000);
    }
}

#[cfg(test)]
mod basic_tests {
    use super::*;

    #[test]
    fn test_plugin_capabilities_default() {
        let caps = PluginCapabilities::default();
        assert!(caps.read_workspace); // Default is true
        assert!(!caps.modify_workspace);
        assert!(!caps.filesystem);
        assert!(!caps.network);
    }

    #[test]
    fn test_plugin_info_from_manifest() {
        let manifest = PluginManifest {
            plugin: PluginMetadata {
                name: "test".to_string(),
                version: "1.0.0".to_string(),
                description: Some("Test plugin".to_string()),
                wasm: "test.wasm".to_string(),
                author: None,
                homepage: None,
            },
            capabilities: PluginCapabilities::default(),
        };

        let info = PluginInfo::from(&manifest);
        assert_eq!(info.name, "test");
        assert_eq!(info.version, "1.0.0");
    }
}
