# Structurizr-rs Demo

This directory contains example workspaces to demonstrate the structurizr-rs CLI.

## Quick Start

From the project root directory:

```bash
# Build the CLI (if not already built)
cargo build --release

# Validate the demo workspace
./target/release/structurizr validate demo/workspace.dsl

# Render all views to SVG
./target/release/structurizr render --workspace demo/workspace.dsl --output demo/output

# Export to different formats
./target/release/structurizr export --workspace demo/workspace.dsl --format json --output demo/output/workspace.json
./target/release/structurizr export --workspace demo/workspace.dsl --format plantuml
./target/release/structurizr export --workspace demo/workspace.dsl --format mermaid

# Start the web server
./target/release/structurizr serve --data-dir demo --port 8080
```

Then open http://localhost:8080 in your browser.

## Example Workspace

The `workspace.dsl` file contains a fictional "Big Bank plc" internet banking system with:

- **People**: Personal Banking Customer
- **Software Systems**:
  - Internet Banking System (main)
  - Mainframe Banking System (external)
  - E-mail System (external)
- **Containers**:
  - Web Application (Java/Spring MVC)
  - Single-Page Application (React)
  - Mobile App (React Native)
  - API Application (Spring Boot)
  - Database (PostgreSQL)

## Views

The workspace defines three views:

1. **SystemLandscape** - Overview showing all systems and people
2. **SystemContext** - Context diagram for the Internet Banking System
3. **Containers** - Container diagram showing internal structure

## Creating Your Own Workspace

```bash
# Initialize a new workspace
./target/release/structurizr init "My System" --output my-workspace.dsl

# Edit the generated file, then validate
./target/release/structurizr validate my-workspace.dsl
```
