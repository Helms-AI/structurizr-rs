# Horizon Platform - Cloud IDE

A comprehensive implementation blueprint for building a cloud-based integrated development environment with AI-powered coding assistance, real-time collaboration, and instant deployments.

## Overview

This workspace documents the architecture of the Horizon Platform - a browser-based IDE that enables developers to write, run, and deploy code without local setup. The design supports:

- **100K+ concurrent users** with isolated development environments
- **50+ programming languages** via Nix-based reproducible environments
- **Real-time collaboration** using CRDT-based multiplayer editing
- **AI pair programming** with Claude-powered multi-agent code generation
- **Instant deployments** with autoscaling and custom domains

## Key Features

| Feature | Description |
|---------|-------------|
| **Web IDE** | Monaco-based editor with IntelliSense, terminal, and live preview |
| **Container Runtime** | gVisor-sandboxed containers with Nix environments |
| **Collaboration** | Yjs CRDT for conflict-free real-time editing |
| **AI Agent** | Multi-agent system (Manager, Editor, Debugger, Reviewer) powered by Claude |
| **Deployments** | Autoscale, Static, Reserved VM deployment types |

## Architecture Highlights

```
┌─────────────────────────────────────────────────────────────────┐
│                        Web IDE (React/Monaco)                    │
└─────────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        ┌──────────┐   ┌──────────────┐  ┌──────────┐
        │API Gateway│   │WebSocket GW  │  │AI Gateway│
        │  (Kong)   │   │  (Crosis)    │  │ (FastAPI)│
        └──────────┘   └──────────────┘  └──────────┘
              │               │               │
    ┌─────────┴─────────┬─────┴───────┬───────┴────────┐
    ▼                   ▼             ▼                ▼
┌─────────┐      ┌───────────┐  ┌──────────┐   ┌──────────────┐
│Workspace│      │Container  │  │Collab    │   │AI Agent      │
│Service  │      │Orchestrator│ │Engine    │   │Orchestrator  │
└─────────┘      └───────────┘  └──────────┘   └──────────────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
        ┌──────────┐     ┌───────────┐
        │Nix Env   │     │Execution  │
        │Service   │     │Engine     │
        └──────────┘     └───────────┘
```

## Documentation

- **[001_Overview](docs/001_Overview.md)** - Main documentation hub
- **[002_Architecture](docs/002_Architecture.md)** - System design and technology stack
- **[003_IDE_Subsystem](docs/003_IDE_Subsystem.md)** - Code editor and terminal
- **[004_Runtime_Subsystem](docs/004_Runtime_Subsystem.md)** - Container and Nix environments
- **[005_Collaboration](docs/005_Collaboration.md)** - Real-time multiplayer editing
- **[006_AI_Agent](docs/006_AI_Agent.md)** - AI code generation system (Claude SDK)
- **[007_API](docs/007_API.md)** - REST and WebSocket APIs
- **[008_Security](docs/008_Security.md)** - Sandboxing and authentication (Keycloak)
- **[009_Infrastructure](docs/009_Infrastructure.md)** - Kubernetes deployment
- **[010_Roadmap](docs/010_Roadmap.md)** - Implementation phases
- **[011_Runbook](docs/011_Runbook.md)** - Operations guide

## Architecture Decision Records

| ADR | Title |
|-----|-------|
| [001](adrs/001_Container_Orchestration.md) | Kubernetes + gVisor for secure multi-tenant execution |
| [002](adrs/002_Crosis_Protocol.md) | Protobuf over WebSocket with channel multiplexing |
| [003](adrs/003_Nix_Environment_Management.md) | Declarative environments with 1TB shared store |
| [004](adrs/004_CRDT_Collaboration.md) | Yjs for conflict-free real-time editing |
| [005](adrs/005_Multi_Agent_AI.md) | Specialized agents with ReAct loops |
| [006](adrs/006_Tiered_Storage.md) | Redis + MinIO + PostgreSQL storage tiers |
| [007](adrs/007_Code_Editor_Engine.md) | Monaco Editor (VS Code core) |
| [008](adrs/008_Security_Sandboxing.md) | gVisor + Seccomp + Network policies |
| [009](adrs/009_Multi_Region_Infrastructure.md) | Cloud-agnostic Kubernetes deployment |
| [010](adrs/010_LSP_Integration.md) | On-demand language server spawning |
| [011](adrs/011_Preview_Deployment.md) | Instant preview URLs |
| [012](adrs/012_Design_System.md) | Emotion CSS-in-JS with TypeScript |
| [013](adrs/013_Auth_Migration.md) | Auth0 to Keycloak migration |
| [014](adrs/014_Vector_DB_Migration.md) | Pinecone to Qdrant migration |
| [015](adrs/015_Payment_Removal.md) | Payment processing removal |
| [016](adrs/016_Cloud_Abstraction.md) | GCP to generic Kubernetes |
| [017](adrs/017_Claude_SDK_Migration.md) | LangChain to Claude Agent SDK |
| [018](adrs/018_NATS_Messaging_Platform.md) | NATS as unified messaging platform |

## C4 Model Views

This workspace includes the following Structurizr views:

- **System Landscape** (`horizon-landscape`) - Complete ecosystem
- **System Context** (`horizon-context`) - Platform in context
- **Container View** (`horizon-containers`) - All services
- **Component Views** - IDE, Gateway, Orchestrator, AI Agent, Collaboration, Nix, Deployment
- **Dynamic Views** - Code execution, AI completion, collaboration, workspace creation, deployment flows
- **Deployment View** (`horizon-deployment`) - Kubernetes production infrastructure

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| **Frontend** | React, TypeScript, Monaco Editor, xterm.js, Yjs |
| **API** | Kong, Go (WebSocket Gateway), FastAPI |
| **Core Services** | Go, Rust, TypeScript |
| **AI** | Python, Claude Code Agent SDK, Qdrant |
| **Runtime** | Kubernetes, gVisor, Nix |
| **Data** | PostgreSQL, Redis, MinIO (S3-compatible), ClickHouse, Qdrant |
| **Auth** | Keycloak (OAuth 2.0, OIDC) |
| **Infrastructure** | Kubernetes (cloud-agnostic), Helm, cert-manager |

## Local Development

```bash
# Start infrastructure services
cd workspaces/platform/horizon
docker-compose up -d

# View the architecture diagrams
cargo run -- serve --workspaces-dir workspaces

# Navigate to: http://localhost:8080/w/platform/horizon
```

### Available Services (Development)

| Service | Port | Description |
|---------|------|-------------|
| PostgreSQL | 5432 | Primary database (metadata, users, workspaces) |
| Redis | 6379 | Application caching and sessions |
| MinIO | 9000/9001 | Object storage (S3-compatible) |
| Qdrant | 6333/6334 | Vector database (AI embeddings) |
| Keycloak | 8180 | Identity provider (OAuth 2.0, OIDC) |
| NATS | 4222/8222/9222 | Messaging platform (JetStream, KV, Object Store, WebSocket) |
| ClickHouse | 8123/9004 | Analytics database (usage metrics, time-series) |
| Elasticsearch | 9200 | Code search service |
| Prometheus | 9090 | Metrics collection |
| Grafana | 3001 | Dashboards and visualization |
| Jaeger | 16686 | Distributed tracing |

## References

- [Anthropic Claude Documentation](https://docs.anthropic.com)
- [Keycloak Documentation](https://www.keycloak.org/documentation)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Nix Package Manager](https://nixos.org)
- [Monaco Editor](https://microsoft.github.io/monaco-editor/)
- [Yjs CRDT](https://yjs.dev)
