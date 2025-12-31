# ADR-003: Nix-Based Environment Management

## Status

Accepted

## Context

The platform must provide users with isolated, reproducible development environments supporting:

- 50+ programming languages
- Thousands of packages per language
- Consistent environments across all workspaces
- Fast environment activation (<2s)
- Minimal storage overhead per workspace

**Options Considered:**

1. **Docker images per language**
   - One image per language with pre-installed packages
   - User installs additional packages at runtime

2. **Conda/virtualenv per language**
   - Language-specific environment managers
   - Different tools for different ecosystems

3. **Nix package manager**
   - Declarative, reproducible package management
   - Single tool for all languages
   - Shared package store

## Decision

We will use **Nix for environment management** with a shared 1TB package store mounted across all workspace containers.

**Key Design:**

1. **Shared Nix Store**: Pre-populated 1TB disk image with all packages, mounted read-only in every container
2. **Declarative Configuration**: `replit.nix` files define workspace dependencies
3. **Environment Activation**: Nix shell activation on workspace start
4. **No Downloads**: Packages already present in shared store

## Alternatives Considered

### Docker Images per Language

**Pros:**
- Simple mental model
- Familiar to developers
- Good tooling

**Cons:**
- Storage explosion (GB per workspace)
- Cold start downloading layers
- Version conflicts
- Limited package selection

**Why Rejected:** Storage costs and cold start times unacceptable at scale.

### Conda/virtualenv

**Pros:**
- Familiar to Python/data science users
- Good isolation

**Cons:**
- Python-centric
- Different tools per language (npm, bundler, etc.)
- Downloads on first use
- Version conflicts possible

**Why Rejected:** Fragmented tooling, no unified approach.

### Buildpacks

**Pros:**
- Automatic detection
- Cloud Native standard

**Cons:**
- Build time on each deploy
- Less control over versions
- Not designed for development

**Why Rejected:** Better suited for deployment than development.

## Consequences

### Positive

- **Near-instant activation**: No download time, packages pre-cached
- **90%+ storage savings**: Deduplication via content-addressed store
- **Reproducibility**: Identical environments every time
- **80K+ packages**: Comprehensive coverage
- **Unified tooling**: Same system for all languages

### Negative

- **Nix learning curve**: Unfamiliar syntax for most developers
- **Complex store management**: 1TB image must be built and distributed
- **Limited Windows packages**: Nix primarily Linux-focused
- **Expression complexity**: Advanced features require Nix expertise

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Learning curve | Simple replit.nix templates, UI for common packages |
| Store management | Automated CI/CD for store updates |
| Windows packages | Document alternatives, WSL support |

## Implementation

### Shared Nix Store

```bash
# Build the shared store image (CI/CD)
nix-build '<nixpkgs>' -A all-packages --out-link /nix-store

# Create disk image
dd if=/dev/zero of=nix-store.img bs=1G count=1024
mkfs.ext4 nix-store.img
mount nix-store.img /mnt/nix-store
cp -r /nix/store/* /mnt/nix-store/

# Distribute to nodes (using S3-compatible storage like MinIO)
mc cp nix-store.img minio/horizon-infra/nix-store.img
```

### Container Mount

```yaml
apiVersion: v1
kind: Pod
metadata:
  name: workspace
spec:
  volumes:
    - name: nix-store
      hostPath:
        path: /mnt/nix-store
        type: Directory
  containers:
    - name: workspace
      volumeMounts:
        - name: nix-store
          mountPath: /nix/store
          readOnly: true
```

### replit.nix Configuration

```nix
# replit.nix - Python web development environment
{ pkgs }: {
  deps = [
    pkgs.python311
    pkgs.python311Packages.pip
    pkgs.python311Packages.flask
    pkgs.python311Packages.sqlalchemy
    pkgs.python311Packages.pytest
    pkgs.postgresql
    pkgs.redis
    pkgs.nodejs_20
    pkgs.yarn
  ];

  env = {
    PYTHONPATH = "${pkgs.python311}/lib/python3.11/site-packages";
    LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
      pkgs.stdenv.cc.cc
      pkgs.zlib
    ];
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
guessImports = true

[languages.python3]
pattern = "**/*.py"

[languages.python3.languageServer]
start = "pylsp"
```

### Environment Activation

```go
func (s *NixService) ActivateEnvironment(workspace Workspace) error {
    // Parse replit.nix
    nixExpr, err := s.parseNixFile(workspace.ReplitNix)
    if err != nil {
        return err
    }

    // Build environment profile
    profile := s.buildProfile(nixExpr.Deps)

    // Set environment variables
    for key, value := range nixExpr.Env {
        os.Setenv(key, value)
    }

    // Update PATH
    paths := []string{}
    for _, dep := range profile.Deps {
        paths = append(paths, filepath.Join("/nix/store", dep.Hash, "bin"))
    }
    os.Setenv("PATH", strings.Join(paths, ":"))

    return nil
}
```

## Language Support Matrix

| Language | Nix Package | Version | Package Manager |
|----------|-------------|---------|-----------------|
| Python | python311 | 3.11.x | pip, poetry |
| Node.js | nodejs_20 | 20.x | npm, yarn, pnpm |
| Go | go_1_22 | 1.22.x | go mod |
| Rust | rustc | stable | cargo |
| Java | jdk21 | 21 | maven, gradle |
| Ruby | ruby_3_3 | 3.3.x | bundler |
| PHP | php83 | 8.3.x | composer |
| C/C++ | gcc13 | 13.x | cmake, make |

## References

- [Nix Package Manager](https://nixos.org/manual/nix/stable/)
- [Replit Nix Blog Post](https://blog.replit.com/nix)
- [Nixpkgs Repository](https://github.com/NixOS/nixpkgs)
