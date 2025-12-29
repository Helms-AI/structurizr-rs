# Text Handling Implementation

This document details the text rendering system in structurizr-rs, including text positioning within elements, word wrapping, label collision detection, and font management.

## Table of Contents

1. [Overview](#overview)
2. [Element Text Layout](#element-text-layout)
3. [Word Wrapping](#word-wrapping)
4. [Label Collision Detection](#label-collision-detection)
5. [Font Management](#font-management)
6. [Text Positioning Strategies](#text-positioning-strategies)
7. [Performance Optimizations](#performance-optimizations)
8. [Implementation Details](#implementation-details)

## Overview

The text handling system in structurizr-rs manages all text rendering within diagrams, from element names and descriptions to relationship labels. The system handles multi-line text, automatic word wrapping, and collision avoidance.

### Text Components

```
Text System
├── Element Text
│   ├── Name (bold, primary)
│   ├── Type Metadata [Software System]
│   ├── Technology [Java, React, etc.]
│   └── Description (wrapped, multi-line)
├── Relationship Labels
│   ├── Description
│   └── Technology
└── Diagram Annotations
    ├── Group Labels
    └── View Titles
```

## Element Text Layout

Text within elements follows a structured vertical layout:

### Layout Positions

```rust
// Text positioning constants (relative to element top)
const NAME_OFFSET_Y: f32 = 40.0;        // Element name
const TYPE_OFFSET_Y: f32 = 18.0;        // Type metadata offset from name
const TECH_OFFSET_Y: f32 = 34.0;        // Technology offset from name
const DESC_OFFSET_Y: f32 = 52.0;        // Description offset (with tech)
const DESC_OFFSET_NO_TECH: f32 = 38.0;  // Description offset (no tech)

struct ElementTextLayout {
    name_y: f32,
    type_y: Option<f32>,
    technology_y: Option<f32>,
    description_y: Option<f32>,
}

impl ElementTextLayout {
    fn calculate(element: &RenderNode, style: &ResolvedElementStyle) -> Self {
        let base_y = element.y;
        let mut layout = ElementTextLayout {
            name_y: base_y + NAME_OFFSET_Y,
            type_y: None,
            technology_y: None,
            description_y: None,
        };

        let mut current_y = layout.name_y;

        // Type metadata (e.g., "[Software System]")
        if style.show_metadata {
            current_y += TYPE_OFFSET_Y;
            layout.type_y = Some(current_y);
        }

        // Technology label (e.g., "[Java]")
        if let Some(_) = &element.technology {
            current_y += TECH_OFFSET_Y - TYPE_OFFSET_Y;
            layout.technology_y = Some(current_y);
        }

        // Description text
        if style.show_description && element.description.is_some() {
            current_y += if element.technology.is_some() {
                DESC_OFFSET_Y - TECH_OFFSET_Y
            } else {
                DESC_OFFSET_NO_TECH
            };
            layout.description_y = Some(current_y);
        }

        layout
    }
}
```

### Text Rendering

```rust
fn render_element_text(element: &RenderNode, style: &ResolvedElementStyle) -> String {
    let mut svg = String::new();
    let center_x = element.x + element.width / 2.0;
    let layout = ElementTextLayout::calculate(element, style);

    // Element name (bold, larger)
    svg.push_str(&format!(
        r##"<text x="{}" y="{}" text-anchor="middle"
             font-family="{}" font-size="{}" font-weight="bold"
             fill="{}">
            {}
        </text>"##,
        center_x,
        layout.name_y,
        style.font_family,
        style.font_size,
        style.color,
        escape_xml(&element.name)
    ));

    // Type metadata
    if let Some(type_y) = layout.type_y {
        svg.push_str(&format!(
            r##"<text x="{}" y="{}" text-anchor="middle"
                 font-family="{}" font-size="{}"
                 fill="{}" opacity="0.8">
                [{}]
            </text>"##,
            center_x,
            type_y,
            style.font_family,
            style.font_size - 5,  // Smaller font
            style.color,
            element.element_type
        ));
    }

    // Technology
    if let Some(tech_y) = layout.technology_y {
        if let Some(tech) = &element.technology {
            svg.push_str(&format!(
                r##"<text x="{}" y="{}" text-anchor="middle"
                     font-family="{}" font-size="{}"
                     fill="{}">
                    [{}]
                </text>"##,
                center_x,
                tech_y,
                style.font_family,
                style.font_size - 2,
                style.color,
                escape_xml(tech)
            ));
        }
    }

    // Description (wrapped)
    if let Some(desc_y) = layout.description_y {
        if let Some(description) = &element.description {
            svg.push_str(&render_wrapped_text(
                description,
                center_x,
                desc_y,
                element.width * 0.9,  // 90% of element width
                style,
            ));
        }
    }

    svg
}
```

## Word Wrapping

Automatic text wrapping for descriptions:

### Wrapping Algorithm

```rust
const MAX_CHARS_PER_LINE: usize = 40;
const MAX_DESCRIPTION_LINES: usize = 3;

fn wrap_text(text: &str, max_width: f32, font_size: i32) -> Vec<String> {
    let words = text.split_whitespace().collect::<Vec<_>>();
    let mut lines = Vec::new();
    let mut current_line = String::new();

    let char_width = estimate_char_width(font_size);
    let max_chars = (max_width / char_width) as usize;
    let max_chars = max_chars.min(MAX_CHARS_PER_LINE);

    for word in words {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else {
            let test_line = format!("{} {}", current_line, word);
            if test_line.len() <= max_chars {
                current_line = test_line;
            } else {
                // Current line is full, start new line
                lines.push(current_line);
                current_line = word.to_string();

                // Check if we've hit max lines
                if lines.len() >= MAX_DESCRIPTION_LINES {
                    break;
                }
            }
        }
    }

    // Add last line if not empty
    if !current_line.is_empty() && lines.len() < MAX_DESCRIPTION_LINES {
        lines.push(current_line);
    }

    // Add ellipsis if text was truncated
    if lines.len() == MAX_DESCRIPTION_LINES && !words.is_empty() {
        if let Some(last) = lines.last_mut() {
            *last = format!("{}...", last.trim_end());
        }
    }

    lines
}
```

### Wrapped Text Rendering

```rust
fn render_wrapped_text(
    text: &str,
    x: f32,
    y: f32,
    max_width: f32,
    style: &ResolvedElementStyle,
) -> String {
    let lines = wrap_text(text, max_width, style.font_size);

    if lines.is_empty() {
        return String::new();
    }

    let mut svg = format!(
        r##"<text x="{}" y="{}" text-anchor="middle"
             font-family="{}" font-size="{}" fill="{}">"##,
        x, y, style.font_family, style.font_size - 3, style.color
    );

    // Render each line as a tspan
    for (i, line) in lines.iter().enumerate() {
        let dy = if i == 0 { 0.0 } else { 16.0 }; // Line height
        svg.push_str(&format!(
            r##"<tspan x="{}" dy="{}">{}</tspan>"##,
            x, dy, escape_xml(line)
        ));
    }

    svg.push_str("</text>");
    svg
}
```

## Label Collision Detection

System for preventing overlapping relationship labels:

### Collision Detection Data Structure

```rust
#[derive(Clone, Debug)]
struct TextBounds {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

impl TextBounds {
    fn overlaps(&self, other: &TextBounds) -> bool {
        !(self.x + self.width < other.x ||
          other.x + other.width < self.x ||
          self.y + self.height < other.y ||
          other.y + other.height < self.y)
    }

    fn expand(&self, margin: f32) -> TextBounds {
        TextBounds {
            x: self.x - margin,
            y: self.y - margin,
            width: self.width + margin * 2.0,
            height: self.height + margin * 2.0,
        }
    }
}
```

### Collision Avoidance Algorithm

```rust
struct LabelPlacer {
    existing_bounds: Vec<TextBounds>,
    collision_margin: f32,  // Default: 5.0px
}

impl LabelPlacer {
    fn find_position(
        &mut self,
        preferred: (f32, f32),
        text: &str,
        font_size: i32,
    ) -> (f32, f32) {
        let width = estimate_text_width(text, font_size);
        let height = font_size as f32 * 1.4;

        // Try preferred position first
        let bounds = TextBounds {
            x: preferred.0 - width / 2.0,
            y: preferred.1 - height / 2.0,
            width,
            height,
        };

        if !self.has_collision(&bounds) {
            self.existing_bounds.push(bounds);
            return preferred;
        }

        // Try alternative positions in a spiral pattern
        let offsets = generate_spiral_offsets(15.0, 8);

        for (dx, dy) in offsets {
            let test_bounds = TextBounds {
                x: bounds.x + dx,
                y: bounds.y + dy,
                width,
                height,
            };

            if !self.has_collision(&test_bounds) {
                self.existing_bounds.push(test_bounds);
                return (preferred.0 + dx, preferred.1 + dy);
            }
        }

        // Fallback: use preferred position despite collision
        self.existing_bounds.push(bounds);
        preferred
    }

    fn has_collision(&self, bounds: &TextBounds) -> bool {
        let expanded = bounds.expand(self.collision_margin);
        self.existing_bounds.iter().any(|b| b.overlaps(&expanded))
    }
}

fn generate_spiral_offsets(step: f32, levels: usize) -> Vec<(f32, f32)> {
    let mut offsets = Vec::new();

    for level in 1..=levels {
        let distance = step * level as f32;
        let points = level * 8; // Points per level in spiral

        for i in 0..points {
            let angle = (i as f32 / points as f32) * 2.0 * std::f32::consts::PI;
            offsets.push((
                distance * angle.cos(),
                distance * angle.sin(),
            ));
        }
    }

    offsets
}
```

## Font Management

### Font Metrics

```rust
struct FontMetrics {
    family: String,
    size: i32,
    char_width: f32,    // Average character width
    line_height: f32,   // Line spacing
    ascent: f32,        // Above baseline
    descent: f32,       // Below baseline
}

impl FontMetrics {
    fn for_font(family: &str, size: i32) -> Self {
        // Approximations for common fonts
        let char_width = match family {
            "monospace" | "Courier" => size as f32 * 0.6,
            "Arial" | "Helvetica" => size as f32 * 0.55,
            "Times" => size as f32 * 0.5,
            _ => size as f32 * 0.55,  // Default
        };

        FontMetrics {
            family: family.to_string(),
            size,
            char_width,
            line_height: size as f32 * 1.4,
            ascent: size as f32 * 0.8,
            descent: size as f32 * 0.2,
        }
    }

    fn text_width(&self, text: &str) -> f32 {
        text.len() as f32 * self.char_width
    }

    fn text_bounds(&self, text: &str) -> (f32, f32) {
        (self.text_width(text), self.line_height)
    }
}
```

### Text Width Estimation

```rust
fn estimate_text_width(text: &str, font_size: i32) -> f32 {
    // Character width factors for different character types
    let mut width = 0.0;

    for ch in text.chars() {
        width += match ch {
            'i' | 'l' | '.' | '!' | '\'' => 0.3,  // Narrow
            'w' | 'm' | 'W' | 'M' => 0.8,         // Wide
            ' ' => 0.25,                          // Space
            _ => 0.55,                             // Average
        } * font_size as f32;
    }

    width
}

// More accurate estimation using font metrics
fn accurate_text_width(text: &str, font: &FontMetrics) -> f32 {
    text.chars().map(|ch| {
        match ch {
            'i' | 'l' | '.' | ',' | '!' | '\'' | '|' => font.char_width * 0.4,
            'w' | 'm' | 'W' | 'M' | '@' => font.char_width * 1.5,
            ' ' => font.char_width * 0.4,
            '\t' => font.char_width * 4.0,
            _ if ch.is_uppercase() => font.char_width * 1.2,
            _ if ch.is_numeric() => font.char_width * 0.9,
            _ => font.char_width,
        }
    }).sum()
}
```

## Text Positioning Strategies

### Anchor Points

```rust
enum TextAnchor {
    Start,    // Left-aligned
    Middle,   // Center-aligned
    End,      // Right-aligned
}

fn apply_text_anchor(x: f32, width: f32, anchor: TextAnchor) -> f32 {
    match anchor {
        TextAnchor::Start => x,
        TextAnchor::Middle => x - width / 2.0,
        TextAnchor::End => x - width,
    }
}
```

### Vertical Alignment

```rust
enum VerticalAlign {
    Top,
    Middle,
    Bottom,
    Baseline,
}

fn calculate_text_y(
    bounds: &Bounds,
    font_metrics: &FontMetrics,
    align: VerticalAlign,
) -> f32 {
    match align {
        VerticalAlign::Top => bounds.y + font_metrics.ascent,
        VerticalAlign::Middle => {
            bounds.y + bounds.height / 2.0 + font_metrics.ascent / 2.0
        }
        VerticalAlign::Bottom => {
            bounds.y + bounds.height - font_metrics.descent
        }
        VerticalAlign::Baseline => bounds.y + bounds.height / 2.0,
    }
}
```

### Shape-Specific Positioning

Different shapes require different text positioning:

```rust
fn text_position_for_shape(shape: Shape, bounds: &Bounds) -> (f32, f32) {
    let center_x = bounds.x + bounds.width / 2.0;

    match shape {
        Shape::Person => {
            // Text below the figure
            (center_x, bounds.y + bounds.height * 0.85)
        }
        Shape::Cylinder => {
            // Text in the middle of the cylinder body
            (center_x, bounds.y + bounds.height * 0.5)
        }
        Shape::Component => {
            // Text slightly higher to avoid tabs
            (center_x, bounds.y + bounds.height * 0.4)
        }
        _ => {
            // Default: centered
            (center_x, bounds.y + bounds.height * 0.5)
        }
    }
}
```

## Performance Optimizations

### Text Caching

Cache rendered text for reuse:

```rust
struct TextCache {
    cache: HashMap<TextCacheKey, String>,
}

#[derive(Hash, PartialEq, Eq)]
struct TextCacheKey {
    text: String,
    font_family: String,
    font_size: i32,
    color: String,
    max_width: Option<i32>,
}

impl TextCache {
    fn get_or_render(
        &mut self,
        text: &str,
        style: &TextStyle,
        renderer: impl FnOnce() -> String,
    ) -> String {
        let key = TextCacheKey {
            text: text.to_string(),
            font_family: style.font_family.clone(),
            font_size: style.font_size,
            color: style.color.clone(),
            max_width: style.max_width,
        };

        self.cache.entry(key)
            .or_insert_with(renderer)
            .clone()
    }
}
```

### Batch Text Rendering

Group text elements for efficient SVG generation:

```rust
fn render_text_batch(texts: &[TextElement]) -> String {
    let mut svg = String::with_capacity(texts.len() * 200);

    // Group by style for efficient rendering
    let mut by_style: HashMap<TextStyle, Vec<&TextElement>> = HashMap::new();

    for text in texts {
        by_style.entry(text.style.clone())
            .or_default()
            .push(text);
    }

    // Render each style group
    for (style, elements) in by_style {
        svg.push_str(&format!(
            r##"<g font-family="{}" font-size="{}" fill="{}">"##,
            style.font_family, style.font_size, style.color
        ));

        for element in elements {
            svg.push_str(&format!(
                r##"<text x="{}" y="{}">{}</text>"##,
                element.x, element.y, escape_xml(&element.text)
            ));
        }

        svg.push_str("</g>");
    }

    svg
}
```

## Implementation Details

### XML Escaping

Properly escape text for SVG:

```rust
fn escape_xml(text: &str) -> String {
    text.chars().map(|ch| {
        match ch {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&apos;".to_string(),
            _ => ch.to_string(),
        }
    }).collect()
}
```

### Unicode Support

Handle multi-byte characters:

```rust
fn unicode_text_width(text: &str, font_size: i32) -> f32 {
    use unicode_width::UnicodeWidthChar;

    text.chars().map(|ch| {
        let width = ch.width().unwrap_or(1) as f32;
        width * font_size as f32 * 0.5
    }).sum()
}
```

### Text Debugging

Debug visualization for text bounds:

```rust
fn render_text_debug_bounds(bounds: &TextBounds) -> String {
    format!(
        r##"<rect x="{}" y="{}" width="{}" height="{}"
             fill="none" stroke="red" stroke-width="1"
             stroke-dasharray="2,2" opacity="0.5"/>"##,
        bounds.x, bounds.y, bounds.width, bounds.height
    )
}
```

## Examples

### Complex Label Placement

```rust
fn place_relationship_labels(
    relationships: &[Relationship],
    paths: &[EdgePath],
) -> Vec<(f32, f32)> {
    let mut placer = LabelPlacer::default();
    let mut positions = Vec::new();

    for (relationship, path) in relationships.iter().zip(paths) {
        // Calculate preferred position
        let preferred = path.point_at(0.5);  // Midpoint

        // Combine description and technology
        let label_text = format!(
            "{} [{}]",
            relationship.description.as_deref().unwrap_or(""),
            relationship.technology.as_deref().unwrap_or("")
        );

        // Find non-colliding position
        let position = placer.find_position(
            preferred,
            &label_text,
            12,  // Font size
        );

        positions.push(position);
    }

    positions
}
```

### Multi-line Element Description

```rust
fn render_element_with_description(element: &Element) -> String {
    let description = "This is a very long description that needs to be \
                       wrapped across multiple lines to fit within the \
                       element bounds without overflowing";

    let wrapped = wrap_text(description, 380.0, 13);
    let mut svg = String::new();

    for (i, line) in wrapped.iter().enumerate() {
        svg.push_str(&format!(
            r##"<text x="200" y="{}" text-anchor="middle"
                 font-size="13" fill="#000000">
                {}</text>"##,
            100.0 + (i as f32 * 16.0),
            escape_xml(line)
        ));
    }

    svg
}
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Main rendering system
- [Edge Routing](edge-routing.md) - Relationship label positioning
- [Style System](style-system.md) - Text styling
- [Shape Rendering](shape-rendering.md) - Text positioning within shapes
- [Coordinate Systems](coordinate-systems.md) - Text coordinate calculations