# ADR-006: Tiered Storage Architecture

## Status

Accepted

> **Note**: Real-time collaboration state (presence, cursors) has been migrated from Redis to NATS KV per [ADR-018](018_NATS_Messaging_Platform.md). Redis remains for application caching and sessions.

## Context

The platform must store various data types with different access patterns:

- **User files**: Frequently accessed, require low latency
- **Project metadata**: Frequently read, occasionally written
- **Session state**: High velocity, short-lived
- **Collaboration data**: Real-time, eventually persisted
- **Backups and history**: Rarely accessed, large volume
- **Analytics events**: Write-heavy, batch reads

**Requirements:**
- Sub-10ms read latency for hot data
- Cost-effective storage for cold data
- Strong consistency for critical data
- Eventual consistency acceptable for analytics

## Decision

We will use a **tiered storage architecture** with three layers:

1. **Hot Tier**: Redis for session state, caching, real-time data
2. **Warm Tier**: PostgreSQL for structured data, metadata
3. **Cold Tier**: S3-compatible object storage for files, backups

**Key Design:**

- Data automatically moves between tiers based on access patterns
- Each tier optimized for its access pattern
- Unified abstraction layer for application code

## Alternatives Considered

### Single Database (PostgreSQL for Everything)

**Pros:**
- Simple operations
- ACID guarantees everywhere
- Familiar technology

**Cons:**
- Poor performance for hot data
- Expensive for large files
- Scalability ceiling

**Why Rejected:** Cannot meet latency requirements for real-time features.

### Pure Object Storage

**Pros:**
- Highly scalable
- Cost-effective
- Simple data model

**Cons:**
- High latency
- No querying capability
- Eventually consistent

**Why Rejected:** Unacceptable latency for interactive use cases.

### NewSQL (CockroachDB, TiDB)

**Pros:**
- Distributed SQL
- Horizontal scaling
- Strong consistency

**Cons:**
- Higher latency than Redis
- More expensive
- Operational complexity

**Why Rejected:** Overkill for our scale; Redis + PostgreSQL sufficient.

## Consequences

### Positive

- **Optimized performance**: Each tier tuned for its workload
- **Cost efficiency**: Cold storage significantly cheaper
- **Scalability**: Each tier scales independently
- **Flexibility**: Easy to add new storage backends

### Negative

- **Operational complexity**: Three systems to manage
- **Data consistency**: Cross-tier consistency harder
- **Migration logic**: Tiering rules must be maintained
- **Debugging**: Data spread across systems

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Operational complexity | Helm charts (Redis, PostgreSQL, MinIO) or managed services |
| Data consistency | Clear ownership rules, saga patterns |
| Migration logic | Automated tiering service |
| Debugging | Unified logging, distributed tracing |

## Implementation

### Storage Layer Architecture

```go
package storage

type StorageLayer interface {
    Get(ctx context.Context, key string) ([]byte, error)
    Set(ctx context.Context, key string, value []byte, opts SetOptions) error
    Delete(ctx context.Context, key string) error
    List(ctx context.Context, prefix string) ([]string, error)
}

type TieredStorage struct {
    hot    *RedisStorage    // Session, cache, real-time
    warm   *PostgresStorage // Metadata, structured data
    cold   *S3Storage       // Files, backups
    tierer *TieringService  // Manages data movement
}

func NewTieredStorage(cfg Config) (*TieredStorage, error) {
    hot, err := NewRedisStorage(cfg.Redis)
    if err != nil {
        return nil, err
    }

    warm, err := NewPostgresStorage(cfg.Postgres)
    if err != nil {
        return nil, err
    }

    cold, err := NewS3Storage(cfg.S3)
    if err != nil {
        return nil, err
    }

    return &TieredStorage{
        hot:    hot,
        warm:   warm,
        cold:   cold,
        tierer: NewTieringService(hot, warm, cold),
    }, nil
}
```

### Hot Tier: Redis

```go
type RedisStorage struct {
    client *redis.ClusterClient
    prefix string
}

func (r *RedisStorage) Get(ctx context.Context, key string) ([]byte, error) {
    return r.client.Get(ctx, r.prefix+key).Bytes()
}

func (r *RedisStorage) Set(ctx context.Context, key string, value []byte, opts SetOptions) error {
    return r.client.Set(ctx, r.prefix+key, value, opts.TTL).Err()
}

// Session storage with automatic expiry
type SessionStore struct {
    redis *RedisStorage
    ttl   time.Duration
}

func (s *SessionStore) GetSession(ctx context.Context, sessionID string) (*Session, error) {
    data, err := s.redis.Get(ctx, "session:"+sessionID)
    if err == redis.Nil {
        return nil, ErrSessionNotFound
    }
    if err != nil {
        return nil, err
    }

    var session Session
    if err := json.Unmarshal(data, &session); err != nil {
        return nil, err
    }

    // Extend TTL on access
    s.redis.Set(ctx, "session:"+sessionID, data, SetOptions{TTL: s.ttl})

    return &session, nil
}

// Real-time collaboration state
type CollabCache struct {
    redis *RedisStorage
}

func (c *CollabCache) SetCursor(ctx context.Context, fileID, userID string, cursor Cursor) error {
    key := fmt.Sprintf("cursor:%s:%s", fileID, userID)
    data, _ := json.Marshal(cursor)
    return c.redis.Set(ctx, key, data, SetOptions{TTL: 30 * time.Second})
}

func (c *CollabCache) GetCursors(ctx context.Context, fileID string) ([]Cursor, error) {
    pattern := fmt.Sprintf("cursor:%s:*", fileID)
    keys, err := c.redis.client.Keys(ctx, pattern).Result()
    if err != nil {
        return nil, err
    }

    cursors := make([]Cursor, 0, len(keys))
    for _, key := range keys {
        data, err := c.redis.Get(ctx, key)
        if err != nil {
            continue
        }
        var cursor Cursor
        json.Unmarshal(data, &cursor)
        cursors = append(cursors, cursor)
    }

    return cursors, nil
}
```

### Warm Tier: PostgreSQL

```go
type PostgresStorage struct {
    db *pgxpool.Pool
}

// Workspace metadata
type WorkspaceRepository struct {
    db *PostgresStorage
}

func (r *WorkspaceRepository) Create(ctx context.Context, ws *Workspace) error {
    _, err := r.db.db.Exec(ctx, `
        INSERT INTO workspaces (id, user_id, name, language, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6)
    `, ws.ID, ws.UserID, ws.Name, ws.Language, ws.CreatedAt, ws.UpdatedAt)
    return err
}

func (r *WorkspaceRepository) GetByUser(ctx context.Context, userID string) ([]*Workspace, error) {
    rows, err := r.db.db.Query(ctx, `
        SELECT id, user_id, name, language, created_at, updated_at
        FROM workspaces
        WHERE user_id = $1
        ORDER BY updated_at DESC
    `, userID)
    if err != nil {
        return nil, err
    }
    defer rows.Close()

    var workspaces []*Workspace
    for rows.Next() {
        ws := &Workspace{}
        err := rows.Scan(&ws.ID, &ws.UserID, &ws.Name, &ws.Language, &ws.CreatedAt, &ws.UpdatedAt)
        if err != nil {
            return nil, err
        }
        workspaces = append(workspaces, ws)
    }

    return workspaces, nil
}

// File metadata (content in S3)
type FileMetadataRepository struct {
    db *PostgresStorage
}

func (r *FileMetadataRepository) Create(ctx context.Context, meta *FileMetadata) error {
    _, err := r.db.db.Exec(ctx, `
        INSERT INTO file_metadata (id, workspace_id, path, size, hash, s3_key, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
    `, meta.ID, meta.WorkspaceID, meta.Path, meta.Size, meta.Hash, meta.S3Key, meta.CreatedAt, meta.UpdatedAt)
    return err
}
```

### Cold Tier: S3

```go
type S3Storage struct {
    client *s3.Client
    bucket string
}

func (s *S3Storage) Get(ctx context.Context, key string) ([]byte, error) {
    output, err := s.client.GetObject(ctx, &s3.GetObjectInput{
        Bucket: aws.String(s.bucket),
        Key:    aws.String(key),
    })
    if err != nil {
        return nil, err
    }
    defer output.Body.Close()

    return io.ReadAll(output.Body)
}

func (s *S3Storage) Set(ctx context.Context, key string, value []byte, opts SetOptions) error {
    _, err := s.client.PutObject(ctx, &s3.PutObjectInput{
        Bucket:      aws.String(s.bucket),
        Key:         aws.String(key),
        Body:        bytes.NewReader(value),
        ContentType: aws.String(opts.ContentType),
    })
    return err
}

// Streaming upload for large files
func (s *S3Storage) StreamUpload(ctx context.Context, key string, reader io.Reader) error {
    uploader := manager.NewUploader(s.client)
    _, err := uploader.Upload(ctx, &s3.PutObjectInput{
        Bucket: aws.String(s.bucket),
        Key:    aws.String(key),
        Body:   reader,
    })
    return err
}

// Presigned URLs for direct client access
func (s *S3Storage) GetPresignedURL(ctx context.Context, key string, ttl time.Duration) (string, error) {
    presigner := s3.NewPresignClient(s.client)
    req, err := presigner.PresignGetObject(ctx, &s3.GetObjectInput{
        Bucket: aws.String(s.bucket),
        Key:    aws.String(key),
    }, s3.WithPresignExpires(ttl))
    if err != nil {
        return "", err
    }
    return req.URL, nil
}
```

### Tiering Service

```go
type TieringService struct {
    hot    *RedisStorage
    warm   *PostgresStorage
    cold   *S3Storage
    rules  []TieringRule
}

type TieringRule struct {
    DataType    string
    HotTTL      time.Duration
    WarmTTL     time.Duration
    Predicate   func(interface{}) bool
}

// Move data between tiers based on rules
func (t *TieringService) ProcessTiering(ctx context.Context) error {
    for _, rule := range t.rules {
        // Hot -> Warm: Expired hot data
        err := t.migrateHotToWarm(ctx, rule)
        if err != nil {
            log.Error("hot->warm migration failed", "rule", rule.DataType, "error", err)
        }

        // Warm -> Cold: Old warm data
        err = t.migrateWarmToCold(ctx, rule)
        if err != nil {
            log.Error("warm->cold migration failed", "rule", rule.DataType, "error", err)
        }
    }
    return nil
}

func (t *TieringService) migrateHotToWarm(ctx context.Context, rule TieringRule) error {
    keys, err := t.hot.ListExpiring(ctx, rule.DataType, rule.HotTTL)
    if err != nil {
        return err
    }

    for _, key := range keys {
        data, err := t.hot.Get(ctx, key)
        if err != nil {
            continue
        }

        if rule.Predicate == nil || rule.Predicate(data) {
            // Write to warm
            err = t.warm.Set(ctx, key, data, SetOptions{})
            if err != nil {
                continue
            }

            // Delete from hot
            t.hot.Delete(ctx, key)
        }
    }

    return nil
}
```

### Data Access Patterns

```go
// Unified file access that handles tiering transparently
type FileService struct {
    tiered   *TieredStorage
    metadata *FileMetadataRepository
    cache    *RedisStorage
}

func (f *FileService) GetFile(ctx context.Context, workspaceID, path string) (*File, error) {
    // Check hot cache first
    cacheKey := fmt.Sprintf("file:%s:%s", workspaceID, path)
    if cached, err := f.cache.Get(ctx, cacheKey); err == nil {
        var file File
        json.Unmarshal(cached, &file)
        return &file, nil
    }

    // Get metadata from warm tier
    meta, err := f.metadata.GetByPath(ctx, workspaceID, path)
    if err != nil {
        return nil, err
    }

    // Get content from cold tier
    content, err := f.tiered.cold.Get(ctx, meta.S3Key)
    if err != nil {
        return nil, err
    }

    file := &File{
        Metadata: meta,
        Content:  content,
    }

    // Cache in hot tier
    data, _ := json.Marshal(file)
    f.cache.Set(ctx, cacheKey, data, SetOptions{TTL: 5 * time.Minute})

    return file, nil
}

func (f *FileService) SaveFile(ctx context.Context, workspaceID, path string, content []byte) error {
    // Generate S3 key
    s3Key := fmt.Sprintf("workspaces/%s/files/%s", workspaceID, path)

    // Write to cold tier (durable)
    err := f.tiered.cold.Set(ctx, s3Key, content, SetOptions{
        ContentType: mime.TypeByExtension(filepath.Ext(path)),
    })
    if err != nil {
        return err
    }

    // Update metadata in warm tier
    meta := &FileMetadata{
        WorkspaceID: workspaceID,
        Path:        path,
        Size:        int64(len(content)),
        Hash:        sha256.Sum256(content),
        S3Key:       s3Key,
        UpdatedAt:   time.Now(),
    }
    err = f.metadata.Upsert(ctx, meta)
    if err != nil {
        return err
    }

    // Invalidate hot cache
    cacheKey := fmt.Sprintf("file:%s:%s", workspaceID, path)
    f.cache.Delete(ctx, cacheKey)

    return nil
}
```

## Database Schema

```sql
-- PostgreSQL schema for warm tier

CREATE TABLE workspaces (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    language VARCHAR(50),
    template_id UUID REFERENCES templates(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,

    CONSTRAINT unique_user_workspace UNIQUE (user_id, name)
);

CREATE INDEX idx_workspaces_user_id ON workspaces(user_id);
CREATE INDEX idx_workspaces_updated_at ON workspaces(updated_at DESC);

CREATE TABLE file_metadata (
    id UUID PRIMARY KEY,
    workspace_id UUID NOT NULL REFERENCES workspaces(id) ON DELETE CASCADE,
    path VARCHAR(1024) NOT NULL,
    size BIGINT NOT NULL,
    hash BYTEA NOT NULL,
    s3_key VARCHAR(1024) NOT NULL,
    mime_type VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_workspace_path UNIQUE (workspace_id, path)
);

CREATE INDEX idx_file_metadata_workspace_id ON file_metadata(workspace_id);
CREATE INDEX idx_file_metadata_path ON file_metadata(path);
```

## References

- [Redis Cluster Documentation](https://redis.io/docs/reference/cluster-spec/)
- [PostgreSQL Performance Tuning](https://wiki.postgresql.org/wiki/Performance_Optimization)
- [S3 Best Practices](https://docs.aws.amazon.com/AmazonS3/latest/userguide/optimizing-performance.html)
- [Data Tiering Patterns](https://aws.amazon.com/blogs/storage/data-tiering-best-practices/)
