# Analytics & AI Platform Documentation

## System Overview

The FreshMart Analytics & AI Platform is the unified data and machine learning platform that powers retail intelligence across all FreshMart operations. Built on a Lambda architecture, it processes 5 petabytes of data and serves 100+ million predictions daily to optimize inventory, pricing, fraud detection, and customer experience.

## Architecture Documentation

- [Architecture Overview](architecture.md) - Detailed Lambda architecture design
- [API Documentation](api.md) - Analytics and ML inference APIs
- [Data Governance Guide](governance.md) - Data lineage and compliance
- [ML Operations Guide](mlops.md) - Model lifecycle management

## Key Capabilities

### Lambda Architecture (Batch + Stream Processing)
- Apache Spark cluster with 100 nodes processing 100TB/day in batch
- Apache Flink streaming engine with sub-100ms latency
- Unified serving layer via Apache Druid for real-time OLAP queries
- Exactly-once processing semantics with RocksDB state backend

### Feature Store
- Feast-based centralized feature management with 5,000+ features
- Online serving via Redis with <10ms latency
- Offline feature store on S3 for model training
- Git-based feature versioning with full lineage tracking

### Fraud Detection
- XGBoost + Isolation Forest ensemble model
- 95% precision, 92% recall on fraudulent transactions
- Real-time scoring in <50ms for all POS transactions
- Continuous model retraining with weekly updates

### Model Serving
- MLflow model registry with staging/production/archived workflows
- KFServing for auto-scaling REST and gRPC inference
- Support for TensorFlow, PyTorch, and XGBoost frameworks
- A/B testing and shadow deployment patterns

### Data Governance
- Apache Atlas for end-to-end data lineage tracking
- Apache Ranger for fine-grained access control with 500+ policies
- Automatic PII detection and classification
- GDPR and CCPA compliance with 7-year audit retention

## Integration Guide

### Analytics Query API
```http
POST /api/v1/analytics/query
Authorization: Bearer {token}
Content-Type: application/json

{
  "datasource": "retail_transactions",
  "granularity": "hour",
  "intervals": ["2024-01-01/2024-01-31"],
  "aggregations": [
    {"type": "sum", "fieldName": "revenue", "name": "total_revenue"},
    {"type": "count", "name": "transaction_count"}
  ],
  "dimensions": ["store_id", "product_category"],
  "filter": {
    "type": "selector",
    "dimension": "region",
    "value": "west"
  }
}
```

### ML Inference API
```http
POST /api/v1/ml/predict
Authorization: Bearer {token}
Content-Type: application/json

{
  "model_name": "demand_forecast",
  "model_version": "v2.3.1",
  "inputs": {
    "sku_id": "SKU_12345",
    "store_id": "STORE_001",
    "forecast_horizon_days": 14,
    "include_confidence_intervals": true
  }
}
```

### Feature Retrieval API
```python
from feast import FeatureStore

store = FeatureStore(repo_path="feature_repo")

# Get online features for real-time inference
features = store.get_online_features(
    features=[
        "customer_features:lifetime_value",
        "customer_features:purchase_frequency",
        "customer_features:avg_basket_size",
        "product_features:category_affinity_score"
    ],
    entity_rows=[
        {"customer_id": "CUST_12345", "product_id": "PROD_67890"}
    ]
).to_dict()
```

### Event Streaming
Subscribe to analytics events via Kafka:
- `analytics.aggregates.updated` - Real-time aggregate metrics updated
- `ml.prediction.completed` - Model inference completed
- `ml.model.deployed` - New model version deployed to production
- `anomaly.detected` - Anomaly detected in streaming data
- `feature.updated` - Feature store values updated

## ML Models

| Model | Algorithm | Accuracy | Latency | Update Frequency |
|-------|-----------|----------|---------|------------------|
| Demand Forecasting | Prophet + LSTM | 92% | 200ms | Daily |
| Pricing Optimization | Deep Q-Network | Revenue +8% | 150ms | Real-time |
| Customer Segmentation | K-Means + DBSCAN | 12 segments | Batch | Weekly |
| Fraud Detection | XGBoost + Isolation Forest | 95% precision | <50ms | Weekly |
| Recommendation Engine | ALS + Neural CF + BERT4Rec | 95% catalog coverage | <100ms | Daily |

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Streaming Throughput | 1M events/sec | 1.2M events/sec |
| Batch Processing Capacity | 100TB/day | 120TB/day |
| Query Latency (p99) | <1s | 850ms |
| Feature Serving Latency | <10ms | 8ms |
| Model Inference Latency | <100ms | 75ms |
| Data Freshness | 1 minute | 45 seconds |
| System Availability | 99.9% | 99.95% |
| Daily Predictions Served | 100M | 125M |

## Data Volume

| Data Source | Daily Volume | Format | Ingestion Method |
|-------------|--------------|--------|------------------|
| POS Transactions | 5M events | JSON | Kafka Streaming |
| Inventory Events | 10M events | Avro | CDC (Debezium) |
| Customer Profiles | 1M records | JSON | Batch (S3) |
| Supply Chain | 100K shipments | XML/EDI | SFTP Batch |
| Weather Data | 10K API calls | JSON | REST API |
| Social Sentiment | 100K mentions | JSON | Streaming API |

## Support

- **Data Platform Team**: analytics-platform@freshmart.com
- **ML Engineering**: ml-engineering@freshmart.com
- **24/7 On-Call**: +1-555-ANALYTICS
- **Slack Channels**:
  - #analytics-platform - General platform questions
  - #ml-models - ML model issues and requests
  - #data-quality - Data quality alerts and issues
- **Wiki**: https://wiki.freshmart.com/analytics-ai
- **Model Registry**: https://mlflow.freshmart.com
- **Feature Store UI**: https://feast.freshmart.com
