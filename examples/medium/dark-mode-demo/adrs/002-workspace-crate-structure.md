# ADR 002: Workspace Crate Structure

## Status

Accepted

## Context

The project needs to be organized in a maintainable way that:

1. Separates concerns clearly
2. Enables independent testing
3. Allows selective dependency inclusion
4. Supports future extensibility

We considered several organizational approaches:

1. **Single crate** - All code in one package
2. **Workspace with logical crates** - Separate crates by domain
3. **Feature flags** - Single crate with optional features
4. **Plugin architecture** - Core with loadable extensions

## Decision

We chose a **Cargo workspace with five specialized crates**:

```
structurizr-rs/
├── Cargo.toml              # Workspace configuration
├── src/main.rs             # CLI entry point
└── crates/
    ├── structurizr-core/   # C4 model types
    ├── structurizr-dsl/    # DSL parser
    ├── structurizr-render/ # SVG rendering
    ├── structurizr-export/ # Format exporters
    └── structurizr-web/    # Web server
```

### Dependency Hierarchy

```
structurizr-core (no dependencies on other crates)
    ↑
structurizr-dsl (depends on core)
    ↑
structurizr-render (depends on core)
    ↑
structurizr-export (depends on core)
    ↑
structurizr-web (depends on all above)
```

### Crate Responsibilities

| Crate | Responsibility |
|-------|----------------|
| core | Data structures, serialization |
| dsl | Lexing, parsing, AST |
| render | SVG generation, layout |
| export | JSON, PlantUML, Mermaid |
| web | HTTP server, file watching |

## Consequences

### Positive

- **Clear boundaries**: Each crate has a single responsibility
- **Independent compilation**: Faster incremental builds
- **Selective inclusion**: Users can depend on specific crates
- **Easier testing**: Isolated unit tests per crate
- **Reusability**: Core types usable without web server

### Negative

- **More configuration**: Multiple Cargo.toml files
- **Version coordination**: Must keep crate versions in sync
- **API surface**: Public API design requires more thought
- **Circular dependencies**: Must be careful to avoid

### Neutral

- Learning curve for new contributors
- Standard Rust workspace practices apply

## Alternatives Considered

### Single Crate with Modules

**Pros**: Simpler setup, single version
**Cons**: All-or-nothing dependency, slower compilation

### Feature Flags

**Pros**: Single crate, optional features
**Cons**: Complex feature combinations, harder testing

### Plugin Architecture

**Pros**: Maximum extensibility
**Cons**: Runtime overhead, complex API

## Implementation Notes

Each crate follows Rust conventions:

```rust
// crates/structurizr-core/src/lib.rs
pub mod model;
pub mod view;
pub mod style;
pub mod workspace;
pub mod error;

pub use model::*;
pub use workspace::Workspace;
```

Inter-crate dependencies are explicit:

```toml
# crates/structurizr-dsl/Cargo.toml
[dependencies]
structurizr-core = { path = "../structurizr-core" }
```
