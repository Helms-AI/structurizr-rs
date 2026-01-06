# Phase 5: WASM Plugin System

This example workspace demonstrates the WebAssembly (WASM) plugin architecture in structurizr-rs.

## Overview

WASM plugins provide language-agnostic extensibility for advanced use cases:

- Write plugins in **Rust**, **C**, **AssemblyScript**, or any WASM-compatible language
- **Sandboxed execution** with capability-based security
- **Near-native performance** for complex operations
- **Cross-platform** binary distribution

## Contents

```
phase5-wasm-plugins/
├── workspace.dsl                    # Workspace modeling the plugin system
├── docs/
│   └── index.md                     # Documentation
├── adrs/
│   └── 001-wasm-plugin-architecture.md  # Architecture decision record
└── plugins/
    └── workspace-analyzer/          # Example plugin
        ├── plugin.toml              # Plugin manifest
        ├── Cargo.toml               # Rust build configuration
        └── src/
            └── lib.rs               # Plugin source code
```

## Building the Example Plugin

```bash
cd plugins/workspace-analyzer

# Add WASM target (one-time setup)
rustup target add wasm32-unknown-unknown

# Build the plugin
cargo build --release --target wasm32-unknown-unknown

# Copy to plugin directory
cp target/wasm32-unknown-unknown/release/workspace_analyzer.wasm plugin.wasm
```

## Plugin Manifest

Each plugin requires a `plugin.toml` manifest:

```toml
[plugin]
name = "workspace-analyzer"
version = "1.0.0"
description = "Analyzes workspace structure"
wasm = "plugin.wasm"

[capabilities]
read_workspace = true
modify_workspace = false
```

## Running the Workspace

```bash
# From repository root
cargo run -- serve --workspaces-dir workspaces

# Open http://localhost:8080/w/examples/phase5-wasm-plugins
```

## Related Examples

- `phase5-scripting/` - Lua scripting (simpler alternative)
- `comprehensive/` - All features combined

## Documentation

- [WASM Plugins Guide](/docs/features/plugins.md)
- [Plugin System Implementation](/docs/development/plugin-system-impl.md)
