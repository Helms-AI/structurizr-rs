# ADR-002: Crosis Protocol for Client-Container Communication

## Status

Accepted

## Context

The platform requires bidirectional real-time communication between the browser-based IDE and workspace containers for:

- Terminal I/O streaming (stdin/stdout/stderr)
- File system operations (read, write, watch)
- Language server protocol (LSP) messages
- Debugger integration
- Process management signals
- Collaboration synchronization

**Requirements:**
- Low latency (<50ms round trip)
- Efficient binary encoding
- Multiplexed channels over single connection
- Auto-reconnect with state recovery
- Cross-platform client support (browser, mobile, CLI)

**Options Considered:**

1. **JSON over WebSocket**
   - Simple, human-readable
   - Higher bandwidth, slower parsing

2. **gRPC-Web**
   - Strong typing, code generation
   - Better for request/response than streaming

3. **Socket.IO**
   - Fallback transports, rooms
   - Higher overhead, less control

4. **Custom Protocol Buffers over WebSocket (Crosis-style)**
   - Efficient binary encoding
   - Channel multiplexing
   - Full control over protocol

## Decision

We will implement a **Crosis-style protocol using Protocol Buffers over WebSocket** with channel multiplexing.

**Key Design:**

1. **Single WebSocket Connection**: One persistent connection per workspace session
2. **Channel Multiplexing**: Logical channels for different services (shell, files, LSP)
3. **Protocol Buffers**: Binary encoding for efficiency
4. **Connection Management**: Automatic reconnection with session resumption

## Alternatives Considered

### JSON over WebSocket

**Pros:**
- Simple to implement and debug
- Human-readable in browser DevTools
- No code generation needed

**Cons:**
- 2-3x larger message sizes
- Slower parsing (especially for large files)
- No type safety

**Why Rejected:** Inefficient for high-frequency terminal I/O and large file transfers.

### gRPC-Web

**Pros:**
- Industry standard
- Strong typing
- Bidirectional streaming

**Cons:**
- Requires envoy proxy for browser support
- Streaming more complex than WebSocket
- Less flexible for custom protocols

**Why Rejected:** Additional proxy layer adds latency and complexity.

### Socket.IO

**Pros:**
- Fallback transports (long-polling)
- Built-in rooms and namespaces
- Large ecosystem

**Cons:**
- Higher overhead per message
- Less control over protocol
- Not optimized for binary data

**Why Rejected:** Overhead not justified; WebSocket is widely supported.

## Consequences

### Positive

- 60% bandwidth reduction vs JSON
- Sub-50ms round-trip latency
- Type-safe messages via Protobuf
- Efficient terminal streaming
- Battle-tested design (used by Replit)
- Flexible channel system

### Negative

- Requires Protocol Buffers tooling
- More complex debugging than JSON
- Client library maintenance burden
- Browser DevTools less useful
- Learning curve for team

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Debugging difficulty | Protocol-aware dev tools, logging |
| Client library maintenance | TypeScript code generation |
| Learning curve | Documentation, examples |

## Implementation

### Protocol Definition

```protobuf
syntax = "proto3";

package crosis;

message Command {
  oneof body {
    // Connection
    OpenChannel open_channel = 1;
    CloseChannel close_channel = 2;
    Ping ping = 3;
    Pong pong = 4;

    // Shell
    bytes input = 10;
    bytes output = 11;
    Resize resize = 12;

    // Files
    ReadRequest read = 20;
    WriteRequest write = 21;
    FileContent file = 22;
    WatchRequest watch = 23;
    FileEvent file_event = 24;

    // Exec
    ExecRequest exec = 30;
    int32 signal = 31;
    int32 exit_code = 32;

    // LSP
    LSPRequest lsp_request = 40;
    LSPResponse lsp_response = 41;
  }
}

message OpenChannel {
  string service = 1;
  string name = 2;
  int32 id = 3;
}

message Resize {
  int32 cols = 1;
  int32 rows = 2;
}

message ReadRequest {
  string path = 1;
}

message WriteRequest {
  string path = 1;
  bytes content = 2;
}
```

### Client Usage

```typescript
import { Client } from '@horizon/crosis';

const client = new Client();

await client.open({
  fetchConnectionMetadata: async () => ({
    token: await getAuthToken(),
    gurl: 'wss://workspace.horizonplatform.io',
  }),
  onDisconnect: ({ willReconnect }) => {
    if (!willReconnect) showReconnectUI();
  },
});

// Open shell channel
const shell = await client.openChannel({ service: 'shell' });

shell.onCommand((cmd) => {
  if (cmd.output) terminal.write(cmd.output);
});

shell.send({ input: 'ls -la\n' });

// Open files channel
const files = await client.openChannel({ service: 'files' });

files.send({ read: { path: '/main.py' } });
files.onCommand((cmd) => {
  if (cmd.file) editor.setValue(cmd.file.content);
});
```

### Server Architecture

```go
type ConnectionManager struct {
    connections map[string]*Connection
    channels    map[int]*Channel
}

type Connection struct {
    ws       *websocket.Conn
    channels map[int]*Channel
    codec    *protobuf.Codec
}

func (c *Connection) handleMessage(msg []byte) error {
    var cmd crosis.Command
    if err := proto.Unmarshal(msg, &cmd); err != nil {
        return err
    }

    switch body := cmd.Body.(type) {
    case *crosis.Command_OpenChannel:
        return c.openChannel(body.OpenChannel)
    case *crosis.Command_Input:
        return c.routeToChannel(cmd.ChannelId, body)
    // ... handle other message types
    }
}
```

### Reconnection Logic

```typescript
class ReconnectionManager {
  private retryCount = 0;
  private maxRetries = 10;

  async reconnect(): Promise<void> {
    const delay = Math.min(1000 * Math.pow(2, this.retryCount), 30000);

    await sleep(delay);
    this.retryCount++;

    try {
      await this.client.open();
      this.retryCount = 0;

      // Reopen channels
      await this.reopenChannels();

      // Resync state
      await this.resyncState();
    } catch (error) {
      if (this.retryCount < this.maxRetries) {
        return this.reconnect();
      }
      throw new Error('Max reconnection attempts reached');
    }
  }
}
```

## References

- [Replit Crosis Client](https://github.com/replit/crosis)
- [Protocol Buffers](https://protobuf.dev/)
- [WebSocket API](https://developer.mozilla.org/en-US/docs/Web/API/WebSocket)
