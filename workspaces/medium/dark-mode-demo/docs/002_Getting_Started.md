# Getting Started

## Installation

### From Source

Clone the repository and build with Cargo:

```bash
git clone https://github.com/yourusername/structurizr-rs.git
cd structurizr-rs
cargo build --release
```

The binary will be available at `target/release/structurizr`.

### Using Cargo Install

```bash
cargo install structurizr-rs
```

## Creating Your First Workspace

Create a new file called `workspace.dsl`:

```dsl
workspace "My First System" "An example architecture" {
    !docs "docs"
    !adrs "adrs"

    model {
        user = person "User" "A user of the system"

        system = softwareSystem "My System" "The main system" {
            webapp = container "Web Application" "User interface" "React"
            api = container "API Server" "Business logic" "Rust"
            db = container "Database" "Data storage" "PostgreSQL"
        }

        user -> webapp "Uses"
        webapp -> api "Calls"
        api -> db "Reads/Writes"
    }

    views {
        systemContext system "Context" "System Context Diagram" {
            include *
            autoLayout
        }

        container system "Containers" "Container Diagram" {
            include *
            autoLayout
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
        }
    }
}
```

## Running the Server

Start the web server to view your diagrams:

```bash
structurizr serve --data-dir . --port 8080
```

Open http://localhost:8080 in your browser.

## CLI Commands

### `serve`
Start the web server:
```bash
structurizr serve --data-dir ./my-workspace --port 8080
```

### `validate`
Validate a DSL file:
```bash
structurizr validate workspace.dsl
```

### `render`
Render diagrams to SVG files:
```bash
structurizr render --workspace workspace.dsl --output ./diagrams
```

### `export`
Export to different formats:
```bash
# JSON format
structurizr export --workspace workspace.dsl --format json

# PlantUML format
structurizr export --workspace workspace.dsl --format plantuml

# Mermaid format
structurizr export --workspace workspace.dsl --format mermaid
```

### `init`
Create a new workspace:
```bash
structurizr init "My New System"
```

## Directory Structure

A typical workspace directory looks like:

```
my-workspace/
├── workspace.dsl     # The main DSL file
├── docs/             # Documentation (Markdown files)
│   ├── 001_Overview.md
│   └── 002_Architecture.md
└── adrs/             # Architecture Decision Records
    ├── 001-initial-architecture.md
    └── 002-database-choice.md
```

## Basic DSL Syntax

### Model Elements

```dsl
model {
    # People
    user = person "User" "Description"

    # Software Systems
    system = softwareSystem "System" "Description" {
        # Containers within a system
        webapp = container "Web App" "Description" "Technology"
        api = container "API" "Description" "Technology"

        # Components within a container (optional)
        api {
            controller = component "Controller" "Description" "Technology"
            service = component "Service" "Description" "Technology"
        }
    }

    # Relationships
    user -> system "Uses"
    webapp -> api "Calls" "HTTPS"
}
```

### Views

```dsl
views {
    # System Context View
    systemContext system "Key" "Description" {
        include *
        autoLayout
    }

    # Container View
    container system "Key" "Description" {
        include *
        autoLayout lr  # Left to right layout
    }

    # Component View
    component api "Key" "Description" {
        include *
        autoLayout
    }
}
```

### Styles

```dsl
views {
    styles {
        element "Person" {
            shape Person
            background "#08427b"
            color "#ffffff"
        }

        relationship "Relationship" {
            color "#707070"
            thickness 2
        }
    }
}
```

## Auto Layout Options

- `autoLayout` - Default top-to-bottom layout
- `autoLayout tb` - Top to bottom
- `autoLayout bt` - Bottom to top
- `autoLayout lr` - Left to right
- `autoLayout rl` - Right to left

## Next Steps

- Read the [DSL Reference](008_DSL_Reference.md) for complete syntax
- Learn about [Styling](009_Styling_Guide.md) your diagrams
- Explore [Dark Mode](010_Dark_Mode.md) features
