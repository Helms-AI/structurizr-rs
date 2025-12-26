# CLAUDE.md - Claude Code Development Guide

This file provides guidance for Claude Code when working on the structurizr-rs project.

## Project Overview

structurizr-rs is a Rust implementation of Structurizr Lite, a tool for creating software architecture diagrams using the C4 model. The original Structurizr Lite is written in Java; this is a native Rust port.

## Architecture

### Workspace Structure

The project uses a Cargo workspace with the following crates:

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

## Key Components

### structurizr-core

- **model.rs**: C4 model elements (Person, SoftwareSystem, Container, Component, DeploymentNode, Relationship)
- **view.rs**: View types (SystemLandscapeView, SystemContextView, ContainerView, ComponentView)
- **style.rs**: Styling (ElementStyle, RelationshipStyle, Shape, Border)
- **workspace.rs**: Workspace struct that holds model, views, and documentation

### structurizr-dsl

- **lexer.rs**: Tokenizer for DSL input (keywords, strings, identifiers, arrows)
- **parser.rs**: Recursive descent parser that builds AST and converts to Workspace
- **ast.rs**: AST node types for the parsed DSL

### structurizr-render

- **svg.rs**: SVG renderer for diagrams
- **layout.rs**: Grid-based auto-layout algorithm

### structurizr-export

- **json.rs**: Structurizr JSON format export
- **plantuml.rs**: PlantUML C4 export
- **mermaid.rs**: Mermaid flowchart export

### structurizr-web

- **server.rs**: Axum web server
- **handlers.rs**: HTTP request handlers
- **state.rs**: Application state and configuration

## Build Commands

```bash
# Build the project
cargo build

# Build release version
cargo build --release

# Run tests
cargo test

# Run the CLI
cargo run -- --help

# Run with a specific command
cargo run -- validate workspace.dsl
cargo run -- serve --port 8080
cargo run -- init "My System"
cargo run -- render --workspace workspace.dsl --output ./output
cargo run -- export --workspace workspace.dsl --format json
```

## DSL Syntax

The Structurizr DSL is the primary input format:

```dsl
workspace "Name" "Description" {
    model {
        user = person "User" "Description"
        system = softwareSystem "System" "Description" {
            container = container "Container" "Description" "Technology"
        }
        user -> system "Uses"
    }
    views {
        systemContext system "ViewKey" "Description" {
            include *
            autoLayout
        }
        styles {
            element "Person" {
                shape Person
                background "#08427b"
            }
        }
    }
}
```

## Important Implementation Details

### Raw String Syntax

When embedding strings with `#` characters (like hex colors) in Rust raw strings, use double-hash delimiters:
```rust
// Use r##"..."## instead of r#"..."# when string contains "#
svg.push_str(r##"fill="#707070""##);
```

### Parser Keywords as Identifiers

Some DSL keywords (like `Person`, `Component`) are also valid shape names. The parser's `expect_identifier_or_shape_keyword()` method handles this.

### Element IDs

Elements have unique IDs generated from their names using UUID v5. The `ElementId` type wraps a UUID.

## Testing

- Unit tests are in each crate
- Integration tests for the DSL parser test various DSL constructs
- The `validate` CLI command is useful for testing DSL parsing

## Common Tasks

### Adding a New DSL Keyword

1. Add to `TokenKind` enum in `lexer.rs`
2. Add keyword mapping in `Lexer::keyword_or_identifier()`
3. Add parsing logic in `parser.rs`
4. Add AST node type in `ast.rs` if needed

### Adding a New Export Format

1. Create a new module in `structurizr-export`
2. Implement the exporter
3. Add to `lib.rs` exports
4. Add CLI command support in `main.rs`

### Adding a New View Type

1. Add view struct to `structurizr-core/src/view.rs`
2. Add to `Views` struct
3. Add parsing in `structurizr-dsl/src/parser.rs`
4. Add rendering in `structurizr-render/src/svg.rs`

## File Organization Guidelines

> **CRITICAL**: Follow these rules for all file organization in this project.

### Documentation Structure

All project documentation MUST be placed in the `/docs` directory:

```
/docs/
├── index.md              # Documentation home - ALWAYS update when adding docs
├── features/             # User-facing feature guides
│   └── <feature>.md      # How to USE a feature
└── development/          # Implementation/developer documentation
    └── <feature>-impl.md # HOW a feature is IMPLEMENTED
```

**Documentation Rules:**
1. **Never create documentation files in the project root** (except CLAUDE.md)
2. **User guides** go in `/docs/features/` - focus on usage, examples, API
3. **Implementation docs** go in `/docs/development/` - focus on internals, architecture
4. **Always update `/docs/index.md`** when adding new documentation
5. **Context-specific READMEs** (like `demo/README.md`) may stay in their directories

### Workspace Structure

> **CRITICAL**: All workspaces MUST follow this structure.

Workspaces are stored in `/workspaces/{size}/{workspace-name}/`:

```
workspaces/{size}/{workspace-name}/
├── workspace.dsl     # The DSL workspace file (with !docs "docs" directive)
├── README.md         # Brief overview for GitHub browsing
├── docs/             # Comprehensive documentation (referenced by !docs)
│   ├── index.md      # Main documentation page
│   └── *.md          # Additional documentation files
└── adrs/             # Architecture Decision Records (referenced by !adrs)
    └── *.md          # ADR files (001-*.md, 002-*.md, etc.)
```

**Size categories:**
- `small/` - Simple workspaces (1-3 containers)
- `medium/` - Moderate complexity (4-8 containers)
- `large/` - Complex workspaces (9+ containers, deployment views)

**Documentation Requirements:**

1. **README.md** - Brief overview for GitHub browsing
2. **docs/index.md** - Comprehensive documentation (rendered in web UI via !docs directive):
   - Overview and purpose of the workspace
   - DSL features demonstrated
   - Business context and use cases
   - Architecture overview
   - How to run and explore the workspace
3. **adrs/*.md** - Architecture Decision Records (rendered in web UI via !adrs directive)

**workspace.dsl MUST include:**
```dsl
!docs "docs"
!adrs "adrs"
```

**Running workspaces:**
```bash
# Multi-workspace mode (serves all workspaces with an index page)
cargo run -- serve --workspaces-dir workspaces

# Single workspace mode (legacy)
cargo run -- serve --data-dir workspaces/small/startup-saas
```

**Do NOT:**
- Put loose `.dsl` files directly in `/workspaces/`
- Create workspaces without `README.md` and `docs/` folder
- Create workspace.dsl without `!docs "docs"` directive

### Temporary/Throwaway Files

Only throwaway files from Claude Code processes go in `/tmp/`:

```
/tmp/                     # Gitignored - never commit
├── *.txt                 # Validation output, test logs
└── *.json                # Temporary debug output
```

**Note:** This is ONLY for files that are not needed after the session ends.

### Assets (Diagrams, Images, etc.)

Persistent assets like SVG diagrams, images, and other resources go in `/assets/`:

```
/assets/
├── diagrams/             # Generated or hand-crafted diagrams
│   └── *.svg
├── images/               # Images used in documentation
└── exports/              # Exported workspace files
```

**File Placement Rules:**
1. **Never put workspace files in the project root** - use `/workspaces/`
2. **Never put documentation in the project root** - use `/docs/`
3. **Throwaway/debug files** go in `/tmp/` (gitignored)
4. **Persistent assets (SVG, images)** go in `/assets/` (committed)
5. **The `/tmp/` directory is gitignored** - only for truly temporary output
