# ADR 004: SVG Rendering Approach

## Status

Accepted

## Context

We need to render architecture diagrams in a visual format. Several rendering approaches were considered:

1. **Direct SVG generation** - Write SVG XML directly
2. **Canvas rendering** - Use HTML Canvas API
3. **External tool** (Graphviz, D3.js) - Delegate to existing tools
4. **PDF generation** - Direct PDF output
5. **Image library** (image-rs) - Bitmap generation

## Decision

We chose **direct SVG generation** for diagram output.

### Approach

Generate SVG XML strings directly in Rust:

```rust
pub fn render_element(element: &Element, style: &ResolvedStyle) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}"
            fill="{}" stroke="{}" stroke-width="{}"/>
           <text x="{}" y="{}" fill="{}">{}</text>"#,
        // ... coordinates and styles
    )
}
```

### Components

| Component | Responsibility |
|-----------|----------------|
| `svg.rs` | SVG document structure |
| `shapes.rs` | Shape-specific rendering |
| `layout.rs` | Element positioning |
| `style_resolver.rs` | Style computation |

## Consequences

### Positive

- **No dependencies**: Pure string generation
- **Full control**: Exact SVG output
- **Portable**: SVG works everywhere
- **Scalable**: Vector graphics at any size
- **Web-ready**: Direct embedding in HTML

### Negative

- **Manual work**: Must implement all shapes
- **Browser quirks**: SVG rendering varies slightly
- **Limited effects**: Some effects need external tools

### Neutral

- Standard SVG 1.1 output
- Compatible with Inkscape, browsers, etc.

## Implementation Details

### SVG Structure

```xml
<svg xmlns="http://www.w3.org/2000/svg"
     width="1200" height="800"
     viewBox="0 0 1200 800">
  <defs>
    <marker id="arrowhead">...</marker>
  </defs>
  <rect class="background" fill="#ffffff"/>
  <g class="elements">
    <!-- Element shapes -->
  </g>
  <g class="relationships">
    <!-- Connection lines -->
  </g>
  <g class="labels">
    <!-- Text labels -->
  </g>
</svg>
```

### Shape Rendering

Each C4 shape has specific SVG output:

```rust
pub enum Shape {
    Box,        // <rect>
    RoundedBox, // <rect rx="10">
    Circle,     // <circle>
    Cylinder,   // <path> (complex shape)
    Person,     // <circle> + <path>
    Hexagon,    // <polygon>
}

fn render_cylinder(bounds: &Bounds, style: &Style) -> String {
    // Draw top ellipse, body, bottom ellipse
    format!(r#"
        <ellipse cx="{}" cy="{}" rx="{}" ry="{}"/>
        <rect x="{}" y="{}" width="{}" height="{}"/>
        <ellipse cx="{}" cy="{}" rx="{}" ry="{}"/>
    "#, /* coordinates */)
}
```

### Text Handling

Multi-line text with proper wrapping:

```rust
fn render_text(text: &str, bounds: &Bounds, style: &Style) -> String {
    let lines = wrap_text(text, bounds.width);
    let line_height = style.font_size * 1.2;

    lines.iter().enumerate().map(|(i, line)| {
        format!(r#"<tspan x="{}" dy="{}">{}</tspan>"#,
            bounds.center_x(), line_height, escape_xml(line))
    }).collect()
}
```

### Relationship Lines

Paths between elements with arrow markers:

```rust
fn render_relationship(source: &Bounds, target: &Bounds) -> String {
    let (x1, y1) = calculate_edge_point(source, target);
    let (x2, y2) = calculate_edge_point(target, source);

    format!(r#"<path d="M{},{} L{},{}"
        stroke="#707070" stroke-width="2"
        marker-end="url(#arrowhead)"/>"#,
        x1, y1, x2, y2)
}
```

## Alternatives Considered

### Graphviz

**Pros**: Mature layout algorithms
**Cons**: External binary dependency, less styling control

### D3.js

**Pros**: Rich visualization library
**Cons**: Requires browser/Node.js environment

### Canvas API

**Pros**: Fast rendering
**Cons**: Not vector, harder to export

### PDF Direct

**Pros**: Print-ready output
**Cons**: Complex implementation, less flexible

## References

- [SVG 1.1 Specification](https://www.w3.org/TR/SVG11/)
- [MDN SVG Tutorial](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial)
