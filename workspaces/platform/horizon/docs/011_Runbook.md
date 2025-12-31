# Operations Runbook

## Overview

This runbook provides operational procedures for maintaining the Horizon Platform, including incident response, common issues, and maintenance tasks.

## On-Call Responsibilities

### Rotation Schedule

| Week | Primary | Secondary | Escalation |
|------|---------|-----------|------------|
| 1 | Platform Eng | Backend Eng | Engineering Manager |
| 2 | Backend Eng | AI Eng | Engineering Manager |
| 3 | AI Eng | Platform Eng | Engineering Manager |
| 4 | Platform Eng | Backend Eng | Engineering Manager |

### Contact Information

| Role | Channel | Response Time |
|------|---------|---------------|
| Primary On-Call | PagerDuty | 5 min |
| Secondary On-Call | PagerDuty | 15 min |
| Engineering Manager | Slack, Phone | 30 min |
| Security Team | security@horizonplatform.io | 1 hour |

### Escalation Matrix

| Severity | Response Time | Escalation Time | Example |
|----------|---------------|-----------------|---------|
| P1 Critical | 5 min | 15 min | Service down, data loss |
| P2 High | 15 min | 1 hour | Major feature broken |
| P3 Medium | 1 hour | 4 hours | Degraded performance |
| P4 Low | 4 hours | Next business day | Minor bug |

## Common Incidents

### Incident 1: Container Start Failure

**Symptoms**:
- Users report "workspace not starting"
- Container orchestrator errors in logs
- High container creation failure rate

**Diagnosis**:

```bash
# Check container orchestrator status
kubectl get pods -n core -l app=container-orchestrator

# View recent logs
kubectl logs -n core -l app=container-orchestrator --tail=100

# Check Nix store availability
kubectl exec -n core deploy/container-orchestrator -- df -h /nix/store

# Check resource quotas
kubectl describe resourcequota -n workspaces
```

**Resolution**:

1. **If Nix store is full/unavailable**:
   ```bash
   # Restart Nix store mount
   kubectl rollout restart daemonset/nix-store-mounter -n kube-system

   # Verify mount
   kubectl exec -n core deploy/container-orchestrator -- ls /nix/store | head
   ```

2. **If resource quota exhausted**:
   ```bash
   # Scale up runtime node pool (adjust for your cluster)
   kubectl scale deployment/runtime-pool --replicas=100 -n horizon-workspaces
   # Or use cluster autoscaler if configured
   ```

3. **If orchestrator is unhealthy**:
   ```bash
   # Restart orchestrator
   kubectl rollout restart deployment/container-orchestrator -n core

   # Monitor rollout
   kubectl rollout status deployment/container-orchestrator -n core
   ```

**Prevention**:
- Alert on Nix store disk usage >80%
- Alert on resource quota usage >90%
- Maintain warm pool of 50 containers

---

### Incident 2: Collaboration Sync Failure

**Symptoms**:
- Users report "changes not syncing"
- Multiple users see different file versions
- WebSocket disconnections

**Diagnosis**:

```bash
# Check collaboration engine pods
kubectl get pods -n collab -l app=collaboration-engine

# Check WebSocket connections
kubectl logs -n api -l app=websocket-gateway --tail=100 | grep -i error

# Check NATS cluster status
kubectl exec -n data nats-0 -- nats server check jetstream

# Check JetStream streams
kubectl exec -n data nats-0 -- nats stream ls

# Check consumer lag
kubectl exec -n data nats-0 -- nats consumer info COLLABORATION collab-engine

# Check KV bucket for presence
kubectl exec -n data nats-0 -- nats kv ls presence
```

**Resolution**:

1. **If NATS cluster is degraded**:
   ```bash
   # Check cluster status
   kubectl exec -n data nats-0 -- nats server check cluster

   # Force leader election if needed
   kubectl exec -n data nats-0 -- nats server raft step_down
   ```

2. **If JetStream consumer is lagging**:
   ```bash
   # Scale up consumers
   kubectl scale deployment/collaboration-engine -n collab --replicas=10

   # Check consumer info
   kubectl exec -n data nats-0 -- nats consumer info COLLABORATION collab-engine
   ```

3. **If presence data is stale**:
   ```bash
   # Check KV bucket health
   kubectl exec -n data nats-0 -- nats kv status presence

   # Purge stale keys (if needed)
   kubectl exec -n data nats-0 -- nats kv purge presence --force
   ```

4. **If document state is corrupted**:
   ```bash
   # Force resync from storage
   curl -X POST http://collaboration-engine:8080/api/resync \
     -H "Content-Type: application/json" \
     -d '{"workspaceId": "ws_xxx", "filePath": "/main.py"}'
   ```

**Prevention**:
- Alert on NATS cluster size < 3
- Alert on JetStream consumer pending > 1000
- Monitor NATS KV operation latency
- Monitor WebSocket connection count

---

### Incident 3: AI Service Degradation

**Symptoms**:
- Slow or failing AI responses
- Timeouts in AI chat
- High error rates on /ai/* endpoints

**Diagnosis**:

```bash
# Check AI Gateway status
kubectl get pods -n ai -l app=ai-gateway

# Check LLM provider status
curl https://status.openai.com/api/v2/status.json
curl https://status.anthropic.com/api/v2/status.json

# Check rate limiting
kubectl logs -n ai -l app=ai-gateway | grep -i "rate limit"

# Check latency metrics
curl http://prometheus:9090/api/v1/query \
  --data-urlencode 'query=histogram_quantile(0.99, rate(ai_request_duration_seconds_bucket[5m]))'
```

**Resolution**:

1. **If OpenAI is down**:
   ```bash
   # Switch to Anthropic fallback
   kubectl set env deployment/ai-gateway -n ai PRIMARY_PROVIDER=anthropic

   # Monitor switch
   kubectl rollout status deployment/ai-gateway -n ai
   ```

2. **If rate limited**:
   ```bash
   # Reduce request rate
   kubectl set env deployment/ai-gateway -n ai MAX_REQUESTS_PER_MINUTE=1000

   # Enable request queuing
   kubectl set env deployment/ai-gateway -n ai ENABLE_QUEUE=true
   ```

3. **If high latency**:
   ```bash
   # Reduce max tokens
   kubectl set env deployment/ai-gateway -n ai DEFAULT_MAX_TOKENS=1000

   # Enable caching
   kubectl set env deployment/ai-gateway -n ai ENABLE_CACHE=true
   ```

**Prevention**:
- Alert on AI latency P99 >10s
- Alert on OpenAI/Anthropic status changes
- Maintain fallback provider always ready

---

### Incident 4: Database Connection Exhaustion

**Symptoms**:
- "too many connections" errors
- Slow API responses
- Database queries timing out

**Diagnosis**:

```bash
# Check connection count
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c \
  "SELECT count(*) FROM pg_stat_activity;"

# Check connection sources
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c \
  "SELECT client_addr, count(*) FROM pg_stat_activity GROUP BY client_addr ORDER BY count DESC;"

# Check long-running queries
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c \
  "SELECT pid, now() - pg_stat_activity.query_start AS duration, query
   FROM pg_stat_activity WHERE state = 'active' ORDER BY duration DESC LIMIT 10;"
```

**Resolution**:

1. **Kill long-running queries**:
   ```bash
   PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c \
     "SELECT pg_terminate_backend(pid) FROM pg_stat_activity
      WHERE state = 'active' AND query_start < now() - interval '10 minutes';"
   ```

2. **Scale connection pool**:
   ```bash
   # Increase PgBouncer connections
   kubectl set env deployment/pgbouncer -n data MAX_CLIENT_CONN=2000

   # Restart PgBouncer
   kubectl rollout restart deployment/pgbouncer -n data
   ```

3. **Restart problematic service**:
   ```bash
   # Identify service with most connections
   # Restart that service to release connections
   kubectl rollout restart deployment/workspace-service -n core
   ```

**Prevention**:
- Alert on connection count >80% of max
- Set query timeout to 30s
- Use connection pooling everywhere

---

## Maintenance Procedures

### Database Maintenance

**Weekly Vacuum and Analyze**:

```bash
# Run vacuum analyze on all tables
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c "VACUUM ANALYZE;"

# Check table bloat
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -f check_bloat.sql
```

**Monthly Index Reindex**:

```bash
# Reindex concurrently
PGPASSWORD=$DB_PASSWORD psql -h $DB_HOST -U postgres -c \
  "REINDEX INDEX CONCURRENTLY idx_workspaces_user_id;"
```

### Certificate Rotation

**SSL Certificate Renewal** (automated via cert-manager):

```bash
# Check certificate status
kubectl get certificates -A

# Force renewal if needed
kubectl delete certificate horizon-tls -n horizon

# Verify new certificate
kubectl get certificate horizon-tls -n horizon -o yaml
```

### Security Patches

**Kubernetes Security Updates**:

```bash
# Check current Kubernetes version
kubectl version --short

# List available node versions (cloud-specific, example for managed K8s)
# For self-managed: check upstream Kubernetes releases

# Perform rolling node upgrade (managed Kubernetes)
# AWS EKS: eksctl upgrade nodegroup --cluster=horizon-prod --name=api
# Azure AKS: az aks nodepool upgrade --cluster-name horizon-prod --name api
# GKE: gcloud container clusters upgrade horizon-prod --node-pool api

# For self-managed clusters, use kubeadm:
# kubeadm upgrade plan
# kubeadm upgrade apply v1.28.0
```

**Container Image Updates**:

```bash
# Scan images for vulnerabilities
trivy image workspace-base:latest

# Update base image
docker build -t workspace-base:v2 .
docker push registry.horizonplatform.io/workspace-base:v2

# Rolling update
kubectl set image deployment/workspace-containers \
  workspace=registry.horizonplatform.io/workspace-base:v2 -n horizon-workspaces
```

---

## Disaster Recovery

### Backup Verification

**Daily Verification**:

```bash
# Verify PostgreSQL backups (using Velero or custom backup solution)
velero backup get | grep horizon-db

# Verify object storage versioning (MinIO)
mc version info minio/horizon-workspaces

# Test backup restoration (in staging)
velero restore create --from-backup horizon-db-daily-latest \
  --namespace-mappings horizon-data:horizon-data-restore

# Or restore PostgreSQL directly
pg_restore -h $STAGING_DB_HOST -U postgres -d horizon_restore \
  /backups/horizon-$(date +%Y%m%d).dump
```

### Failover Procedures

**Regional Failover**:

1. **Verify secondary region health**:
   ```bash
   kubectl --context secondary-region get nodes
   kubectl --context secondary-region get pods -A | grep -v Running
   ```

2. **Update DNS to secondary** (using ExternalDNS or manual):
   ```bash
   # If using Cloudflare
   curl -X PATCH "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$RECORD_ID" \
     -H "Authorization: Bearer $CF_TOKEN" \
     -H "Content-Type: application/json" \
     --data '{"content":"<secondary-lb-ip>"}'

   # Or update via kubectl if using ExternalDNS
   kubectl annotate ingress horizon-ingress \
     external-dns.alpha.kubernetes.io/target=<secondary-lb-ip> --overwrite
   ```

3. **Promote PostgreSQL read replica**:
   ```bash
   # Using Patroni (if deployed)
   patronictl -c /etc/patroni.yml switchover

   # Or manually promote standby
   kubectl exec -n horizon-data postgresql-replica-0 -- \
     pg_ctl promote -D /var/lib/postgresql/data
   ```

4. **Update status page**:
   ```bash
   statuspage incident update --status investigating \
     --message "Failing over to secondary region"
   ```

---

## Monitoring Dashboards

| Dashboard | URL | Purpose |
|-----------|-----|---------|
| Platform Overview | /d/platform | High-level metrics |
| Container Health | /d/containers | Container lifecycle |
| AI Performance | /d/ai | AI service metrics |
| Collaboration | /d/collab | Sync latency, connections |
| Database | /d/database | PostgreSQL metrics |
| Costs | /d/costs | Cloud spending |

---

## Runbook Updates

This runbook should be updated:
- After every P1/P2 incident
- When new services are deployed
- Quarterly review by on-call team

**Last Updated**: 2025-01-15
**Version**: 2.0
**Owner**: Platform Team
