//! WASM Runtime for Plugin Execution.
//!
//! This module provides the wasmtime-based runtime for executing WASM plugins.

use std::path::Path;
use std::sync::{Arc, Mutex};

use structurizr_core::Workspace;
use wasmtime::*;

use crate::error::{Result, ScriptError};
use super::{PluginManifest, PluginCapabilities};

/// WASM Plugin Engine for loading and executing plugins.
pub struct PluginEngine {
    engine: Engine,
    config: PluginEngineConfig,
}

/// Configuration for the plugin engine.
#[derive(Debug, Clone)]
pub struct PluginEngineConfig {
    /// Maximum memory per plugin (bytes)
    pub max_memory: usize,
    /// Maximum execution time (milliseconds)
    pub max_time: u64,
    /// Base path for resolving plugins
    pub base_path: Option<std::path::PathBuf>,
}

impl Default for PluginEngineConfig {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64 MB default
            max_time: 30_000, // 30 seconds default
            base_path: None,
        }
    }
}

impl PluginEngineConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = bytes;
        self
    }

    pub fn with_max_time(mut self, ms: u64) -> Self {
        self.max_time = ms;
        self
    }

    pub fn with_base_path(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.base_path = Some(path.into());
        self
    }
}

impl PluginEngine {
    /// Create a new plugin engine with default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(PluginEngineConfig::default())
    }

    /// Create a new plugin engine with custom configuration.
    pub fn with_config(config: PluginEngineConfig) -> Result<Self> {
        let mut engine_config = Config::new();

        // Enable fuel-based metering for execution time limits
        engine_config.consume_fuel(true);

        // Create the engine
        let engine = Engine::new(&engine_config)
            .map_err(|e| ScriptError::Configuration(format!("Failed to create WASM engine: {}", e)))?;

        Ok(Self { engine, config })
    }

    /// Load and execute a plugin from a manifest file.
    pub fn execute_plugin(
        &self,
        manifest_path: &Path,
        workspace: &mut Workspace,
    ) -> Result<()> {
        // Load the manifest
        let manifest = PluginManifest::load(manifest_path)?;

        // Get the WASM binary path
        let base_dir = manifest_path.parent()
            .ok_or_else(|| ScriptError::FileNotFound("Invalid manifest path".to_string()))?;
        let wasm_path = manifest.wasm_path(base_dir);

        self.execute_wasm(&wasm_path, workspace, &manifest.capabilities)
    }

    /// Execute a WASM binary directly.
    pub fn execute_wasm(
        &self,
        wasm_path: &Path,
        workspace: &mut Workspace,
        capabilities: &PluginCapabilities,
    ) -> Result<()> {
        // Read the WASM binary
        let wasm_bytes = std::fs::read(wasm_path)
            .map_err(|e| ScriptError::FileNotFound(format!("{}: {}", wasm_path.display(), e)))?;

        self.execute_wasm_bytes(&wasm_bytes, workspace, capabilities)
    }

    /// Execute WASM bytes.
    pub fn execute_wasm_bytes(
        &self,
        wasm_bytes: &[u8],
        workspace: &mut Workspace,
        capabilities: &PluginCapabilities,
    ) -> Result<()> {
        // Create a module from the WASM bytes
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| ScriptError::Lua(mlua::Error::external(format!("Failed to compile WASM: {}", e))))?;

        // Create a linker with host functions
        let mut linker = Linker::new(&self.engine);

        // Create workspace state that can be accessed from host functions
        let workspace_state = Arc::new(Mutex::new(workspace.clone()));

        // Register host functions based on capabilities
        self.register_host_functions(&mut linker, workspace_state.clone(), capabilities)?;

        // Create a store with fuel limits
        let mut store = Store::new(&self.engine, ());

        // Set fuel based on max_time (roughly 1000 fuel per ms)
        let max_fuel = if capabilities.max_time > 0 {
            capabilities.max_time * 1000
        } else {
            self.config.max_time * 1000
        };
        store.set_fuel(max_fuel).ok();

        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| ScriptError::Lua(mlua::Error::external(format!("Failed to instantiate WASM: {}", e))))?;

        // Call the plugin's entry point (convention: _start or run)
        if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
            func.call(&mut store, ())
                .map_err(|e| ScriptError::Lua(mlua::Error::external(format!("Plugin execution failed: {}", e))))?;
        } else if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "run") {
            func.call(&mut store, ())
                .map_err(|e| ScriptError::Lua(mlua::Error::external(format!("Plugin execution failed: {}", e))))?;
        } else {
            return Err(ScriptError::Configuration(
                "Plugin must export '_start' or 'run' function".to_string()
            ));
        }

        // Copy modified workspace back
        if capabilities.modify_workspace {
            let modified = workspace_state.lock()
                .map_err(|e| ScriptError::Lua(mlua::Error::external(format!("Mutex poisoned: {}", e))))?;
            *workspace = modified.clone();
        }

        Ok(())
    }

    /// Register host functions that plugins can call.
    fn register_host_functions(
        &self,
        linker: &mut Linker<()>,
        workspace: Arc<Mutex<Workspace>>,
        capabilities: &PluginCapabilities,
    ) -> Result<()> {
        // Only register read functions if read_workspace capability is granted
        if capabilities.read_workspace {
            let ws = workspace.clone();
            linker.func_wrap("env", "get_workspace_name_len", move || -> i32 {
                let ws = ws.lock().unwrap();
                ws.name.len() as i32
            }).map_err(|e| ScriptError::Configuration(format!("Failed to register function: {}", e)))?;
        }

        // Only register write functions if modify_workspace capability is granted
        if capabilities.modify_workspace {
            let ws = workspace.clone();
            linker.func_wrap("env", "set_workspace_name", move |_caller: Caller<'_, ()>, _ptr: i32, _len: i32| {
                // In a real implementation, we'd read the string from WASM memory
                // For now, this is a placeholder
                let mut ws = ws.lock().unwrap();
                ws.name = "Modified by WASM".to_string();
            }).map_err(|e| ScriptError::Configuration(format!("Failed to register function: {}", e)))?;
        }

        // Log function (always available) - reads string from WASM memory
        linker.func_wrap("env", "log", |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            // Get the exported memory
            if let Some(memory) = caller.get_export("memory").and_then(|e| e.into_memory()) {
                let data = memory.data(&caller);
                let start = ptr as usize;
                let end = start + len as usize;

                if end <= data.len() {
                    if let Ok(msg) = std::str::from_utf8(&data[start..end]) {
                        println!("[Plugin] {}", msg);
                    } else {
                        println!("[Plugin] <invalid UTF-8>");
                    }
                } else {
                    println!("[Plugin] <out of bounds>");
                }
            } else {
                println!("[Plugin] <no memory export>");
            }
        }).map_err(|e| ScriptError::Configuration(format!("Failed to register function: {}", e)))?;

        Ok(())
    }

    /// List available plugins in a directory.
    pub fn discover_plugins(dir: &Path) -> Result<Vec<PluginManifest>> {
        let mut plugins = Vec::new();

        if !dir.is_dir() {
            return Ok(plugins);
        }

        for entry in std::fs::read_dir(dir)
            .map_err(|e| ScriptError::FileNotFound(format!("{}: {}", dir.display(), e)))?
        {
            let entry = entry
                .map_err(|e| ScriptError::FileNotFound(format!("Failed to read directory: {}", e)))?;
            let path = entry.path();

            // Check for plugin.toml in subdirectory
            let manifest_path = if path.is_dir() {
                path.join("plugin.toml")
            } else if path.file_name() == Some(std::ffi::OsStr::new("plugin.toml")) {
                path.clone()
            } else {
                continue;
            };

            if manifest_path.exists() {
                match PluginManifest::load(&manifest_path) {
                    Ok(manifest) => plugins.push(manifest),
                    Err(e) => {
                        eprintln!("Warning: Failed to load plugin at {:?}: {}", manifest_path, e);
                    }
                }
            }
        }

        Ok(plugins)
    }
}

impl Default for PluginEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create default plugin engine")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_engine_creation() {
        let engine = PluginEngine::new();
        assert!(engine.is_ok());
    }

    #[test]
    fn test_plugin_engine_config() {
        let config = PluginEngineConfig::new()
            .with_max_memory(128 * 1024 * 1024)
            .with_max_time(60_000);

        assert_eq!(config.max_memory, 128 * 1024 * 1024);
        assert_eq!(config.max_time, 60_000);
    }
}
