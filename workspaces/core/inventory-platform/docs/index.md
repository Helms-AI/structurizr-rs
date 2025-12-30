# Inventory Platform Documentation

## System Overview

The FreshMart Inventory Platform is the enterprise-grade real-time inventory management system that provides unified visibility and control across all FreshMart channels. Built on event sourcing and CQRS architecture, it processes over 50 million inventory events daily, maintaining 99.5% stock accuracy across 500,000+ SKUs and 2,500+ locations.

## Architecture Documentation

- [Architecture Overview](architecture.md) - Detailed CQRS/Event Sourcing design
- [API Documentation](api.md) - Inventory API specifications
- [Event Schema](events.md) - Event types and streaming guide
- [Operations Runbook](runbook.md) - Operational procedures

## Key Capabilities

### Event Sourcing & CQRS
- Immutable event log stored in EventStoreDB with 5B+ events across 10M+ streams
- Complete audit trail with temporal query support for point-in-time reconstruction
- Separate read/write models optimized for their specific workloads
- Event replay capability for projection rebuilds and debugging

### Multi-Channel Synchronization
- Real-time inventory visibility across POS, E-Commerce, and Warehouse systems
- Sub-100ms sync latency for stock level updates
- Consistent available-to-promise (ATP) calculations via Redis-backed projection
- Unified view across 2,500 stores and 5 distribution centers

### Automated Inventory Operations
- Intelligent allocation engine with FIFO, Priority, and Proximity strategies
- Soft reservation system with configurable TTL (30 minutes default)
- Automated cycle count scheduling based on ABC classification
- Stock balancing optimization using OR-Tools for store-to-store transfers

### Real-Time Event Processing
- Apache Flink-based event processor handling 1M events/second
- Complex Event Processing (CEP) for automated alerts: Low stock, Overstock, Variance
- Parallel projection updates for Stock Level, ATP, and Movement History
- 7-day event retention in Kafka with 100+ partitions for scalability

## Integration Guide

### Checking Stock Levels
```http
GET /api/v1/inventory/stock?sku=SKU123&location=STORE001
Authorization: Bearer {token}

Response:
{
  "sku": "SKU123",
  "location": "STORE001",
  "on_hand": 150,
  "reserved": 25,
  "available": 125,
  "in_transit": 50,
  "last_updated": "2024-01-15T10:30:00Z"
}
```

### Reserving Inventory
```http
POST /api/v1/inventory/reserve
Authorization: Bearer {token}
Content-Type: application/json

{
  "sku": "SKU123",
  "location": "STORE001",
  "quantity": 5,
  "reference": "ORDER_12345",
  "ttl_minutes": 30
}

Response:
{
  "reservation_id": "RSV_abc123",
  "status": "confirmed",
  "expires_at": "2024-01-15T11:00:00Z"
}
```

### Adjusting Inventory
```http
POST /api/v1/inventory/adjust
Authorization: Bearer {token}
Content-Type: application/json

{
  "sku": "SKU123",
  "location": "STORE001",
  "quantity_delta": -2,
  "reason": "CYCLE_COUNT",
  "reference": "COUNT_789"
}
```

### Creating Transfers
```http
POST /api/v1/inventory/transfer
Authorization: Bearer {token}
Content-Type: application/json

{
  "sku": "SKU123",
  "source_location": "DC_WEST",
  "destination_location": "STORE001",
  "quantity": 100,
  "priority": "standard"
}
```

### GraphQL Queries
```graphql
query InventorySnapshot {
  inventory(sku: "SKU123") {
    onHand
    reserved
    available
    locations {
      id
      name
      quantity
      lastMovement
    }
    history(days: 7) {
      date
      movements {
        type
        quantity
        reference
      }
    }
  }
}
```

### Event Streaming
Subscribe to inventory events via Kafka:
- `pos.transactions` - POS sale, return, and void events (5M/day)
- `ecom.orders` - E-commerce order and cancellation events
- `supply.receipts` - Supply chain receipt and shipment events
- `wms.movements` - Warehouse receipt, pick, and ship events
- `inventory.reservations` - Reservation create, release, and expire events
- `inventory.alerts` - Low stock, overstock, and variance alerts

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Stock Accuracy | >99% | 99.5% |
| Query Latency (P99) | <50ms | 42ms |
| ATP Lookup Latency | <5ms | 3ms |
| Event Processing Latency | <100ms | 75ms |
| Command Throughput | 10K/sec | 12K/sec |
| Event Throughput | 1M/sec | 1.2M/sec |
| Reservation Success Rate | >99.5% | 99.8% |
| System Availability | 99.99% | 99.99% |

## Support

- **24/7 Operations Center**: +1-555-INVENTORY
- **Escalation**: inventory-oncall@freshmart.com
- **Slack Channel**: #inventory-platform
- **Wiki**: https://wiki.freshmart.com/inventory-platform
- **Grafana Dashboard**: https://grafana.freshmart.com/d/inventory
