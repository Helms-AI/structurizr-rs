# ADR-003: Edge Deployment with K3s

## Status
Accepted

## Context
FreshMart operates 2,500+ retail stores, each requiring robust, locally-resilient computing infrastructure for POS operations. The traditional approach of connecting directly to cloud services creates several challenges:

- Network latency impacts transaction speed
- Cloud outages affect all stores simultaneously
- Bandwidth costs for 5M+ daily transactions are significant
- Regulatory requirements may mandate local data processing
- Real-time hardware integration requires local compute

We need a deployment strategy that provides cloud-like orchestration capabilities while maintaining local resilience and performance.

## Decision
We will deploy a lightweight Kubernetes distribution (K3s) at the store level to orchestrate POS workloads, providing container management, service discovery, and high availability within each store.

### Deployment Architecture

```
Store Infrastructure
├── Store Server (K3s Control Plane)
│   ├── Store Controller (1 replica, active-passive)
│   ├── Sync Services
│   └── Monitoring Stack
│
└── POS Lanes (K3s Worker Nodes)
    ├── POS User Interface (React Native)
    ├── POS Core Engine (Rust)
    ├── Plugin Manager (Rust + WASM)
    ├── Hardware Manager (Rust)
    ├── Offline Engine (SQLite + RocksDB)
    └── Security Manager (Rust)
```

### Workload Distribution

| Workload | Location | Rationale |
|----------|----------|-----------|
| Transaction Processing | Edge (Lane) | Sub-second latency requirement |
| Payment Authorization | Edge + Cloud | Local queue, cloud authorization |
| Product Catalog | Edge (Cached) | Offline capability required |
| Inventory Updates | Edge (Async) | Batch sync to cloud |
| Analytics Aggregation | Edge (Store) | Pre-aggregate before cloud upload |
| ML Model Inference | Edge (Store) | Privacy, latency |
| Model Training | Cloud | Compute-intensive, uses aggregated data |
| Global Reporting | Cloud | Cross-store aggregation |

### K3s Configuration

1. **Control Plane**: Single store server with SQLite backend (lightweight, sufficient for store scale)

2. **Worker Nodes**: Each POS lane runs as a K3s agent with the full POS stack

3. **Storage**: Local persistent volumes for SQLite and RocksDB data

4. **Networking**: Flannel CNI with host-local IPAM for simplicity

5. **High Availability**: Store Controller runs with readiness probes; failover to standby if unhealthy

### Cloud Connectivity

- **Store-to-Cloud**: Encrypted tunnel (WireGuard) for sync traffic
- **Fallback**: REST API over TLS when tunnel unavailable
- **Bandwidth**: Compressed delta sync to minimize data transfer
- **Priority**: Critical traffic (payments) prioritized over bulk sync

## Consequences

### Positive
- Complete store operation during cloud outages
- Sub-10ms local service communication latency
- Consistent deployment model across all stores
- Rolling updates without store downtime
- Local resource isolation prevents cascade failures
- Reduced cloud bandwidth costs (~60% reduction)
- Compliance-friendly local data processing

### Negative
- IT must manage 2,500+ K3s clusters
- Hardware requirements increase (Store Server needed)
- More complex troubleshooting across distributed systems
- Software updates require edge deployment pipeline
- Limited compute capacity vs. cloud (10-20 lanes per store)

### Mitigation
- Fleet management via Rancher for centralized visibility
- Standardized hardware spec reduces variability
- Remote debugging via Store Controller tunnel
- GitOps-based deployment pipeline with staged rollout
- Auto-scaling to cloud for compute-intensive tasks

## Implementation

### Phase 1: Infrastructure
1. Deploy K3s on Store Server hardware
2. Configure networking and storage
3. Establish cloud connectivity tunnel
4. Deploy monitoring stack (Prometheus + Grafana)

### Phase 2: Workload Migration
1. Containerize POS stack components
2. Create Helm charts for deployment
3. Deploy Store Controller with HA configuration
4. Migrate lanes to K3s worker nodes

### Phase 3: Operations
1. Implement GitOps deployment pipeline
2. Configure centralized logging (Loki)
3. Set up alerting and escalation
4. Document runbooks for common scenarios

### Hardware Requirements

| Component | Store Server | POS Lane |
|-----------|--------------|----------|
| CPU | Intel i5 (4 cores) | Intel Celeron (2 cores) |
| RAM | 16GB | 8GB |
| Storage | 256GB SSD | 128GB SSD |
| Network | 1Gbps Ethernet | 1Gbps Ethernet |
| OS | Ubuntu 22.04 LTS | Windows 10 IoT / Android |

### Monitoring

- **Cluster Health**: Node status, pod restarts, resource utilization
- **Application Health**: Transaction throughput, error rates, latency
- **Sync Status**: Queue depth, sync latency, conflict rate
- **Hardware Health**: Device connectivity, paper status, errors

## References
- [Store Controller Components](../docs/index.md)
- [K3s Documentation](https://k3s.io/)
- [Fleet Management Guide](https://wiki.freshmart.com/fleet-management)
- [Edge Deployment Runbook](https://wiki.freshmart.com/edge-runbook)
