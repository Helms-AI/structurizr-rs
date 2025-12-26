# Healthcare Patient Journey

A comprehensive hospital EHR system demonstrating **complex multi-actor dynamic diagrams** with 7 actors and 3 distinct workflows.

## Overview

This example models a 200-bed community hospital with:
- Complete patient journey from admission to discharge
- Integration with Lab, Pharmacy, and Billing systems
- Multiple clinical and administrative workflows
- HL7/FHIR-based interoperability

## Dynamic Diagram Features

This example showcases **three dynamic diagrams** with complex multi-actor interactions:

### PatientAdmission (12 steps)
The patient registration and triage workflow:
1. Patient pre-registers online
2. Appointment requested
3. Patient record created
4. Check-in at front desk
5. Insurance verified
6. Eligibility confirmed
7. Arrival logged
8. Triage initiated
9. Vitals recorded
10. Documentation saved
11. Doctor alerted
12. Chart reviewed

### DiagnosisWorkflow (12 steps)
The examination and lab ordering workflow:
1. Chart opened
2. History loaded
3. Exam documented
4. Labs ordered
5. Order sent via HL7
6. Lab receives order
7. Specimen processed
8. Results returned
9. Results in EHR
10. Doctor notified
11. Diagnosis reviewed
12. Treatment prescribed

### DischargeProcess (11 steps)
The discharge and billing workflow:
1. Discharge signed
2. Prescriptions sent
3. Pharmacy receives
4. Medications prepared
5. Instructions provided
6. Billing triggered
7. Charges sent
8. Claim generated
9. Claim submitted
10. Copay collected
11. Summary sent

## Running the Example

```bash
./serve.sh
```

Navigate to any of the three dynamic views to explore the workflows.

## Architecture Highlights

- **7 Actor types** - Patient, Receptionist, Nurse, Doctor, Lab Tech, Pharmacist, Billing Clerk
- **Multi-system integration** - EHR, Lab, Pharmacy, Billing via HL7/FHIR
- **Integration Hub pattern** - Centralized message routing
- **HIPAA-aware design** - Audit logging and access controls

## DSL Features Demonstrated

- Complex dynamic views with 10+ steps
- Multiple external system integrations
- Constants for theming and configuration
- Multi-tag styling for role-based coloring
- Various container shapes (WebBrowser, Hexagon, Pipe, Cylinder)
