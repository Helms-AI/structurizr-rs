# ADR-009: Multi-Region Infrastructure Strategy

## Status

Accepted (Updated: 2025-01-15)

## Context

The Horizon Platform must serve users globally with low latency and high availability. Requirements include:

- **Latency**: <100ms for API responses, <50ms for WebSocket messages
- **Availability**: 99.9% uptime SLA
- **Disaster Recovery**: RTO <15 minutes, RPO <5 minutes
- **Compliance**: Data residency for EU users (GDPR)
- **Scale**: 100K+ concurrent users globally

**Geographic Distribution:**
- 40% North America
- 30% Europe
- 20% Asia Pacific
- 10% Rest of World

## Decision

We will deploy a **multi-region active-active architecture** on cloud-agnostic Kubernetes with:

1. **Primary Regions**: US West, EU West
2. **Secondary Regions**: Asia Pacific
3. **Global Load Balancing**: Cloudflare or cloud-native load balancing with CDN
4. **Data Replication**: Cross-region with conflict resolution

## Alternatives Considered

### Single Region with CDN

**Pros:**
- Simplest to operate
- Lowest cost
- No data replication complexity

**Cons:**
- High latency for distant users
- Single point of failure
- Cannot meet data residency requirements

**Why Rejected:** Latency and availability requirements not achievable.

### Active-Passive Multi-Region

**Pros:**
- Simpler failover logic
- Lower replication overhead
- Clear primary/secondary roles

**Cons:**
- Wasted capacity in passive region
- Failover introduces downtime
- Higher latency during normal operation

**Why Rejected:** Active-active provides better latency and resource utilization.

### Multi-Cloud (GCP + AWS)

**Pros:**
- No vendor lock-in
- Ultimate redundancy
- Best-of-breed services

**Cons:**
- Significantly higher complexity
- Data egress costs
- Different APIs and tooling
- Operational overhead

**Why Rejected:** Complexity outweighs vendor diversification benefits.

## Consequences

### Positive

- **Low latency**: Users routed to nearest region
- **High availability**: Automatic failover between regions
- **Compliance**: EU data stays in EU region
- **Scalability**: Each region scales independently
- **Blast radius**: Issues contained to single region

### Negative

- **Cost**: 2-3x infrastructure spend
- **Complexity**: Distributed systems challenges
- **Data consistency**: Eventually consistent across regions
- **Debugging**: Harder to trace cross-region issues

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Cost | Reserve instances, auto-scaling, spot VMs for batch |
| Complexity | Infrastructure as Code, standardized patterns |
| Consistency | CRDT for collaboration, conflict resolution for data |
| Debugging | Distributed tracing, centralized logging |

## Implementation

### Regional Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                     Global Load Balancer                         │
│              (Cloudflare / Cloud-native LB + CDN)               │
└─────────────┬───────────────────┬───────────────────┬───────────┘
              │                   │                   │
      ┌───────▼───────┐   ┌───────▼───────┐   ┌───────▼───────┐
      │   US West     │   │   EU West     │   │ Asia Pacific  │
      │   (Primary)   │   │   (Primary)   │   │  (Secondary)  │
      │               │   │               │   │               │
      │ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
      │ │    K8s    │ │   │ │    K8s    │ │   │ │    K8s    │ │
      │ │  Cluster  │ │   │ │  Cluster  │ │   │ │  Cluster  │ │
      │ └─────┬─────┘ │   │ └─────┬─────┘ │   │ └─────┬─────┘ │
      │       │       │   │       │       │   │       │       │
      │ ┌─────▼─────┐ │   │ ┌─────▼─────┐ │   │ ┌─────▼─────┐ │
      │ │PostgreSQL │ │   │ │PostgreSQL │ │   │ │PostgreSQL │ │
      │ │ (Primary) │◄├───┼─┤ (Replica) │◄├───┼─┤ (Replica) │ │
      │ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │
      │               │   │               │   │               │
      │ ┌───────────┐ │   │ ┌───────────┐ │   │ ┌───────────┐ │
      │ │  Redis    │◄├───┼─┤  Redis    │◄├───┼─┤  Redis    │ │
      │ │ Cluster   │ │   │ │ Cluster   │ │   │ │ Cluster   │ │
      │ └───────────┘ │   │ └───────────┘ │   │ └───────────┘ │
      └───────────────┘   └───────────────┘   └───────────────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  │
                    ┌─────────────▼─────────────┐
                    │   MinIO (S3-Compatible)   │
                    │   Multi-Region Replicated │
                    └───────────────────────────┘
```

### Kubernetes/Helm Configuration

```yaml
# Multi-region cluster configuration via Helm values
# regions.yaml - shared across all regions
apiVersion: v1
kind: ConfigMap
metadata:
  name: horizon-regions
data:
  regions: |
    - name: us-west
      role: primary
      nodes: 50
    - name: eu-west
      role: primary
      nodes: 40
    - name: asia-pacific
      role: secondary
      nodes: 20

---
# PostgreSQL with replication (Bitnami Helm Chart)
# helm install postgresql bitnami/postgresql -f postgresql-values.yaml
# postgresql-values.yaml:
postgresql:
  auth:
    database: horizon
    username: horizon
    existingSecret: postgresql-credentials
  primary:
    persistence:
      size: 100Gi
      storageClass: fast-ssd
    resources:
      requests:
        memory: 4Gi
        cpu: 2000m
  readReplicas:
    replicaCount: 2
  metrics:
    enabled: true

---
# Cross-region PostgreSQL streaming replication
# Uses Patroni for HA and automatic failover
apiVersion: v1
kind: ConfigMap
metadata:
  name: patroni-config
data:
  patroni.yml: |
    scope: horizon-cluster
    namespace: horizon-data
    bootstrap:
      dcs:
        postgresql:
          parameters:
            max_connections: 500
            wal_level: replica
            max_wal_senders: 10
    postgresql:
      authentication:
        replication:
          username: replicator
          password: ${REPLICATION_PASSWORD}

---
# Nginx Ingress with cert-manager
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: horizon-global-ingress
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  tls:
    - hosts:
        - horizonplatform.io
        - "*.horizonplatform.io"
      secretName: horizon-tls
  rules:
    - host: api.horizonplatform.io
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: api-gateway
                port:
                  number: 8080
```

### Traffic Routing

```yaml
# Istio VirtualService for regional routing
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: api-routing
  namespace: istio-system
spec:
  hosts:
    - api.horizonplatform.io
  gateways:
    - api-gateway
  http:
    - match:
        - headers:
            x-region:
              exact: "eu"
      route:
        - destination:
            host: api-service.horizon.svc.cluster.local
            subset: eu-west
          weight: 100
    - match:
        - headers:
            x-region:
              exact: "asia"
      route:
        - destination:
            host: api-service.horizon.svc.cluster.local
            subset: asia-pacific
          weight: 100
    - route:
        - destination:
            host: api-service.horizon.svc.cluster.local
            subset: us-west
          weight: 100
---
# Locality-aware load balancing
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: api-service
  namespace: horizon
spec:
  host: api-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 1000
      http:
        h2UpgradePolicy: UPGRADE
        maxRequestsPerConnection: 100
    loadBalancer:
      localityLbSetting:
        enabled: true
        failover:
          - from: us-west
            to: eu-west
          - from: eu-west
            to: us-west
          - from: asia-pacific
            to: us-west
    outlierDetection:
      consecutive5xxErrors: 5
      interval: 30s
      baseEjectionTime: 30s
      maxEjectionPercent: 50
  subsets:
    - name: us-west
      labels:
        region: us-west
    - name: eu-west
      labels:
        region: eu-west
    - name: asia-pacific
      labels:
        region: asia-pacific
```

### Data Residency

```go
package routing

type DataResidencyRouter struct {
    euRegion    string
    defaultDB   *sql.DB
    euDB        *sql.DB
}

// Route user data to appropriate region based on settings
func (r *DataResidencyRouter) RouteUser(ctx context.Context, userID string) (*sql.DB, error) {
    // Check user's data residency preference
    user, err := r.getUserResidency(ctx, userID)
    if err != nil {
        return nil, err
    }

    if user.DataResidency == "EU" {
        return r.euDB, nil
    }

    return r.defaultDB, nil
}

// Ensure GDPR compliance for EU users
func (r *DataResidencyRouter) EnsureEUCompliance(ctx context.Context, userID string) error {
    // Get user's country from profile
    profile, err := r.getProfile(ctx, userID)
    if err != nil {
        return err
    }

    euCountries := []string{
        "AT", "BE", "BG", "HR", "CY", "CZ", "DK", "EE", "FI", "FR",
        "DE", "GR", "HU", "IE", "IT", "LV", "LT", "LU", "MT", "NL",
        "PL", "PT", "RO", "SK", "SI", "ES", "SE",
    }

    for _, country := range euCountries {
        if profile.Country == country {
            // Ensure data is in EU region
            return r.migrateToEU(ctx, userID)
        }
    }

    return nil
}
```

### Cross-Region Replication

```go
package replication

type CrossRegionReplicator struct {
    primary    *sql.DB
    replicas   map[string]*sql.DB
    pubsub     *pubsub.Client
}

// Publish change events for cross-region sync
func (r *CrossRegionReplicator) PublishChange(ctx context.Context, change ChangeEvent) error {
    data, err := json.Marshal(change)
    if err != nil {
        return err
    }

    // Publish to all regions
    topic := r.pubsub.Topic("data-changes")
    result := topic.Publish(ctx, &pubsub.Message{
        Data: data,
        Attributes: map[string]string{
            "source_region": os.Getenv("REGION"),
            "entity_type":   change.EntityType,
            "entity_id":     change.EntityID,
        },
    })

    _, err = result.Get(ctx)
    return err
}

// Subscribe to changes from other regions
func (r *CrossRegionReplicator) SubscribeChanges(ctx context.Context) error {
    sub := r.pubsub.Subscription("data-changes-" + os.Getenv("REGION"))

    return sub.Receive(ctx, func(ctx context.Context, msg *pubsub.Message) {
        // Skip changes from our own region
        if msg.Attributes["source_region"] == os.Getenv("REGION") {
            msg.Ack()
            return
        }

        var change ChangeEvent
        if err := json.Unmarshal(msg.Data, &change); err != nil {
            log.Error("Failed to unmarshal change", "error", err)
            msg.Nack()
            return
        }

        // Apply change to local replica
        if err := r.applyChange(ctx, change); err != nil {
            log.Error("Failed to apply change", "error", err)
            msg.Nack()
            return
        }

        msg.Ack()
    })
}
```

### Failover Automation

```yaml
# Prometheus alerting rules for regional health
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: region-health
  namespace: horizon-monitoring
spec:
  groups:
    - name: region-health
      rules:
        - alert: RegionUnhealthy
          expr: |
            avg(up{job="horizon-api"}) by (region) < 0.5
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "Region {{ $labels.region }} is unhealthy"
            description: "Less than 50% of API pods are up in region {{ $labels.region }}"
```

### Disaster Recovery Runbook

```bash
#!/bin/bash
# Failover procedure for regional outage (cloud-agnostic)

FAILED_REGION=$1
FAILOVER_REGION=$2

echo "Initiating failover from $FAILED_REGION to $FAILOVER_REGION"

# 1. Update ingress weights (via ExternalDNS or manual)
# Option A: Cloudflare
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
  -H "Authorization: Bearer $CF_TOKEN" \
  -H "Content-Type: application/json" \
  --data "{\"content\":\"$FAILOVER_LB_IP\"}"

# Option B: AWS Route53
# aws route53 change-resource-record-sets ...

# 2. Promote PostgreSQL replica (using Patroni)
kubectl --context $FAILOVER_REGION exec -n horizon-data postgresql-0 -- \
  patronictl switchover --force

# 3. Scale up failover region
kubectl config use-context $FAILOVER_REGION
kubectl scale deployment --all --replicas=2 -n horizon
kubectl scale deployment --all --replicas=2 -n horizon-workspaces

# 4. Notify status page
curl -X POST https://api.statuspage.io/v1/pages/$PAGE_ID/incidents \
  -H "Authorization: OAuth $STATUSPAGE_TOKEN" \
  -d '{"incident": {"name": "Regional Failover", "status": "investigating"}}'

echo "Failover complete. Monitor dashboards for recovery."
```

## Latency Targets

| Region | Target Latency | Measured P99 |
|--------|----------------|--------------|
| US West Coast | <50ms | ~35ms |
| US East Coast | <80ms | ~65ms |
| Europe | <50ms | ~40ms |
| Asia Pacific | <80ms | ~70ms |

## References

- [Kubernetes Multi-Cluster Management](https://kubernetes.io/docs/concepts/cluster-administration/federation/)
- [PostgreSQL Streaming Replication](https://www.postgresql.org/docs/current/warm-standby.html)
- [Patroni High Availability](https://patroni.readthedocs.io/)
- [Istio Locality Load Balancing](https://istio.io/latest/docs/tasks/traffic-management/locality-load-balancing/)
- [GDPR Data Residency](https://gdpr.eu/what-is-gdpr/)
