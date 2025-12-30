workspace "FreshMart Loyalty & Rewards" "Customer loyalty program with personalization and gamification" {

    !const CORP_COLOR "#00539C"
    !const CUSTOMER_COLOR "#2ECC71"
    !const REWARD_COLOR "#F39C12"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # People
        customer = person "FreshMart Customer" "Loyalty program member" {
            tags "Person,Customer"
            properties {
                "members" "25M+"
            }
        }

        marketingManager = person "Marketing Manager" "Manages loyalty campaigns" {
            tags "Person"
        }

        dataAnalyst = person "Data Analyst" "Analyzes customer behavior" {
            tags "Person"
        }

        # External Systems
        posSystem = softwareSystem "POS System" "Transaction processing" {
            tags "Software System,External"
        }

        mobileApp = softwareSystem "Mobile App" "Customer mobile application" {
            tags "Software System,External"
        }

        ecommerceSystem = softwareSystem "E-Commerce Platform" "Online shopping" {
            tags "Software System,External"
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Business intelligence" {
            tags "Software System,External"
        }

        emailService = softwareSystem "Email Service" "Email delivery" {
            tags "Software System,External"
        }

        smsService = softwareSystem "SMS Gateway" "SMS delivery" {
            tags "Software System,External"
        }

        # Loyalty Platform
        loyaltyPlatform = softwareSystem "Loyalty & Rewards Platform" "Customer engagement and loyalty" {
            tags "Software System,Core"

            properties {
                "members" "25M+"
                "transactions_per_day" "5M+"
                "points_issued_daily" "500M+"
                "redemption_rate" "85%"
                "perspective_business" "Drives 60% of revenue from members"
                "perspective_technical" "Real-time personalization engine"
                "perspective_operations" "Multi-channel engagement"
            }

            # Profile Management
            profileService = container "Profile Service" "Customer profile management" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "profiles" "25M+"
                }

                profileManager = component "Profile Manager" "CRUD operations" "Spring Service" {
                    tags "Component"
                }

                preferenceEngine = component "Preference Engine" "Customer preferences" "Spring Service" {
                    tags "Component"
                }

                consentManager = component "Consent Manager" "Privacy consent" "Spring Service" {
                    tags "Component"
                }

                customer360 = component "Customer 360" "Unified customer view" "Aggregator" {
                    tags "Component"
                }
            }

            # Points Engine
            pointsEngine = container "Points Engine" "Points earn and burn" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "rules" "500+"
                    "processing" "Real-time"
                }

                earnEngine = component "Earn Engine" "Points accrual" "Rule Engine" {
                    tags "Component"
                    properties {
                        "rules" "Base, Bonus, Promo"
                    }
                }

                burnEngine = component "Burn Engine" "Points redemption" "Spring Service" {
                    tags "Component"
                }

                balanceManager = component "Balance Manager" "Points balance" "Spring Service" {
                    tags "Component"
                }

                expirationEngine = component "Expiration Engine" "Points expiration" "Scheduler" {
                    tags "Component"
                }
            }

            # Tier Management
            tierService = container "Tier Service" "Membership tier management" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "tiers" "4"
                }

                tierCalculator = component "Tier Calculator" "Tier qualification" "Spring Service" {
                    tags "Component"
                    properties {
                        "tiers" "Bronze, Silver, Gold, Platinum"
                    }
                }

                benefitsManager = component "Benefits Manager" "Tier benefits" "Spring Service" {
                    tags "Component"
                }
            }

            # Personalization Engine
            personalizationEngine = container "Personalization Engine" "AI-powered personalization" "Python/TensorFlow" {
                tags "Container,ML"
                properties {
                    "models" "15+"
                    "latency" "<100ms"
                }

                recommendationModel = component "Recommendation Model" "Product recommendations" "Neural CF" {
                    tags "Component,ML"
                }

                segmentationModel = component "Segmentation Model" "Customer clustering" "K-Means" {
                    tags "Component,ML"
                }

                propensityModel = component "Propensity Model" "Purchase likelihood" "XGBoost" {
                    tags "Component,ML"
                }

                churnModel = component "Churn Model" "Churn prediction" "Random Forest" {
                    tags "Component,ML"
                }
            }

            # Offer Management
            offerService = container "Offer Service" "Personalized offers" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "offers" "10,000+"
                }

                offerEngine = component "Offer Engine" "Offer matching" "Rule Engine" {
                    tags "Component"
                }

                targetingEngine = component "Targeting Engine" "Audience targeting" "Spring Service" {
                    tags "Component"
                }

                redemptionTracker = component "Redemption Tracker" "Offer usage" "Spring Service" {
                    tags "Component"
                }
            }

            # Campaign Management
            campaignService = container "Campaign Service" "Marketing campaigns" "Java/Spring Boot" {
                tags "Container"

                campaignOrchestrator = component "Campaign Orchestrator" "Campaign workflow" "Camunda" {
                    tags "Component"
                }

                abTestEngine = component "A/B Test Engine" "Experiment management" "Spring Service" {
                    tags "Component"
                }

                contentManager = component "Content Manager" "Campaign content" "Spring Service" {
                    tags "Component"
                }
            }

            # Gamification
            gamificationService = container "Gamification Service" "Engagement gamification" "Node.js" {
                tags "Container"
                properties {
                    "challenges" "100+"
                    "badges" "50+"
                }

                challengeEngine = component "Challenge Engine" "Challenge management" "Node Service" {
                    tags "Component"
                }

                badgeEngine = component "Badge Engine" "Badge awards" "Node Service" {
                    tags "Component"
                }

                leaderboardService = component "Leaderboard Service" "Rankings" "Redis" {
                    tags "Component"
                }
            }

            # Communication
            communicationHub = container "Communication Hub" "Multi-channel messaging" "Java/Spring Boot" {
                tags "Container"

                notificationEngine = component "Notification Engine" "Push notifications" "Firebase" {
                    tags "Component"
                }

                emailEngine = component "Email Engine" "Email campaigns" "Spring Service" {
                    tags "Component"
                }

                smsEngine = component "SMS Engine" "SMS messaging" "Spring Service" {
                    tags "Component"
                }
            }

            # APIs
            loyaltyAPI = container "Loyalty API" "External API" "Java/Spring Boot" {
                tags "Container,API"
            }

            # Event Bus
            eventBus = container "Event Bus" "Event streaming" "Apache Kafka" {
                tags "Container,Queue"
            }

            # Databases
            profileDatabase = container "Profile Database" "Customer data" "PostgreSQL" {
                tags "Container,Database"
            }

            pointsLedger = container "Points Ledger" "Points transactions" "ScyllaDB" {
                tags "Container,Database"
                properties {
                    "pattern" "Ledger"
                }
            }

            featureStore = container "Feature Store" "ML features" "Redis" {
                tags "Container,Cache,ML"
            }
        }

        # Relationships
        customer -> mobileApp "Uses app" {
            tags "Sync"
        }

        marketingManager -> campaignService "Manages campaigns" {
            tags "Sync"
        }

        dataAnalyst -> analyticsSystem "Analyzes data" {
            tags "Sync"
        }

        # System integrations
        posSystem -> eventBus "Transaction events" {
            tags "Async"
            properties {
                "topic" "transactions.completed"
            }
        }

        mobileApp -> loyaltyAPI "Loyalty operations" {
            tags "Sync"
        }

        ecommerceSystem -> loyaltyAPI "Online loyalty" {
            tags "Sync"
        }

        loyaltyPlatform -> analyticsSystem "Member metrics" {
            tags "Async"
        }

        communicationHub -> emailService "Send emails" {
            tags "Async"
        }

        communicationHub -> smsService "Send SMS" {
            tags "Async"
        }

        # Internal relationships
        loyaltyAPI -> profileService "Profile operations" {
            tags "Sync,Internal"
        }

        loyaltyAPI -> pointsEngine "Points operations" {
            tags "Sync,Internal"
        }

        eventBus -> pointsEngine "Transaction events" {
            tags "Async,Internal"
        }

        pointsEngine -> tierService "Tier check" {
            tags "Sync,Internal"
        }

        pointsEngine -> pointsLedger "Store transaction" {
            tags "Sync,Internal"
        }

        profileService -> profileDatabase "Store profile" {
            tags "Sync,Internal"
        }

        profileService -> personalizationEngine "Get recommendations" {
            tags "Sync,Internal"
        }

        personalizationEngine -> featureStore "Get features" {
            tags "Sync,Internal"
        }

        offerService -> personalizationEngine "Get targeting" {
            tags "Sync,Internal"
        }

        campaignService -> offerService "Create offers" {
            tags "Sync,Internal"
        }

        campaignService -> communicationHub "Send messages" {
            tags "Async,Internal"
        }

        gamificationService -> pointsEngine "Award bonus points" {
            tags "Sync,Internal"
        }

        gamificationService -> eventBus "Publish achievements" {
            tags "Async,Internal"
        }

        # Component relationships
        profileManager -> preferenceEngine "Get preferences" {
            tags "Sync,Internal"
        }

        preferenceEngine -> consentManager "Check consent" {
            tags "Sync,Internal"
        }

        profileManager -> customer360 "Build 360 view" {
            tags "Sync,Internal"
        }

        earnEngine -> balanceManager "Credit points" {
            tags "Sync,Internal"
        }

        burnEngine -> balanceManager "Debit points" {
            tags "Sync,Internal"
        }

        balanceManager -> expirationEngine "Schedule expiry" {
            tags "Async,Internal"
        }

        tierCalculator -> benefitsManager "Get benefits" {
            tags "Sync,Internal"
        }

        recommendationModel -> segmentationModel "Get segment" {
            tags "Sync,Internal"
        }

        propensityModel -> churnModel "Churn risk" {
            tags "Sync,Internal"
        }

        offerEngine -> targetingEngine "Target audience" {
            tags "Sync,Internal"
        }

        targetingEngine -> redemptionTracker "Track usage" {
            tags "Sync,Internal"
        }

        campaignOrchestrator -> abTestEngine "Run experiment" {
            tags "Sync,Internal"
        }

        abTestEngine -> contentManager "Get content" {
            tags "Sync,Internal"
        }

        challengeEngine -> badgeEngine "Award badge" {
            tags "Sync,Internal"
        }

        badgeEngine -> leaderboardService "Update ranking" {
            tags "Sync,Internal"
        }

        notificationEngine -> emailEngine "Email fallback" {
            tags "Async,Internal"
        }

        emailEngine -> smsEngine "SMS fallback" {
            tags "Async,Internal"
        }

        # Deployment Infrastructure
        cloudInfra = deploymentNode "Cloud Infrastructure" "Production environment" {
            apiZone = deploymentNode "API Zone" "API gateway layer" {
                containerInstance loyaltyAPI
            }
            coreServicesZone = deploymentNode "Core Services Zone" "Main services" {
                containerInstance profileService
                containerInstance pointsEngine
                containerInstance tierService
                containerInstance offerService
                containerInstance campaignService
                containerInstance gamificationService
            }
            mlZone = deploymentNode "ML Zone" "Machine learning services" {
                containerInstance personalizationEngine
                containerInstance featureStore
            }
            communicationZone = deploymentNode "Communication Zone" "Messaging services" {
                containerInstance communicationHub
            }
            dataZone = deploymentNode "Data Zone" "Data storage" {
                containerInstance eventBus
                containerInstance profileDatabase
                containerInstance pointsLedger
            }
        }
    }

    views {
        systemLandscape "loyalty-landscape" "Loyalty ecosystem" {
            include *
            autoLayout lr 300 100
        }

        systemContext loyaltyPlatform "loyalty-context" "Loyalty Platform context" {
            include *
            autoLayout lr 250 100
        }

        container loyaltyPlatform "loyalty-containers" "Loyalty containers" {
            include *
            autoLayout tb 200 100
        }

        component profileService "profile-components" "Profile Service components" {
            include *
            autoLayout lr 200 100
        }

        component pointsEngine "points-components" "Points Engine components" {
            include *
            autoLayout lr 200 100
        }

        component personalizationEngine "personalization-components" "Personalization components" {
            include *
            autoLayout tb 150 100
        }

        component offerService "offer-components" "Offer Service components" {
            include *
            autoLayout lr 200 100
        }

        component gamificationService "gamification-components" "Gamification components" {
            include *
            autoLayout lr 200 100
        }

        # Dynamic views
        dynamic loyaltyPlatform "earn-points-flow" "Earn Points from Purchase" {
            posSystem -> eventBus "Transaction completed"
            eventBus -> pointsEngine "Consume event"
            pointsEngine -> earnEngine "Calculate points"

            # Parallel lookups
            {
                earnEngine -> profileService "Get member tier"
                earnEngine -> offerService "Get active promos"
            }

            earnEngine -> balanceManager "Credit points"
            balanceManager -> pointsLedger "Store transaction"

            # Parallel post-processing
            {
                pointsEngine -> tierService "Check tier upgrade"
                pointsEngine -> gamificationService "Check challenges"
                pointsEngine -> eventBus "Publish points.earned"
            }

            tierCalculator -> benefitsManager "Apply new benefits"
            gamificationService -> badgeEngine "Award badges"
            eventBus -> communicationHub "Notify member"
            communicationHub -> notificationEngine "Push notification"

            autoLayout lr 250 100
        }

        dynamic loyaltyPlatform "redeem-points-flow" "Redeem Points" {
            mobileApp -> loyaltyAPI "Redeem request"
            loyaltyAPI -> pointsEngine "Process redemption"
            pointsEngine -> balanceManager "Check balance"

            # Parallel validation
            {
                balanceManager -> pointsLedger "Verify balance"
                pointsEngine -> offerService "Validate offer"
            }

            burnEngine -> balanceManager "Debit points"
            balanceManager -> pointsLedger "Store redemption"

            # Parallel completion
            {
                pointsEngine -> eventBus "Publish points.redeemed"
                offerService -> redemptionTracker "Track redemption"
            }

            loyaltyAPI -> mobileApp "Redemption confirmed"

            autoLayout tb 200 100
        }

        dynamic personalizationEngine "personalization-flow" "Real-time Personalization" {
            mobileApp -> loyaltyAPI "Get recommendations"
            loyaltyAPI -> profileService "Get profile"
            profileService -> customer360 "Build 360 view"

            # Parallel data gathering
            {
                customer360 -> pointsLedger "Transaction history"
                customer360 -> profileDatabase "Preferences"
                customer360 -> featureStore "Behavioral features"
            }

            profileService -> personalizationEngine "Request recommendations"

            # Parallel model inference
            {
                recommendationModel -> featureStore "Get features"
                segmentationModel -> featureStore "Get segment features"
                propensityModel -> featureStore "Get propensity features"
            }

            personalizationEngine -> offerService "Get targeted offers"
            offerEngine -> targetingEngine "Match offers"
            loyaltyAPI -> mobileApp "Personalized response"

            autoLayout lr 250 100
        }

        dynamic campaignService "campaign-execution-flow" "Campaign Execution" {
            marketingManager -> campaignService "Launch campaign"
            campaignOrchestrator -> abTestEngine "Create A/B test"
            abTestEngine -> contentManager "Get variants"

            campaignOrchestrator -> offerService "Create offers"
            offerService -> targetingEngine "Build audience"

            # Parallel audience selection
            {
                targetingEngine -> personalizationEngine "Get segments"
                targetingEngine -> profileDatabase "Filter by criteria"
            }

            campaignOrchestrator -> communicationHub "Send campaign"

            # Parallel channel delivery
            {
                notificationEngine -> mobileApp "Push notifications"
                emailEngine -> emailService "Send emails"
                smsEngine -> smsService "Send SMS"
            }

            campaignService -> eventBus "Publish campaign.sent"
            eventBus -> analyticsSystem "Track campaign"

            autoLayout tb 200 100
        }

        deployment loyaltyPlatform "Production" "loyalty-deployment" "Production deployment" {
            include *
            autoLayout lr 300 100
        }

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "Customer" {
                shape Person
                background "#2ECC71"
                color "#FFFFFF"
            }

            relationship "Async" {
                style Dashed
            }
        }
    }
}