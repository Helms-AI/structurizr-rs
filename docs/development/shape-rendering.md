# Shape Rendering Implementation

This document details the shape rendering system in structurizr-rs, which provides 14 different shape types for C4 model elements, from basic geometric shapes to specialized icons.

## Table of Contents

1. [Overview](#overview)
2. [Shape Types](#shape-types)
3. [Rendering Architecture](#rendering-architecture)
4. [Individual Shape Implementations](#individual-shape-implementations)
5. [SVG Generation Patterns](#svg-generation-patterns)
6. [Shape Styling](#shape-styling)
7. [Performance Considerations](#performance-considerations)
8. [Extension Guide](#extension-guide)

## Overview

The shape rendering system in `crates/structurizr-render/src/shapes.rs` provides individual rendering functions for each shape type supported in C4 diagrams. Each shape is rendered as SVG elements with consistent styling and positioning.

### Design Principles

1. **Modular Functions**: Each shape has its own rendering function
2. **Consistent API**: All shapes accept bounds and style parameters
3. **Pure SVG**: Generate clean, standard SVG without dependencies
4. **Style Independence**: Shapes accept style attributes, don't determine them
5. **Precise Geometry**: Mathematically correct shape construction

## Shape Types

### Available Shapes

```rust
pub enum Shape {
    // Basic geometric shapes
    Box,                    // Rectangle
    RoundedBox,            // Rectangle with rounded corners
    Circle,                // Perfect circle
    Ellipse,               // Oval shape
    Hexagon,               // Six-sided polygon

    // Specialized shapes
    Cylinder,              // Database representation
    Component,             // UML component notation
    Pipe,                  // Horizontal cylinder

    // Icon shapes
    Person,                // Stick figure
    Robot,                 // Android-style robot
    Folder,                // File folder
    WebBrowser,            // Browser window
    MobileDevicePortrait,  // Phone vertical
    MobileDeviceLandscape, // Phone horizontal
}
```

## Rendering Architecture

### Common Pattern

All shape rendering follows this pattern:

```rust
pub fn render_shape(
    bounds: &Bounds,
    style_attrs: &str,
) -> String {
    // Calculate shape geometry from bounds
    let center_x = bounds.x + bounds.width / 2.0;
    let center_y = bounds.y + bounds.height / 2.0;

    // Generate SVG elements
    format!(
        r##"<{element} {attributes} {style_attrs}/>"##,
        element = "rect|circle|path|polygon",
        attributes = geometry_attributes,
        style_attrs = style_attrs
    )
}
```

### Bounds Structure

```rust
pub struct Bounds {
    pub x: f32,      // Top-left X coordinate
    pub y: f32,      // Top-left Y coordinate
    pub width: f32,  // Element width
    pub height: f32, // Element height
}

impl Bounds {
    pub fn center(&self) -> (f32, f32) {
        (
            self.x + self.width / 2.0,
            self.y + self.height / 2.0
        )
    }
}
```

## Individual Shape Implementations

### Box (Rectangle)

Simple rectangle with no special features:

```rust
pub fn render_box(bounds: &Bounds, style_attrs: &str) -> String {
    format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height, style_attrs
    )
}
```

### RoundedBox

Rectangle with 5px corner radius:

```rust
pub fn render_rounded_box(bounds: &Bounds, style_attrs: &str) -> String {
    format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" rx="5" ry="5" {}/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height, style_attrs
    )
}
```

### Circle

Perfect circle fitted within bounds:

```rust
pub fn render_circle(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let radius = bounds.width.min(bounds.height) / 2.0;

    format!(
        r##"<circle cx="{}" cy="{}" r="{}" {}/>"##,
        cx, cy, radius, style_attrs
    )
}
```

### Ellipse

Oval shape using full bounds:

```rust
pub fn render_ellipse(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let rx = bounds.width / 2.0;
    let ry = bounds.height / 2.0;

    format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" {}/>"##,
        cx, cy, rx, ry, style_attrs
    )
}
```

### Hexagon

Six-sided polygon with pointy top/bottom:

```rust
pub fn render_hexagon(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let width = bounds.width;
    let height = bounds.height;

    // Calculate hexagon vertices
    let points = vec![
        (cx, bounds.y),                           // Top
        (cx + width/2.0, cy - height/4.0),       // Top-right
        (cx + width/2.0, cy + height/4.0),       // Bottom-right
        (cx, bounds.y + height),                  // Bottom
        (cx - width/2.0, cy + height/4.0),       // Bottom-left
        (cx - width/2.0, cy - height/4.0),       // Top-left
    ];

    let points_str = points
        .iter()
        .map(|(x, y)| format!("{},{}", x, y))
        .collect::<Vec<_>>()
        .join(" ");

    format!(
        r##"<polygon points="{}" {}/>"##,
        points_str, style_attrs
    )
}
```

### Cylinder

Database shape with elliptical top:

```rust
pub fn render_cylinder(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let rx = bounds.width / 2.0;
    let ry = bounds.height * 0.15; // 15% of height for ellipse

    let mut svg = String::new();

    // Top ellipse
    svg.push_str(&format!(
        r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" {}/>"##,
        cx, bounds.y + ry, rx, ry, style_attrs
    ));

    // Body rectangle
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x, bounds.y + ry, bounds.width, bounds.height - ry, style_attrs
    ));

    // Bottom arc
    svg.push_str(&format!(
        r##"<path d="M {} {} A {} {} 0 0 1 {} {}" fill="none" {}/>"##,
        bounds.x, bounds.y + bounds.height - ry,
        rx, ry,
        bounds.x + bounds.width, bounds.y + bounds.height - ry,
        style_attrs
    ));

    svg
}
```

### Component

UML component notation with tabs:

```rust
pub fn render_component(bounds: &Bounds, style_attrs: &str) -> String {
    let tab_width = 20.0;
    let tab_height = 10.0;
    let tab_spacing = 5.0;

    let mut svg = String::new();

    // Main rectangle
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height, style_attrs
    ));

    // Left tabs (component ports)
    let tab1_y = bounds.y + bounds.height * 0.3;
    let tab2_y = bounds.y + bounds.height * 0.6;

    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x - tab_width/2.0, tab1_y, tab_width, tab_height, style_attrs
    ));

    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x - tab_width/2.0, tab2_y, tab_width, tab_height, style_attrs
    ));

    svg
}
```

### Person

Stick figure representation:

```rust
pub fn render_person(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let scale = bounds.height / 250.0; // Normalize to standard height

    let mut svg = String::new();

    // Head (circle)
    let head_radius = 30.0 * scale;
    svg.push_str(&format!(
        r##"<circle cx="{}" cy="{}" r="{}" {}/>"##,
        cx, bounds.y + head_radius, head_radius, style_attrs
    ));

    // Body (line)
    let body_start_y = bounds.y + head_radius * 2.0;
    let body_end_y = body_start_y + 60.0 * scale;
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke-width="{}" {}/>"##,
        cx, body_start_y, cx, body_end_y, 3.0 * scale, style_attrs
    ));

    // Arms (lines)
    let arm_y = body_start_y + 20.0 * scale;
    let arm_span = 50.0 * scale;
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke-width="{}" {}/>"##,
        cx - arm_span, arm_y, cx + arm_span, arm_y, 3.0 * scale, style_attrs
    ));

    // Legs (lines)
    let leg_span = 30.0 * scale;
    let leg_end_y = body_end_y + 50.0 * scale;
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke-width="{}" {}/>"##,
        cx, body_end_y, cx - leg_span, leg_end_y, 3.0 * scale, style_attrs
    ));
    svg.push_str(&format!(
        r##"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke-width="{}" {}/>"##,
        cx, body_end_y, cx + leg_span, leg_end_y, 3.0 * scale, style_attrs
    ));

    svg
}
```

### Robot

Android-style robot icon:

```rust
pub fn render_robot(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let scale = bounds.height / 250.0;

    let mut svg = String::new();

    // Head (rounded rectangle)
    let head_width = 80.0 * scale;
    let head_height = 60.0 * scale;
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" rx="10" ry="10" {}/>"##,
        cx - head_width/2.0, bounds.y, head_width, head_height, style_attrs
    ));

    // Eyes (circles)
    let eye_radius = 5.0 * scale;
    let eye_y = bounds.y + head_height * 0.4;
    let eye_spacing = 20.0 * scale;

    svg.push_str(&format!(
        r##"<circle cx="{}" cy="{}" r="{}" fill="white"/>"##,
        cx - eye_spacing, eye_y, eye_radius
    ));
    svg.push_str(&format!(
        r##"<circle cx="{}" cy="{}" r="{}" fill="white"/>"##,
        cx + eye_spacing, eye_y, eye_radius
    ));

    // Body (rectangle)
    let body_width = 100.0 * scale;
    let body_height = 80.0 * scale;
    let body_y = bounds.y + head_height + 10.0 * scale;

    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        cx - body_width/2.0, body_y, body_width, body_height, style_attrs
    ));

    // Arms and legs (rectangles)
    // ... simplified for brevity

    svg
}
```

### WebBrowser

Browser window with address bar:

```rust
pub fn render_web_browser(bounds: &Bounds, style_attrs: &str) -> String {
    let mut svg = String::new();

    // Window frame
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" {}/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height, style_attrs
    ));

    // Address bar
    let bar_height = 30.0;
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" fill="#f0f0f0" stroke="#cccccc"/>"##,
        bounds.x, bounds.y, bounds.width, bar_height
    ));

    // Navigation buttons (circles)
    let button_y = bounds.y + bar_height / 2.0;
    let button_radius = 8.0;
    let button_x_start = bounds.x + 20.0;

    for i in 0..3 {
        let button_x = button_x_start + (i as f32) * 25.0;
        let color = match i {
            0 => "#ff5f57", // Red
            1 => "#ffbd2e", // Yellow
            2 => "#28ca42", // Green
            _ => "#cccccc",
        };

        svg.push_str(&format!(
            r##"<circle cx="{}" cy="{}" r="{}" fill="{}"/>"##,
            button_x, button_y, button_radius, color
        ));
    }

    // URL bar
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="20" rx="3" fill="white" stroke="#cccccc"/>"##,
        bounds.x + 100.0, bounds.y + 5.0, bounds.width - 110.0
    ));

    svg
}
```

### MobileDevice

Phone shapes in portrait and landscape:

```rust
pub fn render_mobile_device_portrait(bounds: &Bounds, style_attrs: &str) -> String {
    let mut svg = String::new();

    // Device frame
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" rx="15" ry="15" {}/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height, style_attrs
    ));

    // Screen
    let screen_margin = 10.0;
    svg.push_str(&format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}" rx="5" fill="#333333"/>"##,
        bounds.x + screen_margin,
        bounds.y + screen_margin * 2.0,
        bounds.width - screen_margin * 2.0,
        bounds.height - screen_margin * 4.0
    ));

    // Home button
    let (cx, _) = bounds.center();
    let button_y = bounds.y + bounds.height - screen_margin * 1.5;
    svg.push_str(&format!(
        r##"<circle cx="{}" cy="{}" r="8" fill="#666666"/>"##,
        cx, button_y
    ));

    svg
}

pub fn render_mobile_device_landscape(bounds: &Bounds, style_attrs: &str) -> String {
    // Similar to portrait but with rotated dimensions
    // ... implementation
}
```

## SVG Generation Patterns

### String Building

Use raw strings with double-hash delimiters for special characters:

```rust
// Good: Handles # in hex colors
format!(r##"fill="#707070""##)

// Bad: Will fail with # in content
format!(r#"fill="#707070""#)
```

### Coordinate Precision

Round coordinates to avoid excessive precision:

```rust
fn format_coord(value: f32) -> String {
    format!("{:.1}", value)
}

// Usage
format!(
    r##"<circle cx="{}" cy="{}" r="{}"/>"##,
    format_coord(cx),
    format_coord(cy),
    format_coord(radius)
)
```

### Path Construction

For complex shapes, use SVG path notation:

```rust
pub fn render_complex_shape(bounds: &Bounds, style_attrs: &str) -> String {
    let mut path = String::from("M"); // Move to start

    // Add path commands
    path.push_str(&format!(" {} {}", start_x, start_y));
    path.push_str(&format!(" L {} {}", x1, y1)); // Line to
    path.push_str(&format!(" Q {} {} {} {}", cx, cy, x2, y2)); // Quadratic curve
    path.push_str(" Z"); // Close path

    format!(r##"<path d="{}" {}/>"##, path, style_attrs)
}
```

## Shape Styling

### Style Attributes

All shapes accept pre-formatted style attributes:

```rust
let style_attrs = format!(
    r##"fill="{}" stroke="{}" stroke-width="{}" opacity="{}""##,
    background_color, stroke_color, stroke_width, opacity
);

let shape_svg = render_box(&bounds, &style_attrs);
```

### Special Styling Cases

Some shapes need special handling:

```rust
// Person shape needs stroke for lines even with fill
pub fn render_person_with_fill(bounds: &Bounds, fill: &str, stroke: &str) -> String {
    // Head uses fill
    let head_svg = format!(r##"<circle ... fill="{}"/>"##, fill);

    // Body lines use stroke
    let body_svg = format!(r##"<line ... stroke="{}"/>"##, stroke);

    format!("{}{}", head_svg, body_svg)
}
```

## Performance Considerations

### String Allocation

Pre-allocate string capacity for complex shapes:

```rust
pub fn render_complex_shape(bounds: &Bounds) -> String {
    let mut svg = String::with_capacity(512); // Estimate size

    svg.push_str("<g>");
    // Add multiple elements
    svg.push_str("</g>");

    svg
}
```

### Shape Caching

Cache commonly used shapes:

```rust
lazy_static! {
    static ref PERSON_TEMPLATE: String = {
        // Generate template once
        render_person_template()
    };
}

pub fn render_person_cached(bounds: &Bounds, style: &str) -> String {
    // Apply transforms to cached template
    PERSON_TEMPLATE
        .replace("{x}", &bounds.x.to_string())
        .replace("{y}", &bounds.y.to_string())
        .replace("{style}", style)
}
```

## Extension Guide

### Adding a New Shape

1. **Define the shape enum variant**:

```rust
// In core/src/style.rs
pub enum Shape {
    // ... existing shapes
    NewShape,
}
```

2. **Implement the render function**:

```rust
// In render/src/shapes.rs
pub fn render_new_shape(bounds: &Bounds, style_attrs: &str) -> String {
    // Calculate geometry
    let (cx, cy) = bounds.center();

    // Generate SVG
    format!(
        r##"<path d="..." {}/>"##,
        style_attrs
    )
}
```

3. **Add to the shape dispatcher**:

```rust
// In render/src/svg.rs
match resolved_style.shape {
    Shape::NewShape => shapes::render_new_shape(&bounds, &style_attrs),
    // ... other shapes
}
```

4. **Add DSL support** (optional):

```rust
// In dsl/src/lexer.rs
"NewShape" => TokenKind::NewShape,

// In dsl/src/parser.rs
TokenKind::NewShape => Shape::NewShape,
```

### Shape Guidelines

When creating new shapes:

1. **Use bounds consistently**: All shapes should respect the provided bounds
2. **Accept style attributes**: Don't hardcode colors or styles
3. **Consider scaling**: Shapes should scale proportionally with bounds
4. **Maintain aspect ratio**: For icon shapes, maintain recognizable proportions
5. **Test at different sizes**: Ensure shapes look good from 50x50 to 500x500

### Complex Shape Example

```rust
pub fn render_cloud(bounds: &Bounds, style_attrs: &str) -> String {
    let (cx, cy) = bounds.center();
    let width = bounds.width;
    let height = bounds.height;

    // Create cloud shape using overlapping circles
    let mut svg = String::new();
    svg.push_str("<g>");

    // Main body (3 overlapping ellipses)
    let positions = [
        (cx - width * 0.2, cy, width * 0.3, height * 0.4),
        (cx, cy - height * 0.1, width * 0.4, height * 0.5),
        (cx + width * 0.2, cy, width * 0.3, height * 0.4),
    ];

    for (x, y, w, h) in &positions {
        svg.push_str(&format!(
            r##"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" {}/>"##,
            x, y, w/2.0, h/2.0, style_attrs
        ));
    }

    svg.push_str("</g>");
    svg
}
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Style System](style-system.md) - How shapes receive styles
- [Coordinate Systems](coordinate-systems.md) - Positioning and bounds
- [Layout Algorithms](layout-algorithms.md) - How bounds are determined