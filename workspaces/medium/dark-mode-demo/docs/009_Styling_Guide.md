# Styling Guide

This guide covers how to customize the visual appearance of your architecture diagrams in structurizr-rs.

## Style Basics

Styles are defined in the `views` section and apply to elements based on their tags.

```dsl
views {
    styles {
        element "Tag" {
            // style properties
        }
        relationship "Tag" {
            // style properties
        }
    }
}
```

## Default Tags

Every element automatically has certain tags:

| Element Type | Default Tags |
|-------------|--------------|
| Person | `Element`, `Person` |
| Software System | `Element`, `Software System` |
| Container | `Element`, `Container` |
| Component | `Element`, `Component` |
| Relationship | `Relationship` |

## Custom Tags

Add custom tags to elements for targeted styling:

```dsl
model {
    db = container "Database" "Storage" "PostgreSQL" "Database,Critical"
    cache = container "Cache" "Caching" "Redis" "Database,Performance"
}

views {
    styles {
        element "Database" {
            shape Cylinder
        }
        element "Critical" {
            stroke "#ff0000"
            strokeWidth 3
        }
    }
}
```

## Color Schemes

### C4 Model Colors

The standard C4 color palette:

```dsl
styles {
    element "Person" {
        background "#08427b"  // Dark blue
        color "#ffffff"
    }
    element "Software System" {
        background "#1168bd"  // Blue
        color "#ffffff"
    }
    element "Container" {
        background "#438dd5"  // Light blue
        color "#ffffff"
    }
    element "Component" {
        background "#85bbf0"  // Lighter blue
        color "#000000"
    }
}
```

### External Systems

Distinguish external systems:

```dsl
element "External System" {
    background "#999999"  // Gray
    color "#ffffff"
}
```

### Databases

Highlight data stores:

```dsl
element "Database" {
    shape Cylinder
    background "#438dd5"
}
```

## Shapes

### Available Shapes

```dsl
// People and roles
shape Person
shape Robot

// Systems and containers
shape Box
shape RoundedBox
shape Circle
shape Ellipse
shape Hexagon

// Infrastructure
shape Cylinder      // Database
shape Pipe          // Queue/Message broker
shape Folder        // File storage

// Applications
shape WebBrowser
shape MobileDevicePortrait
shape MobileDeviceLandscape
shape Component
```

### Shape Selection Guide

| Use Case | Recommended Shape |
|----------|-------------------|
| User/Actor | Person |
| Bot/Automated system | Robot |
| Database | Cylinder |
| Message queue | Pipe |
| File storage | Folder |
| Web application | WebBrowser |
| Mobile app | MobileDevicePortrait |
| Generic service | Box or RoundedBox |
| Microservice | Hexagon |

## Icons

Add icons to elements for quick visual identification:

```dsl
element "Database" {
    shape Cylinder
    icon "https://example.com/icons/database.png"
    iconPosition Top
}

element "Container" {
    icon "data:image/svg+xml;base64,..."  // Base64 data URI
}
```

### Icon Positions

- **Top**: Icon appears above the text (default)
- **Bottom**: Icon appears below the text
- **Left**: Icon appears to the left of the text

```dsl
element "Service" {
    iconPosition Top     // Default
}

element "Database" {
    iconPosition Left    // Icon on left side
}
```

## Borders

Customize element borders:

```dsl
element "Container" {
    stroke "#2e6295"     // Border color
    strokeWidth 2        // Border thickness
    border Solid         // Border style: Solid, Dashed, Dotted
}
```

### Border Styles

```dsl
border Solid    // Continuous line
border Dashed   // Dashed line
border Dotted   // Dotted line
```

## Typography

Control text appearance:

```dsl
element "Container" {
    fontSize 24          // Text size in pixels
    color "#ffffff"      // Text color
}
```

## Relationships

Style connections between elements:

```dsl
relationship "Relationship" {
    color "#707070"      // Line color
    thickness 2          // Line thickness
    style Dashed         // Line style
    routing Orthogonal   // Line routing
    fontSize 18          // Label font size
}
```

### Line Styles

```dsl
style Solid     // Continuous line (default)
style Dashed    // Dashed line
style Dotted    // Dotted line
```

### Routing Options

```dsl
routing Direct       // Straight lines
routing Orthogonal   // Right-angle lines
routing Curved       // Curved lines
```

## Opacity

Control element transparency (0-100):

```dsl
element "Deprecated" {
    opacity 50    // 50% transparent
}

relationship "Optional" {
    opacity 75    // 75% opaque
}
```

## Size Control

Set explicit element dimensions:

```dsl
element "Large System" {
    width 500
    height 400
}

element "Compact" {
    width 200
    height 150
}
```

## Metadata Display

Control what information appears on elements:

```dsl
element "Container" {
    metadata true       // Show technology
    description true    // Show description
}
```

## Style Inheritance

Styles cascade based on tag order. More specific tags override general ones:

```dsl
styles {
    // Base style for all elements
    element "Element" {
        background "#1168bd"
        color "#ffffff"
        fontSize 20
    }

    // Override for containers
    element "Container" {
        background "#438dd5"
    }

    // Further override for databases
    element "Database" {
        shape Cylinder
        background "#23527c"
    }
}
```

## Complete Styling Example

```dsl
views {
    styles {
        // People
        element "Person" {
            shape Person
            background "#08427b"
            color "#ffffff"
            fontSize 22
        }

        // Internal systems
        element "Software System" {
            background "#1168bd"
            color "#ffffff"
            strokeWidth 2
        }

        // External systems
        element "External System" {
            background "#999999"
            color "#ffffff"
            border Dashed
        }

        // Containers
        element "Container" {
            background "#438dd5"
            color "#ffffff"
        }

        // Databases
        element "Database" {
            shape Cylinder
            background "#23527c"
        }

        // Web browsers
        element "Web Browser" {
            shape WebBrowser
        }

        // Mobile apps
        element "Mobile" {
            shape MobileDevicePortrait
        }

        // All relationships
        relationship "Relationship" {
            color "#707070"
            thickness 2
            routing Orthogonal
        }

        // Async relationships
        relationship "Async" {
            style Dashed
            color "#999999"
        }
    }
}
```

## Dark Mode Considerations

When using dark mode backgrounds, adjust colors for visibility:

```dsl
views {
    container system "Dark" {
        include *
        background "#1a1a1a"
        autoLayout
    }

    styles {
        element "Container" {
            background "#2d5986"    // Slightly lighter for contrast
            stroke "#5a9fd4"        // Visible border
        }

        relationship "Relationship" {
            color "#888888"          // Lighter for visibility
        }
    }
}
```

See [Dark Mode](010_Dark_Mode.md) for more details.
