workspace "Test Validation" "A workspace to test validation" {

    model {
        # Person with description - OK
        user = person "User" "A user of the system"

        # Person without description - WARNING
        admin = person "Admin"

        # Software system with description
        mainSystem = softwareSystem "Main System" "The main system" {
            # Container without technology - WARNING
            webapp = container "Web App" "The web application"

            # Container with technology - OK
            api = container "API" "REST API" "Node.js"

            # Container with technology
            db = container "Database" "Main database" "PostgreSQL" {
                # Component without technology - INFO
                userRepo = component "User Repository" "Handles user data"
            }
        }

        # External system - OK
        externalSystem = softwareSystem "External System" "External service"

        # Orphan system (no relationships) - INFO
        orphanSystem = softwareSystem "Orphan System" "Not connected to anything"

        # Duplicate name - ERROR
        duplicate = person "User" "Another user with same name"

        # Relationships
        user -> webapp "Uses"
        webapp -> api "Calls" "HTTPS"
        api -> db "Reads/Writes" "SQL"
        mainSystem -> externalSystem "Integrates with"

        # Circular relationship - WARNING
        externalSystem -> externalSystem "Self reference"
    }

    views {
        systemContext mainSystem "SystemContext" "System Context" {
            include *
            autoLayout
        }

        container mainSystem "Containers" "Container View" {
            include *
            autoLayout
        }

        # Note: orphanSystem is not included in any view - INFO

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
        }
    }
}
