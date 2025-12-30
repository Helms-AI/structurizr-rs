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
        customer -> webApp "Accesses banking via" "HTTPS" {
            tags "User Interaction, Frontend"
            properties {
                "authentication" "Session-based"
                "encryption" "TLS 1.3"
            }
            perspectives {
                "Security" "Primary attack surface - requires strong session management"
                "Performance" "CDN-cached static assets, sub-100ms response target"
                "Business" "Primary customer touchpoint for digital banking"
            }
        }
        customer -> mobileApp "Uses banking app on" "HTTPS" {
            tags "User Interaction, Mobile"
            properties {
                "authentication" "Biometric + PIN"
                "encryption" "TLS 1.3 with certificate pinning"
            }
            perspectives {
                "Security" "Device binding and biometric auth required"
                "UX" "Touch-optimized interface with offline support"
            }
        }

        // Internal Staff
        securityOfficer -> auditLog "Reviews transaction logs in" {
            tags "Audit, Compliance"
            perspectives {
                "Compliance" "SOX and PCI-DSS audit trail access"
            }
        }

        // Frontend to Gateway
        webApp -> apiGateway "Makes API calls to" "REST/HTTPS" {
            tags "API, Synchronous"
            properties {
                "rateLimit" "1000 req/min per user"
                "timeout" "30s"
            }
            perspectives {
                "Performance" "Edge caching for read operations"
                "Security" "JWT token validation at gateway"
            }
        }
        mobileApp -> apiGateway "Makes API calls to" "REST/HTTPS" {
            tags "API, Synchronous, Mobile"
            properties {
                "rateLimit" "500 req/min per device"
                "compression" "gzip"
            }
        }

        // Gateway to Services
        apiGateway -> authService "Authenticates via" "gRPC" {
            tags "Authentication, Security Critical"
            properties {
                "protocol" "gRPC"
                "timeout" "5s"
            }
            perspectives {
                "Security" "All requests require valid auth token"
                "Performance" "Sub-10ms auth validation target"
            }
        }
        apiGateway -> accountService "Routes account requests to" "gRPC" {
            tags "API, Account Management"
            properties {
                "circuitBreaker" "enabled"
            }
        }
        apiGateway -> transactionService "Routes transfers to" "gRPC" {
            tags "API, Financial, Critical Path"
            properties {
                "circuitBreaker" "enabled"
                "retryPolicy" "exponential backoff"
            }
            perspectives {
                "Business" "Revenue-critical transaction processing"
                "Compliance" "All transfers logged for regulatory audit"
            }
        }

        // Service to Service
        authService -> database "Stores user credentials in" "SQL" {
            tags "Data, Security Critical"
            properties {
                "encryption" "AES-256 at rest"
                "connectionPool" "100"
            }
            perspectives {
                "Security" "Credentials hashed with bcrypt, salted"
                "Compliance" "GDPR compliant data retention"
            }
        }
        authService -> cache "Manages sessions in" "Redis Protocol" {
            tags "Session, Cache"
            properties {
                "ttl" "30 minutes"
                "evictionPolicy" "LRU"
            }
            perspectives {
                "Performance" "In-memory session lookup < 1ms"
                "Security" "Sessions invalidated on logout"
            }
        }
        authService -> otpProvider "Sends OTP codes via" "REST" {
            tags "MFA, External Integration, Async"
            properties {
                "deliveryMethod" "SMS/Email"
                "otpValidity" "5 minutes"
                "maxRetries" "3"
            }
            perspectives {
                "Security" "Second factor authentication delivery"
                "Reliability" "99.9% delivery SLA required"
                "Cost" "$0.01 per SMS, $0.001 per email"
            }
        }
        accountService -> database "Reads account data from" "SQL" {
            tags "Data, Read Operation"
            properties {
                "cacheEnabled" "true"
                "cacheTTL" "60s"
            }
            perspectives {
                "Performance" "Read replicas for scalability"
            }
        }
        transactionService -> accountService "Validates balances with" "gRPC" {
            tags "Validation, Internal API"
            properties {
                "timeout" "2s"
            }
            perspectives {
                "Business" "Prevents overdraft transactions"
            }
        }
        transactionService -> fraudDetection "Checks transactions with" "REST" {
            tags "Security, ML, External Integration"
            properties {
                "modelVersion" "v2.3"
                "threshold" "0.85"
                "timeout" "3s"
            }
            perspectives {
                "Security" "ML-based anomaly detection"
                "Business" "Reduces fraud losses by 94%"
                "Performance" "P99 latency < 200ms"
            }
        }
        transactionService -> coreBanking "Executes transfers via" "SOAP/XML" {
            tags "Legacy, Financial, Critical Path"
            properties {
                "protocol" "SOAP 1.2"
                "retryPolicy" "at-least-once"
                "timeout" "30s"
            }
            perspectives {
                "Technical" "Legacy mainframe integration"
                "Business" "Source of truth for account balances"
                "Risk" "Single point of failure - DR site required"
            }
        }
        transactionService -> auditLog "Records transactions to" "SQL" {
            tags "Audit, Compliance, Immutable"
            properties {
                "retention" "7 years"
                "immutable" "true"
            }
            perspectives {
                "Compliance" "SOX and PCI-DSS audit trail"
                "Legal" "Immutable record for dispute resolution"
            }
        }
        notificationService -> customer "Sends alerts to" "Email/SMS" {
            tags "Notification, Async, Customer Communication"
            properties {
                "channels" "Email, SMS, Push"
                "priority" "High for transactions"
            }
            perspectives {
                "UX" "Real-time transaction confirmations"
                "Security" "Fraud alert notifications"
            }
        }
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
