# ADR-003: REST/SOAP Bridge for Supplier API Gateway

## Status
Accepted

## Context
FreshMart's supplier ecosystem spans a wide technology spectrum:
- Large CPG companies with modern REST APIs
- Mid-size suppliers with legacy SOAP web services
- Small suppliers relying on the portal or manual processes

We need an API gateway strategy that:
- Provides a unified REST interface for internal systems
- Bridges to suppliers regardless of their integration capability
- Ensures security with proper authentication and authorization
- Handles 10,000+ API requests per minute at peak
- Supports graceful degradation and circuit breaking

## Decision
We will implement a Kong Enterprise-based API gateway with REST/SOAP bridging capabilities:

### 1. API Gateway Architecture
- **Kong Enterprise**: Primary API gateway and management platform
- **Rate Limiting**: 10,000 requests/minute per supplier
- **Authentication**: OAuth 2.0 with JWT tokens
- **Transformation**: Payload mapping between REST and SOAP

### 2. Supplier Authentication
```
OAuth 2.0 Flow:
1. Supplier obtains client credentials from portal onboarding
2. Token request to /oauth/token with client_id and client_secret
3. JWT token issued with supplier-specific claims
4. Token used in Authorization header for API calls
5. Token refresh every 24 hours with rolling expiration
```

### 3. API Endpoints
| Endpoint | Method | Description |
|----------|--------|-------------|
| /api/v1/orders | GET | List purchase orders for supplier |
| /api/v1/orders/{id} | GET | Get purchase order details |
| /api/v1/orders/{id}/acknowledge | POST | Acknowledge purchase order |
| /api/v1/shipments | POST | Create advance ship notice |
| /api/v1/shipments/{id}/tracking | POST | Update shipment tracking |
| /api/v1/invoices | POST | Submit invoice |
| /api/v1/catalog | PUT | Update product catalog |

### 4. SOAP Bridge Implementation
For suppliers with SOAP-only capabilities:
- Kong plugin transforms inbound SOAP to internal REST format
- Outbound responses converted to SOAP envelope
- WSDL generation for supplier compatibility
- Support for WS-Security authentication

### 5. Kong Plugin Stack
```yaml
plugins:
  - name: oauth2
    config:
      scopes: [orders:read, orders:write, shipments:write]
  - name: rate-limiting
    config:
      minute: 10000
      policy: local
  - name: request-transformer
    config:
      # REST to internal format mapping
  - name: response-transformer
    config:
      # Internal to supplier format mapping
  - name: circuit-breaker
    config:
      failure_threshold: 5
      recovery_time: 30
```

### 6. Supplier Portal Integration
- Portal uses same API gateway for consistency
- Additional browser-facing endpoints for UI operations
- Session-based authentication with PKCE flow
- Real-time notifications via WebSocket upgrade

## Consequences

### Positive
- Single API surface for all supplier integrations
- Reduced integration time from 4 weeks to 5 days
- Centralized security policies and rate limiting
- Detailed API analytics and usage metrics
- Self-service API documentation via developer portal

### Negative
- Kong Enterprise licensing costs ($150K/year)
- Added latency (10-20ms) for gateway processing
- SOAP transformation complexity for edge cases
- Single point of failure without proper HA configuration

### Mitigation
- Deploy Kong in multi-region active-active configuration
- Implement caching for frequently accessed data
- Build comprehensive test suite for SOAP transformations
- Maintain manual fallback procedures for critical suppliers

## Implementation
1. Deploy Kong Enterprise cluster across availability zones
2. Configure OAuth 2.0 provider and credential management
3. Build request/response transformer plugins
4. Implement SOAP bridge for legacy suppliers
5. Create developer portal with API documentation
6. Set up monitoring and alerting in Datadog
7. Migrate existing API integrations to gateway

## References
- [Kong Enterprise Documentation](https://wiki.freshmart.com/kong-gateway)
- [OAuth 2.0 Security Standards](https://wiki.freshmart.com/oauth2-standards)
- [Supplier API Onboarding Guide](https://wiki.freshmart.com/supplier-api-onboarding)
- [SOAP Bridge Configuration](https://wiki.freshmart.com/soap-bridge)
