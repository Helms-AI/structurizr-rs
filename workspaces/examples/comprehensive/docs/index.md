# HealthTech Platform - Comprehensive Example

This workspace demonstrates **all features** implemented in structurizr-rs, combining functionality from multiple implementation phases into a realistic healthcare platform architecture.

## Features Demonstrated

### Phase 1: Directives

| Directive | Usage |
|-----------|-------|
| `!const` | Define reusable constants for colors, technologies, organization name |
| `!identifiers` | Set to `hierarchical` for clear element naming |
| `!impliedRelationships` | Enabled to infer system-level relationships |
| `!docs` | Links to this documentation directory |
| `!adrs` | Links to Architecture Decision Records |

### Phase 2: Enterprise & Configuration

| Feature | Usage |
|---------|-------|
| `enterprise` | Defines "HealthTech Solutions" organizational boundary |
| `configuration` | Sets workspace scope, visibility, and custom terminology |
| Internal/External | Automatic tagging based on enterprise boundary |

### Phase 5: Native Scripting

| Script Type | Usage |
|-------------|-------|
| `!script lua` | Adds observability infrastructure dynamically |
| `!script groovy` | Adds compliance system with Groovy syntax (auto-transpiled) |

## Architecture Overview

### Internal Systems (HealthTech Solutions)

1. **EHR Platform** - Core electronic health records system
   - Patient Portal (self-service)
   - Clinician Application (clinical workflows)
   - FHIR Server (standards-compliant data access)
   - Microservices architecture (Patient, Encounter, Order, Document)

2. **Analytics Platform** - Healthcare intelligence
   - BI Dashboard (Tableau)
   - Data Warehouse (Snowflake)
   - ML Platform (predictive models)

3. **Integration Hub** - Interoperability engine
   - Interface Engine (Mirth Connect)
   - FHIR Facade (external API)
   - HL7v2 Adapter (legacy support)

4. **Observability Platform** (script-generated)
   - Metrics, logs, and traces

5. **Compliance Engine** (script-generated)
   - HIPAA audit and reporting

### External Systems

- Lab Information System (HL7v2)
- Pharmacy System (NCPDP SCRIPT)
- Health Information Exchange (FHIR R4)
- Payer System (X12 EDI)

### Actors

**Internal:**
- Physicians, Nurses, Admin Staff, Data Analysts, Operations Team

**External:**
- Patients, Insurance Companies

## Technology Stack

| Layer | Technology |
|-------|------------|
| Frontend | React, Angular, Tableau |
| Backend | Go, Java/Spring Boot, Python/FastAPI |
| Data | PostgreSQL, Snowflake, MinIO S3 |
| Integration | HAPI FHIR, Mirth Connect, Apache Kafka |
| Standards | HL7 FHIR R4, HL7v2, NCPDP SCRIPT, X12 EDI |

## Building This Example

### Without Scripting
```bash
cargo run -- validate workspaces/examples/comprehensive/workspace.dsl
```

### With Scripting (Full Features)
```bash
cargo build --features scripting
cargo run -- validate workspaces/examples/comprehensive/workspace.dsl
```

When scripting is enabled, the workspace will include:
- Modified workspace name: "HealthTech Platform (HIPAA Compliant)"
- Additional systems: Observability Platform, Compliance Engine
- Additional person: Operations Team
