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

        // Relationships
        customer -> posTerminal "Places orders using"
        barista -> orderQueue "Views and fulfills orders from"
        posTerminal -> orderQueue "Submits orders to" "REST/JSON"
        orderQueue -> inventoryService "Reserves ingredients via" "gRPC"
        orderQueue -> database "Persists orders to" "SQL"
        inventoryService -> database "Reads/writes inventory to" "SQL"
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
