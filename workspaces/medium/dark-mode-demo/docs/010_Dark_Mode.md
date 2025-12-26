# Dark Mode

structurizr-rs supports dark-themed diagrams through configurable background colors and adaptive styling.

## Enabling Dark Mode

Set the background color on individual views:

```dsl
views {
    systemContext system "DarkContext" "System Context (Dark)" {
        include *
        background "#1a1a1a"
        autoLayout
    }

    container system "DarkContainers" "Containers (Dark)" {
        include *
        background "#0d1117"
        autoLayout
    }
}
```

## Background Colors

### Recommended Dark Backgrounds

| Color | Hex | Description |
|-------|-----|-------------|
| GitHub Dark | `#0d1117` | GitHub's dark mode |
| VS Code Dark | `#1e1e1e` | VS Code default |
| Material Dark | `#121212` | Material Design |
| Neutral Dark | `#1a1a1a` | Balanced dark |
| Blue Dark | `#0a192f` | Blue-tinted dark |

### Light Mode Backgrounds

| Color | Hex | Description |
|-------|-----|-------------|
| White | `#ffffff` | Default |
| Off-White | `#f5f5f5` | Softer white |
| Warm White | `#fffbf0` | Warm tinted |

## Adaptive Colors

When using dark backgrounds, adjust element colors for visibility:

### Dark Mode Palette

```dsl
styles {
    // People - lighter blue
    element "Person" {
        shape Person
        background "#2d5a87"
        color "#ffffff"
        stroke "#4a90d9"
    }

    // Systems - medium blue
    element "Software System" {
        background "#3a7bc8"
        color "#ffffff"
        stroke "#5a9fd4"
    }

    // Containers - lighter shade
    element "Container" {
        background "#4a8fd9"
        color "#ffffff"
        stroke "#6ab0e8"
    }

    // External systems - visible gray
    element "External System" {
        background "#4a4a4a"
        color "#ffffff"
        stroke "#666666"
    }

    // Databases
    element "Database" {
        shape Cylinder
        background "#2d5a87"
        stroke "#4a90d9"
    }

    // Relationships - lighter lines
    relationship "Relationship" {
        color "#888888"
        thickness 2
    }
}
```

## Icon Support

Icons are rendered on top of element backgrounds and work well in both light and dark modes:

```dsl
element "Database" {
    shape Cylinder
    icon "https://example.com/icons/database-white.svg"
    iconPosition Top
    background "#2d5a87"
}

element "Web Application" {
    shape WebBrowser
    icon "https://example.com/icons/react-white.svg"
    iconPosition Top
}
```

### Icon Guidelines for Dark Mode

1. **Use light-colored icons** on dark backgrounds
2. **SVG icons** work best (scalable, transparent)
3. **Test visibility** against your background color
4. **Consider icon position** - Top is usually most visible

## Group Styling

Groups automatically adapt to dark mode with lighter boundaries:

```dsl
group "External Services" {
    // Group boundary will be visible on dark backgrounds
    payment = softwareSystem "Payment" "External"
    email = softwareSystem "Email" "External"
}
```

## Complete Dark Mode Example

```dsl
workspace "Dark Mode Demo" "Demonstrating dark theme" {
    model {
        user = person "User" "Application user"

        system = softwareSystem "Main System" {
            web = container "Web App" "Frontend" "React"
            api = container "API" "Backend" "Rust"
            db = container "Database" "Storage" "PostgreSQL" "Database"
            cache = container "Cache" "Caching" "Redis" "Database"
        }

        external = softwareSystem "External API" "Third party" "External System"

        user -> web "Uses"
        web -> api "Calls"
        api -> db "Queries"
        api -> cache "Caches"
        api -> external "Integrates"
    }

    views {
        container system "DarkContainers" "Container Diagram (Dark)" {
            include *
            background "#1a1a1a"
            autoLayout
        }

        styles {
            element "Person" {
                shape Person
                background "#2d5a87"
                color "#ffffff"
                stroke "#4a90d9"
                strokeWidth 2
            }

            element "Container" {
                background "#4a8fd9"
                color "#ffffff"
                stroke "#6ab0e8"
                strokeWidth 1
            }

            element "Database" {
                shape Cylinder
                background "#2d5a87"
                stroke "#4a90d9"
            }

            element "External System" {
                background "#4a4a4a"
                color "#ffffff"
                stroke "#666666"
                border Dashed
            }

            relationship "Relationship" {
                color "#888888"
                thickness 2
            }
        }
    }
}
```

## Render Configuration

When using the render API programmatically:

```rust
use structurizr_render::{SvgRenderer, RenderConfig};

let config = RenderConfig {
    background_color: "#1a1a1a".to_string(),
    ..Default::default()
};

let renderer = SvgRenderer::new(workspace)
    .with_config(config);

let svg = renderer.render_view("DarkContainers")?;
```

### Dark Mode Detection

The renderer automatically detects dark mode based on background color:

```rust
impl RenderConfig {
    pub fn is_dark_mode(&self) -> bool {
        // Detects colors starting with #0, #1, or #2
        self.background_color.starts_with("#0") ||
        self.background_color.starts_with("#1") ||
        self.background_color.starts_with("#2")
    }
}
```

This affects:
- Group boundary colors
- Default relationship colors
- Shadow and highlight colors

## Best Practices

### 1. Test Both Modes

Create views for both light and dark backgrounds:

```dsl
views {
    container system "Light" {
        include *
        autoLayout
    }

    container system "Dark" {
        include *
        background "#1a1a1a"
        autoLayout
    }
}
```

### 2. Maintain Contrast

Ensure sufficient contrast between:
- Text and background
- Element fill and stroke
- Relationship lines and background

### 3. Use Consistent Themes

Apply dark styling consistently across all views in a workspace.

### 4. Consider Export

When exporting to PlantUML or Mermaid, dark backgrounds may not transfer. Consider creating separate light-themed views for exports.

### 5. Icon Selection

Choose icons that work on both light and dark backgrounds, or provide different icon URLs for each theme.

## Troubleshooting

### Elements Hard to See

Increase stroke width and use contrasting stroke colors:

```dsl
element "Container" {
    strokeWidth 2
    stroke "#6ab0e8"  // Lighter than background
}
```

### Text Not Visible

Ensure text color contrasts with background:

```dsl
element "Container" {
    background "#2d5a87"
    color "#ffffff"     // White text on dark blue
}
```

### Relationships Disappearing

Use lighter relationship colors:

```dsl
relationship "Relationship" {
    color "#aaaaaa"     // Light gray
    thickness 2         // Thicker lines
}
```
