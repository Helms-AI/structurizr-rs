# ADR-015: Payment Processing Removal

## Status

**Accepted**

## Date

2024-12-31

## Context

The original Horizon Platform (formerly Replit Clone) architecture included Stripe as the payment processing system for:
- Subscription management
- Usage-based billing
- Payment processing

### Current Situation

The platform is being refactored to focus on core IDE and development functionality. Payment processing adds significant complexity:

1. **PCI DSS Compliance**: Handling payment data requires strict security controls
2. **Integration Complexity**: Webhook handling, subscription states, retry logic
3. **Operational Overhead**: Payment disputes, refunds, reconciliation
4. **Scope Creep**: Focus should be on developer experience, not billing

## Decision

We will **remove the payment processing subsystem** from the Horizon Platform architecture.

### Rationale

1. **Focus on Core Value**: The platform's value is in the IDE, collaboration, and AI features
2. **Simplified Architecture**: Fewer external dependencies and failure modes
3. **Reduced Compliance Burden**: No PCI DSS requirements
4. **Open Source Alignment**: Makes the platform more suitable for self-hosting
5. **Future Flexibility**: Payments can be re-added when needed with fresh design

### What Gets Removed

| Component | Status |
|-----------|--------|
| Stripe Software System | Removed |
| Stripe relationships | Removed |
| Billing service | Not implemented |
| Subscription management | Not implemented |
| Usage metering (for billing) | Not implemented |

### What Remains

| Component | Status |
|-----------|--------|
| User authentication | Keycloak |
| Workspace management | Full functionality |
| Usage tracking (analytics) | Retained for insights |
| Rate limiting | Retained for fair use |

## Alternatives Considered

1. **Keep Stripe**: High complexity for uncertain benefit at this stage
2. **Replace with Lake**: Open-source Stripe alternative, but still adds complexity
3. **Implement custom billing**: Even higher complexity and maintenance burden
4. **Usage-based model only**: Still requires payment infrastructure

## Implementation

### Changes to workspace.dsl

```diff
- group "Business Services" {
-     stripe = softwareSystem "Stripe" "Payment processing" {
-         tags "External,Partner"
-         properties {
-             "integration" "REST API, Webhooks"
-             "features" "Subscriptions, usage billing"
-         }
-     }
- }

- horizon -> stripe "Processes payments via" {
-     tags "External"
- }
```

### User Access Model

Without payments, access control will be based on:

1. **Authentication**: Users authenticate via Keycloak
2. **Authorization**: Role-based access control (RBAC)
3. **Quotas**: Resource limits per user/workspace (soft limits)
4. **Rate Limiting**: Fair use policies enforced at API gateway

```python
# Example quota configuration
USER_QUOTAS = {
    "free": {
        "workspaces": 5,
        "storage_gb": 1,
        "ai_completions_per_day": 100,
        "collaborators_per_workspace": 3
    },
    "team": {
        "workspaces": 50,
        "storage_gb": 10,
        "ai_completions_per_day": 1000,
        "collaborators_per_workspace": 20
    }
}
```

## Consequences

### Positive

1. **Simplified architecture**: Fewer moving parts
2. **Reduced maintenance**: No payment infrastructure to manage
3. **Focus on core features**: Team can focus on IDE experience
4. **Easier self-hosting**: No payment dependencies for users
5. **No compliance overhead**: No PCI DSS requirements

### Negative

1. **No monetization path**: Need separate solution for revenue
2. **Limited access control**: Can't enforce paid tiers
3. **Re-implementation cost**: Will need to add payments later if needed

### Mitigations

| Concern | Mitigation |
|---------|------------|
| No monetization | Can use external billing (e.g., Stripe Billing Portal) separately |
| Access control | Use RBAC and quotas for now |
| Future payments | Design user/workspace models to be payment-ready |

## Future Considerations

If payment processing is needed in the future:

1. **Stripe Billing Portal**: External billing, minimal integration
2. **LemonSqueezy**: Developer-friendly, handles tax compliance
3. **Paddle**: SaaS-focused, handles global taxes
4. **Self-hosted**: Lake or custom implementation

The architecture should remain payment-agnostic to allow any of these options.

## References

- [Stripe Documentation](https://stripe.com/docs)
- [PCI DSS Compliance](https://www.pcisecuritystandards.org/)
- [Lake - Open Source Billing](https://github.com/getlake/lake)
