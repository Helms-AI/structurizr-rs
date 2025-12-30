# CLAUDE.md - Claude Code Development Guide

This file provides guidance for Claude Code when working on the structurizr-rs project.

## Project Overview

structurizr-rs is a Rust implementation of Structurizr Lite, a tool for creating software architecture diagrams using the C4 model. The original Structurizr Lite is written in Java; this is a native Rust port.

---

## ⚠️ CRITICAL: DSL Backwards Compatibility Requirement

> **This is essential for the project's survival.**

**ALL features that interact with DSL files MUST be 100% backwards compatible with the official Structurizr Java implementation:**
- https://github.com/structurizr/java

### Rules

1. **DO NOT invent new DSL syntax** - Any `.dsl` file created with structurizr-rs MUST be parseable by the official Structurizr Java tooling.

2. **DO NOT modify existing DSL keywords or behavior** - The semantics of all DSL constructs must match the Java implementation exactly.

3. **Extensions ONLY via plugins** - If you need to extend functionality beyond what the official DSL supports, implement it using the plugin system in the same way Structurizr Java does it.

4. **Test compatibility** - Before adding any DSL-related feature, verify that:
   - The syntax exists in the official Structurizr DSL grammar
   - The behavior matches the Java implementation
   - Workspace files remain interoperable

5. **Web UI enhancements are OK** - You MAY add features to the web interface, rendering, or export formats that don't affect DSL parsing/writing.

### Why This Matters

Users must be able to:
- Use `.dsl` files interchangeably between structurizr-rs and Structurizr Java/Lite
- Migrate to/from the official tooling without any file modifications
- Trust that their architecture documentation remains portable

**When in doubt, check the official Structurizr Java repository for clarification on how functionality works.**

---

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

Workspaces can be organized in any folder structure within `/workspaces/`. The system **recursively discovers** all directories containing a `workspace.dsl` file at any depth.

```
workspaces/
├── my-workspace/               # Top-level workspace
│   └── workspace.dsl
├── team-a/                     # Grouped by team
│   ├── project-x/
│   │   └── workspace.dsl
│   └── project-y/
│       └── workspace.dsl
├── small/                      # Traditional size-based grouping
│   └── startup-saas/
│       └── workspace.dsl
└── demos/deep/nested/          # Arbitrary nesting depth
    └── example/
        └── workspace.dsl
```

**Workspace ID** = relative path from workspaces root:
- `my-workspace` → URL: `/w/my-workspace`
- `team-a/project-x` → URL: `/w/team-a/project-x`
- `demos/deep/nested/example` → URL: `/w/demos/deep/nested/example`

**Recommended workspace contents:**
```
workspace-name/
├── workspace.dsl     # The DSL workspace file (with !docs "docs" directive)
├── README.md         # Brief overview for GitHub browsing
├── docs/             # Comprehensive documentation (referenced by !docs)
│   ├── index.md      # Main documentation page
│   └── *.md          # Additional documentation files
└── adrs/             # Architecture Decision Records (referenced by !adrs)
    └── *.md          # ADR files (001-*.md, 002-*.md, etc.)
```

**Recommended conventions** (not enforced):
- Group by team, project, or purpose
- Keep nesting reasonable (2-3 levels max for readability)
- Use descriptive folder names
- Size categories (`small/`, `medium/`, `large/`) can still be used if helpful

**workspace.dsl SHOULD include:**
```dsl
!docs "docs"
!adrs "adrs"
```

**Running workspaces:**
```bash
# Multi-workspace mode (serves all workspaces with a grouped index page)
cargo run -- serve --workspaces-dir workspaces
```

**Do NOT:**
- Put loose `.dsl` files directly in `/workspaces/` without a folder
- Create `workspace.dsl` files without at least a parent directory

**Workspace dotfiles (ALWAYS commit these):**
- `.notes.json` - User notes/comments on dynamic view steps
- `.positions.json` - Custom element positions for diagram layouts

These files store user-generated data and customizations. Always commit them when they appear in workspace directories.

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
