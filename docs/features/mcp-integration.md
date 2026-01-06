# MCP Integration for structurizr-rs

## Overview

structurizr-rs includes a Model Context Protocol (MCP) server that enables AI assistants like Claude to interact with C4 architecture diagrams through natural language. This integration exposes workspace management, diagram rendering, and export capabilities via a standardized RPC interface using the official Rust MCP SDK (rmcp).

## Features

### Current Capabilities
- **Workspace Management**: Discover, list, load, and validate workspaces
- **Model Inspection**: View people, systems, containers, and relationships
- **Model Manipulation**: Add, update, and remove people, software systems, containers, components, and relationships
- **View Creation**: Create all C4 view types (landscape, context, container, component, dynamic, deployment)
- **View Management**: Add/remove elements, configure auto-layout settings
- **Documentation Management**: Add, update, remove, and list documentation sections
- **ADR Management**: Create, update, and list Architecture Decision Records
- **SVG Rendering**: Render any view to high-quality SVG diagrams
- **Multi-Format Export**: Export views to PlantUML, Mermaid, D2, DOT/Graphviz, and JSON
- **Search**: Find elements by name or description
- **Multiple Transports**: stdio, WebSocket, and HTTP/SSE for flexible integration
- **Web Proxy Integration**: Access MCP via `/mcp/*` endpoints on the web server

### Available Tools (40 total)

#### Read Operations (7 tools)
| Tool | Description |
|------|-------------|
| `workspace_list` | List all available workspaces with metadata |
| `workspace_load` | Load a workspace and get detailed statistics |
| `workspace_validate` | Validate workspace for errors and warnings |
| `workspace_export_json` | Export workspace to Structurizr JSON format |
| `workspace_get_model` | Get model elements (people, systems, containers) |
| `workspace_get_views` | List all views defined in a workspace |
| `workspace_search` | Search for elements by name or description |

#### Render & Export Operations (5 tools)
| Tool | Description |
|------|-------------|
| `render_svg` | Render a specific view to SVG format |
| `export_plantuml` | Export a view to PlantUML C4 format |
| `export_mermaid` | Export a view to Mermaid format |
| `export_d2` | Export a view to D2 format |
| `export_dot` | Export a view to DOT/Graphviz format |

#### Model Manipulation - Create (5 tools)
| Tool | Description |
|------|-------------|
| `model_add_person` | Add a new person to the workspace model |
| `model_add_system` | Add a new software system to the workspace model |
| `model_add_container` | Add a new container to an existing software system |
| `model_add_component` | Add a new component to an existing container |
| `model_add_relationship` | Add a relationship between two elements |

#### Model Manipulation - Update/Delete (3 tools)
| Tool | Description |
|------|-------------|
| `model_update_element` | Update name, description, or technology of any element |
| `model_remove_element` | Remove an element and optionally its relationships |
| `model_list_pending_changes` | Show current modification status of a workspace |

#### View Creation (6 tools)
| Tool | Description |
|------|-------------|
| `view_create_system_landscape` | Create a system landscape view showing all people and systems |
| `view_create_system_context` | Create a system context view for a software system |
| `view_create_container` | Create a container view showing containers within a system |
| `view_create_component` | Create a component view showing components within a container |
| `view_create_dynamic` | Create a dynamic view showing interactions over time |
| `view_create_deployment` | Create a deployment view showing infrastructure deployment |

#### View Management (4 tools)
| Tool | Description |
|------|-------------|
| `view_add_element` | Add an element to an existing view |
| `view_add_all_elements` | Add all elements to a view (equivalent to `include *`) |
| `view_remove_element` | Remove an element from a view |
| `view_set_auto_layout` | Configure auto-layout settings for a view |

#### Documentation Management (4 tools)
| Tool | Description |
|------|-------------|
| `docs_add_section` | Add a documentation section to a workspace |
| `docs_update_section` | Update an existing documentation section |
| `docs_remove_section` | Remove a documentation section from a workspace |
| `docs_list_sections` | List all documentation sections in a workspace |

#### ADR Management (3 tools)
| Tool | Description |
|------|-------------|
| `adr_create` | Create a new Architecture Decision Record (ADR) |
| `adr_update` | Update an existing ADR (title, content, or status) |
| `adr_list` | List all ADRs in a workspace |

#### Persistence (3 tools)
| Tool | Description |
|------|-------------|
| `workspace_save_json` | Save modified workspace to Structurizr-compatible JSON |
| `workspace_save_dsl` | Save modified workspace to Structurizr DSL format |
| `workspace_discard_changes` | Discard pending modifications and revert to original |

## Installation

The MCP server is included as an optional feature. Build with:

```bash
cargo build --features mcp
```

## Configuration

### Configuration File

structurizr-rs supports configuration via `structurizr.toml` file. The configuration system provides:

- **Multiple locations**: Project root, user config, system config
- **Profile support**: Development, production, and custom profiles
- **Environment overrides**: Use environment variables to override config values
- **Workspace scoping**: Control which workspaces MCP can access

#### Configuration Search Order

1. `$STRUCTURIZR_CONFIG` environment variable (explicit path)
2. `./structurizr.toml` (project root)
3. `~/.config/structurizr/config.toml` (user config)
4. `/etc/structurizr/config.toml` (system config)

#### Example Configuration

```toml
[meta]
version = "1.0"
profile = "development"  # or use STRUCTURIZR_PROFILE env var

[server]
port = 8080  # or use STRUCTURIZR_PORT env var
host = "127.0.0.1"
workspaces_dir = "workspaces"  # or use STRUCTURIZR_WORKSPACES_DIR

[mcp]
enabled = true
auto_start = true

[mcp.server]
port = 8586
transport = "websocket"  # stdio | websocket | sse
health_check_interval_ms = 30000

[mcp.workspace_scope]
mode = "all"  # all | allow | deny
patterns = [
    "team-a/*",      # Include team-a workspaces
    "!team-a/secret", # Exclude secret workspace
]
auto_include_created = true

[collaboration]
enable_notifications = true

[collaboration.crdt]
enabled = true
algorithm = "yjs"

# Profile overrides
[profiles.production]
[profiles.production.mcp.workspace_scope]
mode = "deny"
patterns = ["public/*"]
```

### Workspace Scoping

Control which workspaces are accessible to MCP sessions:

- **`all`**: All workspaces accessible (development default)
- **`allow`**: Only workspaces matching patterns
- **`deny`**: All except workspaces matching patterns

Patterns support:
- Glob patterns: `team-a/*`, `**/*.tmp`
- Negation: `!secret/*`
- Auto-include: Workspaces created during session are automatically accessible

## Usage

### Starting the MCP Server

```bash
# Start with stdio transport (for Claude Desktop, Cursor, Claude Code, etc.)
structurizr mcp-serve --workspaces-dir ./workspaces

# Start with WebSocket transport
structurizr mcp-serve --workspaces-dir ./workspaces --transport websocket --port 8586

# Start with HTTP/SSE transport
structurizr mcp-serve --workspaces-dir ./workspaces --transport sse --port 8586
```

### Using the Web Proxy

When running the web server, MCP is automatically available via proxy endpoints:

```bash
# Start the web server (MCP proxy enabled by default)
structurizr serve --port 8080

# MCP is now available at:
# - WebSocket: ws://localhost:8080/mcp/ws
# - Health check: http://localhost:8080/mcp/health
```

### Configuring Claude Desktop

Add to your Claude Desktop configuration (`~/Library/Application Support/Claude/claude_desktop_config.json` on macOS):

```json
{
  "mcpServers": {
    "structurizr": {
      "command": "/path/to/structurizr",
      "args": ["mcp-serve", "--workspaces-dir", "/path/to/workspaces"],
      "env": {}
    }
  }
}
```

### Configuring Claude Code

Add to your Claude Code MCP settings:

```json
{
  "mcpServers": {
    "structurizr": {
      "command": "structurizr",
      "args": ["mcp-serve", "--workspaces-dir", "./workspaces"]
    }
  }
}
```

## Architecture

### Crate Structure

```
crates/structurizr-mcp/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── server.rs           # Full MCP server with rmcp macros
│   ├── server_simple.rs    # Fallback simplified implementation
│   ├── state.rs            # State management
│   ├── error.rs            # Error types
│   ├── tools/              # Tool implementations
│   └── transport/          # Transport implementations
```

### Technology Stack

- **rmcp v0.12**: Official Rust MCP SDK from modelcontextprotocol
- **schemars v1.2**: JSON Schema generation for tool parameters
- **tokio**: Async runtime for server operations
- **tracing**: Structured logging
- **tokio-tungstenite**: WebSocket transport support
- **axum**: HTTP/SSE transport and web proxy integration

### State Management

The MCP server shares the workspace registry with the web server, enabling:
- Unified workspace discovery
- Shared caching
- Consistent state across interfaces

## API Reference

### Workspace Tools

#### workspace_list
Lists all available workspaces.

**Parameters**: None

**Returns**: JSON with workspace IDs, names, descriptions, and view counts.

```json
{
  "workspaces": [
    {
      "id": "platform/horizon",
      "name": "Horizon Platform",
      "description": "Cloud IDE platform",
      "view_count": 17
    }
  ]
}
```

#### workspace_load
Load a workspace and get detailed information.

**Parameters**:
- `workspace_id` (string): The unique identifier of the workspace

**Returns**: Statistics including people count, systems count, relationships count, and views count.

#### workspace_validate
Validate a workspace for errors, warnings, and issues.

**Parameters**:
- `workspace_id` (string): The workspace to validate

**Returns**: Validation results with error/warning/info counts and issue messages.

#### workspace_get_model
Get detailed model elements including people, software systems, containers, and relationships.

**Parameters**:
- `workspace_id` (string): The workspace to inspect

**Returns**: Markdown-formatted list of all model elements.

#### workspace_get_views
List all views defined in a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to inspect

**Returns**: Categorized list of views by type (landscape, context, container, component, dynamic, deployment).

#### workspace_search
Search for elements by name or description.

**Parameters**:
- `workspace_id` (string): The workspace to search
- `query` (string): The search query

**Returns**: Matching elements with their types and descriptions.

### Render and Export Tools

#### render_svg
Render a view to SVG format.

**Parameters**:
- `workspace_id` (string): The workspace containing the view
- `view_key` (string): The key of the view to render

**Returns**: Complete SVG markup for the diagram.

#### export_plantuml
Export a view to PlantUML C4 format.

**Parameters**:
- `workspace_id` (string): The workspace containing the view
- `view_key` (string): The key of the view to export

**Returns**: PlantUML code using C4-PlantUML syntax.

#### export_mermaid
Export a view to Mermaid format.

**Parameters**:
- `workspace_id` (string): The workspace containing the view
- `view_key` (string): The key of the view to export

**Returns**: Mermaid diagram code.

#### export_d2
Export a view to D2 format.

**Parameters**:
- `workspace_id` (string): The workspace containing the view
- `view_key` (string): The key of the view to export

**Returns**: D2 diagram code.

#### export_dot
Export a view to DOT/Graphviz format.

**Parameters**:
- `workspace_id` (string): The workspace containing the view
- `view_key` (string): The key of the view to export

**Returns**: DOT graph code.

### Model Manipulation Tools

These tools allow you to modify workspace models programmatically. Changes are held in memory until explicitly saved with `workspace_save_json`.

#### model_add_person
Add a new person to the workspace model.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `name` (string): The name of the person
- `description` (string, optional): Description of the person
- `external` (boolean, optional): Whether the person is external to the organization

**Returns**: JSON with element_id, element_type, name, and confirmation message.

#### model_add_system
Add a new software system to the workspace model.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `name` (string): The name of the software system
- `description` (string, optional): Description of the system
- `external` (boolean, optional): Whether the system is external to the organization

**Returns**: JSON with element_id, element_type, name, and confirmation message.

#### model_add_container
Add a new container to an existing software system.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `system_name` (string): The name of the parent software system
- `name` (string): The name of the container
- `description` (string, optional): Description of the container
- `technology` (string, optional): Technology/framework used (e.g., "Spring Boot", "PostgreSQL")

**Returns**: JSON with element_id, element_type, name, and confirmation message.

#### model_add_relationship
Add a relationship between two elements.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `source_name` (string): Name of the source element (person, system, or container)
- `destination_name` (string): Name of the destination element
- `description` (string, optional): Description of the relationship (e.g., "Uses", "Sends data to")
- `technology` (string, optional): Technology used (e.g., "REST/HTTP", "gRPC")

**Returns**: JSON with relationship_id, source, destination, description, and confirmation message.

#### workspace_save_json
Save a modified workspace to JSON format. The output is Structurizr-compatible and can be imported into Structurizr.

**Parameters**:
- `workspace_id` (string): The workspace to save
- `filename` (string, optional): Output filename (defaults to workspace_id.json)

**Returns**: JSON with workspace_id, filename, and confirmation message.

#### workspace_save_dsl
Save a modified workspace to DSL format. The output is compatible with Structurizr Java tooling and can be used as a workspace.dsl file.

**Parameters**:
- `workspace_id` (string): The workspace to save
- `filename` (string, optional): Output filename (defaults to workspace_id_modified.dsl)

**Returns**: JSON with workspace_id, filename, and confirmation message.

#### workspace_discard_changes
Discard any pending modifications to a workspace, reverting to the original state from the DSL file.

**Parameters**:
- `workspace_id` (string): The workspace to revert

**Returns**: Confirmation message.

#### model_add_component
Add a new component to an existing container within a software system.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `system_name` (string): The name of the parent software system
- `container_name` (string): The name of the parent container
- `name` (string): The name of the component
- `description` (string, optional): Description of the component
- `technology` (string, optional): Technology used (e.g., "Spring MVC Controller", "React Component")

**Returns**: JSON with element_id, element_type, name, and confirmation message.

#### model_update_element
Update properties of an existing element (person, software system, container, or component).

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `element_name` (string): The current name of the element to update
- `new_name` (string, optional): New name for the element
- `new_description` (string, optional): New description for the element
- `new_technology` (string, optional): New technology (for containers/components only)

**Returns**: JSON with element_name, element_type, list of changes, and confirmation message.

#### model_remove_element
Remove an element from the workspace model.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `element_name` (string): The name of the element to remove
- `cascade_relationships` (boolean, default: true): If true, also removes all relationships involving this element

**Returns**: JSON with element_name, element_type, relationships_removed count, and confirmation message.

#### model_list_pending_changes
List all elements that have been added or modified in a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to check

**Returns**: Markdown-formatted summary showing current element counts and modification status.

### Documentation Management Tools

#### docs_add_section
Add a documentation section to a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `title` (string): Title of the documentation section
- `content` (string): Content of the section (Markdown format)
- `order` (number, optional): Order/position of the section (defaults to end)

**Returns**: JSON with title, order, and confirmation message.

#### docs_update_section
Update an existing documentation section.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `section_title` (string): Title of the section to update
- `new_title` (string, optional): New title for the section
- `new_content` (string, optional): New content for the section

**Returns**: JSON with updated title, order, and confirmation message.

#### docs_remove_section
Remove a documentation section from a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `section_title` (string): Title of the section to remove

**Returns**: JSON with title and confirmation message.

#### docs_list_sections
List all documentation sections in a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to inspect

**Returns**: Markdown-formatted list of all sections with titles, order, and content previews.

### ADR Management Tools

#### adr_create
Create a new Architecture Decision Record (ADR).

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `adr_id` (string): Unique ID for the ADR (e.g., 'ADR-001', '1')
- `title` (string): Title of the architecture decision
- `content` (string): Content/body of the ADR (Markdown format)
- `status` (string, optional): Status: 'Proposed', 'Accepted', 'Superseded', 'Deprecated', 'Rejected' (default: Proposed)
- `date` (string, optional): Date of the decision (YYYY-MM-DD format, defaults to today)

**Returns**: JSON with adr_id, title, status, and confirmation message.

#### adr_update
Update an existing Architecture Decision Record.

**Parameters**:
- `workspace_id` (string): The workspace to modify
- `adr_id` (string): ID of the ADR to update
- `new_title` (string, optional): New title
- `new_content` (string, optional): New content
- `new_status` (string, optional): New status: 'Proposed', 'Accepted', 'Superseded', 'Deprecated', 'Rejected'

**Returns**: JSON with adr_id, title, status, and confirmation message.

#### adr_list
List all Architecture Decision Records in a workspace.

**Parameters**:
- `workspace_id` (string): The workspace to inspect

**Returns**: Markdown table of all ADRs with ID, title, status, and date.

## Example Usage

Once configured, you can interact with structurizr-rs through Claude:

### Read Operations
```
User: "List all available workspaces"
Claude: Found 14 workspaces including:
  - platform/horizon: Horizon Platform (17 views)
  - core/pos-terminal: FreshMart POS Terminal (16 views)
  ...

User: "Load the horizon workspace and show me the views"
Claude: Loaded workspace: Horizon Platform
  Views include:
  - SystemLandscape: Platform overview
  - Containers: IDE architecture
  - Deployment: Cloud infrastructure
  ...

User: "Render the SystemLandscape view to SVG"
Claude: [Returns SVG diagram]

User: "Export the Containers view to PlantUML"
Claude: [Returns PlantUML code]
```

### Model Manipulation
```
User: "Add a new user called 'Mobile Developer' to the horizon workspace"
Claude: Added person 'Mobile Developer' to workspace 'platform/horizon'.
  Element ID: 37c6b670-7c7b-4b86-9625-5b94fa860e99

User: "Add a new system called 'Mobile App' that the Mobile Developer uses"
Claude: Added software system 'Mobile App' to workspace.
  Element ID: 98b6915b-3a32-41ec-ad02-bc287c895a16

User: "Add a relationship from Mobile Developer to Mobile App"
Claude: Added relationship from 'Mobile Developer' to 'Mobile App'.
  Relationship: Uses

User: "Save the changes to a JSON file"
Claude: Workspace saved to 'workspaces/platform_horizon.json'.
  The JSON file is Structurizr-compatible and can be imported.
```

## Development Status

### Phase 1 ✅ Complete
- Basic MCP server structure
- Workspace discovery and loading
- CLI integration
- State management

### Phase 2 ✅ Complete
- Full rmcp SDK integration with macros
- All workspace management tools
- SVG rendering tools
- Multi-format export tools (PlantUML, Mermaid, D2, DOT)

### Phase 3 ✅ Complete
- Model manipulation tools (add person, system, container, relationship)
- In-memory workspace modification with copy-on-write
- JSON export for modified workspaces
- Change discard/revert functionality

### Phase 4 ✅ Complete
- Model add component tool (complete C4 hierarchy)
- Model update element tool (rename, change description/technology)
- Model remove element tool (with cascade relationship deletion)
- Pending changes listing tool

### Phase 5 ✅ Complete
- DSL serialization (write back to .dsl files via `workspace_save_dsl`)
- Round-trip compatibility testing

### Phase 6 ✅ Complete
- View creation tools (all 6 C4 view types)
- View element management (add, remove, auto-layout)
- Documentation management tools (add, update, remove, list sections)
- ADR management tools (create, update, list)

### Phase 7 ✅ Complete
- WebSocket transport via tokio-tungstenite
- HTTP/SSE transport via Axum
- Web server proxy integration (`/mcp/*` endpoints)
- Automatic MCP process spawning from web server
- Health monitoring and reconnection support
- rmcp upgraded to v0.12

### Phase 8 ✅ Complete
- TOML configuration file support (structurizr.toml)
- Configuration profiles (development, production)
- Workspace scoping with glob patterns and auto-include
- CRDT foundation for collaborative editing
- Environment variable interpolation
- Config file discovery (project, user, system locations)

## Security Considerations

- Read operations have no restrictions
- Write operations modify an in-memory copy (original DSL files are preserved)
- Changes must be explicitly saved via `workspace_save_json` or `workspace_save_dsl`
- Script execution is sandboxed
- Resource limits for rendering operations

## Troubleshooting

### MCP Server Won't Start
- Ensure workspaces directory exists and contains valid workspaces
- Check that no other process is using the same transport
- Verify the binary has execute permissions
- Check logs: `RUST_LOG=debug structurizr mcp-serve ...`

### Claude Can't Connect
- Verify Claude Desktop/Code configuration path is correct
- Ensure the MCP server process can be spawned
- Check that the structurizr binary is in PATH or use absolute path
- Restart Claude after configuration changes

### Views Not Rendering
- Use `workspace_get_views` first to get valid view keys
- Ensure the workspace DSL is valid (use `workspace_validate`)
- Check that the view type is supported for the export format

## Related Documentation

- [Model Context Protocol Specification](https://modelcontextprotocol.io)
- [rmcp Rust SDK](https://github.com/modelcontextprotocol/rust-sdk)
- [structurizr-rs Documentation](../index.md)
- [Workspace Management](./workspace-management.md)
