# Manufacturing IoT - Industry 4.0 Smart Factory

A comprehensive Industry 4.0 smart manufacturing platform demonstrating modern IoT, edge computing, and predictive analytics architecture.

## Overview

This example models a complete smart factory digital infrastructure for an automotive parts manufacturer, including:
- Manufacturing Execution System (MES) as the central orchestrator
- SCADA systems for real-time machine control
- IoT platform for sensor data collection
- Predictive maintenance with machine learning
- Integration with enterprise ERP systems
- Energy management and optimization
- Digital twin for production simulation

## Architecture Highlights

### Scale
- **6 user personas:** Plant Manager, Operator, Maintenance Tech, Quality Engineer, Supply Chain Manager, Executive
- **10 software systems:** MES, SCADA, ERP Integration, Quality, Predictive Maintenance, Energy, Supply Chain, Reporting, IoT Platform, Digital Twin
- **30+ containers:** Data collectors, analytics engines, dashboards, APIs, databases, message brokers, ML models
- **Component detail:** Predictive Maintenance system fully decomposed
- **10+ deployment nodes:** Edge gateways, plant servers, cloud infrastructure

### Technology Stack
- **Edge:** Node-RED, MQTT, OPC UA
- **Backend:** Java Spring Boot, Python, Go
- **Databases:** PostgreSQL, InfluxDB (time-series), MongoDB
- **Messaging:** Apache Kafka, MQTT
- **ML/AI:** Python scikit-learn, TensorFlow
- **Cloud:** AWS (EC2, S3, SageMaker, Kinesis)
- **Standards:** OPC UA, ISA-95, MTConnect

## System Groups

### Shop Floor Systems
Direct production and machine control:
- SCADA System
- IoT Platform
- Digital Twin

### Plant Operations
Manufacturing execution and quality:
- Manufacturing Execution System (MES)
- Quality Management System
- Energy Management

### Enterprise Systems
Business-level integration:
- ERP Integration
- Supply Chain Management
- Executive Reporting

### Cloud Services
Analytics and advanced capabilities:
- Predictive Maintenance
- Machine Learning Platform

## Key Workflows

### Production Order Flow (Dynamic View)
1. ERP releases production order
2. MES schedules on production line
3. SCADA configures machines
4. IoT monitors production
5. Quality checks parts
6. MES reports completion to ERP

### Alert Escalation (Dynamic View)
1. Sensor detects anomaly
2. IoT platform aggregates data
3. ML model predicts failure
4. Alert service creates incident
5. Maintenance tech receives notification
6. Work order created in MES
7. Part ordered from supply chain

## Deployment Architecture

### Three-Tier Architecture

#### Edge Tier (Plant Floor)
- Edge Gateways for data collection
- Local MQTT brokers
- OPC UA servers on machines
- Real-time data preprocessing

#### Plant Tier (On-Premise)
- MES application servers
- SCADA servers
- Plant databases (PostgreSQL, InfluxDB)
- Kafka message brokers
- Quality system servers

#### Cloud Tier (AWS)
- Predictive maintenance analytics
- Machine learning training
- Data lake (S3)
- Executive reporting
- Digital twin simulation

## Industrial Protocols & Standards

### OT/IT Integration
- **OPC UA:** Unified machine communication
- **MQTT:** Lightweight IoT messaging
- **ISA-95:** Manufacturing integration standard
- **MTConnect:** Machine tool data collection

### Data Flow
- Edge: Collect at 100ms intervals
- Plant: Aggregate to 1-second metrics
- Cloud: Batch hourly for analytics

## Perspectives

Elements are annotated with multiple perspectives:
- **Criticality:** Production-critical, operational, analytical
- **Security:** OT network, DMZ, IT network, cloud
- **Data Retention:** Real-time, 30-day, 1-year, archival
- **Latency Requirements:** <100ms, <1s, <1min, batch

## Views Included

1. **System Landscape** - Complete smart factory ecosystem
2. **System Context - MES** - Manufacturing execution system context
3. **System Context - IoT** - IoT platform context
4. **Container View - MES** - MES internal architecture
5. **Container View - IoT Platform** - IoT data pipeline
6. **Container View - Predictive Maintenance** - ML-based maintenance
7. **Component View - Predictive Maintenance** - ML pipeline internals
8. **Dynamic View - Production Order** - Order fulfillment workflow
9. **Dynamic View - Alert Escalation** - Failure prediction and response
10. **Deployment View** - 3-tier edge/plant/cloud infrastructure

## ADRs (Architecture Decision Records)

### ADR-001: Edge Computing Strategy
**Decision:** Deploy edge gateways for local data preprocessing

**Rationale:**
- Reduce network bandwidth to cloud (99% reduction)
- Enable real-time response (<100ms) for safety
- Maintain operation during cloud connectivity loss
- Comply with data sovereignty requirements

### ADR-002: Time-Series Database Selection
**Decision:** Use InfluxDB for sensor data storage

**Rationale:**
- Optimized for time-series data patterns
- Built-in downsampling and retention policies
- Superior query performance for time-range queries
- Handles high ingestion rates (100k+ points/sec)

### ADR-003: Kafka for Plant-to-Cloud Streaming
**Decision:** Use Apache Kafka for plant-level event streaming

**Rationale:**
- Durable message storage for replay capability
- High throughput for 1000+ machines
- Decouples producers and consumers
- Enables multiple analytics consumers

### ADR-004: Hybrid ML Deployment
**Decision:** Train models in cloud, deploy inference to edge

**Rationale:**
- Leverage cloud compute for intensive training
- Low-latency inference at edge for real-time decisions
- Centralized model versioning and management
- Cost optimization (edge inference cheaper than streaming)

## Documentation

### Production Processes
Detailed manufacturing workflows:
- Work cell configuration
- Material flow and tracking
- Changeover procedures
- Quality inspection protocols

### Machine Learning Models
- Anomaly detection for vibration sensors
- Predictive failure models for bearings
- Energy consumption forecasting
- Quality defect classification (computer vision)

## OT Security

### Network Segmentation
- Level 0-2: Purdue Model industrial zones
- Firewalls between OT and IT networks
- DMZ for plant-to-cloud communication
- VPN for remote access

### Security Controls
- Asset inventory and vulnerability management
- Network monitoring and anomaly detection
- Role-based access control
- Audit logging for all operations

## Running This Example

### Validate
```bash
cargo run -- validate examples/large/manufacturing-iot/workspace.dsl
```

### Render All Views
```bash
cargo run -- render --workspace examples/large/manufacturing-iot/workspace.dsl --output ./manufacturing-diagrams
```

### Interactive Web View
```bash
cargo run -- serve --workspace examples/large/manufacturing-iot/workspace.dsl --port 8080
```

## Customization Ideas

- Add more production lines with different equipment
- Expand to include warehouse management
- Model AGV (automated guided vehicle) fleet
- Add computer vision quality inspection
- Include sustainability metrics (carbon footprint)
- Model supply chain with vendor systems
- Add simulation and what-if analysis capabilities

## Learning Objectives

This example demonstrates:
1. How to model OT/IT convergence architectures
2. Edge-to-cloud data pipeline design
3. Time-series data handling patterns
4. Machine learning integration in production systems
5. Multi-tier deployment with different latency requirements
6. Industrial protocol integration (OPC UA, MQTT)
7. Organizing by Purdue Model / ISA-95 levels
8. Security patterns for industrial control systems
9. Event-driven architecture with Kafka
10. Component-level decomposition of ML systems

## Industry 4.0 Capabilities

### Smart Factory Features
- **Real-time visibility:** Live production dashboards
- **Predictive maintenance:** ML-based failure prediction
- **Quality 4.0:** Automated defect detection
- **Energy optimization:** AI-driven energy management
- **Digital twin:** Virtual factory simulation

### Business Benefits
- 30% reduction in unplanned downtime
- 20% improvement in OEE (Overall Equipment Effectiveness)
- 15% energy cost savings
- 10% quality improvement
- 25% faster changeovers

## Related Standards & Frameworks
- ISA-95: Enterprise-Control System Integration
- ISA-99/IEC 62443: Industrial Cybersecurity
- RAMI 4.0: Reference Architecture Model Industry 4.0
- Purdue Enterprise Reference Architecture (PERA)
- MTConnect: Manufacturing data standard
- OPC UA: Open Platform Communications Unified Architecture
