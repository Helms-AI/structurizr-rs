// Small Medical Clinic Management System
// A healthcare example demonstrating core DSL features

workspace "Clinic Management System" "Patient management for small medical clinics" {

    !const CLINIC_NAME "HealthFirst Clinic"
    !const DB_TECH "PostgreSQL"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        // Users
        doctor = person "Doctor" "Provides medical care and reviews patient records"
        receptionist = person "Receptionist" "Manages appointments and patient check-in"
        patient = person "Patient" "Receives medical care and views their records"

        // Main System
        clinicSystem = softwareSystem "${CLINIC_NAME} System" "Manages patient care and clinic operations" {
            appointmentApp = container "Appointment Portal" "Web app for scheduling and check-in" "React"
            patientRecords = container "Patient Records Service" "Manages patient medical records" "Java/Spring Boot"
            billingService = container "Billing Service" "Handles invoicing and payments" "Python/FastAPI"
            database = container "Clinic Database" "Stores all patient and billing data" "${DB_TECH}"
        }

        // External Systems
        insuranceApi = softwareSystem "Insurance Provider API" "Verifies coverage and processes claims" {
            tags "External"
        }

        labSystem = softwareSystem "Lab System" "External laboratory for test results" {
            tags "External"
        }

        pharmacySystem = softwareSystem "Pharmacy System" "Prescription management" {
            tags "External"
        }

        // User Relationships
        doctor -> clinicSystem "Views records and writes prescriptions"
        receptionist -> clinicSystem "Schedules appointments and checks in patients"
        patient -> clinicSystem "Books appointments and views records"

        // Internal Relationships
        appointmentApp -> patientRecords "Retrieves patient info" "REST/HTTPS"
        appointmentApp -> billingService "Creates invoices" "REST/HTTPS"
        patientRecords -> database "Reads/writes records" "SQL"
        billingService -> database "Stores billing data" "SQL"

        // External Relationships
        billingService -> insuranceApi "Submits claims" "HL7 FHIR"
        patientRecords -> labSystem "Receives test results" "HL7 FHIR"
        patientRecords -> pharmacySystem "Sends prescriptions" "NCPDP"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of ${CLINIC_NAME} systems" {
            include *
            autoLayout tb
        }

        systemContext clinicSystem "SystemContext" "System Context for ${CLINIC_NAME}" {
            include *
            autoLayout tb
        }

        container clinicSystem "Containers" "Internal structure of the clinic system" {
            include *
            autoLayout tb
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
            relationship "Relationship" {
                color "#707070"
                routing Orthogonal
            }
        }
    }
}
