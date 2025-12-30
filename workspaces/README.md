# FreshMart Corporation - Architecture Workspaces

## Overview

This collection of workspaces documents the complete enterprise architecture for **FreshMart Corporation**, a Fortune 500 grocery retailer with 2,500+ stores building a next-generation Point of Sale system.

## Workspace Organization

Workspaces are organized by **business domain**:

```
workspaces/
├── core/                       # Mission-critical transaction systems
│   ├── payment-gateway/        # Secure payment processing (PCI-DSS)
│   ├── pos-terminal/           # Edge-first POS with plugins
│   └── inventory-platform/     # Event-sourced inventory (CQRS)
├── customer/                   # Customer-facing systems
│   ├── loyalty-rewards/        # ML-powered loyalty program
│   └── mobile-experience/      # PWA with offline support
├── operations/                 # Store and supply chain
│   ├── store-operations/       # IoT-enabled smart stores
│   └── supply-chain/           # B2B integration and logistics
└── platform/                   # Shared capabilities
    └── analytics-ai/           # ML platform and real-time analytics
```

## Workspace Descriptions

### Core Domain

#### [payment-gateway](core/payment-gateway/)
**PCI-compliant payment processing platform**
- Multi-acquirer support (Visa, Mastercard, Amex, digital wallets)
- HSM-backed tokenization vault
- Real-time fraud detection
- Settlement and reconciliation automation
- **DSL Features**: Security perspectives, complex deployment views, filtered views

#### [pos-terminal](core/pos-terminal/)
**Next-generation edge-first POS system**
- Offline-first architecture with store-and-forward
- Plugin extensibility via WASM
- Hardware abstraction layer (scanners, scales, printers)
- Store controller coordination
- **DSL Features**: Edge deployment, hardware integration, plugin architecture

#### [inventory-platform](core/inventory-platform/)
**Real-time inventory with event sourcing**
- Event-sourced with CQRS pattern
- Multiple read model projections
- Reservation and allocation engines
- Cycle counting and variance analysis
- **DSL Features**: Event streaming patterns, saga orchestration, replay flows

### Customer Domain

#### [loyalty-rewards](customer/loyalty-rewards/)
**AI-powered customer engagement platform**
- 25M+ member profiles
- ML personalization engine
- Points earn/burn with gamification
- Multi-channel campaigns
- **DSL Features**: ML components, customer journey flows, real-time personalization

#### [mobile-experience](customer/mobile-experience/)
**Cross-platform mobile experience**
- React Native + PWA
- Offline-first with background sync
- BFF (Backend for Frontend) pattern
- Real-time push notifications
- **DSL Features**: PWA patterns, offline sync flows, multi-platform deployment

### Operations Domain

#### [store-operations](operations/store-operations/)
**IoT-enabled smart store management**
- 100,000+ IoT sensors
- Task management and assignment
- Labor scheduling optimization
- Digital signage and ESL management
- **DSL Features**: IoT integration, workflow orchestration, sensor processing

#### [supply-chain](operations/supply-chain/)
**B2B integration and logistics optimization**
- EDI gateway (ANSI X12)
- Demand forecasting with ML
- Multi-carrier logistics management
- Supplier quality management
- **DSL Features**: EDI integration, B2B patterns, multi-party workflows

### Platform Domain

#### [analytics-ai](platform/analytics-ai/)
**Enterprise data and ML platform**
- Lambda architecture (batch + streaming)
- Feature store and model registry
- Real-time fraud detection
- Demand forecasting ML
- **DSL Features**: Data pipelines, ML infrastructure, Lambda patterns

## DSL Features Demonstrated

Each workspace comprehensively demonstrates Structurizr DSL features:

| Feature | Workspaces |
|---------|------------|
| `!const` | All |
| `!impliedRelationships` | All |
| `!docs` / `!adrs` | All |
| Groups | All |
| Properties | All |
| Perspectives (via properties) | payment-gateway, analytics-ai, supply-chain |
| Parallel dynamics `{ }` | All (5+ dynamic views each) |
| All element types | All |
| Multiple autoLayout directions | All |
| Filtered views | All |
| Themes | All |
| Component-level views | All |
| Deployment views | All |

## Architecture Highlights

### Next-Generation Capabilities

1. **AI-Powered Operations**
   - Real-time fraud detection (analytics-ai, payment-gateway)
   - Demand forecasting (supply-chain, analytics-ai)
   - Personalization engine (loyalty-rewards)
   - Optimal routing (payment-gateway, supply-chain)

2. **Cloud-Native Architecture**
   - Microservices throughout
   - Event-driven integration (Kafka)
   - Kubernetes deployments
   - Auto-scaling and self-healing

3. **Edge Computing**
   - Store-level autonomy (pos-terminal)
   - Offline-first design (mobile-experience)
   - Local transaction processing
   - Edge-cloud synchronization

4. **Event-Driven Patterns**
   - Event sourcing (inventory-platform)
   - CQRS (inventory-platform)
   - Saga orchestration (payment-gateway, inventory-platform)
   - Real-time streaming (analytics-ai)

### Technology Stack

- **Languages**: Java, Rust, Python, TypeScript, Go
- **Databases**: PostgreSQL, ScyllaDB, Redis, Elasticsearch
- **Streaming**: Apache Kafka, Apache Flink
- **ML/AI**: TensorFlow, PyTorch, XGBoost, Prophet
- **Cloud**: AWS, Kubernetes, Terraform
- **Integration**: EDI, REST, GraphQL, gRPC

## Running Workspaces

```bash
# Serve all workspaces
cargo run -- serve --workspaces-dir workspaces

# Validate a specific workspace
cargo run -- validate workspaces/core/payment-gateway/workspace.dsl

# Export a workspace to JSON
cargo run -- export --workspace workspaces/core/payment-gateway/workspace.dsl --format json
```

## Business Impact

- **$10B+ Annual Revenue**: Processed through POS
- **5M+ Daily Transactions**: Real-time processing
- **25M+ Loyalty Members**: Personalized engagement
- **2,500+ Stores**: Connected infrastructure
- **99.999% Uptime**: Mission-critical reliability

## Documentation

Each workspace includes:
- `workspace.dsl` - Complete Structurizr DSL definition
- `README.md` - Architecture overview
- `docs/` - Detailed documentation
- `adrs/` - Architecture Decision Records

## Contributing

When adding new workspaces:
1. Place in appropriate domain folder (core/, customer/, operations/, platform/)
2. Include all C4 levels (Landscape, Context, Container, Component)
3. Add dynamic views with parallel `{ }` blocks
4. Include deployment views
5. Use shared theme: `../../../freshmart-theme.json`
6. Follow naming conventions (lowercase, hyphens)