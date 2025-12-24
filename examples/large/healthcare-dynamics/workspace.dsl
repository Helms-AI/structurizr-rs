workspace "Healthcare Patient Journey" "Hospital patient management with multi-actor workflows" {

    !docs "docs"
    !adrs "adrs"

    !const HOSPITAL_NAME "Metro General Hospital"
    !const EHR_COLOR "#2c5282"
    !const LAB_COLOR "#38a169"
    !const PHARMACY_COLOR "#805ad5"
    !const BILLING_COLOR "#dd6b20"

    model {
        // Healthcare Staff Actors
        patient = person "Patient" "Individual receiving medical care" {
            tags "External"
        }
        receptionist = person "Receptionist" "Front desk scheduling and check-in" {
            tags "Staff,Admin"
        }
        nurse = person "Nurse" "Triage, vitals, and patient care" {
            tags "Staff,Clinical"
        }
        doctor = person "Doctor" "Diagnosis, treatment, and orders" {
            tags "Staff,Clinical"
        }
        labTechnician = person "Lab Technician" "Laboratory test processing" {
            tags "Staff,Lab"
        }
        pharmacist = person "Pharmacist" "Medication dispensing and counseling" {
            tags "Staff,Pharmacy"
        }
        billingClerk = person "Billing Clerk" "Insurance claims and payments" {
            tags "Staff,Admin"
        }

        // External Systems
        labSystem = softwareSystem "Laboratory Information System" "Lab test ordering, processing, and results" {
            tags "External,Lab"
        }
        pharmacySystem = softwareSystem "Pharmacy System" "Medication dispensing and tracking" {
            tags "External,Pharmacy"
        }
        billingSystem = softwareSystem "Billing System" "Claims processing and payment collection" {
            tags "External,Billing"
        }
        insurancePortal = softwareSystem "Insurance Portal" "Payer eligibility and claims submission" {
            tags "External"
        }

        // Main Hospital System
        hospitalSystem = softwareSystem "${HOSPITAL_NAME} EHR" "Electronic Health Record and hospital management" {
            patientPortal = container "Patient Portal" "Patient self-service and health records access" "React" {
                tags "Frontend"
            }
            clinicalWorkstation = container "Clinical Workstation" "Provider interface for patient care" "React" {
                tags "Frontend"
            }
            admissionService = container "Admission Service" "Patient registration and check-in" "Java/Spring Boot" {
                tags "API"
            }
            schedulingService = container "Scheduling Service" "Appointment and resource management" "Java/Spring Boot" {
                tags "API"
            }
            clinicalService = container "Clinical Service" "Orders, documentation, and care plans" "Rust/Actix" {
                tags "API"
            }
            notificationService = container "Notification Service" "Alerts, reminders, and communications" "Node.js" {
                tags "API"
            }
            integrationHub = container "Integration Hub" "HL7/FHIR message routing" "Apache Camel" {
                tags "Integration"
            }
            ehrDatabase = container "EHR Database" "Patient records and clinical data" "PostgreSQL" {
                tags "Database"
            }
        }

        // Patient interactions
        patient -> patientPortal "Accesses health records via" "HTTPS"

        // Staff to system interactions (through hospital containers, not direct to external)
        receptionist -> admissionService "Manages patient check-in with"
        receptionist -> schedulingService "Books appointments via"
        nurse -> clinicalWorkstation "Documents care in"
        doctor -> clinicalWorkstation "Reviews charts and enters orders in"
        labTechnician -> integrationHub "Views lab orders via"
        pharmacist -> integrationHub "Views prescriptions via"
        billingClerk -> integrationHub "Views billing data via"

        // Portal to services
        patientPortal -> schedulingService "Requests appointments from"
        clinicalWorkstation -> clinicalService "Accesses clinical data via"

        // Service to service
        admissionService -> ehrDatabase "Stores patient data in"
        admissionService -> integrationHub "Checks insurance via"
        schedulingService -> ehrDatabase "Stores appointments in"
        clinicalService -> ehrDatabase "Reads/writes clinical records in"
        clinicalService -> integrationHub "Sends orders via"
        // Note: notificationService sends alerts to people (shown in dynamic views)

        // Integration hub connections (unidirectional for clean layout)
        integrationHub -> labSystem "Routes lab orders to" "HL7/FHIR"
        integrationHub -> pharmacySystem "Sends prescriptions to" "HL7/FHIR"
        integrationHub -> billingSystem "Sends charges to" "HL7"
        integrationHub -> insurancePortal "Checks eligibility via" "REST"
        // Note: Bidirectional HL7/FHIR message flow shown in dynamic views
    }

    views {
        systemContext hospitalSystem "SystemContext" "Hospital EHR system context" {
            include *
            autoLayout tb
        }

        container hospitalSystem "Containers" "Hospital EHR container architecture" {
            include *
            autoLayout tb
        }

        // Dynamic view 1: Patient Admission Flow
        dynamic hospitalSystem "PatientAdmission" "New patient registration and triage" {
            patient -> patientPortal "Completes pre-registration online"
            patientPortal -> schedulingService "Requests appointment slot"
            schedulingService -> ehrDatabase "Creates patient record"
            patient -> receptionist "Arrives and checks in at front desk"
            receptionist -> admissionService "Verifies insurance and demographics"
            admissionService -> integrationHub "Checks insurance eligibility"
            receptionist -> schedulingService "Confirms arrival in system"
            nurse -> clinicalWorkstation "Calls patient for triage"
            nurse -> clinicalService "Records vitals and chief complaint"
            clinicalService -> ehrDatabase "Saves triage documentation"
            notificationService -> doctor "Alerts doctor patient is ready"
            doctor -> clinicalWorkstation "Reviews triage notes before exam"
            autoLayout lr
        }

        // Dynamic view 2: Diagnosis and Treatment Flow
        dynamic hospitalSystem "DiagnosisWorkflow" "Doctor examination and lab ordering" {
            doctor -> clinicalWorkstation "Opens patient chart"
            clinicalService -> ehrDatabase "Loads patient history"
            doctor -> clinicalService "Documents examination findings"
            doctor -> clinicalService "Orders laboratory tests"
            clinicalService -> integrationHub "Sends lab order via HL7"
            integrationHub -> labSystem "Lab receives order"
            labTechnician -> labSystem "Collects and processes specimen"
            labSystem -> integrationHub "Sends results via HL7"
            integrationHub -> clinicalService "Results arrive in EHR"
            notificationService -> doctor "Alerts doctor results ready"
            doctor -> clinicalService "Reviews results and diagnosis"
            doctor -> clinicalService "Prescribes medication treatment"
            autoLayout lr
        }

        // Dynamic view 3: Discharge and Billing Flow
        dynamic hospitalSystem "DischargeProcess" "Patient discharge and billing" {
            doctor -> clinicalService "Signs discharge orders"
            clinicalService -> integrationHub "Sends prescriptions electronically"
            integrationHub -> pharmacySystem "Pharmacy receives prescriptions"
            pharmacist -> pharmacySystem "Prepares medications"
            nurse -> clinicalWorkstation "Provides discharge instructions"
            clinicalService -> integrationHub "Triggers billing charges"
            integrationHub -> billingSystem "Billing receives charge data"
            billingClerk -> billingSystem "Generates insurance claim"
            billingSystem -> integrationHub "Submits claim to payer"
            receptionist -> admissionService "Collects patient copay"
            notificationService -> patient "Sends visit summary and follow-up"
            autoLayout lr
        }

        styles {
            element "Person" {
                shape Person
                background "#2b6cb0"
                color "#ffffff"
            }
            element "External" {
                background "#718096"
            }
            element "Staff" {
                background "#2c5282"
            }
            element "Clinical" {
                background "#e53e3e"
            }
            element "Admin" {
                background "#805ad5"
            }
            element "Lab" {
                background "#38a169"
            }
            element "Pharmacy" {
                background "#d69e2e"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "Billing" {
                background "#dd6b20"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Frontend" {
                shape WebBrowser
            }
            element "API" {
                shape Hexagon
            }
            element "Integration" {
                shape Pipe
                background "#ed8936"
            }
            element "Database" {
                shape Cylinder
            }
            relationship "Relationship" {
                color "#707070"
                thickness 2
            }
        }
    }
}
