# Code Analysis for C4 Architecture Generation - Implementation Specification

> **Status**: Phase 0 - Planning Complete
> **Last Updated**: 2026-01-07
> **Author**: Claude Code
> **Plan Reference**: `/Users/kon1790/.claude/plans/golden-sleeping-milner.md`

---

## ⚠️ IMPORTANT: Self-Updating Specification

**This specification should be updated as tasks are completed:**

1. When completing a task, change `[ ]` to `[x]`
2. Add completion date and any relevant notes
3. Update the **Status** field at the top of this document
4. Document any deviations from the original plan
5. Add discovered tasks or requirements as new checklist items
6. Update **Last Updated** timestamp after each modification

**Responsibility**: The implementer (human or AI) should update this spec after each completed task or phase. This serves as the single source of truth for project progress.

---

## Executive Summary

Implement a comprehensive code analysis system in the `structurizr-analysis` crate that automatically generates C4 architecture diagrams from source code across 9+ programming languages using a parallel LSP architecture for maximum speed and accuracy.

### Key Requirements

| Requirement | Target | Priority |
|-------------|--------|----------|
| Analysis Speed | <60 seconds for 100K LOC | HIGH |
| Accuracy | 70-80% relationship detection | MEDIUM |
| Language Support | TypeScript, Go (Phase 1) | HIGH |
| LSP Integration | Required, parallel execution | HIGH |
| Cross-language | Full API detection | HIGH |
| Config Analysis | Docker, K8s, CI/CD | MEDIUM |

---

## Current State

### Existing Implementation (structurizr-analysis crate)

- [x] Rust analyzer using tree-sitter-rust (~1230 LOC)
- [x] `LanguageAnalyzer` trait for extensibility
- [x] `AnalyzerConfig` with sensible defaults
- [x] Project detector for multiple manifest formats
- [x] Intermediate model (AnalyzedProject, AnalyzedContainer, AnalyzedComponent)
- [x] C4 workspace generator with views
- [x] Confidence-based relationship inference
- [x] DSL serialization

### Key Files

| File | Purpose | Status |
|------|---------|--------|
| `crates/structurizr-analysis/src/lib.rs` | Public API | Exists |
| `crates/structurizr-analysis/src/analyzer.rs` | Analyzer trait & registry | Exists |
| `crates/structurizr-analysis/src/detector.rs` | Project detection | Exists |
| `crates/structurizr-analysis/src/model.rs` | Intermediate representation | Exists |
| `crates/structurizr-analysis/src/generator.rs` | C4 workspace generation | Exists |
| `crates/structurizr-analysis/src/languages/rust.rs` | Rust analyzer | Exists |

---

## Phase 1: Core Infrastructure (Weeks 1-4)

### 1.1 Parallel LSP Architecture (Week 1-2)

**Goal**: Design and implement parallel LSP manager using Tokio for concurrent language server execution.

#### Core LSP Module

**File**: `crates/structurizr-analysis/src/lsp/mod.rs` (NEW)

- [ ] Create `LspClient` trait for language server communication
- [ ] Implement `LspMessage` types (Request, Response, Notification)
- [ ] Add JSON-RPC 2.0 protocol handling
- [ ] Create connection lifecycle management (spawn, initialize, shutdown)
- [ ] Add timeout and error recovery

```rust
// Target API
pub trait LspClient: Send + Sync {
    async fn initialize(&mut self, root_uri: &str) -> Result<ServerCapabilities>;
    async fn text_document_symbols(&self, uri: &str) -> Result<Vec<DocumentSymbol>>;
    async fn text_document_references(&self, uri: &str, position: Position) -> Result<Vec<Location>>;
    async fn shutdown(&mut self) -> Result<()>;
}
```

#### Parallel Manager

**File**: `crates/structurizr-analysis/src/lsp/manager.rs` (NEW)

- [ ] Create `ParallelLspManager` struct
- [ ] Implement language → server spawning logic
- [ ] Add concurrent analysis execution with `tokio::spawn`
- [ ] Create result merging with confidence weighting
- [ ] Add server health monitoring and restart

```rust
pub struct ParallelLspManager {
    servers: HashMap<Language, Box<dyn LspClient>>,
    config: LspConfig,
}

impl ParallelLspManager {
    pub async fn analyze_parallel(&self, paths: &[PathBuf]) -> Vec<AnalysisResult>;
}
```

#### TypeScript LSP Client

**File**: `crates/structurizr-analysis/src/lsp/typescript.rs` (NEW)

- [ ] Implement `TsServerClient` using tsserver protocol
- [ ] Add TypeScript-specific symbol extraction
- [ ] Handle .ts, .tsx, .d.ts file types
- [ ] Parse type information for relationships
- [ ] Extract decorator metadata

#### Go LSP Client

**File**: `crates/structurizr-analysis/src/lsp/gopls.rs` (NEW)

- [ ] Implement `GoplsClient` using gopls
- [ ] Add Go-specific symbol extraction
- [ ] Handle go.mod workspace detection
- [ ] Parse interface implementations
- [ ] Extract package relationships

#### LSP Configuration

**File**: `crates/structurizr-analysis/src/lsp/config.rs` (NEW)

- [ ] Create `LspConfig` struct with timeouts, paths, options
- [ ] Add per-language server configuration
- [ ] Support custom server paths/commands
- [ ] Add environment variable overrides

#### Testing

- [ ] Unit tests for JSON-RPC protocol handling
- [ ] Integration test with real tsserver
- [ ] Integration test with real gopls
- [ ] Parallel execution benchmarks
- [ ] Error recovery tests

---

### 1.2 TypeScript Analyzer (Week 2)

**Goal**: Complete TypeScript analyzer with tree-sitter + LSP enrichment.

#### Tree-sitter Parser

**File**: `crates/structurizr-analysis/src/languages/typescript.rs` (NEW)

- [ ] Add tree-sitter-typescript dependency to Cargo.toml
- [ ] Implement `TypeScriptAnalyzer` struct
- [ ] Parse class declarations and exports
- [ ] Extract interface definitions
- [ ] Handle decorators (@Injectable, @Controller, etc.)
- [ ] Parse import/export statements
- [ ] Extract function signatures
- [ ] Handle namespace/module declarations

#### LSP Enrichment

- [ ] Integrate LSP type information
- [ ] Resolve cross-file references
- [ ] Extract implemented interfaces
- [ ] Detect framework patterns (Express, NestJS, React)

#### Framework Pattern Detection

- [ ] Express.js patterns (Router, middleware, app.use)
- [ ] NestJS patterns (@Module, @Controller, @Injectable)
- [ ] React patterns (components, hooks, context)
- [ ] Next.js patterns (pages, API routes, getServerSideProps)

#### Testing

- [ ] Unit tests for parser
- [ ] Integration test with sample TS project
- [ ] NestJS project analysis test
- [ ] Express project analysis test
- [ ] Monorepo (npm workspaces) test

---

### 1.3 Go Analyzer (Week 2-3)

**Goal**: Complete Go analyzer with tree-sitter + gopls enrichment.

#### Tree-sitter Parser

**File**: `crates/structurizr-analysis/src/languages/go.rs` (NEW)

- [ ] Add tree-sitter-go dependency to Cargo.toml
- [ ] Implement `GoAnalyzer` struct
- [ ] Parse struct definitions
- [ ] Extract interface declarations
- [ ] Handle package imports
- [ ] Parse function signatures
- [ ] Extract method receivers
- [ ] Handle embedded structs

#### LSP Enrichment

- [ ] Integrate gopls type information
- [ ] Resolve interface implementations
- [ ] Extract cross-package relationships
- [ ] Detect exported vs unexported symbols

#### Framework Pattern Detection

- [ ] Gin patterns (Router, handlers, middleware)
- [ ] Echo patterns (routes, groups, context)
- [ ] Standard library patterns (http.Handler, http.HandlerFunc)
- [ ] gRPC service patterns

#### Testing

- [ ] Unit tests for parser
- [ ] Integration test with sample Go project
- [ ] Gin project analysis test
- [ ] Multi-module workspace test
- [ ] Interface implementation detection test

---

### 1.4 Pattern Library System (Week 3-4)

**Goal**: Build extensible pattern recognition for architectural patterns.

#### Pattern Trait

**File**: `crates/structurizr-analysis/src/patterns/mod.rs` (NEW)

- [ ] Define `ArchitecturalPattern` trait
- [ ] Create `PatternMatch` result type with confidence
- [ ] Implement `PatternRegistry` for registration/discovery
- [ ] Add pattern priority ordering

```rust
pub trait ArchitecturalPattern: Send + Sync {
    fn name(&self) -> &str;
    fn detect(&self, context: &AnalysisContext) -> Option<PatternMatch>;
    fn confidence(&self) -> f32;
}

pub struct PatternRegistry {
    patterns: Vec<Box<dyn ArchitecturalPattern>>,
}
```

#### Built-in Patterns

**File**: `crates/structurizr-analysis/src/patterns/builtin.rs` (NEW)

- [ ] MVC pattern detector
- [ ] Repository pattern detector
- [ ] Service layer pattern detector
- [ ] Controller/Handler pattern detector
- [ ] Middleware/Pipeline pattern detector
- [ ] Factory pattern detector
- [ ] Dependency injection pattern detector

#### Layer Inference

**File**: `crates/structurizr-analysis/src/patterns/layers.rs` (NEW)

- [ ] Presentation layer detection (Controllers, Handlers, Routes)
- [ ] Business layer detection (Services, UseCases, Interactors)
- [ ] Data layer detection (Repositories, DAOs, Models)
- [ ] Infrastructure layer detection (Adapters, Gateways, Clients)

#### Community Pattern Format

**File**: `crates/structurizr-analysis/src/patterns/community.rs` (NEW)

- [ ] Define JSON/TOML pattern definition format
- [ ] Create pattern loader from files
- [ ] Add pattern validation
- [ ] Support pattern composition

```toml
# Example community pattern definition
[pattern]
name = "Express Router"
language = "typescript"
confidence = 0.85

[[pattern.indicators]]
type = "import"
module = "express"
symbol = "Router"

[[pattern.indicators]]
type = "method_call"
pattern = "router\\.(get|post|put|delete|patch)"

[pattern.mapping]
component_type = "Handler"
layer = "presentation"
```

#### Testing

- [ ] Unit tests for each built-in pattern
- [ ] Pattern priority ordering tests
- [ ] Community pattern loading tests
- [ ] Multi-pattern detection tests

---

### 1.5 API Protocol Detection (Week 4)

**Goal**: Detect REST, GraphQL, gRPC, and WebSocket APIs for cross-language relationships.

#### REST/OpenAPI Detection

**File**: `crates/structurizr-analysis/src/api/rest.rs` (NEW)

- [ ] Parse OpenAPI/Swagger spec files (JSON/YAML)
- [ ] Detect Express/Gin/Echo route definitions
- [ ] Extract HTTP method and path patterns
- [ ] Match endpoints to handlers
- [ ] Detect request/response types

#### GraphQL Detection

**File**: `crates/structurizr-analysis/src/api/graphql.rs` (NEW)

- [ ] Parse .graphql schema files
- [ ] Detect resolver implementations
- [ ] Extract query/mutation/subscription definitions
- [ ] Map schema types to code types
- [ ] Detect Apollo/graphql-js patterns

#### gRPC/Protobuf Detection

**File**: `crates/structurizr-analysis/src/api/grpc.rs` (NEW)

- [ ] Parse .proto files
- [ ] Extract service definitions
- [ ] Map RPC methods to implementations
- [ ] Detect client stub usage
- [ ] Track message types

#### WebSocket Detection

**File**: `crates/structurizr-analysis/src/api/websocket.rs` (NEW)

- [ ] Detect WebSocket server initialization
- [ ] Extract event handlers
- [ ] Map event names to handlers
- [ ] Detect Socket.io patterns
- [ ] Track message types

#### Cross-Language Matching

**File**: `crates/structurizr-analysis/src/api/matcher.rs` (NEW)

- [ ] Match frontend API calls to backend endpoints
- [ ] Create cross-language relationships
- [ ] Handle URL pattern matching
- [ ] Support base URL configuration
- [ ] Generate confidence scores

#### Testing

- [ ] OpenAPI parsing tests
- [ ] GraphQL schema parsing tests
- [ ] Protobuf parsing tests
- [ ] Cross-language matching tests
- [ ] Real-world project integration tests

---

## Phase 2: Advanced Analysis (Weeks 5-8)

### 2.1 Cross-Language Integration (Week 5-6)

**Goal**: Build polyglot relationship detector for multi-language projects.

#### Polyglot Detector

**File**: `crates/structurizr-analysis/src/polyglot.rs` (NEW)

- [ ] Analyze multiple languages in single project
- [ ] Detect language boundaries (directories, packages)
- [ ] Match API contracts across languages
- [ ] Handle shared type definitions
- [ ] Support typed clients (openapi-generator output)

#### API Contract Matching

- [ ] Match TypeScript fetch/axios calls to Go handlers
- [ ] Match Go HTTP clients to TypeScript endpoints
- [ ] Handle path parameter matching
- [ ] Support query parameter detection
- [ ] Match request/response body types

#### Shared Type Detection

- [ ] Detect JSON schema files
- [ ] Parse TypeScript type exports used by other languages
- [ ] Match protobuf message types across implementations
- [ ] Handle graphql-codegen output

#### Monorepo Workspace Detection

**File**: `crates/structurizr-analysis/src/workspace_detection.rs` (NEW)

- [ ] Detect npm/yarn workspaces
- [ ] Parse Cargo workspace definitions
- [ ] Handle Go multi-module workspaces
- [ ] Support Nx/Lerna/Rush configurations
- [ ] Detect pnpm workspaces

#### Testing

- [ ] Multi-language project analysis test
- [ ] API contract matching accuracy test
- [ ] Monorepo detection test
- [ ] Cross-language relationship confidence test

---

### 2.2 Configuration Analysis (Week 7-8)

**Goal**: Parse deployment and infrastructure configurations for runtime relationships.

#### Docker Analysis

**File**: `crates/structurizr-analysis/src/config/docker.rs` (NEW)

- [ ] Parse Dockerfile for container metadata
- [ ] Parse docker-compose.yml for service relationships
- [ ] Extract exposed ports
- [ ] Detect environment variable dependencies
- [ ] Map volumes to data stores

#### Kubernetes Analysis

**File**: `crates/structurizr-analysis/src/config/kubernetes.rs` (NEW)

- [ ] Parse Deployment manifests
- [ ] Extract Service definitions
- [ ] Detect ConfigMap/Secret dependencies
- [ ] Map Ingress routes
- [ ] Parse NetworkPolicy relationships

#### CI/CD Analysis

**File**: `crates/structurizr-analysis/src/config/cicd.rs` (NEW)

- [ ] Parse GitHub Actions workflows
- [ ] Parse GitLab CI configurations
- [ ] Detect deployment targets
- [ ] Extract build dependencies
- [ ] Map artifact relationships

#### Cloud Infrastructure

**File**: `crates/structurizr-analysis/src/config/cloud.rs` (NEW)

- [ ] Parse Terraform configurations
- [ ] Extract AWS/GCP/Azure resource definitions
- [ ] Detect database instances
- [ ] Map queue/messaging services
- [ ] Extract networking relationships

#### Service Discovery

**File**: `crates/structurizr-analysis/src/config/discovery.rs` (NEW)

- [ ] Parse Consul service definitions
- [ ] Extract Eureka configurations
- [ ] Detect environment-based discovery
- [ ] Map service mesh configurations

#### Testing

- [ ] Docker Compose parsing tests
- [ ] Kubernetes manifest tests
- [ ] GitHub Actions parsing tests
- [ ] Terraform parsing tests
- [ ] Integration with code analysis tests

---

## Phase 3: Production Readiness (Weeks 9-12)

### 3.1 Performance Optimization (Week 9-10)

**Goal**: Achieve <60 second analysis for 100K LOC.

#### Parallel File Processing

- [ ] Implement rayon-based parallel file parsing
- [ ] Add file-level caching with modification detection
- [ ] Optimize tree-sitter parsing with reusable parsers
- [ ] Implement lazy loading for large files

#### Incremental Analysis

**File**: `crates/structurizr-analysis/src/cache.rs` (NEW)

- [ ] Create analysis cache with file hash tracking
- [ ] Implement cache invalidation on file change
- [ ] Add partial re-analysis for changed files only
- [ ] Support cache persistence to disk

#### LSP Optimization

- [ ] Batch LSP requests to reduce round-trips
- [ ] Implement request pipelining
- [ ] Add connection pooling for LSP servers
- [ ] Optimize server startup time

#### Progress Reporting

**File**: `crates/structurizr-analysis/src/progress.rs` (NEW)

- [ ] Create progress callback trait
- [ ] Add file/phase progress reporting
- [ ] Support cancellation tokens
- [ ] Implement timeout handling

#### Benchmarking

- [ ] Create benchmark suite with real projects
- [ ] Measure analysis time for various codebase sizes
- [ ] Profile memory usage
- [ ] Identify and optimize bottlenecks

---

### 3.2 Testing Suite (Week 11)

**Goal**: Comprehensive testing for all analyzers.

#### Unit Tests

- [ ] TypeScript analyzer unit tests (>80% coverage)
- [ ] Go analyzer unit tests (>80% coverage)
- [ ] Pattern detection unit tests
- [ ] API protocol detection unit tests
- [ ] Configuration parsing unit tests

#### Integration Tests

**Directory**: `crates/structurizr-analysis/tests/`

- [ ] Real TypeScript project analysis (Express app)
- [ ] Real Go project analysis (Gin API)
- [ ] Multi-language project analysis
- [ ] Monorepo analysis
- [ ] Large codebase performance test

#### Accuracy Tests

- [ ] Compare generated C4 models to hand-drawn diagrams
- [ ] Measure relationship detection precision/recall
- [ ] Test confidence score calibration
- [ ] Validate layer inference accuracy

---

### 3.3 CLI Integration (Week 11-12)

**Goal**: Integrate with structurizr-rs CLI.

#### CLI Commands

**File**: `src/main.rs` (MODIFY)

- [ ] Add `analyze` subcommand
- [ ] Support `--language` flag for specific analyzers
- [ ] Add `--output` for DSL/JSON output
- [ ] Support `--parallel` for LSP configuration
- [ ] Add `--patterns` for custom pattern directory

```bash
# Target CLI usage
structurizr-rs analyze ./my-project --output workspace.dsl
structurizr-rs analyze ./my-project --format json --output model.json
structurizr-rs analyze ./my-project --languages typescript,go
```

#### Web Integration

**File**: `crates/structurizr-web/src/handlers.rs` (MODIFY)

- [ ] Add `/analyze` API endpoint
- [ ] Support workspace generation from analysis
- [ ] Add analysis progress WebSocket endpoint
- [ ] Integrate with live refresh

---

### 3.4 Documentation (Week 12)

**Goal**: Complete documentation for users and contributors.

#### User Documentation

**File**: `docs/features/code-analysis.md` (NEW)

- [ ] Getting started guide
- [ ] Supported languages and frameworks
- [ ] Configuration options
- [ ] CLI usage examples
- [ ] Troubleshooting guide

#### Developer Documentation

**File**: `docs/development/analysis-architecture.md` (NEW)

- [ ] Architecture overview
- [ ] Adding new language analyzers
- [ ] Creating custom patterns
- [ ] LSP integration guide
- [ ] Contributing guidelines

#### API Documentation

- [ ] Rustdoc for all public APIs
- [ ] Example code in documentation
- [ ] Integration examples

---

## Phase 4: Future Languages (Post-MVP)

### 4.1 Python Analyzer

- [ ] Tree-sitter-python integration
- [ ] Pylsp/mypy integration
- [ ] Django pattern detection
- [ ] FastAPI pattern detection
- [ ] Flask pattern detection

### 4.2 C# Analyzer

- [ ] Tree-sitter-c-sharp integration
- [ ] OmniSharp integration
- [ ] ASP.NET Core pattern detection
- [ ] Entity Framework detection

### 4.3 Java Analyzer

- [ ] Tree-sitter-java integration
- [ ] Eclipse JDT integration
- [ ] Spring Boot pattern detection
- [ ] Maven/Gradle dependency analysis

### 4.4 C/C++ Analyzer

- [ ] Tree-sitter-c/cpp integration
- [ ] Libclang integration
- [ ] CMake project detection
- [ ] Header dependency analysis

---

## Dependencies to Add

### Cargo.toml Updates

```toml
[dependencies]
# Tree-sitter parsers
tree-sitter = "0.24"
tree-sitter-typescript = "0.23"
tree-sitter-go = "0.23"

# Async runtime
tokio = { version = "1", features = ["full", "process"] }

# Parallel processing
rayon = "1.10"

# LSP protocol
lsp-types = "0.95"
tower-lsp = "0.20"

# Configuration parsing
serde_yaml = "0.9"

# API spec parsing
openapiv3 = "2.0"
graphql-parser = "0.4"
protobuf-parse = "3"

# Caching
dashmap = "5"
```

---

## Success Criteria

| Metric | Target | Measurement Method |
|--------|--------|-------------------|
| Analysis Speed | <60s for 100K LOC | Benchmark suite |
| TypeScript Accuracy | >75% relationships | Manual validation |
| Go Accuracy | >75% relationships | Manual validation |
| Pattern Detection | >80% common patterns | Known project tests |
| Cross-language | >70% API matching | Contract comparison |

---

## Risk Mitigation

| Risk | Mitigation | Status |
|------|------------|--------|
| LSP server unavailable | Graceful degradation to tree-sitter only | [ ] Planned |
| Performance at scale | Incremental analysis + caching | [ ] Planned |
| False positive relationships | Confidence scoring + filtering | [ ] Planned |
| Parser maintenance burden | Rely on tree-sitter community | [ ] Accepted |

---

## Notes & Deviations Log

*Record any deviations from the plan, discovered issues, or important decisions here.*

| Date | Note |
|------|------|
| 2026-01-07 | Initial specification created from planning session |
| | |

---

## References

- [Tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
- [C4 Model](https://c4model.com/)
- [Structurizr DSL](https://docs.structurizr.com/dsl/language)
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
- [GraphQL Specification](https://spec.graphql.org/)
- [Protocol Buffers](https://protobuf.dev/)
