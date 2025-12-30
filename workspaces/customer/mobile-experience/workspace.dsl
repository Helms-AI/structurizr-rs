workspace "FreshMart Mobile Experience" "Customer mobile app with PWA and offline capabilities" {

    !const CORP_COLOR "#00539C"
    !const MOBILE_COLOR "#27AE60"
    !const OFFLINE_COLOR "#F39C12"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # People
        customer = person "FreshMart Customer" "Mobile app user" {
            tags "Person,Customer"
            properties {
                "devices" "iOS, Android, Web"
            }
        }

        productManager = person "Product Manager" "Manages mobile features" {
            tags "Person"
        }

        # External Systems
        loyaltySystem = softwareSystem "Loyalty Platform" "Customer rewards" {
            tags "Software System,External"
        }

        inventorySystem = softwareSystem "Inventory Platform" "Stock availability" {
            tags "Software System,External"
        }

        paymentGateway = softwareSystem "Payment Gateway" "Payment processing" {
            tags "Software System,External"
        }

        analyticsSystem = softwareSystem "Analytics Platform" "User analytics" {
            tags "Software System,External"
        }

        notificationService = softwareSystem "Push Notification Service" "Firebase/APNs" {
            tags "Software System,External"
        }

        mapService = softwareSystem "Maps Service" "Store locator" {
            tags "Software System,External"
        }

        # Mobile Experience Platform
        mobileExperience = softwareSystem "Mobile Experience Platform" "Customer mobile application" {
            tags "Software System,Core"

            properties {
                "users" "10M+"
                "platforms" "iOS, Android, PWA"
                "offline" "Full offline support"
                "perspective_business" "30% of digital revenue"
                "perspective_technical" "PWA with native bridges"
                "perspective_operations" "99.9% crash-free sessions"
            }

            # Mobile App (Client)
            mobileApp = container "Mobile Application" "Cross-platform mobile app" "React Native" {
                tags "Container,Mobile"
                properties {
                    "framework" "React Native"
                    "state" "Redux + RTK Query"
                }

                homeScreen = component "Home Screen" "Dashboard and promotions" "React Component" {
                    tags "Component"
                }

                shopScreen = component "Shop Screen" "Product browsing" "React Component" {
                    tags "Component"
                }

                cartManager = component "Cart Manager" "Shopping cart" "Redux Slice" {
                    tags "Component"
                }

                checkoutFlow = component "Checkout Flow" "Purchase flow" "React Component" {
                    tags "Component"
                }

                accountScreen = component "Account Screen" "Profile and settings" "React Component" {
                    tags "Component"
                }

                storeLocator = component "Store Locator" "Find stores" "React Component" {
                    tags "Component"
                }

                digitalReceipts = component "Digital Receipts" "Receipt history" "React Component" {
                    tags "Component"
                }
            }

            # Offline Engine
            offlineEngine = container "Offline Engine" "Offline-first data layer" "React Native" {
                tags "Container,Mobile"
                properties {
                    "storage" "WatermelonDB"
                    "sync" "Bidirectional"
                }

                localDB = component "Local Database" "Offline storage" "WatermelonDB" {
                    tags "Component,Database"
                }

                syncManager = component "Sync Manager" "Background sync" "Native Module" {
                    tags "Component"
                }

                cacheManager = component "Cache Manager" "Content caching" "AsyncStorage" {
                    tags "Component,Cache"
                }

                conflictResolver = component "Conflict Resolver" "Sync conflicts" "Custom Logic" {
                    tags "Component"
                }
            }

            # PWA Container
            pwaApp = container "Progressive Web App" "Web-based mobile experience" "React" {
                tags "Container,Web"
                properties {
                    "lighthouse" "95+ score"
                    "installable" "Yes"
                }

                serviceWorker = component "Service Worker" "Offline and caching" "Workbox" {
                    tags "Component"
                }

                appShell = component "App Shell" "Core UI shell" "React Component" {
                    tags "Component"
                }

                indexedDB = component "IndexedDB Store" "Browser storage" "Dexie.js" {
                    tags "Component,Database"
                }
            }

            # Backend for Frontend (BFF)
            mobileBFF = container "Mobile BFF" "Backend for frontend" "Node.js/Express" {
                tags "Container,API"
                properties {
                    "pattern" "BFF"
                    "caching" "Redis"
                }

                graphqlGateway = component "GraphQL Gateway" "Unified API" "Apollo Server" {
                    tags "Component"
                }

                dataAggregator = component "Data Aggregator" "Combines data sources" "Node Service" {
                    tags "Component"
                }

                responseOptimizer = component "Response Optimizer" "Mobile-optimized responses" "Node Service" {
                    tags "Component"
                }

                rateLimiter = component "Rate Limiter" "Request throttling" "Express Middleware" {
                    tags "Component"
                }
            }

            # Content Service
            contentService = container "Content Service" "Dynamic content" "Java/Spring Boot" {
                tags "Container"

                bannerService = component "Banner Service" "Promotional banners" "Spring Service" {
                    tags "Component"
                }

                featureFlagService = component "Feature Flag Service" "Feature toggles" "LaunchDarkly" {
                    tags "Component"
                }

                personalizationBridge = component "Personalization Bridge" "Content targeting" "Spring Service" {
                    tags "Component"
                }
            }

            # Search Service
            searchService = container "Search Service" "Product search" "Elasticsearch" {
                tags "Container"

                searchEngine = component "Search Engine" "Full-text search" "Elasticsearch" {
                    tags "Component"
                }

                autocomplete = component "Autocomplete" "Search suggestions" "Elasticsearch" {
                    tags "Component"
                }

                filterEngine = component "Filter Engine" "Faceted search" "Elasticsearch" {
                    tags "Component"
                }
            }

            # CDN
            cdn = container "Content Delivery Network" "Static assets and images" "CloudFront" {
                tags "Container,Infrastructure"
            }

            # Real-time Service
            realtimeService = container "Real-time Service" "Push and real-time updates" "Socket.io" {
                tags "Container"

                websocketServer = component "WebSocket Server" "Real-time connection" "Socket.io" {
                    tags "Component"
                }

                eventBroadcaster = component "Event Broadcaster" "Push events" "Node Service" {
                    tags "Component"
                }
            }

            # Event Bus
            eventBus = container "Event Bus" "Mobile events" "Apache Kafka" {
                tags "Container,Queue"
            }

            # Cache
            cache = container "API Cache" "Response caching" "Redis" {
                tags "Container,Cache"
            }
        }

        # Relationships
        customer -> mobileApp "Uses native app" {
            tags "Sync"
        }

        customer -> pwaApp "Uses web app" {
            tags "Sync"
        }

        productManager -> contentService "Manages content" {
            tags "Sync"
        }

        # App to BFF
        mobileApp -> mobileBFF "API requests" {
            tags "Sync"
            properties {
                "protocol" "GraphQL"
            }
        }

        pwaApp -> mobileBFF "API requests" {
            tags "Sync"
        }

        # BFF to backend services
        mobileBFF -> loyaltySystem "Loyalty data" {
            tags "Sync"
        }

        mobileBFF -> inventorySystem "Stock data" {
            tags "Sync"
        }

        mobileBFF -> paymentGateway "Process payment" {
            tags "Sync,Critical"
        }

        mobileBFF -> contentService "Get content" {
            tags "Sync"
        }

        mobileBFF -> searchService "Search products" {
            tags "Sync"
        }

        mobileBFF -> cache "Cache responses" {
            tags "Sync,Internal"
        }

        # Real-time
        mobileApp -> realtimeService "WebSocket connection" {
            tags "Async"
        }

        realtimeService -> notificationService "Send push" {
            tags "Async"
        }

        # Content
        mobileApp -> cdn "Load assets" {
            tags "Sync"
        }

        pwaApp -> cdn "Load assets" {
            tags "Sync"
        }

        # Analytics
        mobileApp -> eventBus "User events" {
            tags "Async"
        }

        eventBus -> analyticsSystem "Forward events" {
            tags "Async"
        }

        # Maps
        storeLocator -> mapService "Map data" {
            tags "Sync"
        }

        # Internal mobile relationships
        mobileApp -> offlineEngine "Offline operations" {
            tags "Sync,Internal"
        }

        offlineEngine -> mobileBFF "Sync data" {
            tags "Async,Internal"
        }

        pwaApp -> serviceWorker "Offline requests" {
            tags "Sync,Internal"
        }

        # Component relationships - Mobile App
        homeScreen -> cartManager "Cart state" {
            tags "Sync,Internal"
        }

        shopScreen -> cartManager "Add to cart" {
            tags "Sync,Internal"
        }

        cartManager -> checkoutFlow "Checkout" {
            tags "Sync,Internal"
        }

        accountScreen -> digitalReceipts "View receipts" {
            tags "Sync,Internal"
        }

        # Component relationships - Offline
        localDB -> syncManager "Sync changes" {
            tags "Async,Internal"
        }

        syncManager -> conflictResolver "Resolve" {
            tags "Sync,Internal"
        }

        cacheManager -> localDB "Cache content" {
            tags "Sync,Internal"
        }

        # Component relationships - PWA
        serviceWorker -> indexedDB "Cache data" {
            tags "Sync,Internal"
        }

        appShell -> serviceWorker "Offline fallback" {
            tags "Sync,Internal"
        }

        # Component relationships - BFF
        graphqlGateway -> dataAggregator "Aggregate" {
            tags "Sync,Internal"
        }

        dataAggregator -> responseOptimizer "Optimize" {
            tags "Sync,Internal"
        }

        rateLimiter -> graphqlGateway "Throttle" {
            tags "Sync,Internal"
        }

        # Component relationships - Content
        bannerService -> featureFlagService "Check flags" {
            tags "Sync,Internal"
        }

        bannerService -> personalizationBridge "Personalize" {
            tags "Sync,Internal"
        }

        # Component relationships - Search
        searchEngine -> autocomplete "Suggestions" {
            tags "Sync,Internal"
        }

        searchEngine -> filterEngine "Apply filters" {
            tags "Sync,Internal"
        }

        # Component relationships - Real-time
        websocketServer -> eventBroadcaster "Broadcast" {
            tags "Async,Internal"
        }

        # Deployment Infrastructure
        clientDevices = deploymentNode "Client Devices" "User devices" {
            tags "Edge"
            iosAndroid = deploymentNode "iOS/Android" "Mobile devices" {
                containerInstance mobileApp
                containerInstance offlineEngine
            }
            webBrowser = deploymentNode "Web Browser" "Desktop/mobile browsers" {
                containerInstance pwaApp
            }
        }

        cloudInfra = deploymentNode "Cloud Infrastructure" "AWS Cloud" {
            tags "Cloud"
            edgeZone = deploymentNode "Edge Zone" "CDN edge locations" {
                containerInstance cdn
            }
            apiZone = deploymentNode "API Zone" "API services" {
                containerInstance mobileBFF
                containerInstance realtimeService
            }
            serviceZone = deploymentNode "Service Zone" "Backend services" {
                containerInstance contentService
                containerInstance searchService
            }
            dataZone = deploymentNode "Data Zone" "Data infrastructure" {
                containerInstance eventBus
                containerInstance cache
            }
        }
    }

    views {
        systemLandscape "mobile-landscape" "Mobile ecosystem" {
            include *
            autoLayout lr 300 100
        }

        systemContext mobileExperience "mobile-context" "Mobile Experience context" {
            include *
            autoLayout lr 250 100
        }

        container mobileExperience "mobile-containers" "Mobile Experience containers" {
            include *
            autoLayout tb 200 100
        }

        component mobileApp "app-components" "Mobile App components" {
            include *
            autoLayout lr 200 100
        }

        component offlineEngine "offline-components" "Offline Engine components" {
            include *
            autoLayout lr 200 100
        }

        component pwaApp "pwa-components" "PWA components" {
            include *
            autoLayout lr 200 100
        }

        component mobileBFF "bff-components" "Mobile BFF components" {
            include *
            autoLayout lr 200 100
        }

        component searchService "search-components" "Search Service components" {
            include *
            autoLayout lr 200 100
        }

        # Dynamic views
        dynamic mobileExperience "shopping-flow" "Mobile Shopping Flow" {
            customer -> mobileApp "Browse products"
            mobileApp -> homeScreen "View home"
            homeScreen -> mobileBFF "Load content"

            # Parallel content loading
            {
                mobileBFF -> contentService "Get banners"
                mobileBFF -> loyaltySystem "Get offers"
                mobileBFF -> cache "Check cache"
            }

            customer -> shopScreen "Browse category"
            shopScreen -> mobileBFF "Search products"
            mobileBFF -> searchService "Full-text search"

            # Parallel search
            {
                searchEngine -> autocomplete "Suggestions"
                searchEngine -> filterEngine "Apply filters"
            }

            shopScreen -> cartManager "Add to cart"
            cartManager -> offlineEngine "Save cart locally"
            localDB -> syncManager "Sync cart"

            customer -> checkoutFlow "Checkout"
            checkoutFlow -> mobileBFF "Submit order"

            # Parallel order processing
            {
                mobileBFF -> inventorySystem "Check stock"
                mobileBFF -> loyaltySystem "Apply points"
            }

            mobileBFF -> paymentGateway "Process payment"
            paymentGateway -> mobileBFF "Payment confirmed"
            mobileBFF -> mobileApp "Order confirmed"

            # Parallel post-order
            {
                eventBus -> analyticsSystem "Track purchase"
                realtimeService -> notificationService "Confirmation push"
            }

            autoLayout lr 250 100
        }

        dynamic offlineEngine "offline-sync-flow" "Offline to Online Sync" {
            customer -> mobileApp "Add items offline"
            mobileApp -> cartManager "Update cart"
            cartManager -> localDB "Store locally"

            mobileApp -> syncManager "Network restored"
            syncManager -> localDB "Get pending changes"
            syncManager -> mobileBFF "Sync cart"

            # Parallel sync
            {
                mobileBFF -> inventorySystem "Verify stock"
                mobileBFF -> loyaltySystem "Sync offers"
            }

            mobileBFF -> syncManager "Sync response"
            syncManager -> conflictResolver "Check conflicts"
            conflictResolver -> localDB "Resolve and update"
            syncManager -> mobileApp "Sync complete"

            autoLayout tb 200 100
        }

        dynamic pwaApp "pwa-offline-flow" "PWA Offline Flow" {
            customer -> pwaApp "Open PWA offline"
            pwaApp -> serviceWorker "Intercept request"
            serviceWorker -> indexedDB "Get cached data"
            indexedDB -> appShell "Return cached"
            appShell -> customer "Display cached content"

            customer -> pwaApp "Network restored"
            serviceWorker -> mobileBFF "Fetch updates"

            # Parallel content refresh
            {
                mobileBFF -> contentService "Fresh content"
                mobileBFF -> searchService "Updated catalog"
            }

            serviceWorker -> indexedDB "Update cache"
            pwaApp -> customer "Fresh content"

            autoLayout lr 250 100
        }

        dynamic realtimeService "push-notification-flow" "Push Notification Flow" {
            loyaltySystem -> eventBus "Offer available"
            eventBus -> realtimeService "Consume event"
            realtimeService -> websocketServer "Check connections"

            # Parallel notification
            {
                websocketServer -> mobileApp "In-app notification"
                eventBroadcaster -> notificationService "Push notification"
            }

            notificationService -> customer "Device notification"
            customer -> mobileApp "Open from notification"
            mobileApp -> mobileBFF "Load offer details"

            autoLayout tb 200 100
        }

        deployment mobileExperience "Production" "mobile-deployment" "Production deployment" {
            include *
            autoLayout lr 300 100
        }

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "Mobile" {
                shape MobileDevicePortrait
                background "#27AE60"
                color "#FFFFFF"
            }

            element "Web" {
                shape WebBrowser
                background "#3498DB"
                color "#FFFFFF"
            }
        }
    }
}