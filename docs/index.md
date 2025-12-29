# structurizr-rs Documentation

Welcome to the documentation for **structurizr-rs**, a Rust implementation of Structurizr Lite for creating software architecture diagrams using the C4 model.

## Quick Links

- [Demo Workspace](../demo/README.md) - Get started with the demo
- [Examples](../workspaces/) - Example DSL files and code

---

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

---

## Development Documentation

Implementation details and architecture documentation:

| Document | Description |
|----------|-------------|
| [Animation Implementation](development/animation-impl.md) | How dynamic view animation works |
| [Connector Rendering Implementation](development/connector-rendering-implementation.md) | How to match Structurizr Java's connector rendering |
| [Directives Implementation](development/directives-impl.md) | DSL directive execution internals |
| [Drag-and-Drop Implementation](development/drag-drop-implementation.md) | Interactive element positioning with undo/redo |
| [Legend Implementation](development/legend-impl.md) | Legend rendering implementation |
| [Perspectives Implementation](development/perspectives-impl.md) | Perspectives filtering internals |
| [Validation Implementation](development/validation-impl.md) | Workspace validation internals |

---

## Project Structure

```
structurizr-rs/
├── src/main.rs             # CLI entry point
├── crates/
│   ├── structurizr-core/   # C4 model types and workspace
│   ├── structurizr-dsl/    # DSL parser (lexer + parser)
│   ├── structurizr-render/ # SVG rendering
│   ├── structurizr-export/ # Export formats (JSON, PlantUML, Mermaid, etc.)
│   └── structurizr-web/    # Axum-based web server
├── docs/                   # This documentation
├── demo/                   # Demo workspace
├── workspaces/               # Example files
└── assets/                 # Diagrams, images, exports
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
