// Startup SaaS Analytics Platform
// A small B2B analytics platform example demonstrating core DSL features

workspace "Startup SaaS Analytics" "B2B analytics platform for product insights" {

    !const COMPANY "InsightMetrics"
    !const DB_TECH "PostgreSQL"
    !const CACHE_TECH "Redis"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        // Users
        productManager = person "${COMPANY} Product Manager" "Analyzes user behavior and product metrics"
        developer = person "${COMPANY} Developer" "Integrates analytics SDK into applications"

        // Main System
        analyticsSystem = softwareSystem "${COMPANY} Analytics Platform" "Provides product analytics and insights" {
            webApp = container "Web Dashboard" "Analytics dashboard for viewing metrics" "React/TypeScript"
            api = container "Analytics API" "REST API for data ingestion and queries" "Node.js/Express"
            database = container "Analytics Database" "Stores events and computed metrics" "${DB_TECH}"
            cache = container "Cache Layer" "Caches frequently accessed metrics" "${CACHE_TECH}"
        }

        // External Systems
        authProvider = softwareSystem "Auth0" "Identity and access management" {
            tags "External"
        }

        dataWarehouse = softwareSystem "Snowflake" "Data warehouse for historical analysis" {
            tags "External"
        }

        // Relationships - Users to Systems
        productManager -> analyticsSystem "Views dashboards and reports"
        developer -> analyticsSystem "Sends events via SDK"

        // Internal Relationships
        webApp -> api "Fetches metrics via" "REST/HTTPS"
        api -> database "Reads/writes events" "SQL"
        api -> cache "Caches hot data" "Redis Protocol"

        // External Relationships
        api -> authProvider "Authenticates users via" "OAuth 2.0"
        api -> dataWarehouse "Syncs data for analysis" "Snowflake Connector"
    }

    views {
        systemContext analyticsSystem "SystemContext" "System Context for ${COMPANY} Analytics" {
            include *
            autoLayout tb
        }

        container analyticsSystem "Containers" "Container view showing internal structure" {
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
            }
        }
    }
}
