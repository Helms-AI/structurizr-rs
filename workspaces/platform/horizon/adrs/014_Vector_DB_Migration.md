# ADR-014: Vector Database Migration (Pinecone to Qdrant)

## Status

**Accepted**

## Date

2024-12-31

## Context

The Horizon Platform uses vector embeddings extensively for AI-powered features:
- Code similarity search
- Semantic code understanding
- Context retrieval for AI completions (RAG)
- Codebase navigation and discovery

The original architecture specified Pinecone as the vector database, which is a cloud-only SaaS solution.

### Requirements

1. High-performance vector similarity search
2. Support for billions of vectors
3. Multiple vector dimensions (384-1536)
4. Payload filtering and metadata storage
5. Real-time indexing and updates
6. Horizontal scalability
7. Self-hosted deployment option

### Constraints

1. Prefer open-source, self-hosted solutions
2. Must support Kubernetes deployment
3. Need high availability for production workloads
4. Cost considerations at scale

## Decision

We will migrate from Pinecone to **Qdrant** as our vector database.

### Why Qdrant?

| Criteria | Pinecone | Qdrant |
|----------|----------|--------|
| **License** | Proprietary | Apache 2.0 |
| **Hosting** | Cloud-only | Self-hosted or cloud |
| **Performance** | Excellent | Excellent (Rust-based) |
| **Scalability** | Managed | Distributed clustering |
| **Filtering** | Basic | Advanced (payload + sparse vectors) |
| **Hybrid Search** | Limited | Dense + Sparse vectors |
| **Kubernetes** | ❌ | ✅ (Helm chart available) |
| **Cost** | Per-vector pricing | Infrastructure only |
| **Quantization** | ✅ | ✅ (Scalar, Product, Binary) |

### Alternatives Considered

1. **Milvus**: Highly scalable but more complex to operate
2. **Weaviate**: GraphQL-native, good for RAG but heavier
3. **ChromaDB**: Simple but not production-ready for scale
4. **pgvector**: Good for small datasets, not optimal for billions of vectors

## Implementation

### Qdrant Configuration

```yaml
# Kubernetes Deployment via Helm
apiVersion: v1
kind: ConfigMap
metadata:
  name: qdrant-config
  namespace: horizon
data:
  config.yaml: |
    storage:
      storage_path: /qdrant/storage
      snapshots_path: /qdrant/snapshots

    service:
      http_port: 6333
      grpc_port: 6334

    cluster:
      enabled: true
      p2p:
        port: 6335
```

### Collection Schema

```python
from qdrant_client import QdrantClient
from qdrant_client.models import (
    Distance, VectorParams, PointStruct,
    Filter, FieldCondition, MatchValue
)

client = QdrantClient(host="qdrant", port=6333)

# Create collection for code embeddings
client.recreate_collection(
    collection_name="code_embeddings",
    vectors_config=VectorParams(
        size=384,  # all-MiniLM-L6-v2 dimensions
        distance=Distance.COSINE
    ),
    # Enable on-disk storage for large datasets
    on_disk_payload=True,
    # Configure HNSW index
    hnsw_config={
        "m": 16,
        "ef_construct": 100
    }
)

# Add payload index for filtering
client.create_payload_index(
    collection_name="code_embeddings",
    field_name="workspace_id",
    field_schema="keyword"
)
client.create_payload_index(
    collection_name="code_embeddings",
    field_name="language",
    field_schema="keyword"
)
```

### Similarity Search

```python
async def find_similar_code(
    query_vector: list[float],
    workspace_id: str,
    language: str = None,
    limit: int = 10
) -> list[dict]:
    """Find similar code snippets using Qdrant."""

    filter_conditions = [
        FieldCondition(
            key="workspace_id",
            match=MatchValue(value=workspace_id)
        )
    ]

    if language:
        filter_conditions.append(
            FieldCondition(
                key="language",
                match=MatchValue(value=language)
            )
        )

    results = client.search(
        collection_name="code_embeddings",
        query_vector=query_vector,
        query_filter=Filter(must=filter_conditions),
        limit=limit,
        with_payload=True
    )

    return [
        {
            "id": hit.id,
            "score": hit.score,
            "file_path": hit.payload.get("file_path"),
            "content": hit.payload.get("content"),
            "language": hit.payload.get("language")
        }
        for hit in results
    ]
```

### Embedding Pipeline

```python
from sentence_transformers import SentenceTransformer

# Use open-source embedding model
model = SentenceTransformer('all-MiniLM-L6-v2')

def generate_code_embedding(code: str) -> list[float]:
    """Generate embedding for code snippet."""
    return model.encode(code).tolist()

async def index_code_file(
    workspace_id: str,
    file_path: str,
    content: str,
    language: str
) -> str:
    """Index a code file in Qdrant."""
    embedding = generate_code_embedding(content)

    point_id = f"{workspace_id}:{file_path}"

    client.upsert(
        collection_name="code_embeddings",
        points=[
            PointStruct(
                id=point_id,
                vector=embedding,
                payload={
                    "workspace_id": workspace_id,
                    "file_path": file_path,
                    "content": content[:1000],  # Store snippet
                    "language": language,
                    "indexed_at": datetime.utcnow().isoformat()
                }
            )
        ]
    )

    return point_id
```

## Consequences

### Positive

1. **Open source**: Full control, no vendor lock-in
2. **High performance**: Rust-based, optimized for speed
3. **Self-hosted**: Data sovereignty and compliance
4. **Rich filtering**: Payload filtering with sparse vectors
5. **Cost effective**: Only pay for infrastructure
6. **Kubernetes native**: Easy deployment via Helm

### Negative

1. **Operational overhead**: Need to manage Qdrant cluster
2. **Scaling complexity**: Manual cluster management
3. **Learning curve**: Different API from Pinecone
4. **Backup/restore**: Need to implement snapshot strategy

### Mitigations

| Risk | Mitigation |
|------|------------|
| Operational complexity | Use Qdrant Helm chart with StatefulSets |
| Scaling | Configure distributed mode with sharding |
| Learning curve | API is well-documented and similar concepts |
| Backups | Use Qdrant snapshots with S3/MinIO storage |

## Migration Plan

1. **Phase 1**: Deploy Qdrant alongside Pinecone
2. **Phase 2**: Dual-write to both databases
3. **Phase 3**: Migrate historical data
4. **Phase 4**: Switch reads to Qdrant
5. **Phase 5**: Decommission Pinecone

## References

- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [Qdrant Helm Chart](https://github.com/qdrant/qdrant-helm)
- [Sentence Transformers](https://www.sbert.net/)
- [Vector Database Comparison](https://qdrant.tech/benchmarks/)
