# Groovy Migration Guide

This guide helps you migrate existing Structurizr Groovy scripts to work with structurizr-rs. The transpiler automatically converts common Groovy patterns to Lua, achieving approximately 80% compatibility with typical Structurizr scripts.

## Overview

structurizr-rs uses **Lua** as its native scripting language instead of Groovy/Kotlin. However, to maintain backwards compatibility with existing Structurizr DSL files, Groovy scripts are automatically transpiled to Lua when parsed.

```dsl
# This Groovy script...
!script groovy {
    workspace.name = "Modified"
    def sys = workspace.addSoftwareSystem("New System", "Description")
    println("Done!")
}

# ...is automatically converted to Lua:
!script lua {
    workspace:setName("Modified")
    local sys = workspace:addSoftwareSystem("New System", "Description")
    print("Done!")
}
```

## Supported Patterns

### Comments

| Groovy | Lua |
|--------|-----|
| `// single line` | `-- single line` |
| `/* block */` | `--[[ block ]]` |

**Example:**
```groovy
// Groovy comment
/* Multi-line
   comment */
```
Becomes:
```lua
-- Groovy comment
--[[ Multi-line
   comment ]]
```

### Variable Declarations

| Groovy | Lua |
|--------|-----|
| `def x = value` | `local x = value` |

**Example:**
```groovy
def name = "Test"
def count = 42
```
Becomes:
```lua
local name = "Test"
local count = 42
```

### Method Calls

Groovy dot notation is converted to Lua colon notation for workspace methods:

| Groovy | Lua |
|--------|-----|
| `workspace.addPerson(...)` | `workspace:addPerson(...)` |
| `workspace.addSoftwareSystem(...)` | `workspace:addSoftwareSystem(...)` |
| `workspace.setName(...)` | `workspace:setName(...)` |
| `workspace.setDescription(...)` | `workspace:setDescription(...)` |

**Example:**
```groovy
workspace.addPerson("User", "End user")
workspace.addSoftwareSystem("API", "Backend API")
```
Becomes:
```lua
workspace:addPerson("User", "End user")
workspace:addSoftwareSystem("API", "Backend API")
```

### Property Access

Property getters and setters are converted to method calls:

| Groovy | Lua |
|--------|-----|
| `workspace.name` (getter) | `workspace:getName()` |
| `workspace.name = "x"` (setter) | `workspace:setName("x")` |
| `workspace.description` | `workspace:getDescription()` |
| `workspace.description = "x"` | `workspace:setDescription("x")` |

**Example:**
```groovy
workspace.name = "New Name"
println(workspace.name)
```
Becomes:
```lua
workspace:setName("New Name")
print(workspace:getName())
```

### Print Statements

| Groovy | Lua |
|--------|-----|
| `println(...)` | `print(...)` |

**Example:**
```groovy
println("Hello, World!")
println("Count: " + 42)
```
Becomes:
```lua
print("Hello, World!")
print("Count: " .. 42)
```

### Each Closures (Iteration)

Simple `.each { }` closures are converted to Lua `for` loops:

| Groovy | Lua |
|--------|-----|
| `list.each { item -> ... }` | `for _, item in ipairs(list) do ... end` |

**Example:**
```groovy
systems.each { sys ->
    println(sys.name)
}
```
Becomes:
```lua
for _, sys in ipairs(systems) do
    print(sys.name)
end
```

## Migration Examples

### Example 1: Simple Script

**Original Groovy:**
```groovy
!script groovy {
    // Update workspace
    workspace.name = "Production Environment"
    workspace.description = "Updated by script"

    // Add elements
    def admin = workspace.addPerson("Admin", "System administrator")
    def monitoring = workspace.addSoftwareSystem("Monitoring", "Observability platform")

    println("Script complete!")
}
```

**Equivalent Lua:**
```lua
!script lua {
    -- Update workspace
    workspace:setName("Production Environment")
    workspace:setDescription("Updated by script")

    -- Add elements
    local admin = workspace:addPerson("Admin", "System administrator")
    local monitoring = workspace:addSoftwareSystem("Monitoring", "Observability platform")

    print("Script complete!")
}
```

### Example 2: Iteration

**Original Groovy:**
```groovy
!script groovy {
    def services = ["auth", "users", "orders"]
    services.each { name ->
        workspace.addSoftwareSystem(name.capitalize() + " Service", "Handles " + name)
    }
}
```

**Equivalent Lua:**
```lua
!script lua {
    local services = {"auth", "users", "orders"}
    for _, name in ipairs(services) do
        workspace:addSoftwareSystem(
            name:gsub("^%l", string.upper) .. " Service",
            "Handles " .. name
        )
    end
}
```

## Unsupported Features

The following Groovy features are **not supported** and require manual migration:

### Annotations

```groovy
// NOT SUPPORTED
@Grab('org.example:library:1.0')
@Grapes([@Grab('...')])
```

**Solution:** Remove annotations; external dependencies are not supported in the sandbox.

### Import Statements

```groovy
// NOT SUPPORTED
import groovy.json.JsonSlurper
import java.util.Date
```

**Solution:** Remove imports; use built-in Lua functions instead.

### Class Definitions

```groovy
// NOT SUPPORTED
class MyHelper {
    static String format(String s) { return s.toUpperCase() }
}
```

**Solution:** Convert to Lua functions:
```lua
local function format(s)
    return string.upper(s)
end
```

### Object Instantiation

```groovy
// NOT SUPPORTED
def date = new Date()
def map = new HashMap()
```

**Solution:** Use Lua equivalents:
```lua
local date = os.date()
local map = {}
```

### MetaClass Modifications

```groovy
// NOT SUPPORTED
String.metaClass.toSnakeCase = { ... }
```

**Solution:** No equivalent; use regular functions.

### .with Closures

```groovy
// NOT SUPPORTED
system.with {
    name = "New Name"
    description = "New Description"
}
```

**Solution:** Use explicit method calls:
```lua
workspace:setName("New Name")
workspace:setDescription("New Description")
```

### Try-Catch Blocks

```groovy
// NOT SUPPORTED
try {
    workspace.addPerson("User", "")
} catch (Exception e) {
    println("Error: " + e.message)
}
```

**Solution:** Use Lua's `pcall` for error handling:
```lua
local success, err = pcall(function()
    workspace:addPerson("User", "")
end)
if not success then
    print("Error: " .. tostring(err))
end
```

### Switch Statements

```groovy
// NOT SUPPORTED
switch (type) {
    case "person": workspace.addPerson(name, desc); break
    case "system": workspace.addSoftwareSystem(name, desc); break
}
```

**Solution:** Use Lua if-elseif:
```lua
if type == "person" then
    workspace:addPerson(name, desc)
elseif type == "system" then
    workspace:addSoftwareSystem(name, desc)
end
```

## Compatibility Checking

The transpiler can check scripts for unsupported features before conversion:

```rust
use structurizr_scripting::GroovyTranspiler;

let transpiler = GroovyTranspiler::new();
let issues = transpiler.check_compatibility(groovy_script);

if !issues.is_empty() {
    println!("Migration required for:");
    for issue in issues {
        println!("  - {}", issue);
    }
}
```

## Best Practices for Migration

### 1. Start Simple

Begin by converting the script to pure Lua without relying on auto-transpilation:

```dsl
# Instead of this (auto-transpiled):
!script groovy {
    workspace.name = "Test"
}

# Write native Lua:
!script lua {
    workspace:setName("Test")
}
```

### 2. Test Incrementally

Convert one section at a time and verify the workspace builds correctly.

### 3. Use Print Debugging

Add `print()` statements to verify script execution:

```lua
print("Before modification: " .. workspace:getName())
workspace:setName("New Name")
print("After modification: " .. workspace:getName())
```

### 4. Check for Nil Values

Lua returns `nil` for missing values, which can cause errors:

```lua
local desc = workspace:getDescription()
if desc then
    print("Description: " .. desc)
else
    print("No description set")
end
```

### 5. Handle Element Lookups

Always check if elements exist before using them:

```lua
local system = workspace:findElementByName("API")
if system then
    print("Found: " .. system.name)
else
    print("System not found - creating it")
    workspace:addSoftwareSystem("API", "Backend API")
end
```

## Syntax Reference

| Groovy | Lua | Notes |
|--------|-----|-------|
| `//` | `--` | Line comments |
| `/* */` | `--[[ ]]` | Block comments |
| `def x = y` | `local x = y` | Variable declaration |
| `obj.method()` | `obj:method()` | Method calls |
| `obj.property` | `obj:getProperty()` | Property getter |
| `obj.property = x` | `obj:setProperty(x)` | Property setter |
| `println()` | `print()` | Console output |
| `list.each { x -> }` | `for _, x in ipairs(list) do end` | Iteration |
| `"" + var` | `"" .. var` | String concatenation |
| `true`, `false` | `true`, `false` | Booleans (same) |
| `null` | `nil` | Null/nil value |

## See Also

- [Scripting Guide](scripting.md) - Getting started with scripting
- [Scripting API Reference](scripting-api-reference.md) - Complete API documentation
- [Transpiler Implementation](../development/transpiler-impl.md) - Technical details
