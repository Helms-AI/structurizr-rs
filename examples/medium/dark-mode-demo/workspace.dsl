workspace "Dark Mode Demo" "Demonstrates dark mode theming and icon features" {

    !docs "docs"
    !adrs "adrs"

    model {
        customer = person "Customer" "A customer of the bank" {
            tags "Person"
        }

        bankingSystem = softwareSystem "Internet Banking System" "Allows customers to view and manage their accounts" {
            tags "Primary System"

                webApp = container "Web Application" "Delivers static content and the single page application" "React, TypeScript" {
                    tags "Web Browser"
                }

                apiGateway = container "API Gateway" "Routes and authenticates API requests" "Node.js, Express" {
                    tags "API"
                }

                authService = container "Authentication Service" "Handles user authentication and JWT tokens" "Node.js" {
                    tags "Microservice"
                }

                accountsService = container "Accounts Service" "Manages customer accounts and balances" "Java, Spring Boot" {
                    tags "Microservice"
                }

                paymentsService = container "Payments Service" "Processes payments and transfers" "Python, FastAPI" {
                    tags "Microservice"
                }

                notificationService = container "Notification Service" "Sends emails and push notifications" "Go" {
                    tags "Microservice"
                }

                database = container "Database" "Stores account data and transactions" "PostgreSQL" {
                    tags "Database"
                }

                cache = container "Cache" "Session and API response caching" "Redis" {
                    tags "Database"
                }

                messageQueue = container "Message Queue" "Async message processing" "RabbitMQ" {
                    tags "Queue"
                }
            }

        externalPayments = softwareSystem "External Payment Gateway" "Third-party payment processor" {
            tags "External System"
        }

        emailSystem = softwareSystem "Email System" "Sendgrid email delivery" {
            tags "External System"
        }

        // Relationships
        customer -> webApp "Views account information using" "HTTPS"
        webApp -> apiGateway "Makes API calls to" "HTTPS/JSON"

        apiGateway -> authService "Validates tokens with" "gRPC"
        apiGateway -> accountsService "Retrieves accounts from" "gRPC"
        apiGateway -> paymentsService "Submits payments to" "gRPC"

        authService -> database "Reads/writes user data" "SQL"
        authService -> cache "Caches sessions in" "Redis Protocol"

        accountsService -> database "Reads/writes account data" "SQL"
        accountsService -> messageQueue "Publishes events to" "AMQP"

        paymentsService -> database "Reads/writes transaction data" "SQL"
        paymentsService -> externalPayments "Processes payments via" "REST API"
        paymentsService -> messageQueue "Publishes events to" "AMQP"

        notificationService -> messageQueue "Consumes events from" "AMQP"
        notificationService -> emailSystem "Sends emails via" "SMTP"
    }

    views {
        systemContext bankingSystem "SystemContext" "System context diagram for Internet Banking" {
            include *
            autoLayout
            background "#1a1a1a"
        }

        container bankingSystem "Containers" "Container diagram for Internet Banking" {
            include *
            autoLayout
            background "#1a1a1a"
        }

        styles {
            element "Person" {
                shape Person
                background "#4A90D9"
                color "#ffffff"
            }
            element "Primary System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "External System" {
                background "#999999"
                color "#ffffff"
            }
            element "Web Browser" {
                shape WebBrowser
                background "#438dd5"
                color "#ffffff"
            }
            element "API" {
                shape RoundedBox
                background "#438dd5"
                color "#ffffff"
            }
            element "Microservice" {
                shape Hexagon
                background "#438dd5"
                color "#ffffff"
            }
            element "Database" {
                shape Cylinder
                background "#438dd5"
                color "#ffffff"
            }
            element "Queue" {
                shape Pipe
                background "#438dd5"
                color "#ffffff"
            }
            relationship "Relationship" {
                color "#707070"
                style Dashed
                routing Curved
            }
        }

        themes "https://static.structurizr.com/themes/default/theme.json"
    }

}
