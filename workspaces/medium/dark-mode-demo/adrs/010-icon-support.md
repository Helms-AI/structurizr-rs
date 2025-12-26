# ADR 010: Icon Support

## Status

Accepted

## Context

Users want to add icons to architecture elements to:

1. Quickly identify technologies (Docker, Kubernetes, AWS)
2. Distinguish element types visually
3. Make diagrams more engaging
4. Follow corporate branding guidelines

Options considered:

1. **URL-based icons** - Reference external images
2. **Embedded icons** - Base64 data URIs
3. **Icon library** - Bundled icon set
4. **Font icons** - Icon fonts like Font Awesome
5. **SVG sprites** - Shared SVG definitions

## Decision

We chose to support **URL-based icons with data URI fallback**, allowing both external URLs and embedded base64 images.

### Approach

Icons specified via `icon` property in element styles:

```dsl
styles {
    element "Database" {
        shape Cylinder
        icon "https://example.com/icons/postgresql.svg"
        iconPosition Top
    }

    element "Container" {
        icon "data:image/svg+xml;base64,PHN2ZyB..."
    }
}
```

### Icon Positions

Three positions supported:

- **Top**: Icon above text (default)
- **Bottom**: Icon below text
- **Left**: Icon to the left of text

## Consequences

### Positive

- **Flexibility**: Any image source
- **No bundling**: No icon library maintenance
- **Custom icons**: Corporate or project-specific
- **SVG support**: Scalable icons

### Negative

- **External dependency**: URLs may break
- **CORS issues**: Cross-origin restrictions
- **File size**: Data URIs increase DSL size
- **Caching**: No built-in caching

### Neutral

- Standard web image formats
- Similar to Structurizr Java

## Implementation Details

### Style Definition

Added icon properties to ElementStyle:

```rust
pub struct ElementStyle {
    pub tag: String,
    pub shape: Option<Shape>,
    pub icon: Option<String>,           // NEW
    pub icon_position: Option<IconPosition>, // NEW
    // ...
}

pub enum IconPosition {
    Top,    // Default
    Bottom,
    Left,
}
```

### Parser Support

Parse icon and iconPosition:

```rust
fn parse_element_style(&mut self) -> Result<ElementStyleNode> {
    let mut style = ElementStyleNode::default();

    loop {
        match self.current_identifier() {
            Some("icon") => {
                self.advance();
                style.icon = Some(self.expect_string()?);
            }
            Some("iconPosition") | Some("iconposition") => {
                self.advance();
                let pos = self.expect_identifier()?;
                style.icon_position = Some(pos);
            }
            // ... other properties
            _ => break,
        }
    }

    Ok(style)
}
```

### Style Resolution

Merge icon properties in resolver:

```rust
impl ResolvedElementStyle {
    pub fn apply(&mut self, style: &ElementStyle) {
        if let Some(icon) = &style.icon {
            self.icon = Some(icon.clone());
        }
        if let Some(pos) = style.icon_position {
            self.icon_position = pos;
        }
    }
}
```

### SVG Rendering

Render icons as SVG image elements:

```rust
fn render_icon(
    svg: &mut String,
    icon_url: &str,
    position: IconPosition,
    bounds: &Bounds,
    shape: Shape,
) {
    let icon_size = calculate_icon_size(bounds, shape);
    let (x, y) = calculate_icon_position(position, bounds, icon_size);

    svg.push_str(&format!(
        r#"<image x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}"
            href="{}" preserveAspectRatio="xMidYMid meet"/>"#,
        x, y, icon_size, icon_size, escape_xml(icon_url)
    ));
}

fn calculate_icon_size(bounds: &Bounds, shape: Shape) -> f64 {
    let max_size = 48.0;
    let relative_size = bounds.width * 0.3;
    max_size.min(relative_size)
}

fn calculate_icon_position(
    position: IconPosition,
    bounds: &Bounds,
    icon_size: f64,
) -> (f64, f64) {
    match position {
        IconPosition::Top => (
            bounds.x + (bounds.width - icon_size) / 2.0,
            bounds.y + 10.0,
        ),
        IconPosition::Bottom => (
            bounds.x + (bounds.width - icon_size) / 2.0,
            bounds.y + bounds.height - icon_size - 10.0,
        ),
        IconPosition::Left => (
            bounds.x + 10.0,
            bounds.y + (bounds.height - icon_size) / 2.0,
        ),
    }
}
```

### Text Adjustment

Adjust text position when icon is present:

```rust
fn calculate_text_position(
    bounds: &Bounds,
    style: &ResolvedElementStyle,
) -> (f64, f64) {
    let mut y = bounds.center_y();

    if style.icon.is_some() {
        match style.icon_position {
            IconPosition::Top => y += 25.0,    // Push text down
            IconPosition::Bottom => y -= 25.0, // Push text up
            IconPosition::Left => {}           // No adjustment
        }
    }

    (bounds.center_x(), y)
}
```

## DSL Syntax

### External URL

```dsl
element "Database" {
    shape Cylinder
    icon "https://cdn.example.com/icons/postgresql.svg"
    iconPosition Top
}
```

### Data URI

```dsl
element "Container" {
    icon "data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHZpZXdCb3g9IjAgMCAyNCAyNCI+PC9zdmc+"
}
```

### Position Options

```dsl
element "Service" {
    icon "https://example.com/icon.svg"
    iconPosition Top     // Icon above text (default)
}

element "Database" {
    icon "https://example.com/db.svg"
    iconPosition Left    // Icon to the left
}

element "Queue" {
    icon "https://example.com/queue.svg"
    iconPosition Bottom  // Icon below text
}
```

## Icon Sources

### Recommended Icon Libraries

| Source | Format | License |
|--------|--------|---------|
| Simple Icons | SVG | CC0 |
| Devicons | SVG | MIT |
| AWS Icons | SVG | AWS |
| Azure Icons | SVG | Microsoft |
| GCP Icons | SVG | Google |

### Converting to Data URI

```bash
# Convert SVG to base64 data URI
base64 -i icon.svg | tr -d '\n' | \
  sed 's/^/data:image\/svg+xml;base64,/'
```

## Best Practices

### 1. Use SVG Icons

SVG scales without pixelation:

```dsl
icon "https://example.com/icon.svg"  // Preferred
icon "https://example.com/icon.png"  // Avoid if possible
```

### 2. Consider Dark/Light Mode

Choose icons visible on your background:

```dsl
// Dark mode - use light icons
element "Container" {
    background "#2d5a87"
    icon "https://example.com/icon-white.svg"
}
```

### 3. Consistent Sizing

Icons auto-scale, but keep source icons consistent (~48x48 minimum).

### 4. Cache External Icons

For production, consider:
- Self-hosting icons
- Using data URIs for critical icons
- CDN with long cache times

## Alternatives Considered

### Bundled Icon Library

**Pros**: No external dependencies, consistent
**Cons**: Maintenance burden, limited selection

### Font Icons

**Pros**: Easy styling, small size
**Cons**: Limited to font colors, alignment issues

### SVG Sprites

**Pros**: Single request, efficient
**Cons**: Complex setup, all-or-nothing loading

## References

- [SVG Image Element](https://developer.mozilla.org/en-US/docs/Web/SVG/Element/image)
- [Data URIs](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/Data_URLs)
- [Simple Icons](https://simpleicons.org/)
