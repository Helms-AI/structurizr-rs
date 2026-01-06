# Phase 1 Directives Documentation

This example demonstrates the core DSL directives implemented in Phase 1 of structurizr-rs.

## Directives Demonstrated

### !const - Constants

Constants allow you to define reusable values throughout your workspace:

```dsl
!const COMPANY_NAME "Acme Corp"
!const PRIMARY_COLOR "#1168bd"
```

Use constants with `${CONSTANT_NAME}` syntax:

```dsl
person "Customer" "A customer of ${COMPANY_NAME}"
```

### !identifiers - Identifier Mode

Controls how element identifiers are generated:

```dsl
!identifiers hierarchical
```

Options:
- `flat` (default) - Simple identifiers
- `hierarchical` - Nested identifiers like `system.container.component`

### !impliedRelationships - Automatic Inference

When enabled, relationships between child elements imply relationships between parents:

```dsl
!impliedRelationships true
```

For example, if `Container A -> Container B`, this implies `System A -> System B`.

### !docs and !adrs - Documentation Links

Link to external documentation and Architecture Decision Records:

```dsl
!docs docs
!adrs adrs
```

## Architecture Overview

The E-Commerce Platform consists of:

1. **Web Application** - React-based SPA for customers and support agents
2. **API Gateway** - Routes requests to microservices
3. **Order Service** - Handles order lifecycle
4. **Inventory Service** - Manages product stock
5. **Database** - PostgreSQL for persistent storage
