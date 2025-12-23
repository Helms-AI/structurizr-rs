# Perspectives Quick Start Guide

## What are Perspectives?

Perspectives allow you to create filtered views of your architecture for different stakeholders. The same architecture model can be viewed through different "lenses" to show only what's relevant to each audience.

## Basic Concepts

1. **Elements without perspectives** = visible in ALL perspectives
2. **Elements with perspectives** = visible only in those specific perspectives
3. **Relationships** = automatically filtered based on element visibility

## Quick Example

```rust
use structurizr_core::{Container, Perspective, SoftwareSystem, Workspace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create workspace
    let mut workspace = Workspace::new(
        "Banking System",
        "A modern banking platform"
    );

    // Define perspectives
    workspace.add_perspective(
        Perspective::new("Business")
            .with_description("For business stakeholders")
    );
    workspace.add_perspective(
        Perspective::new("Technical")
            .with_description("For developers")
    );

    // Create system
    let mut banking_system = SoftwareSystem::new("Internet Banking");

    // Add containers with perspectives
    let mut web_app = Container::new("Web Application");
    web_app.properties = web_app.properties
        .with_perspective("Business");  // Visible only in Business perspective

    let mut api = Container::new("API");
    api.properties = api.properties
        .with_perspectives(vec!["Business", "Technical"]);  // Visible in both

    let mut database = Container::new("Database");
    // No perspectives = visible in ALL perspectives

    // Add containers to system
    banking_system.add_container(web_app);
    banking_system.add_container(api);
    banking_system.add_container(database);

    // Add to workspace
    workspace.model_mut().software_systems.push(banking_system);

    // Save
    workspace.to_json_file("workspace.json")?;

    Ok(())
}
```

## Using Perspectives with the Web Server

### Start the server
```bash
cargo run -- serve --port 8080
```

### Access filtered views

**Business perspective:**
```
http://localhost:8080/view/SystemLandscape/svg?perspective=Business
```
Shows: Web Application, API, Database (all Business-relevant)

**Technical perspective:**
```
http://localhost:8080/view/SystemLandscape/svg?perspective=Technical
```
Shows: API, Database (technical components)

**All elements:**
```
http://localhost:8080/view/SystemLandscape/svg
```
Shows: Everything (no filter)

## Common Use Cases

### 1. Business vs Technical Views

```rust
// Business components
let mut ui = Container::new("User Interface");
ui.properties = ui.properties.with_perspective("Business");

let mut reports = Container::new("Reporting Engine");
reports.properties = reports.properties.with_perspective("Business");

// Technical components
let mut cache = Container::new("Redis Cache");
cache.properties = cache.properties.with_perspective("Technical");

let mut message_queue = Container::new("RabbitMQ");
message_queue.properties = message_queue.properties.with_perspective("Technical");

// Shared components (no perspective = visible in both)
let database = Container::new("PostgreSQL Database");
```

### 2. Security-Focused Views

```rust
workspace.add_perspective(
    Perspective::new("Security")
        .with_description("Security-critical components")
);

let mut auth_service = Container::new("Authentication Service");
auth_service.properties = auth_service.properties
    .with_perspectives(vec!["Security", "Technical"]);

let mut encryption_module = Container::new("Encryption Module");
encryption_module.properties = encryption_module.properties
    .with_perspective("Security");
```

### 3. Multi-Stakeholder Views

```rust
// Define all perspectives
workspace.add_perspective(Perspective::new("C-Level"));
workspace.add_perspective(Perspective::new("Product Manager"));
workspace.add_perspective(Perspective::new("Developer"));
workspace.add_perspective(Perspective::new("DevOps"));

// Core system (visible to all)
let core_system = SoftwareSystem::new("Core Platform");

// Monitoring dashboard (DevOps and Developers)
let mut monitoring = Container::new("Monitoring Dashboard");
monitoring.properties = monitoring.properties
    .with_perspectives(vec!["DevOps", "Developer"]);

// Analytics (C-Level and Product Managers)
let mut analytics = Container::new("Business Analytics");
analytics.properties = analytics.properties
    .with_perspectives(vec!["C-Level", "Product Manager"]);
```

## Best Practices

### 1. Use Descriptive Perspective Names
```rust
// Good
Perspective::new("Security Team")
Perspective::new("External Auditors")
Perspective::new("Operations")

// Avoid
Perspective::new("View1")
Perspective::new("A")
```

### 2. Document Perspectives
```rust
workspace.add_perspective(
    Perspective::new("Compliance")
        .with_description("Shows systems involved in regulatory compliance")
);
```

### 3. Keep Core Elements Visible
Elements critical to understanding the system should have no perspectives (visible in all views):
- Users/Actors
- Core databases
- Primary business systems

### 4. Use Perspectives for Filtering, Not Hiding
Perspectives should simplify views, not hide important information. If an element is security-critical, it should still appear in technical views.

## Testing Your Perspectives

### 1. Build the example
```bash
cargo build --example perspectives_example
```

### 2. Run the example
```bash
cargo run --example perspectives_example
```

### 3. View the output
The example creates `perspectives_example.json` demonstrating a complete e-commerce system with Business, Technical, and Security perspectives.

## Troubleshooting

### Elements not showing up
- Check if the element has the perspective you're filtering by
- Remember: empty perspectives = visible in all views
- Verify relationships: orphaned relationships are removed

### All elements showing
- You're probably viewing without a perspective filter
- Add `?perspective=YourPerspectiveName` to the URL

### Unexpected filtering
- Check parent/child relationships
- Verify perspective names match exactly (case-sensitive)
- Ensure perspective is defined in workspace

## Next Steps

1. Read the full documentation: `PERSPECTIVES.md`
2. Check the implementation details: `PERSPECTIVES_IMPLEMENTATION.md`
3. Run the example: `cargo run --example perspectives_example`
4. Try creating your own workspace with perspectives

## Summary

Perspectives provide a powerful way to create stakeholder-specific views of your architecture:

- **Simple API**: Just add perspective names to elements
- **Flexible**: Elements can have multiple perspectives
- **Automatic**: Relationships are filtered automatically
- **Web-enabled**: Filter via HTTP query parameters
- **Backward compatible**: Existing workspaces work unchanged

Start simple with 2-3 perspectives and expand as needed!
