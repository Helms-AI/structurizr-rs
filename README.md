# Structurizr-rs

A native Rust implementation of [Structurizr Lite](https://structurizr.com/help/lite) for creating software architecture diagrams using the [C4 model](https://c4model.com/).

## Overview

Structurizr-rs parses Structurizr DSL files and renders interactive C4 model diagrams. It provides a local web server with live reload, SVG export, and multiple diagram export formats.

### Key Features

- **DSL Parser** - Full support for Structurizr DSL syntax including workspaces, models, views, and styles
- **SVG Rendering** - High-quality SVG output with proper shapes, colors, and styling
- **Web Server** - Interactive diagram viewer with pan, zoom, and live reload
- **Adaptive Layout** - Smart auto-layout that adjusts spacing based on element count
- **Multiple Export Formats** - JSON, PlantUML, Mermaid, D2, DOT, Ilograph, WebSequenceDiagrams
- **All C4 Views** - System Landscape, System Context, Container, Component, Dynamic, Deployment, Filtered
- **Themes** - Support for remote themes and custom styling

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later

### Build from Source

```bash
# Clone the repository
git clone https://github.com/Helms-AI/structurizr-rs.git
cd structurizr-rs

# Build release version
cargo build --release

# The binary will be at target/release/structurizr-rs
```

### Quick Start

```bash
# Run directly with cargo
cargo run -- serve --workspace path/to/workspace.dsl

# Or use the built binary
./target/release/structurizr-rs serve --workspace path/to/workspace.dsl
```

## Usage

### Command Line Interface

```bash
structurizr-rs [COMMAND] [OPTIONS]

Commands:
  serve     Start the web server to view diagrams
  render    Render diagrams to SVG files
  export    Export workspace to various formats
  validate  Validate a DSL file for errors
  init      Initialize a new workspace
  help      Print help information
```

### Starting the Web Server

```bash
# Basic usage
structurizr-rs serve --workspace workspace.dsl

# With custom port
structurizr-rs serve --workspace workspace.dsl --port 8080

# With live reload enabled (default)
structurizr-rs serve --workspace workspace.dsl --watch
```

Then open your browser to `http://localhost:8080` to view your diagrams.

### Rendering to SVG

```bash
# Render all views to SVG files
structurizr-rs render --workspace workspace.dsl --output ./output

# Render a specific view
structurizr-rs render --workspace workspace.dsl --view SystemContext --output ./output
```

### Exporting to Other Formats

```bash
# Export to JSON (Structurizr format)
structurizr-rs export --workspace workspace.dsl --format json --output workspace.json

# Export to PlantUML
structurizr-rs export --workspace workspace.dsl --format plantuml --output diagrams.puml

# Export to Mermaid
structurizr-rs export --workspace workspace.dsl --format mermaid --output diagrams.md

# Available formats: json, plantuml, mermaid, d2, dot, ilograph
```

### Validating DSL Files

```bash
structurizr-rs validate workspace.dsl
```

### Creating a New Workspace

```bash
structurizr-rs init "My System" --output workspace.dsl
```

## DSL Syntax

Structurizr-rs supports the standard [Structurizr DSL](https://docs.structurizr.com/dsl/language) syntax:

```dsl
workspace "My System" "Description of my system" {

    model {
        # Define people
        user = person "User" "A user of the system"

        # Define software systems
        system = softwareSystem "My System" "Does something useful" {
            webapp = container "Web Application" "Delivers content" "React"
            api = container "API" "Provides functionality" "Rust"
            database = container "Database" "Stores data" "PostgreSQL"
        }

        # Define relationships
        user -> webapp "Uses"
        webapp -> api "Makes API calls to" "HTTPS"
        api -> database "Reads from and writes to" "SQL"
    }

    views {
        # System Context diagram
        systemContext system "SystemContext" "System Context diagram" {
            include *
            autoLayout
        }

        # Container diagram
        container system "Containers" "Container diagram" {
            include *
            autoLayout
        }

        # Styling
        styles {
            element "Person" {
                shape Person
                background #08427b
                color #ffffff
            }
            element "Software System" {
                background #1168bd
                color #ffffff
            }
            element "Container" {
                background #438dd5
                color #ffffff
            }
            relationship "Relationship" {
                color #707070
                thickness 2
            }
        }
    }
}
```

### Supported Elements

| Element | Description |
|---------|-------------|
| `person` | A human user of the system |
| `softwareSystem` | A software system |
| `container` | A container within a software system |
| `component` | A component within a container |
| `deploymentEnvironment` | A deployment environment |
| `deploymentNode` | A deployment node |
| `infrastructureNode` | An infrastructure node |

### Supported Views

| View | Description |
|------|-------------|
| `systemLandscape` | Shows all systems and people |
| `systemContext` | Shows a system and its context |
| `container` | Shows containers within a system |
| `component` | Shows components within a container |
| `dynamic` | Shows runtime behavior |
| `deployment` | Shows deployment architecture |
| `filtered` | A filtered version of another view |
| `image` | An external image as a view |
| `custom` | A custom freeform view |

### Supported Shapes

`Box`, `RoundedBox`, `Circle`, `Ellipse`, `Hexagon`, `Cylinder`, `Pipe`, `Person`, `Robot`, `Folder`, `WebBrowser`, `MobileDevicePortrait`, `MobileDeviceLandscape`, `Component`

## Web Interface

The web server provides an interactive diagram viewer with:

- **Pan & Zoom** - Scroll to zoom, drag to pan
- **Fit to View** - Button to fit diagram to screen
- **Reset View** - Button to reset zoom and position
- **Download SVG** - Export the current view as SVG
- **Live Reload** - Automatic refresh when DSL file changes
- **View Navigation** - Easy switching between different views

## Project Structure

```
structurizr-rs/
├── src/main.rs              # CLI entry point
└── crates/
    ├── structurizr-core/    # Core types (Model, View, Style, Workspace)
    ├── structurizr-dsl/     # DSL lexer and parser
    ├── structurizr-render/  # SVG rendering and layout
    ├── structurizr-export/  # Export to various formats
    └── structurizr-web/     # Web server and handlers
```

## Workspaces

The `workspaces/` directory contains sample workspaces of varying complexity:

```
workspaces/
├── small/           # 5-10 elements
│   ├── startup-saas/
│   └── clinic-management/
├── medium/          # 15-30 elements
│   ├── ecommerce-platform/
│   └── fintech-payments/
└── large/           # 50+ elements
    ├── enterprise-healthcare/
    └── manufacturing-iot/
```

Run all workspaces with multi-workspace mode:

```bash
# Serves all workspaces with an index page
cargo run -- serve --workspaces-dir workspaces

# Or run a single workspace
cargo run -- serve --data-dir workspaces/small/startup-saas
```

## Configuration

### Auto Layout

Control diagram layout with the `autoLayout` directive:

```dsl
views {
    container system "Containers" {
        include *
        autoLayout tb 100 100  # direction, rank_separation, node_separation
    }
}
```

Directions: `tb` (top-bottom), `bt` (bottom-top), `lr` (left-right), `rl` (right-left)

### Themes

Apply remote themes to your diagrams:

```dsl
workspace {
    !theme https://static.structurizr.com/themes/amazon-web-services-2023.01.31/theme.json

    model {
        # ...
    }
}
```

## Development

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture
```

### Building Documentation

```bash
cargo doc --workspace --open
```

## Comparison with Structurizr Lite

| Feature | Structurizr Lite | Structurizr-rs |
|---------|------------------|----------------|
| Language | Java | Rust |
| Startup Time | ~5 seconds | <100ms |
| Memory Usage | ~200MB | ~10MB |
| DSL Support | Full | Most features |
| Diagram Editor | Yes | View only |
| Export Formats | JSON, PlantUML | JSON, PlantUML, Mermaid, D2, DOT, Ilograph |

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## License

This project is open source. See the repository for license details.

## Acknowledgments

- [Structurizr](https://structurizr.com/) by Simon Brown for the original implementation and DSL specification
- [C4 Model](https://c4model.com/) for the architecture diagramming approach
