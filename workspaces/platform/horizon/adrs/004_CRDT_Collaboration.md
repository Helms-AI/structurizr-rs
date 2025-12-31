# ADR-004: CRDT-Based Real-Time Collaboration

## Status

Accepted

> **Note**: The persistence layer for CRDT documents has been updated by [ADR-018](018_NATS_Messaging_Platform.md). Presence data is now stored in NATS KV instead of Redis, and document snapshots use NATS Object Store.

## Context

The platform must support real-time collaborative editing where multiple users can simultaneously edit the same file. Requirements include:

- Sub-100ms synchronization latency
- Eventual consistency under network partitions
- No data loss during concurrent edits
- Offline editing capability
- Support for 10+ concurrent editors per file
- Undo/redo with collaboration awareness

**Options Considered:**

1. **Operational Transformation (OT)**
   - Google Docs approach
   - Central server transforms operations
   - Well-understood algorithm

2. **Conflict-Free Replicated Data Types (CRDT)**
   - Distributed by design
   - No central coordination needed
   - Mathematically guaranteed convergence

3. **Last-Write-Wins (LWW)**
   - Simple implementation
   - Conflict resolution by timestamp

## Decision

We will use **Yjs CRDT library** for real-time collaboration with a custom sync layer over WebSocket.

**Key Design:**

1. **Yjs Documents**: Each file represented as a Y.Text CRDT
2. **WebSocket Sync**: Bidirectional updates via Crosis protocol
3. **Awareness Protocol**: Cursor positions and user presence
4. **Persistence**: Document snapshots stored in Redis and S3

## Alternatives Considered

### Operational Transformation (OT)

**Pros:**
- Battle-tested at Google scale
- Smaller message sizes
- Well-documented algorithms

**Cons:**
- Requires central server coordination
- Complex transformation logic
- Difficult to handle offline scenarios
- Server becomes bottleneck

**Why Rejected:** Centralized architecture limits scalability and offline support.

### Last-Write-Wins

**Pros:**
- Trivial to implement
- No conflict resolution needed
- Minimal overhead

**Cons:**
- Data loss on conflicts
- Poor user experience
- No awareness of concurrent changes

**Why Rejected:** Data loss is unacceptable for a code editor.

### Automerge CRDT

**Pros:**
- Rust implementation available
- Good JSON support
- Active development

**Cons:**
- Larger messages than Yjs
- Less mature text editing support
- Smaller ecosystem

**Why Rejected:** Yjs has superior text editing performance and ecosystem.

## Consequences

### Positive

- **True offline support**: Users can edit disconnected and sync later
- **No central coordination**: Scales horizontally without bottleneck
- **Guaranteed convergence**: Mathematical proof of consistency
- **Rich ecosystem**: Monaco, CodeMirror, ProseMirror bindings
- **Sub-50ms sync**: Efficient binary encoding

### Negative

- **Memory overhead**: CRDT metadata ~2x document size
- **Complex debugging**: Non-linear history harder to trace
- **Garbage collection**: Requires periodic compaction
- **Learning curve**: CRDT concepts not widely understood

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Memory overhead | Periodic garbage collection, snapshots |
| Debugging | Custom dev tools, operation logging |
| Garbage collection | Background compaction during idle |
| Learning curve | Team training, comprehensive docs |

## Implementation

### Y.Text Document Structure

```typescript
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';

interface CollaborativeDocument {
  ydoc: Y.Doc;
  text: Y.Text;
  awareness: Awareness;
  provider: WebsocketProvider;
}

function createCollaborativeDocument(
  fileId: string,
  userId: string
): CollaborativeDocument {
  const ydoc = new Y.Doc();
  const text = ydoc.getText('content');

  const provider = new WebsocketProvider(
    'wss://collab.horizonplatform.io',
    fileId,
    ydoc,
    {
      params: { userId },
      resyncInterval: 5000,
    }
  );

  const awareness = provider.awareness;
  awareness.setLocalState({
    user: { id: userId, color: getRandomColor(), name: userName },
    cursor: null,
  });

  return { ydoc, text, awareness, provider };
}
```

### Monaco Editor Binding

```typescript
import { MonacoBinding } from 'y-monaco';

function bindToMonaco(
  doc: CollaborativeDocument,
  editor: monaco.editor.IStandaloneCodeEditor
): MonacoBinding {
  const model = editor.getModel();

  const binding = new MonacoBinding(
    doc.text,
    model,
    new Set([editor]),
    doc.awareness
  );

  // Custom cursor decorations
  doc.awareness.on('change', () => {
    const states = doc.awareness.getStates();
    updateCursorDecorations(editor, states);
  });

  return binding;
}

function updateCursorDecorations(
  editor: monaco.editor.IStandaloneCodeEditor,
  states: Map<number, any>
): void {
  const decorations: monaco.editor.IModelDeltaDecoration[] = [];

  states.forEach((state, clientId) => {
    if (state.cursor && clientId !== doc.awareness.clientID) {
      decorations.push({
        range: new monaco.Range(
          state.cursor.line,
          state.cursor.column,
          state.cursor.line,
          state.cursor.column + 1
        ),
        options: {
          className: `cursor-${state.user.color}`,
          hoverMessage: { value: state.user.name },
        },
      });
    }
  });

  editor.deltaDecorations([], decorations);
}
```

### Server-Side Sync Service

```go
type SyncService struct {
    docs      map[string]*YjsDocument
    redis     *redis.Client
    s3        *s3.Client
    broadcast chan BroadcastMessage
}

type YjsDocument struct {
    ID        string
    State     []byte
    Clients   map[string]*Client
    UpdatedAt time.Time
    mu        sync.RWMutex
}

func (s *SyncService) HandleUpdate(docID string, update []byte) error {
    s.docs[docID].mu.Lock()
    defer s.docs[docID].mu.Unlock()

    // Apply update to document state
    newState, err := yjs.ApplyUpdate(s.docs[docID].State, update)
    if err != nil {
        return err
    }

    s.docs[docID].State = newState
    s.docs[docID].UpdatedAt = time.Now()

    // Broadcast to other clients
    for clientID, client := range s.docs[docID].Clients {
        if client.ID != update.Origin {
            client.Send(update)
        }
    }

    // Persist to Redis (hot storage)
    s.redis.Set(ctx, "doc:"+docID, newState, 24*time.Hour)

    // Async persist to S3 (cold storage)
    go s.persistToS3(docID, newState)

    return nil
}
```

### Awareness Protocol

```typescript
interface AwarenessState {
  user: {
    id: string;
    name: string;
    color: string;
    avatar?: string;
  };
  cursor?: {
    anchor: { line: number; column: number };
    head: { line: number; column: number };
  };
  selection?: {
    start: { line: number; column: number };
    end: { line: number; column: number };
  };
  focus?: boolean;
}

class AwarenessManager {
  private awareness: Awareness;
  private throttledUpdate: () => void;

  constructor(awareness: Awareness) {
    this.awareness = awareness;
    this.throttledUpdate = throttle(this.updateState.bind(this), 50);
  }

  setCursor(position: Position): void {
    const state = this.awareness.getLocalState();
    state.cursor = {
      anchor: position,
      head: position,
    };
    this.throttledUpdate();
  }

  setSelection(selection: Selection): void {
    const state = this.awareness.getLocalState();
    state.selection = selection;
    this.throttledUpdate();
  }

  private updateState(): void {
    this.awareness.setLocalState(this.awareness.getLocalState());
  }
}
```

### Conflict Resolution Strategy

```typescript
// Yjs handles most conflicts automatically via CRDT properties
// For semantic conflicts (e.g., both users rename same variable),
// we use application-level resolution

interface SemanticConflict {
  type: 'rename' | 'delete' | 'format';
  location: Range;
  users: string[];
  options: ConflictOption[];
}

function detectSemanticConflicts(
  operations: Operation[]
): SemanticConflict[] {
  const conflicts: SemanticConflict[] = [];

  // Detect overlapping rename operations
  const renames = operations.filter(op => op.type === 'rename');
  for (let i = 0; i < renames.length; i++) {
    for (let j = i + 1; j < renames.length; j++) {
      if (rangesOverlap(renames[i].range, renames[j].range)) {
        conflicts.push({
          type: 'rename',
          location: mergeRanges(renames[i].range, renames[j].range),
          users: [renames[i].userId, renames[j].userId],
          options: [
            { label: 'Keep both', action: 'merge' },
            { label: `Use ${renames[i].userId}'s`, action: 'pick', index: 0 },
            { label: `Use ${renames[j].userId}'s`, action: 'pick', index: 1 },
          ],
        });
      }
    }
  }

  return conflicts;
}
```

## Performance Benchmarks

| Metric | Target | Achieved |
|--------|--------|----------|
| Sync latency | <100ms | ~45ms |
| Memory per doc | <5MB | ~3MB |
| Max concurrent editors | 50 | 100+ |
| Offline resync time | <2s | ~800ms |

## References

- [Yjs Documentation](https://docs.yjs.dev/)
- [CRDT Papers](https://crdt.tech/papers.html)
- [y-monaco Binding](https://github.com/yjs/y-monaco)
- [Awareness Protocol](https://github.com/yjs/y-protocols)
