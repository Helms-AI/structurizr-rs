# ADR-003: Membership Tier Calculation and Progression Strategy

## Status
Accepted

## Context
FreshMart's loyalty program uses a tiered membership structure to reward customer loyalty. We need to design a tier management system that:
- Calculates tier qualification based on member activity
- Manages tier-specific benefits and multipliers
- Handles anniversary date tracking and tier evaluation cycles
- Provides tier protection to prevent frustrating downgrades
- Scales to 25M members with real-time tier updates

Key business constraints:
- Tiers must feel attainable but meaningful
- Downgrade protection is critical for member satisfaction
- Benefits must be clearly differentiated to drive behavior
- System must support promotional tier accelerators

## Decision
We will implement a **Tier Service** with the following calculation strategy:

### 1. Tier Structure

| Tier | Qualifying Spend (12mo) | Points Multiplier | Key Benefits |
|------|------------------------|-------------------|--------------|
| Bronze | $0 - $499 | 1.0x | Base earning, Member pricing |
| Silver | $500 - $1,499 | 1.25x | Free delivery, Early access |
| Gold | $1,500 - $4,999 | 1.5x | Birthday bonus, Priority support |
| Platinum | $5,000+ | 2.0x | Concierge service, Exclusive events |

### 2. Tier Calculation Logic
The Tier Calculator will evaluate qualification using:

**Primary Metric:** Rolling 12-month qualifying spend
- Includes all retail purchases (in-store + online)
- Excludes: gift cards, fees, taxes, refunded items
- Promotional spend counts at face value

**Calculation Trigger:**
- Real-time check after each transaction
- Upgrade: Immediate upon threshold crossing
- Downgrade: Evaluated on anniversary date only

### 3. Anniversary Date Handling
Each member has an anniversary date (program enrollment date):

```
Anniversary Logic:
1. On anniversary: Recalculate tier based on trailing 12-month spend
2. If new tier < current tier: Apply tier protection rules
3. If new tier >= current tier: Confirm/upgrade tier
4. Reset qualifying spend counter for next period
```

### 4. Tier Protection Rules
To prevent member frustration from downgrades:

| Protection Type | Rule |
|-----------------|------|
| Soft Landing | Drop max 1 tier per year |
| Grace Period | 90-day warning before downgrade |
| Spend Deficit Alert | Notify at 30/60/90 days if at risk |
| Rescue Offer | Targeted bonus points to close gap |

**Example:** Platinum member with only $3,000 spend:
- Would qualify for Gold normally
- Soft landing: Drops to Gold (not Silver)
- Receives 90-day notification before anniversary
- Gets rescue offer: "Spend $500 more to keep Platinum"

### 5. Benefits Manager Integration
The Benefits Manager will:
- Maintain tier-benefit mapping configuration
- Apply benefits immediately upon upgrade
- Manage benefit transition during downgrades
- Support promotional tier status (e.g., "Gold for 90 days" offers)

## Consequences

### Positive
- Clear progression path drives 25% increase in member spend
- Tier protection reduces member churn by 15%
- Real-time upgrades create instant gratification moments
- Anniversary model aligns with member mental model
- Rescue offers recover 40% of at-risk downgrades

### Negative
- Anniversary-based evaluation creates uneven processing load
- Tier protection complexity increases support inquiries
- Promotional tier status adds system complexity
- Historical spend tracking requires significant storage

### Mitigation
- Distribute anniversary processing across daily batches
- Create comprehensive tier FAQ and chatbot support
- Implement clear promotional tier expiration messaging
- Archive spend details after 24-month retention period
- Pre-compute tier risk scores for proactive outreach

## Implementation

1. **Phase 1 - Core Tier Logic (Month 1)**
   - Deploy Tier Service (Java/Spring Boot)
   - Implement Tier Calculator with threshold logic
   - Set up spend tracking aggregation

2. **Phase 2 - Benefits Integration (Month 2)**
   - Deploy Benefits Manager component
   - Integrate with Points Engine for multipliers
   - Connect to offer targeting for tier-specific deals

3. **Phase 3 - Protection & Communication (Month 3)**
   - Implement soft landing logic
   - Deploy anniversary batch processor
   - Create downgrade warning notification flows
   - Build rescue offer automation

4. **Phase 4 - Monitoring & Optimization (Month 4)**
   - Deploy tier analytics dashboard
   - Implement tier movement forecasting
   - A/B test threshold adjustments
   - Create tier health metrics

## Tier Distribution Targets

| Tier | Target % | Current % |
|------|----------|-----------|
| Bronze | 50% | 48% |
| Silver | 30% | 32% |
| Gold | 15% | 15% |
| Platinum | 5% | 5% |

## References
- [Loyalty Program Business Requirements](https://wiki.freshmart.com/loyalty-requirements)
- [Tier Benefits Configuration](https://wiki.freshmart.com/tier-benefits)
- [Member Communication Templates](https://wiki.freshmart.com/member-comms)
- [Tier Analytics Dashboard](https://wiki.freshmart.com/tier-analytics)
