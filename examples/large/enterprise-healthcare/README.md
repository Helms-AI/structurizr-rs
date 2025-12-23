# Enterprise Healthcare System

A comprehensive hospital information system architecture demonstrating a modern, HIPAA-compliant healthcare technology stack.

## Overview

This example models a large hospital's complete IT infrastructure, including:
- Electronic Health Records (EHR) as the central system
- Integrated ancillary systems (Lab, Pharmacy, Radiology)
- Patient engagement platforms
- Administrative and billing systems
- Compliance and security infrastructure

## Architecture Highlights

### Scale
- **8 user personas:** Doctor, Nurse, Patient, Admin, Lab Technician, Pharmacist, Radiologist, Billing Staff
- **12 software systems:** EHR, Lab, Pharmacy, Radiology, Billing, Portal, Emergency, Scheduling, Analytics, Integration Hub, Identity, Compliance
- **35+ containers:** APIs, databases, message brokers, caches, workers, frontends
- **Component detail:** Clinical Decision Support system fully decomposed

### Technology Stack
- **Frontend:** React, Vue.js
- **Backend:** Java Spring Boot, Node.js, Python
- **Databases:** PostgreSQL, MongoDB, Redis
- **Messaging:** RabbitMQ, Kafka
- **Cloud:** Azure (PaaS services)
- **Standards:** HL7 FHIR, DICOM

## System Groups

### Clinical Systems
Core healthcare delivery systems used directly in patient care:
- Electronic Health Record (EHR)
- Emergency Department System
- Lab Information System (LIS)
- Pharmacy Management
- Radiology/PACS

### Administrative Systems
Business and operational support:
- Appointment Scheduling
- Billing System
- Analytics Platform

### Infrastructure Systems
Cross-cutting technical services:
- Integration Hub (HL7 interface engine)
- Identity Management (SSO, RBAC)
- Compliance & Audit

### External Systems
Patient-facing and external interfaces:
- Patient Portal

## Key Workflows

### Patient Admission (Dynamic View)
1. Patient arrives at Emergency Department
2. Nurse creates encounter in EHR
3. Doctor orders lab tests
4. Lab processes samples
5. Results flow back to EHR
6. Doctor updates treatment plan

### Prescription Workflow (Dynamic View)
1. Doctor prescribes medication in EHR
2. Clinical Decision Support checks interactions
3. Pharmacy receives order
4. Pharmacist verifies and dispenses
5. Administration recorded back to EHR

## Deployment Architecture

### Hybrid Cloud Model
- **On-Premise Datacenter:** Core clinical systems for latency and compliance
- **Azure Cloud:** Analytics, patient portal, backup/DR

### On-Premise Components
- EHR Application Servers
- Clinical databases (HIPAA-compliant)
- Integration Hub
- Department systems (Lab, Pharmacy, Radiology)

### Azure Cloud Components
- Patient Portal (App Service)
- Analytics Platform (Synapse, Data Lake)
- Disaster Recovery
- Archive storage

## Security & Compliance

### HIPAA Requirements
- Encryption at rest and in transit
- Audit logging for all PHI access
- Role-based access control (RBAC)
- Multi-factor authentication
- Data retention policies

### Architecture Features
- Identity Management system for centralized auth
- Compliance system for audit trails
- Network segmentation
- API gateway for rate limiting and monitoring

## Perspectives

Elements are annotated with multiple perspectives:
- **Compliance:** HIPAA, FDA, state regulations
- **Criticality:** Mission-critical, high, medium, low
- **Data Classification:** PHI, PII, public

## Views Included

1. **System Landscape** - Complete hospital IT ecosystem
2. **System Context** - EHR and its external dependencies
3. **Container View - EHR** - Internal architecture of core EHR
4. **Container View - Lab System** - Lab information system details
5. **Container View - Pharmacy** - Medication management system
6. **Component View** - Clinical Decision Support internals
7. **Dynamic View - Admission** - Patient admission workflow
8. **Dynamic View - Prescription** - Medication ordering process
9. **Deployment View** - Hybrid on-prem/Azure infrastructure

## ADRs (Architecture Decision Records)

### ADR-001: Hybrid Cloud Strategy
**Decision:** Use hybrid deployment with critical systems on-premise

**Rationale:**
- Regulatory requirements for PHI data residency
- Low-latency requirements for clinical applications
- Cloud benefits for analytics and patient-facing apps

### ADR-002: Event-Driven Integration
**Decision:** Use message broker (RabbitMQ) for system integration

**Rationale:**
- Decouple systems for independent scaling
- Enable reliable async communication
- Support audit trail requirements
- Better resilience than synchronous APIs

### ADR-003: FHIR API Standard
**Decision:** Standardize on HL7 FHIR for all APIs

**Rationale:**
- Industry standard for healthcare interoperability
- Better than legacy HL7 v2 messages
- Supports RESTful APIs
- Enables patient data access requirements

## Documentation

### Clinical Workflow Documentation
Detailed clinical workflows are documented separately, covering:
- Patient registration and demographics
- Order entry and results reporting
- Clinical documentation and notes
- Medication administration record (MAR)

### Integration Patterns
The Integration Hub implements:
- HL7 v2 message routing for legacy systems
- FHIR API gateway for modern applications
- Event streaming for analytics
- Master Patient Index (MPI) management

## Running This Example

### Validate
```bash
cargo run -- validate examples/large/enterprise-healthcare/workspace.dsl
```

### Render All Views
```bash
cargo run -- render --workspace examples/large/enterprise-healthcare/workspace.dsl --output ./healthcare-diagrams
```

### Interactive Web View
```bash
cargo run -- serve --workspace examples/large/enterprise-healthcare/workspace.dsl --port 8080
```

## Customization Ideas

- Add more department systems (Cardiology, Oncology)
- Expand component views for other containers
- Add deployment views for different environments (Dev, Staging, Prod)
- Include external integrations (Health Information Exchanges)
- Model mobile applications for clinicians
- Add IoT medical devices

## Learning Objectives

This example demonstrates:
1. How to model complex, regulated domains
2. Organizing large architectures with groups and tags
3. Using perspectives for compliance metadata
4. Creating meaningful dynamic views for workflows
5. Hybrid deployment architectures
6. Event-driven integration patterns
7. Component-level detail where needed
8. Comprehensive styling for professional diagrams
