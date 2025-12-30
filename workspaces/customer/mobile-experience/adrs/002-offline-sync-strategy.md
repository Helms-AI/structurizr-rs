# ADR-002: Bidirectional Sync with Conflict Resolution

## Status
Accepted

## Context
FreshMart customers frequently shop in environments with unreliable network connectivity (inside stores with poor signal, during commutes, rural areas). The mobile app must:
- Allow customers to browse products, build carts, and view order history offline
- Sync changes reliably when connectivity is restored
- Handle conflicts when the same data is modified offline and on server
- Provide feedback on sync status without disrupting the shopping experience
- Support 95% of core shopping features in offline mode

## Decision
We will implement a bidirectional sync strategy with automatic conflict resolution using a combination of WatermelonDB (React Native) and IndexedDB/Dexie.js (PWA).

### Local Storage Architecture

**React Native (WatermelonDB):**
- SQLite-based reactive database optimized for React Native
- Lazy loading for large datasets (product catalog with 100K+ items)
- Observable queries for real-time UI updates
- Built-in sync primitives for push/pull operations

**PWA (IndexedDB via Dexie.js):**
- Browser-native storage with 50MB+ capacity
- Dexie.js provides Promise-based API and live queries
- Service worker integration for background sync
- Automatic schema migrations

### Conflict Resolution Strategy

We implement a "server-wins-with-merge" strategy:

```
Client State          Server State          Resolution
─────────────────────────────────────────────────────────
Cart item qty: 3   →  Cart item qty: 2   →  Server wins: 2
                      (admin adjustment)

Cart item qty: 3   →  Item removed        →  Server wins: removed
                      (out of stock)

New cart item      →  No conflict         →  Client wins: add item

Price: $10.99      →  Price: $12.99       →  Server wins: $12.99
(client cached)       (price update)          + notify user
```

**Conflict Resolution Rules:**
1. **Inventory-related conflicts**: Server always wins (stock, prices, availability)
2. **User preference conflicts**: Client wins (UI settings, favorites)
3. **Cart quantity conflicts**: Server wins with user notification
4. **Order conflicts**: Server is authoritative (cannot modify submitted orders offline)

### Sync Queue Prioritization

Changes are queued with priority levels:

| Priority | Type | Retry Policy | Example |
|----------|------|--------------|---------|
| Critical | Orders, Payments | Infinite retry with exponential backoff | Checkout submission |
| High | Cart changes | 10 retries, 1hr max | Add/remove items |
| Medium | User preferences | 5 retries, 30min max | Notification settings |
| Low | Analytics events | 3 retries, 5min max | Page views, searches |

### Retry Strategy

```javascript
const retryPolicy = {
  critical: {
    maxAttempts: Infinity,
    backoffMs: [1000, 2000, 4000, 8000, 16000, 30000, 60000],
    maxBackoffMs: 300000  // 5 minutes max
  },
  high: {
    maxAttempts: 10,
    backoffMs: [500, 1000, 2000, 4000, 8000],
    maxBackoffMs: 3600000  // 1 hour max
  },
  medium: {
    maxAttempts: 5,
    backoffMs: [1000, 2000, 4000],
    maxBackoffMs: 1800000  // 30 minutes max
  },
  low: {
    maxAttempts: 3,
    backoffMs: [1000, 2000],
    maxBackoffMs: 300000  // 5 minutes max
  }
};
```

### Sync Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        OFFLINE MODE                             │
├─────────────────────────────────────────────────────────────────┤
│  User Action → Redux Store → Local DB → Sync Queue (pending)   │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (network restored)
┌─────────────────────────────────────────────────────────────────┐
│                         SYNC PROCESS                            │
├─────────────────────────────────────────────────────────────────┤
│  1. Sync Manager detects connectivity                          │
│  2. Pull server changes (delta sync via lastSyncTimestamp)     │
│  3. Detect conflicts between local and server changes          │
│  4. Conflict Resolver applies resolution rules                  │
│  5. Push local changes to server                               │
│  6. Update local DB with server acknowledgments                │
│  7. Clear processed items from sync queue                      │
│  8. Emit syncComplete event to UI                              │
└─────────────────────────────────────────────────────────────────┘
```

## Consequences

### Positive
- Customers can shop uninterrupted regardless of network conditions
- Cart changes are never lost due to network failures
- Predictable conflict resolution maintains data integrity
- Priority-based sync ensures critical operations complete first
- 99.4% sync success rate achieved in production

### Negative
- Increased complexity in state management across online/offline modes
- Local storage limits may constrain catalog size (mitigated by lazy loading)
- Conflict notifications may confuse some users
- Sync operations consume battery and bandwidth

### Mitigation
- Clear UI indicators for offline mode and sync status
- User education through onboarding about offline capabilities
- Configurable sync frequency based on battery and network conditions
- Automatic cleanup of stale cached data (7-day expiry for product catalog)
- Background sync scheduled during charging and WiFi connectivity

## Implementation
1. Integrate WatermelonDB into React Native app with schema for cart, user, catalog
2. Configure Dexie.js in PWA with equivalent schema
3. Build Sync Manager as shared TypeScript module
4. Implement Conflict Resolver with configurable resolution strategies
5. Create sync queue with SQLite/IndexedDB persistence
6. Add network state monitoring with NetInfo (native) and navigator.onLine (web)
7. Build sync status UI components (spinner, badge, toast notifications)
8. Set up monitoring for sync success rates and conflict frequency

## References
- [WatermelonDB Documentation](https://nozbe.github.io/WatermelonDB/)
- [Dexie.js Documentation](https://dexie.org/docs/)
- [Offline-First Patterns](https://wiki.freshmart.com/offline-first)
- [Conflict Resolution Strategies](https://wiki.freshmart.com/sync-conflict-resolution)
