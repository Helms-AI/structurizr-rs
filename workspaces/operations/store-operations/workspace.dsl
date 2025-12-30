workspace "FreshMart Store Operations" "Store management, IoT integration, and workforce optimization" {

    !const CORP_COLOR "#00539C"
    !const IOT_COLOR "#E74C3C"
    !const WORKFLOW_COLOR "#16A085"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # People
        storeManager = person "Store Manager" "Manages daily store operations" {
            tags "Person"
        }

        associate = person "Store Associate" "Front-line store employee" {
            tags "Person,Associate"
        }

        districtManager = person "District Manager" "Oversees multiple stores" {
            tags "Person"
        }

        # External Systems
        posSystem = softwareSystem "POS System" "Transaction processing" {
            tags "Software System,External"
        }

        inventorySystem = softwareSystem "Inventory Platform" "Stock management" {
            tags "Software System,External"
        }

        hrSystem = softwareSystem "HR System" "Employee management" {
            tags "Software System,External"
        }

        analyticsSystem = softwareSystem "Analytics Platform" "Business intelligence" {
            tags "Software System,External"
        }

        # Store Operations Platform
        storeOps = softwareSystem "Store Operations Platform" "Unified store management" {
            tags "Software System,Core"

            properties {
                "stores" "2,500"
                "associates" "75,000+"
                "iot_devices" "100,000+"
                "perspective_business" "15% labor cost reduction"
                "perspective_technical" "IoT-enabled smart stores"
                "perspective_operations" "Real-time store visibility"
            }

            # Task Management
            taskManager = container "Task Manager" "Task creation and assignment" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "tasks_per_day" "500,000+"
                }

                taskEngine = component "Task Engine" "Task workflow processing" "Camunda" {
                    tags "Component"
                }

                priorityEngine = component "Priority Engine" "Dynamic task prioritization" "ML Model" {
                    tags "Component,ML"
                }

                assignmentEngine = component "Assignment Engine" "Optimal task assignment" "OR-Tools" {
                    tags "Component"
                }
            }

            # Labor Management
            laborManager = container "Labor Manager" "Workforce scheduling and optimization" "Java/Spring Boot" {
                tags "Container"
                properties {
                    "algorithm" "Mixed integer programming"
                }

                scheduleOptimizer = component "Schedule Optimizer" "Optimal scheduling" "OR-Tools" {
                    tags "Component,ML"
                }

                forecastEngine = component "Traffic Forecast" "Customer traffic prediction" "Prophet" {
                    tags "Component,ML"
                }

                complianceChecker = component "Compliance Checker" "Labor law compliance" "Rule Engine" {
                    tags "Component"
                }
            }

            # IoT Hub
            iotHub = container "IoT Hub" "Device management and data collection" "Azure IoT Hub" {
                tags "Container,IoT"
                properties {
                    "devices" "100,000+"
                    "protocols" "MQTT, AMQP, HTTP"
                }

                deviceRegistry = component "Device Registry" "Device management" "IoT Hub" {
                    tags "Component"
                }

                telemetryProcessor = component "Telemetry Processor" "Sensor data processing" "Stream Analytics" {
                    tags "Component"
                }

                alertEngine = component "Alert Engine" "Real-time alerting" "Event Grid" {
                    tags "Component"
                }
            }

            # Digital Signage
            signageManager = container "Digital Signage Manager" "Store display management" "Node.js" {
                tags "Container"
                properties {
                    "displays" "25,000+"
                }

                contentScheduler = component "Content Scheduler" "Display scheduling" "Node Service" {
                    tags "Component"
                }

                priceTagManager = component "ESL Manager" "Electronic shelf labels" "Node Service" {
                    tags "Component"
                    properties {
                        "labels" "2M+"
                    }
                }
            }

            # Store Dashboard
            storeDashboard = container "Store Dashboard" "Real-time store monitoring" "React" {
                tags "Container,Web"
            }

            # Mobile App
            associateApp = container "Associate Mobile App" "Employee mobile application" "React Native" {
                tags "Container,Mobile"
                properties {
                    "features" "Tasks, Schedule, Communication"
                }
            }

            # Event Bus
            eventBus = container "Event Bus" "Store event streaming" "Apache Kafka" {
                tags "Container,Queue"
            }

            # Database
            opsDatabase = container "Operations Database" "Operational data store" "PostgreSQL" {
                tags "Container,Database"
            }
        }

        # IoT Devices
        tempSensor = softwareSystem "Temperature Sensors" "Refrigeration monitoring" {
            tags "External,IoT"
        }

        footfallSensor = softwareSystem "Footfall Counters" "Customer traffic" {
            tags "External,IoT"
        }

        eslSystem = softwareSystem "ESL Infrastructure" "Electronic shelf labels" {
            tags "External,IoT"
        }

        hvacSystem = softwareSystem "HVAC System" "Climate control" {
            tags "External,IoT"
        }

        # Relationships
        storeManager -> storeDashboard "Monitors store" {
            tags "Sync"
        }

        associate -> associateApp "Uses mobile app" {
            tags "Sync"
        }

        districtManager -> storeDashboard "Reviews stores" {
            tags "Sync"
        }

        # IoT relationships
        tempSensor -> iotHub "Temperature readings" {
            tags "Async,IoT"
        }

        footfallSensor -> iotHub "Traffic data" {
            tags "Async,IoT"
        }

        eslSystem -> signageManager "Label updates" {
            tags "Sync,IoT"
        }

        hvacSystem -> iotHub "Climate data" {
            tags "Async,IoT"
        }

        # System integrations
        storeOps -> posSystem "Sales data" {
            tags "Async"
        }

        storeOps -> inventorySystem "Stock levels" {
            tags "Async"
        }

        laborManager -> hrSystem "Employee data" {
            tags "Sync"
        }

        storeOps -> analyticsSystem "Metrics" {
            tags "Async"
        }

        # Internal relationships
        storeDashboard -> taskManager "View tasks" {
            tags "Sync,Internal"
        }

        associateApp -> taskManager "Complete tasks" {
            tags "Sync,Internal"
        }

        taskManager -> eventBus "Task events" {
            tags "Async,Internal"
        }

        laborManager -> opsDatabase "Store schedules" {
            tags "Sync,Internal"
        }

        iotHub -> eventBus "Sensor events" {
            tags "Async,Internal"
        }

        signageManager -> opsDatabase "Content data" {
            tags "Sync,Internal"
        }

        eventBus -> analyticsSystem "Forward events" {
            tags "Async,Internal"
        }

        # Component relationships
        taskEngine -> priorityEngine "Get priority" {
            tags "Sync,Internal"
        }

        priorityEngine -> assignmentEngine "Assign task" {
            tags "Sync,Internal"
        }

        scheduleOptimizer -> forecastEngine "Get forecast" {
            tags "Sync,Internal"
        }

        forecastEngine -> complianceChecker "Check compliance" {
            tags "Sync,Internal"
        }

        deviceRegistry -> telemetryProcessor "Route data" {
            tags "Async,Internal"
        }

        telemetryProcessor -> alertEngine "Trigger alerts" {
            tags "Async,Internal"
        }

        contentScheduler -> priceTagManager "Update prices" {
            tags "Sync,Internal"
        }

        # Deployment nodes
        deploymentNode "Cloud Infrastructure" {
            deploymentNode "Application Zone" {
                containerInstance taskManager
                containerInstance laborManager
                containerInstance signageManager
                containerInstance storeDashboard
            }
            deploymentNode "IoT Zone" {
                containerInstance iotHub
            }
            deploymentNode "Data Zone" {
                containerInstance eventBus
                containerInstance opsDatabase
            }
            deploymentNode "Mobile Zone" {
                containerInstance associateApp
            }
        }
    }

    views {
        systemLandscape "store-ops-landscape" "Store operations ecosystem" {
            include *
            autoLayout lr 300 100
        }

        systemContext storeOps "store-ops-context" "Store Operations context" {
            include *
            autoLayout lr 250 100
        }

        container storeOps "store-ops-containers" "Store Operations containers" {
            include *
            autoLayout tb 200 100
        }

        component taskManager "task-components" "Task Manager components" {
            include *
            autoLayout lr 200 100
        }

        component laborManager "labor-components" "Labor Manager components" {
            include *
            autoLayout lr 200 100
        }

        component iotHub "iot-components" "IoT Hub components" {
            include *
            autoLayout lr 200 100
        }

        # Dynamic views
        dynamic storeOps "task-assignment-flow" "Task Assignment Flow" {
            storeManager -> storeDashboard "Create task"
            storeDashboard -> taskManager "Submit task"
            taskManager -> taskEngine "Process task"

            # Parallel processing
            {
                taskEngine -> priorityEngine "Calculate priority"
                taskEngine -> opsDatabase "Check associate availability"
            }

            priorityEngine -> assignmentEngine "Optimal assignment"
            assignmentEngine -> eventBus "Publish task.assigned"
            eventBus -> associateApp "Push notification"
            associate -> associateApp "View and accept task"
            associateApp -> taskManager "Complete task"
            taskManager -> eventBus "Publish task.completed"

            autoLayout lr 250 100
        }

        dynamic iotHub "temperature-alert-flow" "Temperature Alert Flow" {
            tempSensor -> iotHub "Temperature reading"
            iotHub -> deviceRegistry "Validate device"
            deviceRegistry -> telemetryProcessor "Process reading"

            # Parallel processing
            {
                telemetryProcessor -> eventBus "Store telemetry"
                telemetryProcessor -> alertEngine "Check threshold"
            }

            alertEngine -> eventBus "Publish alert"

            # Parallel notifications
            {
                eventBus -> storeDashboard "Dashboard alert"
                eventBus -> associateApp "Mobile notification"
                eventBus -> taskManager "Create repair task"
            }

            autoLayout tb 200 100
        }

        dynamic laborManager "schedule-optimization-flow" "Schedule Optimization" {
            districtManager -> storeDashboard "Request schedule"
            storeDashboard -> laborManager "Generate schedule"
            laborManager -> forecastEngine "Forecast traffic"

            # Parallel data gathering
            {
                forecastEngine -> posSystem "Historical sales"
                forecastEngine -> footfallSensor "Traffic patterns"
                forecastEngine -> analyticsSystem "Events calendar"
            }

            forecastEngine -> scheduleOptimizer "Traffic forecast"
            scheduleOptimizer -> hrSystem "Get employee constraints"
            scheduleOptimizer -> complianceChecker "Check labor laws"
            complianceChecker -> scheduleOptimizer "Compliance verified"
            scheduleOptimizer -> opsDatabase "Save schedule"
            laborManager -> eventBus "Publish schedule"

            # Parallel notifications
            {
                eventBus -> associateApp "Notify associates"
                eventBus -> hrSystem "Sync schedules"
            }

            autoLayout lr 250 100
        }

        deployment storeOps "Production" "store-ops-deployment" "Production deployment" {
            include *
            autoLayout lr 300 100
        }

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "IoT" {
                shape Ellipse
                background "#E74C3C"
                color "#FFFFFF"
            }

            relationship "IoT" {
                color "#E74C3C"
                style Dashed
            }
        }
    }
}