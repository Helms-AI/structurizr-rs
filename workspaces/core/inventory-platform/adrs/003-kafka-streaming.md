# ADR-003: Apache Kafka for Inventory Event Streaming

## Status
Accepted

## Context
The Inventory Platform processes 50M+ events daily from multiple source systems (POS, E-Commerce, WMS, Supply Chain). We need a streaming infrastructure that provides:
- High throughput (1M+ events/second peak capacity)
- Low latency (<100ms end-to-end processing)
- Durability (no event loss during system failures)
- Multiple consumer support (projections, analytics, alerting)
- Event replay capability for projection rebuilds
- Ordering guarantees for inventory consistency

We evaluated Apache Kafka, AWS Kinesis, Google Pub/Sub, and RabbitMQ Streams.

## Decision
We will use Apache Kafka as the event streaming backbone for the Inventory Platform:

### Topic Design
1. **Source Topics** (ingestion from external systems):
   - `pos.transactions` - POS sale, return, void events
   - `ecom.orders` - E-commerce order lifecycle events
   - `wms.movements` - Warehouse operational events
   - `supply.receipts` - Supply chain receipt and shipment events

2. **Domain Topics** (internal inventory events):
   - `inventory.commands` - Incoming commands (adjust, reserve, transfer)
   - `inventory.events` - Emitted domain events from aggregates
   - `inventory.reservations` - Reservation lifecycle events
   - `inventory.alerts` - Stock threshold and variance alerts

3. **Projection Topics** (derived data):
   - `inventory.stock-updates` - Stock level change notifications
   - `inventory.atp-updates` - ATP change notifications
   - `inventory.analytics` - Aggregated data for BI systems

### Partitioning Strategy
- **Key**: `{sku}-{location}` composite key ensures all events for same SKU-location go to same partition
- **Partition Count**: 100 partitions for high-volume topics, 25-50 for lower-volume
- **Benefits**: Maintains ordering per SKU-location; enables parallel processing across partitions

### Consumer Group Management
| Consumer Group | Topics | Purpose | Instances |
|----------------|--------|---------|-----------|
| flink-processor | All source topics | Projection updates | 20 |
| analytics-consumer | All topics | Business intelligence | 5 |
| alert-processor | inventory.events | Real-time alerting | 10 |
| fulfillment-consumer | ecom.orders | Order fulfillment | 10 |
| audit-archiver | All topics | Long-term archival | 3 |

### Retention and Compaction
- **Source Topics**: 7-day retention for replay and debugging
- **Domain Topics**: 7-day retention; snapshots in EventStoreDB for longer
- **Projection Topics**: Log compaction enabled; keep latest per key
- **Analytics Topics**: 30-day retention for trend analysis

## Consequences

### Positive
- **High Throughput**: 1M+ msg/sec capacity with current cluster configuration
- **Durability**: Replication factor 3 ensures no data loss on node failures
- **Ordering**: Per-partition ordering guarantees event sequence for each SKU-location
- **Replay**: 7-day retention enables projection rebuilds without hitting EventStoreDB
- **Ecosystem**: Rich integration with Flink, Spark, and monitoring tools
- **Multi-Consumer**: Multiple consumer groups process same events independently
- **Exactly-Once**: Kafka transactions with Flink provide exactly-once semantics

### Negative
- **Operational Complexity**: Requires dedicated team for cluster management
- **Cost**: High-performance cluster requires significant infrastructure investment
- **Partition Rebalancing**: Consumer group changes trigger rebalancing with temporary processing pause
- **Ordering Limitations**: Ordering only guaranteed within partition, not across topics
- **Retention Trade-offs**: Longer retention increases storage costs

### Mitigation
- **Operations**: Use managed Kafka (Confluent Cloud) for critical workloads; dedicated SRE team
- **Cost**: Right-size partitions; use tiered storage for older data
- **Rebalancing**: Use cooperative rebalancing; design consumers for graceful restart
- **Ordering**: Use composite keys; implement idempotent consumers
- **Retention**: Archive to S3 after retention period; use EventStoreDB as source of truth

## Implementation

### Cluster Configuration
```yaml
Brokers: 12 (4 per AZ across 3 AZs)
Storage: 10TB per broker (NVMe SSD)
Replication Factor: 3
Min ISR: 2
Compression: LZ4
Max Message Size: 1MB
```

### Producer Configuration
```java
Properties props = new Properties();
props.put("bootstrap.servers", "kafka.freshmart.internal:9092");
props.put("acks", "all");  // Wait for all replicas
props.put("retries", 3);
props.put("enable.idempotence", true);
props.put("compression.type", "lz4");
props.put("linger.ms", 5);  // Batch for throughput
props.put("batch.size", 65536);
```

### Consumer Configuration (Flink)
```java
FlinkKafkaConsumer<InventoryEvent> consumer = new FlinkKafkaConsumer<>(
    Arrays.asList("pos.transactions", "ecom.orders", "wms.movements"),
    new InventoryEventDeserializer(),
    kafkaProps
);
consumer.setStartFromLatest();
consumer.setCommitOffsetsOnCheckpoints(true);
```

### Monitoring and Alerting
- **Lag Monitoring**: Alert when consumer lag exceeds 10,000 messages
- **Throughput Tracking**: Dashboard showing events/second by topic
- **Under-Replicated Partitions**: Alert immediately on replication issues
- **Consumer Group Health**: Monitor for stuck or crashed consumers

### Dead Letter Queue (DLQ) Strategy
Events that fail processing after 3 retries are routed to DLQ topics:
- `inventory.dlq.commands` - Failed command processing
- `inventory.dlq.events` - Failed event processing

DLQ events are reviewed manually and either:
1. Fixed and replayed
2. Logged and discarded with audit trail

## References
- [Kafka Operations Guide](https://wiki.freshmart.com/kafka-operations)
- [Event Schema Registry](https://wiki.freshmart.com/schema-registry)
- [Consumer Best Practices](https://wiki.freshmart.com/kafka-consumers)
- [Flink-Kafka Integration](https://wiki.freshmart.com/flink-kafka)
