# ADR 001: Use Constants for Consistency

## Status
Accepted

## Context
We need a way to ensure consistency across the architecture documentation, particularly for:
- Company name references
- Color schemes in diagrams
- Technology stack references

Hardcoding these values throughout the DSL file leads to maintenance issues and inconsistencies.

## Decision
We will use the `!const` directive to define reusable constants at the top of our workspace files.

### Constants to Define
- `COMPANY_NAME` - The company name for use in descriptions
- `PRIMARY_COLOR` - Primary brand color for styling
- `SECONDARY_COLOR` - Secondary brand color
- `DATABASE_TECH` - Database technology with version
- `API_TECH` - API technology standards

## Consequences

### Positive
- Single source of truth for common values
- Easy to update company-wide changes
- Consistent styling across all diagrams
- Technology versions tracked in one place

### Negative
- Slight learning curve for new team members
- Constants must be defined before use

## Example
```dsl
!const COMPANY_NAME "Acme Corp"
!const PRIMARY_COLOR "#1168bd"

person "Customer" "A customer of ${COMPANY_NAME}"
```
