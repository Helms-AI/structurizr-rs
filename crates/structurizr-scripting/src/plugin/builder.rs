//! Plugin builder for automatic compilation of WASM plugins
//!
//! This module provides automatic detection and compilation of plugins
//! written in different languages (Rust, C, AssemblyScript, Go, Zig).
//! It checks timestamps to only rebuild when source files are newer
//! than the compiled WASM binary.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::SystemTime;

use crate::error::{Result, ScriptError};

/// Supported plugin languages
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Language {
    Rust,
    C,
    AssemblyScript,
    Go,
    Zig,
}

/// Plugin builder for automatic compilation
pub struct PluginBuilder {
    workspaces_dir: PathBuf,
}

impl PluginBuilder {
    /// Create a new plugin builder
    pub fn new(workspaces_dir: PathBuf) -> Self {
        Self { workspaces_dir }
    }

    /// Check if a plugin needs rebuilding by comparing timestamps
    pub fn needs_rebuild(&self, plugin_dir: &Path) -> bool {
        let wasm_path = plugin_dir.join("plugin.wasm");

        // If WASM doesn't exist, definitely needs building
        if !wasm_path.exists() {
            return true;
        }

        // Get WASM modification time
        let wasm_modified = wasm_path
            .metadata()
            .and_then(|m| m.modified())
            .unwrap_or(SystemTime::UNIX_EPOCH);

        // Check source file timestamps based on language
        self.get_source_modified(plugin_dir)
            .map(|src_time| src_time > wasm_modified)
            .unwrap_or(true) // Rebuild if we can't determine source time
    }

    /// Get the most recent modification time of source files
    fn get_source_modified(&self, plugin_dir: &Path) -> Option<SystemTime> {
        let lang = self.detect_language(plugin_dir)?;

        let source_patterns = match lang {
            Language::Rust => vec!["src/**/*.rs", "Cargo.toml"],
            Language::C => vec!["src/**/*.c", "src/**/*.h", "Makefile"],
            Language::AssemblyScript => vec!["assembly/**/*.ts", "package.json", "asconfig.json"],
            Language::Go => vec!["**/*.go", "go.mod"],
            Language::Zig => vec!["src/**/*.zig", "build.zig"],
        };

        // Find most recent modification time among all source files
        let mut most_recent = SystemTime::UNIX_EPOCH;

        for pattern in source_patterns {
            if let Ok(paths) = glob::glob(&plugin_dir.join(pattern).to_string_lossy()) {
                for entry in paths.flatten() {
                    if let Ok(metadata) = entry.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            if modified > most_recent {
                                most_recent = modified;
                            }
                        }
                    }
                }
            }
        }

        if most_recent == SystemTime::UNIX_EPOCH {
            None
        } else {
            Some(most_recent)
        }
    }

    /// Detect the language of a plugin based on project files
    pub fn detect_language(&self, plugin_dir: &Path) -> Option<Language> {
        if plugin_dir.join("Cargo.toml").exists() {
            Some(Language::Rust)
        } else if plugin_dir.join("Makefile").exists()
            && plugin_dir.join("src").join("main.c").exists() {
            Some(Language::C)
        } else if plugin_dir.join("package.json").exists()
            && plugin_dir.join("asconfig.json").exists() {
            Some(Language::AssemblyScript)
        } else if plugin_dir.join("go.mod").exists() {
            Some(Language::Go)
        } else if plugin_dir.join("build.zig").exists() {
            Some(Language::Zig)
        } else {
            None
        }
    }

    /// Build a plugin based on detected language
    pub async fn build_plugin(&self, plugin_dir: &Path) -> Result<()> {
        let lang = self.detect_language(plugin_dir)
            .ok_or_else(|| ScriptError::Configuration(
                format!("Unable to detect plugin language in {:?}", plugin_dir)
            ))?;

        println!("Building {} plugin in {:?}",
            match lang {
                Language::Rust => "Rust",
                Language::C => "C",
                Language::AssemblyScript => "AssemblyScript",
                Language::Go => "Go",
                Language::Zig => "Zig",
            },
            plugin_dir
        );

        match lang {
            Language::Rust => self.build_rust(plugin_dir).await,
            Language::C => self.build_c(plugin_dir).await,
            Language::AssemblyScript => self.build_assemblyscript(plugin_dir).await,
            Language::Go => self.build_go(plugin_dir).await,
            Language::Zig => self.build_zig(plugin_dir).await,
        }
    }

    /// Build a Rust plugin
    async fn build_rust(&self, dir: &Path) -> Result<()> {
        // Build with cargo
        let output = Command::new("cargo")
            .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
            .current_dir(dir)
            .output()
            .map_err(|e| ScriptError::Configuration(
                format!("Failed to run cargo: {}", e)
            ))?;

        if !output.status.success() {
            return Err(ScriptError::Configuration(
                format!("Cargo build failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        // Copy the built WASM to plugin.wasm
        let wasm_name = dir.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("plugin")
            .replace('-', "_");

        let source = dir.join("target/wasm32-unknown-unknown/release")
            .join(format!("{}.wasm", wasm_name));
        let dest = dir.join("plugin.wasm");

        std::fs::copy(&source, &dest).map_err(|e| ScriptError::Configuration(
            format!("Failed to copy WASM from {:?} to {:?}: {}", source, dest, e)
        ))?;

        println!("✓ Built Rust plugin: {:?}", dest);
        Ok(())
    }

    /// Build a C plugin with Emscripten
    async fn build_c(&self, dir: &Path) -> Result<()> {
        let output = Command::new("emcc")
            .args(&["src/main.c", "-o", "plugin.wasm", "-s", "STANDALONE_WASM", "-s", "ERROR_ON_UNDEFINED_SYMBOLS=0"])
            .current_dir(dir)
            .output()
            .map_err(|e| ScriptError::Configuration(
                format!("Failed to run emcc: {}. Make sure Emscripten is installed.", e)
            ))?;

        if !output.status.success() {
            return Err(ScriptError::Configuration(
                format!("Emscripten build failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        println!("✓ Built C plugin: {:?}", dir.join("plugin.wasm"));
        Ok(())
    }

    /// Build an AssemblyScript plugin
    async fn build_assemblyscript(&self, dir: &Path) -> Result<()> {
        // Install dependencies if needed
        if !dir.join("node_modules").exists() {
            let output = Command::new("npm")
                .arg("install")
                .current_dir(dir)
                .output()
                .map_err(|e| ScriptError::Configuration(
                    format!("Failed to run npm install: {}", e)
                ))?;

            if !output.status.success() {
                return Err(ScriptError::Configuration(
                    format!("npm install failed: {}", String::from_utf8_lossy(&output.stderr))
                ));
            }
        }

        // Build with npm
        let output = Command::new("npm")
            .args(&["run", "build"])
            .current_dir(dir)
            .output()
            .map_err(|e| ScriptError::Configuration(
                format!("Failed to run npm build: {}", e)
            ))?;

        if !output.status.success() {
            return Err(ScriptError::Configuration(
                format!("npm build failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        println!("✓ Built AssemblyScript plugin: {:?}", dir.join("plugin.wasm"));
        Ok(())
    }

    /// Build a Go plugin with TinyGo
    async fn build_go(&self, dir: &Path) -> Result<()> {
        let output = Command::new("tinygo")
            .args(&["build", "-o", "plugin.wasm", "-target", "wasm", "main.go"])
            .current_dir(dir)
            .output()
            .map_err(|e| ScriptError::Configuration(
                format!("Failed to run tinygo: {}. Make sure TinyGo is installed.", e)
            ))?;

        if !output.status.success() {
            return Err(ScriptError::Configuration(
                format!("TinyGo build failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        println!("✓ Built Go plugin: {:?}", dir.join("plugin.wasm"));
        Ok(())
    }

    /// Build a Zig plugin
    async fn build_zig(&self, dir: &Path) -> Result<()> {
        let output = Command::new("zig")
            .args(&[
                "build-lib",
                "src/main.zig",
                "-target", "wasm32-freestanding",
                "-O", "ReleaseSmall",
                "-femit-bin=plugin.wasm"
            ])
            .current_dir(dir)
            .output()
            .map_err(|e| ScriptError::Configuration(
                format!("Failed to run zig: {}. Make sure Zig is installed.", e)
            ))?;

        if !output.status.success() {
            return Err(ScriptError::Configuration(
                format!("Zig build failed: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }

        println!("✓ Built Zig plugin: {:?}", dir.join("plugin.wasm"));
        Ok(())
    }

    /// Discover all plugin directories in workspaces
    pub fn discover_plugin_directories(&self) -> Result<Vec<PathBuf>> {
        let mut plugins = Vec::new();

        // Look for plugins directories in workspaces
        let pattern = self.workspaces_dir.join("**/plugins/*");

        if let Ok(paths) = glob::glob(&pattern.to_string_lossy()) {
            for entry in paths.flatten() {
                if entry.is_dir() {
                    // Check if it's a valid plugin directory (has plugin.toml)
                    if entry.join("plugin.toml").exists() {
                        plugins.push(entry);
                    }
                }
            }
        }

        Ok(plugins)
    }

    /// Build all out-of-date plugins
    pub async fn build_all_plugins(&self) -> Result<()> {
        let plugins = self.discover_plugin_directories()?;

        if plugins.is_empty() {
            println!("No plugins found to build");
            return Ok(());
        }

        println!("Found {} plugin(s) to check", plugins.len());

        for plugin_dir in plugins {
            if self.needs_rebuild(&plugin_dir) {
                println!("Building plugin: {:?}", plugin_dir);

                // Try to build, but don't fail if one plugin fails
                if let Err(e) = self.build_plugin(&plugin_dir).await {
                    eprintln!("Warning: Failed to build plugin {:?}: {}", plugin_dir, e);
                    // Continue with other plugins
                }
            } else {
                println!("Plugin up-to-date: {:?}", plugin_dir);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        // This test would need actual test fixtures
        // For now, just test the builder creation
        let builder = PluginBuilder::new(PathBuf::from("."));
        assert_eq!(builder.workspaces_dir, PathBuf::from("."));
    }
}