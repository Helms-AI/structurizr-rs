# ADR-001: Offline-First Architecture

## Status
Accepted

## Context
FreshMart operates 2,500+ retail stores with varying network connectivity quality. Many stores are in areas with unreliable internet connections, and even stores with good connectivity experience occasional outages. A POS system that depends on constant cloud connectivity would create unacceptable business risk:

- Checkout cannot stop during network outages
- Payment processing must continue even when the Payment Gateway is unreachable
- Inventory counts must remain accurate regardless of sync status
- Customer loyalty lookups should not block transactions

We need an architecture that treats offline operation as a first-class capability rather than an exception case.

## Decision
We will implement a queue-and-sync architecture where all POS operations are designed to work locally first, with cloud synchronization happening asynchronously when connectivity is available.

### Core Principles

1. **Local-First Data Storage**: All product catalog, pricing, and transaction data is stored locally in SQLite, with a RocksDB cache layer for high-frequency lookups.

2. **Store-and-Forward Payments**: When the Payment Gateway is unreachable, encrypted payment data is queued locally using the Store-and-Forward (SAF) mechanism, then processed when connectivity returns.

3. **CRDT-Based Conflict Resolution**: Sync conflicts are resolved using Conflict-free Replicated Data Types:
   - LWW-Register (Last-Write-Wins) for scalar values like prices
   - OR-Set (Observed-Remove Set) for collections like transaction items

4. **Prioritized Sync Queue**: The queue manager maintains priority lanes:
   - Critical: Payment authorizations, security events
   - High: Completed transactions, inventory updates
   - Normal: Analytics data, non-critical events
   - Low: Historical data, bulk updates

### Offline Transaction Flow

```
1. Cashier scans items
2. ItemProcessor queries LocalDatabase for product info
3. PricingCalculator uses CacheManager for cached prices
4. Transaction stored in LocalDatabase
5. Card payment encrypted and queued in QueueManager
6. Receipt printed from local data
7. SyncManager queues transaction for upload
```

### Sync Recovery Flow

```
1. StoreSync detects connectivity restored
2. SyncManager retrieves pending items from QueueManager
3. Payments forwarded to PaymentGateway
4. Inventory updates sent to InventoryPlatform
5. ConflictResolver handles any data conflicts
6. CacheManager refreshed with latest prices
7. Successfully synced items cleared from queue
```

## Consequences

### Positive
- Zero transaction failures due to network outages
- Consistent sub-30-second checkout times regardless of connectivity
- Store can operate for extended periods (days) without cloud access
- Reduced cloud infrastructure costs through local processing
- Improved perceived performance for cashiers

### Negative
- 10,000 transaction queue limit requires monitoring
- Payment authorization delay for offline transactions
- Potential for stale pricing data (mitigated by TTL)
- Increased local storage requirements (1GB RocksDB cache)
- Complexity in conflict resolution logic

### Mitigation
- Alerting when queue exceeds 5,000 transactions
- SAF transactions flagged for review if >24 hours old
- Aggressive price cache refresh (15-minute TTL for promotions)
- Automatic cache eviction using LRU policy
- Comprehensive integration tests for all conflict scenarios

## Implementation

1. **Local Database Schema**
   - Products table with offline-capable schema
   - Transactions table with sync_status column
   - Queue table with priority and retry_count

2. **Sync Manager Configuration**
   - WebSocket for real-time bidirectional sync
   - REST fallback for environments blocking WebSocket
   - LZ4 compression for bandwidth optimization
   - Exponential backoff for retry logic

3. **SAF Encryption**
   - AES-256-GCM encryption for queued payment data
   - RSA-2048 key exchange with Payment Gateway
   - Encrypted payload stored in LocalDatabase

4. **Monitoring**
   - Queue depth metrics exported to Analytics Platform
   - Sync latency histograms for performance tracking
   - Conflict rate monitoring for data integrity

## References
- [Offline Engine Components](../docs/index.md)
- [CRDT Implementation Guide](https://wiki.freshmart.com/crdt-patterns)
- [Store-and-Forward Specification](https://wiki.freshmart.com/saf-payments)
