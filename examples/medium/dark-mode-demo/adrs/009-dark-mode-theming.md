# ADR 009: Dark Mode Theming

## Status

Accepted

## Context

Users want to create diagrams with dark backgrounds for:

1. Presentations with dark themes
2. Documentation sites with dark mode
3. Reduced eye strain
4. Aesthetic preferences

Options considered:

1. **Global theme setting** - Single dark/light switch
2. **View-level background** - Per-view background color
3. **CSS-based theming** - External stylesheet
4. **Automatic detection** - System preference detection

## Decision

We chose **view-level background color** as the primary dark mode mechanism.

### Approach

Each view can specify its own background color:

```dsl
views {
    container system "DarkView" {
        include *
        background "#1a1a1a"
        autoLayout
    }

    container system "LightView" {
        include *
        background "#ffffff"
        autoLayout
    }
}
```

### Dark Mode Detection

The renderer automatically detects dark mode based on background luminance:

```rust
impl RenderConfig {
    pub fn is_dark_mode(&self) -> bool {
        self.background_color.starts_with("#0") ||
        self.background_color.starts_with("#1") ||
        self.background_color.starts_with("#2")
    }
}
```

## Consequences

### Positive

- **Flexibility**: Different views can have different themes
- **Simplicity**: Single property to set
- **Compatibility**: Works with existing DSL syntax
- **SVG native**: Background embedded in SVG

### Negative

- **Manual styling**: Colors must be adjusted manually
- **No inheritance**: Each view needs explicit background
- **Limited scope**: Only affects diagram background

### Neutral

- Similar to Structurizr Java approach
- Standard CSS color values

## Implementation Details

### View Properties

Added background to ViewProperties:

```rust
pub struct ViewProperties {
    pub title: Option<String>,
    pub description: Option<String>,
    pub background: Option<String>,  // NEW
    // ...
}
```

### Parser Update

Parse background in view properties:

```rust
fn parse_view_properties(&mut self) -> Result<ViewPropertiesNode> {
    let mut props = ViewPropertiesNode::default();

    loop {
        match self.current_kind() {
            Some(TokenKind::Identifier(id)) if id == "background" => {
                self.advance();
                props.background = Some(self.expect_string()?);
            }
            // ... other properties
            _ => break,
        }
    }

    Ok(props)
}
```

### SVG Rendering

Background applied to SVG:

```rust
fn render_background(config: &RenderConfig, width: f64, height: f64) -> String {
    format!(
        r#"<rect x="0" y="0" width="{}" height="{}" fill="{}"/>"#,
        width, height, config.background_color
    )
}
```

### Adaptive Colors

Group boundaries adjust for dark mode:

```rust
fn render_group_boundary(bounds: &Bounds, config: &RenderConfig) -> String {
    let (fill, stroke) = if config.is_dark_mode() {
        ("#333333", "#666666")
    } else {
        ("#f0f0f0", "#999999")
    };

    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}"
            fill="{}" stroke="{}" stroke-dasharray="5,5"/>"#,
        bounds.x, bounds.y, bounds.width, bounds.height, fill, stroke
    )
}
```

### Recommended Color Palettes

**Dark Mode Elements:**

```dsl
styles {
    element "Person" {
        background "#2d5a87"
        color "#ffffff"
        stroke "#4a90d9"
    }
    element "Container" {
        background "#4a8fd9"
        color "#ffffff"
        stroke "#6ab0e8"
    }
    relationship "Relationship" {
        color "#888888"
    }
}
```

**Light Mode Elements:**

```dsl
styles {
    element "Person" {
        background "#08427b"
        color "#ffffff"
    }
    element "Container" {
        background "#438dd5"
        color "#ffffff"
    }
    relationship "Relationship" {
        color "#707070"
    }
}
```

## DSL Syntax

```dsl
views {
    // Dark mode view
    systemContext system "DarkContext" "Context (Dark)" {
        include *
        background "#1a1a1a"
        autoLayout
    }

    // Light mode view (default)
    systemContext system "LightContext" "Context (Light)" {
        include *
        autoLayout
    }
}
```

## Best Practices

### 1. Sufficient Contrast

Ensure text and borders are visible:

```dsl
element "Container" {
    background "#2d5a87"  // Medium blue
    color "#ffffff"       // White text
    stroke "#4a90d9"      // Lighter blue border
    strokeWidth 2         // Visible border
}
```

### 2. Consistent Theming

Apply dark styling to all elements in dark views:

```dsl
styles {
    // Base style for dark mode
    element "Element" {
        color "#ffffff"
        strokeWidth 1
    }
}
```

### 3. Test Both Modes

Create parallel views for testing:

```dsl
views {
    container system "ContainersDark" {
        include *
        background "#1a1a1a"
    }
    container system "ContainersLight" {
        include *
        background "#ffffff"
    }
}
```

## Alternatives Considered

### Global Theme Toggle

**Pros**: Single setting, automatic
**Cons**: No per-view control, complex state

### CSS Theming

**Pros**: Dynamic switching
**Cons**: External dependency, SVG limitations

### Automatic System Detection

**Pros**: Follows OS preference
**Cons**: Server-side rendering can't detect, inconsistent

## References

- [WCAG Contrast Guidelines](https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum.html)
- [Material Design Dark Theme](https://material.io/design/color/dark-theme.html)
