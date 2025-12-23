# Perspectives Feature

## Overview

Perspectives allow you to filter architecture diagrams to show only the elements relevant to specific stakeholders. This enables different views of the same architecture for different audiences (e.g., business users, developers, security teams).

## Implementation

### Core Components

1. **`Perspective` struct** (`structurizr-core/src/workspace.rs`)
   - Defines a named perspective with an optional description
   - Stored in the workspace's `perspectives` field

2. **`perspectives` field in `ElementProperties`** (`structurizr-core/src/model.rs`)
   - Elements can specify which perspectives they belong to
   - Empty list means the element is visible in all perspectives

3. **Perspective filtering in handlers** (`structurizr-web/src/handlers.rs`)
   - Query parameter `?perspective=<name>` filters views by perspective
   - Only shows elements matching the requested perspective

## Usage

### Defining Perspectives

```rust
use structurizr_core::{Workspace, Perspective};

let mut workspace = Workspace::new("My System", "Description");

// Add perspectives to the workspace
workspace.add_perspective(
    Perspective::new("Business")
        .with_description("Shows business-relevant components")
);

workspace.add_perspective(
    Perspective::new("Technical")
        .with_description("Shows technical implementation details")
);
```

### Adding Perspectives to Elements

```rust
use structurizr_core::{SoftwareSystem, Container};

// Create a system visible in all perspectives
let mut system = SoftwareSystem::new("E-Commerce System")
    .with_description("Online shopping platform");

// Create a container visible only in Business and Technical perspectives
let mut web_app = Container::new("Web Application")
    .with_description("Customer-facing UI")
    .with_technology("React");

web_app.properties = web_app.properties
    .with_perspectives(vec!["Business", "Technical"]);

// Create a container visible only in Security perspective
let mut auth_service = Container::new("Auth Service")
    .with_description("Handles authentication and authorization")
    .with_technology("OAuth2");

auth_service.properties = auth_service.properties
    .with_perspective("Security");
```

### Filtering Views by Perspective

When running the web server, you can filter views using the `perspective` query parameter:

```bash
# Start the server
cargo run -- serve --port 8080

# View all elements (no filter)
curl http://localhost:8080/view/SystemLandscape/svg

# View only Business perspective elements
curl http://localhost:8080/view/SystemLandscape/svg?perspective=Business

# View only Technical perspective elements
curl http://localhost:8080/view/SystemLandscape/svg?perspective=Technical

# View only Security perspective elements
curl http://localhost:8080/view/SystemLandscape/svg?perspective=Security
```

## Filtering Rules

1. **Elements without perspectives**: Visible in all perspectives
2. **Elements with perspectives**: Only visible when viewing a matching perspective
3. **Relationships**: Automatically filtered to show only relationships between visible elements

## Example

See `examples/perspectives_example.rs` for a complete working example:

```bash
cargo run --example perspectives_example
```

This creates a workspace with three perspectives (Business, Technical, Security) and demonstrates how different elements are visible in each perspective.

## JSON Format

Perspectives are serialized in the workspace JSON:

```json
{
  "name": "My System",
  "model": {
    "people": [
      {
        "id": "...",
        "name": "Customer",
        "perspectives": []  // Empty = visible in all perspectives
      }
    ],
    "softwareSystems": [
      {
        "id": "...",
        "name": "Payment Service",
        "perspectives": ["Business", "Security"]  // Only visible in these perspectives
      }
    ]
  },
  "perspectives": [
    {
      "name": "Business",
      "description": "Shows business-relevant components"
    },
    {
      "name": "Technical",
      "description": "Shows technical implementation details"
    }
  ]
}
```

## Use Cases

### Business Stakeholders
Show high-level business processes and key systems without technical implementation details.

**Elements to include**:
- Customer-facing systems
- Business processes
- Data flows relevant to business operations

### Development Team
Show technical architecture, services, APIs, and infrastructure.

**Elements to include**:
- Microservices
- Databases
- Message queues
- API gateways
- Development tools

### Security Team
Show security-critical components, authentication flows, and data protection mechanisms.

**Elements to include**:
- Authentication services
- Encryption points
- Security boundaries
- Sensitive data stores
- Audit logging

### Operations Team
Show deployment infrastructure, monitoring, and operational concerns.

**Elements to include**:
- Deployment nodes
- Monitoring systems
- Load balancers
- Backup systems
- Operational tools

## Future Enhancements

Potential future improvements:

1. **Perspective inheritance**: Child elements inherit parent perspectives
2. **DSL syntax**: Add DSL support for defining perspectives
3. **View-level perspectives**: Associate perspectives directly with views
4. **Perspective combinations**: Support filtering by multiple perspectives simultaneously
5. **Perspective metadata**: Add additional metadata like color schemes per perspective
