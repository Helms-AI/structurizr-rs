# ADR 001: Rust Implementation

## Status

Accepted

## Context

We needed to create a software architecture visualization tool compatible with the Structurizr DSL format. The original Structurizr Lite is implemented in Java, requiring the JVM to run. Several implementation options were considered:

1. **Java** - Native port maintaining same technology stack
2. **Go** - Fast compilation, simple deployment
3. **Rust** - Memory safety, performance, single binary
4. **TypeScript/Node.js** - JavaScript ecosystem, web-native
5. **Python** - Rapid development, extensive libraries

## Decision

We chose to implement structurizr-rs in **Rust** for the following reasons:

### Performance

- **Fast startup**: ~50ms vs ~2-3s for Java
- **Low memory**: ~20MB vs ~100-200MB for JVM
- **Native execution**: No runtime overhead

### Safety

- **Memory safety**: Compile-time guarantees prevent common bugs
- **Type system**: Strong typing catches errors early
- **No null pointers**: Option types enforce explicit handling

### Deployment

- **Single binary**: No runtime dependencies
- **Cross-platform**: Compile for Windows, macOS, Linux
- **Small size**: ~10MB binary vs ~50MB+ for JVM apps

### Ecosystem

- **Cargo**: Excellent package management
- **Crates.io**: Rich library ecosystem
- **Async support**: Tokio for efficient I/O

### Developer Experience

- **Compiler messages**: Helpful error messages
- **Documentation**: Excellent tooling with rustdoc
- **Testing**: Built-in test framework

## Consequences

### Positive

- Fast iteration during development (quick builds with hot reload)
- Easy deployment to any environment
- Lower resource requirements for CI/CD pipelines
- Good fit for container deployments
- Memory safety reduces production bugs

### Negative

- Steeper learning curve for new contributors
- Longer initial development time
- Smaller developer pool compared to Java/JavaScript
- Some parsing libraries less mature than Java equivalents

### Neutral

- Different feature set than original Structurizr Lite
- May not have 100% DSL compatibility initially
- Separate maintenance from upstream project

## References

- [Rust Programming Language](https://www.rust-lang.org/)
- [Structurizr](https://structurizr.com/)
- [C4 Model](https://c4model.com/)
