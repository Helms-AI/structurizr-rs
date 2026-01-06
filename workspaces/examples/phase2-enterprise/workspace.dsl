/*
 * Phase 2 Example: Enterprise & Configuration
 *
 * This workspace demonstrates the enterprise boundary and configuration
 * features implemented in Phase 2:
 * - enterprise block - Define organizational boundaries
 * - configuration block - Workspace-level settings
 * - terminology - Custom element type labels
 * - Internal vs External element classification
 */

workspace "Phase 2 - Enterprise Demo" "Demonstrates enterprise boundaries and configuration" {

    !docs "docs"
    !adrs "adrs"

    model {
        # Enterprise boundary - elements inside are "Internal"
        enterprise "TechCorp Financial Services" {
            # Internal actors
            trader = person "Trader" "Executes trades on behalf of clients"
            analyst = person "Risk Analyst" "Monitors and assesses trading risks"

            # Core trading platform (internal)
            tradingPlatform = softwareSystem "Trading Platform" "Core trading system" {
                webPortal = container "Web Portal" "Trading interface" "React"
                orderEngine = container "Order Engine" "Processes orders" "Java"
                riskEngine = container "Risk Engine" "Risk calculations" "C++"
                database = container "Trading Database" "Order storage" "Oracle" {
                    tags "Database"
                }
            }

            # Risk management system (internal)
            riskSystem = softwareSystem "Risk Management System" "Risk monitoring"
        }

        # External entities (outside enterprise boundary)
        client = person "Client" "Institutional client" {
            tags "External"
        }

        # External systems
        exchange = softwareSystem "Stock Exchange" "Securities exchange" {
            tags "External"
        }
        marketData = softwareSystem "Market Data Vendor" "Bloomberg/Reuters" {
            tags "External"
        }

        # Relationships - Internal actors
        trader -> tradingPlatform "Executes trades using"
        analyst -> riskSystem "Monitors risk using"

        # Relationships - External actors
        client -> tradingPlatform "Places orders through"

        # System-to-system relationships
        tradingPlatform -> exchange "Sends orders to"
        marketData -> tradingPlatform "Provides data to"
        tradingPlatform -> riskSystem "Sends trade data to"
    }

    views {
        # System Landscape - shows enterprise boundary
        systemLandscape "Landscape" "TechCorp Financial Services landscape" {
            include *
            autoLayout
        }

        # System Context for Trading Platform
        systemContext tradingPlatform "TradingContext" "Trading Platform in context" {
            include *
            autoLayout
        }

        # Container view for Trading Platform
        container tradingPlatform "TradingContainers" "Trading Platform internals" {
            include *
            autoLayout
        }

        # Custom terminology (healthcare-specific labels)
        terminology {
            person "Actor"
            softwareSystem "Application"
            container "Module"
            enterprise "Organization"
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
            element "External" {
                background "#999999"
                color "#ffffff"
            }
        }
    }
}
