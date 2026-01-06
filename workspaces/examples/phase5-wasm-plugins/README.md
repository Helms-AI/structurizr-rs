# Phase 5: Multi-Language WASM Plugin System

This example workspace demonstrates WebAssembly (WASM) plugin development in **5 different languages**, showcasing educational concepts and automatic building capabilities.

## 🎯 Overview

A comprehensive demonstration of WASM plugin architecture with:

- **5 Language Examples**: Rust, C, AssemblyScript, Go, and Zig
- **Automatic Building**: Plugins rebuild on server startup if out-of-date
- **Educational Focus**: Each plugin teaches language-specific WASM concepts
- **Unified Build System**: Single Makefile orchestrates all languages
- **Sandboxed Execution**: Capability-based security model
- **Future-Ready Design**: Prepared for API expansion

## 📦 Plugin Collection

| Plugin | Language | Purpose | Binary Size | Status |
|--------|----------|---------|-------------|---------|
| `rust-hello-arch` | Rust | Zero-cost abstractions, no_std | ~3KB | ✅ Implemented |
| `c-memory-explorer` | C | Manual memory management | ~5KB | ✅ Implemented |
| `as-type-safe-greeter` | AssemblyScript | TypeScript-to-WASM | ~15KB | ✅ Implemented |
| `go-stats-calc` | Go/TinyGo | Simplicity, interfaces | ~20KB | ✅ Implemented |
| `zig-pattern-matcher` | Zig | Compile-time (comptime) | ~8KB | ✅ Implemented |

## Contents

```
phase5-wasm-plugins/
├── workspace.dsl                    # Workspace modeling the plugin system
├── Makefile                         # Unified build system for all plugins
├── docs/
│   └── index.md                     # Documentation
├── adrs/
│   └── 001-wasm-plugin-architecture.md  # Architecture decision record
└── plugins/
    ├── workspace-analyzer/          # Original example plugin
    ├── rust-hello-arch/             # Rust: Zero-cost abstractions
    │   ├── plugin.toml
    │   ├── Cargo.toml
    │   ├── README.md
    │   └── src/lib.rs
    ├── c-memory-explorer/           # C: Manual memory management
    │   ├── plugin.toml
    │   ├── Makefile
    │   ├── README.md
    │   └── src/main.c
    ├── as-type-safe-greeter/        # AssemblyScript: TypeScript syntax
    │   ├── plugin.toml
    │   ├── package.json
    │   ├── asconfig.json
    │   └── assembly/index.ts
    ├── go-stats-calc/               # Go: Simplicity + interfaces
    │   ├── plugin.toml
    │   ├── go.mod
    │   ├── README.md
    │   └── main.go
    └── zig-pattern-matcher/         # Zig: Comptime + explicit control
        ├── plugin.toml
        ├── build.zig
        ├── README.md
        └── src/main.zig
```

## 🚀 Quick Start

### Build All Plugins

```bash
# Build all plugins automatically
make all

# Build specific language plugins
make rust          # Build all Rust plugins
make c            # Build all C plugins
make as           # Build all AssemblyScript plugins

# Check which plugins need rebuilding
make check

# Clean all builds
make clean
```

### Automatic Building on Server Startup

When you start the server with the `auto-build-plugins` feature:

```bash
# Server automatically builds out-of-date plugins
cargo run --features auto-build-plugins -- serve --workspaces-dir workspaces
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

## 🛠️ Toolchain Requirements

Install the necessary toolchains for each language:

```bash
# Rust
rustup target add wasm32-unknown-unknown

# C (Emscripten)
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk && ./emsdk install latest && ./emsdk activate latest
source ./emsdk_env.sh

# AssemblyScript
npm install -g assemblyscript

# Go (TinyGo)
go install github.com/tinygo-org/tinygo

# Zig
# Download from https://ziglang.org/download/
```

## Running the Workspace

```bash
# From repository root (with auto-build feature)
cargo run --features auto-build-plugins -- serve --workspaces-dir workspaces

# Or without auto-build
cargo run -- serve --workspaces-dir workspaces

# Open http://localhost:8080/w/examples/phase5-wasm-plugins
```

## 📊 Language Comparison

| Aspect | Rust | C | AssemblyScript | Go | Zig |
|--------|------|---|----------------|----|----|
| **Memory Safety** | ✅ Compile-time | ❌ Manual | ✅ GC | ✅ GC | ✅ Compile-time |
| **Learning Curve** | Steep | Moderate | Easy | Easy | Moderate |
| **Binary Size** | Smallest | Small | Medium | Large | Small |
| **Type System** | Strong | Weak | Strong | Strong | Strong |
| **Best For** | Performance | Control | Web devs | Simplicity | Systems |

## 🎓 Educational Value

Each plugin demonstrates unique concepts:

- **Rust**: Ownership, borrowing, zero-cost abstractions
- **C**: Pointers, manual memory, stack vs heap
- **AssemblyScript**: TypeScript syntax, managed memory
- **Go**: Goroutines (limited), interfaces, simplicity
- **Zig**: Compile-time execution, explicit control

## Related Examples

- `phase5-scripting/` - Lua scripting (simpler alternative)
- `comprehensive/` - All features combined

## Documentation

- [WASM Plugins Guide](/docs/features/plugins.md)
- [Plugin System Implementation](/docs/development/plugin-system-impl.md)
- Individual plugin READMEs for language-specific learning
