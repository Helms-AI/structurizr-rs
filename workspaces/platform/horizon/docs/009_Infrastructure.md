# Infrastructure & Deployment

## Overview

The Horizon Platform runs on cloud-agnostic Kubernetes with a multi-region deployment for high availability and low latency. The infrastructure is designed to work with any CNCF-conformant Kubernetes cluster.

## Cloud Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     Cloud Provider (AWS/GCP/Azure/On-Premises)          │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                     Ingress Controller (Nginx)                      │ │
│  │                  cert-manager | ModSecurity | SSL                   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                              │                                           │
│        ┌────────────────────┼────────────────────┐                      │
│        ▼                    ▼                    ▼                      │
│  ┌──────────┐         ┌──────────┐         ┌──────────┐                │
│  │Region 1  │         │Region 2  │         │Region 3  │                │
│  │(Primary) │         │(Secondary)│         │(DR)      │                │
│  └──────────┘         └──────────┘         └──────────┘                │
│       │                    │                    │                       │
│       ▼                    ▼                    ▼                       │
│  ┌──────────┐         ┌──────────┐         ┌──────────┐                │
│  │K8s Cluster│         │K8s Cluster│         │K8s Cluster│               │
│  │  + Pools  │         │  + Pools  │         │  + Pools  │               │
│  └──────────┘         └──────────┘         └──────────┘                │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      Data Services (Self-Hosted)                    │ │
│  │  PostgreSQL | Redis | MinIO | Qdrant | NATS | Elasticsearch        │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

## Kubernetes Cluster

### Node Pools

| Pool | Machine Type | Nodes | Purpose |
|------|--------------|-------|---------|
| api | 8 vCPU, 16GB | 3-20 | API Gateway, services |
| core | 16 vCPU, 32GB | 5-50 | Container orchestrator, file system |
| ai | 8 vCPU, 32GB | 2-20 | AI services |
| collab | 8 vCPU, 16GB | 3-30 | Collaboration engine |
| runtime | 4 vCPU, 8GB | 10-500 | User workspaces (gVisor) |

### Cluster Configuration

```yaml
# Kubernetes cluster configuration (via Helm/Kustomize)
apiVersion: v1
kind: Namespace
metadata:
  name: horizon
  labels:
    app.kubernetes.io/name: horizon
    app.kubernetes.io/part-of: horizon-platform
---
# Ingress Controller
apiVersion: networking.k8s.io/v1
kind: IngressClass
metadata:
  name: nginx
  annotations:
    ingressclass.kubernetes.io/is-default-class: "true"
spec:
  controller: k8s.io/ingress-nginx
```

### Namespace Structure

```
├── horizon           # Core application services
├── horizon-ai        # AI Gateway, Agent Orchestrator
├── horizon-collab    # Collaboration Engine
├── horizon-data      # Databases, caches (PostgreSQL, Redis, etc.)
├── horizon-monitoring # Prometheus, Grafana, Jaeger
├── cert-manager      # TLS certificate management
└── horizon-workspaces # User container namespaces
```

## Database Infrastructure

### PostgreSQL (Bitnami Helm Chart)

```yaml
# PostgreSQL via Helm
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: postgresql
  namespace: horizon-data
spec:
  interval: 5m
  chart:
    spec:
      chart: postgresql
      version: "13.x.x"
      sourceRef:
        kind: HelmRepository
        name: bitnami
  values:
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
```

### Redis (Bitnami Helm Chart + Sentinel)

```yaml
# Redis via Helm
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: redis
  namespace: horizon-data
spec:
  interval: 5m
  chart:
    spec:
      chart: redis
      version: "18.x.x"
      sourceRef:
        kind: HelmRepository
        name: bitnami
  values:
    architecture: replication
    sentinel:
      enabled: true
    master:
      persistence:
        size: 32Gi
        storageClass: fast-ssd
    replica:
      replicaCount: 2
    metrics:
      enabled: true
```

### NATS Cluster (JetStream)

```yaml
# NATS via Helm (replaces Kafka for event streaming)
# See ADR-018 for migration details
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
        size: 100Gi
        storageClassName: fast-ssd
    websocket:
      enabled: true
      port: 9222
    resources:
      requests:
        memory: 1Gi
        cpu: 500m
      limits:
        memory: 4Gi
        cpu: 2000m
    metrics:
      enabled: true
      port: 7777
```

### Object Storage (MinIO)

```yaml
# MinIO for S3-compatible storage
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: minio
  namespace: horizon-data
spec:
  interval: 5m
  chart:
    spec:
      chart: minio
      version: "5.x.x"
      sourceRef:
        kind: HelmRepository
        name: minio
  values:
    mode: distributed
    replicas: 4
    persistence:
      size: 500Gi
      storageClass: fast-ssd
    resources:
      requests:
        memory: 4Gi
        cpu: 2000m
    ingress:
      enabled: true
      hosts:
        - storage.horizonplatform.io
```

### Vector Database (Qdrant)

```yaml
# Qdrant for AI embeddings
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: qdrant
  namespace: horizon-data
spec:
  interval: 5m
  chart:
    spec:
      chart: qdrant
      version: "0.x.x"
      sourceRef:
        kind: HelmRepository
        name: qdrant
  values:
    replicaCount: 3
    persistence:
      size: 100Gi
      storageClass: fast-ssd
    resources:
      requests:
        memory: 4Gi
        cpu: 2000m
```

## Networking

### Ingress Controller

```yaml
# Nginx Ingress with cert-manager
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: horizon-ingress
  namespace: horizon
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/proxy-body-size: "100m"
spec:
  tls:
    - hosts:
        - horizonplatform.io
        - "*.horizonplatform.io"
      secretName: horizon-tls
  rules:
    - host: horizonplatform.io
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: web-ide
                port:
                  number: 80
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

### TLS Certificate Management

```yaml
# cert-manager ClusterIssuer
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@horizonplatform.io
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
      - http01:
          ingress:
            class: nginx
```

### DNS Configuration (ExternalDNS)

```yaml
# ExternalDNS for automatic DNS management
apiVersion: apps/v1
kind: Deployment
metadata:
  name: external-dns
  namespace: horizon
spec:
  template:
    spec:
      containers:
        - name: external-dns
          image: registry.k8s.io/external-dns/external-dns:latest
          args:
            - --source=ingress
            - --domain-filter=horizonplatform.io
            - --provider=cloudflare  # or aws, google, azure, etc.
```

## Observability

### Prometheus + Grafana (kube-prometheus-stack)

```yaml
# Prometheus Stack via Helm
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: kube-prometheus-stack
  namespace: horizon-monitoring
spec:
  interval: 5m
  chart:
    spec:
      chart: kube-prometheus-stack
      sourceRef:
        kind: HelmRepository
        name: prometheus-community
  values:
    prometheus:
      prometheusSpec:
        retention: 30d
        storageSpec:
          volumeClaimTemplate:
            spec:
              storageClassName: fast-ssd
              resources:
                requests:
                  storage: 500Gi
    grafana:
      enabled: true
      ingress:
        enabled: true
        hosts:
          - grafana.horizonplatform.io
```

### Key Metrics

| Metric | Description | Alert Threshold |
|--------|-------------|-----------------|
| `api_request_duration_seconds` | API latency | P99 > 1s |
| `container_cold_start_seconds` | Container startup | P95 > 5s |
| `collaboration_sync_latency_ms` | Collab sync time | P99 > 200ms |
| `ai_request_duration_seconds` | AI response time | P99 > 10s |
| `error_rate` | 5xx errors | > 1% |
| `pod_restart_count` | Pod restarts | > 5/hour |

### Alerting Rules

```yaml
# Alerting rules
groups:
  - name: horizon
    rules:
      - alert: HighErrorRate
        expr: |
          sum(rate(http_requests_total{status=~"5.."}[5m])) /
          sum(rate(http_requests_total[5m])) > 0.01
        for: 5m
        labels:
          severity: critical
        annotations:
          summary: High error rate detected

      - alert: ContainerStartupSlow
        expr: |
          histogram_quantile(0.95,
            rate(container_cold_start_seconds_bucket[5m])
          ) > 5
        for: 10m
        labels:
          severity: warning
        annotations:
          summary: Container cold starts are slow

      - alert: DatabaseConnectionsHigh
        expr: |
          pg_stat_activity_count > 800
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: Database connections approaching limit
```

### Distributed Tracing (Jaeger)

```yaml
# Jaeger deployment
apiVersion: jaegertracing.io/v1
kind: Jaeger
metadata:
  name: horizon-tracing
  namespace: horizon-monitoring
spec:
  strategy: production
  storage:
    type: elasticsearch
    options:
      es:
        server-urls: http://elasticsearch:9200
  ingress:
    enabled: true
```

## CI/CD Pipeline

### GitOps with ArgoCD

```yaml
# ArgoCD Application
apiVersion: argoproj.io/v1alpha1
kind: Application
metadata:
  name: horizon
spec:
  project: default
  source:
    repoURL: https://github.com/horizon/infra
    targetRevision: HEAD
    path: kubernetes/overlays/production
  destination:
    server: https://kubernetes.default.svc
    namespace: horizon
  syncPolicy:
    automated:
      prune: true
      selfHeal: true
    syncOptions:
      - CreateNamespace=true
```

### Deployment Strategy

```yaml
# Canary deployment with Flagger
apiVersion: flagger.app/v1beta1
kind: Canary
metadata:
  name: api-gateway
  namespace: horizon
spec:
  targetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  progressDeadlineSeconds: 600
  service:
    port: 80
    targetPort: 8080
  analysis:
    interval: 1m
    threshold: 5
    maxWeight: 50
    stepWeight: 10
    metrics:
      - name: request-success-rate
        thresholdRange:
          min: 99
        interval: 1m
      - name: request-duration
        thresholdRange:
          max: 500
        interval: 1m
```

## Autoscaling

### Horizontal Pod Autoscaler

```yaml
apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: api-gateway
  namespace: horizon
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: api-gateway
  minReplicas: 3
  maxReplicas: 50
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Pods
      pods:
        metric:
          name: http_requests_per_second
        target:
          type: AverageValue
          averageValue: "1000"
  behavior:
    scaleDown:
      stabilizationWindowSeconds: 300
      policies:
        - type: Percent
          value: 10
          periodSeconds: 60
    scaleUp:
      stabilizationWindowSeconds: 0
      policies:
        - type: Percent
          value: 100
          periodSeconds: 15
```

## Disaster Recovery

### RTO/RPO Targets

| Service | RTO | RPO |
|---------|-----|-----|
| API | 5 min | 0 |
| Workspaces | 15 min | 5 min |
| User Data | 1 hour | 1 min |
| Analytics | 4 hours | 1 hour |

### Failover Procedure

1. **Detection**: Automated health checks detect failure
2. **DNS Failover**: Traffic redirected to healthy region
3. **Database Promotion**: Read replica promoted to primary
4. **Cache Warm-up**: Pre-populate cache from backup
5. **Verification**: Run smoke tests
6. **Communication**: Update status page

### Backup Strategy

```yaml
# Velero for Kubernetes backup
apiVersion: velero.io/v1
kind: Schedule
metadata:
  name: daily-backup
  namespace: velero
spec:
  schedule: "0 2 * * *"
  template:
    includedNamespaces:
      - horizon
      - horizon-data
    storageLocation: default
    ttl: 720h  # 30 days
```

## Local Development

For local development, use the included docker-compose configuration:

```bash
# Start all infrastructure services
cd workspaces/platform/horizon
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

See `docker-compose.yml` for the complete local development environment.
