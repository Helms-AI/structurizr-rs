# Collaboration Subsystem

## Overview

The collaboration subsystem enables real-time multiplayer editing using CRDTs (Conflict-free Replicated Data Types). Multiple users can simultaneously edit the same file with automatic conflict resolution and sub-100ms synchronization latency.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                      Collaboration Engine (Rust)                         │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐ ┌──────────────┐   │
│  │ CRDT Engine  │ │ Cursor Sync  │ │  Presence    │ │  Operation   │   │
│  │    (Yjs)     │ │              │ │  Service     │ │  Transformer │   │
│  └──────────────┘ └──────────────┘ └──────────────┘ └──────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
         │                  │                  │                  │
         ▼                  ▼                  ▼                  ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         NATS JetStream                                   │
│  ┌─────────────────┐ ┌─────────────────┐ ┌─────────────────┐            │
│  │  Object Store   │ │     NATS KV     │ │   JetStream     │            │
│  │  (Snapshots)    │ │  (Presence)     │ │   (Broadcast)   │            │
│  └─────────────────┘ └─────────────────┘ └─────────────────┘            │
└─────────────────────────────────────────────────────────────────────────┘
                                           │                  │
                                           ▼                  ▼
                                    ┌─────────────────────────────────────┐
                                    │             WebSocket Gateway        │
                                    └─────────────────────────────────────┘
```

## CRDT Implementation

### Why CRDTs?

| Approach | Pros | Cons |
|----------|------|------|
| **OT (Operational Transform)** | Well understood | Requires central server, complex algorithms |
| **Locking** | Simple | Poor UX, blocking |
| **Last-write-wins** | Simple | Data loss |
| **CRDTs** | Conflict-free, decentralized | Memory overhead |

We chose CRDTs (specifically Yjs) for:
- Mathematical guarantee of eventual consistency
- True offline-first capability
- No central coordination required
- Proven at scale (Notion, Figma, etc.)

### Yjs Integration

```typescript
// Document setup
import * as Y from 'yjs';
import { WebsocketProvider } from 'y-websocket';
import { MonacoBinding } from 'y-monaco';

// Create Yjs document
const ydoc = new Y.Doc();
const ytext = ydoc.getText('monaco');

// Connect to sync server
const provider = new WebsocketProvider(
  'wss://collab.horizonplatform.io',
  `workspace:${workspaceId}:${filePath}`,
  ydoc
);

// Bind to Monaco editor
const binding = new MonacoBinding(
  ytext,
  editor.getModel()!,
  new Set([editor]),
  provider.awareness
);
```

### CRDT Operations

```typescript
// Text CRDT operations (simplified)
interface YTextOperation {
  type: 'insert' | 'delete';
  position: number;
  content?: string;  // For insert
  length?: number;   // For delete
  clientId: number;
  clock: number;
}

// Yjs handles conflict resolution automatically
// Example: Two users type at same position
// User A: insert("hello", 0)
// User B: insert("world", 0)
// Result: "helloworld" or "worldhello" (deterministic based on clientId)
```

## Synchronization Protocol

### CDP (Collaborative Data Protocol)

```
┌─────────┐                    ┌─────────┐                    ┌─────────┐
│ Client A│                    │  Server │                    │ Client B│
└────┬────┘                    └────┬────┘                    └────┬────┘
     │                              │                              │
     │  ──── sync_step_1 ────▶     │                              │
     │                              │                              │
     │  ◀──── sync_step_2 ────     │                              │
     │                              │                              │
     │  ──── update ──────────▶    │  ──── update ──────────▶    │
     │                              │                              │
     │                              │  ◀──── awareness ────       │
     │  ◀──── awareness ────       │                              │
     │                              │                              │
```

### Message Types

```typescript
interface SyncMessage {
  type: 'sync' | 'update' | 'awareness' | 'ping';
  workspaceId: string;
  filePath: string;
  payload: Uint8Array; // Binary encoded Yjs update
  timestamp: number;
}

// Sync step 1: Client sends state vector
interface SyncStep1 {
  type: 'sync';
  step: 1;
  stateVector: Uint8Array;
}

// Sync step 2: Server sends missing updates
interface SyncStep2 {
  type: 'sync';
  step: 2;
  diff: Uint8Array;
  stateVector: Uint8Array;
}

// Incremental update
interface UpdateMessage {
  type: 'update';
  update: Uint8Array;
}
```

### Binary Encoding

Yjs uses efficient binary encoding for updates:

```typescript
// Encoding an update
const update = Y.encodeStateAsUpdate(ydoc);

// Sending
websocket.send(update); // Uint8Array

// Receiving and applying
Y.applyUpdate(ydoc, update);

// Delta updates (much smaller)
const diff = Y.encodeStateAsUpdate(ydoc, prevStateVector);
```

## Presence System

### Awareness Protocol

```typescript
// Local state
interface LocalAwareness {
  user: {
    id: string;
    name: string;
    color: string;
    avatar?: string;
  };
  cursor?: {
    position: Position;
    selection?: Selection;
  };
  activeFile?: string;
  lastActive: number;
}

// Setting local state
provider.awareness.setLocalState({
  user: {
    id: currentUser.id,
    name: currentUser.name,
    color: generateUserColor(currentUser.id),
  },
  cursor: {
    position: editor.getPosition(),
  },
  activeFile: currentFile,
  lastActive: Date.now(),
});

// Listening for remote states
provider.awareness.on('change', ({ added, updated, removed }) => {
  const states = provider.awareness.getStates();

  for (const [clientId, state] of states) {
    if (clientId !== provider.awareness.clientID) {
      updateRemotePresence(clientId, state);
    }
  }
});
```

### Cursor Rendering

```typescript
// Remote cursor decoration
function renderRemoteCursor(user: User, cursor: Cursor): monaco.editor.IModelDeltaDecoration {
  return {
    range: new monaco.Range(
      cursor.position.lineNumber,
      cursor.position.column,
      cursor.position.lineNumber,
      cursor.position.column
    ),
    options: {
      className: `remote-cursor-${user.id}`,
      beforeContentClassName: `remote-cursor-line-${user.id}`,
      hoverMessage: { value: user.name },
      stickiness: monaco.editor.TrackedRangeStickiness.NeverGrowsWhenTypingAtEdges,
    },
  };
}

// CSS for remote cursor
.remote-cursor-${userId} {
  border-left: 2px solid ${user.color};
}

.remote-cursor-${userId}::before {
  content: '${user.name}';
  position: absolute;
  top: -1.5em;
  left: 0;
  background: ${user.color};
  color: white;
  padding: 2px 6px;
  border-radius: 3px;
  font-size: 12px;
}
```

## Access Control

### Permission Model

| Role | Read | Write | Share | Delete |
|------|------|-------|-------|--------|
| Owner | ✓ | ✓ | ✓ | ✓ |
| Editor | ✓ | ✓ | ✗ | ✗ |
| Viewer | ✓ | ✗ | ✗ | ✗ |

### Sharing Mechanisms

```typescript
// Create share link
interface ShareLink {
  id: string;
  workspaceId: string;
  role: 'editor' | 'viewer';
  expiresAt?: Date;
  maxUses?: number;
  createdBy: string;
}

// Share via email
interface EmailInvite {
  workspaceId: string;
  email: string;
  role: 'editor' | 'viewer';
  message?: string;
}

// Organization workspace
interface OrgWorkspace {
  workspaceId: string;
  orgId: string;
  defaultRole: 'editor' | 'viewer';
  members: OrgMember[];
}
```

### Permission Enforcement

```typescript
// Server-side permission check
async function checkPermission(
  userId: string,
  workspaceId: string,
  action: 'read' | 'write' | 'share' | 'delete'
): Promise<boolean> {
  // Check direct permission
  const permission = await db.workspacePermissions.findOne({
    workspaceId,
    userId,
  });

  if (permission) {
    return hasPermission(permission.role, action);
  }

  // Check organization membership
  const workspace = await db.workspaces.findOne({ id: workspaceId });
  if (workspace.orgId) {
    const orgMember = await db.orgMembers.findOne({
      orgId: workspace.orgId,
      userId,
    });
    if (orgMember) {
      return hasPermission(workspace.defaultRole, action);
    }
  }

  // Check public access
  if (workspace.isPublic && action === 'read') {
    return true;
  }

  return false;
}
```

## Session Management

### Connection Handling

```typescript
// Reconnection with exponential backoff
class CollaborationClient {
  private retryCount = 0;
  private maxRetries = 10;

  async connect(): Promise<void> {
    try {
      await this.websocket.connect();
      this.retryCount = 0;

      // Sync state after reconnection
      await this.syncState();
    } catch (error) {
      if (this.retryCount < this.maxRetries) {
        const delay = Math.min(1000 * Math.pow(2, this.retryCount), 30000);
        this.retryCount++;

        setTimeout(() => this.connect(), delay);
      } else {
        this.emit('disconnected', { permanent: true });
      }
    }
  }

  private async syncState(): Promise<void> {
    // Get local state vector
    const stateVector = Y.encodeStateVector(this.ydoc);

    // Request missing updates from server
    const diff = await this.requestSync(stateVector);

    // Apply updates
    Y.applyUpdate(this.ydoc, diff);

    // Send any local updates server might have missed
    const localUpdates = Y.encodeStateAsUpdate(
      this.ydoc,
      this.lastKnownServerState
    );
    this.sendUpdate(localUpdates);
  }
}
```

### State Persistence

```typescript
// Periodic snapshots to object storage
async function persistState(workspaceId: string, ydoc: Y.Doc): Promise<void> {
  const snapshot = Y.encodeStateAsUpdate(ydoc);
  const stateVector = Y.encodeStateVector(ydoc);

  await objectStorage.put(
    `workspaces/${workspaceId}/collab/snapshot.yjs`,
    snapshot
  );

  await objectStorage.put(
    `workspaces/${workspaceId}/collab/state-vector.bin`,
    stateVector
  );
}

// Restore from snapshot
async function restoreState(workspaceId: string): Promise<Y.Doc> {
  const ydoc = new Y.Doc();

  const snapshot = await objectStorage.get(
    `workspaces/${workspaceId}/collab/snapshot.yjs`
  );

  if (snapshot) {
    Y.applyUpdate(ydoc, snapshot);
  }

  return ydoc;
}
```

## Audit and History

### Change History

```typescript
// Track changes for undo/redo and history
interface ChangeRecord {
  id: string;
  workspaceId: string;
  filePath: string;
  userId: string;
  timestamp: Date;
  update: Uint8Array;
  undoable: boolean;
}

// Store changes in append-only log
async function recordChange(change: ChangeRecord): Promise<void> {
  await db.changeLog.insert(change);

  // Publish to NATS JetStream for real-time streaming
  await nats.jetstream.publish('collab.changes', {
    workspaceId: change.workspaceId,
    value: change,
  });
}
```

### Diff Visualization

```typescript
// Generate diff between versions
function generateDiff(oldContent: string, newContent: string): DiffResult {
  const diff = Diff.diffLines(oldContent, newContent);

  return diff.map(part => ({
    type: part.added ? 'added' : part.removed ? 'removed' : 'unchanged',
    content: part.value,
    lines: part.count,
  }));
}

// Restore to previous version
async function restoreVersion(
  workspaceId: string,
  filePath: string,
  versionId: string
): Promise<void> {
  const version = await db.fileVersions.findOne({ id: versionId });
  const currentContent = await getFileContent(workspaceId, filePath);

  // Create backup of current version
  await createVersion(workspaceId, filePath, currentContent, 'auto-backup');

  // Apply restoration as a CRDT update
  const ydoc = await getDocument(workspaceId, filePath);
  const ytext = ydoc.getText('content');

  ydoc.transact(() => {
    ytext.delete(0, ytext.length);
    ytext.insert(0, version.content);
  });
}
```

## Performance Considerations

### Scalability

| Metric | Target | Strategy |
|--------|--------|----------|
| Users per room | 50 | Horizontal scaling |
| Latency | <100ms | Regional deployment |
| Memory per doc | <10MB | Garbage collection |
| Update throughput | 1000/s | Batching |

### Optimizations

1. **Update Batching**: Coalesce rapid updates
2. **Garbage Collection**: Clean up tombstones periodically
3. **Lazy Loading**: Load document sections on demand
4. **Compression**: Compress updates over WebSocket
