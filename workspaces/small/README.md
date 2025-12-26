# Small Enterprise Examples

This directory contains small-scale enterprise architecture examples for structurizr-rs. Each example demonstrates all DSL features in a realistic business context.

## Examples

### 1. startup-saas/
**Domain:** B2B SaaS Analytics Platform
**Scale:** ~8 elements
**Complexity:** Startup/SMB
**Focus:** Modern cloud-native SaaS architecture with external integrations

**Key Features:**
- System Context and Container views
- Dynamic view showing user authentication flow
- External system integrations (Auth0, Snowflake)
- Modern tech stack (React, Node.js, PostgreSQL)

### 2. clinic-management/
**Domain:** Small Medical Clinic Management System
**Scale:** ~10 elements
**Complexity:** Healthcare SMB
**Focus:** Healthcare workflow automation with regulatory compliance

**Key Features:**
- System Landscape showing complete healthcare ecosystem
- Multiple external system integrations (Insurance, Labs, Pharmacy)
- Healthcare-specific workflows and perspectives
- HIPAA compliance considerations

## DSL Features Demonstrated

All examples showcase the full range of Structurizr DSL capabilities:

- **Constants** (`!const`) - Reusable configuration values
- **Implied Relationships** (`!impliedRelationships`) - Automatic relationship inference
- **Documentation** (`!docs`) - Architecture Decision Records and documentation
- **ADRs** (`!adrs`) - Structured decision tracking
- **Tags** - Element categorization and filtering
- **Groups** - Logical grouping of elements
- **Perspectives** - Multiple viewpoints (security, compliance, cost)
- **Custom Styles** - Brand colors, shapes, and visual customization
- **Dynamic Views** - Sequence diagrams showing runtime behavior

## How to Use

Each example is self-contained with:
- `workspace.dsl` - Complete architecture definition
- `README.md` - Domain context and architecture overview
- Embedded documentation and ADRs

To visualize an example:

```bash
# Validate the DSL
cargo run -- validate workspaces/small/startup-saas/workspace.dsl

# Start the web server
cargo run -- serve --workspace workspaces/small/startup-saas/workspace.dsl

# Export to various formats
cargo run -- export --workspace workspaces/small/startup-saas/workspace.dsl --format json
cargo run -- export --workspace workspaces/small/startup-saas/workspace.dsl --format plantuml
```

## Learning Path

1. Start with **startup-saas/** for modern cloud-native patterns
2. Explore **clinic-management/** for complex multi-stakeholder systems
3. Study the DSL features used in each example
4. Adapt these patterns to your own architecture

## Next Steps

- Explore medium examples in `/workspaces/medium/` for more complex systems
- Check large examples in `/workspaces/large/` for enterprise-scale architectures
- Review the DSL documentation for advanced features
