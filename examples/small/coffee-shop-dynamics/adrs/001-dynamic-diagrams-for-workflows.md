# ADR-001: Using Dynamic Diagrams for Workflow Visualization

## Status

Accepted

## Context

We need to document and communicate the order placement workflow to:
- New baristas learning the system
- Developers maintaining the software
- Stakeholders understanding the customer experience

Traditional static diagrams show relationships but don't convey the sequence of operations.

## Decision

We will use **Structurizr dynamic diagrams** to visualize the order placement workflow.

Dynamic diagrams show:
- The sequence of interactions between system components
- Numbered steps that can be animated
- The same C4 elements as static diagrams, but with temporal ordering

## Consequences

### Positive

- **Clear sequencing** - Steps are numbered and can be animated
- **Reuses C4 elements** - No need to maintain separate sequence diagrams
- **Interactive** - Users can step through the workflow
- **Single source of truth** - Workflow documented in the same DSL as architecture

### Negative

- **Limited complexity** - Not suitable for highly branching workflows
- **Linear focus** - Best for happy-path scenarios rather than error handling
- **Learning curve** - Team needs to understand dynamic view syntax

## Notes

The dynamic view syntax is:

```dsl
dynamic system "Key" "Description" {
    source -> destination "Step description"
    ...
}
```

Each step is rendered in order, creating an animated walkthrough of the workflow.
