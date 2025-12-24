# ADR-003: Clinical Workflow Automation

## Status

Accepted

## Context

Clinical workflows involve many steps and handoffs:
- Patients move through admission, triage, exam, discharge
- Orders flow to Lab, Pharmacy, and back
- Notifications keep staff informed of status changes

Manual coordination is error-prone and slow.

## Decision

We will implement **automated workflow orchestration**:

1. **Event-driven notifications** - Real-time alerts for state changes
2. **Order routing** - Automatic dispatch to appropriate systems
3. **Status tracking** - Workflow state visible to all actors
4. **Escalation rules** - Auto-escalate delayed steps

## Consequences

### Positive

- **Efficiency** - Reduced manual coordination
- **Visibility** - Everyone sees workflow status
- **Timeliness** - Faster response to patient needs
- **Consistency** - Standard process followed

### Negative

- **Rigidity** - May not fit all clinical situations
- **Complexity** - Workflow rules require maintenance
- **Dependencies** - System failures impact workflow

## Notes

All three dynamic diagrams show automated workflow sequences.
