# Style System Implementation

This document explains the cascading style system in structurizr-rs that controls the visual appearance of elements and relationships in C4 diagrams.

## Table of Contents

1. [Overview](#overview)
2. [Style Resolution Pipeline](#style-resolution-pipeline)
3. [Element Styles](#element-styles)
4. [Relationship Styles](#relationship-styles)
5. [Default Styles](#default-styles)
6. [Tag-Based Cascading](#tag-based-cascading)
7. [Theme Support](#theme-support)
8. [Implementation Details](#implementation-details)

## Overview

The style system in structurizr-rs provides a flexible, cascading approach to visual customization of C4 diagrams. Located primarily in `crates/structurizr-render/src/style_resolver.rs`, it supports element shapes, colors, sizes, fonts, icons, and relationship styling.

### Architecture

```
Workspace Styles (DSL)
    ↓
Default Styles (by element kind)
    ↓
Tag-Based Style Rules
    ↓
Resolved Style (final computed style)
    ↓
SVG Attributes
```

## Style Resolution Pipeline

The style resolution process follows a cascade from defaults through tag-based rules to produce final styles.

### Resolution Steps

1. **Start with Defaults**: Based on element kind (Person, SoftwareSystem, etc.)
2. **Apply Workspace Styles**: Global styles defined in the DSL
3. **Apply Tag Styles**: Iterate through element tags, merging matching styles
4. **Resolve Properties**: Fill in any missing properties with defaults
5. **Apply Theme**: Override with theme-specific values (e.g., dark mode)

### Implementation

```rust
// crates/structurizr-render/src/style_resolver.rs
pub fn resolve_element_style(
    element: &Element,
    styles: &Styles,
    theme: Option<&Theme>,
) -> ResolvedElementStyle {
    // Start with defaults
    let mut resolved = default_element_style(&element.kind);

    // Apply workspace-level styles
    if let Some(workspace_style) = styles.default_element_style() {
        resolved.merge(workspace_style);
    }

    // Apply tag-based styles in order
    for tag in &element.tags {
        if let Some(tag_style) = styles.elements.get(tag) {
            resolved.merge(tag_style);
        }
    }

    // Apply theme overrides
    if let Some(theme) = theme {
        resolved.apply_theme(theme);
    }

    // Ensure all required properties are set
    resolved.fill_defaults();

    resolved
}
```

## Element Styles

### Style Properties

```rust
pub struct ElementStyle {
    // Shape and size
    pub shape: Option<Shape>,
    pub width: Option<i32>,
    pub height: Option<i32>,

    // Colors
    pub background: Option<String>,  // Fill color
    pub color: Option<String>,       // Text color
    pub stroke: Option<String>,      // Border color
    pub stroke_width: Option<f32>,
    pub stroke_dasharray: Option<String>,

    // Text
    pub font_size: Option<i32>,
    pub font_family: Option<String>,

    // Visual properties
    pub opacity: Option<i32>,        // 0-100
    pub border: Option<Border>,

    // Icons and metadata
    pub icon: Option<String>,        // URL or data URI
    pub icon_position: Option<IconPosition>,
    pub show_metadata: Option<bool>,
    pub show_description: Option<bool>,
}
```

### Resolved Style

After resolution, all optional properties have values:

```rust
pub struct ResolvedElementStyle {
    pub shape: Shape,
    pub width: i32,
    pub height: i32,
    pub background: String,
    pub color: String,
    pub stroke: String,
    pub stroke_width: f32,
    pub stroke_dasharray: Option<String>,
    pub font_size: i32,
    pub font_family: String,
    pub opacity: i32,
    pub border: Border,
    pub icon: Option<String>,
    pub icon_position: IconPosition,
    pub show_metadata: bool,
    pub show_description: bool,
}
```

### Shape Types

```rust
pub enum Shape {
    Box,
    RoundedBox,
    Circle,
    Ellipse,
    Hexagon,
    Cylinder,
    Component,
    Person,
    Robot,
    Folder,
    WebBrowser,
    MobileDevicePortrait,
    MobileDeviceLandscape,
    Pipe,
}
```

### Border Styles

```rust
pub enum Border {
    Solid,
    Dashed,
    Dotted,
}
```

## Relationship Styles

### Style Properties

```rust
pub struct RelationshipStyle {
    // Line appearance
    pub thickness: Option<i32>,      // Stroke width
    pub color: Option<String>,       // Stroke color
    pub dashed: Option<bool>,        // Dashed line
    pub routing: Option<RoutingStyle>,
    pub opacity: Option<i32>,        // 0-100

    // Text
    pub font_size: Option<i32>,
    pub font_family: Option<String>,

    // Positioning
    pub position: Option<i32>,       // Position along line (0-100)
}
```

### Routing Styles

```rust
pub enum RoutingStyle {
    Direct,       // Straight line
    Orthogonal,   // Right angles
    Curved,       // Bezier curves
}
```

### Resolution Process

```rust
pub fn resolve_relationship_style(
    relationship: &Relationship,
    styles: &Styles,
) -> ResolvedRelationshipStyle {
    let mut resolved = default_relationship_style();

    // Apply tag-based styles
    for tag in &relationship.tags {
        if let Some(tag_style) = styles.relationships.get(tag) {
            resolved.merge(tag_style);
        }
    }

    resolved
}
```

## Default Styles

Default styles are defined based on element kind and provide sensible starting values.

### Element Defaults

```rust
fn default_element_style(kind: &ElementKind) -> ResolvedElementStyle {
    match kind {
        ElementKind::Person => ResolvedElementStyle {
            shape: Shape::Person,
            background: "#08427b".to_string(),  // Dark blue
            color: "#ffffff".to_string(),       // White text
            stroke: "#052e56".to_string(),      // Darker blue
            stroke_width: 2.0,
            width: 400,
            height: 250,
            font_size: 16,
            font_family: "Arial, sans-serif".to_string(),
            opacity: 100,
            border: Border::Solid,
            icon: None,
            icon_position: IconPosition::Top,
            show_metadata: true,
            show_description: true,
            stroke_dasharray: None,
        },

        ElementKind::SoftwareSystem => ResolvedElementStyle {
            shape: Shape::RoundedBox,
            background: "#1168bd".to_string(),  // Blue
            color: "#ffffff".to_string(),
            stroke: "#0b4884".to_string(),
            // ... other properties
        },

        ElementKind::Container => ResolvedElementStyle {
            shape: Shape::RoundedBox,
            background: "#438dd5".to_string(),  // Lighter blue
            color: "#ffffff".to_string(),
            stroke: "#2e6196".to_string(),
            // ... other properties
        },

        ElementKind::Component => ResolvedElementStyle {
            shape: Shape::Component,
            background: "#85bbf0".to_string(),  // Light blue
            color: "#000000".to_string(),       // Black text
            stroke: "#5d82a8".to_string(),
            // ... other properties
        },

        ElementKind::DeploymentNode => ResolvedElementStyle {
            shape: Shape::Box,
            background: "#ffffff".to_string(),  // White
            color: "#000000".to_string(),       // Black text
            stroke: "#888888".to_string(),      // Gray
            stroke_dasharray: Some("5,5".to_string()),  // Dashed
            // ... other properties
        },

        // ... other element kinds
    }
}
```

### Relationship Defaults

```rust
fn default_relationship_style() -> ResolvedRelationshipStyle {
    ResolvedRelationshipStyle {
        thickness: 2,
        color: "#707070".to_string(),  // Gray
        dashed: false,
        routing: RoutingStyle::Direct,
        opacity: 100,
        font_size: 12,
        font_family: "Arial, sans-serif".to_string(),
        position: 50,  // Middle of line
    }
}
```

## Tag-Based Cascading

Tags enable flexible style application through CSS-like selectors.

### Tag Priority

Tags are applied in the order they appear on elements:

```rust
// Element with tags: ["Database", "Critical", "External"]
// Styles applied in order:
// 1. Default for element kind
// 2. Style for tag "Database"
// 3. Style for tag "Critical"
// 4. Style for tag "External"
```

### Style Merging

Later styles override earlier ones:

```rust
impl ResolvedElementStyle {
    fn merge(&mut self, style: &ElementStyle) {
        if let Some(shape) = &style.shape {
            self.shape = shape.clone();
        }
        if let Some(background) = &style.background {
            self.background = background.clone();
        }
        if let Some(color) = &style.color {
            self.color = color.clone();
        }
        // ... merge other properties
    }
}
```

### DSL Example

```dsl
styles {
    element "Database" {
        shape Cylinder
        background "#ff9900"
    }

    element "Critical" {
        stroke "#ff0000"
        stroke-width 4
    }

    element "External" {
        background "#999999"
        stroke-dasharray "10,5"
    }
}
```

## Theme Support

Themes provide coordinated style sets for different contexts (e.g., light/dark mode).

### Dark Mode Implementation

```rust
pub struct Theme {
    pub name: String,
    pub element_overrides: HashMap<String, ElementStyle>,
    pub relationship_overrides: HashMap<String, RelationshipStyle>,
    pub background_color: Option<String>,
}

impl Theme {
    pub fn dark_mode() -> Self {
        Theme {
            name: "dark".to_string(),
            element_overrides: HashMap::from([
                ("*".to_string(), ElementStyle {
                    stroke: Some("#ffffff".to_string()),
                    color: Some("#ffffff".to_string()),
                    ..Default::default()
                }),
            ]),
            relationship_overrides: HashMap::from([
                ("*".to_string(), RelationshipStyle {
                    color: Some("#cccccc".to_string()),
                    ..Default::default()
                }),
            ]),
            background_color: Some("#1e1e1e".to_string()),
        }
    }
}
```

### Theme Application

```rust
impl ResolvedElementStyle {
    fn apply_theme(&mut self, theme: &Theme) {
        // Apply universal overrides
        if let Some(universal) = theme.element_overrides.get("*") {
            self.merge(universal);
        }

        // Apply specific overrides
        for (selector, override_style) in &theme.element_overrides {
            if self.matches_selector(selector) {
                self.merge(override_style);
            }
        }
    }
}
```

## Implementation Details

### SVG Attribute Generation

Resolved styles are converted to SVG attributes:

```rust
pub fn shape_style_attrs(style: &ResolvedElementStyle) -> String {
    let mut attrs = Vec::new();

    attrs.push(format!(r#"fill="{}""#, style.background));
    attrs.push(format!(r#"stroke="{}""#, style.stroke));
    attrs.push(format!(r#"stroke-width="{}""#, style.stroke_width));

    if let Some(dasharray) = &style.stroke_dasharray {
        attrs.push(format!(r#"stroke-dasharray="{}""#, dasharray));
    }

    if style.opacity < 100 {
        let opacity = style.opacity as f32 / 100.0;
        attrs.push(format!(r#"opacity="{}""#, opacity));
    }

    attrs.join(" ")
}

pub fn text_style_attrs(style: &ResolvedElementStyle) -> String {
    format!(
        r#"font-family="{}" font-size="{}" fill="{}""#,
        style.font_family, style.font_size, style.color
    )
}
```

### Dynamic Style Classes

Elements receive CSS classes for interactive styling:

```rust
fn element_css_classes(element: &Element, is_active: bool) -> String {
    let mut classes = vec!["element".to_string()];

    // Add element type class
    classes.push(format!("element-{}", element.kind.to_string().to_lowercase()));

    // Add tag classes
    for tag in &element.tags {
        classes.push(format!("tag-{}", tag.to_lowercase()));
    }

    // Add state classes
    if is_active {
        classes.push("active".to_string());
    }

    classes.join(" ")
}
```

### Style Validation

Styles are validated during resolution:

```rust
fn validate_style(style: &ElementStyle) -> Result<(), StyleError> {
    // Validate color formats
    if let Some(color) = &style.background {
        validate_color(color)?;
    }

    // Validate numeric ranges
    if let Some(opacity) = style.opacity {
        if opacity < 0 || opacity > 100 {
            return Err(StyleError::InvalidOpacity(opacity));
        }
    }

    if let Some(width) = style.width {
        if width <= 0 {
            return Err(StyleError::InvalidDimension("width", width));
        }
    }

    Ok(())
}

fn validate_color(color: &str) -> Result<(), StyleError> {
    // Accept hex colors (#rgb, #rrggbb)
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 3 || hex.len() == 6 {
            if hex.chars().all(|c| c.is_ascii_hexdigit()) {
                return Ok(());
            }
        }
    }

    // Accept named colors
    if is_valid_css_color_name(color) {
        return Ok(());
    }

    // Accept rgb/rgba functions
    if color.starts_with("rgb") {
        return validate_rgb_function(color);
    }

    Err(StyleError::InvalidColor(color.to_string()))
}
```

### Style Inheritance

Container elements can inherit parent styles:

```rust
fn inherit_container_style(
    container: &Container,
    parent_system: &SoftwareSystem,
    styles: &Styles,
) -> ResolvedElementStyle {
    // Start with parent style
    let mut style = resolve_element_style(parent_system, styles, None);

    // Override with container-specific styles
    let container_style = resolve_element_style(container, styles, None);
    style.merge_from(container_style);

    style
}
```

### Performance Optimization

Style resolution is optimized through caching:

```rust
pub struct StyleCache {
    element_styles: HashMap<String, ResolvedElementStyle>,
    relationship_styles: HashMap<String, ResolvedRelationshipStyle>,
}

impl StyleCache {
    pub fn get_or_resolve_element(
        &mut self,
        element: &Element,
        styles: &Styles,
    ) -> &ResolvedElementStyle {
        self.element_styles
            .entry(element.id.clone())
            .or_insert_with(|| resolve_element_style(element, styles, None))
    }
}
```

## Style Examples

### Custom Database Style

```dsl
styles {
    element "Database" {
        shape Cylinder
        background "#ff9900"
        color "#000000"
        icon "https://example.com/database-icon.png"
    }
}
```

### Highlighting Critical Components

```dsl
styles {
    element "Critical" {
        background "#ff0000"
        color "#ffffff"
        stroke "#aa0000"
        stroke-width 4
    }

    relationship "Critical" {
        color "#ff0000"
        thickness 3
    }
}
```

### External System Style

```dsl
styles {
    element "External" {
        background "#999999"
        shape Box
        stroke-dasharray "5,5"
    }
}
```

## Future Enhancements

### Planned Features

1. **CSS Integration**: Support for external CSS stylesheets
2. **Animation Styles**: Transition and animation properties
3. **Gradient Fills**: Linear and radial gradients
4. **Shadow Effects**: Drop shadows and glow effects
5. **Pattern Fills**: Hatching and texture patterns
6. **Conditional Styles**: Style based on element properties

### Extension Points

```rust
// Custom style property trait
pub trait StyleProperty {
    fn apply_to_svg(&self, element: &mut SvgElement);
    fn validate(&self) -> Result<(), StyleError>;
}

// Register custom properties
style_registry.register_property("custom", Box::new(CustomProperty));
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Shape Rendering](shape-rendering.md) - How shapes are drawn
- [Text Handling](text-handling.md) - Text styling and positioning
- [Drag-and-Drop Implementation](drag-drop-implementation.md) - Interactive styling