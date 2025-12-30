# Mobile Experience Platform Documentation

## System Overview

The FreshMart Mobile Experience Platform delivers a unified mobile shopping experience across iOS, Android, and web browsers. Built with a hybrid React Native and Progressive Web App (PWA) architecture, the platform serves over 10 million active users with full offline capabilities.

## Architecture Documentation

- [Architecture Overview](architecture.md) - System design and component structure
- [API Documentation](api.md) - Mobile BFF GraphQL specifications
- [Offline Guide](offline.md) - Offline-first implementation details
- [Performance Guide](performance.md) - Optimization strategies

## Key Capabilities

### Offline-First Architecture
- Bidirectional sync with automatic conflict resolution
- WatermelonDB local storage on native apps (React Native)
- IndexedDB with Dexie.js on PWA for browser persistence
- Background sync manager with priority queue processing
- 95% of core features available without network connectivity

### Push Notifications
- Firebase Cloud Messaging for Android devices
- Apple Push Notification Service (APNs) for iOS
- Socket.io real-time WebSocket connections for in-app updates
- Personalized notification targeting via Loyalty Platform integration
- Event-driven notification broadcasting through Apache Kafka

### Store Locator
- Integration with external Maps Service for geolocation
- Real-time store inventory availability display
- Store hours, directions, and contact information
- Nearby store recommendations based on user location

### Digital Receipts
- Complete purchase history accessible via Account Screen
- Receipt search and filtering capabilities
- Email and PDF export functionality
- Integration with Loyalty Platform for rewards tracking

### Mobile Backend-for-Frontend (BFF)
- Apollo GraphQL Gateway for unified API layer
- Data aggregation from multiple backend services (Loyalty, Inventory, Payment)
- Response optimization for mobile bandwidth constraints
- Redis-backed caching with intelligent cache invalidation
- Express middleware rate limiting for API protection

## Integration Guide

### Cart Operations
```graphql
mutation AddToCart($input: AddToCartInput!) {
  addToCart(input: $input) {
    cart {
      id
      items {
        productId
        name
        quantity
        price
      }
      subtotal
      loyaltyPointsEarned
    }
    syncStatus
  }
}

# Variables
{
  "input": {
    "productId": "prod_12345",
    "quantity": 2,
    "storeId": "store_001"
  }
}
```

### Offline Sync Operations
```javascript
// React Native - Sync Manager Integration
import { SyncManager } from '@freshmart/offline-engine';

const syncManager = new SyncManager({
  localDB: watermelonDB,
  conflictStrategy: 'server-wins-with-merge',
  retryPolicy: {
    maxAttempts: 5,
    backoffMs: [1000, 2000, 4000, 8000, 16000]
  }
});

// Trigger manual sync
await syncManager.syncCart();

// Listen for sync events
syncManager.on('syncComplete', (result) => {
  console.log(`Synced ${result.itemsProcessed} items`);
});

syncManager.on('conflictResolved', (conflict) => {
  console.log(`Resolved conflict for ${conflict.entityId}`);
});
```

### User Profile Operations
```graphql
query GetUserProfile {
  user {
    id
    firstName
    lastName
    email
    loyaltyStatus {
      tier
      points
      lifetimePoints
      nextTierProgress
    }
    preferences {
      pushNotifications
      emailMarketing
      preferredStore
    }
    recentOrders(limit: 5) {
      id
      date
      total
      status
    }
  }
}
```

### PWA Service Worker Registration
```javascript
// Service Worker with Workbox
import { precacheAndRoute } from 'workbox-precaching';
import { registerRoute } from 'workbox-routing';
import { NetworkFirst, CacheFirst, StaleWhileRevalidate } from 'workbox-strategies';

// Precache app shell
precacheAndRoute(self.__WB_MANIFEST);

// API requests - network first with cache fallback
registerRoute(
  ({ url }) => url.pathname.startsWith('/api/'),
  new NetworkFirst({
    cacheName: 'api-cache',
    networkTimeoutSeconds: 10
  })
);

// Static assets - cache first
registerRoute(
  ({ request }) => request.destination === 'image',
  new CacheFirst({
    cacheName: 'image-cache',
    expiration: { maxEntries: 500, maxAgeSeconds: 7 * 24 * 60 * 60 }
  })
);
```

### Event Streaming
Subscribe to mobile events via Kafka:
- `mobile.cart.updated` - Cart modification events
- `mobile.sync.completed` - Offline sync completion
- `mobile.user.session` - User session lifecycle events
- `mobile.push.delivered` - Push notification delivery status
- `mobile.search.performed` - Product search analytics

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Monthly Active Users | 10M | 10.2M |
| Crash-Free Session Rate | >99.9% | 99.92% |
| Offline Sync Success Rate | >99% | 99.4% |
| App Cold Start Time | <2s | 1.8s |
| PWA Lighthouse Score | >90 | 95 |
| API Response Time (p95) | <300ms | 245ms |
| Push Notification Delivery Rate | >98% | 98.7% |
| Digital Revenue Contribution | 30% | 31.2% |

## Technology Stack

| Component | Technology |
|-----------|------------|
| Native App | React Native |
| State Management | Redux + RTK Query |
| Offline Storage (Native) | WatermelonDB |
| PWA Framework | React |
| PWA Offline | Workbox + Dexie.js |
| Mobile BFF | Node.js / Express |
| API Layer | Apollo GraphQL Server |
| Caching | Redis |
| Real-time | Socket.io |
| Content Search | Elasticsearch |
| Event Bus | Apache Kafka |
| CDN | AWS CloudFront |
| Feature Flags | LaunchDarkly |

## Support

- **Mobile Team Lead**: mobile-team@freshmart.com
- **On-Call Support**: +1-555-MOBILE-1
- **Slack Channel**: #mobile-experience
- **Engineering Wiki**: https://wiki.freshmart.com/mobile-experience
- **App Store Support**: appstore-support@freshmart.com
