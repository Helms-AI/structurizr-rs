// Example DSL file demonstrating all directive features

workspace "Directive Features Demo" "Demonstrates all DSL directive capabilities" {

    // Constants - define reusable values that can be substituted with ${NAME}
    !const COMPANY_NAME "TechCorp"
    !const DATABASE_TECH "PostgreSQL 15"
    !const API_PROTOCOL "REST/HTTPS"
    !const CLOUD_PROVIDER "AWS"
    !const ENVIRONMENT "Production"

    // Enable implied relationships - automatically creates transitive relationships
    // If A->B and B->C exist, this will create A->C
    !impliedRelationships true

    // Specify paths for documentation and ADRs
    !docs "docs"
    !adrs "architecture-decisions"

    model {
        // Using constants in element names and descriptions
        customer = person "${COMPANY_NAME} Customer" "A customer using ${COMPANY_NAME} services"
        admin = person "${COMPANY_NAME} Administrator" "System administrator"

        webApp = softwareSystem "Web Application" "Customer-facing web application hosted on ${CLOUD_PROVIDER}" {
            webapp = container "Web App" "Delivers content to customers" "React"
            api = container "API Gateway" "Handles ${API_PROTOCOL} requests" "Node.js/Express"
        }

        database = softwareSystem "Database" "Stores all application data" {
            db = container "Primary DB" "Main data store using ${DATABASE_TECH}" "${DATABASE_TECH}"
        }

        analyticsSystem = softwareSystem "Analytics System" "Processes usage analytics"

        // Relationships using constants
        customer -> webApp "Uses for shopping"
        admin -> webApp "Manages via admin panel"

        // Internal relationships
        webapp -> api "Makes API calls to" "${API_PROTOCOL}"
        api -> database "Reads from and writes to" "${DATABASE_TECH}"

        // This relationship, combined with implied relationships, will create:
        // customer -> database (implied from customer -> webApp -> database)
        // admin -> database (implied from admin -> webApp -> database)
        database -> analyticsSystem "Sends data to for analysis"
    }

    views {
        systemContext webApp "SystemContext" "System Context for ${COMPANY_NAME} Web Application" {
            include *
            autoLayout tb 300 300
        }

        container webApp "Containers" "Container view for Web Application" {
            include *
            autoLayout lr 200 200
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
