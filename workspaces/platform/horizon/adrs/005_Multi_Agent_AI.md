# ADR-005: Multi-Agent AI Architecture

## Status

Accepted (Updated: 2025-01-15)

## Context

The Horizon Platform requires an AI assistant that can help developers with complex coding tasks. Requirements include:

- Code generation from natural language
- Debugging and error explanation
- Code refactoring and optimization
- Multi-file changes with consistency
- Safe execution of file modifications
- Native tool use and agentic workflows

**Options Considered:**

1. **Single LLM Agent**
   - One agent handles all tasks
   - Simpler architecture
   - Direct API calls

2. **Claude Code Agent SDK**
   - Built-in agentic capabilities with ReAct loops
   - Native tool use and file operations
   - Specialized agents via custom system prompts
   - Production-tested in Claude Code CLI

3. **RAG-Only System**
   - Retrieval-augmented generation
   - No autonomous actions
   - Pure Q&A interface

## Decision

We will implement a **Multi-Agent AI system** using the **Claude Code Agent SDK** with specialized agents coordinated through a manager pattern. The SDK provides built-in ReAct loops and tool execution.

**Key Design:**

1. **Agent Specialization**: Separate agents for editing, debugging, reviewing, explaining using custom system prompts
2. **Manager Coordination**: Orchestrator routes tasks to specialized agents
3. **Built-in ReAct Pattern**: Claude Code Agent SDK handles reason-act-observe loops natively
4. **Native Tool System**: Use SDK's built-in tools (FileRead, FileWrite, Shell, Search)
5. **Safety Guardrails**: SDK's permission system and custom validation

## Alternatives Considered

### Single Agent

**Pros:**
- Simpler to implement and maintain
- Lower latency (no coordination)
- Easier to debug

**Cons:**
- Jack of all trades, master of none
- Harder to optimize for specific tasks
- Context window limitations
- Single point of failure for quality

**Why Rejected:** Complex coding tasks require specialized reasoning.

### RAG-Only System

**Pros:**
- No autonomous actions (safer)
- Deterministic responses
- Lower API costs

**Cons:**
- Cannot make changes
- Limited to Q&A
- No multi-step reasoning

**Why Rejected:** Users need AI that can act, not just answer.

### Function Calling Only

**Pros:**
- Native LLM support
- Well-structured outputs
- Predictable behavior

**Cons:**
- Limited reasoning chains
- No iterative refinement
- Harder to recover from errors

**Why Rejected:** Complex tasks require iterative reasoning.

## Consequences

### Positive

- **Improved accuracy**: Specialized agents outperform generalists
- **Parallelization**: Independent agents can work concurrently
- **Extensibility**: New agents can be added without system redesign
- **Safety**: Verification agent catches dangerous operations
- **Observability**: Clear agent handoffs aid debugging

### Negative

- **Latency**: Multi-agent coordination adds overhead
- **Cost**: Multiple LLM calls increase API spend
- **Complexity**: More failure modes to handle
- **Consistency**: Agents may produce conflicting results

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Latency | Parallel execution where possible, streaming |
| Cost | Smart routing, caching, cheaper models for simple tasks |
| Complexity | Comprehensive logging, fallback behaviors |
| Consistency | Manager agent validates and reconciles |

## Implementation

### Agent Configuration with Claude Code Agent SDK

```python
from claude_code_agent import Agent, AgentConfig
from claude_code_agent.tools import (
    FileReadTool,
    FileWriteTool,
    ShellTool,
    SearchTool,
)
from typing import Dict, Any
from dataclasses import dataclass

@dataclass
class AgentResult:
    success: bool
    message: str
    changes: list = None
    error: str = None

# Base configuration for all Horizon agents
def create_base_config(system_prompt: str, workspace_path: str) -> AgentConfig:
    return AgentConfig(
        model="claude-sonnet-4-20250514",
        max_turns=25,
        max_tokens=8192,
        system_prompt=system_prompt,
        tools=[
            FileReadTool(allowed_paths=[workspace_path]),
            FileWriteTool(allowed_paths=[workspace_path]),
            ShellTool(timeout=30, allowed_commands=["python", "node", "npm", "pip"]),
            SearchTool(search_paths=[workspace_path]),
        ],
    )


class ManagerAgent:
    """Coordinates task decomposition and agent routing."""

    SYSTEM_PROMPT = """You are a Manager Agent for Horizon Platform responsible for:
    1. Understanding user requests
    2. Breaking complex tasks into subtasks
    3. Routing subtasks to specialized agents
    4. Aggregating and validating results

    Available agents:
    - EditorAgent: Code modifications, file operations
    - DebuggerAgent: Error analysis, debugging
    - ReviewerAgent: Code review, quality checks
    - ExplainerAgent: Documentation, explanations

    Always verify results before returning to user.
    """

    def __init__(self, workspace_path: str):
        self.config = create_base_config(self.SYSTEM_PROMPT, workspace_path)
        self.agent = Agent(self.config)

    async def run(self, task: str, context: Dict[str, Any]) -> AgentResult:
        response = await self.agent.run(
            f"Task: {task}\nContext: {context}"
        )
        return AgentResult(
            success=response.success,
            message=response.output,
            changes=response.file_changes,
        )


class EditorAgent:
    """Handles code modifications and file operations."""

    SYSTEM_PROMPT = """You are an Editor Agent for Horizon Platform that modifies code.

    Guidelines:
    - Make minimal, targeted changes
    - Preserve code style and formatting
    - Add comments for complex changes
    - Never delete without explicit instruction
    - Use the file tools to read/write files
    - Use search to find relevant code patterns
    """

    def __init__(self, workspace_path: str):
        self.config = create_base_config(self.SYSTEM_PROMPT, workspace_path)
        self.agent = Agent(self.config)

    async def run(self, task: str, context: Dict[str, Any]) -> AgentResult:
        response = await self.agent.run(task)
        return AgentResult(
            success=response.success,
            message=response.output,
            changes=response.file_changes,
        )


class DebuggerAgent:
    """Analyzes errors and suggests fixes."""

    SYSTEM_PROMPT = """You are a Debugger Agent for Horizon Platform that diagnoses code issues.

    Process:
    1. Analyze error message and stack trace
    2. Read relevant source files to understand context
    3. Search for similar patterns in the codebase
    4. Propose fix with explanation

    Be specific about:
    - What went wrong
    - Why it happened
    - How to fix it
    - How to prevent it
    """

    def __init__(self, workspace_path: str):
        self.config = create_base_config(self.SYSTEM_PROMPT, workspace_path)
        self.agent = Agent(self.config)

    async def run(self, error: str, context: Dict[str, Any]) -> AgentResult:
        response = await self.agent.run(
            f"Debug this error:\n{error}\n\nContext: {context}"
        )
        return AgentResult(
            success=response.success,
            message=response.output,
        )


class ReviewerAgent:
    """Reviews code changes for quality and safety."""

    SYSTEM_PROMPT = """You are a Reviewer Agent for Horizon Platform that validates code changes.

    Check for:
    - Security vulnerabilities
    - Performance issues
    - Code style violations
    - Logic errors
    - Test coverage gaps

    Rate changes as: APPROVE, REQUEST_CHANGES, or BLOCK
    """

    def __init__(self, workspace_path: str):
        self.config = create_base_config(self.SYSTEM_PROMPT, workspace_path)
        self.agent = Agent(self.config)

    async def run(self, changes: str, context: Dict[str, Any]) -> AgentResult:
        response = await self.agent.run(
            f"Review these changes:\n{changes}"
        )
        return AgentResult(
            success=response.success,
            message=response.output,
        )
```

### ReAct Loop (Built-in to Claude Code Agent SDK)

The Claude Code Agent SDK handles the ReAct (Reason-Act-Observe) pattern natively. When an agent runs, the SDK automatically:

1. **Reasons** about the task and plans next steps
2. **Acts** by calling tools (file operations, shell commands, search)
3. **Observes** the results and incorporates them into context
4. **Iterates** until the task is complete or max_turns is reached

```python
# The SDK handles ReAct internally - you just configure and run
from claude_code_agent import Agent, AgentConfig

config = AgentConfig(
    model="claude-sonnet-4-20250514",
    max_turns=25,  # Maximum ReAct iterations
    max_tokens=8192,
    system_prompt="Your specialized agent prompt here",
    tools=[...],  # Available tools for the agent
)

agent = Agent(config)

# The run method handles the full ReAct loop internally
response = await agent.run("Complete this coding task")

# Access the conversation history if needed
for turn in response.conversation:
    print(f"Action: {turn.action}")
    print(f"Observation: {turn.observation}")
```

### Tool System (Claude Code Agent SDK Built-in Tools)

The Claude Code Agent SDK provides production-tested tools out of the box:

```python
from claude_code_agent.tools import (
    FileReadTool,
    FileWriteTool,
    ShellTool,
    SearchTool,
    GlobTool,
    GrepTool,
)

# Configure tools with security boundaries
tools = [
    # File operations with path restrictions
    FileReadTool(
        allowed_paths=["/workspace"],
        max_file_size=1024 * 1024,  # 1MB limit
    ),
    FileWriteTool(
        allowed_paths=["/workspace"],
        blocked_patterns=["*.env", "*.key", "*.pem"],
    ),

    # Shell with command allowlisting
    ShellTool(
        timeout=30,
        allowed_commands=["python", "node", "npm", "pip", "cargo", "go"],
        blocked_patterns=["rm -rf", "sudo", "chmod 777"],
    ),

    # Code search tools
    SearchTool(search_paths=["/workspace"]),
    GlobTool(base_path="/workspace"),
    GrepTool(base_path="/workspace"),
]

# Custom tool example (if needed beyond built-in tools)
from claude_code_agent.tools import BaseTool

class CustomDatabaseTool(BaseTool):
    """Custom tool for database operations."""

    name = "database_query"
    description = "Execute a read-only database query"

    async def execute(self, query: str) -> str:
        # Validate query is read-only
        if not query.strip().upper().startswith("SELECT"):
            return "Error: Only SELECT queries are allowed"

        result = await self.db.execute(query)
        return result.to_json()
```

### Agent Orchestration

```python
from dataclasses import dataclass
from typing import Optional

@dataclass
class UserRequest:
    message: str
    workspace_path: str
    relevant_files: list = None
    error_context: str = None

@dataclass
class AgentResponse:
    message: str
    changes: list = None
    review: str = None
    tokens_used: int = 0

class AgentOrchestrator:
    """Orchestrates multi-agent collaboration using Claude Code Agent SDK."""

    def __init__(self, workspace_path: str):
        self.workspace_path = workspace_path
        self.manager = ManagerAgent(workspace_path)
        self.agents = {
            "editor": EditorAgent(workspace_path),
            "debugger": DebuggerAgent(workspace_path),
            "reviewer": ReviewerAgent(workspace_path),
        }

    async def handle_request(self, request: UserRequest) -> AgentResponse:
        # Build context from workspace
        context = {
            "workspace_path": request.workspace_path,
            "files": request.relevant_files or [],
            "error": request.error_context,
        }

        # Route through manager for task decomposition
        result = await self.manager.run(
            task=request.message,
            context=context
        )

        # If there are file changes, run reviewer
        if result.changes:
            review_result = await self.agents["reviewer"].run(
                changes=str(result.changes),
                context=context
            )
            return AgentResponse(
                message=result.message,
                changes=result.changes,
                review=review_result.message,
            )

        return AgentResponse(
            message=result.message,
            changes=result.changes,
        )

    async def debug_error(self, error: str, context: dict) -> AgentResponse:
        """Direct route to debugger agent for error analysis."""
        result = await self.agents["debugger"].run(error, context)
        return AgentResponse(message=result.message)

    async def edit_code(self, task: str, context: dict) -> AgentResponse:
        """Direct route to editor agent for code modifications."""
        result = await self.agents["editor"].run(task, context)
        return AgentResponse(
            message=result.message,
            changes=result.changes,
        )
```

## Safety Guardrails

The Claude Code Agent SDK includes built-in safety features. Additional custom guardrails can be added:

```python
from claude_code_agent import Agent, AgentConfig
from claude_code_agent.permissions import PermissionPolicy

# Configure permission policy for the agent
permissions = PermissionPolicy(
    # File operations
    allow_file_read=True,
    allow_file_write=True,
    blocked_write_patterns=["*.env", "*.key", "*.pem", ".git/*"],
    allowed_paths=["/workspace"],

    # Shell operations
    allow_shell=True,
    allowed_commands=["python", "node", "npm", "pip", "cargo", "go", "git"],
    blocked_shell_patterns=[
        r"rm\s+-rf\s+/",
        r"sudo\s+",
        r"chmod\s+777",
        r"curl.*\|.*sh",
        r"wget.*\|.*sh",
    ],

    # Network operations
    allow_network=False,  # Disable by default

    # Resource limits
    max_file_size=10 * 1024 * 1024,  # 10MB
    max_turns=25,
    timeout=300,  # 5 minutes
)

config = AgentConfig(
    model="claude-sonnet-4-20250514",
    permissions=permissions,
    # ... other config
)

# Custom validation hook (optional)
async def validate_action(action, context):
    """Additional validation before tool execution."""
    if action.tool == "shell":
        # Block access to sensitive directories
        if any(p in action.command for p in ["/etc", "/root", "~/.ssh"]):
            return False, "Access to system directories is blocked"
    return True, None

config.pre_action_hook = validate_action
```

## References

- [Claude Code Agent SDK](https://github.com/anthropics/claude-code)
- [ReAct Pattern Paper](https://arxiv.org/abs/2210.03629)
- [Anthropic Safety Guidelines](https://www.anthropic.com/safety)
- [Claude Tool Use Documentation](https://docs.anthropic.com/en/docs/build-with-claude/tool-use)
