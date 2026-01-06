# ADR 001: WASM Plugin Architecture

## Status

Accepted

## Context

structurizr-rs needs an extensibility mechanism that allows users to add custom functionality beyond what Lua scripting provides. The original Structurizr Java implementation uses JVM-based plugins with Groovy/Kotlin scripts.

Key requirements:
- Language-agnostic plugin support
- Secure sandboxed execution
- Good performance for complex operations
- Cross-platform compatibility
- Clear capability/permission model

## Decision

We chose **WebAssembly (WASM)** as the plugin runtime with **wasmtime** as the execution engine.

### Plugin System Design

```
┌─────────────────────────────────────────────┐
│              Plugin Code                     │
│         (Rust, C, AssemblyScript)           │
├─────────────────────────────────────────────┤
│            WASM Module                       │
├─────────────────────────────────────────────┤
│         Host Functions API                   │
├─────────────────────────────────────────────┤
│          wasmtime Runtime                    │
├─────────────────────────────────────────────┤
│         structurizr-rs Host                  │
└─────────────────────────────────────────────┘
```

### Key Components

1. **Plugin Manifest (`plugin.toml`)**
   - Metadata: name, version, description
   - Capabilities: explicit permission requests
   - Resource limits: memory, execution time

2. **WASM Runtime (wasmtime)**
   - Fuel-based execution metering
   - Memory isolation
   - Host function linking

3. **Host Functions**
   - Capability-gated APIs
   - Workspace read/write operations
   - Logging and diagnostics

### Capability Model

```toml
[capabilities]
read_workspace = true    # Read model data
modify_workspace = false # Modify model data
filesystem = false       # WASI filesystem
network = false          # Network access (reserved)
```

## Alternatives Considered

### 1. JVM Embedding (jni-rs)

**Pros:**
- 100% compatibility with Java plugins
- Run existing Groovy/Kotlin scripts

**Cons:**
- Large runtime dependency (~200MB)
- Complex JNI bridge code
- JVM startup overhead
- Platform-specific issues

### 2. JavaScript (V8/QuickJS)

**Pros:**
- Familiar language for many developers
- Rich ecosystem

**Cons:**
- Not as sandboxable as WASM
- Performance overhead for complex operations
- Memory management challenges

### 3. Native Rust Plugins (dylib)

**Pros:**
- Maximum performance
- Direct API access

**Cons:**
- Platform-specific binaries
- Security concerns (full system access)
- ABI stability challenges

### 4. Lua-only (no plugins)

**Pros:**
- Simpler architecture
- Already implemented for scripting

**Cons:**
- Limited performance for complex operations
- No reusable binary distribution
- Less suitable for team tooling

## Consequences

### Positive

- **Language Agnostic**: Plugins can be written in any WASM-compatible language
- **Secure by Default**: WASM sandbox prevents unauthorized system access
- **Portable**: Same plugin binary works on all platforms
- **Performant**: Near-native execution speed
- **Future-Proof**: WASM is a W3C standard with growing ecosystem

### Negative

- **Complexity**: More complex than pure scripting
- **Build Toolchain**: Users need WASM compilation setup
- **Limited API**: Host functions must be explicitly defined
- **Binary Size**: WASM modules can be larger than scripts

### Mitigations

1. **Complexity**: Provide example plugins and templates
2. **Build Toolchain**: Document setup for common languages
3. **Limited API**: Expand host functions incrementally based on needs
4. **Binary Size**: Use `opt-level = "s"` and LTO for size optimization

## Implementation Notes

### wasmtime Configuration

```rust
let mut config = Config::new();
config.consume_fuel(true);  // Enable execution metering

let engine = Engine::new(&config)?;
let mut store = Store::new(&engine, ());
store.set_fuel(max_time_ms * 1000)?;  // ~1000 fuel/ms
```

### Host Function Registration

```rust
linker.func_wrap("env", "log", |ptr: i32, len: i32| {
    // Read string from WASM memory and log
})?;

if capabilities.read_workspace {
    linker.func_wrap("env", "get_workspace_name_len", || {
        workspace.name.len() as i32
    })?;
}
```

## References

- [WebAssembly Specification](https://webassembly.github.io/spec/)
- [wasmtime Documentation](https://docs.wasmtime.dev/)
- [WASI Standard](https://wasi.dev/)
- [Capability-based Security](https://en.wikipedia.org/wiki/Capability-based_security)
