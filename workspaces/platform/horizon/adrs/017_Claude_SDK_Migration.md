# ADR-017: AI Agent SDK Migration (LangChain to Claude Code Agent SDK)

## Status

**Superseded by ADR-019**

> **Note**: The multi-provider aspects of this ADR (OpenAI fallback, Ollama development) have been superseded by [ADR-019: Claude Code Agent SDK Consolidation](./019_Claude_Only_AI.md), which establishes Claude Code Agent SDK as the exclusive AI provider.

## Original Status

**Accepted**

## Date

2024-12-31

## Context

The Horizon Platform features a multi-agent AI system for:
- Code generation and modification
- Error diagnosis and debugging
- Code review and quality assurance
- Task decomposition and orchestration

The original architecture used **Python/LangChain** with OpenAI GPT-4 as the primary LLM provider.

### Current Architecture

```
┌─────────────────────────────────────────┐
│        AI Agent Orchestrator            │
│            (LangChain)                  │
├─────────────────────────────────────────┤
│ Manager Agent  │ Editor Agent           │
│ Debugger Agent │ Reviewer Agent         │
├─────────────────────────────────────────┤
│         AI Gateway (FastAPI)            │
│   OpenAI API ←→ Anthropic API           │
└─────────────────────────────────────────┘
```

### Problems with Current Approach

1. **LangChain Complexity**: Heavy abstraction layer adds overhead
2. **OpenAI Primary**: Vendor dependency on OpenAI
3. **Limited Tool Control**: Generic tool abstractions
4. **Cost Tracking**: Complex to implement accurately
5. **Provider Switching**: Not seamless between providers

### Requirements

1. Claude Code Agent SDK as the exclusive AI provider
2. Built-in agentic loops with ReAct pattern
3. Native tool use without abstraction layers
4. Streaming responses for real-time feedback
5. Fine-grained tool control and safety

## Decision

We will migrate to the **Claude Code Agent SDK** for all AI-powered features, replacing LangChain entirely.

### Why Claude Code Agent SDK?

| Aspect | LangChain | Claude Code Agent SDK |
|--------|-----------|----------------------|
| Complexity | High (many abstractions) | Low (single SDK) |
| Control | Limited | Full control |
| Performance | Overhead | Optimized |
| Debugging | Difficult | Straightforward |
| Tool Use | Generic | Native, structured |
| Agentic Loops | Custom implementation | Built-in ReAct |
| Streaming | Wrapper-based | Native support |

### Architecture Overview

```
┌─────────────────────────────────────────────────┐
│            AI Agent Orchestrator                │
│          (Claude Code Agent SDK)                │
├─────────────────────────────────────────────────┤
│                                                 │
│  ┌─────────┐ ┌─────────┐ ┌──────────┐          │
│  │ Manager │ │ Editor  │ │ Debugger │          │
│  │  Agent  │ │  Agent  │ │  Agent   │          │
│  └────┬────┘ └────┬────┘ └────┬─────┘          │
│       │           │           │                 │
│       └───────────┼───────────┘                 │
│                   │                             │
│           ┌───────┴────────┐                    │
│           │ ReAct Loop     │                    │
│           │ (Built-in SDK) │                    │
│           └───────┬────────┘                    │
│                   │                             │
├───────────────────┼─────────────────────────────┤
│                   │                             │
│           ┌───────┴────────┐                    │
│           │ Tool System    │                    │
│           │ (Native SDK)   │                    │
│           └───────┬────────┘                    │
│                   │                             │
│    ┌──────────────┼──────────────┐              │
│    │              │              │              │
│  ┌─┴──────┐  ┌────┴────┐  ┌─────┴─────┐        │
│  │ File   │  │  Shell  │  │  Search   │        │
│  │ Tools  │  │  Tool   │  │   Tool    │        │
│  └────────┘  └─────────┘  └───────────┘        │
│                                                 │
└─────────────────────────────────────────────────┘
```

## Implementation

### Agent Configuration

```python
from claude_code_agent import Agent, AgentConfig
from claude_code_agent.tools import (
    FileReadTool,
    FileWriteTool,
    ShellTool,
    SearchTool,
    EditTool,
)

# Configure agents with Claude Code Agent SDK
config = AgentConfig(
    model="claude-sonnet-4-20250514",
    max_turns=25,
    max_tokens=8192,
    system_prompt=HORIZON_SYSTEM_PROMPT,
    tools=[
        FileReadTool(allowed_paths=["/workspace"]),
        FileWriteTool(allowed_paths=["/workspace"]),
        ShellTool(timeout=30),
        SearchTool(),
        EditTool(),
    ],
)

agent = Agent(config)
```

### Multi-Agent Orchestration

```python
from claude_code_agent import Agent, AgentConfig

# Manager Agent - orchestrates other agents
manager_agent = Agent(
    AgentConfig(
        name="Manager",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are an expert software architect who excels at
        understanding requirements and delegating work effectively.

        Your role is to:
        1. Analyze the user's request
        2. Break down complex tasks into subtasks
        3. Route subtasks to specialized agents
        4. Track progress and consolidate results
        """,
        tools=[TaskDecomposer(), AgentInvoker(), ProgressTracker()],
        max_turns=5,
    )
)

# Editor Agent - writes and modifies code
editor_agent = Agent(
    AgentConfig(
        name="Editor",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a senior software engineer with expertise in
        multiple languages and frameworks. You write clean, tested code.""",
        tools=[FileReadTool(), FileWriteTool(), SearchTool(), ShellTool()],
        max_turns=10,
    )
)

# Debugger Agent - diagnoses and fixes errors
debugger_agent = Agent(
    AgentConfig(
        name="Debugger",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a debugging expert who can trace through
        complex error scenarios and identify root causes.""",
        tools=[FileReadTool(), ShellTool(), SearchTool(), TestRunnerTool()],
        max_turns=8,
    )
)

# Reviewer Agent - ensures code quality
reviewer_agent = Agent(
    AgentConfig(
        name="Reviewer",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a code review expert focused on quality,
        security, and maintainability.""",
        tools=[FileReadTool(), SearchTool(), StaticAnalyzerTool()],
        max_turns=5,
    )
)
```

### ReAct Loop Execution

The Claude Code Agent SDK provides built-in ReAct loops:

```python
from claude_code_agent import Agent, AgentResult

async def run_agent_task(task: str, context: dict) -> AgentResult:
    """Execute a task using the Claude Code Agent SDK."""

    # Select appropriate agent
    agent = select_agent(task)

    # Build context for the agent
    agent_context = await build_context(task, context)

    # Run the agent with streaming (ReAct loop is built-in)
    result = await agent.run(
        task=task,
        context=agent_context,
        stream=True,
        on_chunk=lambda chunk: emit_to_client(chunk),
    )

    return result
```

### Tool Implementation

```python
from claude_code_agent import Tool, ToolResult

class TestRunnerTool(Tool):
    """Custom tool for running tests."""

    name = "run_tests"
    description = "Execute test suite and return results"

    async def run(self, test_path: str = None) -> ToolResult:
        """Run tests at the given path."""
        cmd = f"pytest {test_path or '.'} --tb=short -q"
        result = await self.execution_engine.run_command(cmd)

        return ToolResult(
            success=result.exit_code == 0,
            output=result.stdout,
            error=result.stderr if result.exit_code != 0 else None,
        )

class StaticAnalyzerTool(Tool):
    """Custom tool for static analysis."""

    name = "analyze"
    description = "Run static analysis and return issues"

    async def run(self, path: str) -> ToolResult:
        """Analyze code at the given path."""
        issues = await self.analyzers.run(path)

        return ToolResult(
            success=True,
            output=format_issues(issues),
            metadata={"issue_count": len(issues)},
        )
```

### Streaming Responses

```python
from fastapi import FastAPI
from fastapi.responses import StreamingResponse
from claude_code_agent import Agent

app = FastAPI()

@app.post("/api/v1/ai/generate")
async def stream_ai_response(request: GenerateRequest):
    """Stream AI response to client."""

    async def generate():
        agent = get_agent(request.agent_type)
        context = await build_context(request.task, request.context)

        async for chunk in agent.run_stream(
            task=request.task,
            context=context,
        ):
            yield f"data: {json.dumps({'chunk': chunk.content})}\n\n"

            if chunk.tool_use:
                yield f"data: {json.dumps({'tool': chunk.tool_use})}\n\n"

        yield f"data: {json.dumps({'done': True})}\n\n"

    return StreamingResponse(
        generate(),
        media_type="text/event-stream",
    )
```

## Consequences

### Positive

1. **Simplified architecture**: Single SDK, no abstraction layers
2. **Native tool use**: Structured tool calls without wrappers
3. **Built-in ReAct**: No custom loop implementation needed
4. **Streaming support**: Real-time feedback out of the box
5. **Better debugging**: Clear execution flow
6. **Consistent behavior**: Single provider eliminates inconsistencies

### Negative

1. **Single provider**: Locked to Claude/Anthropic
2. **SDK dependency**: Dependent on SDK updates
3. **Migration effort**: Need to rewrite existing agent code

### Mitigations

| Risk | Mitigation |
|------|------------|
| Single provider | Claude Code Agent SDK is actively maintained |
| SDK dependency | Pin versions, comprehensive tests |
| Migration effort | Phased migration, parallel operation during transition |

## References

- [Claude Code Agent SDK Documentation](https://docs.anthropic.com/claude-code-agent-sdk)
- [ReAct Paper](https://arxiv.org/abs/2210.03629)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
