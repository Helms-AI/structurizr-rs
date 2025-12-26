# structurizr-render Crate

The `structurizr-render` crate handles SVG diagram generation and automatic layout of architecture elements. It transforms the abstract model into visual representations.

## Module Overview

```
structurizr-render/
├── src/
│   ├── lib.rs              # Public API
│   ├── svg.rs              # SVG generation
│   ├── layout.rs           # Grid-based layout
│   ├── sugiyama/           # Hierarchical layout algorithm
│   │   ├── mod.rs
│   │   └── positioning.rs
│   ├── style_resolver.rs   # Style computation
│   └── shapes.rs           # Shape rendering
```

## SVG Generation

### Main Renderer

```rust
pub struct SvgRenderer {
    workspace: Workspace,
    config: RenderConfig,
}

impl SvgRenderer {
    pub fn new(workspace: Workspace) -> Self {
        Self {
            workspace,
            config: RenderConfig::default(),
        }
    }

    pub fn with_config(mut self, config: RenderConfig) -> Self {
        self.config = config;
        self
    }

    pub fn render_view(&self, view_key: &str) -> Result<String> {
        let view = self.find_view(view_key)?;
        let layout = self.compute_layout(&view)?;
        self.generate_svg(&view, &layout)
    }
}
```

### Render Configuration

```rust
pub struct RenderConfig {
    pub background_color: String,
    pub default_width: u32,
    pub default_height: u32,
    pub padding: u32,
    pub element_spacing: u32,
}

impl RenderConfig {
    pub fn dark_mode() -> Self {
        Self {
            background_color: "#1a1a1a".to_string(),
            ..Default::default()
        }
    }

    pub fn is_dark_mode(&self) -> bool {
        self.background_color.starts_with("#0") ||
        self.background_color.starts_with("#1") ||
        self.background_color.starts_with("#2")
    }
}
```

## Layout Algorithms

### Grid Layout

The default layout arranges elements in a grid pattern:

```rust
pub struct GridLayout {
    elements: Vec<ElementId>,
    spacing: f64,
    columns: usize,
}

impl GridLayout {
    pub fn compute(&self) -> HashMap<ElementId, Position> {
        let mut positions = HashMap::new();

        for (index, element_id) in self.elements.iter().enumerate() {
            let row = index / self.columns;
            let col = index % self.columns;

            positions.insert(*element_id, Position {
                x: col as f64 * self.spacing,
                y: row as f64 * self.spacing,
            });
        }

        positions
    }
}
```

### Sugiyama Algorithm

For hierarchical layouts, the crate implements the Sugiyama algorithm:

1. **Cycle Removal**: Temporarily reverse edges to make the graph acyclic
2. **Layer Assignment**: Assign nodes to horizontal layers
3. **Crossing Reduction**: Minimize edge crossings within layers
4. **Coordinate Assignment**: Compute final x/y coordinates

```rust
pub struct SugiyamaLayout {
    graph: DirectedGraph,
    direction: LayoutDirection,
}

impl SugiyamaLayout {
    pub fn compute(&self) -> LayoutResult {
        let acyclic = self.remove_cycles();
        let layers = self.assign_layers(&acyclic);
        let ordered = self.reduce_crossings(&layers);
        let positions = self.assign_coordinates(&ordered);

        LayoutResult { positions }
    }
}

pub enum LayoutDirection {
    TopToBottom,
    BottomToTop,
    LeftToRight,
    RightToLeft,
}
```

## Style Resolution

The style resolver computes the final visual properties for each element:

```rust
pub struct StyleResolver {
    styles: Styles,
}

impl StyleResolver {
    pub fn resolve_element(&self, element: &dyn Element) -> ResolvedElementStyle {
        let mut resolved = ResolvedElementStyle::default();

        // Apply base styles
        resolved.apply(&self.styles.base_element_style);

        // Apply tag-specific styles
        for tag in element.tags() {
            if let Some(style) = self.styles.find_element_style(tag) {
                resolved.apply(style);
            }
        }

        resolved
    }
}

pub struct ResolvedElementStyle {
    pub shape: Shape,
    pub background: String,
    pub color: String,
    pub stroke: String,
    pub stroke_width: u32,
    pub font_size: u32,
    pub width: u32,
    pub height: u32,
    pub icon: Option<String>,
    pub icon_position: IconPosition,
}
```

## Shape Rendering

Each shape type has specific rendering logic:

```rust
pub fn render_shape(
    shape: Shape,
    bounds: &Bounds,
    style: &ResolvedElementStyle,
) -> String {
    match shape {
        Shape::Box => render_box(bounds, style),
        Shape::RoundedBox => render_rounded_box(bounds, style),
        Shape::Circle => render_circle(bounds, style),
        Shape::Cylinder => render_cylinder(bounds, style),
        Shape::Person => render_person(bounds, style),
        Shape::Hexagon => render_hexagon(bounds, style),
        // ... other shapes
    }
}

fn render_box(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    format!(
        r#"<rect x="{}" y="{}" width="{}" height="{}"
            fill="{}" stroke="{}" stroke-width="{}"/>"#,
        bounds.x, bounds.y, bounds.width, bounds.height,
        style.background, style.stroke, style.stroke_width
    )
}
```

### Person Shape

```rust
fn render_person(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let cx = bounds.x + bounds.width / 2.0;
    let head_radius = bounds.width * 0.15;
    let head_cy = bounds.y + head_radius + 10.0;

    format!(
        r#"<circle cx="{}" cy="{}" r="{}" fill="{}"/>
           <path d="M {} {} ..." fill="{}"/>"#,
        cx, head_cy, head_radius, style.background,
        // body path coordinates...
        style.background
    )
}
```

## Icon Rendering

Icons can be embedded in elements:

```rust
fn render_icon(
    svg: &mut String,
    icon_url: &str,
    position: IconPosition,
    bounds: &Bounds,
) {
    let icon_size = 48.0_f64.min(bounds.width * 0.3);
    let (x, y) = match position {
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
    };

    svg.push_str(&format!(
        r#"<image x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}"
            href="{}" preserveAspectRatio="xMidYMid meet"/>"#,
        x, y, icon_size, icon_size, escape_xml(icon_url)
    ));
}
```

## Relationship Rendering

```rust
fn render_relationship(
    source: &Position,
    target: &Position,
    style: &ResolvedRelationshipStyle,
) -> String {
    let path = compute_path(source, target, style.routing);

    format!(
        r#"<path d="{}" fill="none" stroke="{}"
            stroke-width="{}" marker-end="url(#arrowhead)"/>"#,
        path, style.color, style.thickness
    )
}
```

## Usage Example

```rust
use structurizr_render::{SvgRenderer, RenderConfig};

let workspace = parse_workspace(dsl)?;
let renderer = SvgRenderer::new(workspace)
    .with_config(RenderConfig::dark_mode());

// Render a specific view
let svg = renderer.render_view("Container")?;

// Save to file
std::fs::write("diagram.svg", svg)?;
```
