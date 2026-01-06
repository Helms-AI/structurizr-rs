# Phase 5 Scripting Documentation

This example demonstrates the native scripting support implemented in Phase 5 of structurizr-rs.

## Features Demonstrated

### Native Lua Scripting

Lua is the native scripting language for structurizr-rs. Scripts can modify the workspace programmatically:

```dsl
!script lua {
    -- Modify workspace properties
    workspace:setDescription("Modified by script")

    -- Add elements dynamically
    local person = workspace:addPerson("Admin", "System administrator")
    local system = workspace:addSoftwareSystem("Tool", "A useful tool")

    -- Debug output
    print("Script executed!")
}
```

### Groovy Compatibility (Auto-Transpilation)

For backwards compatibility with existing Structurizr scripts, Groovy syntax is automatically transpiled to Lua:

```dsl
!script groovy {
    // Groovy property syntax
    workspace.name = workspace.name + " (Modified)"

    // Method calls
    def system = workspace.addSoftwareSystem("Name", "Description")

    // Print statements
    println("Hello from Groovy!")
}
```

### Transpilation Mapping

| Groovy Syntax | Lua Equivalent |
|--------------|----------------|
| `workspace.name` | `workspace:getName()` |
| `workspace.name = "x"` | `workspace:setName("x")` |
| `workspace.addPerson(...)` | `workspace:addPerson(...)` |
| `def x = ...` | `local x = ...` |
| `println(...)` | `print(...)` |
| `// comment` | `-- comment` |
| `/* block */` | `--[[ block ]]` |
| `list.each { x -> }` | `for _, x in ipairs(list) do end` |

## Workspace API

Scripts have access to the `workspace` object with these methods:

### Read Operations
- `workspace:getName()` - Get workspace name
- `workspace:getDescription()` - Get workspace description
- `workspace:getPeople()` - Get list of people
- `workspace:getSoftwareSystems()` - Get list of software systems
- `workspace:findElementByName(name)` - Find element by name

### Write Operations
- `workspace:setName(name)` - Set workspace name
- `workspace:setDescription(desc)` - Set workspace description
- `workspace:addPerson(name, description)` - Add a person
- `workspace:addSoftwareSystem(name, description)` - Add a software system

## Security & Sandboxing

Scripts run in a sandboxed environment with:
- No filesystem access (by default)
- No network access
- CPU time limits
- Memory limits

Configure via `SandboxConfig`:
```rust
let config = SandboxConfig::default()
    .with_max_instructions(1_000_000)
    .with_max_memory(64 * 1024 * 1024);
```

## Building with Scripting Support

Scripting is an optional feature. Enable it with:

```bash
cargo build --features scripting
```

Without this feature, `!script` directives are parsed but not executed.
