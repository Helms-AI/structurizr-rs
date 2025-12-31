# AI Agent Subsystem

## Overview

The AI Agent subsystem provides intelligent code assistance through a multi-agent architecture built on the **Claude Code Agent SDK**. It uses specialized agents (Manager, Editor, Debugger, Reviewer) coordinated through ReAct (Reasoning + Acting) loops to handle complex coding tasks.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                AI Agent Orchestrator (Python/Claude Code Agent SDK)     │
│                                                                          │
│  ┌───────────────────────────────────────────────────────────────────┐  │
│  │                        Manager Agent                               │  │
│  │  Task decomposition | Agent routing | Progress tracking            │  │
│  └───────────────────────────────────────────────────────────────────┘  │
│         │                    │                    │                      │
│         ▼                    ▼                    ▼                      │
│  ┌─────────────┐      ┌─────────────┐      ┌─────────────┐             │
│  │   Editor    │      │  Debugger   │      │  Reviewer   │             │
│  │   Agent     │      │   Agent     │      │   Agent     │             │
│  └─────────────┘      └─────────────┘      └─────────────┘             │
│         │                    │                    │                      │
│         └────────────────────┼────────────────────┘                      │
│                              ▼                                           │
│                    ┌─────────────────┐                                  │
│                    │  Agent Memory   │                                  │
│                    │  (Redis/Qdrant) │                                  │
│                    └─────────────────┘                                  │
└─────────────────────────────────────────────────────────────────────────┘
                               │
          ┌────────────────────┼────────────────────┐
          ▼                    ▼                    ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│  Claude Code    │  │  File System    │  │   Execution     │
│  Agent SDK      │  │   Service       │  │   Engine        │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

## Claude Code Agent SDK Integration

The Horizon Platform uses the Claude Code Agent SDK for all AI-powered features. This provides:

- **Built-in tool use**: File operations, shell commands, code editing
- **Agentic loops**: Autonomous task completion with ReAct patterns
- **Context management**: Automatic context window optimization
- **Streaming responses**: Real-time output for user feedback

### SDK Configuration

```python
from claude_code_agent import Agent, Tool, AgentConfig
from claude_code_agent.tools import (
    FileReadTool,
    FileWriteTool,
    ShellTool,
    SearchTool,
)

# Configure the agent with Horizon-specific settings
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
    ],
)

agent = Agent(config)
```

## Multi-Agent System

### Agent Roles

| Agent | Responsibility | Tools |
|-------|----------------|-------|
| **Manager** | Task decomposition, routing, coordination | Task planner, agent invoker |
| **Editor** | Code generation, modification | File read/write, code gen |
| **Debugger** | Error diagnosis, fixes | Stack trace analysis, test runner |
| **Reviewer** | Code review, suggestions | Static analysis, style checks |

### Agent Definitions

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
        tools=[
            TaskDecomposer(),
            AgentInvoker(),
            ProgressTracker(),
        ],
        max_turns=5,
    )
)

# Editor Agent - writes and modifies code
editor_agent = Agent(
    AgentConfig(
        name="Editor",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a senior software engineer with expertise in
        multiple languages and frameworks. You write clean, tested code.

        Guidelines:
        - Follow existing code style
        - Add appropriate comments
        - Consider edge cases
        - Write testable code
        """,
        tools=[
            FileReadTool(),
            FileWriteTool(),
            SearchTool(),
            ShellTool(),
        ],
        max_turns=10,
    )
)

# Debugger Agent - diagnoses and fixes errors
debugger_agent = Agent(
    AgentConfig(
        name="Debugger",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a debugging expert who can trace through
        complex error scenarios and identify root causes.

        Approach:
        1. Analyze error messages carefully
        2. Trace execution flow
        3. Identify root cause (not symptoms)
        4. Propose minimal, targeted fixes
        """,
        tools=[
            FileReadTool(),
            ShellTool(),
            SearchTool(),
            TestRunnerTool(),
        ],
        max_turns=8,
    )
)

# Reviewer Agent - ensures code quality
reviewer_agent = Agent(
    AgentConfig(
        name="Reviewer",
        model="claude-sonnet-4-20250514",
        system_prompt="""You are a code review expert focused on quality,
        security, and maintainability.

        Review for:
        - Code correctness
        - Security vulnerabilities
        - Performance issues
        - Style consistency
        """,
        tools=[
            FileReadTool(),
            SearchTool(),
            StaticAnalyzerTool(),
            SecurityScannerTool(),
        ],
        max_turns=5,
    )
)
```

## ReAct Loop Pattern

The Claude Code Agent SDK implements ReAct loops natively. Each agent follows:

```
┌─────────────────────────────────────────────────┐
│                    ReAct Loop                    │
│                                                  │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐  │
│  │  Reason  │───▶│   Act    │───▶│ Observe  │──┐│
│  └──────────┘    └──────────┘    └──────────┘  ││
│       ▲                                         ││
│       └─────────────────────────────────────────┘│
│                                                  │
│  Exit conditions:                                │
│  - Task completed successfully                   │
│  - Max iterations reached                        │
│  - Unrecoverable error                          │
│  - User cancellation                            │
└─────────────────────────────────────────────────┘
```

### Agent Execution

```python
from claude_code_agent import Agent, AgentResult

async def run_agent_task(task: str, context: dict) -> AgentResult:
    """Execute a task using the appropriate agent."""

    # Determine which agent to use
    agent = select_agent(task)

    # Build context for the agent
    agent_context = await build_context(task, context)

    # Run the agent with streaming
    result = await agent.run(
        task=task,
        context=agent_context,
        stream=True,
        on_chunk=lambda chunk: emit_to_client(chunk),
    )

    return result

async def build_context(task: str, context: dict) -> str:
    """Build context for agent from workspace state."""

    parts = []

    # Current file content
    if context.get("active_file"):
        content = await read_file(context["active_file"])
        parts.append(f"Current file ({context['active_file']}):\n{content}")

    # Related files via vector search
    if context.get("workspace_id"):
        related = await find_related_files(task, context["workspace_id"])
        for path, snippet in related[:5]:
            parts.append(f"Related ({path}):\n{snippet}")

    # Project structure
    structure = await get_project_structure(context["workspace_id"])
    parts.append(f"Project structure:\n{structure}")

    return "\n\n".join(parts)
```

## Tool System

### Built-in Claude Code Agent SDK Tools

```python
from claude_code_agent.tools import (
    FileReadTool,
    FileWriteTool,
    ShellTool,
    SearchTool,
    EditTool,
)

# File operations - read files from workspace
file_read = FileReadTool(
    allowed_paths=["/workspace"],
    max_file_size=100_000,  # 100KB limit
)

# File operations - write files to workspace
file_write = FileWriteTool(
    allowed_paths=["/workspace"],
    backup_enabled=True,
)

# Shell execution - run commands
shell = ShellTool(
    timeout=30,
    allowed_commands=["python", "node", "npm", "pip", "pytest"],
    blocked_commands=["rm -rf /", "mkfs", "dd"],
)

# Code search - find patterns in codebase
search = SearchTool(
    max_results=20,
    include_context=True,
)

# Code editing - make targeted edits
edit = EditTool(
    backup_enabled=True,
    validate_syntax=True,
)
```

### Custom Horizon Tools

```python
from claude_code_agent import Tool, ToolResult

class TestRunnerTool(Tool):
    """Run tests and return results."""

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
    """Run static analysis on code."""

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

class SecurityScannerTool(Tool):
    """Scan code for security vulnerabilities."""

    name = "security_scan"
    description = "Scan for security vulnerabilities"

    async def run(self, path: str) -> ToolResult:
        """Scan code for security issues."""
        vulns = await self.security.scan(path)

        return ToolResult(
            success=True,
            output=format_vulnerabilities(vulns),
            metadata={"vulnerability_count": len(vulns)},
        )
```

## Context Management

### Context Building with Qdrant

```python
from qdrant_client import QdrantClient

class ContextBuilder:
    def __init__(self, workspace_id: str):
        self.workspace_id = workspace_id
        self.qdrant = QdrantClient(host="qdrant", port=6333)
        self.max_tokens = 8000

    async def build_context(self, task: str, active_file: str) -> list[str]:
        context_parts = []

        # 1. Active file content
        active_content = await self.get_file(active_file)
        context_parts.append(f"Current file ({active_file}):\n{active_content}")

        # 2. Related files via Qdrant vector search
        related = await self.find_related_files(task, active_file)
        for file_path, relevance in related[:5]:
            content = await self.get_file(file_path)
            context_parts.append(f"Related file ({file_path}):\n{content}")

        # 3. Project structure
        structure = await self.get_project_structure()
        context_parts.append(f"Project structure:\n{structure}")

        # 4. Dependencies
        deps = await self.get_dependencies()
        context_parts.append(f"Dependencies:\n{deps}")

        # 5. Recent errors (if debugging)
        if "error" in task.lower() or "bug" in task.lower():
            errors = await self.get_recent_errors()
            context_parts.append(f"Recent errors:\n{errors}")

        return self.fit_to_token_budget(context_parts)

    async def find_related_files(self, task: str, active_file: str) -> list:
        """Find related files using Qdrant similarity search."""
        from sentence_transformers import SentenceTransformer

        model = SentenceTransformer('all-MiniLM-L6-v2')
        task_embedding = model.encode(task).tolist()

        results = self.qdrant.search(
            collection_name="code_embeddings",
            query_vector=task_embedding,
            query_filter={
                "must": [
                    {"key": "workspace_id", "match": {"value": self.workspace_id}}
                ]
            },
            limit=10,
        )

        return [(r.payload["file_path"], r.score) for r in results]
```

## Streaming Responses

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

## Safety and Guardrails

### Content Filtering

```python
class SafetyFilter:
    def __init__(self):
        self.blocked_patterns = [
            r'api[_-]?key\s*=',
            r'password\s*=',
            r'secret\s*=',
            r'token\s*=',
        ]

    async def filter_output(self, content: str) -> str:
        # Check for sensitive data
        for pattern in self.blocked_patterns:
            if re.search(pattern, content, re.IGNORECASE):
                content = re.sub(pattern + r'.*', '[REDACTED]', content)

        # Check for malicious code patterns
        if await self.detect_malicious(content):
            raise SafetyError("Potentially malicious code detected")

        return content

    async def detect_malicious(self, content: str) -> bool:
        dangerous_patterns = [
            r'rm\s+-rf\s+/',
            r':(){ :|:& };:',  # Fork bomb
            r'dd\s+if=.*/dev/zero',
            r'curl.*\|\s*bash',
        ]

        for pattern in dangerous_patterns:
            if re.search(pattern, content):
                return True

        return False
```

### Action Validation

```python
class ActionValidator:
    def __init__(self):
        self.allowed_paths = ['/workspace/']
        self.blocked_commands = ['rm -rf /', 'mkfs', 'dd']

    def validate_file_write(self, path: str, content: str) -> bool:
        # Check path is within workspace
        if not any(path.startswith(p) for p in self.allowed_paths):
            return False

        # Check for path traversal
        if '..' in path:
            return False

        return True

    def validate_command(self, command: str) -> bool:
        # Check against blocklist
        for blocked in self.blocked_commands:
            if blocked in command:
                return False

        return True
```

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Response time (simple) | <3s | 2.5s |
| Response time (complex) | <30s | 25s |
| Success rate | >80% | 82% |
| User satisfaction | >4.0/5 | 4.2/5 |
| Cost per request | <$0.05 | $0.04 |

## References

- [Claude Code Agent SDK Documentation](https://docs.anthropic.com/claude-code-agent-sdk)
- [Qdrant Documentation](https://qdrant.tech/documentation/)
- [ReAct Pattern Paper](https://arxiv.org/abs/2210.03629)
