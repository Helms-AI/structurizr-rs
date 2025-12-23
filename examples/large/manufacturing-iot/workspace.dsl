workspace "Smart Manufacturing IoT Platform" "Industry 4.0 Smart Factory with comprehensive IoT integration, predictive maintenance, and real-time production monitoring" {

    !docs "docs"
    !adrs "adrs"

    model {
        # People
        plantManager = person "Plant Manager" "Oversees overall factory operations, monitors KPIs, and makes strategic decisions"
        operator = person "Factory Operator" "Operates production equipment and monitors manufacturing processes"
        maintenanceTech = person "Maintenance Technician" "Performs preventive and corrective maintenance on factory equipment"
        qualityEngineer = person "Quality Engineer" "Monitors product quality metrics and manages quality control processes"
        supplyChainManager = person "Supply Chain Manager" "Manages inventory, procurement, and logistics operations"
        executive = person "Executive" "Reviews business intelligence dashboards and overall factory performance"

        # Main Manufacturing Execution System (MES)
        mes = softwareSystem "Manufacturing Execution System" "Core platform for production planning, scheduling, and execution tracking" {
            operationsDashboard = container "Operations Dashboard" "Real-time production monitoring and control interface" "React + TypeScript"
            mobileApp = container "Mobile App" "Mobile interface for operators and maintenance staff" "React Native"
            apiGateway = container "API Gateway" "Centralized API gateway for all MES services" "Kong Gateway"
            productionService = container "Production Service" "Manages production orders, work orders, and execution" "Spring Boot + Java"
            schedulingService = container "Scheduling Service" "Advanced production scheduling and optimization" "Spring Boot + Java"
            reportingService = container "Reporting Service" "Generates production reports and analytics" "Spring Boot + Java"
            primaryDB = container "Primary Database" "Stores production data, schedules, and configurations" "PostgreSQL"
            cache = container "Cache Layer" "High-performance caching for frequently accessed data" "Redis"
            messageBroker = container "Message Broker" "Asynchronous messaging and event streaming" "Apache Kafka"
        }

        # IoT Platform
        iotPlatform = softwareSystem "IoT Platform" "Connects and manages factory IoT devices, sensors, and edge computing" {
            deviceGateway = container "Device Gateway" "Secure gateway for IoT device connectivity" "Eclipse Mosquitto MQTT"
            dataIngestion = container "Data Ingestion Service" "Ingests and validates sensor data streams" "Go"
            streamProcessing = container "Stream Processing Engine" "Real-time processing of IoT data streams" "Apache Flink"
            timeSeriesDB = container "Time Series Database" "Optimized storage for sensor time-series data" "InfluxDB"
            alertService = container "Alert Service" "Monitors sensor data and generates alerts" "Python + FastAPI"
        }

        # SCADA System
        scadaSystem = softwareSystem "SCADA System" "Supervisory Control and Data Acquisition for industrial automation" {
            scadaServer = container "SCADA Server" "Central SCADA control server" "Ignition SCADA"
            plcInterface = container "PLC Interface" "Interfaces with Programmable Logic Controllers" "OPC UA"
            hmiClients = container "HMI Clients" "Human-Machine Interface terminals" "Touch Panel HMI"
        }

        # External ERP Integration
        erpSystem = softwareSystem "ERP System" "Enterprise Resource Planning system for business operations" {
            tags "External"
        }

        # Quality Management System
        qualityManagement = softwareSystem "Quality Management System" "Manages quality control, inspections, and compliance" {
            inspectionApp = container "Inspection Application" "Quality inspection data capture" "Angular"
            qualityService = container "Quality Service" "Quality metrics and analysis" "Node.js"
            qualityDB = container "Quality Database" "Stores inspection results and quality data" "MongoDB"
        }

        # Predictive Maintenance System
        predictiveMaintenance = softwareSystem "Predictive Maintenance System" "AI-powered predictive maintenance and asset health monitoring" {
            maintenanceDashboard = container "Maintenance Dashboard" "Maintenance planning and asset health visualization" "Vue.js"
            mlEngine = container "ML Engine" "Machine learning models for failure prediction" "Python + TensorFlow" {
                dataCollector = component "Data Collector" "Collects sensor data for ML training" "Python"
                featureExtractor = component "Feature Extractor" "Extracts features from raw sensor data" "Python + Pandas"
                modelTrainer = component "Model Trainer" "Trains predictive models" "TensorFlow + Keras"
                inferenceEngine = component "Inference Engine" "Runs predictions on live data" "TensorFlow Serving"
            }
            assetService = container "Asset Management Service" "Manages equipment assets and maintenance history" "Spring Boot + Java"
            maintenanceDB = container "Maintenance Database" "Stores asset data and maintenance records" "PostgreSQL"
        }

        # Energy Management System
        energyManagement = softwareSystem "Energy Management System" "Monitors and optimizes factory energy consumption" {
            energyDashboard = container "Energy Dashboard" "Real-time energy consumption monitoring" "React"
            energyService = container "Energy Optimization Service" "Analyzes and optimizes energy usage" "Python"
            meteringDB = container "Metering Database" "Stores energy consumption data" "InfluxDB"
        }

        # Supply Chain Integration
        supplyChainSystem = softwareSystem "Supply Chain System" "Manages inventory, procurement, and logistics" {
            inventoryApp = container "Inventory Application" "Inventory tracking and management" "React"
            procurementService = container "Procurement Service" "Manages supplier orders and deliveries" "Spring Boot + Java"
            warehouseService = container "Warehouse Service" "Warehouse management and logistics" "Spring Boot + Java"
            supplyChainDB = container "Supply Chain Database" "Stores inventory and logistics data" "PostgreSQL"
        }

        # Reporting and Analytics
        analyticsSystem = softwareSystem "Reporting & Analytics Platform" "Business intelligence and advanced analytics" {
            biDashboard = container "BI Dashboard" "Executive dashboards and reports" "Power BI"
            analyticsEngine = container "Analytics Engine" "Advanced analytics and data processing" "Apache Spark"
            dataWarehouse = container "Data Warehouse" "Centralized data warehouse" "Snowflake"
        }

        # Digital Twin
        digitalTwin = softwareSystem "Digital Twin Platform" "Virtual representation of the physical factory for simulation" {
            twinVisualization = container "3D Visualization" "3D model of factory and equipment" "Unity3D"
            simulationEngine = container "Simulation Engine" "Physics-based simulation engine" "MATLAB Simulink"
            twinDataService = container "Twin Data Service" "Synchronizes real-time data with digital twin" "Node.js"
        }

        # Relationships - People to Systems
        plantManager -> mes "Monitors production schedules and KPIs"
        plantManager -> analyticsSystem "Reviews business intelligence reports"
        plantManager -> energyManagement "Monitors energy consumption"

        operator -> mes "Uses to execute production orders"
        operator -> scadaSystem "Controls manufacturing equipment"
        operator -> iotPlatform "Monitors sensor data"

        maintenanceTech -> predictiveMaintenance "Plans and tracks maintenance activities"
        maintenanceTech -> mes "Records maintenance completion"
        maintenanceTech -> scadaSystem "Monitors equipment status"

        qualityEngineer -> qualityManagement "Performs quality inspections"
        qualityEngineer -> mes "Reviews quality metrics"
        qualityEngineer -> analyticsSystem "Analyzes quality trends"

        supplyChainManager -> supplyChainSystem "Manages inventory and procurement"
        supplyChainManager -> mes "Monitors material requirements"
        supplyChainManager -> erpSystem "Coordinates with enterprise systems"

        executive -> analyticsSystem "Views executive dashboards"
        executive -> digitalTwin "Reviews factory simulation"

        # MES Container Relationships
        operationsDashboard -> apiGateway "Makes API calls" "HTTPS/REST"
        mobileApp -> apiGateway "Makes API calls" "HTTPS/REST"

        apiGateway -> productionService "Routes requests" "HTTP"
        apiGateway -> schedulingService "Routes requests" "HTTP"
        apiGateway -> reportingService "Routes requests" "HTTP"

        productionService -> primaryDB "Reads from and writes to" "JDBC"
        productionService -> cache "Caches data in" "Redis Protocol"
        productionService -> messageBroker "Publishes events to" "Kafka Protocol"

        schedulingService -> primaryDB "Reads from and writes to" "JDBC"
        schedulingService -> cache "Caches data in" "Redis Protocol"
        schedulingService -> messageBroker "Publishes events to" "Kafka Protocol"

        reportingService -> primaryDB "Reads from" "JDBC"
        reportingService -> messageBroker "Subscribes to events" "Kafka Protocol"

        # IoT Platform Container Relationships
        deviceGateway -> dataIngestion "Forwards sensor data" "MQTT"
        dataIngestion -> streamProcessing "Streams data to" "gRPC"
        streamProcessing -> timeSeriesDB "Writes processed data" "InfluxDB Line Protocol"
        streamProcessing -> alertService "Sends anomalies to" "HTTP"
        alertService -> messageBroker "Publishes alerts to" "Kafka Protocol"

        # SCADA Container Relationships
        scadaServer -> plcInterface "Communicates with PLCs" "OPC UA"
        hmiClients -> scadaServer "Displays data from" "Proprietary Protocol"

        # Quality Management Container Relationships
        inspectionApp -> qualityService "Submits inspection data" "HTTPS/REST"
        qualityService -> qualityDB "Stores data in" "MongoDB Protocol"

        # Predictive Maintenance Container Relationships
        maintenanceDashboard -> assetService "Retrieves asset data" "HTTPS/REST"
        maintenanceDashboard -> mlEngine "Gets predictions from" "HTTPS/REST"

        dataCollector -> featureExtractor "Sends raw data to" "Internal API"
        featureExtractor -> modelTrainer "Provides features to" "Internal API"
        modelTrainer -> inferenceEngine "Deploys models to" "TensorFlow Serving API"

        mlEngine -> maintenanceDB "Stores predictions in" "JDBC"
        assetService -> maintenanceDB "Reads from and writes to" "JDBC"

        # Energy Management Container Relationships
        energyDashboard -> energyService "Retrieves energy data" "HTTPS/REST"
        energyService -> meteringDB "Queries energy metrics" "InfluxDB HTTP API"

        # Supply Chain Container Relationships
        inventoryApp -> procurementService "Manages orders" "HTTPS/REST"
        inventoryApp -> warehouseService "Tracks inventory" "HTTPS/REST"
        procurementService -> supplyChainDB "Reads from and writes to" "JDBC"
        warehouseService -> supplyChainDB "Reads from and writes to" "JDBC"

        # Analytics Container Relationships
        biDashboard -> analyticsEngine "Queries analytics" "SQL"
        analyticsEngine -> dataWarehouse "Processes data from" "JDBC"

        # Digital Twin Container Relationships
        twinVisualization -> twinDataService "Retrieves real-time data" "WebSocket"
        twinDataService -> simulationEngine "Feeds data to" "REST API"

        # System-to-System Relationships
        mes -> erpSystem "Synchronizes production data" "HTTPS/REST"
        mes -> iotPlatform "Receives sensor data" "Kafka"
        mes -> scadaSystem "Retrieves equipment status" "OPC UA"
        mes -> qualityManagement "Sends quality checkpoints" "HTTPS/REST"
        mes -> predictiveMaintenance "Sends equipment usage data" "Kafka"
        mes -> supplyChainSystem "Requests material availability" "HTTPS/REST"

        iotPlatform -> predictiveMaintenance "Streams sensor data" "Kafka"
        iotPlatform -> energyManagement "Sends power consumption data" "Kafka"
        iotPlatform -> digitalTwin "Provides real-time sensor data" "HTTPS/REST"
        iotPlatform -> scadaSystem "Integrates with" "OPC UA"

        scadaSystem -> mes "Reports production counts" "OPC UA"

        qualityManagement -> mes "Reports quality results" "HTTPS/REST"
        qualityManagement -> analyticsSystem "Sends quality metrics" "HTTPS/REST"

        predictiveMaintenance -> mes "Creates maintenance work orders" "HTTPS/REST"
        predictiveMaintenance -> supplyChainSystem "Orders spare parts" "HTTPS/REST"

        supplyChainSystem -> erpSystem "Synchronizes inventory" "HTTPS/REST"
        supplyChainSystem -> mes "Confirms material availability" "HTTPS/REST"

        analyticsSystem -> mes "Analyzes production data" "JDBC"
        analyticsSystem -> qualityManagement "Analyzes quality data" "JDBC"
        analyticsSystem -> energyManagement "Analyzes energy data" "JDBC"
        analyticsSystem -> iotPlatform "Analyzes sensor data" "JDBC"

        digitalTwin -> iotPlatform "Mirrors sensor data" "HTTPS/REST"
        digitalTwin -> mes "Simulates production scenarios" "HTTPS/REST"
    }

    views {
        systemLandscape "SystemLandscape" "Overview of the Smart Manufacturing IoT Platform ecosystem" {
            include *
            autoLayout lr
        }

        systemContext mes "MESContext" "System context diagram for the Manufacturing Execution System" {
            include *
            autoLayout lr
        }

        systemContext iotPlatform "IoTContext" "System context diagram for the IoT Platform" {
            include *
            autoLayout lr
        }

        container mes "MESContainers" "Container diagram for the Manufacturing Execution System" {
            include *
            autoLayout lr
        }

        container iotPlatform "IoTPlatformContainers" "Container diagram for the IoT Platform" {
            include *
            autoLayout lr
        }

        container predictiveMaintenance "PredictiveMaintenanceContainers" "Container diagram for Predictive Maintenance System" {
            include *
            autoLayout lr
        }

        component mlEngine "MLEngineComponents" "Component diagram for the ML Engine" {
            include *
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
