workspace "FinTech Payment Platform" "A comprehensive payment processing platform for merchants and consumers with compliance and fraud detection" {
    !const PLATFORM_NAME "PayFlow"
    !const COMPANY_NAME "FinTech Solutions Inc"
    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        merchant = person "Merchant" "Business owner who accepts payments through the platform for goods and services"
        consumer = person "Consumer" "End user making payments for goods and services through the platform"
        complianceOfficer = person "Compliance Officer" "Internal staff who monitors transactions for regulatory compliance and investigates suspicious activity"

        paymentPlatform = softwareSystem "Payment Platform" "Core payment processing platform enabling merchants to accept payments from consumers with real-time fraud detection and compliance monitoring" {
            merchantPortal = container "Merchant Portal" "Web application for merchants to manage their account, view transactions, and configure payment settings" "React SPA"
            consumerApp = container "Consumer App" "Mobile and web application for consumers to make payments and manage payment methods" "React Native / React"
            apiGateway = container "API Gateway" "Handles authentication, rate limiting, and routing for all API requests" "Kong Gateway"
            transactionProcessor = container "Transaction Processor" "Orchestrates payment transactions, coordinates with external systems, and ensures transaction integrity" "Spring Boot" {
                paymentController = component "Payment Controller" "REST API endpoints for payment operations" "Spring MVC Controller"
                transactionOrchestrator = component "Transaction Orchestrator" "Coordinates multi-step payment workflows" "Spring Service"
                fraudCheckService = component "Fraud Check Service" "Integrates with fraud detection systems" "Spring Service"
                paymentValidator = component "Payment Validator" "Validates payment requests and business rules" "Spring Service"
                bankIntegration = component "Bank Integration" "Communicates with bank networks for fund transfers" "Spring Service"
                transactionStateMachine = component "Transaction State Machine" "Manages transaction lifecycle states" "Spring State Machine"
            }
            ledgerService = container "Ledger Service" "Maintains accurate financial ledger, handles double-entry bookkeeping, and reconciliation" "Java Microservice"
            notificationService = container "Notification Service" "Sends real-time notifications via email, SMS, and push notifications" "Node.js Microservice"
            complianceEngine = container "Compliance Engine" "Monitors transactions for AML/KYC compliance and generates regulatory reports" "Python Service"
            primaryDatabase = container "Primary Database" "Stores transaction data, user accounts, and merchant configurations" "PostgreSQL" {
                tags "Database"
            }
            auditLog = container "Audit Log" "Immutable audit trail for all system operations and compliance" "Event Store" {
                tags "Database"
            }
            cache = container "Cache Layer" "High-performance cache for frequently accessed data and session management" "Redis" {
                tags "Database"
            }
        }

        bankNetwork = softwareSystem "Bank Network" "External banking networks and payment processors including ACH, Wire, and Card Networks like Visa and Mastercard" {
            tags "External"
        }

        fraudDetection = softwareSystem "Fraud Detection Service" "Machine learning-based fraud detection and risk scoring service for real-time transaction analysis" {
            tags "External"
        }

        kycProvider = softwareSystem "KYC Provider" "Third-party identity verification and Know Your Customer service for regulatory compliance" {
            tags "External"
        }

        reportingSystem = softwareSystem "Reporting & Analytics" "Business intelligence and analytics platform for transaction insights and merchant reporting" {
            tags "External"
        }

        merchant -> merchantPortal "Manages account, views transactions, configures payment settings" "HTTPS"
        consumer -> consumerApp "Makes payments, manages payment methods" "HTTPS"
        complianceOfficer -> complianceEngine "Reviews flagged transactions, generates compliance reports" "HTTPS"

        merchantPortal -> apiGateway "Makes API calls to" "JSON/HTTPS"
        consumerApp -> apiGateway "Makes API calls to" "JSON/HTTPS"

        apiGateway -> transactionProcessor "Routes payment requests to" "JSON/HTTPS"
        apiGateway -> ledgerService "Routes ledger queries to" "JSON/HTTPS"
        apiGateway -> complianceEngine "Routes compliance queries to" "JSON/HTTPS"

        transactionProcessor -> ledgerService "Records transactions in" "gRPC"
        transactionProcessor -> notificationService "Triggers notifications via" "Message Queue"
        transactionProcessor -> complianceEngine "Validates compliance rules via" "REST API"
        transactionProcessor -> primaryDatabase "Reads from and writes to" "JDBC"
        transactionProcessor -> cache "Caches session and transaction data in" "Redis Protocol"
        transactionProcessor -> bankNetwork "Initiates fund transfers via" "ISO 8583 / API"
        transactionProcessor -> fraudDetection "Performs fraud checks via" "REST API"

        ledgerService -> primaryDatabase "Stores ledger entries in" "JDBC"
        ledgerService -> auditLog "Writes immutable audit records to" "Event Sourcing"

        notificationService -> consumer "Sends payment confirmations to" "Email/SMS/Push"
        notificationService -> merchant "Sends transaction alerts to" "Email/SMS"

        complianceEngine -> kycProvider "Verifies customer identity via" "REST API"
        complianceEngine -> auditLog "Reads audit trail from" "Event Sourcing"
        complianceEngine -> primaryDatabase "Queries transaction data from" "JDBC"

        paymentPlatform -> reportingSystem "Exports transaction data to" "Data Pipeline"

        paymentController -> transactionOrchestrator "Delegates payment requests to" "Java Method Call"
        transactionOrchestrator -> paymentValidator "Validates payment via" "Java Method Call"
        transactionOrchestrator -> fraudCheckService "Checks fraud risk via" "Java Method Call"
        transactionOrchestrator -> bankIntegration "Processes payment via" "Java Method Call"
        transactionOrchestrator -> transactionStateMachine "Updates transaction state via" "Java Method Call"
        fraudCheckService -> fraudDetection "Calls external fraud service" "REST API"
        bankIntegration -> bankNetwork "Communicates with banks" "ISO 8583"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of the FinTech payment ecosystem showing all actors and systems" {
            include *
            autoLayout tb
        }

        systemContext paymentPlatform "SystemContext" "System context diagram showing the Payment Platform and its relationships with users and external systems" {
            include *
            autoLayout tb
        }

        container paymentPlatform "Containers" "Container diagram showing the internal architecture of the Payment Platform" {
            include *
            autoLayout tb
        }

        component transactionProcessor "TransactionProcessorComponents" "Component diagram showing the internal structure of the Transaction Processor" {
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
            element "Database" {
                shape Cylinder
                background "#438dd5"
                color "#ffffff"
            }
            element "Component" {
                background "#85bbf0"
                color "#000000"
            }
            relationship "Relationship" {
                color "#707070"
                routing Orthogonal
            }
        }
    }
}
