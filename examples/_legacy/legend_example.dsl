workspace "Legend Example" "Example workspace demonstrating the legend feature" {

    model {
        user = person "User" "A user of the system"
        admin = person "Administrator" "System administrator"

        system = softwareSystem "Main System" "The primary software system" {
            webapp = container "Web Application" "Provides web interface" "React"
            api = container "API" "Provides REST API" "Node.js"
            database = container "Database" "Stores data" "PostgreSQL"
        }

        externalSystem = softwareSystem "External System" "Third-party system"

        user -> webapp "Uses" "HTTPS"
        admin -> webapp "Manages" "HTTPS"
        webapp -> api "Calls" "REST/JSON"
        api -> database "Reads/Writes" "SQL"
        api -> externalSystem "Integrates with" "REST/JSON" {
            tags "Async"
        }
    }

    views {
        systemLandscape "landscape" "System Landscape" {
            include *
            autoLayout
        }

        systemContext system "context" "System Context" {
            include *
            autoLayout
        }

        container system "containers" "Container View" {
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
            relationship "Async" {
                style Dashed
            }
            relationship "Relationship" {
                color "#707070"
                routing Orthogonal
            }
        }
    }
}
