# Legend Feature for structurizr-render

This document describes the legend/key auto-generation feature for SVG diagram rendering.

## Overview

The legend feature automatically generates a visual key showing all unique element types and relationship styles present in a diagram. The legend appears at the bottom-left of the rendered SVG and includes:

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
┌─────────────────────────┐
│ Legend                  │
├─────────────────────────┤
│ 👤 Person               │
│ 📦 Software System      │
│ 📁 Container            │
│ 🔧 Component            │
│ ─── Relationship        │
│ ··· Async Relationship  │
└─────────────────────────┘
```

## Customization

The legend automatically detects:
- **Unique element types** in the current view
- **Unique relationship styles** based on color, line style (solid/dashed/dotted), and thickness
- **Custom tags** on relationships for labeling

## Implementation Details

### Element Detection

The legend scans all elements in the view and creates entries for each unique element type:
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

The legend scans all relationships and creates entries for each unique visual style:
- Line color
- Line thickness
- Line style (solid, dashed, dotted)

Each entry shows:
- A sample line with the actual styling
- A label derived from relationship tags or default names

### Performance

The legend generation has minimal performance impact:
- O(n) complexity for scanning elements and relationships
- Deduplication using HashSets for efficient unique detection
- SVG generation is string concatenation

## Examples

See the following examples for demonstrations:
- `examples/render_with_legend.rs` - Basic usage
- `examples/legend_example.dsl` - Sample DSL workspace

## Testing

Run the tests with:
```bash
cargo test --package structurizr-render test_render_with_legend
cargo test --package structurizr-render test_legend_toggle
```

## Future Enhancements

Potential improvements for future versions:
- Configurable legend position (top-left, top-right, bottom-right)
- Custom legend title
- Legend size customization
- Option to hide specific element types or relationships from legend
- Support for custom colors and styling in legend entries
