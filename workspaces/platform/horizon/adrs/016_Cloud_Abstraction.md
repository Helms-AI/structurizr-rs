# ADR-016: Cloud Provider Abstraction (GCP to Generic Kubernetes)

## Status

**Accepted**

## Date

2024-12-31

## Context

The original Horizon Platform (formerly Replit Clone) architecture was designed specifically for Google Cloud Platform (GCP), utilizing:

- **GKE** (Google Kubernetes Engine) for container orchestration
- **Cloud SQL** for managed PostgreSQL databases
- **Memorystore** for managed Redis
- **Cloud Storage (GCS)** for object storage
- **Cloud CDN** for content delivery
- **Cloud Load Balancer** for traffic management

### Problems with GCP-Specific Design

1. **Vendor Lock-in**: Tightly coupled to GCP services
2. **Cost Constraints**: GCP pricing may not be optimal for all scales
3. **Geographic Limitations**: GCP regions may not cover all deployment needs
4. **Self-Hosting Impossible**: Cannot deploy on-premises or other clouds
5. **Migration Difficulty**: Moving to another cloud requires significant rework

### Requirements

1. Deploy on any CNCF-conformant Kubernetes cluster
2. Support cloud providers: AWS, Azure, GCP, DigitalOcean, Linode
3. Support on-premises: bare metal, OpenStack, VMware
4. Use open-source, self-managed data services
5. Maintain equivalent functionality and performance

## Decision

We will abstract the infrastructure to be **cloud-agnostic**, using generic Kubernetes resources and open-source equivalents for managed services.

### Service Mapping

| GCP Service | Replacement | Deployment |
|-------------|-------------|------------|
| GKE | Generic Kubernetes | Any K8s (EKS, AKS, bare metal) |
| Cloud SQL | PostgreSQL | Bitnami Helm / CrunchyData Operator |
| Memorystore | Redis | Bitnami Helm + Sentinel |
| Cloud Storage | MinIO | Self-hosted, S3-compatible |
| Cloud CDN | Nginx Ingress + Cache | In-cluster or external CDN |
| Cloud Load Balancer | Nginx Ingress / Traefik | Ingress Controller |
| Cloud DNS | ExternalDNS | Works with any DNS provider |
| Cloud Armor (WAF) | ModSecurity | Nginx Ingress module |
| Secret Manager | External Secrets Operator | With Vault or cloud backends |

### Alternatives Considered

1. **Multi-Cloud Abstraction (Crossplane)**: Too complex for current needs
2. **AWS-Specific**: Same vendor lock-in problem
3. **Keep GCP**: Limits deployment options
4. **Hybrid**: Increases complexity without full benefits

## Implementation

### Kubernetes Deployment Architecture

```yaml
# Base Kustomization
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

namespace: horizon

resources:
  # Ingress
  - ingress/nginx-ingress.yaml
  - ingress/cert-manager.yaml
  - ingress/external-dns.yaml

  # Application Services
  - services/api-gateway.yaml
  - services/websocket-gateway.yaml
  - services/workspace-service.yaml
  - services/container-orchestrator.yaml
  - services/collaboration-engine.yaml
  - services/ai-gateway.yaml

  # Data Services (via Helm)
  - data/postgresql-helm.yaml
  - data/redis-helm.yaml
  - data/minio-helm.yaml
  - data/qdrant-helm.yaml
  - data/kafka-helm.yaml

  # Monitoring
  - monitoring/prometheus-stack.yaml
```

### PostgreSQL Deployment

```yaml
# Using Bitnami Helm Chart
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: postgresql
  namespace: horizon
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
          memory: 2Gi
          cpu: 1000m
    readReplicas:
      replicaCount: 2
    metrics:
      enabled: true
```

### MinIO for Object Storage

```yaml
# S3-Compatible Object Storage
apiVersion: helm.toolkit.fluxcd.io/v2beta1
kind: HelmRelease
metadata:
  name: minio
  namespace: horizon
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

## Consequences

### Positive

1. **No vendor lock-in**: Deploy anywhere Kubernetes runs
2. **Cost flexibility**: Choose providers based on cost/performance
3. **Self-hosting**: Enable on-premises deployments
4. **Multi-cloud**: Easy to deploy across multiple providers
5. **Kubernetes ecosystem**: Leverage vast K8s tooling

### Negative

1. **Operational complexity**: Need to manage data services
2. **No managed services**: Lose GCP's operational convenience
3. **Learning curve**: Team needs K8s operations skills
4. **Initial setup**: More upfront configuration

### Mitigations

| Risk | Mitigation |
|------|------------|
| Operational complexity | Use Helm charts and operators |
| No managed services | Implement robust backup/restore |
| Learning curve | Documentation and training |
| Initial setup | Infrastructure as Code (Terraform/Pulumi) |

## Runbook Changes

All `gcloud` commands in the runbook should be replaced with `kubectl` and `helm`:

| Old (GCP) | New (K8s) |
|-----------|-----------|
| `gcloud container clusters resize` | `kubectl scale deployment` |
| `gcloud sql backups create` | `kubectl exec pg_dump` / Velero |
| `gcloud dns record-sets update` | ExternalDNS annotations |
| `gcloud container clusters upgrade` | Provider-specific or kubeadm |

## References

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Bitnami Helm Charts](https://github.com/bitnami/charts)
- [MinIO Documentation](https://min.io/docs/)
- [cert-manager Documentation](https://cert-manager.io/docs/)
- [ExternalDNS](https://github.com/kubernetes-sigs/external-dns)
