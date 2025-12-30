# Loyalty & Rewards Platform Documentation

## System Overview

The FreshMart Loyalty & Rewards Platform is the central customer engagement system that powers FreshMart's loyalty program, serving over 25 million members. The platform manages points earning and redemption, personalized offers, tier-based benefits, and gamification features to drive customer retention and lifetime value.

## Architecture Documentation

- [Architecture Overview](architecture.md) - Detailed architectural design
- [API Documentation](api.md) - Loyalty API specifications
- [Personalization Guide](personalization.md) - ML models and targeting
- [Operations Runbook](runbook.md) - Operational procedures

## Key Capabilities

### Customer 360 Profile
- Unified customer view aggregating data from all touchpoints
- Privacy-compliant consent management with granular preferences
- Real-time preference tracking across 25M+ member profiles
- Behavioral data enrichment for personalization

### Points Engine
- Real-time points calculation for 5M+ daily transactions
- 500+ configurable earning rules including base, bonus, and promotional multipliers
- Multi-currency points ledger built on ScyllaDB for high-throughput operations
- Automated expiration engine with configurable policies

### AI-Powered Personalization
- 15+ machine learning models running inference in under 100ms
- Neural collaborative filtering for product recommendations
- Customer segmentation using K-Means clustering
- Churn prediction with Random Forest and purchase propensity with XGBoost

### Tier Management
- Four-tier membership structure: Bronze, Silver, Gold, Platinum
- Automatic tier calculation based on qualifying activity
- Tier-specific benefits management and rewards multipliers
- Anniversary date tracking with tier protection policies

### Campaign & Offer Management
- 10,000+ active personalized offers at any time
- Multi-channel campaign orchestration via Camunda workflow engine
- A/B testing framework for offer optimization
- Real-time audience targeting with ML-driven segmentation

### Gamification
- 100+ active challenges driving member engagement
- 50+ achievement badges for milestone recognition
- Real-time leaderboards powered by Redis
- Challenge completion tracking with bonus point awards

## Integration Guide

### Points Operations

**Earn Points from Transaction:**
```http
POST /api/v1/loyalty/points/earn
Authorization: Bearer {token}
Content-Type: application/json

{
  "member_id": "MBR_12345678",
  "transaction_id": "TXN_98765432",
  "amount": 125.99,
  "store_id": "STORE_001",
  "items": [
    {"sku": "SKU_001", "category": "grocery", "amount": 85.50},
    {"sku": "SKU_002", "category": "pharmacy", "amount": 40.49}
  ]
}
```

**Response:**
```json
{
  "points_earned": 252,
  "breakdown": {
    "base_points": 126,
    "tier_bonus": 63,
    "promo_bonus": 63
  },
  "new_balance": 15420,
  "tier": "Gold"
}
```

**Redeem Points:**
```http
POST /api/v1/loyalty/points/redeem
Authorization: Bearer {token}
Content-Type: application/json

{
  "member_id": "MBR_12345678",
  "points": 5000,
  "reward_type": "discount",
  "offer_id": "OFFER_20OFF"
}
```

### Personalized Offers

**Get Member Offers:**
```http
GET /api/v1/loyalty/offers?member_id=MBR_12345678&channel=mobile
Authorization: Bearer {token}
```

**Response:**
```json
{
  "offers": [
    {
      "offer_id": "OFFER_001",
      "title": "Double Points on Organic",
      "type": "bonus_multiplier",
      "multiplier": 2.0,
      "categories": ["organic"],
      "expires_at": "2024-12-31T23:59:59Z",
      "personalization_score": 0.92
    }
  ],
  "segment": "health_conscious",
  "propensity_score": 0.78
}
```

### Member Profile Operations

**Get Customer 360 View:**
```http
GET /api/v1/loyalty/members/{member_id}/360
Authorization: Bearer {token}
```

**Response:**
```json
{
  "member_id": "MBR_12345678",
  "tier": "Gold",
  "points_balance": 15420,
  "lifetime_value": 12500.00,
  "segments": ["health_conscious", "weekend_shopper"],
  "preferences": {
    "communication": ["push", "email"],
    "categories": ["organic", "produce"]
  },
  "recent_activity": {
    "last_transaction": "2024-01-15T14:30:00Z",
    "transactions_30d": 12,
    "points_earned_30d": 3200
  },
  "churn_risk": "low",
  "next_tier_progress": 0.65
}
```

### Event Streaming

Subscribe to loyalty events via Kafka:
- `transactions.completed` - POS transaction completed (consumed)
- `points.earned` - Points credited to member account
- `points.redeemed` - Points redeemed for reward
- `tier.upgraded` - Member tier upgraded
- `challenge.completed` - Gamification challenge completed
- `campaign.sent` - Marketing campaign delivered

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Active Members | 25M | 25.2M |
| Daily Transactions Processed | 5M | 5.1M |
| Points Calculated Daily | 500M | 512M |
| Personalized Offers Served Daily | 50M | 52M |
| Points Redemption Rate | 80% | 85% |
| Personalization Latency | <100ms | 87ms |
| System Availability | 99.99% | 99.99% |
| Member Revenue Contribution | 55% | 60% |

## Support

- **Loyalty Operations Team**: loyalty-ops@freshmart.com
- **Escalation**: loyalty-oncall@freshmart.com
- **Slack Channel**: #loyalty-platform
- **24/7 Support Line**: +1-555-REWARDS
- **Wiki**: https://wiki.freshmart.com/loyalty-platform
