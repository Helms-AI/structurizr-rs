# ADR-002: Machine Learning Architecture for Offer Personalization

## Status
Accepted

## Context
FreshMart serves over 10,000 active offers to 25 million loyalty members. Manually targeting offers results in:
- Low redemption rates (12% average)
- Member fatigue from irrelevant offers
- Missed revenue opportunities from poor targeting
- Inability to scale personalization with offer catalog growth

We need an ML-driven approach to match the right offers to the right members at the right time, while maintaining sub-100ms response times for real-time personalization.

## Decision
We will implement an **AI-powered Personalization Engine** using Python/TensorFlow with the following model architecture:

### 1. Model Portfolio
Deploy 15+ specialized models organized by function:

| Model | Algorithm | Purpose | Update Frequency |
|-------|-----------|---------|------------------|
| Recommendation Model | Neural Collaborative Filtering | Product recommendations | Daily |
| Segmentation Model | K-Means Clustering | Customer grouping | Weekly |
| Propensity Model | XGBoost | Purchase likelihood | Daily |
| Churn Model | Random Forest | Churn risk prediction | Weekly |
| Offer Affinity | Matrix Factorization | Offer-member matching | Real-time |

### 2. Feature Engineering
The Feature Store (Redis) will maintain pre-computed features:

**Behavioral Features:**
- Transaction frequency (7d, 30d, 90d windows)
- Category affinity scores
- Price sensitivity index
- Channel preferences (store, mobile, web)

**Contextual Features:**
- Day of week / time of day patterns
- Seasonal purchase patterns
- Location-based preferences
- Recent browsing/search history

**Derived Features:**
- Lifetime value prediction
- Next purchase timing prediction
- Category expansion probability

### 3. Inference Architecture
Real-time inference pipeline:
```
Request -> Feature Lookup (Redis) -> Model Ensemble -> Ranking -> Response
              5ms                      40ms            20ms       <100ms total
```

### 4. A/B Testing Framework
The Campaign Service's A/B Test Engine will support:
- Multi-arm bandit for offer variant testing
- Holdout groups for baseline measurement
- Statistical significance calculation
- Automatic winner promotion

## Consequences

### Positive
- Offer redemption rate increased from 12% to 28%
- Member engagement scores improved by 35%
- Revenue per member increased by 18%
- Scalable personalization for unlimited offer catalog
- Data-driven insights for marketing strategy

### Negative
- ML infrastructure costs (~$80K/month for GPU instances)
- Model training requires dedicated MLOps team
- Cold start problem for new members (first 3-5 transactions)
- Model interpretability challenges for business stakeholders
- Risk of filter bubbles reducing offer diversity

### Mitigation
- Implement exploration/exploitation balance in recommendations
- Use rule-based fallbacks for new members
- Deploy SHAP values for model explainability
- Schedule diversity injection for offer variety
- Monitor for recommendation bias and fairness

## Implementation

1. **Phase 1 - Foundation (Month 1-2)**
   - Deploy Feature Store on Redis Cluster
   - Implement data pipelines for feature engineering
   - Set up ML training infrastructure (Kubernetes + GPU)

2. **Phase 2 - Core Models (Month 3-4)**
   - Train and deploy Segmentation Model
   - Implement Propensity Model for purchase prediction
   - Deploy basic recommendation model

3. **Phase 3 - Advanced Models (Month 5-6)**
   - Deploy Neural Collaborative Filtering model
   - Implement Churn prediction model
   - Add real-time offer affinity scoring

4. **Phase 4 - Optimization (Month 7-8)**
   - Deploy A/B testing framework
   - Implement multi-arm bandit optimization
   - Create model performance dashboards
   - Establish model retraining automation

## Model Performance Targets

| Model | Metric | Target | Current |
|-------|--------|--------|---------|
| Recommendation | Precision@10 | >0.25 | 0.28 |
| Segmentation | Silhouette Score | >0.45 | 0.52 |
| Propensity | AUC-ROC | >0.80 | 0.84 |
| Churn | F1-Score | >0.75 | 0.79 |

## References
- [Feature Store Architecture](https://wiki.freshmart.com/feature-store)
- [MLOps Pipeline Documentation](https://wiki.freshmart.com/mlops)
- [A/B Testing Statistical Methods](https://wiki.freshmart.com/ab-testing)
- [Model Fairness Guidelines](https://wiki.freshmart.com/ml-fairness)
