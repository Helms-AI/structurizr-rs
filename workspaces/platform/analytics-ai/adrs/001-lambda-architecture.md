# ADR-001: Lambda Architecture for Unified Batch and Stream Processing

## Status
Accepted

## Context
FreshMart generates massive volumes of data from diverse sources: 5 million POS transactions daily, 10 million inventory events, customer interactions, supply chain logistics, and external data feeds. We need an architecture that:
- Processes real-time streaming data with sub-second latency for fraud detection and pricing
- Handles large-scale batch analytics for demand forecasting and reporting
- Provides a unified query interface for both real-time and historical data
- Scales to handle 100TB+ daily data processing
- Ensures exactly-once processing semantics for financial accuracy

## Decision
We will implement a Lambda architecture with the following components:

### Speed Layer (Real-time Processing)
- **Apache Flink** for stream processing with exactly-once semantics
- RocksDB state backend for fault-tolerant stateful processing
- Sub-100ms end-to-end latency for critical paths
- Complex Event Processing (CEP) for pattern detection and alerting

### Batch Layer (Historical Processing)
- **Apache Spark** cluster with 100 nodes and 10TB distributed memory
- Spark SQL for ETL pipelines with Great Expectations for data quality
- Hourly feature engineering refreshes with Spark ML
- Daily model retraining pipelines via Kubeflow

### Serving Layer (Unified Query Interface)
- **Apache Druid** for real-time OLAP with 1-minute data freshness
- Redis cluster for low-latency feature serving (<10ms)
- Snowflake data warehouse for complex analytical queries
- Materialized views combining batch and real-time data

### Data Flow
1. All data ingested through Apache NiFi gateway
2. Events published to Apache Kafka (1M events/sec throughput)
3. Flink consumes from Kafka for real-time processing
4. Raw data lands in S3 data lake (5PB storage)
5. Spark processes data lake for batch transformations
6. Both layers write to serving layer for unified access

## Consequences

### Positive
- **Unified analytics**: Business users see consistent metrics regardless of query time
- **Flexibility**: Real-time for urgent decisions, batch for cost efficiency
- **Fault tolerance**: Batch layer corrects any streaming errors
- **Scalability**: Each layer scales independently based on workload
- **Data freshness**: 1-minute freshness for dashboards, sub-second for fraud

### Negative
- **Complexity**: Two separate codebases for batch and stream logic
- **Eventual consistency**: Batch corrections may take hours to propagate
- **Operational overhead**: Three processing engines to maintain
- **Cost**: Running parallel processing infrastructure

### Mitigation
- **Apache Beam** SDK for write-once, run-anywhere pipelines (future migration)
- Automated reconciliation jobs to detect batch/stream divergence
- Unified monitoring with Prometheus + Grafana for all layers
- Infrastructure-as-code with Terraform for consistent deployments

## Implementation
1. Deploy Kafka cluster with 10 brokers (1M msg/sec capacity)
2. Set up Flink cluster with exactly-once checkpointing
3. Configure Spark cluster on Kubernetes with auto-scaling
4. Implement Druid cluster with tiered storage
5. Deploy Airflow DAGs for 500+ batch job orchestration
6. Create unified API gateway for serving layer queries

## Performance Targets
| Component | Metric | Target |
|-----------|--------|--------|
| Kafka | Throughput | 1M events/sec |
| Flink | Latency | <100ms |
| Spark | Daily Processing | 100TB |
| Druid | Query Latency | <1s |
| Redis | Feature Serving | <10ms |

## References
- [Lambda Architecture Overview](https://wiki.freshmart.com/lambda-architecture)
- [Flink Deployment Guide](https://wiki.freshmart.com/flink-deployment)
- [Spark Cluster Configuration](https://wiki.freshmart.com/spark-config)
- [Druid Optimization Guide](https://wiki.freshmart.com/druid-tuning)
