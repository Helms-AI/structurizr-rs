# structurizr-rs Documentation

Welcome to the documentation for **structurizr-rs**, a Rust implementation of Structurizr Lite for creating software architecture diagrams using the C4 model.

## Feature Guides

User-facing documentation for structurizr-rs features:

| Feature | Description |
|---------|-------------|
| [Animation](features/animation.md) | Animate dynamic views step-by-step |
| [Directives](features/directives.md) | DSL directives (`!const`, `!include`, `!impliedRelationships`, etc.) |
| [Legend](features/legend.md) | Auto-generated diagram keys/legends |
| [Perspectives](features/perspectives.md) | Filter diagrams by stakeholder perspective |
| [Perspectives Quickstart](features/perspectives-quickstart.md) | Quick start guide for perspectives |
| [Presentation Mode](features/presentation.md) | Full-screen slideshow of diagrams |
| [Validation](features/validation.md) | Workspace validation and inspections |

### Scripting & Extensibility

| Feature | Description |
|---------|-------------|
| [Scripting](features/scripting.md) | Lua scripting with `!script` directive |
| [Scripting API Reference](features/scripting-api-reference.md) | Complete workspace API for scripts |
| [Groovy Migration](features/groovy-migration.md) | Migrate existing Groovy scripts to Lua |
| [WASM Plugins](features/plugins.md) | Build WASM plugins for advanced extensibility |
| [MCP Integration](features/mcp-integration.md) | Model Context Protocol server for AI assistant integration |

---

## Development Documentation

Implementation details and architecture documentation:

### Core Rendering System

| Document | Description |
|----------|-------------|
| [SVG Rendering Pipeline](development/svg-rendering-pipeline.md) | Complete SVG rendering system architecture and pipeline |
| [Layout Algorithms](development/layout-algorithms.md) | Grid, Adaptive, and Sugiyama hierarchical layout implementations |
| [Style System](development/style-system.md) | Cascading style resolution and theming |
| [Shape Rendering](development/shape-rendering.md) | All 14 shape types and their SVG implementations |
| [Edge Routing](development/edge-routing.md) | Relationship routing strategies (Direct, Orthogonal, Curved) |
| [Text Handling](development/text-handling.md) | Text rendering, word wrapping, and collision detection |
| [Coordinate Systems](development/coordinate-systems.md) | Positioning, transformations, and viewBox management |
| [SVG Integration Patterns](development/svg-integration-patterns.md) | CLI, Web, and export integration patterns |

### Feature Implementations

| Document | Description |
|----------|-------------|
| [Animation Implementation](development/animation-impl.md) | How dynamic view animation works |
| [Connector Rendering Implementation](development/connector-rendering-implementation.md) | How to match Structurizr Java's connector rendering |
| [Directives Implementation](development/directives-impl.md) | DSL directive execution internals |
| [Drag-and-Drop Implementation](development/drag-drop-implementation.md) | Interactive element positioning with undo/redo |
| [Legend Implementation](development/legend-impl.md) | Legend rendering implementation |
| [Perspectives Implementation](development/perspectives-impl.md) | Perspectives filtering internals |
| [Validation Implementation](development/validation-impl.md) | Workspace validation internals |

### Scripting Implementation

| Document | Description |
|----------|-------------|
| [Scripting Implementation](development/scripting-impl.md) | ScriptEngine architecture and integration |
| [Transpiler Implementation](development/transpiler-impl.md) | Groovy-to-Lua transpiler internals |
| [Sandbox Implementation](development/sandbox-impl.md) | Security and sandboxing architecture |
| [Plugin System Implementation](development/plugin-system-impl.md) | WASM plugin runtime internals |

---

## Project Structure

```
structurizr-rs/
├── src/main.rs               # CLI entry point
├── crates/
│   ├── structurizr-core/     # C4 model types and workspace
│   ├── structurizr-dsl/      # DSL parser (lexer + parser)
│   ├── structurizr-render/   # SVG rendering
│   ├── structurizr-export/   # Export formats (JSON, PlantUML, Mermaid, etc.)
│   ├── structurizr-scripting/ # Lua scripting and WASM plugins
│   ├── structurizr-web/      # Axum-based web server
│   └── structurizr-mcp/      # Model Context Protocol server
├── docs/                     # This documentation
├── demo/                     # Demo workspace
├── workspaces/               # Example files
└── assets/                   # Diagrams, images, exports
```

---

## Getting Started

### Build and Run

```bash
# Build the project
cargo build --release

# Run the CLI
./target/release/structurizr --help

# Start the web server
./target/release/structurizr serve --data-dir demo --port 8080
```

### Validate a Workspace

```bash
./target/release/structurizr validate workspace.dsl
```

### Export Diagrams

```bash
# Export to JSON
./target/release/structurizr export --workspace workspace.dsl --format json

# Export to PlantUML
./target/release/structurizr export --workspace workspace.dsl --format plantuml

# Render to SVG
./target/release/structurizr render --workspace workspace.dsl --output ./output
```

---

## Contributing

See [CLAUDE.md](../CLAUDE.md) for development guidelines and file organization rules.
