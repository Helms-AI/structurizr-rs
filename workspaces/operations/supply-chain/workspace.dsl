workspace "FreshMart Supply Chain Integration" "B2B integration platform for supplier management and logistics optimization" {

    !const CORP_COLOR "#00539C"
    !const PARTNER_COLOR "#95A5A6"
    !const LOGISTICS_COLOR "#E67E22"
    !const WAREHOUSE_COLOR "#27AE60"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People
        # ========================================

        supplyChainManager = person "Supply Chain Manager" "Oversees procurement and logistics" {
            tags "Person"
            properties {
                "department" "Supply Chain Operations"
                "systems" "SCM Portal, Analytics"
            }
        }

        procurementSpecialist = person "Procurement Specialist" "Manages supplier relationships and orders" {
            tags "Person"
            properties {
                "focus" "Vendor management, Cost negotiation"
            }
        }

        warehouseManager = person "Warehouse Manager" "Manages distribution center operations" {
            tags "Person"
            properties {
                "facilities" "5 Regional DCs"
            }
        }

        supplierRep = person "Supplier Representative" "External supplier contact" {
            tags "Person,External,Partner"
            properties {
                "companies" "500+ suppliers"
            }
        }

        carrierRep = person "Carrier Representative" "Logistics carrier contact" {
            tags "Person,External,Partner"
            properties {
                "carriers" "50+ logistics partners"
            }
        }

        # ========================================
        # External Systems - Suppliers
        # ========================================

        group "Supplier Network" {
            majorSupplier1 = softwareSystem "Pacific Foods Inc" "Major produce supplier" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "EDI"
                    "volume" "20% of produce"
                    "lead_time" "3 days"
                }
            }

            majorSupplier2 = softwareSystem "National Dairy Co" "Dairy products supplier" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "API"
                    "volume" "40% of dairy"
                    "lead_time" "2 days"
                }
            }

            majorSupplier3 = softwareSystem "Global Packaged Goods" "CPG supplier" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "EDI + Portal"
                    "volume" "30% of packaged goods"
                    "lead_time" "7 days"
                }
            }

            smallSuppliers = softwareSystem "Local Supplier Network" "Regional and specialty suppliers" {
                tags "External,Partner,Software System"
                properties {
                    "count" "450+"
                    "integration" "Portal + Email"
                }
            }
        }

        # ========================================
        # External Systems - Logistics
        # ========================================

        group "Logistics Partners" {
            primaryCarrier = softwareSystem "TransCorp Logistics" "Primary freight carrier" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "API"
                    "coverage" "Nationwide"
                    "fleet" "5000+ trucks"
                }
            }

            lastMileCarrier = softwareSystem "QuickDeliver" "Last-mile delivery service" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "API"
                    "coverage" "Urban areas"
                    "delivery_time" "Same day"
                }
            }

            coldChainCarrier = softwareSystem "ColdLink Transport" "Refrigerated transport" {
                tags "External,Partner,Software System"
                properties {
                    "integration" "IoT + API"
                    "specialization" "Temperature controlled"
                }
            }
        }

        # ========================================
        # External Services
        # ========================================

        group "External Services" {
            weatherService = softwareSystem "Weather Intelligence" "Weather forecasting for demand" {
                tags "External,Software System"
                properties {
                    "api" "REST"
                    "coverage" "Global"
                }
            }

            marketDataService = softwareSystem "Commodity Market Data" "Price and market intelligence" {
                tags "External,Software System"
                properties {
                    "data" "Pricing, Trends, Forecasts"
                }
            }

            geoService = softwareSystem "Geolocation Service" "Route optimization and tracking" {
                tags "External,Software System"
                properties {
                    "features" "Routing, ETA, Tracking"
                }
            }
        }

        # ========================================
        # Internal Systems
        # ========================================

        inventorySystem = softwareSystem "Inventory Platform" "Real-time inventory management" {
            tags "Software System,Core"
            properties {
                "integration" "Event Stream"
            }
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Business intelligence" {
            tags "Software System"
        }

        financeSystem = softwareSystem "Finance System" "Accounts payable and receivable" {
            tags "Software System"
            properties {
                "integration" "Batch"
            }
        }

        posSystem = softwareSystem "POS System" "Point of sale data for demand" {
            tags "Software System,Core"
        }

        # ========================================
        # Supply Chain Platform (Main System)
        # ========================================

        supplyChain = softwareSystem "Supply Chain Platform" "Integrated supply chain management" {
            tags "Software System,Core"

            properties {
                "suppliers" "500+"
                "orders_per_day" "50,000"
                "shipments_tracked" "100,000/day"
                "cost_optimization" "15% savings"
                "perspective_business" "Optimizes $2B annual procurement spend"
                "perspective_technical" "Event-driven architecture with ML forecasting"
                "perspective_operations" "Real-time visibility across supply chain"
            }

            # ========================================
            # Integration Layer
            # ========================================

            ediGateway = container "EDI Gateway" "Electronic data interchange hub" "Sterling B2B Integrator" {
                tags "Container,Integration"
                properties {
                    "standards" "ANSI X12, EDIFACT"
                    "transactions" "850, 855, 856, 810"
                    "partners" "200+"
                }

                ediTranslator = component "EDI Translator" "Converts EDI to internal format" "Sterling Mapper" {
                    tags "Component"
                    properties {
                        "maps" "500+"
                    }
                }

                ediValidator = component "EDI Validator" "Validates EDI compliance" "Sterling Validator" {
                    tags "Component"
                    properties {
                        "rules" "ANSI X12 standards"
                    }
                }

                partnerManager = component "Partner Manager" "Manages trading partner configs" "Sterling Partner" {
                    tags "Component"
                    properties {
                        "partners" "200+"
                    }
                }
            }

            apiGateway = container "API Gateway" "REST API integration hub" "Kong Enterprise" {
                tags "Container,API"
                properties {
                    "endpoints" "100+"
                    "rate_limiting" "10K req/min"
                    "authentication" "OAuth 2.0"
                }

                apiRouter = component "API Router" "Routes API requests" "Kong Router" {
                    tags "Component"
                }

                apiTransformer = component "API Transformer" "Transforms payloads" "Kong Plugin" {
                    tags "Component"
                }

                rateLimiter = component "Rate Limiter" "Enforces rate limits" "Kong Plugin" {
                    tags "Component"
                }
            }

            supplierPortal = container "Supplier Portal" "Self-service supplier interface" "React + Node.js" {
                tags "Container,Web"
                properties {
                    "users" "2000+"
                    "features" "Orders, Invoices, Catalog"
                }
            }

            # ========================================
            # Procurement Domain
            # ========================================

            procurementEngine = container "Procurement Engine" "Purchase order management" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "orders_per_day" "50,000"
                    "automation_rate" "85%"
                }

                poGenerator = component "PO Generator" "Creates purchase orders" "Spring Service" {
                    tags "Component"
                    properties {
                        "triggers" "Reorder point, Manual, Scheduled"
                    }
                }

                approvalWorkflow = component "Approval Workflow" "Multi-level approval process" "Camunda" {
                    tags "Component"
                    properties {
                        "levels" "3"
                        "sla" "24 hours"
                    }
                }

                supplierSelection = component "Supplier Selection" "Optimal supplier routing" "ML Model" {
                    tags "Component,ML"
                    properties {
                        "factors" "Price, Quality, Lead time, Risk"
                    }
                }

                contractManager = component "Contract Manager" "Manages supplier contracts" "Spring Service" {
                    tags "Component"
                    properties {
                        "contracts" "1000+"
                    }
                }
            }

            # ========================================
            # Demand Planning Domain
            # ========================================

            demandPlanning = container "Demand Planning Engine" "Demand forecasting and planning" "Python/Spark" {
                tags "Container,ML"
                properties {
                    "accuracy" "92%"
                    "horizon" "12 weeks"
                    "granularity" "SKU-Store-Day"
                }

                forecastEngine = component "Forecast Engine" "ML-based demand prediction" "Prophet + LSTM" {
                    tags "Component,ML"
                    properties {
                        "models" "5"
                        "update_frequency" "Daily"
                    }
                }

                promotionPlanner = component "Promotion Planner" "Promo impact modeling" "XGBoost" {
                    tags "Component,ML"
                    properties {
                        "lift_accuracy" "85%"
                    }
                }

                seasonalityEngine = component "Seasonality Engine" "Seasonal pattern detection" "Fourier Analysis" {
                    tags "Component"
                }

                externalFactors = component "External Factors" "Weather and event integration" "Python Service" {
                    tags "Component"
                    properties {
                        "factors" "Weather, Holidays, Events"
                    }
                }
            }

            # ========================================
            # Inventory Optimization Domain
            # ========================================

            inventoryOptimization = container "Inventory Optimizer" "Stock level optimization" "Python/OR-Tools" {
                tags "Container,ML"
                properties {
                    "optimization" "Multi-echelon"
                    "savings" "20% inventory reduction"
                }

                safetyStockCalc = component "Safety Stock Calculator" "Dynamic safety stock levels" "OR-Tools" {
                    tags "Component"
                    properties {
                        "service_level" "98%"
                    }
                }

                reorderPointEngine = component "Reorder Point Engine" "Optimal reorder triggers" "ML Model" {
                    tags "Component,ML"
                }

                allocationEngine = component "Allocation Engine" "Store allocation optimization" "Linear Programming" {
                    tags "Component"
                    properties {
                        "constraints" "Capacity, Freshness, Cost"
                    }
                }
            }

            # ========================================
            # Logistics Domain
            # ========================================

            logisticsManager = container "Logistics Manager" "Transportation and delivery management" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "shipments_per_day" "100,000"
                    "carriers" "50+"
                }

                routeOptimizer = component "Route Optimizer" "Delivery route optimization" "Google OR-Tools" {
                    tags "Component"
                    properties {
                        "algorithm" "Vehicle Routing Problem"
                        "savings" "15% fuel costs"
                    }
                }

                carrierSelector = component "Carrier Selector" "Optimal carrier selection" "ML Model" {
                    tags "Component,ML"
                    properties {
                        "factors" "Cost, Speed, Reliability"
                    }
                }

                shipmentTracker = component "Shipment Tracker" "Real-time shipment visibility" "Event Processor" {
                    tags "Component"
                    properties {
                        "updates" "Every 15 minutes"
                    }
                }

                appointmentScheduler = component "Appointment Scheduler" "Dock scheduling" "Spring Service" {
                    tags "Component"
                    properties {
                        "slots" "15-minute intervals"
                    }
                }
            }

            # ========================================
            # Warehouse Domain
            # ========================================

            warehouseManager = container "Warehouse Management" "Distribution center operations" "Manhattan WMS" {
                tags "Container"
                properties {
                    "facilities" "5 DCs"
                    "capacity" "10M sq ft"
                }

                receivingModule = component "Receiving Module" "Inbound shipment processing" "WMS Module" {
                    tags "Component"
                }

                putawayModule = component "Putaway Module" "Storage location assignment" "WMS Module" {
                    tags "Component"
                    properties {
                        "algorithm" "Velocity-based"
                    }
                }

                pickingModule = component "Picking Module" "Order picking optimization" "WMS Module" {
                    tags "Component"
                    properties {
                        "methods" "Wave, Batch, Zone"
                    }
                }

                shippingModule = component "Shipping Module" "Outbound shipment processing" "WMS Module" {
                    tags "Component"
                }

                laborManager = component "Labor Manager" "Workforce scheduling" "WMS Module" {
                    tags "Component"
                }
            }

            # ========================================
            # Quality & Compliance Domain
            # ========================================

            qualityManager = container "Quality Management" "Supplier quality and compliance" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "audits_per_year" "500+"
                    "certifications_tracked" "2000+"
                }

                qualityScorecard = component "Quality Scorecard" "Supplier performance metrics" "Spring Service" {
                    tags "Component"
                    properties {
                        "metrics" "Quality, Delivery, Cost, Service"
                    }
                }

                auditManager = component "Audit Manager" "Supplier audit tracking" "Spring Service" {
                    tags "Component"
                }

                certificationTracker = component "Certification Tracker" "Compliance certificate management" "Spring Service" {
                    tags "Component"
                    properties {
                        "types" "Organic, Non-GMO, Fair Trade"
                    }
                }

                recallManager = component "Recall Manager" "Product recall management" "Spring Service" {
                    tags "Component"
                    properties {
                        "traceability" "Lot-level"
                    }
                }
            }

            # ========================================
            # Infrastructure
            # ========================================

            eventBus = container "Event Bus" "Event streaming platform" "Apache Kafka" {
                tags "Container,Queue"
                properties {
                    "topics" "100+"
                    "throughput" "500K msg/sec"
                }
            }

            scmDatabase = container "SCM Database" "Supply chain data store" "PostgreSQL" {
                tags "Container,Database"
                properties {
                    "size" "5TB"
                    "replication" "Multi-region"
                }
            }

            documentStore = container "Document Store" "EDI and document storage" "MongoDB" {
                tags "Container,Database"
                properties {
                    "documents" "100M+"
                    "retention" "7 years"
                }
            }

            analyticsDB = container "Analytics Database" "Supply chain analytics" "Snowflake" {
                tags "Container,Database"
                properties {
                    "data" "Historical orders, shipments, performance"
                }
            }
        }

        # ========================================
        # Relationships - External Partners
        # ========================================

        supplierRep -> supplierPortal "Manages orders and catalog" {
            tags "Sync,External"
        }

        majorSupplier1 -> ediGateway "Sends EDI transactions" {
            tags "Async,External"
            properties {
                "transactions" "850, 855, 856, 810"
                "protocol" "AS2"
            }
        }

        majorSupplier2 -> apiGateway "API integration" {
            tags "Sync,External"
            properties {
                "protocol" "REST"
                "auth" "OAuth 2.0"
            }
        }

        majorSupplier3 -> ediGateway "Sends EDI transactions" {
            tags "Async,External"
        }

        smallSuppliers -> supplierPortal "Uses portal" {
            tags "Sync,External"
        }

        ediGateway -> majorSupplier1 "Sends purchase orders" {
            tags "Async,External"
            properties {
                "transaction" "EDI 850"
            }
        }

        apiGateway -> majorSupplier2 "Sends API orders" {
            tags "Sync,External"
        }

        # Logistics relationships
        logisticsManager -> primaryCarrier "Books shipments" {
            tags "Sync,External"
        }

        logisticsManager -> lastMileCarrier "Schedules deliveries" {
            tags "Sync,External"
        }

        logisticsManager -> coldChainCarrier "Cold chain shipments" {
            tags "Sync,External"
        }

        primaryCarrier -> logisticsManager "Shipment updates" {
            tags "Async,External"
            properties {
                "protocol" "Webhook"
            }
        }

        carrierRep -> logisticsManager "Manages carrier relationship" {
            tags "Sync,External"
        }

        # External services
        demandPlanning -> weatherService "Gets weather forecasts" {
            tags "Sync,External"
        }

        procurementEngine -> marketDataService "Gets commodity prices" {
            tags "Sync,External"
        }

        logisticsManager -> geoService "Route optimization" {
            tags "Sync,External"
        }

        # ========================================
        # Relationships - Internal Users
        # ========================================

        supplyChainManager -> analyticsDB "Views analytics" {
            tags "Sync"
        }

        supplyChainManager -> demandPlanning "Reviews forecasts" {
            tags "Sync"
        }

        procurementSpecialist -> procurementEngine "Manages orders" {
            tags "Sync"
        }

        procurementSpecialist -> qualityManager "Reviews supplier quality" {
            tags "Sync"
        }

        warehouseManager -> warehouseManager "Manages DC operations" {
            tags "Sync"
        }

        # ========================================
        # Relationships - Internal Systems
        # ========================================

        inventorySystem -> eventBus "Stock level events" {
            tags "Async,Internal"
            properties {
                "topic" "inventory.levels"
            }
        }

        eventBus -> inventoryOptimization "Triggers reorder" {
            tags "Async,Internal"
        }

        posSystem -> eventBus "Sales data" {
            tags "Async,Internal"
            properties {
                "topic" "sales.transactions"
            }
        }

        eventBus -> demandPlanning "Feeds demand signals" {
            tags "Async,Internal"
        }

        supplyChain -> analyticsSystem "Supply chain metrics" {
            tags "Async,Internal"
        }

        procurementEngine -> financeSystem "Invoice data" {
            tags "Batch,Internal"
            properties {
                "frequency" "Daily"
            }
        }

        # ========================================
        # Relationships - Internal Containers
        # ========================================

        # Integration flow
        ediGateway -> eventBus "Publishes EDI events" {
            tags "Async,Internal"
        }

        apiGateway -> eventBus "Publishes API events" {
            tags "Async,Internal"
        }

        supplierPortal -> procurementEngine "Submits orders" {
            tags "Sync,Internal"
        }

        # Procurement flow
        inventoryOptimization -> procurementEngine "Reorder recommendations" {
            tags "Async,Internal"
        }

        procurementEngine -> ediGateway "Sends POs via EDI" {
            tags "Async,Internal"
        }

        procurementEngine -> apiGateway "Sends POs via API" {
            tags "Sync,Internal"
        }

        procurementEngine -> scmDatabase "Stores orders" {
            tags "Sync,Internal"
        }

        # Demand planning flow
        demandPlanning -> inventoryOptimization "Forecast data" {
            tags "Async,Internal"
        }

        demandPlanning -> analyticsDB "Stores forecasts" {
            tags "Batch,Internal"
        }

        # Logistics flow
        procurementEngine -> logisticsManager "Creates shipments" {
            tags "Async,Internal"
        }

        logisticsManager -> warehouseManager "Delivery appointments" {
            tags "Sync,Internal"
        }

        logisticsManager -> eventBus "Shipment events" {
            tags "Async,Internal"
        }

        # Warehouse flow
        warehouseManager -> inventorySystem "Inventory updates" {
            tags "Async,Internal"
        }

        warehouseManager -> logisticsManager "Shipping requests" {
            tags "Async,Internal"
        }

        # Quality flow
        qualityManager -> procurementEngine "Supplier scores" {
            tags "Sync,Internal"
        }

        qualityManager -> scmDatabase "Stores audits" {
            tags "Sync,Internal"
        }

        # Document storage
        ediGateway -> documentStore "Archives EDI documents" {
            tags "Async,Internal"
        }

        # Component relationships
        ediTranslator -> ediValidator "Validates transaction" {
            tags "Sync,Internal"
        }

        ediValidator -> partnerManager "Gets partner config" {
            tags "Sync,Internal"
        }

        poGenerator -> approvalWorkflow "Routes for approval" {
            tags "Sync,Internal"
        }

        approvalWorkflow -> supplierSelection "Selects supplier" {
            tags "Sync,Internal"
        }

        supplierSelection -> contractManager "Checks contracts" {
            tags "Sync,Internal"
        }

        forecastEngine -> promotionPlanner "Gets promo impact" {
            tags "Sync,Internal"
        }

        forecastEngine -> seasonalityEngine "Gets seasonal factors" {
            tags "Sync,Internal"
        }

        forecastEngine -> externalFactors "Gets external data" {
            tags "Sync,Internal"
        }

        safetyStockCalc -> reorderPointEngine "Calculates reorder" {
            tags "Sync,Internal"
        }

        reorderPointEngine -> allocationEngine "Store allocation" {
            tags "Sync,Internal"
        }

        routeOptimizer -> carrierSelector "Selects carrier" {
            tags "Sync,Internal"
        }

        carrierSelector -> shipmentTracker "Tracks shipment" {
            tags "Async,Internal"
        }

        shipmentTracker -> appointmentScheduler "Schedules delivery" {
            tags "Sync,Internal"
        }

        receivingModule -> putawayModule "Directs putaway" {
            tags "Sync,Internal"
        }

        pickingModule -> shippingModule "Ready for ship" {
            tags "Sync,Internal"
        }

        laborManager -> pickingModule "Assigns pickers" {
            tags "Sync,Internal"
        }

        qualityScorecard -> auditManager "Triggers audits" {
            tags "Async,Internal"
        }

        certificationTracker -> qualityScorecard "Certification status" {
            tags "Sync,Internal"
        }
    }

    views {
        # ========================================
        # System Landscape
        # ========================================

        systemLandscape "supply-chain-landscape" "Supply chain ecosystem" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # System Context
        # ========================================

        systemContext supplyChain "supply-chain-context" "Supply Chain Platform context" {
            include *
            autoLayout lr 250 100
        }

        # ========================================
        # Container View
        # ========================================

        container supplyChain "supply-chain-containers" "Supply chain containers" {
            include *
            autoLayout tb 200 100
        }

        # ========================================
        # Component Views
        # ========================================

        component ediGateway "edi-components" "EDI Gateway components" {
            include *
            autoLayout lr 200 100
        }

        component procurementEngine "procurement-components" "Procurement Engine components" {
            include *
            autoLayout lr 200 100
        }

        component demandPlanning "demand-components" "Demand Planning components" {
            include *
            autoLayout lr 200 100
        }

        component logisticsManager "logistics-components" "Logistics Manager components" {
            include *
            autoLayout lr 200 100
        }

        component warehouseManager "warehouse-components" "Warehouse Management components" {
            include *
            autoLayout tb 150 100
        }

        component qualityManager "quality-components" "Quality Management components" {
            include *
            autoLayout lr 200 100
        }

        # ========================================
        # Dynamic Views
        # ========================================

        dynamic supplyChain "purchase-order-flow" "Purchase Order Creation" {
            inventorySystem -> eventBus "Stock below reorder point"
            eventBus -> inventoryOptimization "Trigger reorder analysis"

            # Parallel optimization
            {
                inventoryOptimization -> demandPlanning "Get forecast"
                inventoryOptimization -> safetyStockCalc "Calculate safety stock"
            }

            inventoryOptimization -> procurementEngine "Reorder recommendation"
            procurementEngine -> poGenerator "Generate PO"
            poGenerator -> approvalWorkflow "Route for approval"
            approvalWorkflow -> supplierSelection "Select supplier"

            # Parallel supplier checks
            {
                supplierSelection -> contractManager "Check contracts"
                supplierSelection -> qualityScorecard "Check quality score"
                supplierSelection -> marketDataService "Get current prices"
            }

            supplierSelection -> poGenerator "Selected supplier"
            poGenerator -> ediGateway "Send EDI 850"
            ediGateway -> majorSupplier1 "Transmit PO"

            autoLayout lr 250 100
        }

        dynamic supplyChain "order-acknowledgment-flow" "Order Acknowledgment Processing" {
            majorSupplier1 -> ediGateway "EDI 855 Acknowledgment"
            ediGateway -> ediTranslator "Translate EDI"
            ediTranslator -> ediValidator "Validate"
            ediValidator -> eventBus "Publish event"

            # Parallel processing
            {
                eventBus -> procurementEngine "Update PO status"
                eventBus -> documentStore "Archive document"
                eventBus -> analyticsDB "Log for analytics"
            }

            procurementEngine -> scmDatabase "Store acknowledgment"
            procurementEngine -> logisticsManager "Prepare for receipt"

            autoLayout tb 200 100
        }

        dynamic supplyChain "shipment-tracking-flow" "Shipment Tracking" {
            primaryCarrier -> apiGateway "Shipment status update"
            apiGateway -> eventBus "Publish tracking event"

            # Parallel tracking updates
            {
                eventBus -> shipmentTracker "Update shipment"
                eventBus -> warehouseManager "Update ETA"
                eventBus -> analyticsDB "Log tracking data"
            }

            shipmentTracker -> appointmentScheduler "Update appointment"
            appointmentScheduler -> warehouseManager "Confirm slot"

            # Parallel notifications
            {
                warehouseManager -> laborManager "Schedule crew"
                warehouseManager -> receivingModule "Prepare dock"
            }

            autoLayout lr 250 100
        }

        dynamic supplyChain "demand-forecast-flow" "Demand Forecasting Process" {
            posSystem -> eventBus "Sales transactions"
            eventBus -> demandPlanning "Sales data"

            # Parallel data gathering
            {
                demandPlanning -> weatherService "Weather forecast"
                demandPlanning -> analyticsDB "Historical patterns"
                demandPlanning -> promotionPlanner "Planned promotions"
            }

            forecastEngine -> seasonalityEngine "Apply seasonality"
            forecastEngine -> externalFactors "Apply external factors"
            forecastEngine -> demandPlanning "Generate forecast"

            # Parallel distribution
            {
                demandPlanning -> inventoryOptimization "Update reorder points"
                demandPlanning -> analyticsDB "Store forecast"
                demandPlanning -> procurementEngine "Procurement planning"
            }

            autoLayout tb 200 100
        }

        dynamic warehouseManager "receiving-flow" "Warehouse Receiving Process" {
            primaryCarrier -> warehouseManager "Truck arrival"
            receivingModule -> shipmentTracker "Verify appointment"

            # Parallel receiving
            {
                receivingModule -> qualityManager "Quality inspection"
                receivingModule -> scmDatabase "Log receipt"
            }

            qualityManager -> receivingModule "Quality approved"
            receivingModule -> putawayModule "Direct putaway"
            putawayModule -> inventorySystem "Update inventory"

            # Parallel updates
            {
                putawayModule -> scmDatabase "Update locations"
                putawayModule -> eventBus "Publish receipt event"
            }

            autoLayout lr 250 100
        }

        dynamic supplyChain "supplier-evaluation-flow" "Supplier Performance Evaluation" {
            eventBus -> qualityManager "Delivery completed event"

            # Parallel metric collection
            {
                qualityScorecard -> scmDatabase "Get delivery data"
                qualityScorecard -> documentStore "Get quality reports"
                qualityScorecard -> analyticsDB "Get historical performance"
            }

            qualityScorecard -> auditManager "Check audit status"
            certificationTracker -> qualityScorecard "Certification status"
            qualityScorecard -> procurementEngine "Update supplier score"

            # Parallel actions based on score
            {
                procurementEngine -> supplierSelection "Update routing weights"
                procurementEngine -> supplierPortal "Notify supplier"
            }

            autoLayout tb 200 100
        }

        # ========================================
        # Deployment View
        # ========================================

        deployment supplyChain "Production" "supply-chain-deployment" "Production deployment" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # Styles
        # ========================================

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "Partner" {
                background "#95A5A6"
                color "#FFFFFF"
                shape RoundedBox
                border Dotted
            }

            element "Integration" {
                shape Hexagon
                background "#16A085"
                color "#FFFFFF"
            }

            relationship "External" {
                color "#95A5A6"
                style Dashed
            }

            relationship "Batch" {
                style Dotted
                color "#3498DB"
            }
        }
    }
}