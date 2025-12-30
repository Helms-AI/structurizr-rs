# Store Operations Platform Documentation

## System Overview

The FreshMart Store Operations Platform is the unified store management system that coordinates IoT devices, workforce scheduling, task management, and digital signage across all FreshMart retail locations. It enables smart store capabilities through real-time monitoring, automated alerting, and ML-powered optimization.

## Architecture Documentation

- [Architecture Overview](architecture.md) - Detailed architectural design
- [API Documentation](api.md) - Store Operations API specifications
- [IoT Integration Guide](iot-integration.md) - Device connectivity and protocols
- [Operations Runbook](runbook.md) - Operational procedures

## Key Capabilities

### IoT Integration
- Azure IoT Hub managing 100,000+ connected devices
- Support for MQTT, AMQP, and HTTP protocols
- Real-time telemetry from temperature sensors, footfall counters, and HVAC systems
- Edge processing with Stream Analytics for reduced latency
- Event Grid-based alerting for threshold violations

### Task Management
- Camunda-powered workflow engine processing 500,000+ tasks daily
- ML-based priority engine for dynamic task prioritization
- OR-Tools optimization for optimal associate assignment
- Real-time task tracking via Store Dashboard and Associate Mobile App
- Automated task creation from IoT alerts

### Digital Signage
- Centralized management of 25,000+ digital displays
- Electronic shelf label (ESL) infrastructure with 2M+ price tags
- Content scheduling with Node.js services
- Real-time price synchronization with inventory systems

### Workforce Scheduling
- Mixed integer programming for optimal schedule generation
- Prophet-based customer traffic forecasting
- Labor law compliance verification with rule engine
- Integration with HR System for employee constraints
- Push notifications to associates via React Native mobile app

### Real-Time Alerting
- Event-driven architecture with Apache Kafka
- Temperature threshold monitoring for refrigeration units
- HVAC anomaly detection
- Automatic escalation to dashboard and mobile devices
- Automated repair task creation

## Integration Guide

### Task Operations

Create a new task:
```http
POST /api/v1/tasks
Authorization: Bearer {token}
Content-Type: application/json

{
  "title": "Restock Dairy Aisle",
  "description": "Low inventory detected in dairy section",
  "store_id": "store_2341",
  "priority": "high",
  "due_in_minutes": 30,
  "category": "inventory",
  "source": "inventory_alert"
}
```

Complete a task:
```http
PUT /api/v1/tasks/{task_id}/complete
Authorization: Bearer {token}
Content-Type: application/json

{
  "completed_by": "associate_8821",
  "notes": "Restocked 24 units of milk, 12 units of cheese",
  "completion_time": "2024-01-15T14:32:00Z"
}
```

### Device Operations

Register a new IoT device:
```http
POST /api/v1/devices
Authorization: Bearer {token}
Content-Type: application/json

{
  "device_id": "temp_sensor_0042",
  "device_type": "temperature_sensor",
  "store_id": "store_2341",
  "location": "Freezer Unit 3",
  "protocol": "MQTT",
  "thresholds": {
    "min_temp": -20,
    "max_temp": -15,
    "alert_delay_seconds": 300
  }
}
```

Query telemetry data:
```http
GET /api/v1/devices/{device_id}/telemetry?from=2024-01-15T00:00:00Z&to=2024-01-15T23:59:59Z
Authorization: Bearer {token}
```

### Scheduling Operations

Generate optimized schedule:
```http
POST /api/v1/schedules/optimize
Authorization: Bearer {token}
Content-Type: application/json

{
  "store_id": "store_2341",
  "week_starting": "2024-01-22",
  "constraints": {
    "min_coverage": 3,
    "max_overtime_hours": 10,
    "preferred_associates": ["assoc_1001", "assoc_1002"]
  },
  "optimization_goals": ["minimize_labor_cost", "maximize_coverage"]
}
```

### Event Streaming

Subscribe to store operations events via Kafka:
- `task.created` - New task created
- `task.assigned` - Task assigned to associate
- `task.completed` - Task marked complete
- `iot.telemetry` - Device sensor reading
- `iot.alert` - Threshold violation detected
- `schedule.published` - New schedule published
- `signage.updated` - Display content changed
- `esl.price_change` - Electronic shelf label updated

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Stores Supported | 2,500 | 2,500 |
| Active Associates | 75,000 | 78,432 |
| Daily Tasks Completed | 500,000 | 523,891 |
| Connected IoT Devices | 100,000 | 102,847 |
| Digital Displays | 25,000 | 25,612 |
| Electronic Shelf Labels | 2,000,000 | 2,143,000 |
| Task Assignment Latency | <5s | 3.2s |
| Alert Response Time | <30s | 22s |
| Schedule Generation Time | <2min | 1.4min |
| System Availability | 99.95% | 99.97% |
| Labor Cost Reduction | 15% | 17.3% |

## Support

- **Store Tech Support**: +1-555-STOREOPS
- **IoT Operations**: iot-oncall@freshmart.com
- **Escalation**: store-ops-escalation@freshmart.com
- **Slack Channel**: #store-operations
- **Wiki**: https://wiki.freshmart.com/store-operations
