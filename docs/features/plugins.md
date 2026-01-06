# WASM Plugins

structurizr-rs supports WebAssembly (WASM) plugins for advanced extensibility. This allows you to write plugins in any language that compiles to WASM (Rust, C, AssemblyScript, etc.) and load them at runtime.

> **Note:** WASM plugin support requires the `wasm` feature flag and is intended for advanced users who need capabilities beyond what Lua scripting provides.

## Feature Flag

Enable WASM support when building:

```bash
cargo build --features wasm
```

## Plugin Structure

A WASM plugin consists of:

1. **Plugin manifest** (`plugin.toml`) - Metadata and configuration
2. **WASM binary** (`plugin.wasm`) - Compiled WebAssembly module

### Directory Layout

```
my-plugin/
├── plugin.toml      # Plugin manifest
├── plugin.wasm      # Compiled WASM binary
└── src/             # Source code (optional)
    └── lib.rs
```

## Plugin Manifest

The `plugin.toml` file defines plugin metadata and capabilities:

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "A custom plugin for workspace analysis"
wasm = "plugin.wasm"
author = "Your Name"
homepage = "https://github.com/you/my-plugin"

[capabilities]
read_workspace = true
modify_workspace = false
filesystem = false
network = false
max_memory = 0       # 0 = use default (64MB)
max_time = 0         # 0 = use default (30 seconds)
```

### Plugin Metadata

| Field | Required | Description |
|-------|----------|-------------|
| `name` | Yes | Plugin identifier |
| `version` | Yes | Semantic version (e.g., "1.0.0") |
| `description` | No | Brief description |
| `wasm` | Yes | Path to WASM binary (relative to manifest) |
| `author` | No | Plugin author |
| `homepage` | No | URL to documentation or repository |

### Capabilities

Capabilities define what the plugin is allowed to do:

| Capability | Default | Description |
|------------|---------|-------------|
| `read_workspace` | `true` | Can read workspace data |
| `modify_workspace` | `false` | Can modify workspace data |
| `filesystem` | `false` | Can access sandboxed filesystem |
| `network` | `false` | Can make network requests (reserved) |
| `max_memory` | `0` | Memory limit in bytes (0 = 64MB default) |
| `max_time` | `0` | Execution time in ms (0 = 30s default) |

## Host Functions

WASM plugins communicate with structurizr-rs through host functions:

### Available Host Functions

| Function | Capability Required | Description |
|----------|-------------------|-------------|
| `get_workspace_name_len` | `read_workspace` | Get length of workspace name |
| `set_workspace_name` | `modify_workspace` | Set workspace name |
| `log` | (always) | Log a message to console |

### Function Signatures

```rust
// Get the length of the workspace name
extern "C" fn get_workspace_name_len() -> i32;

// Set the workspace name (ptr: pointer to string, len: string length)
extern "C" fn set_workspace_name(ptr: i32, len: i32);

// Log a message (ptr: pointer to string, len: string length)
extern "C" fn log(ptr: i32, len: i32);
```

## Entry Points

Plugins must export one of these entry point functions:

| Function | Signature | Description |
|----------|-----------|-------------|
| `_start` | `fn()` | WASI-style entry point |
| `run` | `fn()` | Alternative entry point |

## Writing a Plugin in Rust

### 1. Create the Project

```bash
cargo new --lib my-plugin
cd my-plugin
```

### 2. Configure Cargo.toml

```toml
[package]
name = "my-plugin"
version = "1.0.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[profile.release]
opt-level = "s"
lto = true
```

### 3. Write the Plugin

```rust
// src/lib.rs

// Import host functions
extern "C" {
    fn get_workspace_name_len() -> i32;
    fn log(ptr: i32, len: i32);
}

// Helper to log a message
fn log_message(msg: &str) {
    unsafe {
        log(msg.as_ptr() as i32, msg.len() as i32);
    }
}

// Plugin entry point
#[no_mangle]
pub extern "C" fn _start() {
    log_message("Plugin starting...");

    // Get workspace name length
    let name_len = unsafe { get_workspace_name_len() };
    log_message(&format!("Workspace name length: {}", name_len));

    log_message("Plugin complete!");
}
```

### 4. Build for WASM

```bash
# Add WASM target
rustup target add wasm32-unknown-unknown

# Build
cargo build --release --target wasm32-unknown-unknown

# Copy to plugin directory
cp target/wasm32-unknown-unknown/release/my_plugin.wasm plugin.wasm
```

### 5. Create the Manifest

```toml
# plugin.toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "Example Rust plugin"
wasm = "plugin.wasm"

[capabilities]
read_workspace = true
```

## Loading Plugins

### From DSL (Future)

```dsl
workspace "My System" {
    !plugin "./plugins/my-plugin"

    model {
        // ...
    }
}
```

### Programmatic Loading

```rust
use structurizr_scripting::{PluginEngine, PluginEngineConfig};
use structurizr_core::Workspace;
use std::path::Path;

// Create plugin engine
let engine = PluginEngine::with_config(
    PluginEngineConfig::new()
        .with_max_memory(128 * 1024 * 1024)  // 128MB
        .with_max_time(60_000)                // 60 seconds
)?;

// Execute plugin
let mut workspace = Workspace::new("Test", "Test workspace");
engine.execute_plugin(
    Path::new("./plugins/my-plugin/plugin.toml"),
    &mut workspace
)?;
```

## Plugin Discovery

Find all plugins in a directory:

```rust
use structurizr_scripting::PluginEngine;
use std::path::Path;

let plugins = PluginEngine::discover_plugins(Path::new("./plugins"))?;

for manifest in plugins {
    println!("Found plugin: {} v{}",
        manifest.plugin.name,
        manifest.plugin.version
    );
    if manifest.capabilities.modify_workspace {
        println!("  - Can modify workspace");
    }
}
```

## Security

WASM plugins run in a sandboxed environment:

### Isolation

- **Memory isolation**: Each plugin has its own linear memory
- **No direct system access**: All I/O goes through host functions
- **Capability-based permissions**: Explicit opt-in for each capability

### Resource Limits

- **Memory limit**: Default 64MB, configurable per plugin
- **Execution time**: Fuel-based metering, default 30 seconds
- **No network access**: Currently not supported

### Best Practices

1. **Minimal capabilities**: Only request capabilities you need
2. **Validate inputs**: Don't trust data from the workspace
3. **Handle errors**: Return gracefully instead of panicking
4. **Test thoroughly**: Verify behavior with various workspace states

## Comparison: Scripts vs Plugins

| Feature | Lua Scripts | WASM Plugins |
|---------|-------------|--------------|
| Language | Lua | Any (Rust, C, etc.) |
| Performance | Interpreted | Near-native |
| Setup | None | Compilation required |
| Distribution | Inline in DSL | Separate files |
| Debugging | Easy (print) | More complex |
| Capabilities | Full workspace API | Limited host functions |
| Use case | Simple modifications | Complex analysis/transforms |

### When to Use Scripts

- Quick workspace modifications
- Dynamic element creation
- Conditional logic based on properties
- Backwards compatibility with Groovy

### When to Use Plugins

- Performance-critical operations
- Complex analysis algorithms
- Reusable components across workspaces
- Team-shared tooling

## Troubleshooting

### "Plugin must export '_start' or 'run' function"

Ensure your plugin exports an entry point:

```rust
#[no_mangle]
pub extern "C" fn _start() {
    // Plugin code
}
```

### "Failed to compile WASM"

Check that your WASM binary is valid:

```bash
# Validate with wasmtime
wasmtime plugin.wasm
```

### Memory Errors

Increase the memory limit:

```toml
[capabilities]
max_memory = 134217728  # 128MB
```

### Timeout Errors

Increase the time limit:

```toml
[capabilities]
max_time = 60000  # 60 seconds
```

## Future Enhancements

The plugin system is being actively developed. Planned features:

- [ ] Additional host functions for full workspace access
- [ ] WASI filesystem support for sandboxed file access
- [ ] Plugin registry for sharing plugins
- [ ] Hot-reload for development

## See Also

- [Scripting Guide](scripting.md) - Lua scripting (simpler alternative)
- [Plugin System Implementation](../development/plugin-system-impl.md) - Technical details
