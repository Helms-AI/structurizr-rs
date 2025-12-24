# ADR-004: Multi-System Orchestration

## Status

Accepted

## Context

The hospital relies on multiple specialized systems:
- EHR for patient records
- LIS for laboratory management
- Pharmacy system for medications
- Billing system for claims

These systems must work together seamlessly.

## Decision

We will use an **Integration Hub pattern** rather than point-to-point:

1. **Centralized routing** - All messages flow through hub
2. **Protocol normalization** - Translate between system formats
3. **Orchestration** - Coordinate multi-step workflows
4. **Monitoring** - Single view of all integrations

## Consequences

### Positive

- **Loose coupling** - Systems can change independently
- **Visibility** - Central monitoring of all flows
- **Scalability** - Add new systems easily
- **Reliability** - Retry and error handling in one place

### Negative

- **Single point of failure** - Hub availability critical
- **Latency** - Extra hop for all messages
- **Complexity** - Hub requires expertise

## Alternatives Considered

- **Point-to-point** - Simpler but creates spaghetti architecture
- **Service Mesh** - Overkill for healthcare messaging patterns
- **ESB** - Similar but heavier than Apache Camel

## Notes

The DischargeProcess dynamic diagram shows multi-system coordination via the hub.
