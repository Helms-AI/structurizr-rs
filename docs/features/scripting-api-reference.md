# Scripting API Reference

This document provides a complete reference for the Lua API available to scripts in structurizr-rs.

## Global Objects

### workspace

The `workspace` object is automatically available in all scripts and provides access to modify the workspace being parsed.

## Workspace Methods

### Name and Description

#### workspace:getName()

Returns the current workspace name.

**Returns:** `string` - The workspace name

**Example:**
```lua
local name = workspace:getName()
print("Workspace: " .. name)
```

---

#### workspace:setName(name)

Sets the workspace name.

**Parameters:**
- `name` (`string`) - The new workspace name

**Example:**
```lua
workspace:setName("My Updated Workspace")
```

---

#### workspace:getDescription()

Returns the current workspace description.

**Returns:** `string` or `nil` - The workspace description

**Example:**
```lua
local desc = workspace:getDescription()
if desc then
    print("Description: " .. desc)
end
```

---

#### workspace:setDescription(description)

Sets the workspace description.

**Parameters:**
- `description` (`string`) - The new description

**Example:**
```lua
workspace:setDescription("Updated workspace for production environment")
```

---

### Adding Elements

#### workspace:addPerson(name, description)

Adds a new Person element to the model.

**Parameters:**
- `name` (`string`) - The person's name
- `description` (`string`, optional) - The person's description

**Returns:** `string` - The UUID of the created person

**Example:**
```lua
local id = workspace:addPerson("Developer", "Software developer")
print("Created person with ID: " .. id)
```

---

#### workspace:addSoftwareSystem(name, description)

Adds a new Software System element to the model.

**Parameters:**
- `name` (`string`) - The system's name
- `description` (`string`, optional) - The system's description

**Returns:** `string` - The UUID of the created system

**Example:**
```lua
local id = workspace:addSoftwareSystem("Backend API", "RESTful API server")
print("Created system with ID: " .. id)
```

---

#### workspace:addContainerByName(systemName, name, description, technology)

Adds a Container to an existing Software System (identified by name).

**Parameters:**
- `systemName` (`string`) - Name of the parent software system
- `name` (`string`) - The container's name
- `description` (`string`, optional) - The container's description
- `technology` (`string`, optional) - The container's technology stack

**Returns:** `string` - The UUID of the created container

**Throws:** Error if the software system is not found

**Example:**
```lua
-- First create the system
workspace:addSoftwareSystem("Backend", "Backend services")

-- Then add containers to it
workspace:addContainerByName("Backend", "API Server", "REST API", "Node.js")
workspace:addContainerByName("Backend", "Database", "Data storage", "PostgreSQL")
```

---

### Adding Relationships

#### workspace:addRelationshipByName(sourceName, destName, description, technology)

Creates a relationship between two elements (identified by name).

**Parameters:**
- `sourceName` (`string`) - Name of the source element
- `destName` (`string`) - Name of the destination element
- `description` (`string`, optional) - Description of the relationship
- `technology` (`string`, optional) - Technology/protocol used

**Throws:** Error if source or destination element is not found

**Example:**
```lua
-- Create elements first
workspace:addPerson("User", "End user")
workspace:addSoftwareSystem("App", "Mobile application")

-- Create relationship
workspace:addRelationshipByName("User", "App", "Uses", "HTTPS")
```

---

### Finding Elements

#### workspace:findElementByName(name)

Finds an element by its name.

**Parameters:**
- `name` (`string`) - The element name to search for

**Returns:** `table` or `nil` - Element info table, or nil if not found

**Return table structure:**
```lua
{
    id = "uuid-string",      -- Element UUID
    name = "Element Name",   -- Element name
    type = "Person"          -- Element type: "Person", "SoftwareSystem", or "Container"
}
```

**Example:**
```lua
local element = workspace:findElementByName("Backend API")
if element then
    print("Found: " .. element.name .. " (" .. element.type .. ")")
    print("ID: " .. element.id)
else
    print("Element not found")
end
```

---

### Listing Elements

#### workspace:getPeople()

Returns all Person elements in the model.

**Returns:** `table` - Array of person info tables

**Table structure for each person:**
```lua
{
    id = "uuid-string",
    name = "Person Name",
    description = "Person description"
}
```

**Example:**
```lua
local people = workspace:getPeople()
print("People in model:")
for _, person in ipairs(people) do
    print("  - " .. person.name .. ": " .. (person.description or ""))
end
```

---

#### workspace:getSoftwareSystems()

Returns all Software System elements in the model.

**Returns:** `table` - Array of system info tables

**Table structure for each system:**
```lua
{
    id = "uuid-string",
    name = "System Name",
    description = "System description"
}
```

**Example:**
```lua
local systems = workspace:getSoftwareSystems()
print("Software Systems:")
for _, sys in ipairs(systems) do
    print("  - " .. sys.name)
end
```

---

### Properties

#### workspace:getProperty(key)

Gets a custom property value.

**Parameters:**
- `key` (`string`) - The property key

**Returns:** `string` or `nil` - The property value, or nil if not set

**Example:**
```lua
local env = workspace:getProperty("environment")
if env == "production" then
    print("Running in production mode")
end
```

---

#### workspace:setProperty(key, value)

Sets a custom property value.

**Parameters:**
- `key` (`string`) - The property key
- `value` (`string`) - The property value

**Example:**
```lua
workspace:setProperty("version", "2.1.0")
workspace:setProperty("lastUpdated", os.date("%Y-%m-%d"))
```

---

## Helper Functions

### print(...)

Outputs text to the console during script execution. Output is prefixed with `[Script]`.

**Parameters:**
- `...` - Any number of values to print

**Example:**
```lua
print("Processing workspace...")
print("Added", 3, "elements")
```

**Console output:**
```
[Script] Processing workspace...
[Script] Added	3	elements
```

---

### ipairs(table)

Standard Lua function for iterating over array-like tables.

**Parameters:**
- `table` - The table to iterate over

**Returns:** Iterator function, table, and initial index

**Example:**
```lua
local systems = workspace:getSoftwareSystems()
for i, sys in ipairs(systems) do
    print(i .. ": " .. sys.name)
end
```

---

## Available Standard Library

By default, scripts have access to these Lua standard libraries:

| Library | Description |
|---------|-------------|
| `math` | Mathematical functions (`math.floor`, `math.random`, etc.) |
| `string` | String manipulation (`string.upper`, `string.format`, etc.) |
| `table` | Table utilities (`table.insert`, `table.sort`, etc.) |
| `coroutine` | Coroutine functions |

**Not available by default:**
- `io` - File I/O
- `os` - Operating system functions (except `os.date`, `os.time`)
- `debug` - Debug library
- `package` - Package/module loading
- `require` - Module loading function

---

## Complete Example

```lua
-- Comprehensive script example
!script lua {
    -- Get current state
    print("Current workspace: " .. workspace:getName())

    -- Modify metadata
    workspace:setName("E-Commerce Platform")
    workspace:setDescription("Online shopping system architecture")
    workspace:setProperty("domain", "e-commerce")
    workspace:setProperty("version", "3.0.0")

    -- Add actors
    workspace:addPerson("Customer", "Online shopper")
    workspace:addPerson("Admin", "Store administrator")
    workspace:addPerson("Support", "Customer support agent")

    -- Add systems
    workspace:addSoftwareSystem("Web Store", "Customer-facing web application")
    workspace:addSoftwareSystem("Admin Portal", "Back-office management")
    workspace:addSoftwareSystem("Order Service", "Order processing")
    workspace:addSoftwareSystem("Payment Gateway", "Payment processing")
    workspace:addSoftwareSystem("Inventory", "Stock management")

    -- Add containers to Web Store
    workspace:addContainerByName("Web Store", "Frontend", "SPA", "React")
    workspace:addContainerByName("Web Store", "API", "REST API", "Node.js")
    workspace:addContainerByName("Web Store", "Database", "Data store", "PostgreSQL")

    -- Add relationships
    workspace:addRelationshipByName("Customer", "Web Store", "Browses and purchases", "HTTPS")
    workspace:addRelationshipByName("Admin", "Admin Portal", "Manages store", "HTTPS")
    workspace:addRelationshipByName("Support", "Admin Portal", "Handles tickets", "HTTPS")
    workspace:addRelationshipByName("Web Store", "Order Service", "Creates orders", "gRPC")
    workspace:addRelationshipByName("Web Store", "Payment Gateway", "Processes payments", "HTTPS")
    workspace:addRelationshipByName("Order Service", "Inventory", "Reserves stock", "gRPC")

    -- Report what was created
    local people = workspace:getPeople()
    local systems = workspace:getSoftwareSystems()
    print("Created " .. #people .. " people and " .. #systems .. " systems")

    -- Verify an element
    local store = workspace:findElementByName("Web Store")
    if store then
        print("Web Store ID: " .. store.id)
    end
}
```

---

## Error Handling

Scripts that encounter errors will fail the workspace parsing. Common errors include:

| Error | Cause |
|-------|-------|
| `attempt to call a nil value` | Calling a method that doesn't exist |
| `Software system not found` | `addContainerByName` with invalid system name |
| `Source element not found` | `addRelationshipByName` with invalid source name |
| `Destination element not found` | `addRelationshipByName` with invalid destination name |

**Best practices:**
- Use `findElementByName` to verify elements exist before creating relationships
- Check return values when they might be nil
- Use `print` for debugging during development

---

## See Also

- [Scripting Guide](scripting.md) - Getting started with scripting
- [Groovy Migration Guide](groovy-migration.md) - Migrate from Groovy scripts
- [Scripting Implementation](../development/scripting-impl.md) - Technical details
