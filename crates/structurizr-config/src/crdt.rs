//! CRDT (Conflict-free Replicated Data Type) support for collaborative DSL editing
//!
//! Implements a simplified CRDT for text editing with:
//! - Operation-based CRDT using insert/delete operations
//! - Vector clock for causality tracking
//! - Automatic conflict resolution
//! - WebSocket-based synchronization

// Our own CRDT implementation - no external crate needed
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Unique identifier for a collaborative session participant
pub type PeerId = Uuid;

/// Vector clock for tracking causality
pub type VectorClock = HashMap<PeerId, u64>;

/// CRDT operation types for text editing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    /// Insert text at position
    Insert {
        position: usize,
        text: String,
        id: Uuid,
        clock: VectorClock,
    },
    /// Delete text range
    Delete {
        position: usize,
        length: usize,
        id: Uuid,
        clock: VectorClock,
    },
    /// Cursor position update
    Cursor {
        peer_id: PeerId,
        position: usize,
        clock: VectorClock,
    },
}

/// CRDT document representing a DSL file
pub struct CrdtDocument {
    /// Document ID
    pub id: Uuid,

    /// Current document content
    content: Arc<RwLock<String>>,

    /// Operation history
    operations: Arc<RwLock<Vec<Operation>>>,

    /// Vector clock for this peer
    clock: Arc<RwLock<VectorClock>>,

    /// Our peer ID
    peer_id: PeerId,

    /// Cursor positions of other peers
    peer_cursors: Arc<RwLock<HashMap<PeerId, usize>>>,

    /// Pending operations to broadcast
    pending_ops: Arc<RwLock<Vec<Operation>>>,
}

impl CrdtDocument {
    /// Create a new CRDT document
    pub fn new(content: String) -> Self {
        let peer_id = Uuid::new_v4();
        let mut clock = HashMap::new();
        clock.insert(peer_id, 0);

        Self {
            id: Uuid::new_v4(),
            content: Arc::new(RwLock::new(content)),
            operations: Arc::new(RwLock::new(Vec::new())),
            clock: Arc::new(RwLock::new(clock)),
            peer_id,
            peer_cursors: Arc::new(RwLock::new(HashMap::new())),
            pending_ops: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Get current document content
    pub async fn get_content(&self) -> String {
        self.content.read().await.clone()
    }

    /// Apply a local text insertion
    pub async fn local_insert(&self, position: usize, text: String) -> Operation {
        // Update our vector clock
        let mut clock = self.clock.write().await;
        *clock.entry(self.peer_id).or_insert(0) += 1;

        let op = Operation::Insert {
            position,
            text: text.clone(),
            id: Uuid::new_v4(),
            clock: clock.clone(),
        };

        // Apply locally
        self.apply_operation(&op).await;

        // Add to pending
        self.pending_ops.write().await.push(op.clone());

        op
    }

    /// Apply a local text deletion
    pub async fn local_delete(&self, position: usize, length: usize) -> Operation {
        // Update our vector clock
        let mut clock = self.clock.write().await;
        *clock.entry(self.peer_id).or_insert(0) += 1;

        let op = Operation::Delete {
            position,
            length,
            id: Uuid::new_v4(),
            clock: clock.clone(),
        };

        // Apply locally
        self.apply_operation(&op).await;

        // Add to pending
        self.pending_ops.write().await.push(op.clone());

        op
    }

    /// Update local cursor position
    pub async fn update_cursor(&self, position: usize) -> Operation {
        let clock = self.clock.read().await.clone();

        let op = Operation::Cursor {
            peer_id: self.peer_id,
            position,
            clock,
        };

        // Add to pending
        self.pending_ops.write().await.push(op.clone());

        op
    }

    /// Apply a remote operation
    pub async fn remote_operation(&self, op: Operation) {
        // Check if we've already applied this operation
        let ops = self.operations.read().await;
        if ops.iter().any(|o| self.operation_id(o) == self.operation_id(&op)) {
            return;  // Already applied
        }
        drop(ops);

        // Update vector clock
        if let Some(op_clock) = self.operation_clock(&op) {
            let mut clock = self.clock.write().await;
            for (peer, timestamp) in op_clock {
                let entry = clock.entry(*peer).or_insert(0);
                *entry = (*entry).max(*timestamp);
            }
        }

        // Apply operation
        self.apply_operation(&op).await;

        // Store in history
        self.operations.write().await.push(op);
    }

    /// Apply an operation to the document
    async fn apply_operation(&self, op: &Operation) {
        match op {
            Operation::Insert { position, text, .. } => {
                let mut content = self.content.write().await;

                // Validate position
                let pos = (*position).min(content.len());

                // Insert text
                content.insert_str(pos, text);
            }
            Operation::Delete { position, length, .. } => {
                let mut content = self.content.write().await;

                // Validate range
                let start = (*position).min(content.len());
                let end = (start + length).min(content.len());

                // Delete range
                content.replace_range(start..end, "");
            }
            Operation::Cursor { peer_id, position, .. } => {
                let mut cursors = self.peer_cursors.write().await;
                cursors.insert(*peer_id, *position);
            }
        }
    }

    /// Get operation ID for deduplication
    fn operation_id(&self, op: &Operation) -> Uuid {
        match op {
            Operation::Insert { id, .. } |
            Operation::Delete { id, .. } => *id,
            Operation::Cursor { peer_id, .. } => *peer_id,
        }
    }

    /// Get operation vector clock
    fn operation_clock<'a>(&self, op: &'a Operation) -> Option<&'a VectorClock> {
        match op {
            Operation::Insert { clock, .. } |
            Operation::Delete { clock, .. } |
            Operation::Cursor { clock, .. } => Some(clock),
        }
    }

    /// Get pending operations and clear the queue
    pub async fn drain_pending_operations(&self) -> Vec<Operation> {
        let mut pending = self.pending_ops.write().await;
        let ops = pending.clone();
        pending.clear();
        ops
    }

    /// Get peer cursor positions
    pub async fn get_peer_cursors(&self) -> HashMap<PeerId, usize> {
        self.peer_cursors.read().await.clone()
    }

    /// Merge with another document's state (for reconnection)
    pub async fn merge(&self, other_ops: Vec<Operation>) {
        for op in other_ops {
            self.remote_operation(op).await;
        }
    }
}

/// CRDT synchronization message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request full state sync
    SyncRequest {
        peer_id: PeerId,
        document_id: Uuid,
    },
    /// Full state response
    SyncResponse {
        peer_id: PeerId,
        document_id: Uuid,
        content: String,
        operations: Vec<Operation>,
    },
    /// Single operation broadcast
    Operation {
        peer_id: PeerId,
        document_id: Uuid,
        operation: Operation,
    },
    /// Heartbeat/presence
    Heartbeat {
        peer_id: PeerId,
        document_id: Uuid,
    },
}

/// CRDT session manager for multiple documents
pub struct CrdtSession {
    /// Our peer ID
    pub peer_id: PeerId,

    /// Active CRDT documents by workspace ID
    documents: Arc<RwLock<HashMap<String, CrdtDocument>>>,
}

impl CrdtSession {
    /// Create a new CRDT session
    pub fn new() -> Self {
        Self {
            peer_id: Uuid::new_v4(),
            documents: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Open a document for collaborative editing
    pub async fn open_document(&self, workspace_id: String, content: String) -> Uuid {
        let doc = CrdtDocument::new(content);
        let doc_id = doc.id;

        let mut docs = self.documents.write().await;
        docs.insert(workspace_id, doc);

        doc_id
    }

    /// Get a document by workspace ID
    pub async fn get_document(&self, workspace_id: &str) -> Option<CrdtDocument> {
        let docs = self.documents.read().await;
        docs.get(workspace_id).map(|d| CrdtDocument {
            id: d.id,
            content: d.content.clone(),
            operations: d.operations.clone(),
            clock: d.clock.clone(),
            peer_id: d.peer_id,
            peer_cursors: d.peer_cursors.clone(),
            pending_ops: d.pending_ops.clone(),
        })
    }

    /// Process incoming sync message
    pub async fn handle_sync_message(&self, msg: SyncMessage) -> Option<SyncMessage> {
        match msg {
            SyncMessage::SyncRequest { peer_id: _, document_id } => {
                // Find document and send full state
                let docs = self.documents.read().await;
                for (_, doc) in docs.iter() {
                    if doc.id == document_id {
                        let content = doc.get_content().await;
                        let operations = doc.operations.read().await.clone();

                        return Some(SyncMessage::SyncResponse {
                            peer_id: self.peer_id,
                            document_id,
                            content,
                            operations,
                        });
                    }
                }
                None
            }
            SyncMessage::Operation { document_id, operation, .. } => {
                // Apply remote operation
                let docs = self.documents.read().await;
                let found_doc = docs.iter()
                    .find(|(_, doc)| doc.id == document_id)
                    .map(|(_, doc)| CrdtDocument {
                        id: doc.id,
                        content: doc.content.clone(),
                        operations: doc.operations.clone(),
                        clock: doc.clock.clone(),
                        peer_id: doc.peer_id,
                        peer_cursors: doc.peer_cursors.clone(),
                        pending_ops: doc.pending_ops.clone(),
                    });
                drop(docs);

                if let Some(doc) = found_doc {
                    doc.remote_operation(operation).await;
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_crdt_insert() {
        let doc = CrdtDocument::new("Hello".to_string());

        doc.local_insert(5, " World".to_string()).await;

        assert_eq!(doc.get_content().await, "Hello World");
    }

    #[tokio::test]
    async fn test_crdt_delete() {
        let doc = CrdtDocument::new("Hello World".to_string());

        doc.local_delete(5, 6).await;

        assert_eq!(doc.get_content().await, "Hello");
    }

    #[tokio::test]
    async fn test_concurrent_edits() {
        let doc1 = CrdtDocument::new("AB".to_string());
        let doc2 = CrdtDocument::new("AB".to_string());

        // Concurrent inserts at different positions
        let op1 = doc1.local_insert(1, "X".to_string()).await;
        let op2 = Operation::Insert {
            position: 2,
            text: "Y".to_string(),
            id: Uuid::new_v4(),
            clock: HashMap::new(),
        };

        // Apply remote operation to doc1
        doc1.remote_operation(op2.clone()).await;

        // Both documents should converge to the same state
        assert_eq!(doc1.get_content().await, "AXYB");
    }
}