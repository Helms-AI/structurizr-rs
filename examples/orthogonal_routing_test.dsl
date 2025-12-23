workspace "Orthogonal Routing Test" "Testing orthogonal edge routing with minimal crossings" {

    model {
        user = person "User" "A user of the system"

        system = softwareSystem "Banking System" "Core banking system" {
            webapp = container "Web Application" "Frontend web application" "React"
            api = container "API Gateway" "API gateway for all services" "Spring Boot"
            auth = container "Auth Service" "Authentication and authorization" "Node.js"
            accounts = container "Accounts Service" "Account management" "Java"
            payments = container "Payments Service" "Payment processing" "Python"
            db = container "Database" "Main database" "PostgreSQL" {
                tags "Database"
            }
        }

        external = softwareSystem "External Payment Provider" "Third-party payment provider" {
            tags "External"
        }

        user -> webapp "Uses"
        webapp -> api "Makes API calls to"
        api -> auth "Authenticates via"
        api -> accounts "Manages accounts via"
        api -> payments "Processes payments via"
        auth -> db "Reads/writes user data"
        accounts -> db "Reads/writes account data"
        payments -> db "Reads/writes transaction data"
        payments -> external "Sends payments to"
    }

    views {
        container system "ContainerView" "Container view of the banking system" {
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
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Database" {
                shape Cylinder
                background "#438dd5"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
                color "#ffffff"
            }
            relationship "Relationship" {
                color "#707070"
            }
        }
    }
}
