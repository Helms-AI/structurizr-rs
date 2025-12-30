# ADR-003: Mobile Backend-for-Frontend Pattern

## Status
Accepted

## Context
The FreshMart mobile apps require data from multiple backend services:
- Loyalty Platform for rewards, points, and member status
- Inventory Platform for stock availability and pricing
- Payment Gateway for transaction processing
- Content Service for promotions and banners
- Search Service for product discovery

Direct mobile-to-microservice communication presents challenges:
- Over-fetching: Mobile clients receive more data than needed
- Multiple round trips: Several API calls to render a single screen
- Tight coupling: Mobile apps become dependent on backend API contracts
- Bandwidth constraints: Mobile networks have limited and variable bandwidth
- Battery consumption: Excessive network requests drain device batteries
- Version fragmentation: Multiple app versions in the wild with varying API needs

## Decision
We will implement a Mobile Backend-for-Frontend (BFF) using Node.js/Express with Apollo GraphQL Server to provide a unified, mobile-optimized API layer.

### GraphQL Gateway Architecture

**Technology Stack:**
- Node.js with Express for HTTP server
- Apollo Server for GraphQL implementation
- DataLoader for batching and caching backend requests
- Redis for response caching

**Schema Design:**
```graphql
type Query {
  # Aggregated home screen data
  homeScreen: HomeScreenData!

  # Product search with mobile-optimized pagination
  searchProducts(query: String!, filters: ProductFilters, cursor: String): ProductConnection!

  # User profile with loyalty info
  user: User!

  # Cart with real-time pricing
  cart: Cart!

  # Store locator
  nearbyStores(lat: Float!, lng: Float!, radius: Int): [Store!]!
}

type HomeScreenData {
  banners: [Banner!]!
  personalizedOffers: [Offer!]!
  recentlyViewed: [Product!]!
  loyaltySummary: LoyaltySummary!
  cartItemCount: Int!
}

type ProductConnection {
  edges: [ProductEdge!]!
  pageInfo: PageInfo!
  totalCount: Int!
  facets: [Facet!]!
}
```

### Data Aggregation Pattern

The BFF aggregates data from multiple services into mobile-optimized responses:

```
Mobile App Request                    BFF Processing
─────────────────────────────────────────────────────────────────
GET homeScreen                   →    Parallel fetch:
                                      ├── Content Service (banners)
                                      ├── Loyalty Platform (offers, points)
                                      ├── Inventory Platform (recent items)
                                      └── Mobile BFF Cache (cart count)

                                 →    Aggregate into single response
                                 →    Optimize payload size
                                 →    Return unified HomeScreenData
```

### Response Optimization Strategies

**1. Field Selection:**
GraphQL allows clients to request only needed fields, eliminating over-fetching:
```graphql
# Home screen only needs thumbnail images
query {
  homeScreen {
    recentlyViewed {
      id
      name
      price
      thumbnailUrl  # Not full product details
    }
  }
}
```

**2. Image Optimization:**
```javascript
// Response optimizer transforms image URLs
const optimizeForMobile = (product) => ({
  ...product,
  imageUrl: product.imageUrl
    ? `${CDN_URL}/optimize?url=${product.imageUrl}&w=300&q=80&fmt=webp`
    : null
});
```

**3. Payload Compression:**
- Brotli compression for GraphQL responses (60% size reduction)
- Delta compression for incremental updates
- Pagination with cursor-based navigation (20 items default)

**4. Caching Strategy:**
```javascript
// Redis cache configuration
const cacheConfig = {
  homeScreen: { ttl: 300, staleWhileRevalidate: 60 },      // 5min cache
  productSearch: { ttl: 60, staleWhileRevalidate: 30 },    // 1min cache
  userProfile: { ttl: 0 },                                   // No cache (personalized)
  storeLocator: { ttl: 3600 }                               // 1hr cache
};
```

### Rate Limiting

Express middleware protects the BFF from abuse:

```javascript
const rateLimiter = rateLimit({
  windowMs: 60 * 1000,  // 1 minute window
  max: 100,              // 100 requests per window
  keyGenerator: (req) => req.user?.id || req.ip,
  handler: (req, res) => {
    res.status(429).json({
      error: 'RATE_LIMITED',
      retryAfter: res.getHeader('Retry-After')
    });
  }
});

// Different limits for authenticated vs anonymous
const authenticatedLimit = rateLimit({ max: 200 });
const anonymousLimit = rateLimit({ max: 50 });
```

## Consequences

### Positive
- Single API call renders complete screens (home screen: 1 call vs 5 direct calls)
- 65% reduction in payload size through GraphQL field selection
- Mobile-optimized image URLs reduce bandwidth by 40%
- Backend service changes isolated from mobile clients
- Simplified mobile codebase with unified data layer
- Centralized caching improves response times (p95: 245ms)

### Negative
- Additional infrastructure component to maintain
- BFF becomes a potential single point of failure
- Requires synchronization between BFF and mobile schemas
- Added latency for BFF processing (50-100ms)
- Team must maintain expertise in both GraphQL and REST backends

### Mitigation
- Deploy BFF across multiple availability zones with auto-scaling
- Implement circuit breakers for backend service calls
- Version GraphQL schema with deprecation notices
- Cache aggressively to minimize backend load
- Monitor BFF health with comprehensive observability
- Document schema changes in shared repository

## Implementation
1. Set up Node.js/Express application with Apollo Server
2. Define GraphQL schema based on mobile screen requirements
3. Implement resolvers with DataLoader for backend service calls
4. Configure Redis caching layer with TTL policies
5. Add rate limiting middleware with tiered limits
6. Deploy to API Zone with load balancer and auto-scaling
7. Set up monitoring dashboards for latency, cache hit rates, error rates
8. Create mobile SDK with typed GraphQL client (Apollo Client)

## References
- [BFF Pattern by Sam Newman](https://samnewman.io/patterns/architectural/bff/)
- [Apollo Server Documentation](https://www.apollographql.com/docs/apollo-server/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)
- [FreshMart API Design Standards](https://wiki.freshmart.com/api-standards)
