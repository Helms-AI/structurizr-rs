# ADR-011: Preview and Deployment System

## Status

Accepted

## Context

The platform must provide instant preview URLs for web applications and support deployment to production. Requirements include:

- **Instant Preview**: See changes immediately during development
- **Unique URLs**: Each workspace gets a dedicated subdomain
- **HTTPS**: All previews served over TLS
- **Custom Domains**: Users can connect their own domains
- **Zero-Config Deploy**: Deploy without infrastructure knowledge
- **Scaling**: Handle traffic spikes automatically

**Use Cases:**
- Web server preview during development
- API endpoint testing
- Static site hosting
- Full-stack application deployment

## Decision

We will implement a **container-based preview and deployment system** with:

1. **Instant Previews**: Reverse proxy to workspace containers
2. **Wildcard Subdomains**: `{workspace-id}.preview.horizonplatform.io`
3. **Deployment Service**: Container orchestration for production
4. **Custom Domains**: CNAME validation and certificate provisioning
5. **Auto-Scaling**: HPA based on traffic

## Alternatives Considered

### Serverless Functions Only (AWS Lambda style)

**Pros:**
- True zero-config scaling
- Pay per invocation
- No container management

**Cons:**
- Cold start latency
- Limited runtime environments
- File system constraints
- Complex local development

**Why Rejected:** Full container support needed for development experience.

### Static Site Generators Only (Netlify style)

**Pros:**
- Fast deployments
- Global CDN
- Simple pricing

**Cons:**
- No server-side code
- Limited to static output
- No database access

**Why Rejected:** Need full application hosting capability.

### Kubernetes Namespace per Deployment

**Pros:**
- Strong isolation
- Native K8s tooling
- Resource quotas per namespace

**Cons:**
- Namespace overhead
- Complex cleanup
- Slower provisioning

**Why Rejected:** Single namespace with labels more efficient at scale.

## Consequences

### Positive

- **Instant feedback**: Preview updates in <1s
- **Zero-config**: No deployment configuration needed
- **Flexibility**: Supports any web framework
- **Isolation**: Each deployment in separate container
- **Scalability**: Auto-scales based on traffic

### Negative

- **Cost**: Always-on containers for deployments
- **Cold starts**: Scaled-to-zero deployments have startup time
- **Complexity**: Custom domain certificate management
- **Resource limits**: Can't handle unlimited traffic

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Cost | Sleep inactive deployments, spot instances |
| Cold starts | Keep minimum replica, prewarming |
| Complexity | Automated cert-manager, DNS automation |
| Resource limits | Clear tier limits, upgrade paths |

## Implementation

### Preview URL Architecture

```
┌─────────────┐     ┌─────────────────┐     ┌───────────────────┐
│   Browser   │────▶│  Ingress/Envoy  │────▶│ Workspace Container│
│             │     │                 │     │                   │
│ abc123.prev │     │ Route by Host   │     │  Port 3000        │
│ iew.horizon │     │                 │     │  (user app)       │
│ platform.io │     │ TLS Termination │     │                   │
└─────────────┘     └─────────────────┘     └───────────────────┘
```

### Preview Service

```go
package preview

type PreviewService struct {
    k8s           kubernetes.Interface
    certManager   *certmanager.Client
    dnsProvider   DNSProvider
    routingTable  *RoutingTable
}

type PreviewConfig struct {
    WorkspaceID  string
    Port         int
    Protocol     string // http, https, grpc
    HealthPath   string
}

func (s *PreviewService) CreatePreview(ctx context.Context, cfg PreviewConfig) (*Preview, error) {
    // Generate preview URL
    subdomain := cfg.WorkspaceID
    host := fmt.Sprintf("%s.preview.horizonplatform.io", subdomain)

    // Create Kubernetes Service
    svc := &corev1.Service{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("preview-%s", cfg.WorkspaceID),
            Namespace: "workspaces",
            Labels: map[string]string{
                "app":         "preview",
                "workspaceId": cfg.WorkspaceID,
            },
        },
        Spec: corev1.ServiceSpec{
            Selector: map[string]string{
                "workspaceId": cfg.WorkspaceID,
            },
            Ports: []corev1.ServicePort{
                {
                    Port:       80,
                    TargetPort: intstr.FromInt(cfg.Port),
                },
            },
        },
    }

    if _, err := s.k8s.CoreV1().Services("workspaces").Create(ctx, svc, metav1.CreateOptions{}); err != nil {
        return nil, err
    }

    // Create Ingress with TLS
    ingress := &networkingv1.Ingress{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("preview-%s", cfg.WorkspaceID),
            Namespace: "workspaces",
            Annotations: map[string]string{
                "kubernetes.io/ingress.class":                "nginx",
                "cert-manager.io/cluster-issuer":             "letsencrypt-prod",
                "nginx.ingress.kubernetes.io/proxy-body-size": "50m",
                "nginx.ingress.kubernetes.io/websocket-services": svc.Name,
            },
        },
        Spec: networkingv1.IngressSpec{
            TLS: []networkingv1.IngressTLS{
                {
                    Hosts:      []string{host},
                    SecretName: fmt.Sprintf("preview-%s-tls", cfg.WorkspaceID),
                },
            },
            Rules: []networkingv1.IngressRule{
                {
                    Host: host,
                    IngressRuleValue: networkingv1.IngressRuleValue{
                        HTTP: &networkingv1.HTTPIngressRuleValue{
                            Paths: []networkingv1.HTTPIngressPath{
                                {
                                    Path:     "/",
                                    PathType: ptr(networkingv1.PathTypePrefix),
                                    Backend: networkingv1.IngressBackend{
                                        Service: &networkingv1.IngressServiceBackend{
                                            Name: svc.Name,
                                            Port: networkingv1.ServiceBackendPort{
                                                Number: 80,
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    }

    if _, err := s.k8s.NetworkingV1().Ingresses("workspaces").Create(ctx, ingress, metav1.CreateOptions{}); err != nil {
        return nil, err
    }

    // Update routing table
    s.routingTable.Add(host, cfg.WorkspaceID)

    return &Preview{
        URL:         fmt.Sprintf("https://%s", host),
        WorkspaceID: cfg.WorkspaceID,
        Port:        cfg.Port,
        Status:      PreviewStatusActive,
    }, nil
}
```

### Deployment Service

```go
package deployment

type DeploymentService struct {
    k8s           kubernetes.Interface
    registry      *ContainerRegistry
    domainManager *DomainManager
    metrics       *prometheus.Registry
}

type DeploymentConfig struct {
    WorkspaceID    string
    Name           string
    Environment    string // production, staging
    Replicas       int32
    Resources      ResourceSpec
    CustomDomain   *string
    EnvVars        map[string]string
    HealthCheck    *HealthCheckConfig
}

func (s *DeploymentService) Deploy(ctx context.Context, cfg DeploymentConfig) (*Deployment, error) {
    // Build and push container image
    imageTag := fmt.Sprintf("registry.horizonplatform.io/deployments/%s:%s",
        cfg.WorkspaceID, time.Now().Format("20060102-150405"))

    if err := s.registry.BuildAndPush(ctx, cfg.WorkspaceID, imageTag); err != nil {
        return nil, fmt.Errorf("failed to build image: %w", err)
    }

    // Create deployment
    deployment := &appsv1.Deployment{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("deploy-%s", cfg.Name),
            Namespace: "deployments",
            Labels: map[string]string{
                "app":         cfg.Name,
                "workspaceId": cfg.WorkspaceID,
                "environment": cfg.Environment,
            },
        },
        Spec: appsv1.DeploymentSpec{
            Replicas: &cfg.Replicas,
            Selector: &metav1.LabelSelector{
                MatchLabels: map[string]string{
                    "app": cfg.Name,
                },
            },
            Template: corev1.PodTemplateSpec{
                ObjectMeta: metav1.ObjectMeta{
                    Labels: map[string]string{
                        "app":         cfg.Name,
                        "workspaceId": cfg.WorkspaceID,
                    },
                },
                Spec: corev1.PodSpec{
                    Containers: []corev1.Container{
                        {
                            Name:  "app",
                            Image: imageTag,
                            Ports: []corev1.ContainerPort{
                                {ContainerPort: 8080},
                            },
                            Env:       s.buildEnvVars(cfg.EnvVars),
                            Resources: s.buildResources(cfg.Resources),
                            LivenessProbe:  s.buildProbe(cfg.HealthCheck),
                            ReadinessProbe: s.buildProbe(cfg.HealthCheck),
                        },
                    },
                },
            },
            Strategy: appsv1.DeploymentStrategy{
                Type: appsv1.RollingUpdateDeploymentStrategyType,
                RollingUpdate: &appsv1.RollingUpdateDeployment{
                    MaxSurge:       &intstr.IntOrString{IntVal: 1},
                    MaxUnavailable: &intstr.IntOrString{IntVal: 0},
                },
            },
        },
    }

    if _, err := s.k8s.AppsV1().Deployments("deployments").Create(ctx, deployment, metav1.CreateOptions{}); err != nil {
        return nil, err
    }

    // Create HPA for auto-scaling
    hpa := &autoscalingv2.HorizontalPodAutoscaler{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("deploy-%s-hpa", cfg.Name),
            Namespace: "deployments",
        },
        Spec: autoscalingv2.HorizontalPodAutoscalerSpec{
            ScaleTargetRef: autoscalingv2.CrossVersionObjectReference{
                APIVersion: "apps/v1",
                Kind:       "Deployment",
                Name:       deployment.Name,
            },
            MinReplicas: ptr(int32(1)),
            MaxReplicas: 10,
            Metrics: []autoscalingv2.MetricSpec{
                {
                    Type: autoscalingv2.ResourceMetricSourceType,
                    Resource: &autoscalingv2.ResourceMetricSource{
                        Name: corev1.ResourceCPU,
                        Target: autoscalingv2.MetricTarget{
                            Type:               autoscalingv2.UtilizationMetricType,
                            AverageUtilization: ptr(int32(70)),
                        },
                    },
                },
            },
        },
    }

    s.k8s.AutoscalingV2().HorizontalPodAutoscalers("deployments").Create(ctx, hpa, metav1.CreateOptions{})

    // Setup domain
    domain := fmt.Sprintf("%s.horizonplatform.app", cfg.Name)
    if cfg.CustomDomain != nil {
        domain = *cfg.CustomDomain
        if err := s.domainManager.SetupCustomDomain(ctx, *cfg.CustomDomain, deployment.Name); err != nil {
            return nil, fmt.Errorf("failed to setup custom domain: %w", err)
        }
    }

    return &Deployment{
        ID:          uuid.New().String(),
        Name:        cfg.Name,
        WorkspaceID: cfg.WorkspaceID,
        URL:         fmt.Sprintf("https://%s", domain),
        Status:      DeploymentStatusDeploying,
        CreatedAt:   time.Now(),
    }, nil
}
```

### Custom Domain Manager

```go
package domains

type DomainManager struct {
    k8s          kubernetes.Interface
    certManager  *certmanager.Client
    cloudflare   *cloudflare.Client
}

type CustomDomainConfig struct {
    Domain       string
    DeploymentID string
    UserID       string
}

func (m *DomainManager) SetupCustomDomain(ctx context.Context, domain string, deploymentName string) error {
    // Verify domain ownership via DNS TXT record
    if err := m.verifyDomainOwnership(ctx, domain); err != nil {
        return fmt.Errorf("domain verification failed: %w", err)
    }

    // Create Certificate resource
    cert := &certmanagerv1.Certificate{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("custom-%s", strings.ReplaceAll(domain, ".", "-")),
            Namespace: "deployments",
        },
        Spec: certmanagerv1.CertificateSpec{
            SecretName: fmt.Sprintf("custom-%s-tls", strings.ReplaceAll(domain, ".", "-")),
            DNSNames:   []string{domain},
            IssuerRef: cmmeta.ObjectReference{
                Name: "letsencrypt-prod",
                Kind: "ClusterIssuer",
            },
        },
    }

    if _, err := m.certManager.CertmanagerV1().Certificates("deployments").Create(ctx, cert, metav1.CreateOptions{}); err != nil {
        return err
    }

    // Create Ingress for custom domain
    ingress := &networkingv1.Ingress{
        ObjectMeta: metav1.ObjectMeta{
            Name:      fmt.Sprintf("custom-%s", strings.ReplaceAll(domain, ".", "-")),
            Namespace: "deployments",
            Annotations: map[string]string{
                "kubernetes.io/ingress.class":    "nginx",
                "cert-manager.io/cluster-issuer": "letsencrypt-prod",
            },
        },
        Spec: networkingv1.IngressSpec{
            TLS: []networkingv1.IngressTLS{
                {
                    Hosts:      []string{domain},
                    SecretName: cert.Spec.SecretName,
                },
            },
            Rules: []networkingv1.IngressRule{
                {
                    Host: domain,
                    IngressRuleValue: networkingv1.IngressRuleValue{
                        HTTP: &networkingv1.HTTPIngressRuleValue{
                            Paths: []networkingv1.HTTPIngressPath{
                                {
                                    Path:     "/",
                                    PathType: ptr(networkingv1.PathTypePrefix),
                                    Backend: networkingv1.IngressBackend{
                                        Service: &networkingv1.IngressServiceBackend{
                                            Name: deploymentName,
                                            Port: networkingv1.ServiceBackendPort{
                                                Number: 80,
                                            },
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
            },
        },
    }

    _, err := m.k8s.NetworkingV1().Ingresses("deployments").Create(ctx, ingress, metav1.CreateOptions{})
    return err
}

func (m *DomainManager) verifyDomainOwnership(ctx context.Context, domain string) error {
    // Check for TXT record with verification token
    expectedValue := fmt.Sprintf("horizon-verify=%s", hashDomain(domain))

    records, err := net.LookupTXT(fmt.Sprintf("_horizon.%s", domain))
    if err != nil {
        return fmt.Errorf("DNS lookup failed: %w", err)
    }

    for _, record := range records {
        if record == expectedValue {
            return nil
        }
    }

    return fmt.Errorf("verification TXT record not found. Add TXT record '_horizon.%s' with value '%s'", domain, expectedValue)
}
```

### Deployment Dockerfile Generator

```go
package deployment

func (s *DeploymentService) GenerateDockerfile(workspace *Workspace) (string, error) {
    // Detect project type
    projectType := s.detectProjectType(workspace)

    templates := map[string]string{
        "nodejs": `FROM node:20-slim
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY . .
EXPOSE 8080
CMD ["npm", "start"]
`,
        "python": `FROM python:3.11-slim
WORKDIR /app
COPY requirements.txt ./
RUN pip install --no-cache-dir -r requirements.txt
COPY . .
EXPOSE 8080
CMD ["python", "main.py"]
`,
        "go": `FROM golang:1.22-alpine AS builder
WORKDIR /app
COPY go.mod go.sum ./
RUN go mod download
COPY . .
RUN CGO_ENABLED=0 go build -o main .

FROM alpine:3.19
WORKDIR /app
COPY --from=builder /app/main .
EXPOSE 8080
CMD ["./main"]
`,
        "static": `FROM nginx:alpine
COPY . /usr/share/nginx/html
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
`,
    }

    template, ok := templates[projectType]
    if !ok {
        return "", fmt.Errorf("unsupported project type: %s", projectType)
    }

    return template, nil
}

func (s *DeploymentService) detectProjectType(workspace *Workspace) string {
    files := workspace.ListFiles()

    // Check for project files
    fileChecks := map[string]string{
        "package.json":      "nodejs",
        "requirements.txt":  "python",
        "go.mod":           "go",
        "Cargo.toml":       "rust",
        "index.html":       "static",
    }

    for file, projectType := range fileChecks {
        if contains(files, file) {
            return projectType
        }
    }

    return "static"
}
```

### Deployment Tiers

| Tier | Replicas | CPU | Memory | Custom Domain | Price |
|------|----------|-----|--------|---------------|-------|
| Free | 0-1 (scale to zero) | 0.1 | 256Mi | No | $0 |
| Hacker | 1-3 | 0.5 | 1Gi | Yes (1) | $7/mo |
| Pro | 1-10 | 2 | 4Gi | Yes (5) | $20/mo |
| Teams | 1-50 | 4 | 8Gi | Yes (∞) | Custom |

## References

- [Kubernetes Ingress](https://kubernetes.io/docs/concepts/services-networking/ingress/)
- [cert-manager](https://cert-manager.io/docs/)
- [Cloud Build](https://cloud.google.com/build/docs)
- [Horizontal Pod Autoscaling](https://kubernetes.io/docs/tasks/run-application/horizontal-pod-autoscale/)
