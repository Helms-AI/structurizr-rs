# ADR-002: Prophet-Based Demand Forecasting with ML Pipeline

## Status
Accepted

## Context
Accurate demand forecasting is critical for FreshMart's supply chain efficiency. Poor forecasts lead to:
- Stockouts causing lost sales ($50M+ annual impact)
- Overstock resulting in waste, especially for perishables (8% shrink rate)
- Suboptimal purchase order quantities increasing logistics costs
- Reactive rather than proactive supplier negotiations

We need a forecasting system that:
- Predicts demand at SKU-Store-Day granularity (50,000 SKUs x 200 stores)
- Incorporates seasonality, promotions, weather, and external events
- Achieves 90%+ accuracy within a 12-week planning horizon
- Refreshes forecasts daily to capture real-time signals

## Decision
We will implement a multi-model ensemble forecasting system built on Facebook Prophet with supplementary LSTM and XGBoost models:

### 1. Core Forecasting Engine
- **Prophet**: Primary model for trend, seasonality, and holiday effects
- **LSTM**: Deep learning model for complex temporal patterns
- **XGBoost**: Gradient boosting for promotion lift and external factors
- **Ensemble**: Weighted average based on recent performance by category

### 2. Feature Engineering
- **Sales History**: 3 years of daily POS data by SKU-store
- **Seasonality**: Weekly, monthly, annual patterns via Fourier analysis
- **Promotions**: Planned promotional calendar with historical lift data
- **Weather**: 14-day forecasts integrated from Weather Intelligence API
- **Events**: Local events, holidays, competitor activities
- **Price Elasticity**: Historical price-demand relationships

### 3. Model Training Pipeline
```
Daily Pipeline:
1. Ingest POS data from Kafka (sales.transactions topic)
2. Feature extraction and transformation (Spark)
3. Model retraining for each category cluster (distributed)
4. Ensemble weight optimization based on recent accuracy
5. Forecast generation for 12-week horizon
6. Results published to Snowflake and inventory optimizer
```

### 4. Monitoring and Drift Detection
- Daily accuracy metrics by category and store cluster
- Automated alerts when accuracy drops below 85%
- A/B testing framework for model improvements
- Quarterly model review and retraining schedule
- Feature importance tracking for explainability

### 5. Promotion Lift Modeling (XGBoost)
- Separate model for promotional period forecasting
- Features: discount depth, promotion type, media support
- Historical lift database with 85% prediction accuracy
- Integration with merchandising planning calendar

## Consequences

### Positive
- Forecast accuracy improved from 78% to 92%
- Inventory reduction of 20% while maintaining 98% service level
- Perishable waste reduced by 35% through better ordering
- Procurement cost savings of $120M annually
- Proactive supplier collaboration based on forecasts

### Negative
- Significant infrastructure investment (Spark cluster, GPU instances)
- Model complexity requires dedicated ML engineering team
- 4-6 hour daily pipeline runtime for full forecast refresh
- Cold start problem for new products and stores

### Mitigation
- Implement tiered forecasting: simple models for low-volume SKUs
- Use category-level forecasts as priors for new product launches
- Build fallback to rule-based forecasting if ML pipeline fails
- Establish clear escalation path for forecast overrides

## Implementation
1. Deploy Python/Spark infrastructure on AWS EMR
2. Build feature store with historical data ingestion
3. Train baseline Prophet models by category
4. Add LSTM models for high-complexity categories
5. Implement ensemble framework with dynamic weighting
6. Create monitoring dashboards in Grafana
7. Integrate output with inventory optimization system

## References
- [Prophet Documentation](https://wiki.freshmart.com/prophet-forecasting)
- [Feature Store Architecture](https://wiki.freshmart.com/ml-feature-store)
- [Model Monitoring Guide](https://wiki.freshmart.com/ml-monitoring)
- [Forecast Accuracy Standards](https://wiki.freshmart.com/forecast-standards)
