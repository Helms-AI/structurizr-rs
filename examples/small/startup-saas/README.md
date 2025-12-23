# Startup SaaS Analytics Platform

## Overview

This example models a B2B SaaS analytics platform for a startup company. The platform provides real-time analytics and insights for product managers and developers, helping them understand user behavior and application performance.

## Business Context

**Company:** DataInsight Analytics (fictional startup)
**Industry:** SaaS / Analytics
**Stage:** Series A, 15 employees
**Users:** Product managers and developers at B2B companies

### Value Proposition

DataInsight Analytics helps product teams make data-driven decisions by:
- Tracking user behavior in real-time
- Providing actionable insights through dashboards
- Integrating with existing data warehouses
- Offering secure, OAuth-based authentication

## Architecture Overview

### System Context

The Analytics Platform is the core system, integrating with:
- **Auth0** - External OAuth provider for secure authentication
- **Snowflake** - External data warehouse for customer data ingestion

### Containers

The platform follows a modern three-tier architecture:

1. **Web Application** (React + TypeScript)
   - Single-page application
   - Responsive dashboards and visualizations
   - Real-time updates via WebSockets

2. **API Gateway** (Node.js + Express)
   - RESTful API for data access
   - Authentication and authorization
   - WebSocket support for real-time features
   - Rate limiting and request validation

3. **Database** (PostgreSQL)
   - User accounts and configurations
   - Analytics events and aggregations
   - Query result caching

## Key Workflows

### User Login Flow (Dynamic View)

The dynamic view demonstrates the OAuth authentication flow:

1. User opens Web Application
2. Web App redirects to Auth0 for login
3. Auth0 validates credentials
4. Auth0 returns authorization token
5. Web App exchanges token with API Gateway
6. API Gateway validates token with Auth0
7. API Gateway returns session token
8. User accesses authenticated features

## Technical Decisions

### ADR-001: Use Auth0 for Authentication

**Status:** Accepted
**Context:** Need secure, scalable authentication without building in-house
**Decision:** Use Auth0 as external OAuth provider
**Consequences:**
- Faster time to market
- Reduced security risk
- Monthly cost per active user
- Dependency on external service

### ADR-002: PostgreSQL for Analytics Storage

**Status:** Accepted
**Context:** Need reliable, queryable storage for analytics data
**Decision:** Use PostgreSQL with time-series optimizations
**Consequences:**
- Familiar SQL interface
- Strong consistency guarantees
- Need to manage scaling for high-volume data
- Considered TimescaleDB extension for future

### ADR-003: Snowflake Integration for Data Ingestion

**Status:** Accepted
**Context:** Customers want to analyze their existing data warehouse data
**Decision:** Build direct Snowflake connector
**Consequences:**
- Access to large enterprise customers
- Complex integration and security requirements
- Competitive advantage in enterprise market

## Perspectives

### Security Perspective

- OAuth 2.0 authentication via Auth0
- API Gateway enforces authentication on all endpoints
- Database uses encrypted connections
- Secrets managed via environment variables

### Cost Perspective

- Auth0: ~$200/month (startup plan)
- Hosting: ~$150/month (cloud infrastructure)
- Snowflake: Usage-based, ~$500/month average
- Total: ~$850/month operational cost

### Scalability Perspective

- API Gateway: Horizontally scalable via load balancer
- Database: Vertical scaling initially, read replicas planned
- Web App: Static assets served via CDN
- Target: Support 1000 concurrent users

## Technology Stack

- **Frontend:** React 18, TypeScript, Recharts, WebSocket
- **Backend:** Node.js 18, Express, JWT, WebSocket
- **Database:** PostgreSQL 15, TimescaleDB extension
- **Infrastructure:** Docker, AWS ECS, RDS
- **Authentication:** Auth0
- **Data Integration:** Snowflake Node.js SDK

## Future Enhancements

1. Add mobile application for iOS/Android
2. Implement data export to CSV/Excel
3. Add custom alerting and notifications
4. Build query builder for non-technical users
5. Add data pipeline orchestration
