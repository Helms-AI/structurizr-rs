workspace "FreshMart Payment Gateway" "Secure payment processing platform with PCI compliance and multi-acquirer support" {

    !const CORP_COLOR "#00539C"
    !const SECURITY_COLOR "#DC143C"
    !const EXTERNAL_COLOR "#999999"
    !const SUCCESS_COLOR "#2ECC71"
    !const WARNING_COLOR "#F39C12"
    !const PCI_ZONE "#8B0000"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People and External Systems
        # ========================================

        customer = person "FreshMart Customer" "Shopper making purchases at FreshMart stores" {
            tags "Customer,Person"
            properties {
                "persona" "Regular shopper"
                "payment_methods" "Credit, Debit, Mobile Wallet, BNPL"
            }
        }

        cashier = person "Store Cashier" "FreshMart associate processing transactions" {
            tags "Associate,Person"
            properties {
                "training_level" "PCI-DSS certified"
                "system_access" "POS Terminal only"
            }
        }

        fraudAnalyst = person "Fraud Analyst" "Security team member investigating suspicious transactions" {
            tags "Person,Security"
            properties {
                "clearance" "Level 2"
                "tools" "Fraud Console, SIEM"
            }
        }

        complianceOfficer = person "Compliance Officer" "Ensures PCI-DSS and regulatory compliance" {
            tags "Person"
            properties {
                "certifications" "PCI-ISA, CISA"
                "audit_frequency" "Quarterly"
            }
        }

        # ========================================
        # External Payment Systems
        # ========================================

        group "Payment Networks" {
            visaNetwork = softwareSystem "Visa Network" "Global payment processing network" {
                tags "External,Partner"
                properties {
                    "api_version" "v2.1"
                    "sla" "99.999%"
                }
            }

            mastercardNetwork = softwareSystem "Mastercard Network" "Global payment processing network" {
                tags "External,Partner"
                properties {
                    "api_version" "v3.0"
                    "sla" "99.999%"
                }
            }

            amexNetwork = softwareSystem "Amex Network" "American Express payment network" {
                tags "External,Partner"
                properties {
                    "api_version" "v1.5"
                    "sla" "99.99%"
                }
            }
        }

        group "Alternative Payment Providers" {
            applePayGateway = softwareSystem "Apple Pay" "Mobile wallet payment service" {
                tags "External,Partner,Mobile"
                properties {
                    "integration" "SDK"
                    "tokenization" "Device-based"
                }
            }

            googlePayGateway = softwareSystem "Google Pay" "Mobile wallet payment service" {
                tags "External,Partner,Mobile"
                properties {
                    "integration" "API"
                    "tokenization" "Cloud-based"
                }
            }

            paypalGateway = softwareSystem "PayPal" "Digital payment platform" {
                tags "External,Partner"
                properties {
                    "integration" "REST API"
                    "settlement" "T+1"
                }
            }

            klarnaGateway = softwareSystem "Klarna" "Buy now pay later service" {
                tags "External,Partner"
                properties {
                    "integration" "Webhook"
                    "risk_model" "ML-based"
                }
            }
        }

        bankingPartner = softwareSystem "Banking Partner" "Primary banking relationship for settlement" {
            tags "External,Partner"
            properties {
                "settlement_time" "T+2"
                "account_type" "Merchant"
            }
        }

        # ========================================
        # Internal Systems
        # ========================================

        posSystem = softwareSystem "POS Terminal System" "Point of Sale system in stores" {
            tags "Software System,Core"
            properties {
                "deployment" "Store Edge"
                "instances" "25,000+"
                "perspective_business" "Critical for checkout operations"
                "perspective_technical" "Edge-deployed, resilient architecture"
                "perspective_security" "PCI-DSS compliant terminal software"
            }
        }

        inventorySystem = softwareSystem "Inventory Platform" "Real-time inventory management" {
            tags "Software System,Core"
            properties {
                "integration" "Event Stream"
            }
        }

        loyaltySystem = softwareSystem "Loyalty & Rewards" "Customer loyalty program management" {
            tags "Software System"
            properties {
                "points_processing" "Real-time"
            }
        }

        fraudDetectionSystem = softwareSystem "Fraud Detection Platform" "ML-based fraud detection and prevention" {
            tags "Software System,ML,Security"
            properties {
                "ml_models" "12"
                "detection_rate" "99.7%"
                "false_positive_rate" "0.3%"
                "perspective_security" "Real-time threat detection using ML"
                "perspective_operations" "Automated fraud scoring and alerting"
                "perspective_business" "Reduces chargebacks by 40%"
            }
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Business intelligence and reporting" {
            tags "Software System,Data"
            properties {
                "data_retention" "7 years"
            }
        }

        auditSystem = softwareSystem "Audit & Compliance System" "Audit trail and compliance reporting" {
            tags "Software System,Security"
            properties {
                "retention_period" "7 years"
                "encryption" "AES-256"
            }
        }

        # ========================================
        # Payment Gateway System (Main Focus)
        # ========================================

        paymentGateway = softwareSystem "Payment Gateway" "Central payment processing platform" {
            tags "Software System,Core,Security"

            properties {
                "transactions_per_day" "5M+"
                "peak_tps" "10,000"
                "availability_target" "99.999%"
                "compliance" "PCI-DSS Level 1"
                "encryption" "End-to-end"
                "perspective_business" "Processes $10B+ annually with 99.999% uptime"
                "perspective_technical" "Microservices architecture with multi-region deployment"
                "perspective_security" "PCI-DSS Level 1 certified with end-to-end encryption"
                "perspective_operations" "Auto-scaling, self-healing infrastructure"
                "perspective_compliance" "Meets PCI-DSS, PA-DSS, SOC2, ISO 27001 requirements"
            }

            # ========================================
            # Containers within Payment Gateway
            # ========================================

            apiGateway = container "API Gateway" "Manages all inbound payment requests" "Kong Enterprise" {
                tags "Container,API"
                properties {
                    "rate_limiting" "10K req/sec"
                    "authentication" "mTLS + OAuth2"
                    "caching" "Redis"
                    "perspective_security" "Rate limiting, DDoS protection, mTLS"
                    "perspective_technical" "Kong Enterprise with custom plugins"
                    "perspective_operations" "Auto-scaling based on request volume"
                }
            }

            paymentOrchestrator = container "Payment Orchestrator" "Orchestrates payment flow across services" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "framework" "Spring Boot 3.0"
                    "messaging" "Apache Kafka"
                    "database" "PostgreSQL"
                }

                # Components within Payment Orchestrator
                transactionManager = component "Transaction Manager" "Manages transaction lifecycle" "Spring Component" {
                    tags "Component"
                    properties {
                        "pattern" "Saga"
                        "timeout" "30 seconds"
                    }
                }

                routingEngine = component "Routing Engine" "Routes to optimal acquirer" "Spring Component" {
                    tags "Component,ML"
                    properties {
                        "algorithm" "ML-based"
                        "factors" "Cost, Success Rate, Latency"
                    }
                }

                retryManager = component "Retry Manager" "Handles failed transaction retries" "Spring Component" {
                    tags "Component"
                    properties {
                        "max_retries" "3"
                        "backoff" "Exponential"
                    }
                }

                responseProcessor = component "Response Processor" "Processes acquirer responses" "Spring Component" {
                    tags "Component"
                    properties {
                        "response_codes" "150+"
                    }
                }
            }

            tokenizationVault = container "Tokenization Vault" "Secure storage of payment tokens" "HashiCorp Vault" {
                tags "Container,Security,PCI,Database"
                properties {
                    "encryption" "AES-256-GCM"
                    "key_rotation" "Monthly"
                    "hsm_backed" "true"
                    "compliance" "PCI-DSS Level 1"
                    "perspective_security" "HSM-backed encryption with key rotation"
                    "perspective_compliance" "PCI-DSS certified token vault"
                    "perspective_technical" "HashiCorp Vault with HSM integration"
                }

                tokenGenerator = component "Token Generator" "Generates secure tokens" "Vault Plugin" {
                    tags "Component,Security"
                    properties {
                        "algorithm" "Format-preserving encryption"
                        "entropy" "256 bits"
                    }
                }

                tokenMapper = component "Token Mapper" "Maps tokens to card data" "Vault Core" {
                    tags "Component,Security"
                    properties {
                        "index_encryption" "true"
                    }
                }

                keyManager = component "Key Manager" "Manages encryption keys" "Vault KMS" {
                    tags "Component,Security"
                    properties {
                        "rotation_period" "30 days"
                        "key_derivation" "PBKDF2"
                    }
                }
            }

            acquirerGateway = container "Acquirer Gateway" "Manages connections to payment acquirers" "Go" {
                tags "Container,Integration"
                properties {
                    "protocols" "ISO 8583, REST, SOAP"
                    "connections" "50+"
                    "circuit_breaker" "Hystrix"
                }

                protocolAdapter = component "Protocol Adapter" "Adapts various payment protocols" "Go Module" {
                    tags "Component"
                    properties {
                        "supported_protocols" "ISO 8583, REST, SOAP"
                    }
                }

                connectionPool = component "Connection Pool" "Manages acquirer connections" "Go Module" {
                    tags "Component"
                    properties {
                        "pool_size" "100"
                        "timeout" "30s"
                    }
                }

                circuitBreaker = component "Circuit Breaker" "Prevents cascade failures" "Hystrix" {
                    tags "Component"
                    properties {
                        "failure_threshold" "50%"
                        "recovery_time" "30s"
                    }
                }
            }

            fraudEngine = container "Fraud Detection Engine" "Real-time fraud scoring" "Python/TensorFlow" {
                tags "Container,ML,Security"
                properties {
                    "ml_framework" "TensorFlow"
                    "model_count" "12"
                    "inference_time" "<100ms"
                    "accuracy" "99.7%"
                    "perspective_security" "ML-based fraud detection with 99.7% accuracy"
                    "perspective_technical" "TensorFlow Serving with GPU acceleration"
                    "perspective_business" "Reduces fraud losses by 60%"
                }

                riskScorer = component "Risk Scorer" "Calculates transaction risk score" "ML Model" {
                    tags "Component,ML"
                    properties {
                        "features" "200+"
                        "model_type" "XGBoost"
                    }
                }

                velocityChecker = component "Velocity Checker" "Checks transaction velocity" "Python Service" {
                    tags "Component"
                    properties {
                        "time_windows" "1m, 5m, 1h, 24h"
                    }
                }

                deviceProfiler = component "Device Profiler" "Profiles payment devices" "ML Model" {
                    tags "Component,ML"
                    properties {
                        "fingerprinting" "Advanced"
                    }
                }

                behaviorAnalyzer = component "Behavior Analyzer" "Analyzes customer behavior" "ML Model" {
                    tags "Component,ML"
                    properties {
                        "model_type" "LSTM"
                        "sequence_length" "100"
                    }
                }
            }

            settlementEngine = container "Settlement Engine" "Manages settlement and reconciliation" "Java/Spring Batch" {
                tags "Container"
                properties {
                    "batch_framework" "Spring Batch"
                    "schedule" "Daily at 2 AM"
                    "reconciliation_accuracy" "100%"
                }

                batchProcessor = component "Batch Processor" "Processes settlement batches" "Spring Batch Job" {
                    tags "Component"
                    properties {
                        "chunk_size" "1000"
                        "parallel_threads" "10"
                    }
                }

                reconciler = component "Reconciliation Engine" "Reconciles transactions" "Spring Component" {
                    tags "Component"
                    properties {
                        "matching_algorithm" "Three-way"
                    }
                }

                reportGenerator = component "Report Generator" "Generates settlement reports" "JasperReports" {
                    tags "Component"
                    properties {
                        "formats" "PDF, CSV, XML"
                    }
                }
            }

            eventStream = container "Event Stream" "Payment event streaming platform" "Apache Kafka" {
                tags "Container,Queue"
                properties {
                    "topics" "50+"
                    "throughput" "1M msg/sec"
                    "retention" "7 days"
                    "replication" "3"
                    "perspective_technical" "Kafka cluster with 1M msg/sec throughput"
                    "perspective_operations" "Multi-AZ deployment with 3x replication"
                }
            }

            monitoringService = container "Monitoring Service" "Payment monitoring and alerting" "Prometheus/Grafana" {
                tags "Container"
                properties {
                    "metrics" "500+"
                    "dashboards" "25"
                    "alerts" "150"
                    "retention" "90 days"
                }
            }

            database = container "Payment Database" "Transactional database for payments" "PostgreSQL" {
                tags "Container,Database"
                properties {
                    "version" "15"
                    "replication" "Master-Slave"
                    "backup" "Continuous"
                    "encryption" "TDE"
                }
            }

            cache = container "Payment Cache" "High-speed payment data cache" "Redis Enterprise" {
                tags "Container,Cache"
                properties {
                    "memory" "256GB"
                    "persistence" "AOF"
                    "cluster_size" "6 nodes"
                }
            }

            complianceService = container "Compliance Service" "PCI compliance and reporting" "Java/Spring Boot" {
                tags "Container,Security"
                properties {
                    "standards" "PCI-DSS, PA-DSS"
                    "audit_frequency" "Quarterly"
                }

                pciScanner = component "PCI Scanner" "Scans for PCI compliance" "Spring Component" {
                    tags "Component,Security"
                    properties {
                        "scan_frequency" "Daily"
                        "vulnerability_db" "CVE, NVD"
                    }
                }

                auditLogger = component "Audit Logger" "Logs all payment activities" "Log4j2" {
                    tags "Component,Security"
                    properties {
                        "retention" "7 years"
                        "tamper_proof" "true"
                    }
                }
            }
        }

        # ========================================
        # Relationships - External to Gateway
        # ========================================

        customer -> posSystem "Makes payment" {
            tags "Sync"
            properties {
                "channel" "In-store"
            }
        }

        cashier -> posSystem "Processes transaction" {
            tags "Sync"
            properties {
                "frequency" "300/day"
            }
        }

        posSystem -> paymentGateway "Submits payment request" {
            tags "Sync,Critical,Encrypted"
            properties {
                "protocol" "HTTPS/TLS 1.3"
                "timeout" "30s"
                "retry" "3"
            }
            perspectives {
                "Security" "End-to-end encrypted with mTLS"
                "Technical" "REST API with circuit breaker"
                "Operations" "99.99% availability SLA"
            }
        }

        paymentGateway -> visaNetwork "Process Visa payment" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "ISO 8583"
                "latency" "<500ms"
            }
            perspectives {
                "Technical" "ISO 8583 over secure VPN"
                "Security" "Hardware-encrypted connection"
                "Business" "40% of transaction volume"
            }
        }

        paymentGateway -> mastercardNetwork "Process Mastercard payment" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "ISO 8583"
                "latency" "<500ms"
            }
            perspectives {
                "Technical" "ISO 8583 over secure VPN"
                "Security" "Hardware-encrypted connection"
                "Business" "35% of transaction volume"
            }
        }

        paymentGateway -> amexNetwork "Process Amex payment" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "REST API"
                "latency" "<600ms"
            }
        }

        paymentGateway -> applePayGateway "Process Apple Pay" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "REST API"
                "tokenization" "Device-based"
            }
        }

        paymentGateway -> googlePayGateway "Process Google Pay" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "REST API"
                "tokenization" "Cloud-based"
            }
        }

        paymentGateway -> paypalGateway "Process PayPal payment" {
            tags "Sync,External"
            properties {
                "protocol" "REST API"
                "webhook" "true"
            }
        }

        paymentGateway -> klarnaGateway "Process BNPL payment" {
            tags "Sync,External"
            properties {
                "protocol" "REST API"
                "approval_rate" "85%"
            }
        }

        paymentGateway -> fraudDetectionSystem "Check fraud score" {
            tags "Sync,Critical,Internal"
            properties {
                "latency" "<100ms"
                "threshold" "0.7"
            }
            perspectives {
                "Security" "Real-time fraud scoring for every transaction"
                "Technical" "gRPC with protobuf serialization"
                "Business" "Prevents $50M+ annual fraud losses"
            }
        }

        paymentGateway -> bankingPartner "Settlement transfer" {
            tags "Batch,External,Encrypted"
            properties {
                "frequency" "Daily"
                "protocol" "SFTP"
                "format" "NACHA"
            }
        }

        paymentGateway -> inventorySystem "Update inventory" {
            tags "Async,Internal"
            properties {
                "protocol" "Kafka"
                "topic" "payment.completed"
            }
        }

        paymentGateway -> loyaltySystem "Process loyalty points" {
            tags "Async,Internal"
            properties {
                "protocol" "Kafka"
                "topic" "payment.loyalty"
            }
        }

        paymentGateway -> analyticsSystem "Send payment data" {
            tags "Async,Internal"
            properties {
                "protocol" "Kafka"
                "topic" "payment.analytics"
            }
        }

        paymentGateway -> auditSystem "Log compliance data" {
            tags "Async,Critical,Internal"
            properties {
                "protocol" "Kafka"
                "retention" "7 years"
            }
        }

        fraudAnalyst -> fraudDetectionSystem "Investigates alerts" {
            tags "Sync"
            properties {
                "tool" "Fraud Console"
            }
        }

        complianceOfficer -> auditSystem "Reviews compliance" {
            tags "Sync"
            properties {
                "frequency" "Quarterly"
            }
        }

        # ========================================
        # Internal Container Relationships
        # ========================================

        apiGateway -> paymentOrchestrator "Routes payment request" {
            tags "Sync,Internal"
            properties {
                "protocol" "HTTP/2"
                "load_balancing" "Round-robin"
            }
        }

        paymentOrchestrator -> tokenizationVault "Tokenize card data" {
            tags "Sync,Critical,Encrypted,Internal"
            properties {
                "protocol" "gRPC"
                "encryption" "TLS 1.3"
            }
        }

        paymentOrchestrator -> acquirerGateway "Send to acquirer" {
            tags "Sync,Internal"
            properties {
                "protocol" "HTTP/2"
            }
        }

        paymentOrchestrator -> fraudEngine "Get fraud score" {
            tags "Sync,Critical,Internal"
            properties {
                "protocol" "gRPC"
                "timeout" "100ms"
            }
        }

        paymentOrchestrator -> database "Store transaction" {
            tags "Sync,Internal"
            properties {
                "protocol" "JDBC"
                "pool_size" "100"
            }
        }

        paymentOrchestrator -> cache "Cache payment data" {
            tags "Sync,Internal"
            properties {
                "protocol" "Redis Protocol"
                "ttl" "3600s"
            }
        }

        paymentOrchestrator -> eventStream "Publish events" {
            tags "Async,Internal"
            properties {
                "protocol" "Kafka"
                "topics" "10+"
            }
        }

        acquirerGateway -> visaNetwork "Forward to Visa" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "ISO 8583"
            }
        }

        acquirerGateway -> mastercardNetwork "Forward to Mastercard" {
            tags "Sync,External,Encrypted"
            properties {
                "protocol" "ISO 8583"
            }
        }

        fraudEngine -> cache "Check velocity cache" {
            tags "Sync,Internal"
            properties {
                "protocol" "Redis Protocol"
            }
        }

        fraudEngine -> database "Query historical data" {
            tags "Sync,Internal"
            properties {
                "protocol" "JDBC"
            }
        }

        settlementEngine -> database "Read transactions" {
            tags "Batch,Internal"
            properties {
                "protocol" "JDBC"
                "batch_size" "10000"
            }
        }

        settlementEngine -> bankingPartner "Send settlement file" {
            tags "Batch,External,Encrypted"
            properties {
                "protocol" "SFTP"
            }
        }

        settlementEngine -> eventStream "Publish settlement events" {
            tags "Async,Internal"
            properties {
                "protocol" "Kafka"
            }
        }

        monitoringService -> apiGateway "Collect metrics" {
            tags "Async,Internal"
            properties {
                "protocol" "Prometheus"
            }
        }

        monitoringService -> paymentOrchestrator "Collect metrics" {
            tags "Async,Internal"
            properties {
                "protocol" "Prometheus"
            }
        }

        monitoringService -> fraudEngine "Collect metrics" {
            tags "Async,Internal"
            properties {
                "protocol" "Prometheus"
            }
        }

        complianceService -> database "Audit transactions" {
            tags "Batch,Internal"
            properties {
                "protocol" "JDBC"
                "frequency" "Daily"
            }
        }

        complianceService -> auditSystem "Send compliance report" {
            tags "Batch,Internal"
            properties {
                "protocol" "REST API"
                "format" "JSON"
            }
        }

        # ========================================
        # Component-level Relationships
        # ========================================

        transactionManager -> routingEngine "Determine best route" {
            tags "Sync,Internal"
        }

        routingEngine -> retryManager "Handle failures" {
            tags "Sync,Internal"
        }

        retryManager -> responseProcessor "Process response" {
            tags "Sync,Internal"
        }

        tokenGenerator -> keyManager "Get encryption key" {
            tags "Sync,Critical,Internal"
        }

        tokenMapper -> keyManager "Decrypt token" {
            tags "Sync,Critical,Internal"
        }

        protocolAdapter -> connectionPool "Get connection" {
            tags "Sync,Internal"
        }

        connectionPool -> circuitBreaker "Check circuit state" {
            tags "Sync,Internal"
        }

        riskScorer -> velocityChecker "Get velocity data" {
            tags "Sync,Internal"
        }

        riskScorer -> deviceProfiler "Get device profile" {
            tags "Sync,Internal"
        }

        riskScorer -> behaviorAnalyzer "Get behavior score" {
            tags "Sync,Internal"
        }

        batchProcessor -> reconciler "Reconcile batch" {
            tags "Batch,Internal"
        }

        reconciler -> reportGenerator "Generate report" {
            tags "Batch,Internal"
        }

        pciScanner -> auditLogger "Log scan results" {
            tags "Async,Internal"
        }

        # ========================================
        # Deployment Infrastructure
        # ========================================

        # Production AWS Environment
        awsCloud = deploymentNode "Amazon Web Services" "AWS Cloud Infrastructure" {
            tags "Cloud,Infrastructure"

            pciVpc = deploymentNode "PCI-Compliant VPC" "Isolated network for payment processing" {
                tags "Security,Infrastructure"

                eksCluster = deploymentNode "EKS Cluster" "Kubernetes cluster" {
                    technology "Kubernetes 1.28"
                    tags "Infrastructure"

                    apiGatewayPod = deploymentNode "API Gateway Pod" "Container pod" {
                        technology "Docker"
                        containerInstance apiGateway
                    }

                    orchestratorPod = deploymentNode "Payment Orchestrator Pod" "Container pod" {
                        technology "Docker"
                        containerInstance paymentOrchestrator
                    }

                    fraudPod = deploymentNode "Fraud Engine Pod" "Container pod" {
                        technology "Docker"
                        containerInstance fraudEngine
                    }

                    acquirerPod = deploymentNode "Acquirer Gateway Pod" "Container pod" {
                        technology "Docker"
                        containerInstance acquirerGateway
                    }

                    settlementPod = deploymentNode "Settlement Engine Pod" "Container pod" {
                        technology "Docker"
                        containerInstance settlementEngine
                    }
                }

                rdsCluster = deploymentNode "RDS Multi-AZ" "PostgreSQL cluster" {
                    technology "PostgreSQL 15"
                    tags "Database,Infrastructure"
                    containerInstance database
                }

                redisCluster = deploymentNode "ElastiCache Cluster" "Redis cluster" {
                    technology "Redis 7"
                    tags "Cache,Infrastructure"
                    containerInstance cache
                }

                kafkaCluster = deploymentNode "MSK Cluster" "Kafka cluster" {
                    technology "Apache Kafka 3.5"
                    tags "Queue,Infrastructure"
                    containerInstance eventStream
                }
            }

            hsmCluster = deploymentNode "CloudHSM Cluster" "Hardware Security Module" {
                technology "AWS CloudHSM"
                tags "Security,PCI,Infrastructure"
            }

            vaultCluster = deploymentNode "Vault Cluster" "Secrets management" {
                technology "HashiCorp Vault"
                tags "Security,Infrastructure"
                containerInstance tokenizationVault
            }
        }

        # Store Edge Environment
        storeNetwork = deploymentNode "Store Network" "Physical store infrastructure" {
            tags "Edge,Infrastructure"

            storeRouter = deploymentNode "Store Router" "Network edge device" {
                technology "Cisco"
                tags "Infrastructure"
            }

            paymentTerminal = deploymentNode "Payment Terminal" "In-store payment device" {
                technology "Ingenico"
                tags "Hardware,Security"
            }
        }
    }

    views {
        # ========================================
        # System Landscape View
        # ========================================

        systemLandscape "payment-landscape" "Payment ecosystem overview" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # System Context View
        # ========================================

        systemContext paymentGateway "payment-context" "Payment Gateway context" {
            include *
            autoLayout lr 200 100
        }

        # ========================================
        # Container View
        # ========================================

        container paymentGateway "payment-containers" "Payment Gateway containers" {
            include *
            autoLayout tb 150 100
        }

        # ========================================
        # Component Views
        # ========================================

        component paymentOrchestrator "orchestrator-components" "Payment Orchestrator components" {
            include *
            autoLayout lr 200 100
        }

        component tokenizationVault "vault-components" "Tokenization Vault components" {
            include *
            autoLayout tb 150 100
        }

        component fraudEngine "fraud-components" "Fraud Detection Engine components" {
            include *
            autoLayout lr 200 100
        }

        component settlementEngine "settlement-components" "Settlement Engine components" {
            include *
            autoLayout lr 200 100
        }

        # ========================================
        # Dynamic Views - Payment Authorization Flow
        # ========================================

        dynamic paymentGateway "payment-auth-flow" "Payment Authorization with Parallel Processing" {
            customer -> posSystem "Insert/tap card"
            posSystem -> apiGateway "POST /payment/authorize"
            apiGateway -> paymentOrchestrator "Route request"

            # Parallel processing block
            {
                paymentOrchestrator -> tokenizationVault "Tokenize card data"
                paymentOrchestrator -> fraudEngine "Check fraud score"
                paymentOrchestrator -> cache "Check velocity limits"
            }

            paymentOrchestrator -> acquirerGateway "Send to optimal acquirer"
            acquirerGateway -> visaNetwork "Process authorization"
            visaNetwork -> acquirerGateway "Authorization response"
            acquirerGateway -> paymentOrchestrator "Return response"

            # Parallel post-processing
            {
                paymentOrchestrator -> database "Store transaction"
                paymentOrchestrator -> eventStream "Publish payment.authorized"
                paymentOrchestrator -> cache "Update velocity cache"
            }

            paymentOrchestrator -> apiGateway "Return result"
            apiGateway -> posSystem "Authorization response"
            posSystem -> customer "Transaction approved"

            autoLayout lr 250 100
        }

        dynamic paymentGateway "fraud-detection-flow" "Real-time Fraud Detection Process" {
            apiGateway -> paymentOrchestrator "Payment request"
            paymentOrchestrator -> fraudEngine "Request fraud check"

            # Parallel fraud checks
            {
                fraudEngine -> riskScorer "Calculate risk score"
                fraudEngine -> velocityChecker "Check velocity"
                fraudEngine -> deviceProfiler "Profile device"
                fraudEngine -> behaviorAnalyzer "Analyze behavior"
            }

            riskScorer -> fraudEngine "Risk score: 0.85"
            velocityChecker -> fraudEngine "Velocity: Normal"
            deviceProfiler -> fraudEngine "Device: Trusted"
            behaviorAnalyzer -> fraudEngine "Behavior: Anomalous"

            fraudEngine -> paymentOrchestrator "Fraud score: HIGH"
            paymentOrchestrator -> apiGateway "Decline transaction"

            # Alert fraud team
            {
                fraudEngine -> eventStream "Publish fraud.alert"
                fraudEngine -> monitoringService "Trigger alert"
            }

            autoLayout tb 150 100
        }

        dynamic settlementEngine "settlement-flow" "Daily Settlement Process" {
            settlementEngine -> database "Query day's transactions"
            settlementEngine -> batchProcessor "Process batch"

            # Parallel reconciliation
            {
                batchProcessor -> visaNetwork "Get Visa settlement"
                batchProcessor -> mastercardNetwork "Get MC settlement"
                batchProcessor -> amexNetwork "Get Amex settlement"
            }

            batchProcessor -> reconciler "Reconcile all sources"
            reconciler -> reportGenerator "Generate settlement report"

            # Parallel settlement actions
            {
                reportGenerator -> bankingPartner "Send NACHA file"
                reportGenerator -> eventStream "Publish settlement.complete"
                reportGenerator -> auditSystem "Archive report"
            }

            autoLayout lr 200 100
        }

        dynamic tokenizationVault "tokenization-flow" "Card Tokenization Process" {
            paymentOrchestrator -> tokenizationVault "Tokenize request"
            tokenizationVault -> tokenGenerator "Generate token"
            tokenGenerator -> keyManager "Get encryption key"
            keyManager -> tokenGenerator "Return DEK"
            tokenGenerator -> tokenMapper "Store mapping"
            tokenMapper -> database "Persist token map"
            tokenMapper -> tokenGenerator "Confirm storage"
            tokenGenerator -> tokenizationVault "Return token"
            tokenizationVault -> paymentOrchestrator "Token: tok_xxxx"

            autoLayout tb 150 100
        }

        # ========================================
        # Deployment Views
        # ========================================

        deployment paymentGateway "Production" "payment-deployment-aws" "Production deployment on AWS" {
            include *
            autoLayout lr 300 100
        }

        deployment paymentGateway "Production" "payment-deployment-edge" "Edge deployment in stores" {
            include *
            autoLayout tb 200 100
        }

        # ========================================
        # Styling
        # ========================================

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            # Additional custom styles
            element "Software System" {
                background "#1168BD"
                color "#FFFFFF"
                shape RoundedBox
            }

            element "Container" {
                background "#438DD5"
                color "#FFFFFF"
            }

            element "Component" {
                background "#85BBF0"
                color "#000000"
                shape Component
            }

            relationship "Sync" {
                style Solid
                thickness 2
            }

            relationship "Async" {
                style Dashed
                thickness 2
            }

            relationship "Batch" {
                style Dotted
                thickness 2
            }

            relationship "Critical" {
                color "#DC143C"
                thickness 3
            }

            relationship "Encrypted" {
                color "#8B0000"
                thickness 3
            }
        }
    }
}