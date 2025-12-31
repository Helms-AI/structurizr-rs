# ADR-001: Container Orchestration Strategy

## Status

Accepted

## Context

The Horizon Platform must run isolated, ephemeral development environments for thousands of concurrent users. Each workspace requires:

- Secure isolation from other workspaces
- Access to a full Linux environment
- Persistent storage for code and data
- Network access for package installation
- Resource limits to prevent abuse

**Options Considered:**

1. **Docker containers on Kubernetes**
   - Mature ecosystem, widely understood
   - Good performance
   - Limited security isolation (shared kernel)

2. **Firecracker microVMs**
   - Strong isolation (separate kernels)
   - Fast boot times (~125ms)
   - More complex orchestration

3. **gVisor sandboxed containers**
   - User-space kernel intercepts syscalls
   - Defense in depth with containers
   - Compatible with Kubernetes

4. **Kata Containers**
   - VM-based isolation with container UX
   - Heavier than gVisor
   - Strong security

## Decision

We will use **Kubernetes with gVisor (runsc) runtime** for workspace container orchestration.

**Rationale:**

1. **Security**: gVisor provides an additional security layer by intercepting syscalls in user space, reducing the kernel attack surface. This is critical for running untrusted user code.

2. **Kubernetes Integration**: gVisor integrates seamlessly with Kubernetes via RuntimeClass, allowing us to use standard Kubernetes tooling, monitoring, and scaling.

3. **Performance Balance**: While gVisor has ~10-20% overhead compared to native containers, this is acceptable for development workloads and much lighter than full VMs.

4. **Ecosystem**: Leverage existing Kubernetes ecosystem (Prometheus, Istio, ArgoCD) without building custom orchestration.

5. **Operational Simplicity**: Platform team already has Kubernetes expertise; no need to learn new orchestration systems.

## Alternatives Considered

### Firecracker

**Pros:**
- Stronger isolation (separate kernel per workspace)
- Sub-second boot times
- Used by AWS Lambda and Fly.io

**Cons:**
- Requires custom orchestration layer
- Less mature Kubernetes integration
- More complex networking setup
- Higher memory overhead per workspace

**Why Rejected:** The operational complexity of building custom orchestration outweighs the marginal security benefits for our use case.

### Docker without gVisor

**Pros:**
- Best performance
- Simplest setup
- Most tooling support

**Cons:**
- Shared kernel with all containers
- History of container escape vulnerabilities
- Insufficient for running arbitrary user code

**Why Rejected:** Security is paramount when running untrusted code. The performance trade-off is acceptable.

### Kata Containers

**Pros:**
- VM-level isolation
- Container-like UX
- Cloud Hypervisor support

**Cons:**
- Higher resource overhead than gVisor
- Slower startup times
- More complex nested virtualization on GCP

**Why Rejected:** gVisor provides sufficient isolation with better performance characteristics.

## Consequences

### Positive

- Strong security isolation for user workspaces
- Standard Kubernetes deployment and monitoring
- Automatic scaling with HPA
- Pod disruption budgets for availability
- Network policies for egress control
- Resource quotas per namespace

### Negative

- 10-20% performance overhead from gVisor
- Some syscalls not supported by gVisor
- Debugging more complex (two layers: container + gVisor)
- GPU workloads may not work with gVisor
- Need dedicated node pools with gVisor runtime

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Performance overhead | Use warm pool to hide cold start |
| Unsupported syscalls | Document limitations, provide alternatives |
| Debugging complexity | Enhanced logging, remote debugging support |
| GPU workloads | Separate node pool without gVisor for ML |

## Implementation

### RuntimeClass Configuration

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: gvisor
handler: runsc
scheduling:
  nodeSelector:
    runtime: gvisor
```

### Pod Security Standards

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: workspace
spec:
  runtimeClassName: gvisor
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
    - name: workspace
      securityContext:
        allowPrivilegeEscalation: false
        capabilities:
          drop:
            - ALL
        readOnlyRootFilesystem: false
```

### Node Pool Configuration

For cloud providers with native gVisor support:
- **GKE**: Use `--sandbox type=gvisor` when creating node pools
- **EKS**: Deploy gVisor as a DaemonSet with containerd configuration
- **AKS**: Configure containerd with runsc runtime
- **Self-managed**: Install gVisor and configure containerd

```yaml
# Example node configuration (cloud-agnostic via Helm/Kustomize)
apiVersion: v1
kind: ConfigMap
metadata:
  name: gvisor-node-config
data:
  containerd-config.toml: |
    [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runsc]
      runtime_type = "io.containerd.runsc.v1"
      [plugins."io.containerd.grpc.v1.cri".containerd.runtimes.runsc.options]
        TypeUrl = "io.containerd.runsc.v1.options"
        ConfigPath = "/etc/containerd/runsc.toml"
```

## References

- [gVisor Documentation](https://gvisor.dev/docs/)
- [gVisor on Kubernetes](https://gvisor.dev/docs/user_guide/quick_start/kubernetes/)
- [Container Security Best Practices](https://kubernetes.io/docs/concepts/security/)
