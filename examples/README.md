# structurizr-rs Examples

A comprehensive suite of enterprise-level architecture examples demonstrating the Structurizr DSL capabilities across different complexity levels and industry domains.

## Quick Start

```bash
# Validate an example
cargo run -- validate examples/small/startup-saas/workspace.dsl

# Start web server with an example
cargo run -- serve --data-dir examples/medium/ecommerce-platform --port 8080

# Export to different formats
cargo run -- export --workspace examples/large/enterprise-healthcare/workspace.dsl --format plantuml
```

---

## Examples by Complexity

### [Small Examples](small/) (5-10 elements)

Perfect for learning the basics and quick demonstrations.

| Example | Domain | Elements | Views |
|---------|--------|----------|-------|
| [Startup SaaS](small/startup-saas/) | B2B Analytics Platform | ~8 | Context, Container, Dynamic |
| [Clinic Management](small/clinic-management/) | Healthcare Clinic | ~10 | Landscape, Context, Container |

---

### [Medium Examples](medium/) (15-30 elements)

Realistic architectures for mid-size applications.

| Example | Domain | Elements | Views |
|---------|--------|----------|-------|
| [E-commerce Platform](medium/ecommerce-platform/) | Online Retail | ~25 | All 6 types |
| [FinTech Payments](medium/fintech-payments/) | Payment Processing | ~20 | All 6 types |

---

### [Large Examples](large/) (50+ elements)

Enterprise-scale systems with complex interactions.

| Example | Domain | Elements | Views |
|---------|--------|----------|-------|
| [Enterprise Healthcare](large/enterprise-healthcare/) | Hospital System | ~60 | All types + hybrid deployment |
| [Smart Manufacturing](large/manufacturing-iot/) | Industry 4.0 Factory | ~55 | All types + edge/cloud deployment |

---

## DSL Features Demonstrated

All examples showcase the full Structurizr DSL feature set:

| Feature | Description |
|---------|-------------|
| `!const` | Reusable constants for company names, tech stacks |
| `!impliedRelationships` | Automatic transitive relationship generation |
| `!docs` | Documentation path references |
| `!adrs` | Architecture Decision Records path |
| **Tags** | Element categorization (External, Database, Queue, etc.) |
| **Groups** | Logical grouping by domain or team |
| **Perspectives** | Business, Technical, Security, Operations views |
| **Styles** | Custom colors, shapes, borders |
| **All View Types** | Landscape, Context, Container, Component, Dynamic, Deployment |

---

## Complexity Guide

| Level | Elements | Systems | Containers | Components | Views | DSL Lines |
|-------|----------|---------|------------|------------|-------|-----------|
| Small | 5-10 | 2-4 | 2-4 | 0 | 2-3 | ~100-150 |
| Medium | 15-30 | 4-8 | 8-15 | 2-5 | 4-6 | ~250-400 |
| Large | 50+ | 8-15 | 25-40 | 5-10 | 6-10 | ~600-1000 |

---

## Legacy Examples

Previous examples (feature demonstrations, tests) have been moved to [_legacy/](_legacy/).

---

## Running Examples

### Web Server

```bash
# Start with any example
cargo run -- serve --data-dir examples/medium/ecommerce-platform --port 8080

# Then open http://localhost:8080
```

### CLI Commands

```bash
# Validate DSL syntax
cargo run -- validate examples/small/startup-saas/workspace.dsl

# Render to SVG
cargo run -- render --workspace examples/medium/fintech-payments/workspace.dsl --output ./output

# Export formats
cargo run -- export --workspace examples/large/manufacturing-iot/workspace.dsl --format json
cargo run -- export --workspace examples/large/manufacturing-iot/workspace.dsl --format plantuml
cargo run -- export --workspace examples/large/manufacturing-iot/workspace.dsl --format mermaid
```

---

## Contributing

When adding new examples:

1. Choose appropriate complexity level (small/medium/large)
2. Create a new directory under the complexity folder
3. Include `workspace.dsl` and `README.md`
4. Use all DSL features (constants, tags, styles, perspectives)
5. Update this README with the new example
