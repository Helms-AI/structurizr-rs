# ADR 001: FHIR R4 as Primary Interoperability Standard

## Status
Accepted

## Context
Healthcare systems must exchange data with external partners including:
- Health Information Exchanges (HIEs)
- Other healthcare providers
- Payers and insurance companies
- Public health agencies

We need a standard that provides:
- Semantic interoperability (shared data meaning)
- Wide industry adoption
- Modern API design principles
- Regulatory compliance (21st Century Cures Act)

## Decision
We will adopt **HL7 FHIR R4** as our primary interoperability standard for new integrations.

### Implementation
1. **Internal FHIR Server** - HAPI FHIR for clinical data access
2. **External FHIR Facade** - Public API for HIE connections
3. **US Core Profiles** - Compliance with US regulatory requirements

### Legacy Support
For systems that don't support FHIR, we maintain:
- HL7v2 Adapter for legacy lab/pharmacy integrations
- X12 EDI support for payer transactions
- NCPDP SCRIPT for pharmacy prescriptions

## Consequences

### Positive
- Future-proof architecture aligned with industry direction
- Regulatory compliance (CMS Interoperability Rule)
- Rich ecosystem of tools and libraries
- Modern REST/JSON API patterns

### Negative
- Requires FHIR expertise (training investment)
- Legacy system adapters add complexity
- Performance overhead for complex transformations

## References
- [HL7 FHIR R4 Specification](https://hl7.org/fhir/R4/)
- [US Core Implementation Guide](https://www.hl7.org/fhir/us/core/)
- [21st Century Cures Act](https://www.healthit.gov/curesrule/)
