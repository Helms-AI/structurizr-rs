# E-commerce Platform Architecture

## Overview

This example demonstrates a complete online retail platform architecture with approximately 25 elements, showcasing a modern microservices-based e-commerce system.

## Business Context

**Domain:** Online Retail
**Business Model:** B2C E-commerce
**Scale:** Medium enterprise with millions of products and thousands of concurrent users

### Key Business Capabilities
- Product catalog browsing and search
- Shopping cart and checkout
- Order management and fulfillment
- User account management
- Inventory tracking
- Payment processing
- Shipping integration
- Customer support
- Analytics and reporting

## Architecture Overview

### People (4)
- **Customer** - Browses and purchases products
- **Admin** - Manages products, orders, and system configuration
- **Warehouse Staff** - Fulfills orders and manages inventory
- **Support Agent** - Assists customers with issues

### Software Systems (6)
- **E-commerce Platform** - Main system (this system)
- **Payment Gateway** - External payment processing (Stripe/PayPal)
- **Shipping Provider** - External shipping and logistics (FedEx/UPS)
- **Email Service** - External email delivery (SendGrid)
- **Analytics Platform** - External analytics (Google Analytics)
- **Fraud Detection** - External fraud prevention

### Containers (12)
- **Web Store** - React-based customer-facing web application
- **Mobile App** - React Native mobile application
- **API Gateway** - Kong API Gateway for routing and authentication
- **Order Service** - Order processing and management
- **Inventory Service** - Stock and warehouse management
- **User Service** - User accounts and authentication
- **Product Catalog** - Product information management
- **Search Service** - Full-text product search
- **Notification Service** - Email and push notifications
- **Cache** - Redis for session and data caching
- **Primary Database** - PostgreSQL for transactional data
- **Search Index** - Elasticsearch for product search

### Components (3 in Order Service)
- **Order Controller** - REST API endpoints
- **Payment Handler** - Payment processing logic
- **Inventory Checker** - Stock validation

## Technical Architecture

### Technology Stack
- **Frontend:** React (Web), React Native (Mobile)
- **API Gateway:** Kong
- **Backend Services:** Java with Spring Boot
- **Databases:** PostgreSQL (primary), Redis (cache), Elasticsearch (search)
- **Message Queue:** RabbitMQ
- **Deployment:** AWS (ECS, RDS, ElastiCache, CloudFront)

### Key Patterns
- Microservices architecture
- API Gateway pattern
- CQRS (Command Query Responsibility Segregation) for search
- Cache-aside pattern for performance
- Event-driven notifications
- Circuit breaker for external services

## Views

### System Landscape View
Shows all users, the e-commerce platform, and external systems in context.

### System Context View
Focuses on the e-commerce platform and its interactions with users and external systems.

### Container View
Details all 12 containers within the platform and their relationships.

### Component View (Order Service)
Breaks down the Order Service into its key components.

### Dynamic View (Checkout Flow)
Illustrates the step-by-step process of a customer completing a purchase:
1. Customer adds item to cart (Web Store → Product Catalog)
2. Customer initiates checkout (Web Store → API Gateway)
3. Order creation (API Gateway → Order Service)
4. Stock validation (Order Service → Inventory Service)
5. Payment processing (Order Service → Payment Gateway)
6. Stock reservation (Order Service → Inventory Service)
7. Order confirmation (Order Service → Notification Service)
8. Confirmation email sent (Notification Service → Email Service)

### Deployment View (AWS Production)
Shows the deployment topology on AWS:
- **CloudFront CDN** - Static asset delivery
- **ECS Cluster** - Container orchestration for all services
- **RDS PostgreSQL** - Managed database
- **ElastiCache Redis** - Managed cache
- **Elasticsearch Service** - Managed search

## DSL Features Demonstrated

### Constants
- Color schemes for different element types
- Environment-specific values

### Implied Relationships
- Enabled to automatically infer container-to-container relationships from component relationships

### Documentation
- Embedded architecture documentation
- Links to external resources

### ADRs (Architecture Decision Records)
- ADR-001: Use of microservices architecture
- ADR-002: Selection of PostgreSQL over NoSQL
- ADR-003: Elasticsearch for product search

### Tags
- `Web Browser` - Browser-based applications
- `Mobile App` - Native mobile applications
- `Database` - Data storage systems
- `Cache` - Caching layers
- `External` - Third-party systems
- `Critical` - Business-critical components

### Groups
- Customer-facing applications
- Backend services
- Data stores

### Perspectives
- **Security:** Authentication, encryption, PCI compliance
- **Performance:** Caching strategy, search optimization
- **Cost:** Infrastructure costs per component

### Styles
- Person shapes for users
- Cylinder shapes for databases
- Pipe shapes for caches
- Custom colors for different layers
- Dotted lines for async relationships

## Key Architectural Decisions

### ADR-001: Microservices Architecture
**Decision:** Use microservices architecture instead of monolithic design
**Rationale:**
- Independent scaling of services (search vs checkout)
- Team autonomy for faster development
- Technology diversity where beneficial
- Fault isolation

**Consequences:**
- Increased operational complexity
- Need for service mesh/API gateway
- Distributed transaction challenges

### ADR-002: PostgreSQL for Primary Database
**Decision:** Use PostgreSQL instead of NoSQL databases
**Rationale:**
- ACID transactions critical for orders and payments
- Complex queries for reporting
- Mature ecosystem and tooling
- Strong consistency guarantees

**Consequences:**
- May need denormalization for performance
- Horizontal scaling more complex
- Redis caching essential for read performance

### ADR-003: Elasticsearch for Search
**Decision:** Use Elasticsearch for product search instead of database full-text search
**Rationale:**
- Superior full-text search capabilities
- Faceted search and filtering
- Scalable for large product catalogs
- Better relevance ranking

**Consequences:**
- Data synchronization complexity
- Additional infrastructure to maintain
- Eventual consistency acceptable for search

## Security Considerations

- **Authentication:** OAuth2 with JWT tokens via User Service
- **Authorization:** Role-based access control (Customer, Admin, Warehouse, Support)
- **PCI Compliance:** Payment Gateway handles sensitive card data
- **Data Encryption:** TLS in transit, encryption at rest for sensitive data
- **API Security:** Rate limiting and DDoS protection at API Gateway
- **Fraud Prevention:** Integration with external fraud detection service

## Performance Characteristics

- **Caching Strategy:**
  - Redis for session data (TTL: 30 minutes)
  - Redis for product catalog (TTL: 1 hour)
  - CloudFront for static assets (TTL: 24 hours)

- **Search Performance:**
  - Elasticsearch indexing: Near real-time (1 second refresh)
  - Search latency: <100ms for 99th percentile

- **Database:**
  - Read replicas for reporting queries
  - Connection pooling for efficiency

## Cost Optimization

- **Auto-scaling:** ECS services scale based on CPU/memory
- **Reserved Instances:** RDS and ElastiCache for cost savings
- **CloudFront:** Reduces origin load and egress costs
- **Right-sizing:** Different instance sizes per service needs

## Running the Example

```bash
# Validate the DSL
cargo run -- validate workspaces/medium/ecommerce-platform/workspace.dsl

# Render diagrams
cargo run -- render --workspace workspaces/medium/ecommerce-platform/workspace.dsl --output ./output

# Serve interactively
cargo run -- serve --workspace workspaces/medium/ecommerce-platform/workspace.dsl --port 8080
```

## Future Enhancements

- Add recommendation engine container
- Implement event sourcing for order history
- Add warehouse management system integration
- Implement GraphQL API alongside REST
- Add real-time inventory updates via WebSockets
