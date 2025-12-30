# ADR-002: Centralized Feature Store for ML Feature Management

## Status
Accepted

## Context
FreshMart's 200+ ML models require consistent access to 5,000+ features computed from diverse data sources. Current challenges include:
- Feature duplication across teams (Data Science, ML Engineering, Analytics)
- Training-serving skew causing model performance degradation
- No visibility into feature lineage or data quality issues
- Inconsistent feature computation between batch training and online inference
- Long lead time (weeks) to add new features to production models

## Decision
We will implement a centralized Feature Store based on **Feast** with the following architecture:

### Offline Store (Training)
- **AWS S3** with Parquet format for historical feature storage
- Point-in-time correct feature retrieval for training datasets
- Integration with Spark for large-scale feature computation
- Support for time-travel queries to reproduce historical model behavior

### Online Store (Inference)
- **Redis Cluster** (1TB memory) for low-latency feature serving
- Sub-10ms p99 latency for real-time inference
- Automatic synchronization from offline to online store
- TTL-based feature expiration for freshness guarantees

### Feature Registry
- Git-based versioning for feature definitions
- Centralized metadata catalog with search and discovery
- Full lineage tracking from source data to feature values
- Data quality metrics and monitoring per feature

### Feature Computation
- Spark jobs for batch feature engineering (hourly refresh)
- Flink streaming for real-time feature updates
- Declarative feature definitions in Python DSL
- Automatic schema validation and type checking

## Feature Categories
| Category | Count | Refresh Rate | Example |
|----------|-------|--------------|---------|
| Customer | 1,200 | Hourly | lifetime_value, churn_probability |
| Product | 800 | Daily | sales_velocity, price_elasticity |
| Store | 500 | Hourly | foot_traffic, conversion_rate |
| Transaction | 1,500 | Real-time | basket_size, payment_method_pref |
| External | 1,000 | Varies | weather_impact, competitor_price |

## Consequences

### Positive
- **Consistency**: Same feature values for training and serving eliminates skew
- **Reusability**: Teams share features instead of duplicating computation
- **Velocity**: New features available in production within hours, not weeks
- **Governance**: Full visibility into feature usage, lineage, and data quality
- **Cost reduction**: 40% reduction in feature computation through sharing

### Negative
- **Single point of failure**: All models depend on Feature Store availability
- **Migration effort**: Existing models need refactoring to use Feature Store
- **Learning curve**: Teams must learn Feast SDK and feature engineering patterns
- **Storage costs**: Maintaining historical feature values increases storage

### Mitigation
- Redis cluster with 99.99% SLA and automatic failover
- Gradual migration with feature-by-feature onboarding
- Comprehensive documentation and training program
- Tiered storage with S3 lifecycle policies for cost optimization

## Implementation
1. Deploy Feast infrastructure on Kubernetes
2. Configure Redis cluster with 1TB memory across 6 nodes
3. Set up S3 buckets with partitioned Parquet storage
4. Implement Spark jobs for batch feature computation
5. Create Python SDK wrapper for simplified feature access
6. Build feature discovery UI integrated with data catalog

## Feature Definition Example
```python
from feast import Entity, Feature, FeatureView, FileSource, ValueType
from datetime import timedelta

customer = Entity(
    name="customer_id",
    value_type=ValueType.STRING,
    description="Unique customer identifier"
)

customer_features = FeatureView(
    name="customer_features",
    entities=["customer_id"],
    ttl=timedelta(hours=1),
    features=[
        Feature(name="lifetime_value", dtype=ValueType.FLOAT),
        Feature(name="purchase_frequency", dtype=ValueType.FLOAT),
        Feature(name="avg_basket_size", dtype=ValueType.FLOAT),
        Feature(name="days_since_last_purchase", dtype=ValueType.INT32),
        Feature(name="preferred_category", dtype=ValueType.STRING),
        Feature(name="churn_probability", dtype=ValueType.FLOAT),
    ],
    batch_source=FileSource(
        path="s3://freshmart-features/customer/",
        event_timestamp_column="event_timestamp",
    ),
    online=True,
    tags={"team": "customer-analytics", "pii": "false"}
)
```

## Performance Targets
| Metric | Target | Current |
|--------|--------|---------|
| Online Serving Latency (p99) | <10ms | 8ms |
| Offline Retrieval (1M rows) | <5min | 3.5min |
| Feature Freshness | 1 hour | 45 min |
| Feature Coverage | 100% models | 95% |
| Training-Serving Skew | <0.1% | 0.05% |

## References
- [Feast Documentation](https://docs.feast.dev)
- [Feature Store Best Practices](https://wiki.freshmart.com/feature-store-guide)
- [Feature Engineering Standards](https://wiki.freshmart.com/feature-standards)
- [Data Quality Monitoring](https://wiki.freshmart.com/data-quality)
