/*
 * Phase 1 Example: Directives
 *
 * This workspace demonstrates the core DSL directives implemented in Phase 1:
 * - !const - Define constants for reuse throughout the workspace
 * - !identifiers - Control identifier generation behavior
 * - !impliedRelationships - Enable automatic relationship inference
 * - !docs - Link to documentation directory
 * - !adrs - Link to Architecture Decision Records
 */

workspace "Phase 1 - Directives Demo" "Demonstrates !const, !identifiers, !impliedRelationships, !docs, and !adrs directives" {

    # Define constants for consistent styling and naming
    !const COMPANY_NAME "Acme Corp"
    !const PRIMARY_COLOR "#1168bd"
    !const SECONDARY_COLOR "#438dd5"
    !const DATABASE_TECH "PostgreSQL 15"
    !const API_TECH "REST/JSON over HTTPS"

    # Use flat identifiers (default)
    !identifiers flat

    # Enable implied relationships - relationships between containers
    # imply relationships between their parent software systems
    !impliedRelationships true

    # Link to documentation
    !docs "docs"

    # Link to Architecture Decision Records
    !adrs "adrs"

    model {
        # External users
        customer = person "Customer" "A customer of ${COMPANY_NAME}" {
            tags "External"
        }
        support = person "Support Agent" "Helps customers with their issues" {
            tags "Internal"
        }

        # Core software system
        ecommerce = softwareSystem "E-Commerce Platform" "The ${COMPANY_NAME} online store" {
            webApp = container "Web Application" "Delivers the static content and SPA" "React"
            apiGateway = container "API Gateway" "Routes API requests" "${API_TECH}"
            orderService = container "Order Service" "Handles order processing" "Node.js"
            inventoryService = container "Inventory Service" "Manages inventory" "Go"
            database = container "Database" "Stores data" "${DATABASE_TECH}" {
                tags "Database"
            }
        }

        # External systems
        paymentProvider = softwareSystem "Payment Provider" "Handles payment processing" {
            tags "External"
        }
        emailService = softwareSystem "Email Service" "Sends transactional emails" {
            tags "External"
        }

        # System-level relationships (implied from container relationships)
        customer -> ecommerce "Uses"
        support -> ecommerce "Uses"
        ecommerce -> paymentProvider "Processes payments via"
        ecommerce -> emailService "Sends emails via"
    }

    views {
        # System Landscape - shows all systems and people
        systemLandscape "SystemLandscape" "Overview of ${COMPANY_NAME} systems" {
            include *
            autoLayout
        }

        # System Context - focused on the e-commerce platform
        systemContext ecommerce "SystemContext" "The ${COMPANY_NAME} E-Commerce Platform context" {
            include *
            autoLayout
        }

        # Container view - shows internal structure
        container ecommerce "Containers" "Internal structure of the E-Commerce Platform" {
            include *
            autoLayout
        }

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
                background "#999999"
            }
            element "Internal" {
                background "${PRIMARY_COLOR}"
            }
        }
    }
}
