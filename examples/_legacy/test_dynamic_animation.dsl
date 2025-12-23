workspace "E-Commerce System" "A simple e-commerce system demonstrating dynamic views" {
    model {
        customer = person "Customer" "A customer of the e-commerce system"

        ecommerceSystem = softwareSystem "E-Commerce System" "Allows customers to purchase products" {
            webApp = container "Web Application" "Serves the web interface" "React"
            apiGateway = container "API Gateway" "Routes API requests" "Node.js"
            orderService = container "Order Service" "Handles order processing" "Java/Spring"
            paymentService = container "Payment Service" "Processes payments" "Python"
            database = container "Database" "Stores order data" "PostgreSQL"
        }

        externalPaymentGateway = softwareSystem "Payment Gateway" "External payment processor" "Stripe"

        customer -> webApp "Views products and places orders using"
        webApp -> apiGateway "Makes API calls to" "HTTPS/JSON"
        apiGateway -> orderService "Routes order requests to" "REST"
        apiGateway -> paymentService "Routes payment requests to" "REST"
        orderService -> database "Reads from and writes to" "JDBC"
        paymentService -> externalPaymentGateway "Processes payments via" "HTTPS/REST"
    }

    views {
        systemContext ecommerceSystem "SystemContext" "System context view for e-commerce" {
            include *
            autoLayout
        }

        container ecommerceSystem "ContainerView" "Container view for e-commerce" {
            include *
            autoLayout
        }

        dynamic ecommerceSystem "OrderPlacement" "The process of placing an order" {
            customer -> webApp "1. Selects products and clicks checkout"
            webApp -> apiGateway "2. Submits order request"
            apiGateway -> orderService "3. Creates new order"
            orderService -> database "4. Saves order details"
            apiGateway -> paymentService "5. Initiates payment"
            paymentService -> externalPaymentGateway "6. Processes payment"
            externalPaymentGateway -> paymentService "7. Returns payment confirmation"
            paymentService -> apiGateway "8. Confirms payment success"
            apiGateway -> webApp "9. Returns order confirmation"
            webApp -> customer "10. Displays order confirmation"
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
            }
            relationship "Relationship" {
                color "#707070"
                routing Orthogonal
            }
        }
    }
}
