workspace "Coffee Shop Ordering" "Simple coffee shop demonstrating dynamic diagram workflows" {

    !docs "docs"
    !adrs "adrs"
    !impliedRelationships true

    model {
        // Actors
        customer = person "Customer" "A person ordering coffee drinks"
        barista = person "Barista" "Prepares and serves drinks"

        // Main System
        coffeeShopSystem = softwareSystem "Coffee Shop System" "Manages orders and inventory" {
            posTerminal = container "POS Terminal" "Touch-screen ordering interface" "React/Electron" {
                tags "Frontend"
            }
            orderQueue = container "Order Queue Service" "Manages order lifecycle and queue display" "Node.js/Express" {
                tags "API"
            }
            inventoryService = container "Inventory Service" "Tracks ingredient stock levels" "Python/FastAPI" {
                tags "API"
            }
            database = container "Database" "Stores orders and inventory data" "PostgreSQL" {
                tags "Database"
            }
        }

        // Relationships with rich metadata
        customer -> posTerminal "Places orders using" {
            tags "user-interaction, frontend"
            properties {
                "latency" "<100ms"
                "availability" "99.9%"
            }
            perspectives {
                "Security" "Customer authentication via loyalty card or guest mode. PCI-DSS compliance required for payment data."
                "Performance" "Touch interface must respond within 100ms for smooth UX. Offline mode caches menu data."
                "Business" "Primary revenue touchpoint. Upselling opportunities via recommendations. Customer satisfaction critical."
                "Technical" "React/Electron app with local state management. WebSocket for real-time updates."
                "Operations" "Self-service reduces staff workload. Training required for edge cases."
            }
        }

        barista -> orderQueue "Views and fulfills orders from" {
            tags "staff-interaction, backend"
            properties {
                "refresh-rate" "1s"
                "priority-sorting" "enabled"
            }
            perspectives {
                "Security" "Staff authentication via PIN. Audit logging for order modifications."
                "Performance" "Real-time updates via WebSocket. Queue sorted by priority and wait time."
                "Business" "Efficient order display reduces fulfillment time. Color coding for drink complexity."
                "Technical" "Server-sent events for push updates. Responsive design for various screen sizes."
                "Operations" "Training mode available. Shift handover preserves queue state."
            }
        }

        posTerminal -> orderQueue "Submits orders to" "REST/JSON" {
            tags "api, synchronous, critical-path"
            properties {
                "timeout" "5s"
                "retry-policy" "3 attempts"
                "rate-limit" "100/min"
            }
            perspectives {
                "Security" "JWT authentication. Input validation prevents injection. Rate limiting prevents abuse."
                "Performance" "P99 latency <200ms. Connection pooling enabled. Gzip compression for payloads."
                "Business" "Order acceptance is revenue-critical. Failed orders must be recoverable."
                "Technical" "RESTful API with OpenAPI spec. Idempotency keys prevent duplicates."
                "Reliability" "Circuit breaker pattern. Fallback to local queue if service unavailable."
            }
        }

        orderQueue -> inventoryService "Reserves ingredients via" "gRPC" {
            tags "api, synchronous, inventory"
            properties {
                "protocol" "gRPC/Protobuf"
                "timeout" "2s"
                "retry-policy" "exponential backoff"
            }
            perspectives {
                "Security" "mTLS between services. Service mesh handles identity. No PII in requests."
                "Performance" "gRPC binary protocol 10x faster than JSON. Streaming for bulk operations."
                "Business" "Inventory accuracy prevents overselling. Real-time stock prevents customer disappointment."
                "Technical" "Protobuf schemas versioned. Backward compatible changes only."
                "Data Integrity" "Distributed transaction with saga pattern. Compensating actions on failure."
            }
        }

        orderQueue -> database "Persists orders to" "SQL" {
            tags "database, persistent, critical"
            properties {
                "connection-pool" "20"
                "query-timeout" "10s"
                "replication" "async"
            }
            perspectives {
                "Security" "Encrypted at rest and in transit. Row-level security for multi-tenant. SQL injection prevention."
                "Performance" "Connection pooling. Read replicas for reporting. Indexed on order_id and timestamp."
                "Business" "Order history for analytics. Customer insights for marketing. Compliance retention."
                "Technical" "PostgreSQL with JSONB for flexible order data. Partitioned by date."
                "Disaster Recovery" "Point-in-time recovery. Daily backups. 15-minute RPO."
            }
        }

        inventoryService -> database "Reads/writes inventory to" "SQL" {
            tags "database, persistent, inventory"
            properties {
                "isolation-level" "serializable"
                "deadlock-retry" "enabled"
                "audit-logging" "enabled"
            }
            perspectives {
                "Security" "Audit trail for all inventory changes. Role-based access. Anomaly detection for theft."
                "Performance" "Optimistic locking for concurrent updates. Batch operations for restocking."
                "Business" "Accurate inventory prevents waste. Low-stock alerts for reordering. Shrinkage tracking."
                "Technical" "Event sourcing for inventory changes. Materialized views for reporting."
                "Compliance" "FDA requirements for ingredient tracking. Allergen information mandatory."
            }
        }

        // Additional relationships for dynamic view
        inventoryService -> orderQueue "Confirms ingredients available" "gRPC" {
            tags "api, async-response, inventory"
            properties {
                "responseType" "availability-check"
                "timeout" "1s"
            }
            perspectives {
                "Business" "Prevents accepting orders for out-of-stock items"
                "Technical" "Callback pattern for async inventory verification"
                "Operations" "Suggests alternatives when items unavailable"
            }
        }

        orderQueue -> barista "Displays order on queue screen" "WebSocket" {
            tags "notification, real-time, staff"
            properties {
                "protocol" "WebSocket"
                "priority" "High for mobile orders"
                "displayFormat" "Color-coded by complexity"
            }
            perspectives {
                "Operations" "FIFO queue with priority override for mobile pickup"
                "Business" "Efficient order routing reduces wait times"
                "Technical" "Server-sent events with automatic reconnection"
                "UX" "Audible notification for new orders"
            }
        }
    }

    views {
        systemContext coffeeShopSystem "SystemContext" "Coffee shop system context" {
            include *
            autoLayout tb
        }

        container coffeeShopSystem "Containers" "Container architecture" {
            include *
            autoLayout tb
        }

        // Dynamic view showing order placement workflow
        dynamic coffeeShopSystem "OrderFlow" "Customer places a coffee order" {
            customer -> posTerminal "Selects drink and customizations"
            posTerminal -> orderQueue "Submits order with details"
            orderQueue -> inventoryService "Checks ingredient availability"
            inventoryService -> database "Reads current stock levels"
            inventoryService -> orderQueue "Confirms ingredients available"
            orderQueue -> database "Saves order with pending status"
            orderQueue -> barista "Displays order on queue screen"
            autoLayout lr
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
            element "Frontend" {
                shape WebBrowser
            }
            element "API" {
                shape Hexagon
            }
            relationship "Relationship" {
                color "#707070"
                thickness 2
            }
        }
    }
}
