workspace "FreshMart POS Terminal System" "Next-generation point of sale with edge computing and plugin architecture" {

    !const CORP_COLOR "#00539C"
    !const EDGE_COLOR "#2C3E50"
    !const HARDWARE_COLOR "#E74C3C"
    !const OFFLINE_COLOR "#F39C12"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People
        # ========================================

        customer = person "Customer" "FreshMart shopper making purchases" {
            tags "Person,Customer"
            properties {
                "transactions_per_visit" "1-3"
                "payment_methods" "Card, Cash, Mobile"
            }
        }

        cashier = person "Cashier" "Store associate operating POS terminal" {
            tags "Person,Associate"
            properties {
                "training" "POS certified"
                "transactions_per_shift" "200-400"
            }
        }

        storeManager = person "Store Manager" "Manages store operations and staff" {
            tags "Person,Associate"
            properties {
                "access" "Manager override, Reports"
            }
        }

        itSupport = person "IT Support" "Technical support for POS systems" {
            tags "Person"
            properties {
                "scope" "Hardware, Software, Network"
            }
        }

        # ========================================
        # External Systems
        # ========================================

        paymentGateway = softwareSystem "Payment Gateway" "Central payment processing" {
            tags "Software System,External,Core"
            properties {
                "integration" "REST API"
                "fallback" "Store-and-forward"
            }
        }

        inventoryPlatform = softwareSystem "Inventory Platform" "Real-time inventory management" {
            tags "Software System,External,Core"
            properties {
                "integration" "Kafka"
            }
        }

        loyaltySystem = softwareSystem "Loyalty & Rewards" "Customer loyalty program" {
            tags "Software System,External"
            properties {
                "integration" "REST API"
            }
        }

        pricingEngine = softwareSystem "Pricing Engine" "Dynamic pricing and promotions" {
            tags "Software System,External"
            properties {
                "integration" "REST + Cache"
            }
        }

        storeOperations = softwareSystem "Store Operations" "Task and workforce management" {
            tags "Software System,External"
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Transaction analytics" {
            tags "Software System,External"
        }

        # ========================================
        # POS Terminal System (Main Focus)
        # ========================================

        posTerminal = softwareSystem "POS Terminal System" "Point of sale application" {
            tags "Software System,Core,Edge"

            properties {
                "terminals" "25,000+"
                "transactions_per_day" "5M+"
                "offline_capable" "Yes"
                "avg_transaction_time" "<30 seconds"
                "perspective_business" "Processes $10B+ annual revenue"
                "perspective_technical" "Edge-first with cloud sync"
                "perspective_operations" "99.99% uptime with offline resilience"
            }

            # ========================================
            # User Interface Layer
            # ========================================

            posUI = container "POS User Interface" "Touch-optimized checkout interface" "React Native" {
                tags "Container,UI"
                properties {
                    "platform" "Windows, Android"
                    "accessibility" "WCAG 2.1 AA"
                }

                checkoutScreen = component "Checkout Screen" "Main transaction interface" "React Component" {
                    tags "Component"
                    properties {
                        "interactions" "Scan, Search, Quantity, Void"
                    }
                }

                paymentScreen = component "Payment Screen" "Payment method selection" "React Component" {
                    tags "Component"
                }

                customerDisplay = component "Customer Display" "Customer-facing screen" "React Component" {
                    tags "Component"
                    properties {
                        "features" "Items, Total, Promotions"
                    }
                }

                managerScreen = component "Manager Functions" "Override and admin functions" "React Component" {
                    tags "Component"
                    properties {
                        "functions" "Void, Return, Price Override, Reports"
                    }
                }

                receiptPreview = component "Receipt Preview" "Digital receipt display" "React Component" {
                    tags "Component"
                }
            }

            # ========================================
            # Application Core Layer
            # ========================================

            posCore = container "POS Core Engine" "Transaction processing logic" "Rust" {
                tags "Container"
                properties {
                    "language" "Rust"
                    "performance" "<10ms transaction processing"
                    "memory" "<100MB"
                }

                transactionManager = component "Transaction Manager" "Manages transaction lifecycle" "Rust Module" {
                    tags "Component"
                    properties {
                        "states" "Open, Suspended, Completed, Voided"
                    }
                }

                itemProcessor = component "Item Processor" "Processes scanned items" "Rust Module" {
                    tags "Component"
                    properties {
                        "lookup" "Barcode, PLU, Search"
                    }
                }

                pricingCalculator = component "Pricing Calculator" "Calculates prices and discounts" "Rust Module" {
                    tags "Component"
                    properties {
                        "rules" "BOGO, %, $, Mix-Match"
                    }
                }

                taxEngine = component "Tax Engine" "Calculates applicable taxes" "Rust Module" {
                    tags "Component"
                    properties {
                        "jurisdictions" "State, County, City"
                    }
                }

                tenderManager = component "Tender Manager" "Processes payment tenders" "Rust Module" {
                    tags "Component"
                    properties {
                        "tenders" "Cash, Card, Gift, EBT"
                    }
                }

                receiptGenerator = component "Receipt Generator" "Generates receipts" "Rust Module" {
                    tags "Component"
                    properties {
                        "formats" "Print, Email, SMS"
                    }
                }
            }

            # ========================================
            # Plugin System
            # ========================================

            pluginManager = container "Plugin Manager" "Extensible plugin architecture" "Rust" {
                tags "Container"
                properties {
                    "plugins" "50+"
                    "hot_reload" "Yes"
                    "sandboxed" "Yes"
                }

                pluginLoader = component "Plugin Loader" "Loads and manages plugins" "Rust Module" {
                    tags "Component"
                    properties {
                        "format" "WASM"
                    }
                }

                pluginSandbox = component "Plugin Sandbox" "Secure execution environment" "WASM Runtime" {
                    tags "Component,Security"
                    properties {
                        "isolation" "Memory, CPU, I/O"
                    }
                }

                pluginAPI = component "Plugin API" "API for plugin development" "Rust Traits" {
                    tags "Component"
                    properties {
                        "hooks" "Pre-scan, Post-scan, Pre-tender, Post-tender"
                    }
                }

                pluginRegistry = component "Plugin Registry" "Plugin configuration store" "SQLite" {
                    tags "Component,Database"
                }
            }

            # ========================================
            # Hardware Abstraction Layer
            # ========================================

            hardwareManager = container "Hardware Manager" "Device abstraction layer" "Rust" {
                tags "Container,Hardware"
                properties {
                    "devices" "Scanner, Scale, Printer, Payment Terminal, Cash Drawer"
                }

                scannerDriver = component "Scanner Driver" "Barcode scanner integration" "Rust Driver" {
                    tags "Component,Hardware"
                    properties {
                        "types" "1D, 2D, QR"
                        "connection" "USB, Serial"
                    }
                }

                scaleDriver = component "Scale Driver" "Weight scale integration" "Rust Driver" {
                    tags "Component,Hardware"
                    properties {
                        "certification" "NTEP certified"
                        "precision" "0.01 lb"
                    }
                }

                printerDriver = component "Printer Driver" "Receipt printer integration" "Rust Driver" {
                    tags "Component,Hardware"
                    properties {
                        "protocols" "ESC/POS, Star"
                    }
                }

                paymentTerminalDriver = component "Payment Terminal Driver" "Card reader integration" "Rust Driver" {
                    tags "Component,Hardware,Security"
                    properties {
                        "emv" "Contact, Contactless"
                        "p2pe" "Certified"
                    }
                }

                cashDrawerDriver = component "Cash Drawer Driver" "Cash drawer control" "Rust Driver" {
                    tags "Component,Hardware"
                }

                deviceMonitor = component "Device Monitor" "Hardware health monitoring" "Rust Service" {
                    tags "Component"
                    properties {
                        "checks" "Connectivity, Status, Paper"
                    }
                }
            }

            # ========================================
            # Offline/Sync Layer
            # ========================================

            offlineEngine = container "Offline Engine" "Offline-first data management" "Rust" {
                tags "Container,Edge"
                properties {
                    "storage" "SQLite + RocksDB"
                    "sync" "Bidirectional"
                    "conflict" "Last-write-wins + CRDT"
                }

                localDatabase = component "Local Database" "Transaction and product data" "SQLite" {
                    tags "Component,Database"
                    properties {
                        "tables" "Products, Transactions, Customers, Settings"
                    }
                }

                syncManager = component "Sync Manager" "Cloud synchronization" "Rust Service" {
                    tags "Component"
                    properties {
                        "protocol" "WebSocket + REST"
                        "compression" "LZ4"
                    }
                }

                conflictResolver = component "Conflict Resolver" "Handles sync conflicts" "CRDT Engine" {
                    tags "Component"
                    properties {
                        "strategy" "LWW-Register, OR-Set"
                    }
                }

                queueManager = component "Queue Manager" "Offline transaction queue" "Rust Service" {
                    tags "Component"
                    properties {
                        "capacity" "10,000 transactions"
                        "priority" "FIFO with priority lanes"
                    }
                }

                cacheManager = component "Cache Manager" "Product and price cache" "RocksDB" {
                    tags "Component,Cache"
                    properties {
                        "size" "1GB"
                        "eviction" "LRU"
                    }
                }
            }

            # ========================================
            # Store Controller Layer
            # ========================================

            storeController = container "Store Controller" "Local store coordination" "Rust" {
                tags "Container,Edge"
                properties {
                    "role" "Store-level orchestration"
                    "ha" "Active-passive"
                }

                laneManager = component "Lane Manager" "Manages POS lanes" "Rust Service" {
                    tags "Component"
                    properties {
                        "lanes" "10-20 per store"
                    }
                }

                storeSync = component "Store Sync" "Cloud-store synchronization" "Rust Service" {
                    tags "Component"
                    properties {
                        "interval" "Real-time + Batch"
                    }
                }

                reportingEngine = component "Reporting Engine" "Store-level reports" "Rust Service" {
                    tags "Component"
                    properties {
                        "reports" "Sales, Cashier, Hourly"
                    }
                }

                alertManager = component "Alert Manager" "Store alerts and notifications" "Rust Service" {
                    tags "Component"
                }
            }

            # ========================================
            # Security Layer
            # ========================================

            securityManager = container "Security Manager" "POS security and compliance" "Rust" {
                tags "Container,Security"
                properties {
                    "encryption" "AES-256"
                    "compliance" "PCI-DSS, PA-DSS"
                }

                authManager = component "Auth Manager" "User authentication" "Rust Service" {
                    tags "Component,Security"
                    properties {
                        "methods" "PIN, Badge, Biometric"
                    }
                }

                auditLogger = component "Audit Logger" "Security event logging" "Rust Service" {
                    tags "Component,Security"
                    properties {
                        "retention" "7 years"
                        "tamper_proof" "Yes"
                    }
                }

                encryptionService = component "Encryption Service" "Data encryption" "Rust Crypto" {
                    tags "Component,Security"
                    properties {
                        "algorithms" "AES-256-GCM, RSA-2048"
                    }
                }
            }
        }

        # ========================================
        # Hardware Devices (External)
        # ========================================

        barcodeScanner = softwareSystem "Barcode Scanner" "Item scanning device" {
            tags "External,Hardware"
        }

        weightScale = softwareSystem "Weight Scale" "Product weighing device" {
            tags "External,Hardware"
        }

        receiptPrinter = softwareSystem "Receipt Printer" "Thermal receipt printer" {
            tags "External,Hardware"
        }

        paymentReader = softwareSystem "Payment Terminal" "EMV card reader" {
            tags "External,Hardware,Security"
        }

        cashDrawer = softwareSystem "Cash Drawer" "Cash storage device" {
            tags "External,Hardware"
        }

        # ========================================
        # Relationships - People
        # ========================================

        customer -> posUI "Views transaction" {
            tags "Sync"
        }

        cashier -> posUI "Operates terminal" {
            tags "Sync"
            properties {
                "frequency" "200-400 transactions/shift"
            }
        }

        storeManager -> posUI "Manager functions" {
            tags "Sync"
            properties {
                "functions" "Override, Reports, Close"
            }
        }

        itSupport -> storeController "Monitors and troubleshoots" {
            tags "Sync"
        }

        # ========================================
        # Relationships - Hardware
        # ========================================

        barcodeScanner -> hardwareManager "Scan events" {
            tags "Sync,Hardware"
            properties {
                "protocol" "HID/Serial"
            }
        }

        weightScale -> hardwareManager "Weight data" {
            tags "Sync,Hardware"
        }

        hardwareManager -> receiptPrinter "Print commands" {
            tags "Sync,Hardware"
        }

        hardwareManager -> paymentReader "Payment requests" {
            tags "Sync,Hardware,Critical"
        }

        hardwareManager -> cashDrawer "Open drawer" {
            tags "Sync,Hardware"
        }

        # ========================================
        # Relationships - External Systems
        # ========================================

        posTerminal -> paymentGateway "Payment authorization" {
            tags "Sync,Critical,Encrypted"
            properties {
                "protocol" "HTTPS"
                "fallback" "Store-and-forward"
            }
        }

        posTerminal -> inventoryPlatform "Inventory updates" {
            tags "Async"
            properties {
                "protocol" "Kafka"
            }
        }

        posTerminal -> loyaltySystem "Loyalty lookup and accrual" {
            tags "Sync"
            properties {
                "protocol" "REST API"
            }
        }

        posTerminal -> pricingEngine "Price and promo lookup" {
            tags "Sync"
            properties {
                "caching" "Local with TTL"
            }
        }

        posTerminal -> analyticsSystem "Transaction data" {
            tags "Async"
            properties {
                "protocol" "Kafka"
            }
        }

        storeController -> storeOperations "Store events" {
            tags "Async"
        }

        # ========================================
        # Relationships - Internal Containers
        # ========================================

        posUI -> posCore "User actions" {
            tags "Sync,Internal"
        }

        posCore -> hardwareManager "Device operations" {
            tags "Sync,Internal"
        }

        posCore -> offlineEngine "Data access" {
            tags "Sync,Internal"
        }

        posCore -> pluginManager "Plugin hooks" {
            tags "Sync,Internal"
        }

        posCore -> securityManager "Auth and audit" {
            tags "Sync,Internal"
        }

        offlineEngine -> storeController "Sync operations" {
            tags "Async,Internal"
        }

        storeController -> paymentGateway "Route payments" {
            tags "Sync,Internal"
        }

        storeController -> inventoryPlatform "Aggregate updates" {
            tags "Async,Internal"
        }

        pluginManager -> posCore "Extend functionality" {
            tags "Sync,Internal"
        }

        # ========================================
        # Component Relationships - UI
        # ========================================

        checkoutScreen -> itemProcessor "Add item" {
            tags "Sync,Internal"
        }

        paymentScreen -> tenderManager "Process payment" {
            tags "Sync,Internal"
        }

        managerScreen -> transactionManager "Manager override" {
            tags "Sync,Internal"
        }

        receiptPreview -> receiptGenerator "Get receipt" {
            tags "Sync,Internal"
        }

        customerDisplay -> transactionManager "Display updates" {
            tags "Sync,Internal"
        }

        # ========================================
        # Component Relationships - Core
        # ========================================

        transactionManager -> itemProcessor "Process items" {
            tags "Sync,Internal"
        }

        itemProcessor -> pricingCalculator "Get price" {
            tags "Sync,Internal"
        }

        pricingCalculator -> taxEngine "Calculate tax" {
            tags "Sync,Internal"
        }

        transactionManager -> tenderManager "Accept payment" {
            tags "Sync,Internal"
        }

        tenderManager -> receiptGenerator "Generate receipt" {
            tags "Sync,Internal"
        }

        # ========================================
        # Component Relationships - Plugins
        # ========================================

        pluginLoader -> pluginSandbox "Execute plugin" {
            tags "Sync,Internal"
        }

        pluginSandbox -> pluginAPI "Call hooks" {
            tags "Sync,Internal"
        }

        pluginLoader -> pluginRegistry "Load config" {
            tags "Sync,Internal"
        }

        # ========================================
        # Component Relationships - Hardware
        # ========================================

        scannerDriver -> itemProcessor "Barcode data" {
            tags "Sync,Internal"
        }

        scaleDriver -> itemProcessor "Weight data" {
            tags "Sync,Internal"
        }

        receiptGenerator -> printerDriver "Print receipt" {
            tags "Sync,Internal"
        }

        tenderManager -> paymentTerminalDriver "Card payment" {
            tags "Sync,Critical,Internal"
        }

        tenderManager -> cashDrawerDriver "Open drawer" {
            tags "Sync,Internal"
        }

        deviceMonitor -> scannerDriver "Check status" {
            tags "Async,Internal"
        }

        deviceMonitor -> scaleDriver "Check status" {
            tags "Async,Internal"
        }

        deviceMonitor -> printerDriver "Check status" {
            tags "Async,Internal"
        }

        # ========================================
        # Component Relationships - Offline
        # ========================================

        itemProcessor -> localDatabase "Product lookup" {
            tags "Sync,Internal"
        }

        transactionManager -> localDatabase "Store transaction" {
            tags "Sync,Internal"
        }

        syncManager -> queueManager "Queue for sync" {
            tags "Async,Internal"
        }

        syncManager -> conflictResolver "Resolve conflicts" {
            tags "Sync,Internal"
        }

        itemProcessor -> cacheManager "Price cache" {
            tags "Sync,Internal"
        }

        # ========================================
        # Component Relationships - Store Controller
        # ========================================

        laneManager -> syncManager "Coordinate sync" {
            tags "Async,Internal"
        }

        storeSync -> syncManager "Aggregate sync" {
            tags "Async,Internal"
        }

        reportingEngine -> localDatabase "Query data" {
            tags "Sync,Internal"
        }

        alertManager -> deviceMonitor "Hardware alerts" {
            tags "Async,Internal"
        }

        # ========================================
        # Component Relationships - Security
        # ========================================

        authManager -> transactionManager "Authorize action" {
            tags "Sync,Internal"
        }

        auditLogger -> transactionManager "Log events" {
            tags "Async,Internal"
        }

        encryptionService -> tenderManager "Encrypt PAN" {
            tags "Sync,Critical,Internal"
        }

        encryptionService -> localDatabase "Encrypt at rest" {
            tags "Sync,Internal"
        }

        # ========================================
        # Deployment Model
        # ========================================

        storeInfrastructure = deploymentNode "Store Infrastructure" "Physical store deployment" {
            posLane = deploymentNode "POS Lane" "Terminal lane" {
                containerInstance posUI
                containerInstance posCore
                containerInstance pluginManager
                containerInstance hardwareManager
                containerInstance offlineEngine
                containerInstance securityManager
            }
            storeServer = deploymentNode "Store Server" "Store backend" {
                containerInstance storeController
            }
        }
    }

    views {
        # ========================================
        # System Landscape
        # ========================================

        systemLandscape "pos-landscape" "POS ecosystem overview" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # System Context
        # ========================================

        systemContext posTerminal "pos-context" "POS Terminal context" {
            include *
            autoLayout lr 250 100
        }

        # ========================================
        # Container View
        # ========================================

        container posTerminal "pos-containers" "POS Terminal containers" {
            include *
            autoLayout tb 200 100
        }

        # ========================================
        # Component Views
        # ========================================

        component posUI "ui-components" "User Interface components" {
            include *
            autoLayout lr 200 100
        }

        component posCore "core-components" "POS Core components" {
            include *
            autoLayout lr 200 100
        }

        component pluginManager "plugin-components" "Plugin Manager components" {
            include *
            autoLayout tb 150 100
        }

        component hardwareManager "hardware-components" "Hardware Manager components" {
            include *
            autoLayout tb 150 100
        }

        component offlineEngine "offline-components" "Offline Engine components" {
            include *
            autoLayout lr 200 100
        }

        component storeController "controller-components" "Store Controller components" {
            include *
            autoLayout lr 200 100
        }

        component securityManager "security-components" "Security Manager components" {
            include *
            autoLayout lr 200 100
        }

        # ========================================
        # Dynamic Views
        # ========================================

        dynamic posTerminal "checkout-flow" "Complete Checkout Transaction" {
            cashier -> checkoutScreen "Start transaction"
            barcodeScanner -> scannerDriver "Scan item"
            scannerDriver -> itemProcessor "Barcode data"

            # Parallel item processing
            {
                itemProcessor -> localDatabase "Product lookup"
                itemProcessor -> cacheManager "Price lookup"
                itemProcessor -> pluginManager "Pre-scan hook"
            }

            itemProcessor -> pricingCalculator "Calculate price"
            pricingCalculator -> taxEngine "Add tax"
            pricingCalculator -> transactionManager "Add to transaction"
            transactionManager -> customerDisplay "Update display"

            cashier -> paymentScreen "Select payment"
            paymentScreen -> tenderManager "Card payment"

            # Parallel payment processing
            {
                tenderManager -> encryptionService "Encrypt PAN"
                tenderManager -> paymentTerminalDriver "Read card"
            }

            paymentTerminalDriver -> paymentReader "EMV transaction"
            paymentReader -> paymentTerminalDriver "Card data"

            tenderManager -> paymentGateway "Authorize"
            paymentGateway -> tenderManager "Approved"

            # Parallel completion
            {
                tenderManager -> receiptGenerator "Generate receipt"
                tenderManager -> cashDrawerDriver "Open drawer"
                transactionManager -> localDatabase "Store transaction"
                transactionManager -> auditLogger "Log completion"
            }

            receiptGenerator -> printerDriver "Print receipt"
            transactionManager -> syncManager "Queue for sync"

            autoLayout lr 250 100
        }

        dynamic posTerminal "offline-transaction-flow" "Offline Transaction Processing" {
            cashier -> checkoutScreen "Start transaction"
            barcodeScanner -> scannerDriver "Scan item"
            scannerDriver -> itemProcessor "Barcode data"

            # Offline lookup
            {
                itemProcessor -> localDatabase "Product lookup"
                itemProcessor -> cacheManager "Cached price"
            }

            itemProcessor -> pricingCalculator "Calculate"
            pricingCalculator -> transactionManager "Add item"

            cashier -> paymentScreen "Card payment"
            paymentScreen -> tenderManager "Process payment"
            tenderManager -> paymentTerminalDriver "Read card"

            paymentTerminalDriver -> encryptionService "Encrypt for SAF"
            tenderManager -> queueManager "Store-and-forward"
            queueManager -> localDatabase "Persist encrypted"

            transactionManager -> localDatabase "Store transaction"
            transactionManager -> receiptGenerator "Generate receipt"
            receiptGenerator -> printerDriver "Print receipt"

            autoLayout lr 250 100
        }

        dynamic offlineEngine "sync-flow" "Cloud Synchronization Flow" {
            storeSync -> syncManager "Trigger sync"
            syncManager -> queueManager "Get pending"

            # Parallel sync operations
            {
                syncManager -> paymentGateway "Forward payments"
                syncManager -> inventoryPlatform "Sync inventory"
                syncManager -> analyticsSystem "Upload transactions"
            }

            paymentGateway -> syncManager "Payment results"
            syncManager -> conflictResolver "Check conflicts"
            conflictResolver -> localDatabase "Apply updates"

            # Parallel cache refresh
            {
                syncManager -> pricingEngine "Download prices"
                syncManager -> loyaltySystem "Sync customers"
            }

            syncManager -> cacheManager "Update cache"
            syncManager -> queueManager "Clear synced"

            autoLayout tb 200 100
        }

        dynamic pluginManager "plugin-execution-flow" "Plugin Execution" {
            itemProcessor -> pluginManager "Pre-scan hook"
            pluginManager -> pluginLoader "Load plugins"
            pluginLoader -> pluginRegistry "Get config"
            pluginLoader -> pluginSandbox "Execute"

            # Parallel plugin execution
            {
                pluginSandbox -> pluginAPI "Age verification plugin"
                pluginSandbox -> pluginAPI "Loyalty plugin"
                pluginSandbox -> pluginAPI "Promotion plugin"
            }

            pluginSandbox -> pluginLoader "Results"
            pluginLoader -> itemProcessor "Plugin responses"

            autoLayout lr 250 100
        }

        dynamic hardwareManager "hardware-initialization-flow" "Hardware Initialization" {
            storeController -> hardwareManager "Initialize devices"

            # Parallel device initialization
            {
                hardwareManager -> scannerDriver "Init scanner"
                hardwareManager -> scaleDriver "Init scale"
                hardwareManager -> printerDriver "Init printer"
                hardwareManager -> paymentTerminalDriver "Init terminal"
                hardwareManager -> cashDrawerDriver "Init drawer"
            }

            deviceMonitor -> scannerDriver "Health check"
            deviceMonitor -> scaleDriver "Health check"
            deviceMonitor -> printerDriver "Health check"

            deviceMonitor -> alertManager "Report status"
            alertManager -> storeOperations "Send alerts"

            autoLayout tb 200 100
        }

        # ========================================
        # Deployment View
        # ========================================

        deployment posTerminal "Production" "pos-deployment" "Store deployment" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # Styles
        # ========================================

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "Edge" {
                background "#2C3E50"
                color "#FFFFFF"
                border Dashed
            }

            element "Hardware" {
                background "#E74C3C"
                color "#FFFFFF"
                shape Box
            }

            element "UI" {
                background "#3498DB"
                color "#FFFFFF"
                shape WebBrowser
            }

            relationship "Hardware" {
                color "#E74C3C"
                thickness 2
            }

            relationship "Critical" {
                color "#DC143C"
                thickness 3
            }
        }
    }
}