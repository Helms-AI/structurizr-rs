# ADR-003: Real-Time ML Model Serving Strategy

## Status
Accepted

## Context
FreshMart serves 100+ million predictions daily across critical business functions: fraud detection (<50ms SLA), product recommendations, demand forecasting, and dynamic pricing. We need a model serving infrastructure that:
- Supports multiple ML frameworks (TensorFlow, PyTorch, XGBoost, scikit-learn)
- Provides sub-100ms inference latency for real-time use cases
- Enables safe model deployment with rollback capabilities
- Supports A/B testing and canary deployments
- Scales automatically based on traffic patterns (10x variation)

## Decision
We will implement a comprehensive ML model serving platform using **KFServing** (now KServe) integrated with **MLflow** for model registry and lifecycle management.

### Model Registry (MLflow)
- Centralized model versioning with staging/production/archived workflows
- Artifact storage on S3 with model signatures and metadata
- Automated model validation before promotion
- Integration with experiment tracking for full lineage

### Model Serving (KFServing)
- Kubernetes-native serving with automatic scaling (HPA + VPA)
- Protocol support for REST and gRPC inference
- Built-in support for TensorFlow, PyTorch, XGBoost, scikit-learn
- Custom inference services for complex preprocessing pipelines

### Deployment Patterns
1. **Blue-Green Deployment**: Zero-downtime model updates
2. **Canary Deployment**: Gradual traffic shifting (1% -> 10% -> 50% -> 100%)
3. **Shadow Deployment**: Run new models in parallel without serving traffic
4. **A/B Testing**: Split traffic for model comparison with statistical significance

### Inference Pipeline
```
Request -> API Gateway -> Feature Enrichment -> Model Inference -> Post-processing -> Response
           (Kong)         (Feature Store)      (KFServing)       (Business Rules)
```

## Model Categories and SLAs
| Model | Latency SLA | Throughput | Deployment Pattern |
|-------|-------------|------------|-------------------|
| Fraud Detection | <50ms | 200 req/sec | Blue-Green |
| Recommendations | <100ms | 5000 req/sec | Canary |
| Demand Forecast | <200ms | 500 req/sec | Shadow -> Blue-Green |
| Pricing | <150ms | 1000 req/sec | A/B Testing |
| Customer Segmentation | Batch | N/A | Scheduled |

## Consequences

### Positive
- **Framework agnostic**: Support for all major ML frameworks
- **Safe deployments**: Multiple deployment patterns reduce production incidents
- **Auto-scaling**: Handle 10x traffic spikes without manual intervention
- **Observability**: Built-in metrics, logging, and tracing for all predictions
- **Cost efficiency**: Scale-to-zero for low-traffic models

### Negative
- **Kubernetes complexity**: Requires strong K8s expertise to operate
- **Cold start latency**: Scale-from-zero adds 30-60 seconds delay
- **Resource overhead**: Sidecar containers increase memory footprint
- **Learning curve**: Teams must learn KFServing abstractions

### Mitigation
- Dedicated ML Platform team for infrastructure operations
- Minimum replica count (1) for latency-sensitive models
- Resource optimization through model quantization and ONNX conversion
- Comprehensive training and documentation program

## Implementation
1. Deploy KFServing on dedicated Kubernetes namespace
2. Configure MLflow server with S3 artifact store
3. Implement CI/CD pipeline for model deployment
4. Set up Prometheus metrics and Grafana dashboards
5. Create model promotion workflows with approval gates
6. Build A/B testing framework with statistical analysis

## Model Deployment Workflow
```yaml
# Example KFServing InferenceService
apiVersion: serving.kserve.io/v1beta1
kind: InferenceService
metadata:
  name: fraud-detection
  namespace: ml-models
spec:
  predictor:
    model:
      modelFormat:
        name: xgboost
      storageUri: s3://freshmart-models/fraud-detection/v2.1.0
      resources:
        requests:
          cpu: "2"
          memory: "4Gi"
        limits:
          cpu: "4"
          memory: "8Gi"
    minReplicas: 2
    maxReplicas: 10
    scaleTarget: 50
    scaleMetric: concurrency
  transformer:
    containers:
      - name: feature-enricher
        image: freshmart/feature-enricher:v1.2
        resources:
          requests:
            cpu: "1"
            memory: "2Gi"
```

## A/B Testing Configuration
```python
from freshmart.ml.serving import ABTest

ab_test = ABTest(
    name="pricing_model_v3_test",
    models={
        "control": "pricing-optimization:v2.1.0",
        "treatment": "pricing-optimization:v3.0.0-beta"
    },
    traffic_split={"control": 0.9, "treatment": 0.1},
    metrics=["revenue_per_transaction", "conversion_rate"],
    min_sample_size=10000,
    confidence_level=0.95,
    duration_days=14
)

ab_test.start()
```

## Performance Targets
| Metric | Target | Current |
|--------|--------|---------|
| Inference Latency (p50) | <50ms | 35ms |
| Inference Latency (p99) | <100ms | 85ms |
| Model Deployment Time | <15min | 12min |
| Rollback Time | <5min | 3min |
| Scale-up Time | <2min | 90sec |
| Daily Predictions | 100M+ | 125M |
| Model Availability | 99.99% | 99.995% |

## Monitoring and Alerting
- **Latency alerts**: P99 > 100ms triggers PagerDuty
- **Error rate alerts**: >1% inference errors triggers investigation
- **Model drift detection**: Weekly statistical tests on prediction distributions
- **Feature drift detection**: Daily monitoring of feature value distributions
- **Business metric alerts**: Revenue/conversion anomalies trigger model review

## References
- [KFServing Documentation](https://kserve.github.io/website)
- [MLflow Model Registry Guide](https://mlflow.org/docs/latest/model-registry.html)
- [A/B Testing Best Practices](https://wiki.freshmart.com/ab-testing-guide)
- [Model Monitoring Standards](https://wiki.freshmart.com/model-monitoring)
