# Security Architecture

## Overview

Security is implemented through defense in depth with multiple layers of protection for user code execution, data storage, and access control.

## Security Layers

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         Network Layer                                    │
│  TLS 1.3 | mTLS | Network Policies | WAF | DDoS Protection             │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                       Application Layer                                  │
│  JWT Auth | RBAC | Input Validation | Rate Limiting | CORS              │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                        Container Layer                                   │
│  gVisor | Seccomp | AppArmor | Namespaces | Cgroups                     │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
┌─────────────────────────────────────────────────────────────────────────┐
│                          Data Layer                                      │
│  Encryption at Rest | Encryption in Transit | Secrets Management        │
└─────────────────────────────────────────────────────────────────────────┘
```

## Container Sandboxing

### gVisor Runtime

gVisor provides a user-space kernel that intercepts system calls:

```yaml
# RuntimeClass for gVisor
apiVersion: node.k8s.io/v1
kind: RuntimeClass
metadata:
  name: gvisor
handler: runsc
scheduling:
  nodeSelector:
    runtime: gvisor
```

```yaml
# Pod using gVisor
apiVersion: v1
kind: Pod
metadata:
  name: workspace-container
spec:
  runtimeClassName: gvisor
  containers:
    - name: workspace
      image: workspace-base:latest
      securityContext:
        runAsNonRoot: true
        runAsUser: 1000
        allowPrivilegeEscalation: false
        capabilities:
          drop:
            - ALL
```

### Seccomp Profile

```json
{
  "defaultAction": "SCMP_ACT_ERRNO",
  "architectures": ["SCMP_ARCH_X86_64"],
  "syscalls": [
    {
      "names": [
        "read", "write", "open", "close", "stat", "fstat",
        "lstat", "poll", "lseek", "mmap", "mprotect", "munmap",
        "brk", "rt_sigaction", "rt_sigprocmask", "ioctl",
        "access", "pipe", "select", "sched_yield", "mremap",
        "msync", "mincore", "madvise", "dup", "dup2", "pause",
        "nanosleep", "getpid", "socket", "connect", "accept",
        "sendto", "recvfrom", "sendmsg", "recvmsg", "shutdown",
        "bind", "listen", "getsockname", "getpeername",
        "clone", "fork", "vfork", "execve", "exit", "wait4",
        "kill", "uname", "fcntl", "flock", "fsync", "fdatasync",
        "truncate", "ftruncate", "getdents", "getcwd", "chdir",
        "mkdir", "rmdir", "creat", "link", "unlink", "symlink",
        "readlink", "chmod", "chown", "lchown", "gettimeofday",
        "getrlimit", "getrusage", "sysinfo", "times", "getuid",
        "getgid", "geteuid", "getegid", "setuid", "setgid",
        "getgroups", "setgroups", "getpgrp", "setpgid",
        "getppid", "setsid", "utime", "mknod", "statfs",
        "fstatfs", "sched_setparam", "sched_getparam",
        "sched_setscheduler", "sched_getscheduler",
        "sched_get_priority_max", "sched_get_priority_min",
        "sched_rr_get_interval", "mlock", "munlock",
        "mlockall", "munlockall", "prctl", "arch_prctl",
        "futex", "epoll_create", "epoll_ctl", "epoll_wait",
        "exit_group", "set_tid_address", "clock_gettime",
        "clock_nanosleep", "tgkill", "openat", "mkdirat",
        "fchownat", "futimesat", "newfstatat", "unlinkat",
        "renameat", "linkat", "symlinkat", "readlinkat",
        "fchmodat", "faccessat", "pselect6", "ppoll",
        "set_robust_list", "get_robust_list", "epoll_pwait",
        "eventfd", "timerfd_create", "signalfd", "eventfd2",
        "epoll_create1", "dup3", "pipe2", "inotify_init1",
        "preadv", "pwritev", "accept4", "signalfd4",
        "timerfd_settime", "timerfd_gettime", "recvmmsg",
        "fanotify_init", "prlimit64", "sendmmsg", "setns",
        "getcpu", "getrandom", "memfd_create", "execveat"
      ],
      "action": "SCMP_ACT_ALLOW"
    }
  ]
}
```

### Resource Limits

```yaml
# Cgroups v2 configuration
resources:
  limits:
    cpu: "2"
    memory: "2Gi"
    ephemeral-storage: "10Gi"
  requests:
    cpu: "500m"
    memory: "512Mi"
```

```go
// Resource enforcement
type ResourceLimits struct {
    CPUQuota      int64 // microseconds per second (1000000 = 1 core)
    MemoryLimit   int64 // bytes
    PidsLimit     int64 // max processes
    DiskReadBPS   int64 // bytes per second
    DiskWriteBPS  int64 // bytes per second
    NetworkBPS    int64 // bytes per second
}

func (r *ResourceManager) ApplyLimits(pid int, limits ResourceLimits) error {
    // Set CPU quota
    cgroupPath := fmt.Sprintf("/sys/fs/cgroup/cpu/%d", pid)
    ioutil.WriteFile(
        filepath.Join(cgroupPath, "cpu.max"),
        []byte(fmt.Sprintf("%d 100000", limits.CPUQuota)),
        0644,
    )

    // Set memory limit
    ioutil.WriteFile(
        filepath.Join(cgroupPath, "memory.max"),
        []byte(fmt.Sprintf("%d", limits.MemoryLimit)),
        0644,
    )

    return nil
}
```

## Network Security

### Network Policies

```yaml
# Default deny all ingress/egress
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: workspace-default-deny
spec:
  podSelector:
    matchLabels:
      type: workspace
  policyTypes:
    - Ingress
    - Egress

---
# Allow specific egress
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: workspace-egress
spec:
  podSelector:
    matchLabels:
      type: workspace
  policyTypes:
    - Egress
  egress:
    # Allow DNS
    - to:
        - namespaceSelector: {}
      ports:
        - protocol: UDP
          port: 53
    # Allow package registries
    - to:
        - ipBlock:
            cidr: 0.0.0.0/0
      ports:
        - protocol: TCP
          port: 443
```

### Egress Filtering

```go
// Domain-based egress filtering
var allowedDomains = []string{
    "*.npmjs.org",
    "*.pypi.org",
    "*.github.com",
    "*.githubusercontent.com",
    "*.docker.io",
    "*.docker.com",
}

var blockedDomains = []string{
    "*.malware.com",
    "*.phishing.net",
    // Dynamically updated threat intelligence
}

func (f *EgressFilter) Allow(domain string) bool {
    // Check blocklist first
    for _, blocked := range blockedDomains {
        if matchWildcard(blocked, domain) {
            return false
        }
    }

    // Check allowlist
    for _, allowed := range allowedDomains {
        if matchWildcard(allowed, domain) {
            return true
        }
    }

    return false // Deny by default
}
```

## Authentication & Authorization

### JWT Authentication

```go
// JWT validation middleware
func AuthMiddleware() gin.HandlerFunc {
    return func(c *gin.Context) {
        token := extractToken(c.Request)
        if token == "" {
            c.AbortWithStatusJSON(401, gin.H{"error": "unauthorized"})
            return
        }

        claims, err := validateJWT(token)
        if err != nil {
            c.AbortWithStatusJSON(401, gin.H{"error": "invalid token"})
            return
        }

        // Check token expiration
        if claims.ExpiresAt.Before(time.Now()) {
            c.AbortWithStatusJSON(401, gin.H{"error": "token expired"})
            return
        }

        // Set user context
        c.Set("userId", claims.Subject)
        c.Set("scopes", claims.Scopes)

        c.Next()
    }
}
```

### RBAC Implementation

```go
// Permission model
type Permission string

const (
    PermissionRead   Permission = "read"
    PermissionWrite  Permission = "write"
    PermissionShare  Permission = "share"
    PermissionDelete Permission = "delete"
    PermissionAdmin  Permission = "admin"
)

type Role string

const (
    RoleOwner  Role = "owner"
    RoleEditor Role = "editor"
    RoleViewer Role = "viewer"
)

var rolePermissions = map[Role][]Permission{
    RoleOwner:  {PermissionRead, PermissionWrite, PermissionShare, PermissionDelete, PermissionAdmin},
    RoleEditor: {PermissionRead, PermissionWrite},
    RoleViewer: {PermissionRead},
}

func (a *Authorizer) Check(userId, workspaceId string, required Permission) bool {
    // Get user's role for workspace
    role, err := a.getRole(userId, workspaceId)
    if err != nil {
        return false
    }

    // Check if role has required permission
    permissions := rolePermissions[role]
    for _, p := range permissions {
        if p == required {
            return true
        }
    }

    return false
}
```

## Data Protection

### Encryption at Rest

```yaml
# Kubernetes secrets with encryption provider
apiVersion: v1
kind: Secret
metadata:
  name: workspace-encryption-key
  annotations:
    kubernetes.io/encryption-provider-class: "gcp-kms"
type: Opaque
data:
  key: <kms-encrypted-key>
```

```go
// Field-level encryption for sensitive data
type EncryptedField struct {
    Ciphertext []byte `json:"ciphertext"`
    KeyID      string `json:"keyId"`
    Algorithm  string `json:"algorithm"`
}

func (e *Encryptor) Encrypt(plaintext []byte) (*EncryptedField, error) {
    // Get current encryption key
    key, err := e.kms.GetCurrentKey()
    if err != nil {
        return nil, err
    }

    // Generate nonce
    nonce := make([]byte, 12)
    rand.Read(nonce)

    // Encrypt with AES-256-GCM
    block, _ := aes.NewCipher(key.Material)
    gcm, _ := cipher.NewGCM(block)
    ciphertext := gcm.Seal(nonce, nonce, plaintext, nil)

    return &EncryptedField{
        Ciphertext: ciphertext,
        KeyID:      key.ID,
        Algorithm:  "AES-256-GCM",
    }, nil
}
```

### Secrets Management

```go
// Vault integration for secrets
type SecretsManager struct {
    vault *vault.Client
}

func (s *SecretsManager) GetSecret(workspaceId, key string) (string, error) {
    path := fmt.Sprintf("secret/workspaces/%s/%s", workspaceId, key)

    secret, err := s.vault.Logical().Read(path)
    if err != nil {
        return "", err
    }

    return secret.Data["value"].(string), nil
}

func (s *SecretsManager) InjectSecrets(workspaceId string, env map[string]string) error {
    secrets, err := s.listSecrets(workspaceId)
    if err != nil {
        return err
    }

    for _, key := range secrets {
        value, err := s.GetSecret(workspaceId, key)
        if err != nil {
            continue
        }
        env[key] = value
    }

    return nil
}
```

## Compliance

### SOC 2 Type II

| Control | Implementation |
|---------|----------------|
| Access Control | RBAC, MFA, SSO |
| Change Management | GitOps, audit logs |
| Risk Assessment | Quarterly reviews |
| Monitoring | 24/7 alerting |
| Incident Response | Documented procedures |
| Encryption | AES-256, TLS 1.3 |
| Backup | Daily, geo-redundant |

### GDPR

| Requirement | Implementation |
|-------------|----------------|
| Data Portability | Export API |
| Right to Erasure | Account deletion |
| Data Minimization | Retention policies |
| Purpose Limitation | Privacy policy |
| Consent | Opt-in features |
| DPO | Designated contact |

### Audit Logging

```go
// Comprehensive audit logging
type AuditEvent struct {
    ID          string    `json:"id"`
    Timestamp   time.Time `json:"timestamp"`
    UserID      string    `json:"userId"`
    WorkspaceID string    `json:"workspaceId,omitempty"`
    Action      string    `json:"action"`
    Resource    string    `json:"resource"`
    Details     any       `json:"details,omitempty"`
    IPAddress   string    `json:"ipAddress"`
    UserAgent   string    `json:"userAgent"`
    Result      string    `json:"result"` // success, failure
}

func (a *AuditLogger) Log(event AuditEvent) error {
    event.ID = uuid.New().String()
    event.Timestamp = time.Now().UTC()

    // Write to immutable audit log
    return a.storage.Append(event)
}
```

## Security Monitoring

### Threat Detection

```yaml
# Falco rules for runtime security
- rule: Shell Spawned in Container
  desc: Detect shell spawned in a container
  condition: >
    spawned_process and container and
    shell_procs and
    not expected_shell
  output: >
    Shell spawned in container
    (user=%user.name container=%container.name
    shell=%proc.name parent=%proc.pname)
  priority: WARNING

- rule: Unexpected Network Connection
  desc: Detect unexpected outbound connection
  condition: >
    outbound and container and
    not allowed_network
  output: >
    Unexpected network connection
    (user=%user.name container=%container.name
    connection=%fd.name)
  priority: CRITICAL
```

### Vulnerability Scanning

```yaml
# Container scanning in CI/CD
- name: Scan container image
  uses: trivy-action@v1
  with:
    image-ref: 'workspace-base:latest'
    format: 'sarif'
    severity: 'CRITICAL,HIGH'
    exit-code: '1'
```

## Incident Response

### Response Procedures

1. **Detection**: Automated alerts from monitoring
2. **Triage**: Severity classification
3. **Containment**: Isolate affected systems
4. **Investigation**: Root cause analysis
5. **Remediation**: Fix and verify
6. **Recovery**: Restore services
7. **Post-mortem**: Document lessons learned

### Contact

- Security Team: security@horizonplatform.io
- Bug Bounty: https://hackerone.com/horizonplatform
- Status: https://status.horizonplatform.io
