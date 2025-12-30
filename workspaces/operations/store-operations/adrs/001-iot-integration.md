# ADR-001: IoT Gateway and Sensor Data Pipeline Architecture

## Status
Accepted

## Context
FreshMart operates 2,500 stores with diverse IoT devices including temperature sensors for refrigeration units, footfall counters for traffic analysis, HVAC systems for climate control, and electronic shelf labels. We need a scalable architecture to:
- Connect and manage 100,000+ IoT devices
- Process millions of telemetry events per hour
- Detect threshold violations and trigger alerts in real-time
- Support multiple communication protocols (MQTT, AMQP, HTTP)
- Enable edge processing to reduce cloud bandwidth and latency

## Decision
We will implement an IoT integration architecture using Azure IoT Hub as the central gateway with the following components:

1. **Device Registry**: Azure IoT Hub's device identity registry for secure device authentication and management
2. **Protocol Support**: Native MQTT broker for lightweight sensor communication, with AMQP and HTTP gateways for legacy devices
3. **Telemetry Processing**: Azure Stream Analytics for real-time aggregation, filtering, and anomaly detection at scale
4. **Alert Engine**: Azure Event Grid for event-driven alerting with guaranteed delivery
5. **Event Bus Integration**: Apache Kafka bridge for enterprise-wide event distribution

The data pipeline will:
- Authenticate devices using X.509 certificates or symmetric keys
- Route telemetry based on device type and priority
- Apply windowed aggregations (1-min, 5-min, 15-min) for dashboards
- Evaluate alert rules against configurable thresholds
- Forward events to Kafka for downstream consumption

## Consequences

### Positive
- Unified management of all store IoT devices through single platform
- Sub-second alert detection for critical thresholds (temperature, HVAC failures)
- Horizontal scalability to handle peak telemetry loads during busy hours
- Reduced operational complexity with managed Azure services
- Built-in device twin for configuration management

### Negative
- Azure IoT Hub vendor lock-in
- Cost scales with message volume and device count
- Requires network connectivity for cloud-based processing
- Complex debugging across distributed pipeline components

### Mitigation
- Abstract IoT Hub behind internal API layer for portability
- Implement tiered data retention to manage costs
- Deploy edge modules for critical local processing
- Centralize logging in Azure Monitor with correlation IDs

## Implementation
1. Deploy Azure IoT Hub with S1 tier (400K messages/day per unit)
2. Configure device provisioning service for automated enrollment
3. Create Stream Analytics jobs for each telemetry type
4. Set up Event Grid subscriptions for alert routing
5. Deploy Kafka Connect for event bus integration
6. Implement device health monitoring and auto-remediation

## References
- [Azure IoT Hub Documentation](https://docs.microsoft.com/azure/iot-hub)
- [Stream Analytics Windowing](https://wiki.freshmart.com/stream-analytics-patterns)
- [Device Onboarding Procedures](https://wiki.freshmart.com/iot-device-onboarding)
