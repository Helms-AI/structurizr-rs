# ADR-001: Points Engine Design - Real-time Calculation Architecture

## Status
Accepted

## Context
FreshMart's loyalty program processes over 5 million transactions daily, each requiring points calculation. We need to determine the optimal architecture for the Points Engine to handle:
- Real-time points calculation at transaction time
- Complex earning rules including base points, tier multipliers, and promotional bonuses
- High-throughput ledger operations for 500M+ daily points movements
- Point expiration management across 25M member accounts

The key trade-off is between real-time processing (immediate member gratification) versus batch processing (simpler architecture, potential cost savings).

## Decision
We will implement a **real-time points calculation engine** with the following architecture:

### 1. Rule Engine Architecture
The Earn Engine will use a configurable rule engine supporting 500+ rules organized in three tiers:

1. **Base Rules**: Standard points per dollar spent (1 point = $1)
2. **Bonus Rules**: Tier-based multipliers (Silver 1.25x, Gold 1.5x, Platinum 2x)
3. **Promotional Rules**: Time-bound and category-specific bonuses

Rule evaluation order:
```
final_points = base_points * tier_multiplier * max(promo_multipliers)
```

### 2. Points Ledger Design
We will use ScyllaDB as the Points Ledger for its high-throughput write capabilities:

- **Partition Key**: `member_id` for member-centric queries
- **Clustering Key**: `transaction_timestamp` for time-ordered history
- **Write Pattern**: Append-only ledger for audit trail
- **Read Pattern**: Balance materialized view updated on each transaction

### 3. Balance Management
The Balance Manager will maintain:
- Current available balance (real-time)
- Pending points (promotional holds)
- Expiring points by date bucket
- Lifetime earned/redeemed totals

### 4. Expiration Engine
Points expiration will run as a scheduled batch process:
- Daily job identifies expiring points batches
- Grace period notifications sent 30/7/1 days before expiry
- Expired points moved to separate audit partition
- FIFO expiration policy (oldest points first)

## Consequences

### Positive
- Immediate points visibility increases member engagement by 23%
- Real-time tier upgrades create instant gratification moments
- Supports dynamic promotional campaigns with immediate effect
- Enables real-time gamification triggers (challenges, badges)
- Sub-second points calculation maintains checkout speed

### Negative
- Higher infrastructure costs for real-time processing (~$50K/month additional)
- More complex failure handling for distributed transactions
- Rule engine complexity requires dedicated configuration management
- Peak load handling requires auto-scaling capacity

### Mitigation
- Implement circuit breakers for downstream service failures
- Cache frequently accessed rules and tier configurations
- Use event sourcing pattern for eventual consistency recovery
- Deploy auto-scaling based on transaction volume metrics
- Maintain batch fallback mode for disaster recovery

## Implementation

1. **Phase 1 - Core Engine (Month 1-2)**
   - Deploy Points Engine service (Java/Spring Boot)
   - Implement base earning rules and Balance Manager
   - Set up ScyllaDB cluster with initial schema

2. **Phase 2 - Advanced Rules (Month 3)**
   - Implement tier multiplier integration
   - Add promotional bonus rule support
   - Deploy rule configuration admin interface

3. **Phase 3 - Expiration & Optimization (Month 4)**
   - Implement Expiration Engine scheduler
   - Add expiration notification workflows
   - Performance tuning for 10K TPS target

4. **Phase 4 - Monitoring & Analytics (Month 5)**
   - Deploy real-time dashboards
   - Implement anomaly detection for points fraud
   - Create A/B testing framework for rule optimization

## References
- [ScyllaDB Best Practices for Ledger Applications](https://wiki.freshmart.com/scylladb-ledger)
- [Points Rule Configuration Guide](https://wiki.freshmart.com/points-rules)
- [Loyalty Program Business Requirements](https://wiki.freshmart.com/loyalty-requirements)
