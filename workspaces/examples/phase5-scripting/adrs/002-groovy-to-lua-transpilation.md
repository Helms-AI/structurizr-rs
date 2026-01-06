# ADR 002: Groovy to Lua Transpilation

## Status
Accepted

## Context
Many existing Structurizr workspaces contain Groovy scripts that perform programmatic workspace modifications. To maintain backwards compatibility, we need to support these scripts without requiring a JVM.

## Decision
We will implement a **Groovy-to-Lua transpiler** that converts common Groovy patterns to equivalent Lua code before execution.

### Supported Patterns

#### Property Access
```groovy
// Groovy
workspace.name
```
```lua
-- Lua (transpiled)
workspace:getName()
```

#### Property Assignment
```groovy
// Groovy
workspace.name = "New Name"
```
```lua
-- Lua (transpiled)
workspace:setName("New Name")
```

#### Method Calls
```groovy
// Groovy
workspace.addPerson("Name", "Desc")
```
```lua
-- Lua (transpiled)
workspace:addPerson("Name", "Desc")
```

#### Variable Declarations
```groovy
// Groovy
def myVar = "value"
```
```lua
-- Lua (transpiled)
local myVar = "value"
```

#### Collections Iteration
```groovy
// Groovy
list.each { item ->
    println(item.name)
}
```
```lua
-- Lua (transpiled)
for _, item in ipairs(list) do
    print(item:getName())
end
```

#### Comments
```groovy
// Groovy single-line
/* Groovy
   multi-line */
```
```lua
-- Lua single-line
--[[ Lua
   multi-line ]]
```

### Not Supported
- Closures with complex syntax
- Groovy-specific operators (`?.`, `*:`, etc.)
- Meta-programming features
- Java interop calls

## Consequences

### Positive
- ~80% of existing scripts work without modification
- Gradual migration path to native Lua
- Clear error messages for unsupported features

### Negative
- Not 100% compatible with all Groovy features
- Some scripts will need manual conversion
- Transpilation adds slight parsing overhead

## Recommendation
For new projects, write scripts in Lua directly for best performance and compatibility. Use Groovy syntax only when migrating existing workspaces.
