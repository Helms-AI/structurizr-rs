# Edge Routing Implementation

This document explains how relationships (edges) are routed and rendered between elements in structurizr-rs diagrams, including different routing strategies, arrow rendering, and label positioning.

## Table of Contents

1. [Overview](#overview)
2. [Routing Strategies](#routing-strategies)
3. [Line-Rectangle Intersection](#line-rectangle-intersection)
4. [Edge Distribution](#edge-distribution)
5. [Arrow Markers](#arrow-markers)
6. [Label Positioning](#label-positioning)
7. [Collision Detection](#collision-detection)
8. [Implementation Details](#implementation-details)

## Overview

The edge routing system in structurizr-rs handles the complex task of drawing relationship lines between elements. Located in `crates/structurizr-render/src/routing/` and integrated with `svg.rs`, it supports three routing strategies and sophisticated label placement.

### Architecture

```
Edge Routing System
├── Routing Strategy (Direct/Orthogonal/Curved)
├── Intersection Calculation
├── Port Distribution (for multiple edges)
├── Arrow Marker Generation
├── Label Collision Avoidance
└── Style Application
```

## Routing Strategies

### Direct Routing

Straight lines between element centers:

```rust
// crates/structurizr-render/src/routing/direct.rs
pub struct DirectRouter;

impl DirectRouter {
    pub fn route(source: &Bounds, target: &Bounds) -> EdgePath {
        let source_center = source.center();
        let target_center = target.center();

        EdgePath::Direct {
            start: Point {
                x: source_center.0,
                y: source_center.1,
            },
            end: Point {
                x: target_center.0,
                y: target_center.1,
            },
        }
    }
}
```

### Orthogonal Routing

Right-angle paths through channels:

```rust
// crates/structurizr-render/src/routing/orthogonal.rs
pub struct OrthogonalRouter {
    config: OrthogonalConfig,
}

pub struct OrthogonalConfig {
    pub direction: Direction,
    pub min_segment_length: f32,  // Default: 20px
    pub channel_spacing: f32,      // Default: 50px
}

impl OrthogonalRouter {
    pub fn route(
        &self,
        source: &Bounds,
        target: &Bounds,
        obstacles: &[Bounds],
    ) -> EdgePath {
        // Calculate waypoints for orthogonal path
        let waypoints = self.calculate_waypoints(source, target, obstacles);

        EdgePath::Orthogonal { waypoints }
    }

    fn calculate_waypoints(
        &self,
        source: &Bounds,
        target: &Bounds,
        obstacles: &[Bounds],
    ) -> Vec<Point> {
        let mut waypoints = Vec::new();

        // Start from source center
        let start = source.center();
        waypoints.push(Point::from(start));

        // Determine routing direction
        let dx = target.center().0 - source.center().0;
        let dy = target.center().1 - source.center().1;

        if dx.abs() > dy.abs() {
            // Horizontal routing dominant
            self.route_horizontal(&mut waypoints, source, target);
        } else {
            // Vertical routing dominant
            self.route_vertical(&mut waypoints, source, target);
        }

        // End at target center
        let end = target.center();
        waypoints.push(Point::from(end));

        // Smooth path to avoid obstacles
        self.avoid_obstacles(&mut waypoints, obstacles);

        waypoints
    }

    fn route_horizontal(
        &self,
        waypoints: &mut Vec<Point>,
        source: &Bounds,
        target: &Bounds,
    ) {
        let source_port = self.get_port(source, Side::Right);
        let target_port = self.get_port(target, Side::Left);

        // Exit source horizontally
        waypoints.push(source_port);

        // Create channel point
        let channel_x = (source_port.x + target_port.x) / 2.0;
        waypoints.push(Point {
            x: channel_x,
            y: source_port.y,
        });

        // Move to target level
        waypoints.push(Point {
            x: channel_x,
            y: target_port.y,
        });

        // Enter target
        waypoints.push(target_port);
    }
}
```

### Curved Routing

Bezier curves for smooth paths:

```rust
// crates/structurizr-render/src/routing/curved.rs
pub struct CurvedRouter {
    curve_factor: f32,  // Default: 0.3
}

impl CurvedRouter {
    pub fn route(source: &Bounds, target: &Bounds) -> EdgePath {
        let start = source.center();
        let end = target.center();

        // Calculate control points for quadratic Bezier
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        // Control point perpendicular to line
        let mid_x = (start.0 + end.0) / 2.0;
        let mid_y = (start.1 + end.1) / 2.0;

        // Offset perpendicular to create curve
        let offset = (dx.powi(2) + dy.powi(2)).sqrt() * 0.3;
        let angle = dy.atan2(dx) + std::f32::consts::PI / 2.0;

        let control = Point {
            x: mid_x + offset * angle.cos(),
            y: mid_y + offset * angle.sin(),
        };

        EdgePath::Curved {
            start: Point::from(start),
            control1: control,
            control2: None,  // Quadratic curve
            end: Point::from(end),
        }
    }

    pub fn route_cubic(source: &Bounds, target: &Bounds) -> EdgePath {
        let start = source.center();
        let end = target.center();

        // Calculate control points for cubic Bezier
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;

        let control1 = Point {
            x: start.0 + dx * 0.3,
            y: start.1,
        };

        let control2 = Point {
            x: end.0 - dx * 0.3,
            y: end.1,
        };

        EdgePath::Curved {
            start: Point::from(start),
            control1,
            control2: Some(control2),
            end: Point::from(end),
        }
    }
}
```

### EdgePath Enum

Common representation for all routing strategies:

```rust
pub enum EdgePath {
    Direct {
        start: Point,
        end: Point,
    },
    Orthogonal {
        waypoints: Vec<Point>,
    },
    Curved {
        start: Point,
        control1: Point,
        control2: Option<Point>,
        end: Point,
    },
}

impl EdgePath {
    pub fn to_svg_path(&self) -> String {
        match self {
            EdgePath::Direct { start, end } => {
                format!("M {} {} L {} {}", start.x, start.y, end.x, end.y)
            }
            EdgePath::Orthogonal { waypoints } => {
                let mut path = String::new();
                if let Some(first) = waypoints.first() {
                    path.push_str(&format!("M {} {}", first.x, first.y));
                    for point in waypoints.iter().skip(1) {
                        path.push_str(&format!(" L {} {}", point.x, point.y));
                    }
                }
                path
            }
            EdgePath::Curved { start, control1, control2, end } => {
                if let Some(c2) = control2 {
                    // Cubic Bezier
                    format!(
                        "M {} {} C {} {}, {} {}, {} {}",
                        start.x, start.y,
                        control1.x, control1.y,
                        c2.x, c2.y,
                        end.x, end.y
                    )
                } else {
                    // Quadratic Bezier
                    format!(
                        "M {} {} Q {} {}, {} {}",
                        start.x, start.y,
                        control1.x, control1.y,
                        end.x, end.y
                    )
                }
            }
        }
    }
}
```

## Line-Rectangle Intersection

Calculate where relationship lines intersect element boundaries:

```rust
// crates/structurizr-render/src/svg.rs (lines 309-457)
fn line_rect_intersection(
    line_start: (f32, f32),
    line_end: (f32, f32),
    rect: &Bounds,
) -> Option<(f32, f32)> {
    // Ray from start to end
    let dx = line_end.0 - line_start.0;
    let dy = line_end.1 - line_start.1;

    // Rectangle edges
    let edges = [
        // Top edge
        ((rect.x, rect.y), (rect.x + rect.width, rect.y)),
        // Right edge
        ((rect.x + rect.width, rect.y), (rect.x + rect.width, rect.y + rect.height)),
        // Bottom edge
        ((rect.x, rect.y + rect.height), (rect.x + rect.width, rect.y + rect.height)),
        // Left edge
        ((rect.x, rect.y), (rect.x, rect.y + rect.height)),
    ];

    let mut closest_intersection = None;
    let mut min_distance = f32::MAX;

    for (edge_start, edge_end) in &edges {
        if let Some(intersection) = line_line_intersection(
            line_start,
            line_end,
            *edge_start,
            *edge_end,
        ) {
            let distance = distance_squared(line_start, intersection);
            if distance < min_distance {
                min_distance = distance;
                closest_intersection = Some(intersection);
            }
        }
    }

    closest_intersection
}

fn line_line_intersection(
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    p4: (f32, f32),
) -> Option<(f32, f32)> {
    let x1 = p1.0;
    let y1 = p1.1;
    let x2 = p2.0;
    let y2 = p2.1;
    let x3 = p3.0;
    let y3 = p3.1;
    let x4 = p4.0;
    let y4 = p4.1;

    let denom = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);

    if denom.abs() < 0.001 {
        return None; // Lines are parallel
    }

    let t = ((x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4)) / denom;
    let u = -((x1 - x2) * (y1 - y3) - (y1 - y2) * (x1 - x3)) / denom;

    if t >= 0.0 && t <= 1.0 && u >= 0.0 && u <= 1.0 {
        Some((
            x1 + t * (x2 - x1),
            y1 + t * (y2 - y1),
        ))
    } else {
        None
    }
}
```

## Edge Distribution

Distribute multiple edges between same elements across different ports:

```rust
fn line_rect_intersection_with_offset(
    line_start: (f32, f32),
    line_end: (f32, f32),
    rect: &Bounds,
    edge_index: usize,
    edge_count: usize,
) -> (f32, f32) {
    // Calculate base intersection
    let base = line_rect_intersection(line_start, line_end, rect)
        .unwrap_or(rect.center());

    if edge_count <= 1 {
        return base;
    }

    // Determine which edge was hit
    let edge_side = determine_edge_side(base, rect);

    // Calculate offset along the edge
    let spacing = match edge_side {
        Side::Top | Side::Bottom => rect.width / (edge_count + 1) as f32,
        Side::Left | Side::Right => rect.height / (edge_count + 1) as f32,
    };

    let offset = spacing * (edge_index + 1) as f32;

    // Apply offset
    match edge_side {
        Side::Top | Side::Bottom => (rect.x + offset, base.1),
        Side::Left | Side::Right => (base.0, rect.y + offset),
    }
}

fn determine_edge_side(point: (f32, f32), rect: &Bounds) -> Side {
    let tolerance = 1.0;

    if (point.1 - rect.y).abs() < tolerance {
        Side::Top
    } else if (point.1 - (rect.y + rect.height)).abs() < tolerance {
        Side::Bottom
    } else if (point.0 - rect.x).abs() < tolerance {
        Side::Left
    } else {
        Side::Right
    }
}
```

## Arrow Markers

Dynamic arrow marker generation per color:

```rust
fn generate_arrow_markers(relationships: &[RenderEdge]) -> String {
    let mut markers = String::new();
    let mut colors = HashSet::new();

    // Collect unique colors
    for edge in relationships {
        colors.insert(edge.style.color.clone());
    }

    markers.push_str("<defs>");

    for color in colors {
        // Standard arrow
        markers.push_str(&format!(
            r##"
            <marker id="arrow-{}" markerWidth="10" markerHeight="10"
                    refX="9" refY="5" orient="auto">
                <polygon points="0,0 10,5 0,10" fill="{}"/>
            </marker>
            "##,
            color.trim_start_matches('#'),
            color
        ));

        // Bold arrow for outbound relationships
        markers.push_str(&format!(
            r##"
            <marker id="arrow-{}-bold" markerWidth="15" markerHeight="15"
                    refX="13.5" refY="7.5" orient="auto">
                <polygon points="0,0 15,7.5 0,15" fill="{}"/>
            </marker>
            "##,
            color.trim_start_matches('#'),
            color
        ));
    }

    markers.push_str("</defs>");
    markers
}
```

### Arrow Application

```rust
fn render_relationship_line(edge: &RenderEdge, is_outbound: bool) -> String {
    let path = edge.routing.to_svg_path();
    let color = &edge.style.color;
    let thickness = if is_outbound {
        edge.style.thickness * 2  // Bold for outbound
    } else {
        edge.style.thickness
    };

    let marker_id = if is_outbound {
        format!("arrow-{}-bold", color.trim_start_matches('#'))
    } else {
        format!("arrow-{}", color.trim_start_matches('#'))
    };

    format!(
        r##"<path d="{}" stroke="{}" stroke-width="{}"
             fill="none" marker-end="url(#{})"
             class="relationship-line {}"/>"##,
        path,
        color,
        thickness,
        marker_id,
        if is_outbound { "outbound" } else { "" }
    )
}
```

## Label Positioning

Sophisticated collision-aware label placement:

```rust
// crates/structurizr-render/src/svg.rs (lines 240-307)
fn find_non_overlapping_position(
    line: &EdgePath,
    label_text: &str,
    existing_bounds: &[TextBounds],
    font_size: i32,
) -> (f32, f32, String) {
    let label_width = estimate_text_width(label_text, font_size);
    let label_height = font_size as f32 * 1.2;

    // Try multiple positions along the line
    let positions = [0.5, 0.35, 0.65, 0.25, 0.75, 0.4, 0.6];

    for position in &positions {
        let (base_x, base_y) = line.point_at(*position);

        // Try multiple perpendicular offsets
        let perpendicular = line.perpendicular_at(*position);
        let offsets = [0.0, 15.0, -15.0, 30.0, -30.0];

        for offset_multiplier in &offsets {
            let x = base_x + perpendicular.0 * offset_multiplier;
            let y = base_y + perpendicular.1 * offset_multiplier;

            let bounds = TextBounds {
                x: x - label_width / 2.0,
                y: y - label_height / 2.0,
                width: label_width,
                height: label_height,
            };

            // Check for collisions
            if !has_collision(&bounds, existing_bounds) {
                return (x, y, "middle".to_string());
            }
        }
    }

    // Fallback to middle position
    let (x, y) = line.point_at(0.5);
    (x, y, "middle".to_string())
}

fn has_collision(bounds: &TextBounds, existing: &[TextBounds]) -> bool {
    for other in existing {
        if bounds_overlap(bounds, other) {
            return true;
        }
    }
    false
}

fn bounds_overlap(a: &TextBounds, b: &TextBounds) -> bool {
    !(a.x + a.width < b.x ||
      b.x + b.width < a.x ||
      a.y + a.height < b.y ||
      b.y + b.height < a.y)
}
```

### Label Rendering

```rust
fn render_relationship_label(
    edge: &RenderEdge,
    position: (f32, f32),
) -> String {
    let mut svg = String::new();

    // Background rectangle for readability
    if let Some(description) = &edge.description {
        let width = estimate_text_width(description, edge.style.font_size);
        let height = edge.style.font_size as f32 * 1.2;

        svg.push_str(&format!(
            r##"<rect x="{}" y="{}" width="{}" height="{}"
                 fill="white" opacity="0.9" rx="2"/>"##,
            position.0 - width / 2.0,
            position.1 - height / 2.0,
            width,
            height
        ));

        // Description text
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" text-anchor="middle"
                 font-family="{}" font-size="{}" fill="{}">
                {}</text>"##,
            position.0,
            position.1 + 4.0,
            edge.style.font_family,
            edge.style.font_size,
            edge.style.color,
            description
        ));
    }

    // Technology label (below description)
    if let Some(technology) = &edge.technology {
        let tech_y = position.1 + edge.style.font_size as f32 + 4.0;

        svg.push_str(&format!(
            r##"<text x="{}" y="{}" text-anchor="middle"
                 font-family="{}" font-size="{}" fill="{}">
                [{}]</text>"##,
            position.0,
            tech_y,
            edge.style.font_family,
            edge.style.font_size - 1,
            edge.style.color,
            technology
        ));
    }

    svg
}
```

## Collision Detection

### Text Width Estimation

```rust
fn estimate_text_width(text: &str, font_size: i32) -> f32 {
    // Approximation based on font metrics
    let char_width = font_size as f32 * 0.6; // Average character width
    text.len() as f32 * char_width
}
```

### Collision Grid

For performance with many labels:

```rust
struct CollisionGrid {
    cell_size: f32,
    cells: HashMap<(i32, i32), Vec<TextBounds>>,
}

impl CollisionGrid {
    fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: HashMap::new(),
        }
    }

    fn add(&mut self, bounds: TextBounds) {
        let cells = self.get_cells(&bounds);
        for cell in cells {
            self.cells.entry(cell).or_default().push(bounds.clone());
        }
    }

    fn has_collision(&self, bounds: &TextBounds) -> bool {
        let cells = self.get_cells(bounds);
        for cell in cells {
            if let Some(existing) = self.cells.get(&cell) {
                for other in existing {
                    if bounds_overlap(bounds, other) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn get_cells(&self, bounds: &TextBounds) -> Vec<(i32, i32)> {
        let min_x = (bounds.x / self.cell_size).floor() as i32;
        let max_x = ((bounds.x + bounds.width) / self.cell_size).ceil() as i32;
        let min_y = (bounds.y / self.cell_size).floor() as i32;
        let max_y = ((bounds.y + bounds.height) / self.cell_size).ceil() as i32;

        let mut cells = Vec::new();
        for x in min_x..=max_x {
            for y in min_y..=max_y {
                cells.push((x, y));
            }
        }
        cells
    }
}
```

## Implementation Details

### Rendering Pipeline Integration

```rust
// In svg.rs
fn render_relationships(
    relationships: &[Relationship],
    elements: &HashMap<String, RenderNode>,
    style_cache: &mut StyleCache,
) -> String {
    let mut svg = String::new();
    let mut label_bounds = Vec::new();

    // Pre-compute edge indices for port distribution
    let edge_indices = compute_edge_indices(relationships);

    for (i, relationship) in relationships.iter().enumerate() {
        let source = elements.get(&relationship.source);
        let target = elements.get(&relationship.target);

        if source.is_none() || target.is_none() {
            continue;
        }

        let source = source.unwrap();
        let target = target.unwrap();

        // Determine routing strategy
        let routing = determine_routing(relationship, source, target);

        // Calculate intersection points with offset
        let (edge_index, edge_count) = edge_indices.get(&(
            relationship.source.clone(),
            relationship.target.clone()
        )).unwrap();

        let start = line_rect_intersection_with_offset(
            source.center(),
            target.center(),
            &source.bounds(),
            *edge_index,
            *edge_count,
        );

        let end = line_rect_intersection_with_offset(
            target.center(),
            source.center(),
            &target.bounds(),
            *edge_index,
            *edge_count,
        );

        // Apply routing
        let path = match routing {
            RoutingStyle::Direct => DirectRouter::route_points(start, end),
            RoutingStyle::Orthogonal => OrthogonalRouter::route_bounds(source, target),
            RoutingStyle::Curved => CurvedRouter::route_points(start, end),
        };

        // Resolve style
        let style = style_cache.get_or_resolve_relationship(relationship);

        // Determine if outbound (for bold rendering)
        let is_outbound = is_active_element(&relationship.source);

        // Render line
        svg.push_str(&render_relationship_line(&path, &style, is_outbound));

        // Find label position
        if relationship.description.is_some() || relationship.technology.is_some() {
            let (x, y, anchor) = find_non_overlapping_position(
                &path,
                relationship.description.as_deref().unwrap_or(""),
                &label_bounds,
                style.font_size,
            );

            // Render label
            svg.push_str(&render_relationship_label(relationship, (x, y), &anchor));

            // Track bounds for collision detection
            label_bounds.push(TextBounds { x, y, /* ... */ });
        }
    }

    svg
}
```

### Performance Optimizations

1. **Edge Index Caching**: Pre-compute indices for edge distribution
2. **Style Caching**: Resolve styles once per relationship type
3. **Collision Grid**: Use spatial partitioning for many labels
4. **Path Simplification**: Remove redundant waypoints in orthogonal routing
5. **Marker Deduplication**: Generate each arrow marker once

### Edge Cases

Handle special cases gracefully:

```rust
fn handle_self_loop(element: &RenderNode) -> EdgePath {
    // Create loop that exits and re-enters the same element
    let center = element.center();
    let offset = 50.0;

    EdgePath::Curved {
        start: Point { x: center.0 + 20.0, y: center.1 },
        control1: Point { x: center.0 + offset, y: center.1 - offset },
        control2: Some(Point { x: center.0 - offset, y: center.1 - offset }),
        end: Point { x: center.0 - 20.0, y: center.1 },
    }
}

fn handle_overlapping_elements(source: &Bounds, target: &Bounds) -> EdgePath {
    // Elements overlap - create minimal visible line
    EdgePath::Direct {
        start: Point::from(source.center()),
        end: Point::from(target.center()),
    }
}
```

## Future Enhancements

### Planned Features

1. **Smart Routing**: A* pathfinding around obstacles
2. **Bundled Edges**: Group parallel edges together
3. **Edge Animations**: Animated flow along relationships
4. **Interactive Labels**: Clickable/hoverable labels
5. **Custom Routing**: User-defined waypoints

### Extension Example

```rust
// Custom routing strategy
pub trait EdgeRouter {
    fn route(
        &self,
        source: &Bounds,
        target: &Bounds,
        context: &RoutingContext,
    ) -> EdgePath;
}

struct SmartRouter;

impl EdgeRouter for SmartRouter {
    fn route(
        &self,
        source: &Bounds,
        target: &Bounds,
        context: &RoutingContext,
    ) -> EdgePath {
        // Use A* to find optimal path
        let path = astar_pathfind(source, target, &context.obstacles);
        EdgePath::Orthogonal { waypoints: path }
    }
}
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Text Handling](text-handling.md) - Label rendering and collision detection
- [Coordinate Systems](coordinate-systems.md) - Point and bounds calculations
- [Style System](style-system.md) - Relationship styling
- [Connector Rendering Implementation](connector-rendering-implementation.md) - Matching Java behavior