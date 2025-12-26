# Medium Enterprise Examples

This directory contains medium-scale enterprise architecture examples demonstrating comprehensive use of the Structurizr DSL.

## Examples

### 1. E-commerce Platform (`ecommerce-platform/`)
A complete online retail platform with approximately 25 elements, showcasing:
- Multi-tier web and mobile architecture
- Microservices design with API Gateway
- Integration with external payment, shipping, and analytics services
- Search and caching infrastructure
- AWS deployment topology

**Domain:** Online Retail
**Complexity:** ~25 elements (4 people, 6 systems, 12 containers, 3 components)
**Technologies:** React, React Native, Java/Spring Boot, PostgreSQL, Redis, Elasticsearch

### 2. FinTech Payment Platform (`fintech-payments/`)
A payment processing platform with approximately 20 elements, demonstrating:
- High-compliance financial architecture
- Real-time transaction processing
- Audit and compliance requirements
- Multi-region deployment for high availability
- Integration with banking networks and KYC providers

**Domain:** Financial Services / Payments
**Complexity:** ~20 elements (3 people, 5 systems, 10 containers, 2 components)
**Technologies:** Angular, Swift, Go, PostgreSQL, Redis, Kafka

## DSL Features Demonstrated

All examples include comprehensive use of:
- **Constants** (`!const`) - For reusable values
- **Implied Relationships** (`!impliedRelationships`) - Automatic relationship inference
- **Documentation** (`!docs`) - Architecture Decision Records and documentation
- **ADRs** (`!adrs`) - Architecture Decision Records
- **Tags** - For categorization and styling
- **Groups** - Logical grouping of elements
- **Perspectives** - Multiple viewpoints (Security, Performance, Cost)
- **Custom Styles** - Comprehensive styling with colors, shapes, and borders
- **All View Types:**
  - System Landscape View
  - System Context View
  - Container View
  - Component View
  - Dynamic View
  - Deployment View

## How to Use

Each example contains:
- `workspace.dsl` - Complete Structurizr DSL definition
- `README.md` - Architecture overview and details
- Referenced documentation (inline in DSL)

To validate an example:
```bash
cargo run -- validate workspaces/medium/ecommerce-platform/workspace.dsl
```

To render an example:
```bash
cargo run -- render --workspace workspaces/medium/ecommerce-platform/workspace.dsl --output ./output
```

To serve an example:
```bash
cargo run -- serve --workspace workspaces/medium/ecommerce-platform/workspace.dsl --port 8080
```

## Learning Path

1. Start with the **e-commerce platform** to understand common web architecture patterns
2. Move to the **fintech payments** example to see compliance-heavy, high-reliability systems
3. Compare the deployment strategies between cloud-native (AWS) and multi-region deployments
4. Study the dynamic views to understand key user flows
5. Examine the use of perspectives for cross-cutting concerns

## Comparison

| Aspect | E-commerce | FinTech Payments |
|--------|-----------|------------------|
| Complexity | Higher (25 elements) | Medium (20 elements) |
| Focus | Customer experience, scalability | Compliance, reliability |
| External Integrations | 4 systems | 3 systems |
| Deployment | Single-region AWS | Multi-region (US/EU) |
| Key Challenges | Search, inventory, performance | Audit, security, regulations |
| Data Stores | PostgreSQL, Redis, Elasticsearch | PostgreSQL, Redis, Kafka |
| Dynamic Views | Checkout flow | Payment authorization |

## Next Steps

After studying these medium examples:
- Examine the simple examples in `/workspaces/simple/` for basic concepts
- Review the complex examples in `/workspaces/complex/` for large-scale systems
- Create your own architecture using these as templates
