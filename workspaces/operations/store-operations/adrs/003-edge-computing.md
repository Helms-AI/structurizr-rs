# ADR-003: Store-Level Edge Computing Strategy

## Status
Accepted

## Context
With 100,000+ IoT devices generating continuous telemetry, centralized cloud processing creates challenges:
- Network bandwidth costs for raw sensor data transmission
- Latency in critical alert scenarios (e.g., refrigeration failures)
- Dependency on WAN connectivity for basic store operations
- Privacy concerns with transmitting all data to cloud
- High cloud compute costs for pre-processing operations

We need an edge computing strategy that balances local processing with cloud analytics.

## Decision
We will deploy edge computing nodes at each store with Azure IoT Edge runtime for local data processing:

1. **Edge Gateway**: Industrial PC running Azure IoT Edge at each store
2. **Local Processing Modules**: Containerized services for data aggregation and filtering
3. **Store-and-Forward**: Message queuing for offline resilience
4. **Alert Detection**: Local threshold evaluation for critical alerts
5. **Data Reduction**: Aggregate raw telemetry before cloud transmission

Edge processing tiers:
- **Tier 1 (Real-time)**: Temperature threshold alerts, HVAC failures - process locally in <1 second
- **Tier 2 (Near real-time)**: Traffic counting, energy monitoring - aggregate locally, send every minute
- **Tier 3 (Batch)**: Equipment diagnostics, historical trends - buffer locally, sync hourly

Data reduction strategy:
- Raw sensor readings: 1 per second per device
- After edge aggregation: 1 per minute statistical summary (min, max, avg, count)
- Bandwidth reduction: 98% for typical telemetry streams
- Full-fidelity data retained for anomaly investigation

## Consequences

### Positive
- 98% reduction in cloud ingress bandwidth and associated costs
- Sub-second local alerting for critical thresholds
- Continued store operations during WAN outages (up to 24 hours)
- Reduced cloud compute costs through pre-aggregation
- Lower latency for time-sensitive decisions

### Negative
- Additional hardware costs ($2,000 per store edge node)
- Increased deployment and maintenance complexity
- Need for remote edge device management
- Potential for inconsistent processing across stores

### Mitigation
- Centralized edge device management through Azure IoT Edge
- Standardized edge modules deployed via container registry
- Automated health monitoring and self-healing
- Tiered rollout with pilot stores before full deployment

## Implementation
1. Procure and deploy edge hardware to pilot stores (50 locations)
2. Develop edge modules for each telemetry type
3. Configure IoT Edge runtime with deployment manifests
4. Implement store-and-forward with 24-hour message retention
5. Create edge monitoring dashboard in Azure Monitor
6. Develop edge update and rollback procedures
7. Expand to all 2,500 stores over 6-month rollout

## Cost Analysis
| Item | Per Store | Total (2,500 stores) |
|------|-----------|---------------------|
| Edge Hardware | $2,000 | $5,000,000 |
| Annual Maintenance | $200 | $500,000 |
| Cloud Savings (Annual) | $1,500 | $3,750,000 |
| Net Annual Benefit | - | $3,250,000 |

Break-even: 18 months after full deployment

## References
- [Azure IoT Edge Documentation](https://docs.microsoft.com/azure/iot-edge)
- [Edge Computing Patterns](https://wiki.freshmart.com/edge-patterns)
- [Store Network Architecture](https://wiki.freshmart.com/store-network)
