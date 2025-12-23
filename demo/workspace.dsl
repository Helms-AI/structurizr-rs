workspace "Big Bank plc" "An example C4 model for a fictional internet banking system" {

    model {
        // People
        customer = person "Personal Banking Customer" "A customer of the bank with personal accounts"

        // External systems
        mainframe = softwareSystem "Mainframe Banking System" "Stores all core banking information" {
            tags "External"
        }
        email = softwareSystem "E-mail System" "Internal Microsoft Exchange email system" {
            tags "External"
        }

        // The main system
        internetBanking = softwareSystem "Internet Banking System" "Allows customers to view account info and make payments" {
            webapp = container "Web Application" "Delivers static content and the single page application" "Java/Spring MVC"
            spa = container "Single-Page Application" "Provides banking functionality via browser" "JavaScript/React"
            mobileApp = container "Mobile App" "Provides banking functionality via mobile" "React Native"
            api = container "API Application" "Provides internet banking via JSON/HTTPS API" "Java/Spring Boot"
            database = container "Database" "Stores user registration, auth, and access logs" "PostgreSQL"
        }

        // Relationships
        customer -> webapp "Visits bigbank.com using" "HTTPS"
        customer -> spa "Views account and makes payments using"
        customer -> mobileApp "Views account and makes payments using"

        webapp -> spa "Delivers to the customer's browser"
        spa -> api "Makes API calls to" "JSON/HTTPS"
        mobileApp -> api "Makes API calls to" "JSON/HTTPS"

        api -> database "Reads from and writes to" "SQL/TCP"
        api -> mainframe "Gets account info from" "XML/HTTPS"
        api -> email "Sends email using" "SMTP"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of the banking landscape" {
            include *
            autoLayout
        }

        systemContext internetBanking "SystemContext" "System Context for Internet Banking" {
            include *
            autoLayout
        }

        container internetBanking "Containers" "Container diagram for Internet Banking" {
            include *
            autoLayout
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
            relationship "Relationship" {
                color "#707070"
            }
        }
    }
}
