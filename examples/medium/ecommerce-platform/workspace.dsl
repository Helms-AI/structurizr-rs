// E-commerce Platform
// Medium complexity online retail platform with microservices architecture

workspace "E-commerce Platform" "Online retail platform with microservices" {

    !const COMPANY "ShopMax"
    !const DB_TECH "PostgreSQL"
    !const CACHE_TECH "Redis"
    !const SEARCH_TECH "Elasticsearch"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        // Users
        customer = person "Customer" "Browses products and places orders"
        admin = person "Admin" "Manages products, orders, and users"
        warehouseStaff = person "Warehouse Staff" "Fulfills and ships orders"
        supportAgent = person "Support Agent" "Handles customer inquiries"

        // Main E-commerce System
        ecommerceSystem = softwareSystem "${COMPANY} Platform" "Online retail platform" {
            // Frontend
            webStore = container "Web Store" "Customer-facing storefront" "React/Next.js"
            mobileApp = container "Mobile App" "iOS and Android shopping app" "React Native"
            adminPortal = container "Admin Portal" "Back-office management" "React"

            // API Layer
            apiGateway = container "API Gateway" "Routes and authenticates requests" "Kong"

            // Core Services
            orderService = container "Order Service" "Manages orders and checkout" "Java/Spring Boot" {
                orderController = component "Order Controller" "REST API endpoints" "Spring MVC"
                paymentHandler = component "Payment Handler" "Payment processing logic" "Spring"
                inventoryChecker = component "Inventory Checker" "Stock validation" "Spring"
            }

            userService = container "User Service" "Authentication and profiles" "Node.js/Express"
            productCatalog = container "Product Catalog" "Product information and pricing" "Go"
            inventoryService = container "Inventory Service" "Stock management" "Python/FastAPI"
            searchService = container "Search Service" "Product search and filtering" "Java/Spring Boot"
            notificationService = container "Notification Service" "Email and push notifications" "Node.js"

            // Data Stores
            primaryDb = container "Primary Database" "Orders, users, products" "${DB_TECH}"
            searchIndex = container "Search Index" "Product search data" "${SEARCH_TECH}"
            cache = container "Cache" "Session and product cache" "${CACHE_TECH}"
        }

        // External Systems
        paymentGateway = softwareSystem "Payment Gateway" "Stripe payment processing" {
            tags "External"
        }

        shippingProvider = softwareSystem "Shipping Provider" "FedEx/UPS integration" {
            tags "External"
        }

        emailService = softwareSystem "Email Service" "SendGrid email delivery" {
            tags "External"
        }

        analytics = softwareSystem "Analytics" "Google Analytics tracking" {
            tags "External"
        }

        fraudDetection = softwareSystem "Fraud Detection" "Sift fraud prevention" {
            tags "External"
        }

        // User Relationships
        customer -> ecommerceSystem "Browses and purchases products"
        admin -> ecommerceSystem "Manages platform"
        warehouseStaff -> ecommerceSystem "Processes shipments"
        supportAgent -> ecommerceSystem "Resolves customer issues"

        // Frontend to API
        webStore -> apiGateway "Makes API calls" "HTTPS/REST"
        mobileApp -> apiGateway "Makes API calls" "HTTPS/REST"
        adminPortal -> apiGateway "Makes API calls" "HTTPS/REST"

        // API Gateway to Services
        apiGateway -> orderService "Routes order requests" "gRPC"
        apiGateway -> userService "Routes auth requests" "gRPC"
        apiGateway -> productCatalog "Routes product requests" "gRPC"
        apiGateway -> searchService "Routes search requests" "gRPC"

        // Service Interactions
        orderService -> inventoryService "Checks stock" "gRPC"
        orderService -> notificationService "Sends order updates" "AMQP"
        productCatalog -> searchService "Syncs product data" "AMQP"
        searchService -> searchIndex "Queries products" "REST"

        // Data Access
        orderService -> primaryDb "Stores orders" "SQL"
        userService -> primaryDb "Stores users" "SQL"
        productCatalog -> primaryDb "Stores products" "SQL"
        inventoryService -> primaryDb "Stores inventory" "SQL"
        userService -> cache "Caches sessions" "Redis"
        productCatalog -> cache "Caches products" "Redis"

        // External Integrations
        orderService -> paymentGateway "Processes payments" "HTTPS"
        orderService -> fraudDetection "Validates orders" "HTTPS"
        orderService -> shippingProvider "Creates shipments" "HTTPS"
        notificationService -> emailService "Sends emails" "HTTPS"
        webStore -> analytics "Tracks events" "HTTPS"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of ${COMPANY} ecosystem" {
            include *
            autoLayout tb
        }

        systemContext ecommerceSystem "SystemContext" "System context for ${COMPANY}" {
            include *
            autoLayout tb
        }

        container ecommerceSystem "Containers" "Container architecture" {
            include *
            autoLayout tb
        }

        component orderService "Components" "Order Service internals" {
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
