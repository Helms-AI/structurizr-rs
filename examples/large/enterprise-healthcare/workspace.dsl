workspace "Enterprise Healthcare System" "A comprehensive hospital information system managing clinical, administrative, and patient care operations across multiple departments" {

    !docs "docs"
    !adrs "adrs"

    model {
        doctor = person "Doctor" "Medical practitioner who diagnoses and treats patients"
        nurse = person "Nurse" "Healthcare professional who provides patient care and administers treatments"
        patient = person "Patient" "Individual receiving medical care and services"
        admin = person "Administrative Staff" "Hospital administrator managing operations and staff"
        labTech = person "Lab Technician" "Laboratory professional who conducts diagnostic tests"
        pharmacist = person "Pharmacist" "Licensed professional who dispenses medications and provides drug information"
        radiologist = person "Radiologist" "Medical doctor specializing in diagnosing diseases using medical imaging"
        billingStaff = person "Billing Staff" "Financial administrator who processes claims and manages patient billing"

        ehrSystem = softwareSystem "Electronic Health Record System" "Core clinical system managing patient records, orders, and care coordination" {
            clinicalPortal = container "Clinical Portal" "Web-based interface for healthcare providers" "React/TypeScript"
            mobileApp = container "Mobile App" "Mobile application for physicians and nurses" "React Native"
            apiGateway = container "API Gateway" "Central API gateway handling authentication and routing" "Kong/Nginx"
            patientService = container "Patient Service" "Manages patient demographics and medical records" "Rust/Actix"
            clinicalService = container "Clinical Service" "Handles clinical workflows, orders, and decision support" "Rust/Actix" {
                rulesEngine = component "Clinical Rules Engine" "Executes clinical decision support rules" "Rust"
                drugChecker = component "Drug Interaction Checker" "Validates medication orders for interactions" "Rust"
                alertGenerator = component "Alert Generator" "Generates clinical alerts and notifications" "Rust"
                orderProcessor = component "Order Processor" "Processes clinical orders and requisitions" "Rust"
            }
            labIntegration = container "Lab Integration Service" "Integrates with laboratory systems" "Rust/Actix"
            pharmacyIntegration = container "Pharmacy Integration Service" "Integrates with pharmacy systems" "Rust/Actix"
            primaryDb = container "Primary Database" "Stores patient and clinical data" "PostgreSQL"
            cacheLayer = container "Cache Layer" "Distributed cache for performance" "Redis"
            messageQueue = container "Message Queue" "Asynchronous message processing" "RabbitMQ"
        }

        labSystem = softwareSystem "Laboratory Information System" "Manages lab test orders, results, and quality control" {
            labPortal = container "Lab Portal" "Web interface for lab technicians" "Angular"
            labProcessor = container "Lab Result Processor" "Processes and validates lab results" "Java/Spring"
            instrumentInterface = container "Instrument Interface" "Connects to lab equipment" "Java"
            labDb = container "Lab Database" "Stores test results and reference ranges" "PostgreSQL"
        }

        pharmacySystem = softwareSystem "Pharmacy Management System" "Manages medication dispensing, inventory, and formulary" {
            pharmacyPortal = container "Pharmacy Portal" "Web interface for pharmacists" "Vue.js"
            dispensingService = container "Dispensing Service" "Manages medication dispensing workflows" "Java/Spring"
            inventoryService = container "Inventory Management" "Tracks medication stock and ordering" "Java/Spring"
            pharmacyDb = container "Pharmacy Database" "Stores medication and inventory data" "PostgreSQL"
        }

        radiologySystem = softwareSystem "Radiology Information System" "Manages imaging orders, studies, and PACS integration" {
            radiologyPortal = container "Radiology Portal" "Web interface for radiologists" "React"
            studyManager = container "Study Manager" "Manages imaging studies and reports" "Python/Django"
            pacsConnector = container "PACS Connector" "Integrates with picture archiving system" "Python"
            radiologyDb = container "Radiology Database" "Stores imaging metadata and reports" "PostgreSQL"
        }

        billingSystem = softwareSystem "Billing and Claims System" "Manages patient billing, insurance claims, and revenue cycle" {
            billingPortal = container "Billing Portal" "Web interface for billing staff" "Angular"
            claimsProcessor = container "Claims Processor" "Processes insurance claims" "Java/Spring"
            chargeCapture = container "Charge Capture Service" "Captures and validates charges" "Java/Spring"
            billingDb = container "Billing Database" "Stores billing and claims data" "PostgreSQL"
        }

        patientPortal = softwareSystem "Patient Portal" "Patient-facing portal for appointments, records, and communication" {
            patientWeb = container "Patient Web Portal" "Web interface for patients" "React"
            patientApi = container "Patient API" "Backend API for patient services" "Node.js/Express"
            appointmentService = container "Appointment Service" "Manages patient appointments" "Node.js"
            messagingService = container "Secure Messaging" "Enables patient-provider communication" "Node.js"
            patientDb = container "Patient Portal Database" "Stores patient portal data" "MongoDB"
        }

        emergencySystem = softwareSystem "Emergency Department System" "Manages emergency department workflows and triage" {
            edPortal = container "ED Portal" "Emergency department tracking board" "React"
            triageService = container "Triage Service" "Manages patient triage and prioritization" "Rust/Actix"
            edDb = container "ED Database" "Stores emergency department data" "PostgreSQL"
        }

        schedulingSystem = softwareSystem "Enterprise Scheduling System" "Manages appointments, resources, and provider schedules" {
            schedulingPortal = container "Scheduling Portal" "Web interface for scheduling staff" "Vue.js"
            schedulingEngine = container "Scheduling Engine" "Optimizes appointment scheduling" "Python/FastAPI"
            schedulingDb = container "Scheduling Database" "Stores schedules and appointments" "PostgreSQL"
        }

        analyticsSystem = softwareSystem "Clinical Analytics Platform" "Provides reporting, analytics, and business intelligence" {
            analyticsPortal = container "Analytics Portal" "Dashboard and reporting interface" "React"
            dataWarehouse = container "Data Warehouse" "Central data repository for analytics" "PostgreSQL"
            etlPipeline = container "ETL Pipeline" "Extracts and transforms data from source systems" "Apache Spark"
            reportingEngine = container "Reporting Engine" "Generates reports and visualizations" "Python"
        }

        integrationHub = softwareSystem "Integration Hub" "Enterprise service bus for system integration" {
            tags "External"
        }

        identityManagement = softwareSystem "Identity Management System" "Centralized authentication and authorization" {
            tags "External"
        }

        complianceSystem = softwareSystem "Compliance and Audit System" "Manages regulatory compliance and audit trails" {
            compliancePortal = container "Compliance Portal" "Web interface for compliance officers" "Angular"
            auditService = container "Audit Service" "Tracks and reports on system access and changes" "Java/Spring"
            complianceDb = container "Compliance Database" "Stores audit logs and compliance data" "PostgreSQL"
        }

        doctor -> clinicalPortal "Views patient records and enters orders using"
        doctor -> mobileApp "Reviews patient information and signs orders using"
        nurse -> clinicalPortal "Documents patient care and administers medications using"
        nurse -> mobileApp "Records vital signs and care activities using"
        patient -> patientWeb "Views health records and schedules appointments using"
        admin -> schedulingPortal "Manages provider schedules and resources using"
        labTech -> labPortal "Enters and validates lab results using"
        pharmacist -> pharmacyPortal "Reviews and dispenses medications using"
        radiologist -> radiologyPortal "Reviews imaging studies and dictates reports using"
        billingStaff -> billingPortal "Processes claims and manages billing using"

        clinicalPortal -> apiGateway "Makes API calls to" "HTTPS/REST"
        mobileApp -> apiGateway "Authenticates and sends requests to" "HTTPS/REST"
        apiGateway -> patientService "Routes patient requests to" "HTTP/REST"
        apiGateway -> clinicalService "Routes clinical requests to" "HTTP/REST"
        apiGateway -> identityManagement "Validates authentication tokens with" "HTTPS/OAuth2"

        patientService -> primaryDb "Reads and writes patient data to" "SQL"
        patientService -> cacheLayer "Caches patient data in" "Redis Protocol"
        patientService -> messageQueue "Publishes patient events to" "AMQP"

        clinicalService -> primaryDb "Reads and writes clinical data to" "SQL"
        clinicalService -> messageQueue "Publishes order events to" "AMQP"
        clinicalService -> cacheLayer "Caches clinical data in" "Redis Protocol"

        rulesEngine -> drugChecker "Validates medication orders with" "Internal API"
        rulesEngine -> alertGenerator "Triggers clinical alerts via" "Internal API"
        orderProcessor -> rulesEngine "Executes clinical rules through" "Internal API"
        orderProcessor -> alertGenerator "Generates order-related alerts via" "Internal API"

        labIntegration -> labProcessor "Sends lab orders to" "HL7/FHIR"
        labIntegration -> messageQueue "Receives lab results from" "AMQP"
        labIntegration -> primaryDb "Stores lab results in" "SQL"

        pharmacyIntegration -> dispensingService "Sends medication orders to" "HL7"
        pharmacyIntegration -> messageQueue "Receives dispense notifications from" "AMQP"
        pharmacyIntegration -> primaryDb "Updates medication administration in" "SQL"

        labProcessor -> labDb "Reads and writes lab data to" "SQL"
        labProcessor -> instrumentInterface "Receives results from" "HL7"
        labProcessor -> integrationHub "Sends results to" "HL7/FHIR"

        dispensingService -> pharmacyDb "Reads and writes pharmacy data to" "SQL"
        dispensingService -> inventoryService "Updates inventory via" "HTTP/REST"
        dispensingService -> integrationHub "Sends dispense notifications to" "HL7"

        studyManager -> radiologyDb "Reads and writes imaging data to" "SQL"
        studyManager -> pacsConnector "Retrieves images from" "DICOM"
        studyManager -> integrationHub "Sends reports to" "HL7/FHIR"

        claimsProcessor -> billingDb "Reads and writes billing data to" "SQL"
        claimsProcessor -> chargeCapture "Receives charges from" "HTTP/REST"
        chargeCapture -> integrationHub "Receives clinical charges from" "HL7"

        patientWeb -> patientApi "Makes API requests to" "HTTPS/REST"
        patientApi -> appointmentService "Manages appointments via" "HTTP/REST"
        patientApi -> messagingService "Sends messages via" "HTTP/REST"
        patientApi -> identityManagement "Authenticates patients with" "HTTPS/OAuth2"
        appointmentService -> patientDb "Stores appointment data in" "MongoDB"
        messagingService -> patientDb "Stores messages in" "MongoDB"
        appointmentService -> schedulingEngine "Books appointments with" "HTTP/REST"

        triageService -> edDb "Stores triage data in" "SQL"
        triageService -> integrationHub "Sends ED admissions to" "HL7"
        edPortal -> triageService "Displays triage status from" "HTTP/REST"

        schedulingEngine -> schedulingDb "Reads and writes schedules to" "SQL"
        schedulingEngine -> integrationHub "Publishes appointment events to" "HL7/FHIR"

        etlPipeline -> primaryDb "Extracts clinical data from" "SQL"
        etlPipeline -> labDb "Extracts lab data from" "SQL"
        etlPipeline -> pharmacyDb "Extracts pharmacy data from" "SQL"
        etlPipeline -> radiologyDb "Extracts radiology data from" "SQL"
        etlPipeline -> billingDb "Extracts billing data from" "SQL"
        etlPipeline -> dataWarehouse "Loads transformed data into" "SQL"
        reportingEngine -> dataWarehouse "Queries analytics data from" "SQL"
        analyticsPortal -> reportingEngine "Requests reports from" "HTTP/REST"

        integrationHub -> messageQueue "Routes messages to" "AMQP"
        integrationHub -> apiGateway "Sends external events to" "HTTPS/REST"

        auditService -> complianceDb "Stores audit logs in" "SQL"
        auditService -> integrationHub "Receives audit events from" "HL7/FHIR"
        compliancePortal -> auditService "Retrieves audit reports from" "HTTP/REST"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of the entire enterprise healthcare ecosystem" {
            include *
            autoLayout
        }

        systemContext ehrSystem "EHRSystemContext" "System context diagram for the Electronic Health Record System" {
            include *
            autoLayout
        }

        container ehrSystem "EHRContainers" "Container diagram showing the internal architecture of the EHR System" {
            include *
            autoLayout
        }

        component clinicalService "ClinicalServiceComponents" "Component diagram showing the Clinical Service internal structure" {
            include *
            autoLayout
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Component" {
                background "#85bbf0"
                color "#000000"
            }
            relationship "Relationship" {
                color "#707070"
                thickness 2
                routing Orthogonal
            }
        }
    }
}
