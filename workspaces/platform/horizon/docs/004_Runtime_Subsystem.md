# Container Runtime Subsystem

## Overview

The runtime subsystem provides secure, isolated, and reproducible development environments for each workspace. It uses Nix for environment management and gVisor for container sandboxing.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Container Orchestrator (Go)                       │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │  Container   │ │   Health     │ │   Resource   │ │   Image      │   │
│  │  Scheduler   │ │   Monitor    │ │   Manager    │ │   Builder    │   │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
              ┌─────────────────────┼─────────────────────┐
              ▼                     ▼                     ▼
┌─────────────────────┐ ┌─────────────────────┐ ┌─────────────────────┐
│   Nix Environment   │ │   Execution Engine  │ │   Snapshot Manager  │
│      Service        │ │                     │ │                     │
└─────────────────────┘ └─────────────────────┘ └─────────────────────┘
              │                     │                     │
              ▼                     ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                      Kubernetes / gVisor Runtime                         │
└─────────────────────────────────────────────────────────────────────────┘
```

## Nix Environment System

### Why Nix?

Nix provides reproducible, declarative package management:

| Feature | Benefit |
|---------|---------|
| Reproducibility | Same environment every time |
| Isolation | No package conflicts |
| Rollback | Easy version changes |
| Caching | 1TB shared store |
| 80K+ packages | Comprehensive coverage |

### replit.nix Configuration

```nix
# replit.nix - Example Python environment
{ pkgs }: {
  deps = [
    pkgs.python311
    pkgs.python311Packages.pip
    pkgs.python311Packages.flask
    pkgs.python311Packages.sqlalchemy
    pkgs.python311Packages.pytest
    pkgs.postgresql
    pkgs.redis
  ];

  env = {
    PYTHONPATH = "${pkgs.python311}/lib/python3.11/site-packages";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [ pkgs.stdenv.cc.cc ];
  };
}
```

### .replit Configuration

```toml
# .replit - Workspace configuration
run = "python main.py"
language = "python3"
entrypoint = "main.py"

[nix]
channel = "stable-23_11"

[env]
PYTHONUNBUFFERED = "1"

[packager]
language = "python3"

[packager.features]
enabledForHosting = true
packageSearch = true

[languages.python3]
pattern = "**/*.py"

[languages.python3.languageServer]
start = "pylsp"

[deployment]
run = ["sh", "-c", "python main.py"]
```

### Shared Nix Store

All workspaces share a 1TB Nix store:

```
/nix/store/
├── hash1-python-3.11.0/
├── hash2-nodejs-20.0.0/
├── hash3-rust-1.75.0/
├── hash4-go-1.22.0/
└── ... (80K+ packages)
```

**Benefits:**
- Near-instant environment activation
- 90%+ storage savings
- Zero network downloads for cached packages

### Environment Resolution

```go
// Package resolution flow
func (s *NixService) BuildEnvironment(workspace Workspace) error {
    // 1. Parse replit.nix
    nixExpr, err := s.parseNixFile(workspace.ReplitNix)
    if err != nil {
        return err
    }

    // 2. Evaluate dependencies
    deps, err := s.evaluator.Resolve(nixExpr)
    if err != nil {
        return err
    }

    // 3. Check cache for each dependency
    uncached := []Derivation{}
    for _, dep := range deps {
        if !s.cache.Has(dep.Hash) {
            uncached = append(uncached, dep)
        }
    }

    // 4. Build uncached derivations
    if len(uncached) > 0 {
        if err := s.builder.Build(uncached); err != nil {
            return err
        }
    }

    // 5. Generate environment profile
    return s.generateProfile(workspace.ID, deps)
}
```

## Container Orchestration

### Container Lifecycle

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ Created │───▶│ Starting│───▶│ Running │───▶│ Stopping│───▶│ Stopped │
└─────────┘    └─────────┘    └─────────┘    └─────────┘    └─────────┘
     │              │              │              │              │
     │              ▼              ▼              ▼              │
     │         ┌─────────┐   ┌─────────┐   ┌─────────┐          │
     └────────▶│  Error  │◀──│ OOM     │   │ Timeout │◀─────────┘
               └─────────┘   └─────────┘   └─────────┘
```

### Scheduler Algorithm

```go
// Bin-packing with affinity
func (s *Scheduler) Schedule(request ContainerRequest) (*Node, error) {
    candidates := s.filterNodes(request)

    // Score nodes
    scores := make(map[*Node]float64)
    for _, node := range candidates {
        score := 0.0

        // Resource fit score
        score += s.resourceFitScore(node, request)

        // Affinity score (same user's containers together)
        score += s.affinityScore(node, request.UserID)

        // Anti-affinity score (spread across zones)
        score -= s.zoneConcentrationPenalty(node)

        // Nix cache locality
        score += s.nixCacheScore(node, request.NixDeps)

        scores[node] = score
    }

    return s.selectBestNode(scores), nil
}
```

### Resource Quotas

| Tier | CPU | Memory | Disk | Network |
|------|-----|--------|------|---------|
| Free | 0.5 cores | 512MB | 1GB | 10Mbps |
| Starter | 2 cores | 2GB | 10GB | 50Mbps |
| Pro | 4 cores | 8GB | 50GB | 100Mbps |
| Enterprise | 8 cores | 16GB | 256GB | 1Gbps |

```go
// Resource limits in Kubernetes
resources := corev1.ResourceRequirements{
    Requests: corev1.ResourceList{
        corev1.ResourceCPU:    resource.MustParse("500m"),
        corev1.ResourceMemory: resource.MustParse("512Mi"),
    },
    Limits: corev1.ResourceList{
        corev1.ResourceCPU:    resource.MustParse("2"),
        corev1.ResourceMemory: resource.MustParse("2Gi"),
    },
}
```

## Language Support Matrix

| Language | Version | Package Manager | LSP | Debugger |
|----------|---------|-----------------|-----|----------|
| Python | 3.9-3.12 | pip, poetry, uv | pylsp | debugpy |
| JavaScript | ES2023 | npm, yarn, pnpm | tsserver | node --inspect |
| TypeScript | 5.x | npm, yarn, pnpm | tsserver | node --inspect |
| Node.js | 18, 20, 22 | npm, yarn, pnpm | tsserver | node --inspect |
| Go | 1.21, 1.22 | go mod | gopls | delve |
| Rust | stable, nightly | cargo | rust-analyzer | lldb |
| Java | 17, 21 | Maven, Gradle | Eclipse JDT | JDWP |
| C/C++ | GCC 13, Clang 17 | CMake, Make | clangd | gdb, lldb |
| Ruby | 3.2, 3.3 | bundler | solargraph | ruby-debug-ide |
| PHP | 8.2, 8.3 | composer | intelephense | xdebug |
| C# | .NET 8 | NuGet | OmniSharp | netcoredbg |
| Swift | 5.9 | SwiftPM | sourcekit-lsp | lldb |
| Kotlin | 1.9 | Gradle | kotlin-language-server | - |
| Scala | 3.x | sbt | Metals | - |
| Elixir | 1.16 | mix | elixir-ls | - |
| Haskell | GHC 9.6 | cabal, stack | hls | - |
| Clojure | 1.11 | Leiningen | clojure-lsp | - |
| R | 4.3 | CRAN | languageserver | - |
| Julia | 1.10 | Pkg | LanguageServer.jl | - |
| Zig | 0.11 | build.zig | zls | - |

## Execution Engine

### Process Management

```go
// Process spawning with resource limits
func (e *ExecutionEngine) Spawn(cmd *Command) (*Process, error) {
    ctx, cancel := context.WithTimeout(context.Background(), cmd.Timeout)

    // Create process with isolated namespaces
    process := exec.CommandContext(ctx, cmd.Path, cmd.Args...)
    process.Dir = cmd.WorkDir
    process.Env = cmd.Env

    // Set up PTY for terminal
    pty, err := pty.Start(process)
    if err != nil {
        return nil, err
    }

    // Set resource limits via cgroups
    if err := e.cgroupManager.Apply(process.Process.Pid, cmd.Limits); err != nil {
        return nil, err
    }

    // Start output streaming
    go e.streamOutput(pty, cmd.OutputChannel)

    return &Process{
        PID:    process.Process.Pid,
        PTY:    pty,
        Cancel: cancel,
    }, nil
}
```

### Signal Handling

```go
// POSIX signal forwarding
func (p *Process) Signal(sig os.Signal) error {
    switch sig {
    case syscall.SIGTERM, syscall.SIGKILL:
        return p.cmd.Process.Signal(sig)
    case syscall.SIGINT:
        // Send to process group
        return syscall.Kill(-p.cmd.Process.Pid, syscall.SIGINT)
    case syscall.SIGWINCH:
        // Window resize
        return pty.Setsize(p.pty, p.cols, p.rows)
    default:
        return p.cmd.Process.Signal(sig)
    }
}
```

### Port Allocation

```go
// Dynamic port assignment
func (e *ExecutionEngine) AllocatePort(workspaceID string) (int, error) {
    // Find available port in range 3000-9999
    for port := 3000; port < 10000; port++ {
        if e.isPortAvailable(port) && !e.isPortReserved(port) {
            e.reservePort(workspaceID, port)
            return port, nil
        }
    }
    return 0, ErrNoPortsAvailable
}

// Port forwarding to preview URL
func (e *ExecutionEngine) ForwardPort(workspaceID string, port int) string {
    return fmt.Sprintf("https://%s-%d.preview.horizonplatform.io", workspaceID, port)
}
```

## Storage Architecture

### Workspace Storage

```
Container filesystem layout:
/
├── home/
│   └── runner/
│       └── workspace/        # Mounted from object storage
│           ├── .replit
│           ├── replit.nix
│           └── src/
├── nix/
│   └── store/                # Read-only, shared 1TB store
├── tmp/                      # Ephemeral tmpfs
└── var/
    └── cache/                # Build cache (persistent)
```

### File Synchronization

```go
// Delta sync protocol
type SyncMessage struct {
    Type     string // "create", "update", "delete"
    Path     string
    Content  []byte // For small files
    Checksum string // For change detection
    Chunks   []Chunk // For large files (delta encoding)
}

func (s *SyncEngine) SyncFile(path string, content []byte) error {
    remote, err := s.storage.Get(path)
    if err != nil && !errors.Is(err, ErrNotFound) {
        return err
    }

    if remote == nil {
        // New file
        return s.storage.Put(path, content)
    }

    // Compute delta
    delta := s.computeDelta(remote, content)
    if len(delta) < len(content) {
        return s.storage.PatchDelta(path, delta)
    }

    return s.storage.Put(path, content)
}
```

## Cold Start Optimization

### Strategies

1. **Warm Pool**: Pre-warmed containers ready to assign
2. **Snapshot Restore**: CRIU-based container snapshots
3. **Lazy Loading**: Load packages on first use
4. **Predictive Warming**: Pre-warm based on user patterns

```go
// Warm pool management
type WarmPool struct {
    pools map[string]chan *Container // Key: language
    size  int
}

func (wp *WarmPool) Get(language string) (*Container, error) {
    select {
    case container := <-wp.pools[language]:
        return container, nil
    default:
        // Pool empty, create new
        return wp.createContainer(language)
    }
}

func (wp *WarmPool) Replenish() {
    for language, pool := range wp.pools {
        for len(pool) < wp.size {
            container, _ := wp.createContainer(language)
            pool <- container
        }
    }
}
```

### Cold Start Targets

| Scenario | Target | Current |
|----------|--------|---------|
| Warm pool hit | <500ms | 400ms |
| Nix cached | <2s | 1.8s |
| Nix partial cache | <5s | 4.5s |
| Nix cold | <30s | 25s |
