# Horizon Platform Documentation

## System Overview

The Horizon Platform is a cloud-based integrated development environment (IDE) that enables developers to write, run, and deploy code entirely from a web browser. Unlike traditional local development setups, this platform provides instant access to fully-configured development environments with zero installation required.

**Key Metrics:**
- **Concurrent Users**: 100K+ simultaneous developers
- **Active Workspaces**: 1M+ development environments
- **Languages Supported**: 50+ programming languages
- **AI Completions/Day**: 10M+ code suggestions
- **Container Cold Start**: <2 seconds
- **Collaboration Latency**: <100ms sync

## Architecture Documentation

| Document | Description |
|----------|-------------|
| [002_Architecture](002_Architecture.md) | System design, technology stack, component interactions |
| [003_IDE_Subsystem](003_IDE_Subsystem.md) | Code editor, file browser, terminal, preview |
| [004_Runtime_Subsystem](004_Runtime_Subsystem.md) | Container orchestration, Nix environments, execution |
| [005_Collaboration](005_Collaboration.md) | Real-time editing, presence, permissions |
| [006_AI_Agent](006_AI_Agent.md) | Multi-agent AI (Claude SDK), code generation, debugging |
| [007_API](007_API.md) | REST, WebSocket, gRPC specifications |
| [008_Security](008_Security.md) | Sandboxing, authentication (Keycloak), compliance |
| [009_Infrastructure](009_Infrastructure.md) | Kubernetes deployment, databases |
| [010_Roadmap](010_Roadmap.md) | Implementation phases and milestones |
| [011_Runbook](011_Runbook.md) | Operations procedures and incident response |

## Key Capabilities

### IDE & Editor
- **Monaco Editor**: VS Code's editing experience in the browser
- **50+ Language Support**: Syntax highlighting, IntelliSense, diagnostics
- **Integrated Terminal**: Full PTY with scrollback and multiple tabs
- **File Browser**: Drag-and-drop, search, version history
- **Live Preview**: Hot reload with automatic port detection

### Container Runtime
- **Nix Environments**: Reproducible, declarative package management
- **gVisor Sandboxing**: Secure isolation for user code
- **Sub-2s Cold Start**: Instant workspace availability
- **256GB Storage**: Generous per-workspace storage
- **Resource Quotas**: Fair scheduling across users

### Collaboration
- **Real-time Editing**: Conflict-free CRDT-based synchronization
- **Multiplayer Cursors**: See collaborators' positions and selections
- **Presence Awareness**: Know who's online and active
- **Role-Based Access**: Owner, Editor, Viewer permissions
- **Share Links**: Easy workspace sharing with expiration

### AI Agent
- **Code Generation**: Natural language to code transformation
- **Inline Completions**: Context-aware suggestions as you type
- **Debugging Assistant**: Error explanation and fix suggestions
- **Multi-Agent System**: Specialized agents (Manager, Editor, Debugger, Reviewer)
- **ReAct Loops**: Iterative refinement for complex tasks
- **Claude SDK**: Primary AI provider with factory pattern for multi-provider support

### Deployments
- **Autoscale**: Automatic scaling based on traffic
- **Static Sites**: Optimized hosting for static content
- **Reserved VMs**: Dedicated resources for production
- **Custom Domains**: Bring your own domain with SSL
- **Instant Previews**: Every workspace gets a preview URL

## Integration Guide

### Creating a Workspace

```http
POST /api/v1/workspaces
Authorization: Bearer {token}
Content-Type: application/json

{
  "name": "my-project",
  "language": "python",
  "template": "flask-starter"
}
```

### WebSocket Connection (Crosis Protocol)

```typescript
import { Client } from '@horizon/crosis';

const client = new Client<Context>();

await client.open({
  fetchConnectionMetadata: async () => ({
    token: await getToken(),
    gurl: 'wss://workspace.horizonplatform.io',
    conmanURL: 'https://conman.horizonplatform.io'
  })
});

// Open a shell channel
const shellChannel = await client.openChannel({ service: 'shell' });
shellChannel.onCommand((cmd) => {
  console.log('Output:', cmd.output);
});
shellChannel.send({ input: 'echo Hello World\n' });
```

### AI Code Generation

```http
POST /api/v1/ai/generate
Authorization: Bearer {token}
Content-Type: application/json

{
  "prompt": "Create a REST API endpoint for user authentication",
  "context": {
    "language": "python",
    "framework": "fastapi",
    "files": ["main.py", "models.py"]
  }
}
```

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Editor Keystroke Latency | <50ms | 35ms |
| Container Cold Start | <2s | 1.8s |
| Collaboration Sync | <100ms | 85ms |
| AI Response Time | <3s | 2.5s |
| Workspace Persistence | 99.99% | 99.99% |
| System Availability | 99.9% | 99.95% |
| API Response (P99) | <200ms | 150ms |

## Event Streaming

Subscribe to workspace events via NATS JetStream/WebSocket (see [ADR-018](../adrs/018_NATS_Messaging_Platform.md)):

| Subject | Description |
|---------|-------------|
| `horizon.events.workspace.created` | New workspace created |
| `horizon.events.container.started` | Container started |
| `horizon.events.file.changed` | File modified |
| `horizon.events.deployment.completed` | App deployed |
| `horizon.events.collaboration.room.joined` | User joined session |
| `horizon.events.ai.completion` | AI suggestion generated |

## Support

- **Documentation**: https://docs.horizonplatform.io
- **API Reference**: https://api.horizonplatform.io/docs
- **Status Page**: https://status.horizonplatform.io
- **Community**: https://community.horizonplatform.io
- **Enterprise Support**: enterprise@horizonplatform.io
