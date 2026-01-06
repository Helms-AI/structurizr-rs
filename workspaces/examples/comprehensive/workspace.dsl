/*
 * Comprehensive Example: All Features Combined
 *
 * This workspace demonstrates all the features implemented in structurizr-rs,
 * combining functionality from Phase 1, Phase 2, and Phase 5:
 *
 * Phase 1 (Directives):
 * - !const - Reusable constants
 * - !identifiers - Identifier mode
 * - !impliedRelationships - Automatic relationship inference
 * - !docs / !adrs - Documentation links
 *
 * Phase 2 (Enterprise & Config):
 * - enterprise block - Organizational boundary
 * - terminology - Custom element type labels
 *
 * Phase 5 (Scripting):
 * - !script lua - Native Lua scripts
 * - !script groovy - Auto-transpiled Groovy scripts
 *
 * This example models a complete healthcare platform with internal and
 * external systems, demonstrating real-world architecture documentation.
 */

workspace "HealthTech Platform" "Comprehensive healthcare data platform with EHR integration" {

    # =====================================================
    # PHASE 1: Directives
    # =====================================================

    # Constants for consistent styling and naming
    !const ORG_NAME "HealthTech Solutions"
    !const PRIMARY_COLOR "#0066cc"
    !const SECONDARY_COLOR "#4d94ff"
    !const EXTERNAL_COLOR "#666666"
    !const DB_TECH "PostgreSQL 15"
    !const API_STANDARD "REST/JSON over HTTPS"

    # Use flat identifiers
    !identifiers flat

    # Enable implied relationships
    !impliedRelationships true

    # Link documentation
    !docs "docs"
    !adrs "adrs"

    # =====================================================
    # PHASE 5: Scripting
    # =====================================================

    # Lua script to add monitoring infrastructure
    !script lua {
        -- Add operational components via script
        local ops = workspace:addPerson("Operations Team", "24/7 monitoring and incident response")

        -- Add observability stack
        local observability = workspace:addSoftwareSystem("Observability Platform", "Metrics, logs, and traces")

        print("Added operational infrastructure via Lua script")
    }

    # Groovy script for compliance tagging (auto-transpiled)
    !script groovy {
        // Modify workspace metadata
        workspace.setName("HealthTech Platform HIPAA Edition")

        // Add compliance system
        def compliance = workspace.addSoftwareSystem("Compliance Engine", "HIPAA audit and reporting")

        println("Added compliance components via Groovy script")
    }

    # =====================================================
    # MODEL
    # =====================================================

    model {
        # Define enterprise boundary
        enterprise "${ORG_NAME}" {

            # Internal actors
            physician = person "Physician" "Licensed healthcare provider"
            nurse = person "Nurse" "Clinical nursing staff"
            dataAnalyst = person "Data Analyst" "Healthcare data scientist"

            # Core EHR Platform
            ehrPlatform = softwareSystem "EHR Platform" "Electronic Health Records management system" {
                patientPortal = container "Patient Portal" "Self-service patient interface" "React"
                clinicianApp = container "Clinician Application" "Clinical workflow interface" "Angular"
                apiGateway = container "API Gateway" "Authentication and routing" "${API_STANDARD}"
                patientService = container "Patient Service" "Patient demographics" "Go"
                fhirServer = container "FHIR Server" "HL7 FHIR R4 data store" "HAPI FHIR"
                ehrDatabase = container "EHR Database" "Patient health records" "${DB_TECH}" {
                    tags "Database"
                }
            }

            # Analytics Platform
            analyticsPlat = softwareSystem "Analytics Platform" "Healthcare intelligence" {
                biDashboard = container "BI Dashboard" "Executive dashboards" "Tableau"
                dataWarehouse = container "Data Warehouse" "Analytical data store" "Snowflake" {
                    tags "Database"
                }
            }
        }

        # External actors (outside enterprise boundary)
        patient = person "Patient" "Healthcare consumer" {
            tags "External"
        }

        # External systems
        labSystem = softwareSystem "Lab Information System" "External lab provider" {
            tags "External"
        }
        hie = softwareSystem "Health Information Exchange" "Regional HIE network" {
            tags "External"
        }

        # =====================================================
        # RELATIONSHIPS
        # =====================================================

        # Patient interactions
        patient -> ehrPlatform "Views records, schedules appointments"

        # Clinical staff interactions
        physician -> ehrPlatform "Documents encounters, places orders"
        nurse -> ehrPlatform "Records vitals, administers meds"
        dataAnalyst -> analyticsPlat "Analyzes population health data"

        # System-to-system relationships
        ehrPlatform -> labSystem "Sends lab orders"
        labSystem -> ehrPlatform "Returns lab results"
        hie -> ehrPlatform "Queries patient data"
        ehrPlatform -> analyticsPlat "Streams clinical events"
    }

    # =====================================================
    # VIEWS
    # =====================================================

    views {
        # System Landscape - everything including scripted systems
        systemLandscape "Landscape" "${ORG_NAME} Healthcare Technology Landscape" {
            include *
            autoLayout
        }

        # EHR Platform context
        systemContext ehrPlatform "EHRContext" "EHR Platform in context" {
            include *
            autoLayout
        }

        # EHR Platform containers
        container ehrPlatform "EHRContainers" "EHR Platform internal architecture" {
            include *
            autoLayout
        }

        # Analytics Platform containers
        container analyticsPlat "AnalyticsContainers" "Analytics Platform architecture" {
            include *
            autoLayout
        }

        # Custom terminology
        terminology {
            person "Actor"
            softwareSystem "System"
            container "Service"
            enterprise "Organization"
        }

        # =====================================================
        # STYLES
        # =====================================================

        styles {
            element "Person" {
                shape Person
                background "${PRIMARY_COLOR}"
                color "#ffffff"
            }
            element "Software System" {
                background "${SECONDARY_COLOR}"
                color "#ffffff"
            }
            element "Container" {
                background "#85bbf0"
                color "#000000"
            }
            element "Database" {
                shape Cylinder
            }
            element "External" {
                background "${EXTERNAL_COLOR}"
                color "#ffffff"
            }
        }
    }
}
