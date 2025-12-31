# ADR-018: NATS as Unified Messaging Platform

## Status

**Accepted**

## Date

2025-01-01

## Context

The Horizon Platform currently uses multiple messaging and state management systems:

1. **Kafka** - Event streaming and async messaging (100+ topics, 500K msg/sec)
2. **Redis** - Presence, sessions, caching, real-time state
3. **MinIO** - CRDT snapshots and document persistence

### Current Architecture

```
+-----------------------------------------------------------------------+
|                          DATA & INFRASTRUCTURE                         |
|  +----------------------+  +----------------------+  +---------------+ |
|  |   Kafka (Events)     |  |   Redis (Cache +     |  |    MinIO      | |
|  |   - workspace.*      |  |   Presence)          |  |  (Snapshots)  | |
|  |   - file.*           |  |   - Sessions         |  |  - CRDT state | |
|  |   - deployment.*     |  |   - Awareness        |  |  - Documents  | |
|  |   - collaboration.*  |  |   - Cursor positions |  |               | |
|  +----------------------+  +----------------------+  +---------------+ |
+-----------------------------------------------------------------------+
```

### Problems with Current Approach

| Problem | Impact |
|---------|--------|
| **Operational Complexity** | Three separate systems to deploy, monitor, and maintain |
| **Kafka Overhead** | ZooKeeper dependency, JVM memory footprint, complex configuration |
| **Redis Single Point** | Presence data loss on Redis failure affects all collaborators |
| **Latency Stack** | Multiple hops between systems adds latency |
| **Cost** | Separate resource pools for each system |
| **Consistency** | No transactional guarantees across systems |

### Requirements

1. Sub-50ms pub/sub latency for real-time collaboration
2. Persistent message streaming with replay capability
3. Distributed key-value store for presence and state
4. Object storage for CRDT snapshots (up to 10MB)
5. Horizontal scalability across regions
6. Simpler operational footprint
7. Cloud-native Kubernetes deployment

## Decision

We will migrate to **NATS** with **JetStream** as the unified messaging platform, replacing Kafka, Redis (for presence), and MinIO (for CRDT snapshots).

### Why NATS?

| Criteria | Kafka | Redis | NATS + JetStream |
|----------|-------|-------|------------------|
| **Pub/Sub Latency** | ~5-10ms | <1ms | <1ms |
| **Message Persistence** | Yes | Limited | Yes (JetStream) |
| **Key-Value Store** | No | Yes | Yes (NATS KV) |
| **Object Storage** | No | No | Yes (NATS Object Store) |
| **Memory Footprint** | High (JVM) | Medium | Low (Go) |
| **Dependencies** | ZooKeeper | None | None |
| **Cluster Setup** | Complex | Medium | Simple |
| **Cloud Native** | Helm available | Helm available | Helm + Operator |

### NATS Subsystem Mapping

| Current System | NATS Replacement | Use Case |
|----------------|------------------|----------|
| Kafka Topics | JetStream Streams | Event streaming, message replay |
| Redis Pub/Sub | Core NATS | Real-time notifications |
| Redis Keys (presence) | NATS KV Buckets | Sessions, presence, awareness |
| MinIO (snapshots) | NATS Object Store | CRDT snapshots, document state |

### Architecture Overview

```
+-----------------------------------------------------------------------+
|                          NATS CLUSTER (JetStream)                      |
|                                                                        |
|  +-------------------+  +-------------------+  +-------------------+   |
|  |  Core NATS        |  |  JetStream        |  |  NATS KV          |   |
|  |  (Pub/Sub)        |  |  (Streams)        |  |  (State)          |   |
|  |  - Real-time      |  |  - workspace.*    |  |  - sessions       |   |
|  |  - Notifications  |  |  - file.*         |  |  - presence       |   |
|  |  - Ephemeral      |  |  - deployment.*   |  |  - cursors        |   |
|  +-------------------+  +-------------------+  +-------------------+   |
|                                                                        |
|  +-------------------------------------------------------------------+ |
|  |                    NATS Object Store                               | |
|  |  - CRDT snapshots (collab/{workspace}/{file}/snapshot.yjs)        | |
|  |  - Document state vectors                                          | |
|  +-------------------------------------------------------------------+ |
+-----------------------------------------------------------------------+
```

## Implementation

### NATS Cluster Configuration

```yaml
# Kubernetes Deployment via Helm
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: nats
  namespace: horizon-data
spec:
  interval: 5m
  chart:
    spec:
      chart: nats
      version: "1.x.x"
      sourceRef:
        kind: HelmRepository
        name: nats
  values:
    cluster:
      enabled: true
      replicas: 3
    jetstream:
      enabled: true
      memoryStore:
        enabled: true
        size: 2Gi
      fileStore:
        enabled: true
        size: 50Gi
        storageClassName: fast-ssd
    natsBox:
      enabled: true
    resources:
      requests:
        cpu: 500m
        memory: 1Gi
      limits:
        cpu: 2000m
        memory: 4Gi
```

### JetStream Streams

| Stream | Subjects | Retention | Storage |
|--------|----------|-----------|---------|
| CONTAINER_EVENTS | horizon.events.container.> | 7 days | File |
| WORKSPACE_EVENTS | horizon.events.workspace.> | 30 days | File |
| DEPLOYMENT_EVENTS | horizon.events.deployment.> | 90 days | File |
| COLLABORATION_EVENTS | horizon.events.collaboration.> | 24 hours | Memory |
| NOTIFICATION_EVENTS | horizon.events.notification.> | 7 days | File |

### NATS KV Buckets

| Bucket | Purpose | TTL | Storage |
|--------|---------|-----|---------|
| PRESENCE | User online status | 30s | Memory |
| CURSORS | Cursor positions | 5s | Memory |
| SESSIONS | Active sessions | None | File |
| COLLAB_STATE | Room state | 24h | File |

### Subject Hierarchy

```
horizon.
+-- events.                              # JetStream Streams
|   +-- container.{created|started|stopped|health.*}
|   +-- workspace.{created|updated|deleted|forked}
|   +-- deployment.{started|progress.*|completed|failed}
|   +-- collaboration.{room.*|operations.*|sync.*}
|   +-- notification.{user.*|workspace.*|broadcast}
|   +-- ai.{completion.*|agent.*|embedding.*}
+-- presence.{workspace_id}.{user_id}    # NATS KV
+-- cursor.{workspace_id}.{file_id}.*    # NATS KV
+-- state.{session|collaboration}.*      # NATS KV
+-- rpc.{workspace|container|ai}.*       # Request-Reply
```

## Migration Strategy

### Phase 1: Deploy NATS Alongside Existing Systems (Week 1-2)

- Deploy NATS cluster in `horizon-data` namespace
- Configure JetStream streams mirroring Kafka topics
- Set up KV buckets for presence data
- Create Object Store for snapshots

### Phase 2: Dual-Write Implementation (Week 3-4)

- Implement dual-write to both Kafka/Redis and NATS
- Add feature flags for gradual traffic shifting
- Monitor latency and error rates

### Phase 3: Read Migration (Week 5-6)

- Shift reads from Kafka to JetStream (with fallback)
- Shift presence reads from Redis to NATS KV
- Validate data consistency

### Phase 4: Write Migration (Week 7-8)

- Shift writes to NATS as primary
- Maintain Redis/Kafka as backup
- Run parallel validation

### Phase 5: Decommission Legacy Systems (Week 9-10)

- Remove Kafka cluster (keep ZooKeeper removal separate)
- Remove Redis presence data (keep Redis for caching if needed)
- Update monitoring and alerting
- Archive MinIO CRDT data and migrate to NATS Object Store

## Consequences

### Positive

1. **Unified Platform**: Single system for pub/sub, streaming, KV, and objects
2. **Lower Latency**: Sub-millisecond pub/sub, no cross-system hops
3. **Simpler Operations**: One cluster to deploy, monitor, scale
4. **Resource Efficiency**: Lower memory footprint than Kafka+Redis+MinIO
5. **Cloud Native**: Kubernetes operator, Helm charts, easy horizontal scaling
6. **Built-in Clustering**: Native clustering without ZooKeeper
7. **Consistent APIs**: Single client library for all messaging needs

### Negative

1. **Learning Curve**: Team needs to learn NATS/JetStream APIs
2. **Migration Risk**: Complex migration with multiple systems
3. **Ecosystem**: Smaller ecosystem than Kafka for enterprise integrations
4. **Object Store Limits**: 10MB per object (sufficient for CRDT snapshots)

### Mitigations

| Risk | Mitigation |
|------|------------|
| Learning curve | NATS documentation is excellent; similar concepts to Kafka |
| Migration risk | Phased migration with dual-write and feature flags |
| Ecosystem | NATS has growing adoption; covers our use cases |
| Object limits | CRDT snapshots typically <5MB; chunking for larger files |

## Backwards Compatibility

### API Compatibility

- Event schemas remain unchanged (same JSON payloads)
- WebSocket clients unaffected (collaboration engine internal change)
- REST API unchanged

### Data Migration

- Historical Kafka events: Export and replay into JetStream
- Redis presence: Ephemeral data, no migration needed
- MinIO snapshots: Bulk copy to NATS Object Store

## Monitoring

### Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `nats_jetstream_consumer_ack_pending` | Unacked messages | >10000 |
| `nats_jetstream_stream_messages` | Stream message count | varies |
| `nats_kv_operation_latency_ms` | KV operation latency | P99 >50ms |
| `nats_core_pub_latency_ms` | Pub/sub latency | P99 >10ms |
| `nats_cluster_size` | Cluster members | <3 |

## References

- [NATS Documentation](https://docs.nats.io/)
- [JetStream Documentation](https://docs.nats.io/nats-concepts/jetstream)
- [NATS KV Store](https://docs.nats.io/nats-concepts/jetstream/key-value-store)
- [NATS Object Store](https://docs.nats.io/nats-concepts/jetstream/obj_store)
- [NATS Helm Chart](https://github.com/nats-io/k8s/tree/main/helm/charts/nats)
- [async-nats Rust Crate](https://crates.io/crates/async-nats)
