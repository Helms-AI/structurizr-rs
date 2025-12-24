# ADR-002: HL7/FHIR Integration Strategy

## Status

Accepted

## Context

Healthcare interoperability requires standardized messaging:
- Legacy systems use HL7 v2 messages
- Modern systems prefer FHIR resources
- Real-time and batch exchange needed
- Error handling and retry logic required

## Decision

We will implement an **Integration Hub** using Apache Camel for:

1. **HL7 v2 Support** - ORM, ORU, RDE, DFT messages
2. **FHIR R4 Support** - Patient, Observation, MedicationRequest
3. **Protocol Translation** - Convert between HL7 and FHIR
4. **Message Routing** - Route to appropriate destinations
5. **Error Handling** - Dead letter queues, retry policies

## Consequences

### Positive

- **Standard compliance** - Industry-standard protocols
- **Flexibility** - Support for legacy and modern systems
- **Centralized routing** - Single point for message flow
- **Monitoring** - Track all integration activity

### Negative

- **Complexity** - Multiple protocol support
- **Performance** - Translation overhead
- **Maintenance** - Schema updates required

## Notes

The DiagnosisWorkflow dynamic diagram shows HL7 message flow for lab orders.
