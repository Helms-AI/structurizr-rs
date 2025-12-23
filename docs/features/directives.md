# DSL Directives

This document describes the DSL directives supported by structurizr-rs.

## Overview

Directives are special commands in the DSL that control parsing and workspace behavior. They are prefixed with `!` and must appear at the workspace level (before or after `model` and `views` blocks).

## Supported Directives

### !const - Constants

Define reusable constants that can be substituted throughout your DSL.

**Syntax:**
```dsl
!const NAME "value"
```

**Usage:**
```dsl
workspace {
    !const COMPANY "Acme Corp"
    !const DATABASE "PostgreSQL 15"
    !const API_PROTOCOL "HTTPS/REST"

    model {
        user = person "${COMPANY} User" "A user of ${COMPANY} systems"
        system = softwareSystem "System" "Uses ${DATABASE}"
        user -> system "Connects via ${API_PROTOCOL}"
    }
}
```

Constants are substituted using the `${CONST_NAME}` syntax and can be used in:
- Element names
- Element descriptions
- Relationship descriptions
- Relationship technologies
- View titles and descriptions

### !impliedRelationships - Transitive Relationships

Automatically generate implied relationships based on existing relationships.

**Syntax:**
```dsl
!impliedRelationships true|false
```

**Usage:**
```dsl
workspace {
    !impliedRelationships true

    model {
        a = person "A"
        b = softwareSystem "B"
        c = softwareSystem "C"

        a -> b "Uses"
        b -> c "Calls"
        // Automatically creates: a -> c (implied)
    }
}
```

When enabled, if there are relationships A→B and B→C, the parser will automatically create an implied relationship A→C. This is useful for showing transitive dependencies in system diagrams.

**Default:** `false` (disabled)

### !include - File Inclusion

Include external DSL files to modularize your architecture description.

**Syntax:**
```dsl
!include "path/to/file.dsl"
```

**Usage:**

Main file (`workspace.dsl`):
```dsl
workspace "My System" {
    !include "model/people.dsl"
    !include "model/systems.dsl"
    !include "views/views.dsl"
}
```

Included file (`model/people.dsl`):
```dsl
model {
    user = person "User"
    admin = person "Admin"
}
```

**Notes:**
- Paths are relative to the directory containing the workspace file
- Included files can include other files (recursive inclusion supported)
- Use `parse_with_base_path()` function to enable include support
- Included files should contain only `model` or `views` blocks, not full workspace definitions

### !docs - Documentation Path

Specify the path to documentation files.

**Syntax:**
```dsl
!docs "path/to/docs"
```

**Usage:**
```dsl
workspace {
    !docs "documentation"

    model {
        // ...
    }
}
```

The path is stored in the workspace properties as `structurizr.docs` and can be used by tools to locate and load documentation.

### !adrs - Architecture Decision Records Path

Specify the path to Architecture Decision Records (ADRs).

**Syntax:**
```dsl
!adrs "path/to/adrs"
```

**Usage:**
```dsl
workspace {
    !adrs "architecture-decisions"

    model {
        // ...
    }
}
```

The path is stored in the workspace properties as `structurizr.adrs` and can be used by tools to locate and load ADR files.

## Complete Example

```dsl
workspace "E-Commerce Platform" {
    // Define constants for reuse
    !const COMPANY "ShopCo"
    !const DATABASE "PostgreSQL"
    !const PROTOCOL "HTTPS"

    // Enable automatic transitive relationships
    !impliedRelationships true

    // Specify documentation locations
    !docs "docs"
    !adrs "decisions"

    // Include modular definitions
    !include "model/users.dsl"
    !include "model/systems.dsl"

    model {
        customer = person "${COMPANY} Customer"
        webapp = softwareSystem "Web App" "Built with ${DATABASE}"
        database = softwareSystem "Database"
        payment = softwareSystem "Payment Gateway"

        customer -> webapp "Uses via ${PROTOCOL}"
        webapp -> database "Stores data in ${DATABASE}"
        webapp -> payment "Processes payments"
        // Implied relationships will be created:
        // customer -> database
        // customer -> payment
    }

    views {
        systemContext webapp "Context" "System context for ${COMPANY}" {
            include *
            autoLayout
        }
    }
}
```

## API Usage

### Basic Parsing (without includes)

```rust
use structurizr_dsl::parse;

let dsl = r#"
workspace {
    !const NAME "Value"
    model {
        // ...
    }
}
"#;

let workspace = parse(dsl)?;
```

### Parsing with Include Support

```rust
use structurizr_dsl::parse_with_base_path;
use std::path::Path;

let dsl_content = std::fs::read_to_string("workspace.dsl")?;
let base_path = Path::new(".");
let workspace = parse_with_base_path(&dsl_content, Some(base_path))?;
```

### Accessing Directive Results

```rust
// Access constants (already substituted in the workspace)
let system_name = workspace.model().software_systems[0].name();

// Access docs and adrs paths
let docs_path = workspace.get_property("structurizr.docs");
let adrs_path = workspace.get_property("structurizr.adrs");

// Implied relationships are automatically added to the model
let relationships = workspace.model().relationships;
```

## Implementation Details

### Execution Order

Directives are processed in the following order:

1. **!include** - Files are included and merged into the AST
2. **!const** - Constants are collected into a map
3. **Constant substitution** - All `${NAME}` references are replaced
4. **Model building** - Elements and relationships are created
5. **!impliedRelationships** - Transitive relationships are generated
6. **!docs and !adrs** - Paths are stored in workspace properties

### Constant Substitution

Constants are substituted in:
- Workspace name and description
- Element names, descriptions, and technologies
- Relationship descriptions and technologies
- Group names
- View keys, titles, and descriptions

The substitution uses the format `${CONST_NAME}` and is case-sensitive.

### Implied Relationships Algorithm

The algorithm for generating implied relationships:

1. Build a map of source → [destinations] from existing relationships
2. For each relationship A→B:
   - Find all destinations of B (let's call them C)
   - For each C, check if A→C already exists
   - If not, create an implied relationship A→C with empty description

This is a single-pass algorithm that generates first-level implied relationships. For deeper transitive chains, you may need to run the parser multiple times or implement a more sophisticated transitive closure algorithm.

## Limitations

- **!include**: Included files should contain only model/views blocks, not full workspace definitions
- **!impliedRelationships**: Only generates first-level implied relationships in a single pass
- **Constants**: Substitution is simple string replacement, no expression evaluation
- All directives must be at workspace level, not inside model or views blocks

## Future Enhancements

Possible future directive enhancements:

- `!identifiers hierarchical|flat` - Control identifier scoping (already parsed but not executed)
- `!extends "workspace.json"` - Extend from existing workspace
- `!plugin "name"` - Load parser plugins
- Expression support in constants (e.g., `!const PORT ${BASE_PORT + 1}`)
- Multi-pass implied relationships for complete transitive closure
