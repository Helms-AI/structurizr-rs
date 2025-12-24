# DSL Reference

Complete reference for the Structurizr DSL syntax supported by structurizr-rs.

## Workspace

The top-level container for all architecture definitions:

```dsl
workspace [name] [description] {
    // directives
    // model
    // views
}
```

### Example

```dsl
workspace "My System" "A software system" {
    !docs "docs"
    !adrs "adrs"

    model {
        // ...
    }

    views {
        // ...
    }
}
```

## Directives

Directives are prefixed with `!` and configure workspace-level settings.

### !docs

Specifies the documentation directory:

```dsl
!docs "docs"
```

### !adrs

Specifies the ADR (Architecture Decision Records) directory:

```dsl
!adrs "adrs"
```

### !include

Includes another DSL file:

```dsl
!include "common.dsl"
!include "systems/external.dsl"
```

### !const

Defines a constant for reuse:

```dsl
!const TECH_STACK "Rust, PostgreSQL, React"
```

### !impliedRelationships

Enables automatic relationship inference:

```dsl
!impliedRelationships true
```

## Model

The model section defines architecture elements and relationships.

```dsl
model {
    // people
    // software systems
    // relationships
    // groups
}
```

### Person

```dsl
identifier = person "Name" "Description" [tags]
```

Examples:
```dsl
user = person "User" "A customer"
admin = person "Administrator" "System administrator" "Admin"
```

### Software System

```dsl
identifier = softwareSystem "Name" "Description" [tags] {
    // containers
}
```

Examples:
```dsl
system = softwareSystem "My System" "Main application"

system = softwareSystem "My System" "Description" {
    webapp = container "Web App" "User interface" "React"
    api = container "API" "Backend" "Rust"
}
```

### Container

Containers are defined within a software system:

```dsl
identifier = container "Name" "Description" "Technology" [tags] {
    // components
}
```

Examples:
```dsl
webapp = container "Web Application" "Frontend" "React, TypeScript"
api = container "API Server" "Backend services" "Rust, Axum"
db = container "Database" "Data storage" "PostgreSQL" "Database"
```

### Component

Components are defined within a container:

```dsl
identifier = component "Name" "Description" "Technology" [tags]
```

Examples:
```dsl
controller = component "API Controller" "HTTP handlers" "Axum"
service = component "Business Service" "Business logic" "Rust"
repository = component "Repository" "Data access" "SQLx"
```

### Relationships

Relationships connect elements:

```dsl
source -> destination "Description" [technology] [tags]
```

Examples:
```dsl
user -> system "Uses"
webapp -> api "Calls" "HTTPS/JSON"
api -> db "Reads/Writes" "SQL"
```

Relationships can also be defined inline:

```dsl
user = person "User" {
    -> system "Uses"
}
```

### Groups

Groups organize related elements:

```dsl
group "Group Name" {
    // elements
}
```

Example:
```dsl
group "External Systems" {
    payment = softwareSystem "Payment Gateway" "Processes payments"
    email = softwareSystem "Email Service" "Sends emails"
}
```

## Views

Views define how the model is visualized.

```dsl
views {
    // view definitions
    // styles
}
```

### System Landscape View

Shows all software systems:

```dsl
systemLandscape "Key" "Description" {
    include *
    autoLayout
}
```

### System Context View

Shows a system and its context:

```dsl
systemContext <system> "Key" "Description" {
    include *
    exclude <element>
    autoLayout [direction] [rankSeparation] [nodeSeparation]
}
```

Example:
```dsl
systemContext system "Context" "System Context Diagram" {
    include *
    autoLayout lr
}
```

### Container View

Shows containers within a system:

```dsl
container <system> "Key" "Description" {
    include *
    autoLayout
}
```

### Component View

Shows components within a container:

```dsl
component <container> "Key" "Description" {
    include *
    autoLayout
}
```

### Include/Exclude

Control which elements appear in views:

```dsl
include *                    # Include all elements
include element              # Include specific element
include ->element->          # Include element and relationships
exclude element              # Exclude specific element
```

### Auto Layout

Configure automatic layout:

```dsl
autoLayout                   # Default (top to bottom)
autoLayout tb               # Top to bottom
autoLayout bt               # Bottom to top
autoLayout lr               # Left to right
autoLayout rl               # Right to left
autoLayout lr 300 200       # With rank/node separation
```

### Background

Set view background color (for dark mode):

```dsl
background "#1a1a1a"
```

## Styles

Define visual styles for elements and relationships.

```dsl
styles {
    element "Tag" {
        // properties
    }
    relationship "Tag" {
        // properties
    }
}
```

### Element Style Properties

| Property | Description | Example |
|----------|-------------|---------|
| shape | Element shape | `shape Box` |
| icon | Icon URL | `icon "https://..."` |
| iconPosition | Icon placement | `iconPosition Top` |
| width | Element width | `width 450` |
| height | Element height | `height 300` |
| background | Fill color | `background "#438dd5"` |
| color | Text color | `color "#ffffff"` |
| stroke | Border color | `stroke "#2e6295"` |
| strokeWidth | Border width | `strokeWidth 2` |
| fontSize | Text size | `fontSize 24` |
| border | Border style | `border Solid` |
| opacity | Transparency | `opacity 100` |
| metadata | Show metadata | `metadata true` |
| description | Show description | `description true` |

### Shapes

Available shapes:
- `Box` - Rectangle
- `RoundedBox` - Rounded rectangle
- `Circle` - Circle
- `Ellipse` - Ellipse
- `Hexagon` - Hexagon
- `Cylinder` - Database cylinder
- `Pipe` - Queue/pipe
- `Person` - Person shape
- `Robot` - Robot/bot shape
- `Folder` - Folder shape
- `WebBrowser` - Browser window
- `MobileDevicePortrait` - Mobile portrait
- `MobileDeviceLandscape` - Mobile landscape
- `Component` - Component shape

### Icon Positions

- `Top` - Icon at top of element
- `Bottom` - Icon at bottom
- `Left` - Icon on left side

### Relationship Style Properties

| Property | Description | Example |
|----------|-------------|---------|
| thickness | Line thickness | `thickness 2` |
| color | Line color | `color "#707070"` |
| style | Line style | `style Dashed` |
| routing | Line routing | `routing Orthogonal` |
| fontSize | Label size | `fontSize 24` |
| width | Label width | `width 200` |
| position | Label position | `position 50` |
| opacity | Transparency | `opacity 100` |

### Line Styles

- `Solid` - Solid line
- `Dashed` - Dashed line
- `Dotted` - Dotted line

### Routing

- `Direct` - Straight lines
- `Orthogonal` - Right-angle lines
- `Curved` - Curved lines

## Complete Example

```dsl
workspace "E-Commerce Platform" "Online shopping system" {
    !docs "docs"
    !adrs "adrs"

    model {
        customer = person "Customer" "Online shopper"

        ecommerce = softwareSystem "E-Commerce" "Shopping platform" {
            web = container "Web App" "Frontend" "React"
            api = container "API" "Backend" "Rust"
            db = container "Database" "Storage" "PostgreSQL" "Database"
        }

        payment = softwareSystem "Payment Gateway" "External" "External System"

        customer -> web "Browses" "HTTPS"
        web -> api "Calls" "REST"
        api -> db "Queries" "SQL"
        api -> payment "Processes" "HTTPS"
    }

    views {
        systemContext ecommerce "Context" {
            include *
            autoLayout
        }

        container ecommerce "Containers" {
            include *
            autoLayout lr
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
                color "#ffffff"
            }
            element "Software System" {
                background "#1168bd"
                color "#ffffff"
            }
            element "Container" {
                background "#438dd5"
                color "#ffffff"
            }
            element "Database" {
                shape Cylinder
            }
            element "External System" {
                background "#999999"
            }
            relationship "Relationship" {
                color "#707070"
            }
        }
    }
}
```
