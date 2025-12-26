# structurizr-rs Overview

## What is structurizr-rs?

**structurizr-rs** is a native Rust implementation of [Structurizr Lite](https://structurizr.com/), a tool for creating software architecture diagrams using the C4 model. While the original Structurizr is written in Java, structurizr-rs provides a lightweight, fast, and portable alternative.

## The C4 Model

The C4 model is a lean approach to software architecture documentation created by Simon Brown. It provides a hierarchical set of abstractions:

### Level 1: System Context
Shows the software system in the context of the users and other systems it interacts with. This is the "big picture" view.

### Level 2: Container
Zooms into the software system to show the high-level technical building blocks (web applications, databases, file systems, etc.).

### Level 3: Component
Zooms into each container to show the major structural building blocks and their interactions.

### Level 4: Code
(Optional) Shows how components are implemented using classes, interfaces, and other code-level constructs.

## Architecture

structurizr-rs is organized as a Cargo workspace with five specialized crates:

```
structurizr-rs/
├── Cargo.toml              # Workspace configuration
├── src/main.rs             # CLI entry point
└── crates/
    ├── structurizr-core/   # C4 model types and workspace structure
    ├── structurizr-dsl/    # DSL parser (lexer + parser)
    ├── structurizr-render/ # SVG rendering
    ├── structurizr-export/ # JSON, PlantUML, Mermaid export
    └── structurizr-web/    # Axum-based web server
```

### Crate Dependencies

The crates have a clear dependency hierarchy:

```
structurizr-core (no dependencies on other crates)
    ↑
structurizr-dsl (depends on core)
    ↑
structurizr-render (depends on core)
    ↑
structurizr-export (depends on core)
    ↑
structurizr-web (depends on core, dsl, render, export)
    ↑
structurizr-rs (main binary, depends on all)
```

## Key Features

### DSL Support
Full support for the Structurizr DSL syntax, allowing you to define architecture models in a human-readable text format.

### SVG Rendering
Native SVG diagram generation with automatic layout algorithms and customizable styling.

### Multiple Export Formats
Export your architecture to JSON, PlantUML, or Mermaid formats for integration with other tools.

### Web Server
Built-in web server for viewing and navigating your architecture documentation.

### Dark Mode
Support for dark-themed diagrams with customizable background colors.

### Icons
Embed icons in your architecture elements using URLs or data URIs.

## Comparison with Structurizr Java

| Feature | structurizr-rs | Structurizr Java |
|---------|----------------|------------------|
| Language | Rust | Java |
| Startup Time | ~50ms | ~2-3s |
| Memory Usage | ~20MB | ~100-200MB |
| Binary Size | ~10MB | ~50MB + JVM |
| Dependencies | Minimal | JVM required |
| DSL Support | Full | Full |
| Web UI | Simple | Full-featured |

## When to Use structurizr-rs

- **Fast Iteration**: Quick startup makes it ideal for development workflows
- **Resource-Constrained Environments**: Lower memory footprint for CI/CD pipelines
- **Static Site Generation**: Generate diagrams for documentation sites
- **Container Deployments**: Small binary size for Docker images

## Getting Started

See the [Getting Started](002_Getting_Started.md) guide for installation and first steps.

## Further Reading

- [Core Crate](003_Core_Crate.md) - Understanding the C4 model types
- [DSL Crate](004_DSL_Crate.md) - Parser architecture and DSL syntax
- [Render Crate](005_Render_Crate.md) - SVG generation and layout
- [Export Crate](006_Export_Crate.md) - Export formats
- [Web Crate](007_Web_Crate.md) - Web server and UI
