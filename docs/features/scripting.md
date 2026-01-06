# Scripting

structurizr-rs supports native Lua scripting through the `!script` directive, allowing you to programmatically modify workspaces during parsing. It also provides automatic transpilation of Groovy scripts for backwards compatibility with existing Structurizr DSL files.

## Quick Start

Add a script block to your workspace file:

```dsl
workspace "My System" "A system with scripted elements" {

    !script lua {
        -- Add a person dynamically
        workspace:addPerson("Admin", "System administrator")

        -- Add a software system
        workspace:addSoftwareSystem("Monitoring", "Observability platform")

        print("Script executed!")
    }

    model {
        user = person "User" "End user of the system"
        system = softwareSystem "Main System" "The primary application"

        user -> system "Uses"
    }

    views {
        systemLandscape "Landscape" "All systems" {
            include *
            autoLayout
        }
    }
}
```

## Feature Flag

Scripting is enabled by default. If you need to disable it:

```bash
# Build without scripting
cargo build --no-default-features
```

## Script Languages

### Lua (Recommended)

Lua is the native scripting language for structurizr-rs. Use the `!script lua { ... }` syntax:

```dsl
!script lua {
    -- Lua comments use double-dash
    workspace:setName("Modified Workspace")
    workspace:setDescription("Updated by script")

    -- Add elements programmatically
    local admin = workspace:addPerson("Admin", "System administrator")
    local monitoring = workspace:addSoftwareSystem("Monitoring", "Observability")

    -- Print debugging output
    print("Added admin and monitoring system")
}
```

### Groovy (Backwards Compatible)

For compatibility with existing Structurizr DSL files, Groovy scripts are automatically transpiled to Lua:

```dsl
!script groovy {
    // Groovy-style comments work
    workspace.setName("Scripted Workspace")

    // Add elements using Groovy syntax
    def cicd = workspace.addSoftwareSystem("CI/CD", "Build pipeline")

    println("Groovy script transpiled and executed!")
}
```

**Note:** Not all Groovy features are supported. See the [Groovy Migration Guide](groovy-migration.md) for details.

## Common Use Cases

### Dynamic Element Creation

Create elements based on configuration or environment:

```dsl
!script lua {
    -- Create microservices dynamically
    local services = {"auth", "users", "orders", "inventory"}

    for _, name in ipairs(services) do
        workspace:addSoftwareSystem(
            name:gsub("^%l", string.upper) .. " Service",
            "Handles " .. name .. " domain"
        )
    end
}
```

### Workspace Metadata

Modify workspace properties:

```dsl
!script lua {
    -- Update workspace metadata
    workspace:setName("Production Environment")
    workspace:setDescription("Updated: " .. os.date("%Y-%m-%d"))

    -- Set custom properties
    workspace:setProperty("environment", "production")
    workspace:setProperty("version", "2.1.0")
}
```

### Conditional Elements

Add elements based on conditions:

```dsl
!script lua {
    -- Check a property or environment
    local env = workspace:getProperty("environment") or "development"

    if env == "production" then
        workspace:addSoftwareSystem("WAF", "Web Application Firewall")
        workspace:addSoftwareSystem("CDN", "Content Delivery Network")
    end
}
```

### Bulk Relationship Creation

Create relationships programmatically:

```dsl
!script lua {
    -- First add the systems
    workspace:addPerson("User", "End user")
    workspace:addSoftwareSystem("API Gateway", "Entry point")
    workspace:addSoftwareSystem("Auth Service", "Authentication")
    workspace:addSoftwareSystem("User Service", "User management")

    -- Create relationships
    workspace:addRelationshipByName("User", "API Gateway", "Calls", "HTTPS")
    workspace:addRelationshipByName("API Gateway", "Auth Service", "Authenticates via", "gRPC")
    workspace:addRelationshipByName("API Gateway", "User Service", "Routes to", "gRPC")
}
```

## External Script Files

Load scripts from external files:

```dsl
workspace "My System" {
    !script "scripts/setup.lua"

    model {
        // ...
    }
}
```

The file extension determines the language:
- `.lua` - Lua script
- `.groovy` - Groovy script (transpiled to Lua)

## API Reference

See the [Scripting API Reference](scripting-api-reference.md) for the complete list of available functions.

### Key Methods

| Method | Description |
|--------|-------------|
| `workspace:getName()` | Get workspace name |
| `workspace:setName(name)` | Set workspace name |
| `workspace:addPerson(name, desc)` | Add a person element |
| `workspace:addSoftwareSystem(name, desc)` | Add a software system |
| `workspace:findElementByName(name)` | Find element by name |
| `workspace:setProperty(key, value)` | Set custom property |

## Error Handling

Script errors are reported with line numbers and context:

```
Error executing script: [string "script"]:3: attempt to call a nil value (method 'unknownMethod')
```

**Tips for debugging:**
- Use `print()` to output debug information
- Check the console for `[Script]` prefixed output
- Verify method names match the API exactly

## Security

Scripts run in a sandboxed Lua environment with:

- **Limited stdlib**: Only math, string, table, coroutine by default
- **Memory limits**: Default 10MB maximum
- **Execution timeout**: Default 5 seconds
- **No filesystem access**: By default
- **No network access**: Always disabled

For advanced configuration, see the [Sandbox Implementation](../development/sandbox-impl.md).

## Examples

See the example workspaces for more scripting patterns:

- `workspaces/examples/phase5-scripting/` - Basic scripting examples
- `workspaces/examples/comprehensive/` - Combined features including scripting

## Limitations

1. **Script execution order**: Scripts execute in document order, before the model is built
2. **No access to DSL-defined elements**: Scripts cannot reference elements defined later in the DSL
3. **Groovy subset**: Only common Groovy patterns are transpiled
4. **No external dependencies**: Cannot load external Lua modules by default

## Next Steps

- [Scripting API Reference](scripting-api-reference.md) - Complete API documentation
- [Groovy Migration Guide](groovy-migration.md) - Migrate existing Groovy scripts
- [WASM Plugins](plugins.md) - Advanced extensibility with WASM
