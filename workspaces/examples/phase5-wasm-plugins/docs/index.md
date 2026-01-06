# WASM Plugin System

This workspace demonstrates the WebAssembly (WASM) plugin architecture in structurizr-rs, providing language-agnostic extensibility for advanced use cases.

## Overview

WASM plugins allow you to extend structurizr-rs with custom functionality written in any language that compiles to WebAssembly:

- **Rust** - Native performance, excellent WASM support
- **C/C++** - Via Emscripten or wasi-sdk
- **AssemblyScript** - TypeScript-like syntax for WASM
- **Go** - Via TinyGo compiler
- **Zig** - First-class WASM target

## When to Use Plugins vs Scripts

| Use Case | Recommendation |
|----------|----------------|
| Simple workspace modifications | Lua scripting |
| Dynamic element creation | Lua scripting |
| Complex analysis algorithms | WASM plugins |
| Performance-critical operations | WASM plugins |
| Reusable tools across workspaces | WASM plugins |
| Team-shared extensions | WASM plugins |

## Plugin Architecture

```
Plugin Directory Structure:
my-plugin/
├── plugin.toml      # Manifest (required)
├── plugin.wasm      # Compiled WASM (required)
├── Cargo.toml       # Build configuration (if Rust)
└── src/
    └── lib.rs       # Source code
```

### Manifest Format

The `plugin.toml` file defines plugin metadata and capabilities:

```toml
[plugin]
name = "my-plugin"
version = "1.0.0"
description = "What the plugin does"
wasm = "plugin.wasm"

[capabilities]
read_workspace = true    # Can read workspace data
modify_workspace = false # Can modify workspace
filesystem = false       # Can access filesystem
network = false          # Can make network requests
max_memory = 33554432    # Memory limit (32MB)
max_time = 5000          # Time limit (5 seconds)
```

### Capability-Based Security

Plugins run in a sandboxed environment with explicit permissions:

| Capability | Default | Description |
|------------|---------|-------------|
| `read_workspace` | true | Read workspace structure |
| `modify_workspace` | false | Modify workspace data |
| `filesystem` | false | Access sandboxed files |
| `network` | false | Make network requests |

## Example Plugin

This workspace includes an example plugin in `./plugins/workspace-analyzer/`:

```rust
// Import host functions
extern "C" {
    fn get_workspace_name_len() -> i32;
    fn log(ptr: i32, len: i32);
}

#[no_mangle]
pub extern "C" fn _start() {
    // Plugin entry point
    let name_len = unsafe { get_workspace_name_len() };
    // ... analyze workspace ...
}
```

### Building the Example

```bash
cd plugins/workspace-analyzer

# Add WASM target (one-time)
rustup target add wasm32-unknown-unknown

# Build
cargo build --release --target wasm32-unknown-unknown

# Copy to plugin directory
cp target/wasm32-unknown-unknown/release/workspace_analyzer.wasm plugin.wasm
```

## Host Functions

Plugins communicate with structurizr-rs through host functions:

| Function | Capability | Description |
|----------|------------|-------------|
| `get_workspace_name_len` | read_workspace | Get workspace name length |
| `set_workspace_name` | modify_workspace | Set workspace name |
| `log` | (always) | Log message to console |

More host functions will be added as the API expands.

## Future Development

Planned enhancements:

- [ ] Full workspace read API (people, systems, relationships)
- [ ] Workspace modification API
- [ ] WASI filesystem support
- [ ] Plugin registry for sharing
- [ ] Hot-reload during development

## Related Documentation

- [WASM Plugins Guide](/docs/features/plugins.md)
- [Plugin System Implementation](/docs/development/plugin-system-impl.md)
- [Lua Scripting](/docs/features/scripting.md) (simpler alternative)
