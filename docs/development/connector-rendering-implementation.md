# Connector Rendering Implementation Guide

## Executive Summary

After extensive analysis of Structurizr Java's implementation, we've discovered that it uses **JointJS** (a JavaScript diagramming library) for client-side rendering rather than server-side SVG generation. This guide documents the implementation in structurizr-rs that matches Structurizr Java's appearance.

## Implementation Status

**Completed Features:**
- JointJS-compatible filled triangle arrow markers
- Per-color arrow markers (each relationship gets its own colored arrow)
- Smooth cubic bezier curve connectors
- Parallel relationship spreading algorithm (JointJS adjustVertices equivalent)
- JointJS-standard dash patterns (5,5 for dashed, 2,2 for dotted)

## Key Findings from Structurizr Java Analysis

### Architecture
- **Frontend**: JointJS 3.6.5 for diagram rendering (JavaScript)
- **Backend**: Java provides JSON data with layout positions
- **Layout**: Graphviz for auto-layout calculations
- **Rendering**: Pure client-side SVG generation

### Connector Routing Algorithms
Structurizr supports three routing styles:
1. **Direct**: Straight lines between elements
2. **Orthogonal**: Manhattan-style routing (horizontal/vertical segments)
3. **Curved**: Smooth bezier curves

## Implementation Recommendations

### 1. Arrow Head Rendering (Priority: HIGH)

**Current Issue**: Arrow heads don't match JointJS style
**Solution**: Implement SVG markers matching JointJS exactly

```rust
// In crates/structurizr-render/src/svg.rs

fn create_arrow_markers(color: &str) -> String {
    let marker_id = format!("arrow-{}", color.trim_start_matches('#'));
    format!(r##"
    <defs>
      <marker id="{}" markerWidth="10" markerHeight="10"
              refX="9" refY="3" orient="auto" markerUnits="strokeWidth">
        <path d="M 0 0 L 0 6 L 9 3 z" fill="{}" />
      </marker>
    </defs>"##, marker_id, color)
}

// Usage in relationship rendering:
// Add marker-end="url(#arrow-COLOR)" to path element
```

### 2. Line Styling for Interaction Styles (Priority: HIGH)

**Current Issue**: Missing proper dash patterns for async relationships
**Solution**: Apply correct stroke-dasharray patterns

```rust
// In relationship rendering
impl InteractionStyle {
    pub fn to_svg_stroke_dasharray(&self) -> &'static str {
        match self {
            InteractionStyle::Synchronous => "",
            InteractionStyle::Asynchronous => "5,5",
            // Future styles:
            // InteractionStyle::Dotted => "2,2",
            // InteractionStyle::Bold => "" (with stroke-width="3")
        }
    }
}

// In SVG generation:
let dash_attr = if !style.to_svg_stroke_dasharray().is_empty() {
    format!(r#" stroke-dasharray="{}""#, style.to_svg_stroke_dasharray())
} else {
    String::new()
};
```

### 3. Curved Connector Routing (Priority: MEDIUM)

**Current Issue**: Only straight lines, no curves
**Solution**: Implement bezier curve routing

```rust
// In crates/structurizr-render/src/routing/curved.rs

pub fn route_curved_bezier(
    source: &Bounds,
    target: &Bounds,
    vertices: &[Point]  // Optional control points
) -> EdgePath {
    let start = find_edge_point(source, target);
    let end = find_edge_point(target, source);

    if vertices.is_empty() {
        // Auto-generate control point for smooth curve
        let dx = end.x - start.x;
        let dy = end.y - start.y;

        // Offset perpendicular to line for natural curve
        let control = Point {
            x: start.x + dx * 0.5 + dy * 0.1,
            y: start.y + dy * 0.5 - dx * 0.1,
        };

        EdgePath::Curved {
            control_points: vec![start, control, end],
        }
    } else {
        // Use provided vertices as control points
        let mut points = vec![start];
        points.extend_from_slice(vertices);
        points.push(end);

        EdgePath::Curved {
            control_points: points,
        }
    }
}

// SVG path generation for curves:
impl EdgePath {
    pub fn to_svg_path(&self) -> String {
        match self {
            EdgePath::Curved { control_points } => {
                if control_points.len() == 3 {
                    // Quadratic bezier
                    format!("M {} {} Q {} {} {} {}",
                        control_points[0].x, control_points[0].y,
                        control_points[1].x, control_points[1].y,
                        control_points[2].x, control_points[2].y)
                } else if control_points.len() == 4 {
                    // Cubic bezier
                    format!("M {} {} C {} {} {} {} {} {}",
                        control_points[0].x, control_points[0].y,
                        control_points[1].x, control_points[1].y,
                        control_points[2].x, control_points[2].y,
                        control_points[3].x, control_points[3].y)
                } else {
                    // Fall back to polyline for multiple points
                    self.to_polyline_path()
                }
            }
            // ... other variants
        }
    }
}
```

### 4. Orthogonal (Manhattan) Routing (Priority: LOW)

**Current Issue**: No orthogonal routing support
**Solution**: Implement manhattan-style pathfinding

```rust
// In crates/structurizr-render/src/routing/orthogonal.rs

pub fn route_manhattan(
    source: &Bounds,
    target: &Bounds,
    avoid: &[Bounds]  // Other elements to avoid
) -> EdgePath {
    // Simplified manhattan routing:
    // 1. Exit source horizontally or vertically
    // 2. Navigate around obstacles
    // 3. Enter target horizontally or vertically

    let start = find_best_exit_point(source, target);
    let end = find_best_entry_point(target, source);

    // Create orthogonal path
    let waypoints = if should_go_horizontal_first(start, end) {
        vec![
            start,
            Point::new(end.x, start.y),  // Horizontal then vertical
            end,
        ]
    } else {
        vec![
            start,
            Point::new(start.x, end.y),  // Vertical then horizontal
            end,
        ]
    };

    EdgePath::Orthogonal { waypoints }
}
```

### 5. Smart Label Placement (Priority: MEDIUM)

**Current Issue**: Labels may overlap with elements
**Solution**: Intelligent label positioning

```rust
// In crates/structurizr-render/src/routing/mod.rs

impl EdgePath {
    pub fn label_position(&self) -> (f64, f64, f64) {
        match self {
            EdgePath::Curved { control_points } => {
                // Place at curve apex
                if control_points.len() >= 3 {
                    let mid = control_points[control_points.len() / 2];
                    (mid.x, mid.y - 10.0, 0.0)  // Offset above curve
                } else {
                    self.midpoint_label_position()
                }
            }
            EdgePath::Orthogonal { waypoints } => {
                // Find longest segment
                let (segment_start, segment_end) = self.find_longest_segment();
                let mid_x = (segment_start.x + segment_end.x) / 2.0;
                let mid_y = (segment_start.y + segment_end.y) / 2.0;

                // Offset perpendicular to segment
                let is_horizontal = (segment_end.y - segment_start.y).abs() < 0.1;
                if is_horizontal {
                    (mid_x, mid_y - 12.0, 0.0)  // Above horizontal lines
                } else {
                    (mid_x + 12.0, mid_y, 0.0)  // Right of vertical lines
                }
            }
            EdgePath::Direct { start, end } => {
                let mid_x = (start.x + end.x) / 2.0;
                let mid_y = (start.y + end.y) / 2.0;

                // Calculate perpendicular offset
                let dx = end.x - start.x;
                let dy = end.y - start.y;
                let len = (dx * dx + dy * dy).sqrt();

                if len > 0.0 {
                    // Perpendicular unit vector
                    let px = -dy / len * 12.0;
                    let py = dx / len * 12.0;

                    (mid_x + px, mid_y + py, 0.0)
                } else {
                    (mid_x, mid_y, 0.0)
                }
            }
        }
    }
}
```

## Testing Recommendations

### Visual Regression Tests
1. Create reference diagrams with Structurizr Java
2. Generate same diagrams with structurizr-rs
3. Compare visual output pixel-by-pixel

### Test Cases
- [ ] Simple two-element diagram with single relationship
- [ ] Multi-element diagram with crossing relationships
- [ ] Synchronous vs asynchronous relationship styles
- [ ] Curved routing with multiple control points
- [ ] Orthogonal routing around obstacles
- [ ] Label placement on various connector angles

## Performance Considerations

1. **Curve Calculation**: Cache bezier calculations for static diagrams
2. **Path Finding**: Use A* for complex orthogonal routing
3. **Collision Detection**: Spatial indexing for large diagrams

## Migration Path

### Phase 1 (Quick Wins)
1. Implement arrow markers
2. Add dash patterns for async relationships
3. Basic curved routing

### Phase 2 (Visual Polish)
4. Smart label placement
5. Advanced curved routing with control points
6. Relationship style variations

### Phase 3 (Advanced Features)
7. Orthogonal routing
8. Collision avoidance
9. Interactive vertex editing (if needed)

## References

- [JointJS Documentation](https://www.jointjs.com/docs)
- [SVG Path Specification](https://www.w3.org/TR/SVG/paths.html)
- [Bezier Curves in SVG](https://developer.mozilla.org/en-US/docs/Web/SVG/Tutorial/Paths)
- Structurizr Java Source: `github.com/structurizr/java`
- Structurizr UI Source: `github.com/structurizr/ui`