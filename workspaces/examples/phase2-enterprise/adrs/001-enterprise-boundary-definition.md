# ADR 001: Enterprise Boundary Definition

## Status
Accepted

## Context
In a financial services context, it's critical to clearly distinguish between:
- Systems and people within our organization (Internal)
- External entities we interact with (External)

This distinction affects:
- Security considerations
- Compliance requirements
- Trust boundaries
- Integration patterns

## Decision
We will use the Structurizr DSL `enterprise` block to explicitly define our organizational boundary.

```dsl
enterprise "TechCorp Financial Services" {
    # All internal systems and people go here
}

# External entities defined outside
client = person "Client" "..." "External"
```

### What Goes Inside Enterprise
- All employees (traders, analysts, compliance officers)
- All internally developed systems
- All internally managed databases
- Internal APIs and services

### What Goes Outside Enterprise
- Clients and customers
- Regulatory bodies
- Third-party systems (exchanges, clearing houses)
- External data vendors

## Consequences

### Positive
- Clear visual distinction in diagrams
- Automatic tagging for styling
- Explicit documentation of trust boundaries
- Helps identify integration security requirements

### Negative
- Requires careful consideration of boundary placement
- Hybrid cloud scenarios may need additional thought
- SaaS products used internally need classification

## Notes
For systems that span the boundary (e.g., client-facing portals), place them inside the enterprise but document the external interface clearly.
