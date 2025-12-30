workspace "FreshMart Inventory Platform" "Real-time inventory management with event sourcing and CQRS" {

    !const CORP_COLOR "#00539C"
    !const EVENT_COLOR "#9B59B6"
    !const REALTIME_COLOR "#2ECC71"
    !const WAREHOUSE_COLOR "#27AE60"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People
        # ========================================

        inventoryManager = person "Inventory Manager" "Oversees inventory across all channels" {
            tags "Person"
            properties {
                "scope" "Enterprise inventory"
            }
        }

        storeAssociate = person "Store Associate" "Manages store-level inventory" {
            tags "Person,Associate"
        }

        warehouseWorker = person "Warehouse Worker" "Handles DC inventory operations" {
            tags "Person"
        }

        merchandiser = person "Merchandiser" "Plans product assortment" {
            tags "Person"
        }

        # ========================================
        # External Systems
        # ========================================

        posSystem = softwareSystem "POS System" "Point of sale transactions" {
            tags "Software System,External"
            properties {
                "events" "Sale, Return, Void"
            }
        }

        supplyChain = softwareSystem "Supply Chain Platform" "Procurement and logistics" {
            tags "Software System,External"
            properties {
                "events" "PO, Receipt, Shipment"
            }
        }

        ecommerceSystem = softwareSystem "E-Commerce Platform" "Online ordering" {
            tags "Software System,External"
            properties {
                "events" "Order, Cancel, Return"
            }
        }

        wmsSystem = softwareSystem "Warehouse Management" "DC operations" {
            tags "Software System,External"
            properties {
                "events" "Receipt, Pick, Ship"
            }
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Business intelligence" {
            tags "Software System,External"
        }

        pricingEngine = softwareSystem "Pricing Engine" "Dynamic pricing" {
            tags "Software System,External"
        }

        # ========================================
        # Inventory Platform (Main System)
        # ========================================

        inventoryPlatform = softwareSystem "Inventory Platform" "Real-time inventory management" {
            tags "Software System,Core"

            properties {
                "sku_count" "500,000+"
                "locations" "2,500 stores + 5 DCs"
                "events_per_day" "50M+"
                "accuracy" "99.5%"
                "perspective_business" "Enables $500M inventory optimization"
                "perspective_technical" "Event-sourced CQRS architecture"
                "perspective_operations" "Real-time visibility across channels"
            }

            # ========================================
            # Command Side (Write Model)
            # ========================================

            commandService = container "Command Service" "Handles inventory commands" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "pattern" "CQRS - Command"
                    "throughput" "10K commands/sec"
                }

                commandHandler = component "Command Handler" "Processes inventory commands" "Spring Service" {
                    tags "Component"
                    properties {
                        "commands" "Adjust, Reserve, Transfer, Count"
                    }
                }

                commandValidator = component "Command Validator" "Validates business rules" "Spring Service" {
                    tags "Component"
                    properties {
                        "rules" "Negative check, Location valid"
                    }
                }

                aggregateRoot = component "Inventory Aggregate" "Domain aggregate root" "DDD Aggregate" {
                    tags "Component"
                    properties {
                        "pattern" "Event Sourcing"
                    }
                }

                sagaOrchestrator = component "Saga Orchestrator" "Manages distributed transactions" "Axon Saga" {
                    tags "Component"
                    properties {
                        "sagas" "Reserve, Transfer, Allocate"
                    }
                }
            }

            eventStore = container "Event Store" "Immutable event log" "EventStoreDB" {
                tags "Container,Database,Event"
                properties {
                    "events" "5B+"
                    "streams" "10M+"
                    "retention" "Forever"
                    "replay" "Supported"
                }
            }

            # ========================================
            # Query Side (Read Model)
            # ========================================

            queryService = container "Query Service" "Handles inventory queries" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "pattern" "CQRS - Query"
                    "latency" "<50ms"
                }

                queryHandler = component "Query Handler" "Processes inventory queries" "Spring Service" {
                    tags "Component"
                    properties {
                        "queries" "Stock level, Availability, History"
                    }
                }

                projectionManager = component "Projection Manager" "Manages read models" "Spring Service" {
                    tags "Component"
                }

                cacheManager = component "Cache Manager" "Manages query cache" "Caffeine" {
                    tags "Component,Cache"
                }
            }

            # ========================================
            # Read Models (Projections)
            # ========================================

            stockLevelProjection = container "Stock Level Projection" "Current stock by location" "PostgreSQL" {
                tags "Container,Database"
                properties {
                    "records" "1.25B"
                    "index" "SKU + Location"
                }
            }

            availabilityProjection = container "Availability Projection" "Available-to-promise" "Redis" {
                tags "Container,Cache"
                properties {
                    "latency" "<5ms"
                    "atp_calculation" "Real-time"
                }
            }

            movementProjection = container "Movement History" "Inventory movement log" "TimescaleDB" {
                tags "Container,Database"
                properties {
                    "retention" "2 years"
                    "granularity" "Transaction level"
                }
            }

            # ========================================
            # Event Processing
            # ========================================

            eventProcessor = container "Event Processor" "Consumes and processes events" "Apache Flink" {
                tags "Container,Realtime"
                properties {
                    "throughput" "1M events/sec"
                    "latency" "<100ms"
                }

                eventRouter = component "Event Router" "Routes events to handlers" "Flink Job" {
                    tags "Component"
                }

                projectionUpdater = component "Projection Updater" "Updates read models" "Flink Job" {
                    tags "Component"
                }

                alertGenerator = component "Alert Generator" "Generates stock alerts" "Flink CEP" {
                    tags "Component"
                    properties {
                        "alerts" "Low stock, Overstock, Variance"
                    }
                }

                aggregator = component "Event Aggregator" "Aggregates for analytics" "Flink Job" {
                    tags "Component"
                }
            }

            eventBus = container "Event Bus" "Event streaming backbone" "Apache Kafka" {
                tags "Container,Queue,Event"
                properties {
                    "topics" "50+"
                    "partitions" "100+"
                    "throughput" "1M msg/sec"
                    "retention" "7 days"
                }
            }

            # ========================================
            # Domain Services
            # ========================================

            reservationService = container "Reservation Service" "Manages inventory reservations" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "pattern" "Soft reservation"
                    "ttl" "30 minutes default"
                }

                reservationManager = component "Reservation Manager" "Handles reserve/release" "Spring Service" {
                    tags "Component"
                }

                expirationHandler = component "Expiration Handler" "Expires stale reservations" "Scheduler" {
                    tags "Component"
                }
            }

            allocationService = container "Allocation Service" "Allocates inventory to orders" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "algorithm" "Priority + Proximity"
                    "channels" "Store, Online, Wholesale"
                }

                allocationEngine = component "Allocation Engine" "Runs allocation logic" "Spring Service" {
                    tags "Component"
                    properties {
                        "strategies" "FIFO, Priority, Proximity"
                    }
                }

                channelPrioritizer = component "Channel Prioritizer" "Prioritizes by channel" "Spring Service" {
                    tags "Component"
                }
            }

            countService = container "Cycle Count Service" "Manages inventory counts" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "count_types" "Cycle, Physical, Spot"
                }

                countScheduler = component "Count Scheduler" "Schedules counts" "Spring Service" {
                    tags "Component"
                    properties {
                        "frequency" "ABC classification"
                    }
                }

                varianceAnalyzer = component "Variance Analyzer" "Analyzes count variances" "Spring Service" {
                    tags "Component"
                    properties {
                        "threshold" "Configurable by category"
                    }
                }
            }

            transferService = container "Transfer Service" "Manages inventory transfers" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "types" "Store-to-store, DC-to-store"
                }

                transferOrchestrator = component "Transfer Orchestrator" "Coordinates transfers" "Spring Service" {
                    tags "Component"
                }

                balancingEngine = component "Balancing Engine" "Optimizes stock balance" "OR-Tools" {
                    tags "Component,ML"
                }
            }

            # ========================================
            # APIs
            # ========================================

            inventoryAPI = container "Inventory API" "REST API for clients" "Java/Spring Boot" {
                tags "Container,API"
                properties {
                    "endpoints" "50+"
                    "rate_limit" "10K req/sec"
                }

                stockEndpoint = component "Stock Endpoint" "Stock level queries" "REST Controller" {
                    tags "Component"
                }

                reserveEndpoint = component "Reserve Endpoint" "Reservation operations" "REST Controller" {
                    tags "Component"
                }

                adjustEndpoint = component "Adjust Endpoint" "Inventory adjustments" "REST Controller" {
                    tags "Component"
                }

                transferEndpoint = component "Transfer Endpoint" "Transfer operations" "REST Controller" {
                    tags "Component"
                }
            }

            graphqlAPI = container "GraphQL API" "Flexible query API" "Java/GraphQL" {
                tags "Container,API"
                properties {
                    "queries" "Stock, History, Allocation"
                }
            }

            # ========================================
            # Infrastructure
            # ========================================

            cache = container "Distributed Cache" "High-speed cache" "Redis Cluster" {
                tags "Container,Cache"
                properties {
                    "memory" "512GB"
                    "operations" "1M ops/sec"
                }
            }
        }

        # ========================================
        # Relationships - External
        # ========================================

        inventoryManager -> inventoryAPI "Manages inventory" {
            tags "Sync"
        }

        storeAssociate -> inventoryAPI "Stock inquiries and counts" {
            tags "Sync"
        }

        warehouseWorker -> inventoryAPI "DC operations" {
            tags "Sync"
        }

        merchandiser -> graphqlAPI "Assortment planning" {
            tags "Sync"
        }

        posSystem -> eventBus "Sale events" {
            tags "Async,Realtime"
            properties {
                "topic" "pos.transactions"
                "volume" "5M/day"
            }
        }

        supplyChain -> eventBus "Receipt events" {
            tags "Async"
            properties {
                "topic" "supply.receipts"
            }
        }

        ecommerceSystem -> inventoryAPI "Reserve inventory" {
            tags "Sync,Critical"
            properties {
                "sla" "<100ms"
            }
        }

        ecommerceSystem -> eventBus "Order events" {
            tags "Async"
            properties {
                "topic" "ecom.orders"
            }
        }

        wmsSystem -> eventBus "Warehouse events" {
            tags "Async"
            properties {
                "topic" "wms.movements"
            }
        }

        inventoryPlatform -> analyticsSystem "Inventory metrics" {
            tags "Async"
        }

        inventoryPlatform -> pricingEngine "Stock levels for pricing" {
            tags "Sync"
        }

        # ========================================
        # Internal Relationships
        # ========================================

        # API to Services
        inventoryAPI -> commandService "Write operations" {
            tags "Sync,Internal"
        }

        inventoryAPI -> queryService "Read operations" {
            tags "Sync,Internal"
        }

        graphqlAPI -> queryService "Complex queries" {
            tags "Sync,Internal"
        }

        # Command flow
        commandService -> eventStore "Append events" {
            tags "Sync,Critical,Internal"
            properties {
                "pattern" "Event Sourcing"
            }
        }

        commandService -> eventBus "Publish events" {
            tags "Async,Internal"
        }

        commandService -> reservationService "Reserve inventory" {
            tags "Sync,Internal"
        }

        commandService -> transferService "Transfer inventory" {
            tags "Sync,Internal"
        }

        # Event processing
        eventBus -> eventProcessor "Consume events" {
            tags "Async,Realtime,Internal"
        }

        eventProcessor -> stockLevelProjection "Update stock" {
            tags "Async,Internal"
        }

        eventProcessor -> availabilityProjection "Update ATP" {
            tags "Async,Internal"
        }

        eventProcessor -> movementProjection "Log movement" {
            tags "Async,Internal"
        }

        # Query flow
        queryService -> stockLevelProjection "Query stock" {
            tags "Sync,Internal"
        }

        queryService -> availabilityProjection "Query ATP" {
            tags "Sync,Internal"
        }

        queryService -> movementProjection "Query history" {
            tags "Sync,Internal"
        }

        queryService -> cache "Cache queries" {
            tags "Sync,Internal"
        }

        # Domain services
        reservationService -> eventBus "Reservation events" {
            tags "Async,Internal"
        }

        reservationService -> cache "Reservation cache" {
            tags "Sync,Internal"
        }

        allocationService -> queryService "Check availability" {
            tags "Sync,Internal"
        }

        allocationService -> commandService "Allocate" {
            tags "Sync,Internal"
        }

        countService -> commandService "Adjust inventory" {
            tags "Sync,Internal"
        }

        countService -> eventBus "Count events" {
            tags "Async,Internal"
        }

        transferService -> commandService "Execute transfer" {
            tags "Sync,Internal"
        }

        # Component relationships
        commandHandler -> commandValidator "Validate" {
            tags "Sync,Internal"
        }

        commandValidator -> aggregateRoot "Apply command" {
            tags "Sync,Internal"
        }

        aggregateRoot -> sagaOrchestrator "Start saga" {
            tags "Sync,Internal"
        }

        queryHandler -> projectionManager "Get projection" {
            tags "Sync,Internal"
        }

        projectionManager -> cacheManager "Cache result" {
            tags "Sync,Internal"
        }

        eventRouter -> projectionUpdater "Route to updater" {
            tags "Async,Internal"
        }

        projectionUpdater -> alertGenerator "Check alerts" {
            tags "Async,Internal"
        }

        alertGenerator -> aggregator "Aggregate" {
            tags "Async,Internal"
        }

        reservationManager -> expirationHandler "Schedule expiry" {
            tags "Async,Internal"
        }

        allocationEngine -> channelPrioritizer "Get priority" {
            tags "Sync,Internal"
        }

        countScheduler -> varianceAnalyzer "Analyze" {
            tags "Sync,Internal"
        }

        transferOrchestrator -> balancingEngine "Optimize" {
            tags "Sync,Internal"
        }

        stockEndpoint -> queryHandler "Get stock" {
            tags "Sync,Internal"
        }

        reserveEndpoint -> reservationManager "Reserve" {
            tags "Sync,Internal"
        }

        adjustEndpoint -> commandHandler "Adjust" {
            tags "Sync,Internal"
        }

        transferEndpoint -> transferOrchestrator "Transfer" {
            tags "Sync,Internal"
        }

        # ========================================
        # Deployment Nodes
        # ========================================

        cloudInfra = deploymentNode "Cloud Infrastructure" "Production cloud environment" {
            apiZone = deploymentNode "API Zone" "API services" {
                containerInstance inventoryAPI
                containerInstance graphqlAPI
            }
            commandZone = deploymentNode "Command Zone" "Command services" {
                containerInstance commandService
                containerInstance reservationService
                containerInstance allocationService
                containerInstance countService
                containerInstance transferService
            }
            queryZone = deploymentNode "Query Zone" "Query services" {
                containerInstance queryService
            }
            eventZone = deploymentNode "Event Zone" "Event infrastructure" {
                containerInstance eventBus
                containerInstance eventProcessor
                containerInstance eventStore
            }
            dataZone = deploymentNode "Data Zone" "Data stores" {
                containerInstance stockLevelProjection
                containerInstance availabilityProjection
                containerInstance movementProjection
                containerInstance cache
            }
        }
    }

    views {
        systemLandscape "inventory-landscape" "Inventory ecosystem" {
            include *
            autoLayout lr 300 100
        }

        systemContext inventoryPlatform "inventory-context" "Inventory Platform context" {
            include *
            autoLayout lr 250 100
        }

        container inventoryPlatform "inventory-containers" "Inventory containers" {
            include *
            autoLayout tb 200 100
        }

        component commandService "command-components" "Command Service components" {
            include *
            autoLayout lr 200 100
        }

        component queryService "query-components" "Query Service components" {
            include *
            autoLayout lr 200 100
        }

        component eventProcessor "processor-components" "Event Processor components" {
            include *
            autoLayout lr 200 100
        }

        component inventoryAPI "api-components" "Inventory API components" {
            include *
            autoLayout tb 150 100
        }

        # Dynamic Views
        dynamic inventoryPlatform "sale-event-flow" "POS Sale Event Processing" {
            posSystem -> eventBus "Sale completed event"
            eventBus -> eventProcessor "Consume event"

            # Parallel event processing
            {
                eventProcessor -> stockLevelProjection "Decrement stock"
                eventProcessor -> movementProjection "Log sale movement"
                eventProcessor -> availabilityProjection "Update ATP"
            }

            eventProcessor -> alertGenerator "Check stock level"
            alertGenerator -> eventBus "Low stock alert"
            eventBus -> analyticsSystem "Forward to analytics"

            autoLayout lr 250 100
        }

        dynamic inventoryPlatform "reservation-flow" "E-Commerce Reservation Flow" {
            ecommerceSystem -> inventoryAPI "Reserve request"
            inventoryAPI -> reserveEndpoint "Route to endpoint"
            reserveEndpoint -> reservationManager "Process reservation"

            # Parallel checks
            {
                reservationManager -> queryService "Check availability"
                reservationManager -> cache "Check existing reservations"
            }

            queryService -> availabilityProjection "Get ATP"
            reservationManager -> commandService "Create reservation"
            commandService -> commandHandler "Handle command"
            commandHandler -> aggregateRoot "Apply reservation"
            aggregateRoot -> eventStore "Store event"

            # Parallel updates
            {
                commandService -> eventBus "Publish event"
                reservationManager -> cache "Cache reservation"
            }

            eventBus -> eventProcessor "Process event"
            eventProcessor -> availabilityProjection "Update ATP"

            reservationManager -> expirationHandler "Schedule expiry"
            inventoryAPI -> ecommerceSystem "Reservation confirmed"

            autoLayout tb 200 100
        }

        dynamic inventoryPlatform "transfer-flow" "Store-to-Store Transfer" {
            inventoryManager -> inventoryAPI "Create transfer"
            inventoryAPI -> transferEndpoint "Route request"
            transferEndpoint -> transferOrchestrator "Process transfer"

            transferOrchestrator -> balancingEngine "Optimize route"
            transferOrchestrator -> queryService "Check source stock"
            queryService -> stockLevelProjection "Get stock level"

            transferOrchestrator -> commandService "Execute transfer"
            commandHandler -> sagaOrchestrator "Start transfer saga"

            # Parallel saga steps
            {
                sagaOrchestrator -> aggregateRoot "Decrement source"
                sagaOrchestrator -> aggregateRoot "Increment destination"
            }

            aggregateRoot -> eventStore "Store events"
            commandService -> eventBus "Publish transfer events"

            eventBus -> eventProcessor "Process events"
            eventProcessor -> stockLevelProjection "Update both locations"
            eventProcessor -> movementProjection "Log transfer"

            autoLayout lr 250 100
        }

        dynamic eventProcessor "event-replay-flow" "Event Replay for Projection Rebuild" {
            inventoryManager -> commandService "Trigger replay"
            commandService -> eventStore "Read all events"
            eventStore -> eventProcessor "Stream events"

            # Parallel projection rebuild
            {
                eventRouter -> projectionUpdater "Route to stock"
                eventRouter -> projectionUpdater "Route to ATP"
                eventRouter -> projectionUpdater "Route to movement"
            }

            projectionUpdater -> stockLevelProjection "Rebuild stock"
            projectionUpdater -> availabilityProjection "Rebuild ATP"
            projectionUpdater -> movementProjection "Rebuild history"

            eventProcessor -> commandService "Replay complete"

            autoLayout tb 200 100
        }

        dynamic countService "cycle-count-flow" "Cycle Count Process" {
            countScheduler -> inventoryAPI "Get items to count"
            inventoryAPI -> queryService "Query high-variance items"
            queryService -> movementProjection "Get movement history"

            storeAssociate -> inventoryAPI "Submit count"
            inventoryAPI -> adjustEndpoint "Route count"
            adjustEndpoint -> commandHandler "Process count"

            commandHandler -> varianceAnalyzer "Analyze variance"
            varianceAnalyzer -> commandHandler "Variance report"

            # Parallel processing
            {
                commandHandler -> aggregateRoot "Apply adjustment"
                commandHandler -> eventBus "Publish count event"
            }

            aggregateRoot -> eventStore "Store adjustment event"
            eventBus -> eventProcessor "Process adjustment"
            eventProcessor -> stockLevelProjection "Update stock"
            eventProcessor -> analyticsSystem "Report variance"

            autoLayout lr 250 100
        }

        deployment inventoryPlatform "Production" "inventory-deployment" "Production deployment" {
            include *
            autoLayout lr 300 100
        }

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "Event" {
                shape Pipe
                background "#9B59B6"
                color "#FFFFFF"
            }

            relationship "Realtime" {
                color "#2ECC71"
                thickness 3
            }
        }
    }
}