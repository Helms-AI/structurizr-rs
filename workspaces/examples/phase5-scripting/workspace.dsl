/*
 * Phase 5 Example: Native Scripting Support
 *
 * This workspace demonstrates the scripting features implemented in Phase 5:
 * - !script lua { ... } - Native Lua scripting
 * - !script groovy { ... } - Groovy scripts (auto-transpiled to Lua)
 * - Programmatic workspace modification
 * - Dynamic element creation
 *
 * NOTE: Script execution requires the "scripting" feature flag.
 * Build with: cargo build --features scripting
 */

workspace "Phase 5 - Scripting Demo" "Demonstrates native Lua scripting and Groovy compatibility" {

    !docs "docs"
    !adrs "adrs"

    /*
     * Lua Script Example
     *
     * This script modifies the workspace programmatically.
     * Lua is the native scripting language for structurizr-rs.
     */
    !script lua {
        -- Lua comments use double-dash syntax
        -- Modify the workspace description
        workspace:setDescription("Enhanced by Lua scripting at build time")

        -- Add a dynamically created person
        local admin = workspace:addPerson("System Administrator", "Manages infrastructure and deployments")

        -- Add a software system via script
        local monitoring = workspace:addSoftwareSystem("Monitoring Stack", "Observability and alerting platform")

        -- Log to console during parsing (useful for debugging)
        print("Lua script executed successfully!")
    }

    /*
     * Groovy Script Example (Auto-Transpiled)
     *
     * Groovy scripts are automatically converted to Lua for execution.
     * This provides backwards compatibility with existing Structurizr scripts.
     */
    !script groovy {
        // Groovy-style comments work too
        // Property assignment syntax
        workspace.setName("Scripted Workspace")

        // Add another system using Groovy syntax
        def cicd = workspace.addSoftwareSystem("CI/CD Pipeline", "Automated build and deployment")

        // Method calls are converted to Lua colon notation
        println("Groovy script transpiled and executed!")
    }

    model {
        # Static elements defined in DSL
        developer = person "Developer" "Writes and maintains application code"
        devops = person "DevOps Engineer" "Manages infrastructure and CI/CD"

        # Primary application (static)
        webapp = softwareSystem "Web Application" "The main customer-facing application" {
            frontend = container "Frontend" "Single-page application" "React/TypeScript"
            backend = container "Backend API" "RESTful API server" "Node.js/Express"
            database = container "Database" "Application data store" "PostgreSQL" {
                tags "Database"
            }
            cache = container "Cache" "Session and data cache" "Redis" {
                tags "Database"
            }

            # Container relationships (defined inside the system)
            frontend -> backend "Calls API" "JSON/HTTPS"
            backend -> database "Reads/writes data" "SQL"
            backend -> cache "Caches sessions" "Redis Protocol"
        }

        # Relationships for static elements
        developer -> webapp "Develops and maintains"
        devops -> webapp "Deploys and monitors"
    }

    views {
        systemLandscape "Landscape" "All systems including script-generated ones" {
            include *
            autoLayout
        }

        systemContext webapp "WebAppContext" "Web Application in context" {
            include *
            autoLayout
        }

        container webapp "WebAppContainers" "Web Application internals" {
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
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Database" {
                shape Cylinder
            }
        }
    }
}
