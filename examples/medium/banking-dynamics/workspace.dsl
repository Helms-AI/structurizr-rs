workspace "Online Banking Platform" "Secure online banking with multi-factor authentication" {

    !docs "docs"
    !adrs "adrs"
    !impliedRelationships true

    !const BANK_NAME "SecureBank"
    !const PRIMARY_COLOR "#1a365d"
    !const SECONDARY_COLOR "#2b6cb0"

    model {
        // Actors
        customer = person "Bank Customer" "A customer with an online banking account" {
            tags "Customer"
        }
        securityOfficer = person "Security Officer" "Monitors for suspicious activity" {
            tags "Internal"
        }

        // External Systems
        fraudDetection = softwareSystem "Fraud Detection Service" "ML-based fraud scoring and pattern detection" {
            tags "External"
        }
        otpProvider = softwareSystem "OTP Provider" "SMS and email one-time password delivery" {
            tags "External"
        }
        coreBanking = softwareSystem "Core Banking System" "Legacy core banking ledger" {
            tags "External,Legacy"
        }

        // Main Banking Platform
        bankingPlatform = softwareSystem "${BANK_NAME} Online Banking" "Digital banking platform for customers" {
            webApp = container "Web Application" "Customer web portal for banking" "React/TypeScript" {
                tags "Frontend,Web"
            }
            mobileApp = container "Mobile App" "iOS and Android banking app" "React Native" {
                tags "Frontend,Mobile"
            }
            apiGateway = container "API Gateway" "API routing, rate limiting, and request validation" "Kong" {
                tags "Gateway"
            }
            authService = container "Auth Service" "Authentication and MFA management" "Java/Spring Security" {
                tags "Security,API"
            }
            accountService = container "Account Service" "Account management and balance queries" "Java/Spring Boot" {
                tags "API"
            }
            transactionService = container "Transaction Service" "Transfer processing and validation" "Java/Spring Boot" {
                tags "API"
            }
            notificationService = container "Notification Service" "Alerts, confirmations, and notifications" "Node.js" {
                tags "API"
            }
            auditLog = container "Audit Log" "Immutable transaction audit trail" "PostgreSQL" {
                tags "Database,Security"
            }
            database = container "Database" "Core banking data and user accounts" "PostgreSQL" {
                tags "Database"
            }
            cache = container "Session Cache" "Authenticated session storage" "Redis" {
                tags "Cache"
            }
        }

        // Customer Relationships
        customer -> webApp "Accesses banking via" "HTTPS"
        customer -> mobileApp "Uses banking app on" "HTTPS"

        // Internal Staff
        securityOfficer -> auditLog "Reviews transaction logs in"

        // Frontend to Gateway
        webApp -> apiGateway "Makes API calls to" "REST/HTTPS"
        mobileApp -> apiGateway "Makes API calls to" "REST/HTTPS"

        // Gateway to Services
        apiGateway -> authService "Authenticates via"
        apiGateway -> accountService "Routes account requests to"
        apiGateway -> transactionService "Routes transfers to"

        // Service to Service
        authService -> database "Stores user credentials in" "SQL"
        authService -> cache "Manages sessions in" "Redis Protocol"
        authService -> otpProvider "Sends OTP codes via"
        accountService -> database "Reads account data from" "SQL"
        transactionService -> accountService "Validates balances with"
        transactionService -> fraudDetection "Checks transactions with" "REST"
        transactionService -> coreBanking "Executes transfers via" "SOAP/XML"
        transactionService -> auditLog "Records transactions to" "SQL"
        notificationService -> customer "Sends alerts to" "Email/SMS"
    }

    views {
        systemContext bankingPlatform "SystemContext" "Online banking system context" {
            include *
            autoLayout tb
        }

        container bankingPlatform "Containers" "Container architecture" {
            include *
            autoLayout tb
        }

        // Dynamic view: Login authentication flow
        dynamic bankingPlatform "LoginFlow" "Customer authentication with MFA" {
            customer -> webApp "Enters username and password"
            webApp -> apiGateway "Submits credentials"
            apiGateway -> authService "Validates credentials"
            authService -> database "Verifies user account"
            authService -> otpProvider "Sends OTP to registered device"
            customer -> webApp "Enters OTP code"
            webApp -> authService "Validates OTP"
            authService -> cache "Creates authenticated session"
            autoLayout lr
        }

        // Dynamic view: Fund transfer flow
        dynamic bankingPlatform "TransferFlow" "Money transfer between accounts" {
            customer -> webApp "Initiates transfer request"
            webApp -> apiGateway "Submits transfer details"
            apiGateway -> authService "Validates session token"
            apiGateway -> transactionService "Creates pending transaction"
            transactionService -> fraudDetection "Checks for fraud patterns"
            transactionService -> accountService "Validates source account balance"
            accountService -> database "Reads account data"
            transactionService -> coreBanking "Executes fund transfer"
            transactionService -> auditLog "Records transaction audit"
            notificationService -> customer "Sends transfer confirmation"
            autoLayout lr
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Customer" {
                background "#2b6cb0"
            }
            element "Internal" {
                background "#553c9a"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "External" {
                background "#999999"
            }
            element "Legacy" {
                background "#666666"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Frontend" {
                shape WebBrowser
            }
            element "Mobile" {
                shape MobileDevicePortrait
            }
            element "Gateway" {
                shape Hexagon
                background "#68d391"
                color "#1a202c"
            }
            element "API" {
                shape Hexagon
            }
            element "Database" {
                shape Cylinder
            }
            element "Cache" {
                shape Cylinder
                background "#ed8936"
            }
            element "Security" {
                background "#e53e3e"
            }
            relationship "Relationship" {
                color "#707070"
                thickness 2
            }
        }
    }
}
