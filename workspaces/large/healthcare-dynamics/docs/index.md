# Healthcare Patient Journey

## Overview

This example demonstrates **complex multi-actor dynamic diagrams** for visualizing healthcare workflows with 7 different actor types and external system integrations.

## Business Context

**Metro General Hospital** is a 200-bed community hospital. They need to:

- Coordinate care across multiple departments
- Integrate with external Lab, Pharmacy, and Billing systems
- Maintain HIPAA-compliant patient records
- Support efficient clinical workflows

## Actors

| Actor | Role | Department |
|-------|------|------------|
| Patient | Individual receiving care | External |
| Receptionist | Check-in and scheduling | Administration |
| Nurse | Triage and patient care | Clinical |
| Doctor | Diagnosis and treatment | Clinical |
| Lab Technician | Specimen processing | Laboratory |
| Pharmacist | Medication dispensing | Pharmacy |
| Billing Clerk | Claims processing | Finance |

## Available Views

### System Context
Shows the hospital EHR and external systems (Lab, Pharmacy, Billing, Insurance).

### Container Diagram
Shows the internal architecture:
- **Patient Portal** - Self-service access
- **Clinical Workstation** - Provider interface
- **Admission Service** - Registration
- **Scheduling Service** - Appointments
- **Clinical Service** - Orders and documentation
- **Notification Service** - Alerts
- **Integration Hub** - HL7/FHIR routing
- **EHR Database** - Patient records

### Dynamic: PatientAdmission (12 steps)
Shows the complete admission workflow from pre-registration to triage.

### Dynamic: DiagnosisWorkflow (12 steps)
Shows the examination, lab ordering, and diagnosis workflow.

### Dynamic: DischargeProcess (11 steps)
Shows the discharge, prescribing, and billing workflow.

## Integration Architecture

The **Integration Hub** (Apache Camel) provides:
- HL7 v2 message transformation
- FHIR R4 resource handling
- Message routing and orchestration
- Error handling and retry logic

### Integration Patterns

| Flow | Protocol | Description |
|------|----------|-------------|
| Lab Orders | HL7 ORM | Order messages to LIS |
| Lab Results | HL7 ORU | Result messages from LIS |
| Prescriptions | HL7 RDE | Pharmacy orders |
| Billing | HL7 DFT | Charge posting |
| Claims | X12 837 | Insurance claims |

## Running This Example

```bash
cd workspaces/large/healthcare-dynamics
./serve.sh
```

## DSL Features Demonstrated

- Complex `dynamic` views with 10+ steps
- Multiple actor interactions in workflows
- External system integrations via Integration Hub
- `!const` for configuration and theming
- Multi-tag styling (e.g., `"Staff,Clinical"`)
- Various shapes: Person, WebBrowser, Hexagon, Pipe, Cylinder
