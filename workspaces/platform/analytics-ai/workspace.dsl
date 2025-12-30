workspace "FreshMart Analytics & AI Platform" "Real-time analytics and machine learning platform for retail intelligence" {

    !const CORP_COLOR "#00539C"
    !const ML_COLOR "#9370DB"
    !const DATA_COLOR "#008080"
    !const REALTIME_COLOR "#2ECC71"
    !const BATCH_COLOR "#3498DB"

    !impliedRelationships true
    !docs "docs"
    !adrs "adrs"

    model {
        # ========================================
        # People
        # ========================================

        dataScientist = person "Data Scientist" "Develops and trains ML models" {
            tags "Person"
            properties {
                "tools" "Python, Jupyter, MLflow"
                "expertise" "Deep Learning, Statistics"
            }
        }

        dataAnalyst = person "Data Analyst" "Creates reports and dashboards" {
            tags "Person"
            properties {
                "tools" "SQL, Tableau, Python"
                "focus" "Business Intelligence"
            }
        }

        businessUser = person "Business User" "Consumes analytics and insights" {
            tags "Person"
            properties {
                "departments" "Marketing, Operations, Finance"
            }
        }

        mlEngineer = person "ML Engineer" "Deploys and maintains ML models" {
            tags "Person"
            properties {
                "skills" "MLOps, Kubernetes, Python"
            }
        }

        # ========================================
        # External Data Sources
        # ========================================

        group "Data Sources" {
            posSystem = softwareSystem "POS System" "Transaction data source" {
                tags "External,Software System"
                properties {
                    "data_volume" "5M transactions/day"
                    "format" "JSON"
                }
            }

            inventorySystem = softwareSystem "Inventory System" "Stock and movement data" {
                tags "External,Software System"
                properties {
                    "data_volume" "10M events/day"
                    "format" "Avro"
                }
            }

            customerSystem = softwareSystem "Customer Platform" "Customer and loyalty data" {
                tags "External,Software System"
                properties {
                    "data_volume" "1M profiles"
                    "format" "JSON"
                }
            }

            supplyChainSystem = softwareSystem "Supply Chain" "Logistics and supplier data" {
                tags "External,Software System"
                properties {
                    "data_volume" "100K shipments/day"
                    "format" "XML/EDI"
                }
            }

            weatherAPI = softwareSystem "Weather Service" "External weather data" {
                tags "External,Partner"
                properties {
                    "api_calls" "10K/day"
                    "coverage" "2500 locations"
                }
            }

            socialMedia = softwareSystem "Social Media APIs" "Social sentiment data" {
                tags "External,Partner"
                properties {
                    "platforms" "Twitter, Facebook, Instagram"
                    "volume" "100K mentions/day"
                }
            }
        }

        # ========================================
        # Analytics & AI Platform (Main System)
        # ========================================

        analyticsAI = softwareSystem "Analytics & AI Platform" "Unified data and AI platform" {
            tags "Software System,Core,ML,Data"

            properties {
                "data_lake_size" "5PB"
                "ml_models" "200+"
                "daily_predictions" "100M+"
                "streaming_throughput" "1M events/sec"
                "batch_jobs" "500/day"
                "perspective_business" "Drives $500M+ annual value through optimization"
                "perspective_technical" "Lambda architecture with ML platform"
                "perspective_operations" "Auto-scaling, self-service, 99.9% SLA"
            }

            # ========================================
            # Data Ingestion Layer
            # ========================================

            dataIngestion = container "Data Ingestion Gateway" "Unified data ingestion service" "Apache NiFi" {
                tags "Container,Integration"
                properties {
                    "connectors" "50+"
                    "throughput" "1GB/sec"
                    "protocols" "REST, Kafka, JDBC, S3"
                }

                cdcConnector = component "CDC Connector" "Change data capture from databases" "Debezium" {
                    tags "Component"
                    properties {
                        "databases" "PostgreSQL, MySQL, Oracle"
                    }
                }

                streamCollector = component "Stream Collector" "Collects streaming data" "Kafka Connect" {
                    tags "Component"
                    properties {
                        "topics" "100+"
                    }
                }

                fileCollector = component "File Collector" "Batch file ingestion" "NiFi Processors" {
                    tags "Component"
                    properties {
                        "formats" "CSV, JSON, Parquet, Avro"
                    }
                }

                apiCollector = component "API Collector" "REST API data collection" "Custom Python" {
                    tags "Component"
                    properties {
                        "apis" "20+"
                    }
                }
            }

            # ========================================
            # Stream Processing Layer
            # ========================================

            streamProcessor = container "Stream Processing Engine" "Real-time data processing" "Apache Flink" {
                tags "Container,Realtime"
                properties {
                    "job_count" "50+"
                    "checkpointing" "Exactly-once"
                    "state_backend" "RocksDB"
                    "latency" "<100ms"
                }

                eventEnricher = component "Event Enricher" "Enriches events with context" "Flink Job" {
                    tags "Component"
                    properties {
                        "enrichment_sources" "10+"
                    }
                }

                windowAggregator = component "Window Aggregator" "Time-window aggregations" "Flink Job" {
                    tags "Component"
                    properties {
                        "window_types" "Tumbling, Sliding, Session"
                    }
                }

                anomalyDetector = component "Anomaly Detector" "Real-time anomaly detection" "Flink ML" {
                    tags "Component,ML"
                    properties {
                        "algorithms" "Isolation Forest, DBSCAN"
                    }
                }

                alertEngine = component "Alert Engine" "Real-time alerting" "Flink CEP" {
                    tags "Component"
                    properties {
                        "rules" "200+"
                        "channels" "Email, SMS, Slack"
                    }
                }
            }

            # ========================================
            # Batch Processing Layer
            # ========================================

            batchProcessor = container "Batch Processing Engine" "Large-scale batch processing" "Apache Spark" {
                tags "Container,Batch"
                properties {
                    "cluster_size" "100 nodes"
                    "memory" "10TB"
                    "jobs_per_day" "500"
                    "processing_capacity" "100TB/day"
                }

                etlPipeline = component "ETL Pipeline" "Extract, transform, load jobs" "Spark SQL" {
                    tags "Component"
                    properties {
                        "pipelines" "200+"
                        "schedule" "Airflow"
                    }
                }

                dataQuality = component "Data Quality Engine" "Data validation and cleansing" "Great Expectations" {
                    tags "Component"
                    properties {
                        "rules" "1000+"
                        "validation_rate" "100%"
                    }
                }

                featureEngineering = component "Feature Engineering" "Feature computation for ML" "Spark ML" {
                    tags "Component,ML"
                    properties {
                        "features" "5000+"
                        "refresh_rate" "Hourly"
                    }
                }
            }

            # ========================================
            # Storage Layer
            # ========================================

            dataLake = container "Data Lake" "Centralized data storage" "AWS S3" {
                tags "Container,Database,Data"
                properties {
                    "storage" "5PB"
                    "format" "Parquet, ORC, Avro"
                    "partitioning" "Year/Month/Day/Hour"
                    "compression" "Snappy"
                    "zones" "Raw, Curated, Refined"
                }
            }

            dataWarehouse = container "Data Warehouse" "Analytical data store" "Snowflake" {
                tags "Container,Database"
                properties {
                    "size" "500TB"
                    "tables" "1000+"
                    "views" "2000+"
                    "compute" "Auto-scaling"
                }
            }

            featureStore = container "Feature Store" "ML feature management" "Feast" {
                tags "Container,ML,Database"
                properties {
                    "features" "5000+"
                    "serving_latency" "<10ms"
                    "storage" "Redis + S3"
                    "versioning" "Git-based"
                }
            }

            vectorDB = container "Vector Database" "Embedding storage for AI" "Pinecone" {
                tags "Container,ML,Database"
                properties {
                    "vectors" "1B+"
                    "dimensions" "768"
                    "similarity" "Cosine, Euclidean"
                }
            }

            # ========================================
            # ML Platform Layer
            # ========================================

            mlPlatform = container "ML Platform" "End-to-end ML lifecycle management" "Kubeflow + MLflow" {
                tags "Container,ML"
                properties {
                    "models" "200+"
                    "experiments" "10000+"
                    "deployment_targets" "Batch, Real-time, Edge"
                }

                experimentTracker = component "Experiment Tracker" "Track ML experiments" "MLflow Tracking" {
                    tags "Component,ML"
                    properties {
                        "metrics" "Accuracy, F1, AUC, RMSE"
                    }
                }

                modelRegistry = component "Model Registry" "Model versioning and governance" "MLflow Registry" {
                    tags "Component,ML"
                    properties {
                        "stages" "Staging, Production, Archived"
                        "approval_workflow" "Automated + Manual"
                    }
                }

                modelTraining = component "Model Training" "Distributed model training" "Kubeflow Pipelines" {
                    tags "Component,ML"
                    properties {
                        "frameworks" "TensorFlow, PyTorch, XGBoost"
                        "hardware" "GPU, TPU"
                    }
                }

                modelServing = component "Model Serving" "Model deployment and inference" "KFServing" {
                    tags "Component,ML"
                    properties {
                        "protocols" "REST, gRPC"
                        "autoscaling" "HPA + VPA"
                    }
                }
            }

            # ========================================
            # ML Models (Specific)
            # ========================================

            demandForecasting = container "Demand Forecasting Model" "Predicts product demand" "Prophet + LSTM" {
                tags "Container,ML"
                properties {
                    "accuracy" "92%"
                    "horizon" "4 weeks"
                    "granularity" "SKU + Store"
                    "update_frequency" "Daily"
                }
            }

            pricingOptimization = container "Pricing Optimization Model" "Dynamic pricing recommendations" "Reinforcement Learning" {
                tags "Container,ML"
                properties {
                    "algorithm" "Deep Q-Network"
                    "optimization_target" "Revenue + Margin"
                    "constraints" "Competition, Inventory"
                }
            }

            customerSegmentation = container "Customer Segmentation Model" "Customer clustering and profiling" "K-Means + DBSCAN" {
                tags "Container,ML"
                properties {
                    "segments" "12"
                    "features" "100+"
                    "update_frequency" "Weekly"
                }
            }

            fraudDetection = container "Fraud Detection Model" "Identifies fraudulent transactions" "XGBoost + Isolation Forest" {
                tags "Container,ML,Security"
                properties {
                    "precision" "95%"
                    "recall" "92%"
                    "latency" "<50ms"
                }
            }

            recommendationEngine = container "Recommendation Engine" "Product recommendations" "Collaborative Filtering + Deep Learning" {
                tags "Container,ML"
                properties {
                    "algorithms" "ALS, Neural CF, BERT4Rec"
                    "personalization" "Real-time"
                    "coverage" "95% of catalog"
                }
            }

            # ========================================
            # Analytics Layer
            # ========================================

            analyticsEngine = container "Analytics Engine" "OLAP and ad-hoc analytics" "Apache Druid" {
                tags "Container,Data"
                properties {
                    "query_latency" "<1s"
                    "data_freshness" "1 minute"
                    "concurrent_queries" "1000"
                }
            }

            reportingService = container "Reporting Service" "Scheduled reports and dashboards" "Apache Superset" {
                tags "Container"
                properties {
                    "dashboards" "500+"
                    "reports" "1000+"
                    "users" "5000+"
                }
            }

            notebookEnvironment = container "Notebook Environment" "Data science notebooks" "JupyterHub" {
                tags "Container"
                properties {
                    "users" "200+"
                    "kernels" "Python, R, Scala"
                    "compute" "On-demand"
                }
            }

            # ========================================
            # Orchestration & Governance
            # ========================================

            orchestrator = container "Workflow Orchestrator" "Job scheduling and orchestration" "Apache Airflow" {
                tags "Container"
                properties {
                    "dags" "500+"
                    "tasks_per_day" "50000"
                    "sla_monitoring" "Enabled"
                }
            }

            dataGovernance = container "Data Governance Platform" "Metadata and governance" "Apache Atlas + Ranger" {
                tags "Container,Security"
                properties {
                    "data_lineage" "End-to-end"
                    "classifications" "PII, PCI, Sensitive"
                    "policies" "500+"
                }
            }

            # ========================================
            # Infrastructure
            # ========================================

            messageQueue = container "Message Queue" "Event streaming backbone" "Apache Kafka" {
                tags "Container,Queue"
                properties {
                    "brokers" "10"
                    "topics" "200+"
                    "throughput" "1M msg/sec"
                }
            }

            cache = container "Distributed Cache" "High-speed data cache" "Redis Cluster" {
                tags "Container,Cache"
                properties {
                    "memory" "1TB"
                    "operations" "1M ops/sec"
                    "persistence" "AOF"
                }
            }

            monitoring = container "Monitoring Stack" "Platform monitoring" "Prometheus + Grafana" {
                tags "Container"
                properties {
                    "metrics" "10000+"
                    "dashboards" "100+"
                    "alerts" "500+"
                }
            }
        }

        # ========================================
        # External Consumption Systems
        # ========================================

        operationalSystems = softwareSystem "Operational Systems" "Consume analytics for operations" {
            tags "External,Software System"
            properties {
                "systems" "ERP, CRM, SCM"
            }
        }

        mobileApp = softwareSystem "FreshMart Mobile App" "Customer mobile application" {
            tags "External,Software System"
            properties {
                "users" "10M+"
            }
        }

        # ========================================
        # Relationships - External to Platform
        # ========================================

        dataScientist -> notebookEnvironment "Develops models" {
            tags "Sync"
        }

        dataScientist -> mlPlatform "Trains and deploys models" {
            tags "Sync"
        }

        dataAnalyst -> reportingService "Creates dashboards" {
            tags "Sync"
        }

        dataAnalyst -> analyticsEngine "Runs queries" {
            tags "Sync"
        }

        businessUser -> reportingService "Views reports" {
            tags "Sync"
        }

        mlEngineer -> mlPlatform "Manages ML operations" {
            tags "Sync"
        }

        # Data source relationships
        posSystem -> dataIngestion "Streams transactions" {
            tags "Realtime,Async"
            properties {
                "protocol" "Kafka"
                "volume" "5M/day"
            }
        }

        inventorySystem -> dataIngestion "Sends inventory updates" {
            tags "Realtime,Async"
            properties {
                "protocol" "CDC"
                "volume" "10M/day"
            }
        }

        customerSystem -> dataIngestion "Syncs customer data" {
            tags "Batch"
            properties {
                "protocol" "S3"
                "frequency" "Hourly"
            }
        }

        supplyChainSystem -> dataIngestion "Sends logistics data" {
            tags "Batch"
            properties {
                "protocol" "SFTP"
                "frequency" "Daily"
            }
        }

        weatherAPI -> dataIngestion "Weather data feed" {
            tags "Realtime,External"
            properties {
                "protocol" "REST API"
            }
        }

        socialMedia -> dataIngestion "Social sentiment" {
            tags "Realtime,External"
            properties {
                "protocol" "Streaming API"
            }
        }

        # Output relationships
        analyticsAI -> operationalSystems "Provides insights" {
            tags "Async"
            properties {
                "format" "REST API, Events"
            }
        }

        recommendationEngine -> mobileApp "Product recommendations" {
            tags "Sync,Realtime"
            properties {
                "latency" "<100ms"
            }
        }

        fraudDetection -> posSystem "Fraud scores" {
            tags "Sync,Critical"
            properties {
                "latency" "<50ms"
            }
        }

        # ========================================
        # Internal Container Relationships
        # ========================================

        # Ingestion flow
        dataIngestion -> messageQueue "Publishes events" {
            tags "Async,Internal"
        }

        dataIngestion -> dataLake "Stores raw data" {
            tags "Batch,Internal"
        }

        # Stream processing
        messageQueue -> streamProcessor "Consumes events" {
            tags "Realtime,Internal"
        }

        streamProcessor -> cache "Updates cache" {
            tags "Realtime,Internal"
        }

        streamProcessor -> dataWarehouse "Writes aggregates" {
            tags "Realtime,Internal"
        }

        streamProcessor -> alertEngine "Triggers alerts" {
            tags "Realtime,Internal"
        }

        # Batch processing
        dataLake -> batchProcessor "Processes data" {
            tags "Batch,Internal"
        }

        batchProcessor -> dataWarehouse "Loads refined data" {
            tags "Batch,Internal"
        }

        batchProcessor -> featureStore "Updates features" {
            tags "Batch,Internal"
        }

        # ML pipeline
        featureStore -> mlPlatform "Provides features" {
            tags "Sync,Internal"
        }

        mlPlatform -> modelRegistry "Registers models" {
            tags "Sync,Internal"
        }

        modelRegistry -> modelServing "Deploys models" {
            tags "Sync,Internal"
        }

        # Analytics flow
        dataWarehouse -> analyticsEngine "Serves data" {
            tags "Sync,Internal"
        }

        analyticsEngine -> reportingService "Powers dashboards" {
            tags "Sync,Internal"
        }

        cache -> analyticsEngine "Speeds queries" {
            tags "Sync,Internal"
        }

        # ML model specific flows
        modelServing -> demandForecasting "Serves forecast model" {
            tags "Sync,Internal"
        }

        modelServing -> pricingOptimization "Serves pricing model" {
            tags "Sync,Internal"
        }

        modelServing -> customerSegmentation "Serves segmentation" {
            tags "Sync,Internal"
        }

        modelServing -> fraudDetection "Serves fraud model" {
            tags "Sync,Internal"
        }

        modelServing -> recommendationEngine "Serves recommendations" {
            tags "Sync,Internal"
        }

        # Vector search
        recommendationEngine -> vectorDB "Similarity search" {
            tags "Sync,Internal"
        }

        # Orchestration
        orchestrator -> batchProcessor "Schedules jobs" {
            tags "Async,Internal"
        }

        orchestrator -> mlPlatform "Triggers training" {
            tags "Async,Internal"
        }

        # Governance
        dataGovernance -> dataLake "Manages metadata" {
            tags "Sync,Internal"
        }

        dataGovernance -> dataWarehouse "Enforces policies" {
            tags "Sync,Internal"
        }

        # Monitoring
        monitoring -> streamProcessor "Collects metrics" {
            tags "Async,Internal"
        }

        monitoring -> batchProcessor "Collects metrics" {
            tags "Async,Internal"
        }

        monitoring -> mlPlatform "Collects metrics" {
            tags "Async,Internal"
        }

        # Component relationships
        cdcConnector -> streamCollector "Streams changes" {
            tags "Realtime,Internal"
        }

        streamCollector -> messageQueue "Publishes" {
            tags "Realtime,Internal"
        }

        eventEnricher -> windowAggregator "Enriched events" {
            tags "Realtime,Internal"
        }

        windowAggregator -> anomalyDetector "Aggregated data" {
            tags "Realtime,Internal"
        }

        anomalyDetector -> alertEngine "Anomalies" {
            tags "Realtime,Internal"
        }

        etlPipeline -> dataQuality "Validates data" {
            tags "Batch,Internal"
        }

        dataQuality -> featureEngineering "Clean data" {
            tags "Batch,Internal"
        }

        experimentTracker -> modelRegistry "Promotes models" {
            tags "Sync,Internal"
        }

        modelTraining -> experimentTracker "Logs experiments" {
            tags "Sync,Internal"
        }

        modelRegistry -> modelServing "Deploy models" {
            tags "Sync,Internal"
        }
    }

    views {
        # ========================================
        # System Landscape
        # ========================================

        systemLandscape "analytics-landscape" "Complete analytics ecosystem" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # System Context
        # ========================================

        systemContext analyticsAI "analytics-context" "Analytics & AI Platform context" {
            include *
            autoLayout lr 250 100
        }

        # ========================================
        # Container View
        # ========================================

        container analyticsAI "analytics-containers" "Analytics platform containers" {
            include *
            autoLayout tb 200 100
        }

        # ========================================
        # Component Views
        # ========================================

        component dataIngestion "ingestion-components" "Data ingestion components" {
            include *
            autoLayout lr 200 100
        }

        component streamProcessor "stream-components" "Stream processing components" {
            include *
            autoLayout lr 200 100
        }

        component batchProcessor "batch-components" "Batch processing components" {
            include *
            autoLayout lr 200 100
        }

        component mlPlatform "ml-components" "ML platform components" {
            include *
            autoLayout tb 150 100
        }

        # ========================================
        # Dynamic Views
        # ========================================

        dynamic analyticsAI "real-time-pipeline" "Real-time data pipeline" {
            posSystem -> dataIngestion "Transaction event"
            dataIngestion -> messageQueue "Publish to Kafka"

            # Parallel processing
            {
                messageQueue -> streamProcessor "Process stream"
                messageQueue -> fraudDetection "Fraud check"
            }

            streamProcessor -> cache "Update cache"
            streamProcessor -> dataWarehouse "Store aggregates"

            # Parallel analytics
            {
                cache -> analyticsEngine "Serve from cache"
                dataWarehouse -> reportingService "Update dashboard"
            }

            fraudDetection -> posSystem "Return score"
            analyticsEngine -> businessUser "Real-time metrics"

            autoLayout lr 250 100
        }

        dynamic analyticsAI "batch-pipeline" "Batch processing pipeline" {
            orchestrator -> dataIngestion "Trigger daily batch"

            # Parallel ingestion
            {
                supplyChainSystem -> dataIngestion "Supply data"
                customerSystem -> dataIngestion "Customer data"
                weatherAPI -> dataIngestion "Weather data"
            }

            dataIngestion -> dataLake "Store raw data"
            orchestrator -> batchProcessor "Start processing"
            batchProcessor -> dataLake "Read raw data"

            # Parallel processing
            {
                batchProcessor -> dataQuality "Quality checks"
                batchProcessor -> featureEngineering "Compute features"
            }

            dataQuality -> dataWarehouse "Load clean data"
            featureEngineering -> featureStore "Update features"

            # Parallel downstream
            {
                dataWarehouse -> analyticsEngine "Refresh cubes"
                featureStore -> mlPlatform "Available for training"
            }

            autoLayout tb 200 100
        }

        dynamic mlPlatform "ml-training-pipeline" "ML model training pipeline" {
            dataScientist -> notebookEnvironment "Explore data"
            notebookEnvironment -> dataWarehouse "Query data"
            dataScientist -> experimentTracker "Create experiment"

            experimentTracker -> modelTraining "Start training"

            # Parallel data access
            {
                modelTraining -> featureStore "Load features"
                modelTraining -> dataLake "Load training data"
            }

            modelTraining -> experimentTracker "Log metrics"
            experimentTracker -> modelRegistry "Register model"

            dataScientist -> modelRegistry "Review model"
            modelRegistry -> modelServing "Deploy to production"

            # Parallel model deployment
            {
                modelServing -> demandForecasting "Update forecast model"
                modelServing -> pricingOptimization "Update pricing model"
                modelServing -> fraudDetection "Update fraud model"
            }

            autoLayout tb 200 100
        }

        dynamic recommendationEngine "recommendation-flow" "Product recommendation flow" {
            mobileApp -> recommendationEngine "Request recommendations"

            # Parallel data fetching
            {
                recommendationEngine -> cache "Get user profile"
                recommendationEngine -> featureStore "Get user features"
                recommendationEngine -> vectorDB "Find similar items"
            }

            recommendationEngine -> modelServing "Score candidates"
            modelServing -> recommendationEngine "Ranked products"

            # Parallel post-processing
            {
                recommendationEngine -> cache "Cache results"
                recommendationEngine -> messageQueue "Log impression"
            }

            recommendationEngine -> mobileApp "Return recommendations"

            autoLayout lr 250 100
        }

        dynamic analyticsAI "data-governance-flow" "Data governance and lineage" {
            dataIngestion -> dataGovernance "Register dataset"
            dataGovernance -> dataLake "Tag with metadata"

            batchProcessor -> dataGovernance "Track transformation"
            dataGovernance -> dataWarehouse "Apply policies"

            # Parallel governance
            {
                dataGovernance -> dataLake "Scan for PII"
                dataGovernance -> dataWarehouse "Enforce retention"
                dataGovernance -> featureStore "Track lineage"
            }

            dataAnalyst -> dataGovernance "Request access"
            dataGovernance -> dataWarehouse "Grant permission"
            dataAnalyst -> analyticsEngine "Query data"

            autoLayout tb 200 100
        }

        # ========================================
        # Deployment View (simplified for parser)
        # ========================================

        deployment analyticsAI "Production" "analytics-deployment" "Production deployment on cloud" {
            include *
            autoLayout lr 300 100
        }

        # ========================================
        # Styles
        # ========================================

        themes "https://static.structurizr.com/themes/default/theme.json" "../../../freshmart-theme.json"

        styles {
            element "ML" {
                shape Robot
                background "#9370DB"
                color "#FFFFFF"
            }

            element "Data" {
                shape Cylinder
                background "#008080"
                color "#FFFFFF"
            }

            element "Realtime" {
                background "#2ECC71"
                color "#FFFFFF"
            }

            element "Batch" {
                background "#3498DB"
                color "#FFFFFF"
            }

            relationship "Realtime" {
                color "#2ECC71"
                thickness 3
                style Solid
            }

            relationship "Batch" {
                color "#3498DB"
                thickness 2
                style Dashed
            }

            relationship "ML" {
                color "#9370DB"
                thickness 2
            }
        }
    }
}