# ADR-003: Transaction Audit Trail

## Status

Accepted

## Context

Financial regulations require:
- Complete records of all transactions
- Immutable audit logs for investigations
- Tamper-proof evidence for disputes

## Decision

We will maintain an **immutable audit log** in PostgreSQL:

1. All transactions logged before completion
2. Append-only table with no UPDATE/DELETE permissions
3. Includes: timestamp, user, action, amounts, status
4. Retained for 7 years per regulatory requirements

## Consequences

### Positive

- **Compliance** - Meets SOX, PCI-DSS requirements
- **Dispute resolution** - Complete transaction history
- **Forensics** - Investigation support for fraud cases
- **Reproducibility** - Can recreate transaction state

### Negative

- **Storage growth** - Logs accumulate over time
- **Performance** - Additional write for each transaction
- **Complexity** - Special handling for log rotation

## Notes

The TransferFlow dynamic diagram shows audit logging at step 9.
