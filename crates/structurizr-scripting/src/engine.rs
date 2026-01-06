//! Script execution engine.
//!
//! This module provides the main entry point for executing scripts
//! against Structurizr workspaces.

use mlua::{Lua, StdLib};
use structurizr_core::Workspace;
use std::path::Path;
use std::fs;

use crate::api::{WorkspaceHandle, register_workspace_api, register_helpers};
use crate::sandbox::SandboxConfig;
use crate::transpiler::GroovyTranspiler;
use crate::error::{Result, ScriptError};

/// Supported script languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptLanguage {
    /// Lua (native support)
    Lua,
    /// Groovy (transpiled to Lua)
    Groovy,
    /// Kotlin (transpiled to Lua, limited support)
    Kotlin,
}

impl ScriptLanguage {
    /// Parse a script language from a string.
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "lua" => Some(Self::Lua),
            "groovy" => Some(Self::Groovy),
            "kotlin" => Some(Self::Kotlin),
            _ => None,
        }
    }
}

/// Configuration for the script engine.
#[derive(Debug, Clone)]
pub struct ScriptConfig {
    /// Sandbox configuration for security
    pub sandbox: SandboxConfig,
    /// Whether to automatically transpile Groovy/Kotlin scripts
    pub auto_transpile: bool,
    /// Base path for resolving relative script paths
    pub base_path: Option<std::path::PathBuf>,
}

impl Default for ScriptConfig {
    fn default() -> Self {
        Self {
            sandbox: SandboxConfig::default(),
            auto_transpile: true,
            base_path: None,
        }
    }
}

impl ScriptConfig {
    /// Create a new script configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the sandbox configuration.
    pub fn with_sandbox(mut self, sandbox: SandboxConfig) -> Self {
        self.sandbox = sandbox;
        self
    }

    /// Set the base path for script resolution.
    pub fn with_base_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.base_path = Some(path.into());
        self
    }

    /// Enable or disable auto-transpilation.
    pub fn with_auto_transpile(mut self, enable: bool) -> Self {
        self.auto_transpile = enable;
        self
    }
}

/// The main script execution engine.
pub struct ScriptEngine {
    config: ScriptConfig,
    transpiler: GroovyTranspiler,
}

impl ScriptEngine {
    /// Create a new script engine with the given configuration.
    pub fn new(config: ScriptConfig) -> Result<Self> {
        Ok(Self {
            config,
            transpiler: GroovyTranspiler::new(),
        })
    }

    /// Create a new script engine with default configuration.
    pub fn with_defaults() -> Result<Self> {
        Self::new(ScriptConfig::default())
    }

    /// Execute an inline script against a workspace.
    pub fn execute(
        &self,
        workspace: &mut Workspace,
        script: &str,
        language: ScriptLanguage,
    ) -> Result<()> {
        // Transpile if needed
        let lua_script = match language {
            ScriptLanguage::Lua => script.to_string(),
            ScriptLanguage::Groovy => {
                if self.config.auto_transpile {
                    self.transpiler.transpile(script)?
                } else {
                    return Err(ScriptError::UnsupportedLanguage(
                        "Groovy scripts require transpilation (enable auto_transpile)".to_string()
                    ));
                }
            }
            ScriptLanguage::Kotlin => {
                if self.config.auto_transpile {
                    // Kotlin uses similar syntax to Groovy for our use case
                    self.transpiler.transpile(script)?
                } else {
                    return Err(ScriptError::UnsupportedLanguage(
                        "Kotlin scripts require transpilation (enable auto_transpile)".to_string()
                    ));
                }
            }
        };

        self.execute_lua(workspace, &lua_script)
    }

    /// Execute a Lua script directly against a workspace.
    pub fn execute_lua(&self, workspace: &mut Workspace, script: &str) -> Result<()> {
        // Clone the workspace for script execution
        let ws_clone = workspace.clone();
        let handle = WorkspaceHandle::new(ws_clone);

        // Create a sandboxed Lua state and execute the script in a block
        // so the Lua state is dropped before we call into_workspace()
        {
            let lua = self.create_sandboxed_lua()?;

            // Register the workspace API
            register_workspace_api(&lua, handle.clone_handle())?;
            register_helpers(&lua)?;

            // Execute the script
            lua.load(script)
                .set_name("script")
                .exec()
                .map_err(|e| ScriptError::Lua(e))?;

            // lua is dropped here, releasing the Arc reference
        }

        // Get the modified workspace back
        *workspace = handle.into_workspace();

        Ok(())
    }

    /// Execute a script file against a workspace.
    pub fn execute_file(
        &self,
        workspace: &mut Workspace,
        path: &Path,
        language: Option<ScriptLanguage>,
    ) -> Result<()> {
        // Resolve the path
        let full_path = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(ref base) = self.config.base_path {
            base.join(path)
        } else {
            path.to_path_buf()
        };

        if !full_path.exists() {
            return Err(ScriptError::FileNotFound(full_path.display().to_string()));
        }

        // Read the script
        let script = fs::read_to_string(&full_path)?;

        // Determine the language from extension if not specified
        let lang = language.unwrap_or_else(|| {
            match full_path.extension().and_then(|e| e.to_str()) {
                Some("lua") => ScriptLanguage::Lua,
                Some("groovy") => ScriptLanguage::Groovy,
                Some("kts") | Some("kt") => ScriptLanguage::Kotlin,
                _ => ScriptLanguage::Lua, // Default to Lua
            }
        });

        self.execute(workspace, &script, lang)
    }

    /// Create a sandboxed Lua state based on configuration.
    fn create_sandboxed_lua(&self) -> Result<Lua> {
        // Start with minimal standard library
        let mut libs = StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::COROUTINE;

        if self.config.sandbox.allow_io {
            libs |= StdLib::IO;
        }
        if self.config.sandbox.allow_os {
            libs |= StdLib::OS;
        }
        if self.config.sandbox.allow_debug {
            libs |= StdLib::DEBUG;
        }
        if self.config.sandbox.allow_package {
            libs |= StdLib::PACKAGE;
        }

        let lua = Lua::new_with(libs, mlua::LuaOptions::default())
            .map_err(|e| ScriptError::Lua(e))?;

        // Set memory limit if configured
        if self.config.sandbox.max_memory > 0 {
            lua.set_memory_limit(self.config.sandbox.max_memory)?;
        }

        // Remove dangerous globals if not allowed
        let globals = lua.globals();

        if !self.config.sandbox.allow_require {
            globals.set("require", mlua::Value::Nil)?;
            globals.set("dofile", mlua::Value::Nil)?;
            globals.set("loadfile", mlua::Value::Nil)?;
        }

        Ok(lua)
    }

    /// Get the transpiler for manual use.
    pub fn transpiler(&self) -> &GroovyTranspiler {
        &self.transpiler
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_simple_lua() {
        let engine = ScriptEngine::with_defaults().unwrap();
        let mut workspace = Workspace::new("Test", "Test workspace");

        engine.execute_lua(&mut workspace, r#"
            workspace:setName("Modified by Lua")
        "#).unwrap();

        assert_eq!(workspace.name, "Modified by Lua");
    }

    #[test]
    fn test_execute_add_elements() {
        let engine = ScriptEngine::with_defaults().unwrap();
        let mut workspace = Workspace::new("Test", "Test workspace");

        engine.execute_lua(&mut workspace, r#"
            workspace:addPerson("User", "A user of the system")
            workspace:addSoftwareSystem("Backend", "The backend system")
        "#).unwrap();

        assert_eq!(workspace.model().people.len(), 1);
        assert_eq!(workspace.model().software_systems.len(), 1);
    }

    #[test]
    fn test_script_language_parsing() {
        assert_eq!(ScriptLanguage::from_str("lua"), Some(ScriptLanguage::Lua));
        assert_eq!(ScriptLanguage::from_str("Groovy"), Some(ScriptLanguage::Groovy));
        assert_eq!(ScriptLanguage::from_str("KOTLIN"), Some(ScriptLanguage::Kotlin));
        assert_eq!(ScriptLanguage::from_str("python"), None);
    }
}
