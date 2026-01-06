# MCP Integration Phases Specification

> **Status**: Phase 7-8 Complete, Phase 9+ Pending
> **Last Updated**: 2026-01-06
> **Author**: Claude Code

This document specifies the remaining work for the MCP (Model Context Protocol) integration in structurizr-rs.

---

## Completed Phases

### Phase 7: rmcp Upgrade and Web Proxy (COMPLETE)

**Summary**: Upgraded rmcp from v0.5 to v0.12, implemented WebSocket/SSE transports, created web proxy integration.

**Files Modified**:
- `crates/structurizr-mcp/Cargo.toml` - rmcp v0.12
- `crates/structurizr-mcp/src/server.rs` - Multi-transport support
- `crates/structurizr-web/src/mcp_proxy.rs` - MCP proxy module (NEW)
- `crates/structurizr-web/src/server.rs` - Proxy route integration
- `Cargo.toml` - Feature flags for mcp-websocket, mcp-http, mcp-all

**Endpoints**:
- `GET /mcp/health` - Health check endpoint
- `GET /mcp/ws` - WebSocket proxy endpoint
- `ANY /mcp` - Streamable HTTP proxy endpoint (GET for SSE, POST for JSON-RPC, DELETE for session)

---

### Phase 8: Configuration System (COMPLETE)

**Summary**: Created `structurizr-config` crate with TOML configuration, workspace scoping, CRDT foundation.

**Files Created**:
- `crates/structurizr-config/Cargo.toml`
- `crates/structurizr-config/src/lib.rs` - Main config with profiles
- `crates/structurizr-config/src/scope.rs` - Workspace scoping with globs
- `crates/structurizr-config/src/crdt.rs` - Custom CRDT implementation
- `crates/structurizr-config/src/validation.rs` - Config validation
- `crates/structurizr-config/src/discovery.rs` - Config file discovery
- `structurizr.toml` - Default configuration file

**Key Features**:
- TOML configuration with profile support (development, production)
- Workspace scoping: `all`, `allow`, `deny` modes with glob patterns
- Auto-include workspaces created during session
- CRDT foundation with vector clocks
- Config file discovery: project → user → system → defaults

---

## Remaining Phases

### Phase 9: Real-time Collaboration Integration (IN PROGRESS)

**Current State**: `collaboration.rs` created but not integrated into web server.

#### 9.1 Web Server Integration

**File**: `crates/structurizr-web/src/lib.rs`

Add collaboration module export:
```rust
pub mod collaboration;
pub use collaboration::{CollaborationServer, CollaborationConfig};
```

**File**: `crates/structurizr-web/src/server.rs`

Add collaboration server to state and routes:
```rust
use crate::collaboration::{CollaborationServer, CollaborationConfig, collaboration_ws_handler};

// In Server::new() or run():
let collab_config = CollaborationConfig {
    crdt_enabled: config.collaboration.crdt.enabled,
    presence_heartbeat_ms: config.collaboration.presence_heartbeat_ms,
    presence_timeout_ms: config.collaboration.presence_timeout_ms,
    max_clients_per_workspace: config.collaboration.max_clients_per_workspace,
};
let collab_server = Arc::new(CollaborationServer::new(collab_config));

// Add route:
.route("/ws/collab", get(collaboration_ws_handler))
// Add extension:
.layer(Extension(collab_server.clone()))
```

#### 9.2 File Watcher Integration

**File**: `crates/structurizr-web/src/watcher.rs`

Integrate collaboration notifications:
```rust
impl FileWatcher {
    pub fn with_collaboration(self, collab: Arc<CollaborationServer>) -> Self {
        // Store collab server reference
    }

    async fn handle_change(&self, path: &Path, change_type: &str) {
        // Existing hot-reload logic...

        // Notify collaboration server
        if let Some(workspace_id) = self.path_to_workspace(path) {
            self.collab.notify_file_change(&workspace_id, path.to_str().unwrap(), change_type).await;
        }
    }
}
```

#### 9.3 Configuration Bridge

**File**: `crates/structurizr-config/src/lib.rs`

Add collaboration config to main Config struct (already exists, verify fields):
```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CollaborationConfig {
    #[serde(default = "bool_true")]
    pub enabled: bool,

    #[serde(default = "default_notification_transport")]
    pub notification_transport: String,  // "websocket" | "sse"

    #[serde(default)]
    pub crdt: CrdtConfig,

    #[serde(default)]
    pub file_watch: FileWatchConfig,

    #[serde(default = "default_presence_heartbeat")]
    pub presence_heartbeat_ms: u64,

    #[serde(default = "default_presence_timeout")]
    pub presence_timeout_ms: u64,

    #[serde(default = "default_max_clients")]
    pub max_clients_per_workspace: usize,
}
```

#### 9.4 Testing Requirements

1. **Unit Tests** (in `collaboration.rs`):
   - [x] Session creation/cleanup
   - [x] Client join/leave
   - [x] Cursor updates
   - [ ] CRDT operation application
   - [ ] Presence aggregation
   - [ ] File change notifications

2. **Integration Tests** (new file `tests/collaboration_integration.rs`):
   - [ ] WebSocket connection lifecycle
   - [ ] Multi-client synchronization
   - [ ] CRDT convergence
   - [ ] File watcher → collaboration notification flow

3. **Manual Testing**:
   - [ ] Connect via `websocat ws://localhost:8080/ws/collab`
   - [ ] Send join message: `{"type":"Join","workspace_id":"test","name":"User1"}`
   - [ ] Verify presence updates

---

### Phase 10: Full CRDT Implementation

**Goal**: Complete CRDT integration for true collaborative editing.

#### 10.1 CRDT Document Persistence

**File**: `crates/structurizr-config/src/crdt.rs`

Add persistence methods:
```rust
impl CrdtDocument {
    /// Save CRDT state to disk alongside workspace
    pub async fn save_to_file(&self, path: &Path) -> Result<(), std::io::Error> {
        let state = CrdtPersistentState {
            content: self.get_content().await,
            operations: self.operations.read().await.clone(),
            clock: self.clock.read().await.clone(),
        };
        let json = serde_json::to_string_pretty(&state)?;
        tokio::fs::write(path.join(".crdt-state.json"), json).await
    }

    /// Load CRDT state from disk
    pub async fn load_from_file(path: &Path) -> Result<Self, std::io::Error> {
        let json = tokio::fs::read_to_string(path.join(".crdt-state.json")).await?;
        let state: CrdtPersistentState = serde_json::from_str(&json)?;
        // Reconstruct CrdtDocument from state
    }
}
```

#### 10.2 CRDT ↔ File Synchronization

When DSL file changes on disk:
1. Parse new content
2. Generate diff operations
3. Apply as CRDT operations
4. Broadcast to connected clients

When CRDT content changes:
1. Debounce rapid edits (500ms)
2. Write CRDT content to DSL file
3. File watcher ignores self-triggered changes
4. Workspace re-validates on save

#### 10.3 Conflict Resolution UI

**File**: `crates/structurizr-web/static/js/collaboration.js` (NEW)

```javascript
class CollaborationClient {
    constructor(workspaceId) {
        this.ws = new WebSocket(`ws://${location.host}/ws/collab`);
        this.peers = new Map();
        this.init();
    }

    init() {
        this.ws.onopen = () => this.join();
        this.ws.onmessage = (e) => this.handleMessage(JSON.parse(e.data));
    }

    handleMessage(msg) {
        switch (msg.type) {
            case 'PresenceUpdate':
                this.updatePeers(msg.clients);
                break;
            case 'CursorMoved':
                this.showPeerCursor(msg.client_id, msg.line, msg.column);
                break;
            case 'CrdtOp':
                this.applyRemoteOp(msg.operation);
                break;
        }
    }

    showPeerCursor(clientId, line, column) {
        // Render colored cursor indicator in editor
    }
}
```

---

### Phase 11: MCP Tool Enhancements

**Goal**: Add collaboration-aware MCP tools.

#### 11.1 New Tools

**File**: `crates/structurizr-mcp/src/tools/collaboration.rs` (NEW)

```rust
// Get current collaborators on a workspace
#[tool(
    name = "collaboration_get_presence",
    description = "Get list of users currently collaborating on a workspace"
)]
pub async fn get_presence(
    #[param(description = "Workspace ID")] workspace_id: String,
) -> Result<String, Error> {
    // Return JSON list of { client_id, name, cursor, active_view }
}

// Send cursor position (for AI showing where it's "looking")
#[tool(
    name = "collaboration_set_cursor",
    description = "Set AI cursor position in workspace DSL"
)]
pub async fn set_cursor(
    #[param(description = "Workspace ID")] workspace_id: String,
    #[param(description = "Line number")] line: usize,
    #[param(description = "Column number")] column: usize,
) -> Result<String, Error> {
    // Connect to collaboration server, send cursor update
}

// Collaborative edit via CRDT
#[tool(
    name = "workspace_edit_collaborative",
    description = "Make a collaborative edit that syncs to all connected clients"
)]
pub async fn edit_collaborative(
    #[param(description = "Workspace ID")] workspace_id: String,
    #[param(description = "Line number")] line: usize,
    #[param(description = "Old text to replace")] old_text: String,
    #[param(description = "New text")] new_text: String,
) -> Result<String, Error> {
    // Apply via CRDT operations
}
```

#### 11.2 Tool Registration

**File**: `crates/structurizr-mcp/src/server.rs`

```rust
// In McpServerBuilder:
.tool(tools::collaboration_get_presence)
.tool(tools::collaboration_set_cursor)
.tool(tools::workspace_edit_collaborative)
```

---

### Phase 12: Authentication and Authorization

**Goal**: Add optional authentication for MCP and collaboration endpoints.

#### 12.1 Configuration

**File**: `structurizr.toml`

```toml
[auth]
enabled = false  # Optional, default false
provider = "api-key"  # "api-key" | "oauth" | "jwt"

[auth.api_key]
# API keys are stored in environment or separate secrets file
header_name = "X-API-Key"
env_var = "STRUCTURIZR_API_KEY"

[auth.jwt]
issuer = "https://auth.example.com"
audience = "structurizr"
public_key_url = "https://auth.example.com/.well-known/jwks.json"
```

#### 12.2 Middleware

**File**: `crates/structurizr-web/src/auth.rs` (NEW)

```rust
pub struct AuthLayer {
    config: AuthConfig,
}

impl AuthLayer {
    pub fn new(config: AuthConfig) -> Self { ... }
}

// Axum middleware
pub async fn auth_middleware<B>(
    State(config): State<AuthConfig>,
    request: Request<B>,
    next: Next<B>,
) -> Result<Response, StatusCode> {
    if !config.enabled {
        return Ok(next.run(request).await);
    }

    // Validate based on provider type
    match config.provider.as_str() {
        "api-key" => validate_api_key(&request, &config),
        "jwt" => validate_jwt(&request, &config).await,
        _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }?;

    Ok(next.run(request).await)
}
```

#### 12.3 Protected Routes

```rust
// In server.rs
let protected_routes = Router::new()
    .route("/mcp/ws", get(mcp_ws_handler))
    .route("/mcp", any(mcp_http_handler))  // Streamable HTTP (GET/POST/DELETE)
    .route("/ws/collab", get(collaboration_ws_handler))
    .layer(middleware::from_fn_with_state(auth_config.clone(), auth_middleware));

let public_routes = Router::new()
    .route("/", get(index_handler))
    .route("/w/:workspace_id", get(workspace_handler))
    .route("/mcp/health", get(mcp_health_handler));

let app = public_routes.merge(protected_routes);
```

---

### Phase 13: Metrics and Observability

**Goal**: Add Prometheus metrics and structured logging.

#### 13.1 Metrics Configuration

**File**: `structurizr.toml`

```toml
[metrics]
enabled = true
endpoint = "/metrics"
include_labels = ["workspace_id", "transport", "client_id"]

[metrics.histograms]
request_duration_buckets = [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
```

#### 13.2 Metrics Implementation

**File**: `crates/structurizr-web/src/metrics.rs` (NEW)

```rust
use prometheus::{Counter, Histogram, IntGauge, Registry};

pub struct Metrics {
    pub requests_total: Counter,
    pub request_duration: Histogram,
    pub active_connections: IntGauge,
    pub active_collaborators: IntGauge,
    pub crdt_operations_total: Counter,
    pub workspace_reloads_total: Counter,
}

impl Metrics {
    pub fn new(registry: &Registry) -> Self {
        // Register all metrics with registry
    }
}
```

#### 13.3 Metrics Endpoint

```rust
// In server.rs
.route("/metrics", get(metrics_handler))

async fn metrics_handler(Extension(metrics): Extension<Arc<Metrics>>) -> impl IntoResponse {
    let encoder = prometheus::TextEncoder::new();
    let metric_families = prometheus::gather();
    encoder.encode_to_string(&metric_families).unwrap()
}
```

---

### Phase 14: Multi-Workspace MCP Sessions

**Goal**: Allow a single MCP session to access multiple workspaces based on scoping.

#### 14.1 Session Context

**File**: `crates/structurizr-mcp/src/session.rs` (NEW)

```rust
pub struct McpSession {
    pub id: Uuid,
    pub scope: WorkspaceScope,
    pub created_workspaces: HashSet<String>,
    pub accessed_workspaces: HashSet<String>,
    pub started_at: Instant,
}

impl McpSession {
    pub fn can_access(&self, workspace_id: &str) -> bool {
        self.scope.is_accessible(workspace_id) ||
        self.created_workspaces.contains(workspace_id)
    }

    pub fn register_created(&mut self, workspace_id: String) {
        self.created_workspaces.insert(workspace_id);
    }
}
```

#### 14.2 Scoped Tool Wrapper

```rust
// Wrap all workspace tools with scope check
async fn with_scope_check<F, R>(
    session: &McpSession,
    workspace_id: &str,
    f: F,
) -> Result<R, Error>
where
    F: Future<Output = Result<R, Error>>,
{
    if !session.can_access(workspace_id) {
        return Err(Error::AccessDenied(format!(
            "Workspace '{}' not accessible in this session",
            workspace_id
        )));
    }
    f.await
}
```

---

### Phase 15: WebSocket Transport Improvements

**Goal**: Improve WebSocket reliability and performance.

#### 15.1 Reconnection Logic

**File**: `crates/structurizr-web/static/js/ws-client.js` (NEW)

```javascript
class ReconnectingWebSocket {
    constructor(url, options = {}) {
        this.url = url;
        this.maxRetries = options.maxRetries || 10;
        this.retryDelay = options.retryDelay || 1000;
        this.retryCount = 0;
        this.connect();
    }

    connect() {
        this.ws = new WebSocket(this.url);
        this.ws.onclose = () => this.handleClose();
        this.ws.onerror = () => this.handleError();
    }

    handleClose() {
        if (this.retryCount < this.maxRetries) {
            setTimeout(() => {
                this.retryCount++;
                this.connect();
            }, this.retryDelay * Math.pow(2, this.retryCount));
        }
    }
}
```

#### 15.2 Message Batching

```rust
// In collaboration.rs
pub struct MessageBatcher {
    buffer: Vec<CollaborationEvent>,
    max_batch_size: usize,
    flush_interval: Duration,
    last_flush: Instant,
}

impl MessageBatcher {
    pub fn add(&mut self, event: CollaborationEvent) {
        self.buffer.push(event);
        if self.buffer.len() >= self.max_batch_size ||
           self.last_flush.elapsed() > self.flush_interval {
            self.flush();
        }
    }

    pub fn flush(&mut self) -> Vec<CollaborationEvent> {
        self.last_flush = Instant::now();
        std::mem::take(&mut self.buffer)
    }
}
```

#### 15.3 Compression

```toml
# structurizr.toml
[websocket]
compression = true
compression_threshold_bytes = 1024
```

---

## Implementation Priority

| Phase | Priority | Complexity | Dependencies |
|-------|----------|------------|--------------|
| 9 (Collab Integration) | **HIGH** | Medium | Phase 8 |
| 10 (Full CRDT) | HIGH | High | Phase 9 |
| 11 (MCP Tools) | Medium | Medium | Phase 9, 10 |
| 12 (Auth) | Medium | Medium | None |
| 13 (Metrics) | Low | Low | None |
| 14 (Multi-Workspace) | Medium | Medium | Phase 8 |
| 15 (WS Improvements) | Low | Low | Phase 9 |

---

## Testing Strategy

### Unit Tests

Each new module should have >80% test coverage:
- `collaboration.rs` - Session management, CRDT operations
- `auth.rs` - Token validation, middleware behavior
- `metrics.rs` - Counter increments, histogram observations
- `session.rs` - Scope checking, workspace registration

### Integration Tests

Located in `tests/`:
- `tests/collaboration_e2e.rs` - Full WebSocket flow
- `tests/mcp_auth.rs` - Authenticated MCP requests
- `tests/crdt_convergence.rs` - Multi-client CRDT sync

### Manual Testing Checklist

```bash
# Phase 9 - Collaboration
websocat ws://localhost:8080/ws/collab
> {"type":"Join","workspace_id":"demo","name":"Test User"}
< {"type":"PresenceUpdate","clients":[...]}

# Phase 12 - Auth
curl -H "X-API-Key: test-key" http://localhost:8080/mcp/health

# Phase 13 - Metrics
curl http://localhost:8080/metrics | grep structurizr
```

---

## Migration Notes

### Breaking Changes

None expected. All new features are opt-in via configuration.

### Configuration Defaults

All new features default to disabled or permissive:
- `auth.enabled = false`
- `metrics.enabled = false`
- `websocket.compression = false`
- `mcp.workspace_scope.mode = "all"`

### Upgrade Path

1. Update binary
2. Optionally add new config sections to `structurizr.toml`
3. Restart server

---

## Open Questions

1. **CRDT Algorithm Choice**: Currently using custom implementation. Consider integrating `yrs` (Yjs Rust) for production.

2. **Collaboration Persistence**: Should CRDT state persist across server restarts? Current implementation is in-memory only.

3. **Authentication Providers**: Which OAuth providers should we support? GitHub, Google, Okta?

4. **Rate Limiting**: Should we add rate limiting for MCP tools? What limits?

5. **WebSocket Protocol**: Should we use a standard sub-protocol like `graphql-ws` or keep custom JSON messages?

---

## References

- [MCP Specification](https://modelcontextprotocol.io/)
- [rmcp Crate](https://crates.io/crates/rmcp)
- [CRDT Resources](https://crdt.tech/)
- [Yjs/Yrs](https://github.com/y-crdt/y-crdt)
- [Structurizr DSL](https://docs.structurizr.com/dsl/language)
