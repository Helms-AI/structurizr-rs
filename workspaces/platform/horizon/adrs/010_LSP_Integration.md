# ADR-010: Language Server Protocol Integration

## Status

Accepted

## Context

The platform requires intelligent code assistance for 50+ programming languages:

- Autocomplete/IntelliSense
- Go to definition
- Find references
- Hover information
- Diagnostics (errors, warnings)
- Code actions (quick fixes)
- Rename refactoring
- Signature help

**Challenges:**
- Language servers are memory-intensive (100MB-1GB each)
- Different servers for each language
- Network latency between client and server
- Connection management at scale
- Consistent experience across languages

## Decision

We will implement an **LSP Hub service** that proxies Language Server Protocol requests between the browser and language server instances running in workspace containers.

**Key Design:**

1. **LSP Proxy**: WebSocket-to-stdio bridge for each language
2. **On-Demand Spawning**: Language servers start when needed
3. **Connection Pooling**: Reuse servers across requests
4. **Workspace-Local**: Servers run inside workspace containers

## Alternatives Considered

### Cloud-Hosted Language Servers

**Pros:**
- Centralized management
- Shared resources
- Easier scaling

**Cons:**
- High latency for file operations
- Security concerns (server sees all code)
- Complex file synchronization
- Limited to supported languages

**Why Rejected:** Latency and security concerns outweigh operational benefits.

### Client-Side Language Analysis

**Pros:**
- Zero network latency
- Works offline
- No server resources needed

**Cons:**
- Browser memory constraints
- Limited analysis capability
- No access to full project context
- JavaScript/WASM only

**Why Rejected:** Cannot provide full LSP feature set in browser.

### Third-Party LSP Services (Sourcegraph, GitHub)

**Pros:**
- Pre-built infrastructure
- Advanced features (cross-repo)
- Enterprise support

**Cons:**
- Vendor dependency
- Cost at scale
- Limited customization
- Data sovereignty concerns

**Why Rejected:** Need full control for custom features and privacy.

## Consequences

### Positive

- **Full LSP support**: All protocol features available
- **Language parity**: Same experience for all languages
- **Project context**: Server has access to full workspace
- **Extensibility**: Easy to add new language servers
- **Security**: Code stays in user's container

### Negative

- **Memory overhead**: Each workspace needs server RAM
- **Cold start**: First request spawns server (~1-5s)
- **Complexity**: Managing many server processes
- **Network hops**: Client → Hub → Container → Server

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Memory overhead | Shared servers for popular languages, lazy loading |
| Cold start | Prewarmed servers for common languages |
| Complexity | Unified supervisor, health monitoring |
| Network hops | Efficient binary protocol, connection keepalive |

## Implementation

### LSP Hub Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────────────┐
│   Browser   │────▶│   LSP Hub   │────▶│  Workspace Container │
│   Monaco    │◀────│   Service   │◀────│                     │
│             │ WS  │             │Crosis│  ┌───────────────┐  │
└─────────────┘     └─────────────┘     │  │ LSP Supervisor │  │
                                        │  │   (per lang)   │  │
                                        │  │  ┌─────────┐   │  │
                                        │  │  │ Python  │   │  │
                                        │  │  │  pylsp  │   │  │
                                        │  │  └─────────┘   │  │
                                        │  │  ┌─────────┐   │  │
                                        │  │  │  TypeScript │   │
                                        │  │  │  tsserver│  │  │
                                        │  │  └─────────┘   │  │
                                        │  └───────────────┘  │
                                        └─────────────────────┘
```

### LSP Hub Service

```go
package lsphub

type LSPHub struct {
    workspaces map[string]*WorkspaceConnection
    servers    map[string]*LSPServer
    mu         sync.RWMutex
}

type WorkspaceConnection struct {
    WorkspaceID string
    CrosisConn  *crosis.Connection
    Servers     map[string]*LSPServer
}

type LSPServer struct {
    Language  string
    ProcessID int
    Stdin     io.WriteCloser
    Stdout    io.ReadCloser
    Status    ServerStatus
    StartedAt time.Time
}

func (h *LSPHub) HandleRequest(ctx context.Context, req LSPRequest) (*LSPResponse, error) {
    h.mu.RLock()
    ws, exists := h.workspaces[req.WorkspaceID]
    h.mu.RUnlock()

    if !exists {
        return nil, ErrWorkspaceNotConnected
    }

    // Get or spawn language server
    server, err := h.getOrSpawnServer(ctx, ws, req.Language)
    if err != nil {
        return nil, err
    }

    // Forward request to language server
    return h.forwardRequest(ctx, server, req)
}

func (h *LSPHub) getOrSpawnServer(ctx context.Context, ws *WorkspaceConnection, language string) (*LSPServer, error) {
    ws.mu.Lock()
    defer ws.mu.Unlock()

    // Check if server already running
    if server, exists := ws.Servers[language]; exists && server.Status == StatusRunning {
        return server, nil
    }

    // Spawn new server in container
    server, err := h.spawnServer(ctx, ws, language)
    if err != nil {
        return nil, err
    }

    ws.Servers[language] = server
    return server, nil
}

func (h *LSPHub) spawnServer(ctx context.Context, ws *WorkspaceConnection, language string) (*LSPServer, error) {
    config := LanguageServerConfigs[language]
    if config == nil {
        return nil, ErrUnsupportedLanguage
    }

    // Execute command in container via Crosis
    execReq := &crosis.ExecRequest{
        Args:       config.Command,
        Env:        config.Env,
        Background: true,
    }

    resp, err := ws.CrosisConn.Exec(ctx, execReq)
    if err != nil {
        return nil, err
    }

    server := &LSPServer{
        Language:  language,
        ProcessID: resp.PID,
        Stdin:     resp.Stdin,
        Stdout:    resp.Stdout,
        Status:    StatusRunning,
        StartedAt: time.Now(),
    }

    // Start reading responses
    go h.readServerOutput(ws.WorkspaceID, server)

    return server, nil
}
```

### Language Server Configurations

```go
var LanguageServerConfigs = map[string]*LSPConfig{
    "python": {
        Command:   []string{"pylsp"},
        Env:       []string{"PYTHONPATH=/nix/store/.../lib/python3.11/site-packages"},
        InitOptions: map[string]interface{}{
            "pylsp.plugins.pycodestyle.enabled":  true,
            "pylsp.plugins.pyflakes.enabled":     true,
            "pylsp.plugins.autopep8.enabled":     true,
        },
    },
    "typescript": {
        Command: []string{
            "typescript-language-server",
            "--stdio",
        },
        InitOptions: map[string]interface{}{
            "tsserver": map[string]interface{}{
                "logVerbosity": "off",
            },
        },
    },
    "go": {
        Command: []string{"gopls", "serve"},
        Env:     []string{"GOPATH=/home/runner/go"},
        InitOptions: map[string]interface{}{
            "staticcheck":   true,
            "gofumpt":       true,
            "usePlaceholders": true,
        },
    },
    "rust": {
        Command: []string{"rust-analyzer"},
        InitOptions: map[string]interface{}{
            "checkOnSave": map[string]interface{}{
                "command": "clippy",
            },
            "cargo": map[string]interface{}{
                "loadOutDirsFromCheck": true,
            },
        },
    },
    "java": {
        Command: []string{
            "java",
            "-jar", "/nix/store/.../jdtls/plugins/org.eclipse.equinox.launcher.jar",
            "-configuration", "/nix/store/.../jdtls/config_linux",
            "-data", "/home/runner/.jdtls-workspace",
        },
    },
}
```

### Monaco LSP Client

```typescript
import {
  MonacoLanguageClient,
  MessageConnection,
  CloseAction,
  ErrorAction,
} from 'monaco-languageclient';
import {
  toSocket,
  WebSocketMessageReader,
  WebSocketMessageWriter,
} from 'vscode-ws-jsonrpc';

class LSPClient {
  private clients: Map<string, MonacoLanguageClient> = new Map();
  private connections: Map<string, WebSocket> = new Map();

  async connect(language: string, workspaceId: string): Promise<void> {
    if (this.clients.has(language)) {
      return;
    }

    // Connect to LSP Hub via WebSocket
    const wsUrl = `wss://lsp.horizonplatform.io/${workspaceId}/${language}`;
    const ws = new WebSocket(wsUrl);

    await new Promise<void>((resolve, reject) => {
      ws.onopen = () => resolve();
      ws.onerror = (e) => reject(e);
    });

    this.connections.set(language, ws);

    // Create JSON-RPC connection
    const socket = toSocket(ws);
    const reader = new WebSocketMessageReader(socket);
    const writer = new WebSocketMessageWriter(socket);

    // Create language client
    const client = new MonacoLanguageClient({
      name: `${language} Language Client`,
      clientOptions: {
        documentSelector: [{ language }],
        errorHandler: {
          error: () => ErrorAction.Continue,
          closed: () => CloseAction.Restart,
        },
        synchronize: {
          fileEvents: [
            monaco.languages.createFileSystemWatcher('**/*'),
          ],
        },
      },
      connectionProvider: {
        get: () => Promise.resolve({ reader, writer }),
      },
    });

    // Start the client
    client.start();
    this.clients.set(language, client);

    // Register Monaco providers
    this.registerProviders(language);
  }

  private registerProviders(language: string): void {
    // Completion provider (already handled by Monaco language client)
    // Additional custom providers can be added here

    // Custom inline completion for AI suggestions
    monaco.languages.registerInlineCompletionsProvider(language, {
      provideInlineCompletionItems: async (model, position, context, token) => {
        // AI completions handled separately
        return { items: [] };
      },
    });
  }

  disconnect(language: string): void {
    const client = this.clients.get(language);
    if (client) {
      client.stop();
      this.clients.delete(language);
    }

    const ws = this.connections.get(language);
    if (ws) {
      ws.close();
      this.connections.delete(language);
    }
  }
}
```

### LSP Supervisor (in Container)

```go
package supervisor

type LSPSupervisor struct {
    servers  map[string]*ManagedServer
    mu       sync.RWMutex
    config   SupervisorConfig
}

type ManagedServer struct {
    Language    string
    Process     *exec.Cmd
    Stdin       io.WriteCloser
    Stdout      io.ReadCloser
    Health      HealthStatus
    LastRequest time.Time
}

func (s *LSPSupervisor) Start(ctx context.Context) error {
    // Start health check loop
    go s.healthCheckLoop(ctx)

    // Start idle timeout loop
    go s.idleTimeoutLoop(ctx)

    return nil
}

func (s *LSPSupervisor) GetOrStart(language string) (*ManagedServer, error) {
    s.mu.Lock()
    defer s.mu.Unlock()

    // Return existing server
    if server, exists := s.servers[language]; exists {
        server.LastRequest = time.Now()
        return server, nil
    }

    // Start new server
    config, ok := LanguageServerConfigs[language]
    if !ok {
        return nil, fmt.Errorf("unsupported language: %s", language)
    }

    cmd := exec.Command(config.Command[0], config.Command[1:]...)
    cmd.Env = append(os.Environ(), config.Env...)

    stdin, err := cmd.StdinPipe()
    if err != nil {
        return nil, err
    }

    stdout, err := cmd.StdoutPipe()
    if err != nil {
        return nil, err
    }

    if err := cmd.Start(); err != nil {
        return nil, err
    }

    server := &ManagedServer{
        Language:    language,
        Process:     cmd,
        Stdin:       stdin,
        Stdout:      stdout,
        Health:      HealthOK,
        LastRequest: time.Now(),
    }

    s.servers[language] = server

    // Wait for initialization
    if err := s.waitForInit(server, config); err != nil {
        s.stopServer(server)
        return nil, err
    }

    return server, nil
}

func (s *LSPSupervisor) healthCheckLoop(ctx context.Context) {
    ticker := time.NewTicker(30 * time.Second)
    defer ticker.Stop()

    for {
        select {
        case <-ctx.Done():
            return
        case <-ticker.C:
            s.mu.Lock()
            for lang, server := range s.servers {
                if !s.isHealthy(server) {
                    log.Warn("Unhealthy language server", "language", lang)
                    s.restartServer(server)
                }
            }
            s.mu.Unlock()
        }
    }
}

func (s *LSPSupervisor) idleTimeoutLoop(ctx context.Context) {
    ticker := time.NewTicker(time.Minute)
    defer ticker.Stop()

    for {
        select {
        case <-ctx.Done():
            return
        case <-ticker.C:
            s.mu.Lock()
            for lang, server := range s.servers {
                if time.Since(server.LastRequest) > s.config.IdleTimeout {
                    log.Info("Stopping idle language server", "language", lang)
                    s.stopServer(server)
                    delete(s.servers, lang)
                }
            }
            s.mu.Unlock()
        }
    }
}
```

### Request Routing

```go
func (h *LSPHub) forwardRequest(ctx context.Context, server *LSPServer, req LSPRequest) (*LSPResponse, error) {
    // Encode LSP request as JSON-RPC
    rpcReq := &jsonrpc.Request{
        ID:     req.ID,
        Method: req.Method,
        Params: req.Params,
    }

    data, err := json.Marshal(rpcReq)
    if err != nil {
        return nil, err
    }

    // Add Content-Length header
    header := fmt.Sprintf("Content-Length: %d\r\n\r\n", len(data))

    // Write to server stdin
    if _, err := server.Stdin.Write([]byte(header)); err != nil {
        return nil, err
    }
    if _, err := server.Stdin.Write(data); err != nil {
        return nil, err
    }

    // Wait for response (with timeout)
    select {
    case resp := <-server.ResponseChan:
        return resp, nil
    case <-time.After(30 * time.Second):
        return nil, ErrRequestTimeout
    case <-ctx.Done():
        return nil, ctx.Err()
    }
}
```

## Supported Languages

| Language | Server | Memory | Cold Start |
|----------|--------|--------|------------|
| Python | pylsp | ~150MB | ~2s |
| TypeScript | tsserver | ~200MB | ~3s |
| JavaScript | tsserver | ~200MB | ~3s |
| Go | gopls | ~100MB | ~1s |
| Rust | rust-analyzer | ~500MB | ~5s |
| Java | jdtls | ~800MB | ~10s |
| C/C++ | clangd | ~300MB | ~3s |
| Ruby | solargraph | ~200MB | ~3s |
| PHP | intelephense | ~150MB | ~2s |
| Kotlin | kotlin-language-server | ~400MB | ~5s |

## References

- [Language Server Protocol Specification](https://microsoft.github.io/language-server-protocol/)
- [Monaco Language Client](https://github.com/TypeFox/monaco-languageclient)
- [gopls Documentation](https://pkg.go.dev/golang.org/x/tools/gopls)
- [rust-analyzer Manual](https://rust-analyzer.github.io/manual.html)
