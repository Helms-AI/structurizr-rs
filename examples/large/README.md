# Large Enterprise Examples

This directory contains comprehensive enterprise-scale examples demonstrating all features of structurizr-rs.

## Examples

### 1. Enterprise Healthcare (`enterprise-healthcare/`)

A complete Hospital Information System architecture showcasing:
- **Scale:** 60+ elements across 8 user types, 12 systems, 35+ containers
- **Domain:** Healthcare with EHR, Lab, Pharmacy, Radiology, and supporting systems
- **Architecture:** Microservices with event-driven integration
- **Deployment:** Hybrid on-premise + Azure cloud
- **Views:** All view types including dynamic workflows and deployment diagrams

### 2. Manufacturing IoT (`manufacturing-iot/`)

An Industry 4.0 Smart Factory platform featuring:
- **Scale:** 55+ elements across 6 user types, 10 systems, 30+ containers
- **Domain:** Industrial IoT with MES, SCADA, predictive maintenance
- **Architecture:** Edge computing, plant servers, cloud analytics
- **Deployment:** 3-tier architecture (Edge, Plant, Cloud on AWS)
- **Views:** Complete coverage including IoT data flows and ML pipelines

## DSL Features Demonstrated

Both examples include:

### Advanced DSL Features
- `!const` - Constants for reusable values
- `!impliedRelationships` - Automatic relationship inference
- `!docs` - Documentation embedding
- `!adrs` - Architecture Decision Records

### Model Features
- Multiple people with different roles
- Complex system hierarchies
- Containers with various technologies
- Components with detailed internals
- Groups for organizing elements
- Tags for categorization
- Perspectives for multi-dimensional metadata

### View Features
- System Landscape views
- System Context views
- Multiple Container views
- Component views with details
- Dynamic views showing workflows
- Deployment views with infrastructure
- Custom styles with shapes, colors, and icons

### Styling
- Element styles by tag
- Relationship styles with routing
- Shape variations (Person, Box, Cylinder, etc.)
- Color schemes for visual hierarchy
- Custom properties and metadata

## How to Use

### Validate DSL
```bash
cargo run -- validate examples/large/enterprise-healthcare/workspace.dsl
cargo run -- validate examples/large/manufacturing-iot/workspace.dsl
```

### Render Diagrams
```bash
cargo run -- render --workspace examples/large/enterprise-healthcare/workspace.dsl --output ./output/healthcare
cargo run -- render --workspace examples/large/manufacturing-iot/workspace.dsl --output ./output/manufacturing
```

### Export to Other Formats
```bash
# JSON export
cargo run -- export --workspace examples/large/enterprise-healthcare/workspace.dsl --format json > healthcare.json

# PlantUML export
cargo run -- export --workspace examples/large/manufacturing-iot/workspace.dsl --format plantuml > manufacturing.puml
```

### Start Web Server
```bash
cargo run -- serve --workspace examples/large/enterprise-healthcare/workspace.dsl --port 8080
```

## Learning Path

1. **Start with README files** - Each example has detailed documentation
2. **Explore the workspace.dsl** - Read through the complete architecture
3. **Generate diagrams** - Use the render command to visualize
4. **Modify and experiment** - Add your own elements and views
5. **Study the styles** - Learn how visual presentation is customized

## Best Practices Illustrated

### Organization
- Logical grouping of related elements
- Consistent naming conventions
- Hierarchical structure for complex systems

### Documentation
- Inline descriptions for all elements
- Separate documentation files
- ADRs for architectural decisions

### Visualization
- Multiple views for different stakeholders
- Appropriate view types for each use case
- Dynamic views to show workflows
- Deployment views for infrastructure understanding

### Reusability
- Constants for repeated values
- Tags for common categorization
- Groups for organizational structure
- Implied relationships to reduce repetition

## Architecture Patterns

### Healthcare Example
- HIPAA-compliant architecture
- Event-driven integration
- Clinical workflow automation
- Multi-tenancy support
- Audit and compliance logging

### Manufacturing Example
- Edge-to-cloud data pipeline
- Real-time monitoring and control
- Predictive analytics with ML
- OT/IT convergence
- Time-series data management

## Additional Resources

- [C4 Model Documentation](https://c4model.com/)
- [Structurizr DSL Reference](https://github.com/structurizr/dsl)
- Project documentation in `/docs`

## Contributing

These examples are maintained to demonstrate best practices. If you find improvements or want to add new patterns, please contribute!
