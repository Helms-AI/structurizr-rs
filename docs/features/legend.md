# Diagram Legend Feature

The legend feature automatically generates a visual key showing all unique element types and relationship styles present in a diagram.

## Overview

The legend appears at the bottom-left of rendered SVG diagrams and includes:

- **Element types**: Small icons showing the shape and color for each element type (Person, Software System, Container, Component, etc.)
- **Relationship types**: Line samples showing the style, color, and pattern for each relationship type

## Usage

### Basic Usage

```rust
use structurizr_render::SvgRenderer;

// Create a renderer with legend enabled
let renderer = SvgRenderer::default().with_legend(true);

// Render a view
let svg = renderer.render_system_landscape(&workspace, &view)?;
```

### Builder Pattern

```rust
let renderer = SvgRenderer::new(2000, 1500)
    .with_legend(true);
```

### Runtime Toggle

```rust
let mut renderer = SvgRenderer::default();

// Enable legend
renderer.enable_legend();

// Disable legend
renderer.disable_legend();
```

## Legend Layout

The legend box is positioned at the bottom-left corner of the diagram with:
- White background
- Light gray border with rounded corners
- Title "Legend" in bold
- Separator line below title
- Element type entries (with small shape icons)
- Relationship type entries (with line samples)

### Example Layout

```
+-------------------------+
| Legend                  |
+-------------------------+
| [person icon] Person    |
| [box icon] Software System |
| [box icon] Container    |
| [box icon] Component    |
| --- Relationship        |
| ... Async Relationship  |
+-------------------------+
```

## What the Legend Shows

### Element Detection

The legend automatically detects and displays unique element types in the current view:

- Person
- Software System
- Container
- Component
- External System
- Deployment Node
- Infrastructure Node

Each entry shows:
- A small icon (20x20 pixels) rendered with the actual shape and colors
- The element type label

### Relationship Detection

The legend shows unique relationship visual styles based on:

- Line color
- Line thickness
- Line style (solid, dashed, dotted)

Each entry shows:
- A sample line with the actual styling
- A label derived from relationship tags or default names

## Web Server Usage

When using the web server, legends can be enabled in the rendered SVG views:

```bash
# Start the server
cargo run -- serve --workspace workspace.dsl

# Access views with legends (enabled by default in recent versions)
http://localhost:8080/view/SystemLandscape
```

## Testing

Run the legend-specific tests:

```bash
cargo test --package structurizr-render test_render_with_legend
cargo test --package structurizr-render test_legend_toggle
```

## See Also

- [Legend Implementation Details](../development/legend-impl.md)
- [Animation Feature](animation.md)
- [Presentation Mode](presentation.md)
