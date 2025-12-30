# ADR-001: Event Sourcing with CQRS for Inventory State Management

## Status
Accepted

## Context
FreshMart's inventory system must handle 50M+ daily events across 500,000 SKUs and 2,500+ locations. We need an architecture that provides:
- Complete audit trail of all inventory movements for compliance and dispute resolution
- Ability to reconstruct inventory state at any point in time
- High throughput for both write operations (adjustments, transfers, reservations) and read operations (stock queries, ATP lookups)
- Support for complex temporal queries for analytics and forecasting

Traditional CRUD-based inventory systems lose historical context and cannot efficiently support these requirements.

## Decision
We will implement Event Sourcing with CQRS (Command Query Responsibility Segregation) as the foundational architecture pattern:

### Event Sourcing
1. **Immutable Event Log**: All inventory state changes are captured as immutable events in EventStoreDB
2. **Inventory Aggregate**: Domain aggregate root that processes commands and emits events
3. **Event Types**: Adjust, Reserve, Release, Transfer, Count, Receive, Ship, Allocate
4. **Stream-per-SKU-Location**: Each SKU-location combination has its own event stream for optimal partitioning

### CQRS
1. **Command Service**: Handles write operations (Java/Spring Boot, 10K commands/sec throughput)
   - Command Handler validates and processes inventory commands
   - Commands are validated against business rules before execution
   - Events are appended to EventStoreDB and published to Kafka

2. **Query Service**: Handles read operations (Java/Spring Boot, <50ms latency)
   - Stock Level Projection (PostgreSQL): Current stock by SKU and location (1.25B records)
   - Availability Projection (Redis): Real-time ATP with <5ms lookup
   - Movement History (TimescaleDB): 2-year transaction-level audit trail

3. **Event Processor**: Apache Flink-based event consumer (1M events/sec)
   - Updates all read model projections asynchronously
   - Generates alerts for stock thresholds
   - Aggregates data for analytics

## Consequences

### Positive
- **Complete Audit Trail**: Every inventory change is recorded with full context, enabling regulatory compliance and dispute resolution
- **Temporal Queries**: Can reconstruct inventory state at any point in time for auditing and analysis
- **Optimized Read/Write**: Separate models allow each to be optimized for its specific workload
- **Scalability**: Event streams partition naturally by SKU-location; projections can be rebuilt from events
- **Debugging**: Event replay enables reproduction of any historical state for troubleshooting
- **Analytics**: Rich event data feeds downstream analytics and ML systems

### Negative
- **Eventual Consistency**: Read models may lag behind writes by up to 100ms
- **Increased Complexity**: Multiple data stores and event processing infrastructure to maintain
- **Storage Requirements**: EventStoreDB retains all events indefinitely (5B+ events, growing)
- **Projection Rebuild Time**: Full rebuild from events can take hours for large projections
- **Learning Curve**: Team must understand event sourcing patterns and eventual consistency

### Mitigation
- **Consistency**: Use Redis cache for critical ATP queries; implement optimistic concurrency control
- **Complexity**: Standardize on Axon Framework for event sourcing patterns; comprehensive monitoring
- **Storage**: Implement event compaction for analytics-only streams; use snapshots for aggregate optimization
- **Rebuild Time**: Parallel projection rebuilds; incremental replay from snapshots
- **Training**: Document patterns; conduct architecture workshops for engineering teams

## Implementation

### Infrastructure
1. **EventStoreDB**: Primary event store with 10M+ streams, forever retention
2. **Apache Kafka**: Event bus with 50+ topics, 100+ partitions, 1M msg/sec capacity
3. **Apache Flink**: Stream processor for projection updates and alert generation
4. **PostgreSQL**: Stock level projection with SKU+Location composite index
5. **Redis Cluster**: ATP cache with 512GB memory, 1M ops/sec
6. **TimescaleDB**: Movement history with time-based partitioning

### Event Schema Example
```json
{
  "event_type": "InventoryAdjusted",
  "event_id": "evt_abc123",
  "aggregate_id": "SKU123-STORE001",
  "timestamp": "2024-01-15T10:30:00Z",
  "data": {
    "sku": "SKU123",
    "location": "STORE001",
    "quantity_delta": -5,
    "reason": "SALE",
    "reference": "TXN_12345",
    "operator_id": "user_456"
  },
  "metadata": {
    "correlation_id": "corr_789",
    "causation_id": "cmd_012"
  }
}
```

### Saga Pattern for Distributed Operations
Complex operations like transfers use the Saga Orchestrator (Axon Saga):
1. Transfer initiated: Reserve at source location
2. Decrement source inventory
3. Increment destination inventory
4. Complete transfer or compensate on failure

## References
- [Event Sourcing Pattern](https://wiki.freshmart.com/event-sourcing)
- [CQRS Pattern](https://wiki.freshmart.com/cqrs)
- [EventStoreDB Documentation](https://wiki.freshmart.com/eventstoredb)
- [Axon Framework Guide](https://wiki.freshmart.com/axon-framework)
