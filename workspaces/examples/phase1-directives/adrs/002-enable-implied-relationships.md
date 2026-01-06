# ADR 002: Enable Implied Relationships

## Status
Accepted

## Context
When modeling complex systems with multiple containers and components, explicitly defining relationships at every level creates redundancy and maintenance burden.

For example, if `OrderService` (in `ECommerce` system) calls `PaymentAPI` (in `PaymentProvider` system), we shouldn't need to also define that `ECommerce -> PaymentProvider`.

## Decision
We will enable implied relationships using:

```dsl
!impliedRelationships true
```

This means:
- Container-to-container relationships imply system-to-system relationships
- Component-to-component relationships imply container-to-container relationships

## Consequences

### Positive
- Reduced redundancy in relationship definitions
- System-level diagrams automatically show connections
- Less maintenance when adding/removing integrations
- Cleaner, more focused DSL files

### Negative
- Less explicit control over system-level relationships
- Implied relationships may not have meaningful descriptions
- Need to be careful about transitive implications

## Recommendation
Use implied relationships for most projects, but consider disabling when:
- System-level relationships need specific descriptions
- Strict control over diagram content is required
