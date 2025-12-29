# Coordinate Systems and Transformations

This document explains the coordinate systems used in structurizr-rs for positioning elements, handling transformations, and managing different layout directions.

## Table of Contents

1. [Overview](#overview)
2. [SVG Coordinate System](#svg-coordinate-system)
3. [Element Positioning](#element-positioning)
4. [ViewBox and Content Bounds](#viewbox-and-content-bounds)
5. [Layout Transformations](#layout-transformations)
6. [Drag and Drop Coordinates](#drag-and-drop-coordinates)
7. [Coordinate Utilities](#coordinate-utilities)
8. [Implementation Details](#implementation-details)

## Overview

The coordinate system in structurizr-rs manages the positioning of all visual elements in diagrams. It handles transformations between different coordinate spaces and supports multiple layout directions.

### Coordinate Spaces

```
Coordinate Space Hierarchy:
├── Workspace Coordinates (abstract model)
├── Layout Coordinates (algorithm output)
├── SVG User Coordinates (element positions)
├── SVG ViewBox Coordinates (visible area)
└── Screen Coordinates (drag-and-drop)
```

## SVG Coordinate System

### Origin and Axes

SVG uses a standard 2D coordinate system:
- **Origin (0,0)**: Top-left corner
- **X-axis**: Increases to the right
- **Y-axis**: Increases downward (unlike mathematical coordinates)

```rust
// SVG coordinate representation
#[derive(Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn new(x: f32, y: f32) -> Self {
        Point { x, y }
    }

    pub fn distance_to(&self, other: &Point) -> f32 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn midpoint(&self, other: &Point) -> Point {
        Point {
            x: (self.x + other.x) / 2.0,
            y: (self.y + other.y) / 2.0,
        }
    }
}
```

### Bounds Representation

Elements are positioned using bounding boxes:

```rust
#[derive(Clone, Debug)]
pub struct Bounds {
    pub x: f32,      // Top-left X
    pub y: f32,      // Top-left Y
    pub width: f32,  // Width
    pub height: f32, // Height
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Bounds { x, y, width, height }
    }

    pub fn center(&self) -> Point {
        Point {
            x: self.x + self.width / 2.0,
            y: self.y + self.height / 2.0,
        }
    }

    pub fn top_left(&self) -> Point {
        Point { x: self.x, y: self.y }
    }

    pub fn top_right(&self) -> Point {
        Point { x: self.x + self.width, y: self.y }
    }

    pub fn bottom_left(&self) -> Point {
        Point { x: self.x, y: self.y + self.height }
    }

    pub fn bottom_right(&self) -> Point {
        Point { x: self.x + self.width, y: self.y + self.height }
    }

    pub fn contains(&self, point: &Point) -> bool {
        point.x >= self.x &&
        point.x <= self.x + self.width &&
        point.y >= self.y &&
        point.y <= self.y + self.height
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }
}
```

## Element Positioning

### Absolute Positioning

Elements use absolute coordinates within the SVG canvas:

```rust
pub struct ElementPosition {
    pub x: i32,         // Absolute X coordinate
    pub y: i32,         // Absolute Y coordinate
    pub explicit: bool, // True if manually positioned
}

impl ElementPosition {
    pub fn from_layout(node: &LayoutNode) -> Self {
        ElementPosition {
            x: node.x,
            y: node.y,
            explicit: false,
        }
    }

    pub fn from_explicit(x: i32, y: i32) -> Self {
        ElementPosition {
            x,
            y,
            explicit: true,
        }
    }

    pub fn to_bounds(&self, width: f32, height: f32) -> Bounds {
        Bounds {
            x: self.x as f32,
            y: self.y as f32,
            width,
            height,
        }
    }
}
```

### Relative Positioning

For nested elements and groups:

```rust
pub struct RelativePosition {
    pub offset_x: f32,
    pub offset_y: f32,
    pub parent: Option<String>, // Parent element ID
}

impl RelativePosition {
    pub fn to_absolute(&self, parent_bounds: &Bounds) -> Point {
        Point {
            x: parent_bounds.x + self.offset_x,
            y: parent_bounds.y + self.offset_y,
        }
    }
}
```

## ViewBox and Content Bounds

### Content Bounds Calculation

Track the bounding box of all content:

```rust
pub struct ContentBounds {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl ContentBounds {
    pub fn new() -> Self {
        ContentBounds {
            min_x: f32::MAX,
            min_y: f32::MAX,
            max_x: f32::MIN,
            max_y: f32::MIN,
        }
    }

    pub fn update(&mut self, bounds: &Bounds) {
        self.min_x = self.min_x.min(bounds.x);
        self.min_y = self.min_y.min(bounds.y);
        self.max_x = self.max_x.max(bounds.x + bounds.width);
        self.max_y = self.max_y.max(bounds.y + bounds.height);
    }

    pub fn update_point(&mut self, point: &Point) {
        self.min_x = self.min_x.min(point.x);
        self.min_y = self.min_y.min(point.y);
        self.max_x = self.max_x.max(point.x);
        self.max_y = self.max_y.max(point.y);
    }

    pub fn width(&self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(&self) -> f32 {
        self.max_y - self.min_y
    }

    pub fn to_viewbox(&self, padding: f32) -> String {
        format!(
            "{} {} {} {}",
            self.min_x - padding,
            self.min_y - padding,
            self.width() + 2.0 * padding,
            self.height() + 2.0 * padding
        )
    }
}
```

### Dynamic ViewBox Generation

Create a viewBox that fits all content:

```rust
fn calculate_viewbox(elements: &[RenderNode], relationships: &[RenderEdge]) -> String {
    let mut bounds = ContentBounds::new();

    // Include all elements
    for element in elements {
        bounds.update(&element.bounds());
    }

    // Include relationship paths
    for relationship in relationships {
        for point in relationship.path.points() {
            bounds.update_point(&point);
        }
    }

    // Add padding
    const PADDING: f32 = 50.0;
    bounds.to_viewbox(PADDING)
}
```

## Layout Transformations

### Direction Transformations

Support different layout directions:

```rust
pub enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

pub fn transform_coordinates(
    point: Point,
    direction: Direction,
    bounds: &ContentBounds,
) -> Point {
    match direction {
        Direction::TopBottom => point, // No transformation

        Direction::BottomTop => Point {
            x: point.x,
            y: bounds.height() - point.y,
        },

        Direction::LeftRight => Point {
            x: point.y,  // Swap axes
            y: point.x,
        },

        Direction::RightLeft => Point {
            x: bounds.width() - point.y,  // Swap and flip
            y: point.x,
        },
    }
}
```

### Rotation Transformations

Apply rotations to elements:

```rust
pub struct Transform {
    pub translate: Option<(f32, f32)>,
    pub rotate: Option<f32>,  // Degrees
    pub scale: Option<(f32, f32)>,
}

impl Transform {
    pub fn to_svg_transform(&self) -> String {
        let mut transforms = Vec::new();

        if let Some((tx, ty)) = self.translate {
            transforms.push(format!("translate({}, {})", tx, ty));
        }

        if let Some(angle) = self.rotate {
            transforms.push(format!("rotate({})", angle));
        }

        if let Some((sx, sy)) = self.scale {
            transforms.push(format!("scale({}, {})", sx, sy));
        }

        transforms.join(" ")
    }

    pub fn apply_to_point(&self, point: &Point) -> Point {
        let mut result = point.clone();

        // Apply transformations in order
        if let Some((tx, ty)) = self.translate {
            result.x += tx;
            result.y += ty;
        }

        if let Some(angle) = self.rotate {
            let radians = angle.to_radians();
            let cos = radians.cos();
            let sin = radians.sin();
            let new_x = result.x * cos - result.y * sin;
            let new_y = result.x * sin + result.y * cos;
            result.x = new_x;
            result.y = new_y;
        }

        if let Some((sx, sy)) = self.scale {
            result.x *= sx;
            result.y *= sy;
        }

        result
    }
}
```

### Normalization

Normalize positions to start from origin:

```rust
pub fn normalize_positions(nodes: &mut [LayoutNode]) {
    if nodes.is_empty() {
        return;
    }

    // Find minimum coordinates
    let min_x = nodes.iter().map(|n| n.x).min().unwrap_or(0);
    let min_y = nodes.iter().map(|n| n.y).min().unwrap_or(0);

    // Apply offset to all nodes
    for node in nodes {
        node.x -= min_x;
        node.y -= min_y;
    }

    // Add padding
    const PADDING: i32 = 100;
    for node in nodes {
        node.x += PADDING;
        node.y += PADDING;
    }
}
```

## Drag and Drop Coordinates

### Screen to SVG Conversion

Convert mouse coordinates to SVG coordinates:

```rust
pub struct CoordinateMapper {
    svg_viewbox: Bounds,
    svg_element_bounds: Bounds,
}

impl CoordinateMapper {
    pub fn screen_to_svg(&self, screen_x: f32, screen_y: f32) -> Point {
        // Calculate scale factors
        let scale_x = self.svg_viewbox.width / self.svg_element_bounds.width;
        let scale_y = self.svg_viewbox.height / self.svg_element_bounds.height;

        // Convert to SVG coordinates
        Point {
            x: self.svg_viewbox.x + (screen_x - self.svg_element_bounds.x) * scale_x,
            y: self.svg_viewbox.y + (screen_y - self.svg_element_bounds.y) * scale_y,
        }
    }

    pub fn svg_to_screen(&self, svg_point: &Point) -> Point {
        // Calculate scale factors
        let scale_x = self.svg_element_bounds.width / self.svg_viewbox.width;
        let scale_y = self.svg_element_bounds.height / self.svg_viewbox.height;

        // Convert to screen coordinates
        Point {
            x: self.svg_element_bounds.x + (svg_point.x - self.svg_viewbox.x) * scale_x,
            y: self.svg_element_bounds.y + (svg_point.y - self.svg_viewbox.y) * scale_y,
        }
    }
}
```

### Drag State Management

Track element positions during drag:

```rust
pub struct DragState {
    element_id: String,
    start_position: Point,
    current_position: Point,
    offset: Point,  // Mouse offset from element center
}

impl DragState {
    pub fn start_drag(element_id: String, mouse: Point, element: &Bounds) -> Self {
        let center = element.center();
        DragState {
            element_id,
            start_position: center.clone(),
            current_position: center.clone(),
            offset: Point {
                x: mouse.x - center.x,
                y: mouse.y - center.y,
            },
        }
    }

    pub fn update_position(&mut self, mouse: Point) {
        self.current_position = Point {
            x: mouse.x - self.offset.x,
            y: mouse.y - self.offset.y,
        };
    }

    pub fn get_transform(&self) -> String {
        let dx = self.current_position.x - self.start_position.x;
        let dy = self.current_position.y - self.start_position.y;
        format!("translate({}, {})", dx, dy)
    }
}
```

## Coordinate Utilities

### Geometric Calculations

Common coordinate operations:

```rust
pub mod geometry {
    use super::*;

    pub fn distance(p1: &Point, p2: &Point) -> f32 {
        ((p1.x - p2.x).powi(2) + (p1.y - p2.y).powi(2)).sqrt()
    }

    pub fn angle(p1: &Point, p2: &Point) -> f32 {
        (p2.y - p1.y).atan2(p2.x - p1.x)
    }

    pub fn perpendicular(p1: &Point, p2: &Point) -> Point {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let length = (dx * dx + dy * dy).sqrt();

        if length > 0.0 {
            Point {
                x: -dy / length,
                y: dx / length,
            }
        } else {
            Point { x: 0.0, y: 1.0 }
        }
    }

    pub fn interpolate(p1: &Point, p2: &Point, t: f32) -> Point {
        Point {
            x: p1.x + (p2.x - p1.x) * t,
            y: p1.y + (p2.y - p1.y) * t,
        }
    }

    pub fn point_on_line_closest_to(
        line_start: &Point,
        line_end: &Point,
        point: &Point,
    ) -> Point {
        let dx = line_end.x - line_start.x;
        let dy = line_end.y - line_start.y;

        if dx == 0.0 && dy == 0.0 {
            return line_start.clone();
        }

        let t = ((point.x - line_start.x) * dx + (point.y - line_start.y) * dy)
            / (dx * dx + dy * dy);

        let t = t.max(0.0).min(1.0);

        Point {
            x: line_start.x + t * dx,
            y: line_start.y + t * dy,
        }
    }
}
```

### Grid Snapping

Snap coordinates to a grid:

```rust
pub struct GridSnap {
    pub enabled: bool,
    pub grid_size: f32,
}

impl GridSnap {
    pub fn snap(&self, point: Point) -> Point {
        if !self.enabled {
            return point;
        }

        Point {
            x: (point.x / self.grid_size).round() * self.grid_size,
            y: (point.y / self.grid_size).round() * self.grid_size,
        }
    }

    pub fn snap_bounds(&self, bounds: Bounds) -> Bounds {
        if !self.enabled {
            return bounds;
        }

        let snapped_top_left = self.snap(bounds.top_left());

        Bounds {
            x: snapped_top_left.x,
            y: snapped_top_left.y,
            width: (bounds.width / self.grid_size).round() * self.grid_size,
            height: (bounds.height / self.grid_size).round() * self.grid_size,
        }
    }
}
```

## Implementation Details

### Position Persistence Format

Positions saved in `.positions.json`:

```rust
#[derive(Serialize, Deserialize)]
pub struct PositionData {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ViewPositions {
    pub elements: HashMap<String, PositionData>,
    pub relationships: HashMap<String, Vec<PositionData>>, // Waypoints
}

#[derive(Serialize, Deserialize)]
pub struct PositionsFile {
    pub version: u32,
    pub views: HashMap<String, ViewPositions>,
}

impl PositionsFile {
    pub fn get_element_position(
        &self,
        view_key: &str,
        element_id: &str,
    ) -> Option<Point> {
        self.views
            .get(view_key)?
            .elements
            .get(element_id)
            .map(|data| Point {
                x: data.x as f32,
                y: data.y as f32,
            })
    }

    pub fn set_element_position(
        &mut self,
        view_key: &str,
        element_id: &str,
        position: Point,
    ) {
        let view = self.views.entry(view_key.to_string())
            .or_insert_with(ViewPositions::default);

        view.elements.insert(
            element_id.to_string(),
            PositionData {
                x: position.x as i32,
                y: position.y as i32,
            },
        );
    }
}
```

### Coordinate Validation

Ensure coordinates are within reasonable bounds:

```rust
pub fn validate_coordinates(point: &Point) -> Result<(), CoordinateError> {
    const MAX_COORDINATE: f32 = 100000.0;
    const MIN_COORDINATE: f32 = -100000.0;

    if point.x < MIN_COORDINATE || point.x > MAX_COORDINATE {
        return Err(CoordinateError::OutOfBounds("x", point.x));
    }

    if point.y < MIN_COORDINATE || point.y > MAX_COORDINATE {
        return Err(CoordinateError::OutOfBounds("y", point.y));
    }

    if point.x.is_nan() || point.y.is_nan() {
        return Err(CoordinateError::InvalidValue("NaN detected"));
    }

    if point.x.is_infinite() || point.y.is_infinite() {
        return Err(CoordinateError::InvalidValue("Infinity detected"));
    }

    Ok(())
}
```

### Coordinate Precision

Handle floating point precision:

```rust
pub fn round_coordinate(value: f32, precision: u32) -> f32 {
    let multiplier = 10_f32.powi(precision as i32);
    (value * multiplier).round() / multiplier
}

pub fn format_svg_number(value: f32) -> String {
    // Round to 1 decimal place for SVG output
    format!("{:.1}", value)
}
```

## Examples

### Complete Positioning Pipeline

```rust
fn position_element(
    element: &Element,
    layout: &LayoutNode,
    explicit_positions: &HashMap<String, Point>,
    transform: &Transform,
) -> Bounds {
    // Check for explicit position
    let position = if let Some(explicit) = explicit_positions.get(&element.id) {
        explicit.clone()
    } else {
        // Use layout position
        Point {
            x: layout.x as f32,
            y: layout.y as f32,
        }
    };

    // Apply transformation
    let transformed = transform.apply_to_point(&position);

    // Snap to grid if enabled
    let snapped = GridSnap { enabled: true, grid_size: 10.0 }
        .snap(transformed);

    // Create bounds
    Bounds {
        x: snapped.x,
        y: snapped.y,
        width: layout.width as f32,
        height: layout.height as f32,
    }
}
```

### ViewBox Calculation with Margin

```rust
fn create_diagram_viewbox(diagram: &Diagram) -> String {
    let mut bounds = ContentBounds::new();

    // Include all elements
    for element in &diagram.elements {
        bounds.update(&element.bounds);
    }

    // Include all relationship labels
    for label in &diagram.labels {
        bounds.update_point(&label.position);
    }

    // Add margin
    const MARGIN_PERCENT: f32 = 0.1;
    let margin_x = bounds.width() * MARGIN_PERCENT;
    let margin_y = bounds.height() * MARGIN_PERCENT;

    format!(
        "{} {} {} {}",
        bounds.min_x - margin_x,
        bounds.min_y - margin_y,
        bounds.width() + 2.0 * margin_x,
        bounds.height() + 2.0 * margin_y
    )
}
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Layout Algorithms](layout-algorithms.md) - Coordinate generation
- [Drag-and-Drop Implementation](drag-drop-implementation.md) - Interactive positioning
- [Edge Routing](edge-routing.md) - Path coordinate calculations
- [Shape Rendering](shape-rendering.md) - Element coordinate usage