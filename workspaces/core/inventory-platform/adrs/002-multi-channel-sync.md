# ADR-002: Multi-Channel Inventory Synchronization Strategy

## Status
Accepted

## Context
FreshMart operates across multiple sales channels that all draw from shared inventory:
- **2,500 Retail Stores**: POS systems processing 5M transactions daily
- **E-Commerce Platform**: Online orders requiring real-time availability checks
- **5 Distribution Centers**: Warehouse operations with receipt, pick, and ship events
- **Wholesale Channel**: B2B orders with bulk allocation requirements

Each channel has different latency requirements:
- E-Commerce needs sub-100ms availability checks for cart and checkout
- POS needs immediate decrement on sale completion
- Warehouse needs accurate pick lists with reserved quantities
- Wholesale needs bulk availability for large order quotes

Without real-time synchronization, we risk overselling (selling inventory that does not exist) or underselling (showing items as unavailable when stock exists elsewhere).

## Decision
We will implement a near-real-time multi-channel synchronization strategy using event-driven architecture:

### Event Ingestion
All channels publish inventory-affecting events to Apache Kafka:
1. **POS System** -> `pos.transactions` topic (Sale, Return, Void events)
2. **E-Commerce** -> `ecom.orders` topic (Order, Cancel, Return events)
3. **WMS** -> `wms.movements` topic (Receipt, Pick, Ship events)
4. **Supply Chain** -> `supply.receipts` topic (PO, Receipt, Shipment events)

### Consistency Model: Eventually Consistent with Reservations
1. **Soft Reservations**: E-commerce and wholesale use reservations to temporarily hold inventory
2. **Reservation TTL**: 30-minute default expiration with configurable override
3. **ATP Calculation**: Available-to-Promise = On-Hand - Reserved - Allocated
4. **Optimistic Updates**: UI shows estimated availability; server validates on commit

### Conflict Resolution
When multiple channels attempt to claim the same inventory:
1. **First-Write-Wins**: Reservations are processed in order received
2. **Channel Priority**: Configurable priority (e.g., E-Commerce > Wholesale for promotions)
3. **Backorder Support**: If stock depleted, create backorder and notify customer
4. **Reallocation Engine**: Periodically rebalances allocations based on demand signals

### Sync Latency Targets
| Channel | Event Type | Target Latency | Actual |
|---------|------------|----------------|--------|
| POS | Sale Decrement | <1s | 500ms |
| E-Commerce | ATP Query | <100ms | 75ms |
| WMS | Pick Decrement | <2s | 1.2s |
| Analytics | Aggregation | <5min | 3min |

## Consequences

### Positive
- **Near-Real-Time Visibility**: All channels see consistent inventory within 500ms-2s
- **Oversell Prevention**: Reservation system prevents double-selling of same inventory
- **Channel Flexibility**: Each channel can operate independently with eventual sync
- **Scalability**: Event-driven architecture handles 50M+ daily events
- **Resilience**: Channels continue operating if others are temporarily unavailable

### Negative
- **Eventual Consistency**: Brief windows where channels may show different availability
- **Reservation Complexity**: Must handle expiration, extension, and conversion to hard allocation
- **Conflict Scenarios**: Edge cases where multiple channels race for last item
- **Monitoring Overhead**: Need comprehensive observability across all sync paths
- **Customer Experience**: Occasional "item no longer available" at checkout

### Mitigation
- **Safety Stock**: Maintain buffer inventory for high-velocity items
- **Real-Time Alerts**: Flink CEP generates immediate alerts when sync lag exceeds threshold
- **Graceful Degradation**: If ATP cache unavailable, fall back to Stock Level query
- **Customer Communication**: Clear messaging when reservation expires or stock depleted
- **Sync Health Dashboard**: Real-time visualization of cross-channel consistency

## Implementation

### Event Flow Architecture
```
POS System -----> Kafka (pos.transactions) -----> Flink -----> Stock Level Projection
E-Commerce ----> Kafka (ecom.orders) -----------> Flink -----> ATP Projection (Redis)
WMS -----------> Kafka (wms.movements) ---------> Flink -----> Movement History
Supply Chain --> Kafka (supply.receipts) -------> Flink -----> All Projections
```

### ATP Calculation Logic
```
ATP = on_hand_quantity
    - reserved_quantity (soft holds)
    - allocated_quantity (hard commits)
    - safety_stock (buffer)
    + in_transit_quantity (if within delivery window)
```

### Reservation Service Design
- **Redis-backed**: Sub-5ms reservation lookups and updates
- **TTL Management**: Scheduler expires stale reservations every minute
- **Extension API**: Allow reservation extension before expiration
- **Conversion**: On order confirmation, convert reservation to allocation

### Kafka Topic Configuration
| Topic | Partitions | Retention | Consumer Groups |
|-------|------------|-----------|-----------------|
| pos.transactions | 100 | 7 days | flink-processor, analytics-consumer |
| ecom.orders | 50 | 7 days | flink-processor, fulfillment-consumer |
| wms.movements | 50 | 7 days | flink-processor, analytics-consumer |
| supply.receipts | 25 | 7 days | flink-processor, planning-consumer |

## References
- [Event-Driven Architecture Guide](https://wiki.freshmart.com/eda)
- [Reservation Service Design](https://wiki.freshmart.com/reservations)
- [Kafka Configuration Standards](https://wiki.freshmart.com/kafka-config)
- [ATP Calculation Logic](https://wiki.freshmart.com/atp-calculation)
