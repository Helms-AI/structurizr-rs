# ADR-019: Claude Code Agent SDK Consolidation

## Status

**Accepted**

Supersedes: ADR-017 (Claude SDK Migration)

## Date

2024-12-31

## Context

ADR-017 established the migration from LangChain to Claude Code Agent SDK as the primary AI framework. However, the current architecture still maintains:

1. **OpenAI as fallback provider** - GPT-4 models for when Claude is unavailable
2. **Ollama for local development** - Local LLMs to avoid API costs during development
3. **Provider Factory pattern** - Abstract factory to switch between providers at runtime

### Current Multi-Provider Architecture

```
┌─────────────────────────────────────────────────┐
│              AI Gateway (FastAPI)               │
├─────────────────────────────────────────────────┤
│                Provider Factory                  │
│    ┌─────────┬─────────┬──────────┐             │
│    │ Claude  │ OpenAI  │  Ollama  │             │
│    │Provider │Provider │ Provider │             │
│    │(Primary)│(Fallback)│  (Dev)  │             │
│    └────┬────┴────┬────┴────┬─────┘             │
│         │         │         │                    │
└─────────┼─────────┼─────────┼────────────────────┘
          ▼         ▼         ▼
     Anthropic   OpenAI    Ollama
       API        API      (Local)
```

### Problems with Multi-Provider Approach

1. **Inconsistent Behavior**: Different LLMs produce different outputs for the same prompts
2. **Maintenance Overhead**: Three provider implementations to maintain and test
3. **Configuration Complexity**: Environment variables and fallback logic
4. **Cost Tracking Complexity**: Different pricing models per provider
5. **Tool Use Differences**: OpenAI and Claude have different tool use APIs
6. **Agent SDK Lock-in**: Claude Code Agent SDK is tightly coupled to Claude models

### Why Consolidation Now?

1. **Claude Code Agent SDK maturity**: The SDK is now production-ready with excellent reliability
2. **Claude model quality**: Claude models match or exceed alternatives for code tasks
3. **Simplified architecture**: Single provider eliminates inconsistencies
4. **OAuth integration**: Claude Code OAuth provides seamless authentication
5. **Cost predictability**: Single provider simplifies billing and quotas

## Decision

We will **consolidate all AI functionality through Claude Code Agent SDK exclusively**, removing:

- OpenAI fallback provider
- Ollama local development option
- Provider Factory pattern

### New Architecture

```
┌─────────────────────────────────────────────────┐
│         AI Gateway (FastAPI)                    │
├─────────────────────────────────────────────────┤
│                                                 │
│        ┌────────────────────────┐               │
│        │  Claude SDK Client     │               │
│        │  (Anthropic SDK)       │               │
│        └──────────┬─────────────┘               │
│                   │                             │
│    ┌──────────────┼──────────────┐              │
│    │              │              │              │
│  ┌─┴────────┐ ┌───┴───────┐ ┌───┴──────┐       │
│  │   Rate   │ │ Response  │ │   Cost   │       │
│  │ Limiter  │ │ Streamer  │ │ Tracker  │       │
│  └──────────┘ └───────────┘ └──────────┘       │
│                                                 │
└─────────────────────────────────────────────────┘
                    │
                    ▼
              Anthropic API
           (Claude Code OAuth)
```

### Implementation Details

#### Authentication

Replace API key authentication with Claude Code OAuth:

```python
# Before: Multiple API keys
ANTHROPIC_API_KEY=sk-ant-...
OPENAI_API_KEY=sk-...

# After: Single OAuth token
CLAUDE_CODE_OAUTH_TOKEN=...
```

#### AI Gateway Simplification

```python
from anthropic import Anthropic

class AIGateway:
    """Simplified AI Gateway using Claude Code Agent SDK exclusively."""

    def __init__(self):
        self.client = Anthropic()  # Uses CLAUDE_CODE_OAUTH_TOKEN
        self.rate_limiter = RateLimiter()
        self.cost_tracker = CostTracker()

    async def generate(self, request: GenerateRequest) -> AsyncGenerator[str, None]:
        """Generate AI response using Claude."""
        async with self.rate_limiter.acquire(request.user_id):
            async for chunk in self.client.messages.stream(
                model=request.model or "claude-sonnet-4-20250514",
                messages=request.messages,
                tools=request.tools,
                max_tokens=request.max_tokens,
            ):
                self.cost_tracker.record(chunk)
                yield chunk.content
```

#### Model Selection

| Use Case | Model | Rationale |
|----------|-------|-----------|
| Code generation | claude-sonnet-4 | Balance of speed and quality |
| Complex reasoning | claude-opus-4 | Maximum capability |
| Quick completions | claude-sonnet-4 | Low latency |
| Code review | claude-sonnet-4 | Detailed analysis |

#### Local Development

For local development without API costs:

1. **Use test fixtures**: Pre-recorded responses for common scenarios
2. **Rate limiting**: Development tier with generous limits
3. **Caching**: Cache identical requests during development

```python
# Development mode with caching
class CachedAIGateway(AIGateway):
    def __init__(self, cache_dir: str = ".ai_cache"):
        super().__init__()
        self.cache = DiskCache(cache_dir)

    async def generate(self, request: GenerateRequest):
        cache_key = self._hash_request(request)
        if cached := self.cache.get(cache_key):
            for chunk in cached:
                yield chunk
            return

        chunks = []
        async for chunk in super().generate(request):
            chunks.append(chunk)
            yield chunk

        self.cache.set(cache_key, chunks)
```

## Consequences

### Positive

1. **Simplified architecture**: Single provider, single SDK, single authentication method
2. **Consistent behavior**: All AI responses come from Claude, eliminating inconsistencies
3. **Reduced maintenance**: No fallback logic, no provider switching code
4. **Simplified configuration**: One environment variable instead of multiple API keys
5. **Better tool use**: Native Claude Code Agent SDK tools without abstraction layers
6. **Unified monitoring**: Single provider simplifies metrics and cost tracking

### Negative

1. **Single point of failure**: No fallback if Claude API is unavailable
2. **Vendor commitment**: Fully dependent on Anthropic's Claude models
3. **Local development changes**: Developers need Claude API access (no free local option)

### Mitigations

| Risk | Mitigation |
|------|------------|
| API unavailability | Response caching, graceful degradation, retry logic |
| Vendor lock-in | Clean interface boundaries for potential future changes |
| Development costs | Generous development tier, response caching |
| Rate limits | Intelligent caching, request batching, quota management |

## Migration Steps

1. **Create ADR-019** (this document)
2. **Update workspace.dsl**:
   - Remove `openaiApi` external system
   - Remove `providerFactory`, `openaiProvider`, `ollamaProvider` components
   - Update AI Gateway properties and relationships
3. **Update docker-compose.yml**:
   - Remove Ollama service
   - Remove `ollama_data` volume
4. **Update docker-compose.override.yml**:
   - Replace `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` with `CLAUDE_CODE_OAUTH_TOKEN`
5. **Update docker/.env.example**:
   - Update AI section for Claude Code OAuth
6. **Update documentation**:
   - README.md: Remove OpenAI from tech stack, remove Ollama from services
   - docs/002_Architecture.md: Update AI Gateway description
7. **Update ADR-017**:
   - Add note that ADR-019 supersedes multi-provider aspects

## References

- [ADR-017: Claude SDK Migration](./017_Claude_SDK_Migration.md)
- [Claude Code Agent SDK Documentation](https://docs.anthropic.com/claude-code-agent-sdk)
- [Anthropic OAuth Documentation](https://docs.anthropic.com/oauth)
