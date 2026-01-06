# ADR 001: Lua as Native Scripting Language

## Status
Accepted

## Context
The original Structurizr DSL supports Groovy and Kotlin scripts via JVM integration. Since structurizr-rs is a native Rust implementation, we cannot directly execute JVM languages.

We need a scripting solution that:
- Is embeddable in Rust
- Has good performance
- Provides adequate security (sandboxing)
- Is easy to learn for most developers

## Decision
We will use **Lua 5.4** as the native scripting language for structurizr-rs.

### Rationale
1. **Rust Integration**: The `mlua` crate provides excellent Lua bindings with:
   - Safe Rust FFI
   - Vendored Lua (no system dependency)
   - Full Lua 5.4 support

2. **Security**: Lua's design enables effective sandboxing:
   - No implicit filesystem/network access
   - Controllable standard library
   - Execution limits via fuel metering

3. **Performance**: Lua is one of the fastest scripting languages:
   - Small memory footprint
   - Fast startup time
   - Efficient bytecode compilation

4. **Simplicity**: Lua has a minimal syntax that's easy to learn:
   - Similar to JavaScript/Python concepts
   - Small standard library
   - Clear semantics

## Alternatives Considered

### JavaScript (QuickJS/V8)
- Pros: Widely known, rich ecosystem
- Cons: Larger runtime, complex sandboxing

### Python (PyO3/RustPython)
- Pros: Very popular, extensive libraries
- Cons: Large runtime, GIL concerns, harder to sandbox

### Rhai (Rust-native)
- Pros: Designed for Rust embedding
- Cons: Less well-known, smaller community

## Consequences

### Positive
- Fast, lightweight scripting runtime
- Effective sandboxing for security
- No external dependencies (vendored)
- Good documentation and community support

### Negative
- Learning curve for developers unfamiliar with Lua
- Need to provide Groovy compatibility layer
- Fewer libraries than Python/JavaScript

## Migration Path
For users with existing Groovy scripts, we provide automatic transpilation (see ADR-002).
