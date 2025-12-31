# ADR-008: Security Sandboxing Strategy

## Status

Accepted

## Context

The platform executes arbitrary user code in a multi-tenant environment. Critical security requirements:

- Prevent container escape to host
- Isolate workspaces from each other
- Limit resource consumption (CPU, memory, network)
- Protect against cryptomining and abuse
- Audit all security-relevant actions
- Comply with SOC 2 and GDPR

**Threat Model:**
- Malicious code execution
- Container breakout attempts
- Resource exhaustion attacks
- Network-based attacks on other tenants
- Data exfiltration attempts

## Decision

We will implement a **defense-in-depth sandboxing strategy** using:

1. **gVisor (runsc)**: User-space kernel for syscall interception
2. **Seccomp**: System call filtering as backup
3. **Network Policies**: Kubernetes NetworkPolicy for egress control
4. **Resource Limits**: cgroups for CPU/memory limits
5. **Read-only filesystems**: Immutable base with overlay for user data

## Alternatives Considered

### Docker Containers Only

**Pros:**
- Simple, well-understood
- Best performance
- Widest compatibility

**Cons:**
- Shared kernel with host
- History of escape vulnerabilities
- Insufficient for untrusted code

**Why Rejected:** Shared kernel is unacceptable for running arbitrary user code.

### Firecracker microVMs

**Pros:**
- True VM-level isolation
- Separate kernel per workspace
- Used by AWS Lambda

**Cons:**
- Higher resource overhead
- Complex orchestration
- Slower boot times
- More memory per workspace

**Why Rejected:** gVisor provides sufficient isolation with better density.

### Kata Containers

**Pros:**
- VM isolation with container UX
- OCI compatible
- Cloud Hypervisor support

**Cons:**
- Higher latency than gVisor
- More complex nested virtualization
- Heavier resource footprint

**Why Rejected:** gVisor offers better performance for our workload.

### Secure Containers (Nabla, Unikernels)

**Pros:**
- Minimal attack surface
- Single-purpose kernels

**Cons:**
- Limited language support
- Complex development
- Immature ecosystem

**Why Rejected:** Too restrictive for general-purpose development.

## Consequences

### Positive

- **Strong isolation**: User-space kernel intercepts all syscalls
- **Audit logging**: All syscalls can be logged
- **Defense in depth**: Multiple layers of protection
- **Kubernetes native**: Integrates with existing infrastructure
- **Performance acceptable**: 10-20% overhead, acceptable for dev workloads

### Negative

- **Compatibility**: Some syscalls not supported
- **Debugging complexity**: Two kernel layers to analyze
- **GPU limitations**: gVisor doesn't support GPU passthrough
- **Overhead**: Higher than native containers

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Syscall compatibility | Document unsupported syscalls, provide alternatives |
| Debugging complexity | Enhanced logging, remote debugging tools |
| GPU limitations | Separate node pool without gVisor for ML workloads |
| Performance overhead | Warm pools, optimized images |

## Implementation

### gVisor RuntimeClass

```yaml
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: gvisor
handler: runsc
scheduling:
  nodeSelector:
    runtime: gvisor
---
# Pod using gVisor
apiVersion: v1
kind: Pod
metadata:
  name: workspace-abc123
  labels:
    app: workspace
    workspaceId: abc123
spec:
  runtimeClassName: gvisor
  securityContext:
    runAsNonRoot: true
    runAsUser: 1000
    runAsGroup: 1000
    fsGroup: 1000
    seccompProfile:
      type: RuntimeDefault
  containers:
    - name: workspace
      image: registry.horizonplatform.io/workspace-base:v1
      securityContext:
        allowPrivilegeEscalation: false
        capabilities:
          drop:
            - ALL
        readOnlyRootFilesystem: false
        privileged: false
      resources:
        limits:
          cpu: "2"
          memory: "4Gi"
          ephemeral-storage: "10Gi"
        requests:
          cpu: "100m"
          memory: "256Mi"
```

### Seccomp Profile

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": [
        "read", "write", "open", "close", "stat", "fstat", "lstat",
        "poll", "lseek", "mmap", "mprotect", "munmap", "brk",
        "ioctl", "access", "pipe", "select", "sched_yield",
        "dup", "dup2", "nanosleep", "getpid", "socket", "connect",
        "accept", "sendto", "recvfrom", "bind", "listen",
        "clone", "fork", "vfork", "execve", "exit", "wait4",
        "kill", "uname", "fcntl", "flock", "fsync", "fdatasync",
        "truncate", "ftruncate", "getdents", "getcwd", "chdir",
        "mkdir", "rmdir", "unlink", "readlink", "chmod", "chown",
        "arch_prctl", "time", "futex", "epoll_create", "epoll_wait",
        "epoll_ctl", "getuid", "getgid", "geteuid", "getegid",
        "setuid", "setgid", "getgroups", "set_tid_address",
        "set_robust_list", "clock_gettime", "clock_nanosleep"
      ],
      "action": "SCMP_ACT_ALLOW"
    },
    {
      "names": [
        "ptrace", "process_vm_readv", "process_vm_writev",
        "mount", "umount2", "pivot_root", "chroot",
        "kexec_load", "reboot", "syslog", "acct",
        "settimeofday", "swapon", "swapoff", "mknod"
      ],
      "action": "SCMP_ACT_ERRNO",
      "args": [],
      "comment": "Dangerous syscalls - always blocked"
    }
  ]
}
```

### Network Policies

```yaml
# Default deny all ingress/egress for workspace namespace
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: default-deny-all
  namespace: workspaces
spec:
  podSelector: {}
  policyTypes:
    - Ingress
    - Egress
---
# Allow egress to package registries
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-package-registries
  namespace: workspaces
spec:
  podSelector:
    matchLabels:
      app: workspace
  policyTypes:
    - Egress
  egress:
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
      ports:
        - protocol: TCP
          port: 443
        - protocol: TCP
          port: 80
---
# Allow egress to internal services only
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: allow-internal-services
  namespace: workspaces
spec:
  podSelector:
    matchLabels:
      app: workspace
  policyTypes:
    - Egress
  egress:
    - to:
        - namespaceSelector:
            matchLabels:
              name: core
        - namespaceSelector:
            matchLabels:
              name: collab
      ports:
        - protocol: TCP
          port: 8080
---
# Block access to cloud metadata
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: block-metadata
  namespace: workspaces
spec:
  podSelector:
    matchLabels:
      app: workspace
  policyTypes:
    - Egress
  egress:
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
            except:
              - 169.254.169.254/32  # Cloud metadata (GCP/AWS/Azure)
```

### Resource Limits

```go
package sandbox

type ResourceLimits struct {
    CPULimit       string // e.g., "2"
    MemoryLimit    string // e.g., "4Gi"
    StorageLimit   string // e.g., "10Gi"
    PIDLimit       int64  // e.g., 512
    NetworkEgress  int64  // bytes per second
    OpenFiles      int64  // e.g., 1024
}

var TierLimits = map[string]ResourceLimits{
    "free": {
        CPULimit:      "0.5",
        MemoryLimit:   "512Mi",
        StorageLimit:  "1Gi",
        PIDLimit:      128,
        NetworkEgress: 1 * 1024 * 1024, // 1 MB/s
        OpenFiles:     256,
    },
    "hacker": {
        CPULimit:      "2",
        MemoryLimit:   "4Gi",
        StorageLimit:  "10Gi",
        PIDLimit:      512,
        NetworkEgress: 10 * 1024 * 1024, // 10 MB/s
        OpenFiles:     1024,
    },
    "pro": {
        CPULimit:      "4",
        MemoryLimit:   "8Gi",
        StorageLimit:  "50Gi",
        PIDLimit:      1024,
        NetworkEgress: 50 * 1024 * 1024, // 50 MB/s
        OpenFiles:     4096,
    },
}

func (s *SandboxService) ApplyLimits(pod *v1.Pod, tier string) {
    limits := TierLimits[tier]

    pod.Spec.Containers[0].Resources = v1.ResourceRequirements{
        Limits: v1.ResourceList{
            v1.ResourceCPU:              resource.MustParse(limits.CPULimit),
            v1.ResourceMemory:           resource.MustParse(limits.MemoryLimit),
            v1.ResourceEphemeralStorage: resource.MustParse(limits.StorageLimit),
        },
        Requests: v1.ResourceList{
            v1.ResourceCPU:    resource.MustParse("100m"),
            v1.ResourceMemory: resource.MustParse("128Mi"),
        },
    }

    // PID limit via PodSpec
    pod.Spec.Containers[0].SecurityContext.ProcMount = nil

    // Annotations for gVisor-specific limits
    pod.ObjectMeta.Annotations["run.sandbox.gvisor.dev/pids-limit"] = strconv.FormatInt(limits.PIDLimit, 10)
}
```

### Abuse Detection

```go
package security

type AbuseDetector struct {
    metrics   *prometheus.Registry
    alerts    chan Alert
    thresholds AbuseThresholds
}

type AbuseThresholds struct {
    CPUThreshold      float64       // Sustained CPU %
    CPUDuration       time.Duration // How long before alert
    NetworkThreshold  int64         // Bytes per second
    CryptoPatterns    []string      // Mining pool patterns
}

func (d *AbuseDetector) MonitorWorkspace(ctx context.Context, workspaceID string) {
    ticker := time.NewTicker(30 * time.Second)
    defer ticker.Stop()

    cpuWindow := NewSlidingWindow(10) // 5 min window

    for {
        select {
        case <-ctx.Done():
            return
        case <-ticker.C:
            metrics := d.collectMetrics(workspaceID)

            // CPU abuse detection (cryptomining)
            cpuWindow.Add(metrics.CPUPercent)
            if cpuWindow.Average() > d.thresholds.CPUThreshold {
                d.alerts <- Alert{
                    Type:        AlertCPUAbuse,
                    WorkspaceID: workspaceID,
                    Value:       cpuWindow.Average(),
                    Message:     "Sustained high CPU usage detected",
                }
            }

            // Network abuse detection (DDoS, spam)
            if metrics.NetworkEgress > d.thresholds.NetworkThreshold {
                d.alerts <- Alert{
                    Type:        AlertNetworkAbuse,
                    WorkspaceID: workspaceID,
                    Value:       float64(metrics.NetworkEgress),
                    Message:     "Abnormal network egress detected",
                }
            }

            // Crypto mining detection (connection patterns)
            for _, conn := range metrics.Connections {
                for _, pattern := range d.thresholds.CryptoPatterns {
                    if strings.Contains(conn.RemoteAddr, pattern) {
                        d.alerts <- Alert{
                            Type:        AlertCryptoMining,
                            WorkspaceID: workspaceID,
                            Value:       0,
                            Message:     fmt.Sprintf("Connection to mining pool: %s", conn.RemoteAddr),
                        }
                    }
                }
            }
        }
    }
}

// Known mining pool patterns
var CryptoPatterns = []string{
    "stratum+tcp://",
    "pool.minexmr.com",
    "xmrpool.eu",
    "supportxmr.com",
    "nanopool.org",
    "nicehash.com",
    "2miners.com",
    "f2pool.com",
}
```

### Audit Logging

```go
package audit

type AuditLogger struct {
    sink   AuditSink
    buffer chan AuditEvent
}

type AuditEvent struct {
    Timestamp   time.Time         `json:"timestamp"`
    WorkspaceID string            `json:"workspace_id"`
    UserID      string            `json:"user_id"`
    Action      string            `json:"action"`
    Resource    string            `json:"resource"`
    Outcome     string            `json:"outcome"`
    Details     map[string]string `json:"details,omitempty"`
    SourceIP    string            `json:"source_ip"`
    UserAgent   string            `json:"user_agent"`
}

// Security-relevant events to audit
const (
    ActionWorkspaceCreate  = "workspace.create"
    ActionWorkspaceDelete  = "workspace.delete"
    ActionFileWrite        = "file.write"
    ActionFileDelete       = "file.delete"
    ActionCommandExecute   = "command.execute"
    ActionNetworkConnect   = "network.connect"
    ActionSecretAccess     = "secret.access"
    ActionPermissionChange = "permission.change"
    ActionCollabInvite     = "collab.invite"
    ActionDeployCreate     = "deploy.create"
)

func (l *AuditLogger) Log(event AuditEvent) {
    event.Timestamp = time.Now().UTC()
    select {
    case l.buffer <- event:
    default:
        // Buffer full, log warning
        log.Warn("Audit buffer full, event dropped", "action", event.Action)
    }
}

func (l *AuditLogger) processLoop(ctx context.Context) {
    batch := make([]AuditEvent, 0, 100)
    ticker := time.NewTicker(5 * time.Second)

    for {
        select {
        case <-ctx.Done():
            // Flush remaining
            if len(batch) > 0 {
                l.sink.Write(batch)
            }
            return
        case event := <-l.buffer:
            batch = append(batch, event)
            if len(batch) >= 100 {
                l.sink.Write(batch)
                batch = batch[:0]
            }
        case <-ticker.C:
            if len(batch) > 0 {
                l.sink.Write(batch)
                batch = batch[:0]
            }
        }
    }
}
```

### Container Image Hardening

```dockerfile
# Base workspace image with security hardening
FROM ubuntu:22.04

# Security: Remove unnecessary packages
RUN apt-get purge -y \
    telnet \
    ftp \
    rsh-client \
    && apt-get autoremove -y

# Security: Create non-root user
RUN groupadd -r runner -g 1000 \
    && useradd -r -g runner -u 1000 -d /home/runner runner \
    && mkdir -p /home/runner \
    && chown -R runner:runner /home/runner

# Security: Set filesystem permissions
RUN chmod 755 /tmp \
    && chmod 1777 /tmp

# Security: Remove setuid/setgid binaries
RUN find / -perm /6000 -type f -exec chmod a-s {} \; 2>/dev/null || true

# Security: Disable core dumps
RUN echo "* hard core 0" >> /etc/security/limits.conf

# Install languages (Nix-based, see separate store)
# Nix store mounted read-only at runtime

USER runner
WORKDIR /home/runner

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
    CMD curl -f http://localhost:8080/health || exit 1
```

## Compliance Mapping

| Requirement | Implementation |
|-------------|----------------|
| SOC 2 CC6.1 | Network policies, gVisor isolation |
| SOC 2 CC6.6 | Audit logging, access controls |
| SOC 2 CC7.2 | Abuse detection, resource limits |
| GDPR Art 32 | Encryption, isolation, access controls |
| PCI DSS 6.5 | Seccomp, read-only filesystem |

## References

- [gVisor Security Model](https://gvisor.dev/docs/architecture_guide/security/)
- [Kubernetes Security Best Practices](https://kubernetes.io/docs/concepts/security/)
- [Seccomp in Kubernetes](https://kubernetes.io/docs/tutorials/security/seccomp/)
- [gVisor on Kubernetes](https://gvisor.dev/docs/user_guide/quick_start/kubernetes/)
