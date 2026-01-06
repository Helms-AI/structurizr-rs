# Plugin System Implementation

This document describes the WASM plugin system architecture in structurizr-rs.

## Overview

The plugin system uses WebAssembly (WASM) to provide language-agnostic extensibility. Plugins are sandboxed and communicate with the host through a defined set of host functions.

## Architecture

```
crates/structurizr-scripting/src/plugin/
├── mod.rs           # PluginManifest, PluginCapabilities, PluginInfo
└── wasm_runtime.rs  # PluginEngine, PluginEngineConfig (wasm feature)
```

## Plugin Manifest

### Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    pub wasm: String,  // Path to WASM binary
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
}
```

### Capabilities

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCapabilities {
    #[serde(default = "default_true")]
    pub read_workspace: bool,     // Default: true
    #[serde(default)]
    pub modify_workspace: bool,   // Default: false
    #[serde(default)]
    pub filesystem: bool,         // Default: false
    #[serde(default)]
    pub network: bool,            // Default: false
    #[serde(default)]
    pub max_memory: usize,        // 0 = use engine default
    #[serde(default)]
    pub max_time: u64,            // 0 = use engine default
}
```

### Example Manifest

```toml
[plugin]
name = "workspace-analyzer"
version = "1.0.0"
description = "Analyzes workspace structure"
wasm = "plugin.wasm"
author = "Your Name"
homepage = "https://github.com/you/plugin"

[capabilities]
read_workspace = true
modify_workspace = false
max_memory = 67108864  # 64MB
max_time = 10000       # 10 seconds
```

## PluginEngine

### Configuration

```rust
#[derive(Debug, Clone)]
pub struct PluginEngineConfig {
    pub max_memory: usize,   // Default: 64MB
    pub max_time: u64,       // Default: 30 seconds
    pub base_path: Option<PathBuf>,
}

impl Default for PluginEngineConfig {
    fn default() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024,
            max_time: 30_000,
            base_path: None,
        }
    }
}
```

### Engine Creation

```rust
pub struct PluginEngine {
    engine: Engine,
    config: PluginEngineConfig,
}

impl PluginEngine {
    pub fn new() -> Result<Self> {
        Self::with_config(PluginEngineConfig::default())
    }

    pub fn with_config(config: PluginEngineConfig) -> Result<Self> {
        let mut engine_config = Config::new();
        engine_config.consume_fuel(true);  // Enable fuel-based metering

        let engine = Engine::new(&engine_config)?;
        Ok(Self { engine, config })
    }
}
```

## Wasmtime Integration

### Module Loading

```rust
pub fn execute_wasm_bytes(
    &self,
    wasm_bytes: &[u8],
    workspace: &mut Workspace,
    capabilities: &PluginCapabilities,
) -> Result<()> {
    // Compile WASM module
    let module = Module::new(&self.engine, wasm_bytes)?;

    // Create linker with host functions
    let mut linker = Linker::new(&self.engine);

    // Shared workspace state
    let workspace_state = Arc::new(Mutex::new(workspace.clone()));

    // Register host functions
    self.register_host_functions(&mut linker, workspace_state.clone(), capabilities)?;

    // Create store with fuel limits
    let mut store = Store::new(&self.engine, ());

    // Set execution fuel
    let max_fuel = if capabilities.max_time > 0 {
        capabilities.max_time * 1000
    } else {
        self.config.max_time * 1000
    };
    store.set_fuel(max_fuel).ok();

    // Instantiate and run
    let instance = linker.instantiate(&mut store, &module)?;

    // Call entry point
    if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "_start") {
        func.call(&mut store, ())?;
    } else if let Ok(func) = instance.get_typed_func::<(), ()>(&mut store, "run") {
        func.call(&mut store, ())?;
    }

    // Copy modified workspace back
    if capabilities.modify_workspace {
        *workspace = workspace_state.lock()?.clone();
    }

    Ok(())
}
```

### Fuel-Based Metering

Wasmtime uses "fuel" to limit execution:

```rust
// Roughly 1000 fuel units per millisecond
let max_fuel = max_time_ms * 1000;
store.set_fuel(max_fuel).ok();
```

When fuel is exhausted, the plugin terminates with an error.

## Host Functions

### Registration

```rust
fn register_host_functions(
    &self,
    linker: &mut Linker<()>,
    workspace: Arc<Mutex<Workspace>>,
    capabilities: &PluginCapabilities,
) -> Result<()> {
    // Read functions (require read_workspace capability)
    if capabilities.read_workspace {
        let ws = workspace.clone();
        linker.func_wrap("env", "get_workspace_name_len", move || -> i32 {
            let ws = ws.lock().unwrap();
            ws.name.len() as i32
        })?;
    }

    // Write functions (require modify_workspace capability)
    if capabilities.modify_workspace {
        let ws = workspace.clone();
        linker.func_wrap("env", "set_workspace_name",
            move |_caller: Caller<'_, ()>, ptr: i32, len: i32| {
                let mut ws = ws.lock().unwrap();
                ws.name = "Modified by WASM".to_string();
            })?;
    }

    // Always available
    linker.func_wrap("env", "log",
        |_caller: Caller<'_, ()>, ptr: i32, len: i32| {
            println!("[WASM Plugin] log called");
        })?;

    Ok(())
}
```

### Available Host Functions

| Function | Capability | Signature | Description |
|----------|------------|-----------|-------------|
| `get_workspace_name_len` | `read_workspace` | `fn() -> i32` | Get workspace name length |
| `set_workspace_name` | `modify_workspace` | `fn(ptr: i32, len: i32)` | Set workspace name |
| `log` | (always) | `fn(ptr: i32, len: i32)` | Log message to console |

## Plugin Discovery

```rust
pub fn discover_plugins(dir: &Path) -> Result<Vec<PluginManifest>> {
    let mut plugins = Vec::new();

    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check for plugin.toml in subdirectory or directly
        let manifest_path = if path.is_dir() {
            path.join("plugin.toml")
        } else if path.file_name() == Some(OsStr::new("plugin.toml")) {
            path.clone()
        } else {
            continue;
        };

        if manifest_path.exists() {
            match PluginManifest::load(&manifest_path) {
                Ok(manifest) => plugins.push(manifest),
                Err(e) => eprintln!("Warning: Failed to load {:?}: {}", manifest_path, e),
            }
        }
    }

    Ok(plugins)
}
```

## Memory Management

### WASM Linear Memory

Each plugin has its own linear memory:
- Isolated from host memory
- Cannot access other plugins' memory
- Size limited by `max_memory` capability

### String Passing

Strings are passed between host and plugin via linear memory:

```rust
// Plugin allocates memory and passes pointer + length
// Host reads string from plugin memory
fn read_string_from_wasm(memory: &Memory, store: &Store<()>, ptr: i32, len: i32) -> String {
    let mut buffer = vec![0u8; len as usize];
    memory.read(&store, ptr as usize, &mut buffer).unwrap();
    String::from_utf8_lossy(&buffer).to_string()
}
```

## Security Model

### Capability-Based Security

```
┌─────────────────────────────────────────────┐
│                 Plugin Code                  │
├─────────────────────────────────────────────┤
│         Capability Check Layer               │
├─────────────────────────────────────────────┤
│           Host Functions                     │
├─────────────────────────────────────────────┤
│            Workspace State                   │
└─────────────────────────────────────────────┘
```

Each host function call is gated by capability checks:

1. Plugin requests capability in `plugin.toml`
2. Engine validates requested capabilities
3. Only permitted host functions are registered
4. Runtime checks prevent capability bypass

### Sandboxing Guarantees

- **Memory isolation**: WASM linear memory is separate from host
- **No direct system calls**: All I/O through host functions
- **Fuel-based limits**: Prevents infinite loops
- **Capability enforcement**: Explicit permission model

## Error Handling

```rust
pub enum ScriptError {
    // Plugin-specific errors
    FileNotFound(String),      // Missing WASM or manifest
    Configuration(String),     // Invalid manifest or config
    Lua(mlua::Error),          // Wrapped wasmtime errors
}
```

Common errors:

| Error | Cause | Solution |
|-------|-------|----------|
| "Plugin must export '_start' or 'run' function" | Missing entry point | Export `_start` or `run` |
| "Failed to compile WASM" | Invalid WASM binary | Check compilation |
| Fuel exhausted | Execution too long | Increase `max_time` |
| Memory error | Memory limit exceeded | Increase `max_memory` |

## Testing

```rust
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

#[test]
fn test_plugin_manifest_parsing() {
    let toml_str = r#"
[plugin]
name = "test-plugin"
version = "1.0.0"
wasm = "plugin.wasm"

[capabilities]
read_workspace = true
modify_workspace = false
"#;
    let manifest: PluginManifest = toml::from_str(toml_str).unwrap();
    assert_eq!(manifest.plugin.name, "test-plugin");
    assert!(manifest.capabilities.read_workspace);
    assert!(!manifest.capabilities.modify_workspace);
}
```

## Future Enhancements

### Additional Host Functions

```rust
// Planned host functions
fn get_people_count() -> i32;
fn get_systems_count() -> i32;
fn get_person_name(index: i32, ptr: i32, max_len: i32) -> i32;
fn add_person(name_ptr: i32, name_len: i32, desc_ptr: i32, desc_len: i32);
```

### WASI Integration

For sandboxed filesystem access:

```rust
// Future: WASI filesystem support
let wasi = wasmtime_wasi::WasiCtxBuilder::new()
    .preopened_dir(dir, "/workspace")?
    .build()?;
```

### Network Capabilities

Reserved for future implementation:

```rust
pub network: bool,  // Currently always false
```

## See Also

- [WASM Plugins Guide](../features/plugins.md) - User documentation
- [Scripting Implementation](scripting-impl.md) - Lua scripting system
- [Sandbox Implementation](sandbox-impl.md) - Security architecture
