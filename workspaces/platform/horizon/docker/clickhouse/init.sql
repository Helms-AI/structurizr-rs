-- ClickHouse Initialization for Horizon Platform Analytics

-- Create the analytics database
CREATE DATABASE IF NOT EXISTS horizon;

-- Usage events table (append-only, time-series)
CREATE TABLE IF NOT EXISTS horizon.usage_events
(
    event_id UUID DEFAULT generateUUIDv4(),
    timestamp DateTime64(3) DEFAULT now64(3),
    user_id UUID,
    workspace_id UUID,
    event_type LowCardinality(String),
    event_data String,
    session_id UUID,
    ip_address IPv4,
    user_agent String,
    country LowCardinality(String),
    city String
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (event_type, timestamp, user_id)
TTL timestamp + INTERVAL 90 DAY;

-- AI usage metrics
CREATE TABLE IF NOT EXISTS horizon.ai_metrics
(
    timestamp DateTime64(3) DEFAULT now64(3),
    user_id UUID,
    workspace_id UUID,
    model LowCardinality(String),
    request_type LowCardinality(String),
    prompt_tokens UInt32,
    completion_tokens UInt32,
    total_tokens UInt32,
    latency_ms UInt32,
    success UInt8,
    error_type LowCardinality(String)
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (model, timestamp)
TTL timestamp + INTERVAL 365 DAY;

-- Container metrics
CREATE TABLE IF NOT EXISTS horizon.container_metrics
(
    timestamp DateTime64(3) DEFAULT now64(3),
    workspace_id UUID,
    container_id String,
    cpu_usage Float32,
    memory_usage UInt64,
    memory_limit UInt64,
    network_rx_bytes UInt64,
    network_tx_bytes UInt64,
    disk_read_bytes UInt64,
    disk_write_bytes UInt64
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (workspace_id, timestamp)
TTL timestamp + INTERVAL 30 DAY;

-- Collaboration sessions
CREATE TABLE IF NOT EXISTS horizon.collaboration_sessions
(
    session_id UUID DEFAULT generateUUIDv4(),
    workspace_id UUID,
    file_path String,
    started_at DateTime64(3),
    ended_at DateTime64(3),
    participant_count UInt16,
    edit_count UInt32,
    conflict_count UInt16,
    avg_sync_latency_ms Float32
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(started_at)
ORDER BY (workspace_id, started_at)
TTL started_at + INTERVAL 180 DAY;

-- Deployment metrics
CREATE TABLE IF NOT EXISTS horizon.deployment_metrics
(
    timestamp DateTime64(3) DEFAULT now64(3),
    deployment_id UUID,
    workspace_id UUID,
    request_count UInt64,
    error_count UInt64,
    avg_response_time_ms Float32,
    p99_response_time_ms Float32,
    bytes_transferred UInt64,
    unique_visitors UInt32
)
ENGINE = MergeTree()
PARTITION BY toYYYYMM(timestamp)
ORDER BY (deployment_id, timestamp)
TTL timestamp + INTERVAL 90 DAY;

-- Materialized view for daily active users
CREATE MATERIALIZED VIEW IF NOT EXISTS horizon.daily_active_users
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY date
AS SELECT
    toDate(timestamp) AS date,
    uniqExact(user_id) AS unique_users,
    count() AS total_events
FROM horizon.usage_events
GROUP BY date;

-- Materialized view for AI usage by model
CREATE MATERIALIZED VIEW IF NOT EXISTS horizon.ai_usage_by_model
ENGINE = SummingMergeTree()
PARTITION BY toYYYYMM(date)
ORDER BY (date, model)
AS SELECT
    toDate(timestamp) AS date,
    model,
    count() AS request_count,
    sum(total_tokens) AS total_tokens,
    avg(latency_ms) AS avg_latency_ms
FROM horizon.ai_metrics
GROUP BY date, model;
