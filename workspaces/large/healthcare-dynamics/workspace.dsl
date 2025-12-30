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
        patient -> patientPortal "Accesses health records via" "HTTPS" {
            tags "Patient Access, Self-Service"
            properties {
                "authentication" "MyChart SSO"
                "mobileApp" "Available"
            }
            perspectives {
                "Patient Experience" "24/7 access to health records and appointments"
                "Security" "HIPAA-compliant patient data access"
                "Compliance" "Patient portal required for Meaningful Use"
            }
        }
        patient -> receptionist "Arrives and checks in at front desk" {
            tags "In-Person, Registration"
            perspectives {
                "Patient Experience" "Average wait time target: 5 minutes"
                "Operations" "Photo ID and insurance card verification"
            }
        }

        // Staff to system interactions
        receptionist -> admissionService "Manages patient check-in with" {
            tags "Registration, ADT"
            properties {
                "scanners" "ID and insurance card"
                "kiosk" "Self-service available"
            }
            perspectives {
                "Workflow" "Average check-in time: 3 minutes"
                "Compliance" "Identity verification required"
            }
        }
        receptionist -> schedulingService "Books appointments via" {
            tags "Scheduling, Admin"
            properties {
                "overbooking" "10% allowed"
            }
            perspectives {
                "Operations" "Real-time availability view"
            }
        }
        nurse -> clinicalWorkstation "Documents care in" {
            tags "Clinical Documentation, Nursing"
            properties {
                "flowsheets" "Vitals, I/O, assessments"
                "barcodeScanning" "Medication administration"
            }
            perspectives {
                "Clinical" "Standardized nursing documentation"
                "Safety" "5 rights of medication administration"
                "Compliance" "Required for Joint Commission"
            }
        }
        doctor -> clinicalWorkstation "Reviews charts and enters orders in" {
            tags "Clinical Documentation, CPOE"
            properties {
                "orderSets" "Evidence-based templates"
                "alerts" "Drug-drug interactions"
            }
            perspectives {
                "Clinical" "Comprehensive patient chart view"
                "Safety" "Clinical decision support alerts"
                "Compliance" "CPOE required for quality metrics"
            }
        }
        labTechnician -> integrationHub "Views lab orders via" "HL7" {
            tags "Lab, Orders"
            perspectives {
                "Workflow" "Real-time order queue"
            }
        }
        pharmacist -> integrationHub "Views prescriptions via" "HL7" {
            tags "Pharmacy, Rx"
            perspectives {
                "Safety" "Drug utilization review"
            }
        }
        billingClerk -> integrationHub "Views billing data via" {
            tags "Billing, Revenue Cycle"
            perspectives {
                "Financial" "Real-time charge capture"
            }
        }

        // Portal to services
        patientPortal -> schedulingService "Requests appointments from" "REST" {
            tags "Patient Self-Service, Scheduling"
            properties {
                "realTimeAvailability" "true"
                "confirmationMethod" "Email + SMS"
            }
            perspectives {
                "Patient Experience" "Online booking reduces call volume 40%"
                "Operations" "Automated appointment reminders"
            }
        }
        clinicalWorkstation -> clinicalService "Accesses clinical data via" "gRPC" {
            tags "Clinical, Real-time"
            properties {
                "caching" "Chart data cached 5 min"
                "auditLogging" "All access logged"
            }
            perspectives {
                "Performance" "Sub-second chart load time"
                "Security" "Break-the-glass for restricted records"
                "Compliance" "HIPAA access audit trail"
            }
        }

        // Service to service
        admissionService -> ehrDatabase "Stores patient data in" "SQL" {
            tags "Data, Patient Demographics"
            properties {
                "encryption" "AES-256"
                "backup" "Every 15 minutes"
            }
            perspectives {
                "Data" "Master patient index source"
                "Compliance" "HIPAA data protection"
            }
        }
        admissionService -> integrationHub "Checks insurance via" "HL7" {
            tags "Eligibility, Insurance"
            properties {
                "responseTime" "Real-time 270/271"
                "caching" "24 hours"
            }
            perspectives {
                "Financial" "Reduces claim denials 30%"
                "Operations" "Real-time eligibility verification"
            }
        }
        schedulingService -> ehrDatabase "Stores appointments in" "SQL" {
            tags "Data, Scheduling"
            properties {
                "conflictDetection" "Automatic"
            }
            perspectives {
                "Operations" "Resource optimization"
            }
        }
        clinicalService -> ehrDatabase "Reads/writes clinical records in" "SQL" {
            tags "Data, Clinical, PHI"
            properties {
                "versionControl" "All changes tracked"
                "attestation" "Digital signatures"
            }
            perspectives {
                "Clinical" "Single source of truth for patient care"
                "Legal" "Legal medical record"
                "Compliance" "21 CFR Part 11 compliant"
            }
        }
        clinicalService -> integrationHub "Sends orders via" "HL7" {
            tags "Orders, Integration"
            properties {
                "format" "HL7 v2.5.1 ORM/ORU"
                "acknowledgment" "Required"
            }
            perspectives {
                "Clinical" "CPOE to ancillary departments"
                "Workflow" "Real-time order transmission"
            }
        }
        notificationService -> doctor "Alerts doctor results ready" "Secure Message" {
            tags "Notification, Clinical Alert"
            properties {
                "priority" "Critical/Routine"
                "escalation" "15 min for critical"
            }
            perspectives {
                "Clinical" "Time-sensitive result notification"
                "Safety" "Critical value alerts"
            }
        }
        notificationService -> patient "Sends visit summary and follow-up" "Email/SMS" {
            tags "Notification, Patient Communication"
            properties {
                "methods" "Email, SMS, Patient Portal"
                "language" "Patient preferred"
            }
            perspectives {
                "Patient Experience" "Post-visit engagement"
                "Compliance" "Discharge summary within 24 hours"
            }
        }

        // Integration hub connections
        integrationHub -> labSystem "Routes lab orders to" "HL7/FHIR" {
            tags "Integration, Lab, HL7"
            properties {
                "messageType" "ORM^O01, ORU^R01"
                "resultsInterface" "Bidirectional"
            }
            perspectives {
                "Clinical" "Electronic lab ordering and results"
                "Workflow" "Eliminates paper requisitions"
                "Quality" "Reduced transcription errors"
            }
        }
        integrationHub -> pharmacySystem "Sends prescriptions to" "HL7/FHIR" {
            tags "Integration, Pharmacy, eRx"
            properties {
                "messageType" "RDE^O11, RDS^O13"
                "controlledSubstances" "EPCS compliant"
            }
            perspectives {
                "Safety" "Electronic prescribing reduces errors"
                "Compliance" "EPCS for controlled substances"
                "Workflow" "Direct pharmacy transmission"
            }
        }
        integrationHub -> billingSystem "Sends charges to" "HL7" {
            tags "Integration, Billing, Revenue"
            properties {
                "messageType" "DFT^P03"
                "realTime" "true"
            }
            perspectives {
                "Financial" "Real-time charge capture"
                "Compliance" "Accurate billing documentation"
            }
        }
        integrationHub -> insurancePortal "Checks eligibility via" "REST" {
            tags "Integration, Insurance, Eligibility"
            properties {
                "standard" "X12 270/271"
                "caching" "24 hours"
            }
            perspectives {
                "Financial" "Reduces claim denials"
                "Patient Experience" "Accurate copay estimates"
            }
        }

        // Additional relationships for dynamic views
        labSystem -> integrationHub "Sends results via HL7" "HL7" {
            tags "Integration, Lab Results"
            properties {
                "messageType" "ORU^R01"
                "criticalValues" "Immediate notification"
            }
            perspectives {
                "Clinical" "Electronic lab results"
                "Safety" "Critical value alerting"
            }
        }
        integrationHub -> clinicalService "Results arrive in EHR" "HL7" {
            tags "Integration, Results"
            perspectives {
                "Clinical" "Automatic chart update"
            }
        }
        billingSystem -> integrationHub "Submits claim to payer" "X12" {
            tags "Integration, Claims"
            properties {
                "format" "X12 837"
            }
            perspectives {
                "Financial" "Electronic claims submission"
            }
        }
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

        // Dynamic view 3: Concurrent Order Processing - Lab and Pharmacy in Parallel
        dynamic hospitalSystem "ConcurrentOrders" "Parallel lab and pharmacy processing" {
            doctor -> clinicalWorkstation "Opens patient chart"
            doctor -> clinicalService "Places lab and medication orders"
            clinicalService -> integrationHub "Sends orders to integration hub"
            {
                integrationHub -> labSystem "Routes lab order"
                labTechnician -> labSystem "Processes specimen"
                labSystem -> integrationHub "Returns lab results"
            }
            {
                integrationHub -> pharmacySystem "Routes prescription"
                pharmacist -> pharmacySystem "Prepares medication"
            }
            integrationHub -> clinicalService "All results ready"
            notificationService -> doctor "Alerts doctor"
            autoLayout lr
        }

        // Dynamic view 4: Parallel Discharge Processing
        dynamic hospitalSystem "ParallelDischarge" "Concurrent discharge activities" {
            doctor -> clinicalService "Signs discharge orders"
            clinicalService -> integrationHub "Initiates discharge workflow"
            {
                integrationHub -> pharmacySystem "Sends prescriptions"
                pharmacist -> pharmacySystem "Prepares take-home meds"
            }
            {
                integrationHub -> billingSystem "Sends charges"
                billingClerk -> billingSystem "Prepares final bill"
            }
            {
                nurse -> clinicalWorkstation "Prepares discharge instructions"
                clinicalService -> ehrDatabase "Saves discharge summary"
            }
            notificationService -> patient "Sends visit summary"
            receptionist -> admissionService "Completes checkout"
            autoLayout lr
        }

        // Dynamic view 5: Nested Parallel Processing
        dynamic hospitalSystem "NestedParallel" "Complex parallel workflows with nesting" {
            patient -> patientPortal "Requests appointment"
            schedulingService -> ehrDatabase "Checks availability"
            {
                admissionService -> integrationHub "Verifies insurance"
                {
                    integrationHub -> insurancePortal "Checks eligibility"
                }
                {
                    integrationHub -> billingSystem "Gets copay estimate"
                }
            }
            {
                schedulingService -> notificationService "Prepares confirmation"
            }
            notificationService -> patient "Sends appointment details"
            autoLayout lr
        }

        // Dynamic view 6: Discharge and Billing Flow (original, renamed)
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
