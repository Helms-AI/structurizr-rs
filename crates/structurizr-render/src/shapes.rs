//! Shape rendering for diagram elements.
//!
//! This module provides SVG rendering for all supported C4/Structurizr shapes.

use structurizr_core::style::Shape;
use crate::style_resolver::{border_dasharray, ResolvedElementStyle};

/// Bounds for rendering a shape.
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl Bounds {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn center_x(&self) -> f64 {
        self.x + self.width / 2.0
    }

    pub fn center_y(&self) -> f64 {
        self.y + self.height / 2.0
    }
}

/// Render a shape to SVG.
pub fn render_shape(shape: Shape, bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    match shape {
        Shape::Box => render_box(bounds, style),
        Shape::RoundedBox => render_rounded_box(bounds, style),
        Shape::Circle => render_circle(bounds, style),
        Shape::Ellipse => render_ellipse(bounds, style),
        Shape::Hexagon => render_hexagon(bounds, style),
        Shape::Cylinder => render_cylinder(bounds, style),
        Shape::Pipe => render_pipe(bounds, style),
        Shape::Person => render_person(bounds, style),
        Shape::Robot => render_robot(bounds, style),
        Shape::Folder => render_folder(bounds, style),
        Shape::WebBrowser => render_web_browser(bounds, style),
        Shape::MobileDevicePortrait => render_mobile_device(bounds, style, true),
        Shape::MobileDeviceLandscape => render_mobile_device(bounds, style, false),
        Shape::Component => render_component(bounds, style),
    }
}

/// Generate common style attributes for a shape.
fn shape_style_attrs(style: &ResolvedElementStyle) -> String {
    let mut attrs = format!(
        r#"fill="{}" stroke="{}" stroke-width="{}""#,
        style.background, style.stroke, style.stroke_width
    );

    if let Some(dasharray) = border_dasharray(style.border) {
        attrs.push_str(&format!(r#" stroke-dasharray="{}""#, dasharray));
    }

    if style.opacity < 100 {
        attrs.push_str(&format!(r#" opacity="{}""#, style.opacity as f64 / 100.0));
    }

    attrs
}

/// Render a box (rectangle with no corner radius).
fn render_box(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" {}/>"#,
        bounds.x, bounds.y, bounds.width, bounds.height,
        shape_style_attrs(style)
    )
}

/// Render a rounded box.
fn render_rounded_box(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let radius = 5.0;
    format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="{:.0}" {}/>"#,
        bounds.x, bounds.y, bounds.width, bounds.height, radius,
        shape_style_attrs(style)
    )
}

/// Render a circle.
fn render_circle(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let radius = bounds.width.min(bounds.height) / 2.0;
    format!(
        r#"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" {}/>"#,
        bounds.center_x(), bounds.center_y(), radius,
        shape_style_attrs(style)
    )
}

/// Render an ellipse.
fn render_ellipse(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    format!(
        r#"<ellipse cx="{:.0}" cy="{:.0}" rx="{:.0}" ry="{:.0}" {}/>"#,
        bounds.center_x(), bounds.center_y(),
        bounds.width / 2.0, bounds.height / 2.0,
        shape_style_attrs(style)
    )
}

/// Render a hexagon.
fn render_hexagon(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let cx = bounds.center_x();
    let cy = bounds.center_y();
    let h = bounds.height / 2.0;

    // Hexagon points (pointy top/bottom)
    let points = format!(
        "{:.0},{:.0} {:.0},{:.0} {:.0},{:.0} {:.0},{:.0} {:.0},{:.0} {:.0},{:.0}",
        cx, bounds.y,                     // Top point
        bounds.x + bounds.width, cy - h * 0.5, // Top right
        bounds.x + bounds.width, cy + h * 0.5, // Bottom right
        cx, bounds.y + bounds.height,     // Bottom point
        bounds.x, cy + h * 0.5,           // Bottom left
        bounds.x, cy - h * 0.5,           // Top left
    );

    format!(
        r#"<polygon points="{}" {}/>"#,
        points, shape_style_attrs(style)
    )
}

/// Render a cylinder (database shape).
fn render_cylinder(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let ellipse_height = bounds.height * 0.15;
    let body_height = bounds.height - ellipse_height;

    let mut svg = String::new();

    // Main body (rectangle)
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="{}" stroke="none"/>"#,
        bounds.x, bounds.y + ellipse_height / 2.0,
        bounds.width, body_height,
        style.background
    ));

    // Bottom ellipse
    svg.push_str(&format!(
        r#"<ellipse cx="{:.0}" cy="{:.0}" rx="{:.0}" ry="{:.0}" {}/>"#,
        bounds.center_x(),
        bounds.y + bounds.height - ellipse_height / 2.0,
        bounds.width / 2.0,
        ellipse_height / 2.0,
        shape_style_attrs(style)
    ));

    // Side lines
    svg.push_str(&format!(
        r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}"/>"#,
        bounds.x, bounds.y + ellipse_height / 2.0,
        bounds.x, bounds.y + bounds.height - ellipse_height / 2.0,
        style.stroke, style.stroke_width
    ));
    svg.push_str(&format!(
        r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}"/>"#,
        bounds.x + bounds.width, bounds.y + ellipse_height / 2.0,
        bounds.x + bounds.width, bounds.y + bounds.height - ellipse_height / 2.0,
        style.stroke, style.stroke_width
    ));

    // Top ellipse (drawn last to be on top)
    svg.push_str(&format!(
        r#"<ellipse cx="{:.0}" cy="{:.0}" rx="{:.0}" ry="{:.0}" {}/>"#,
        bounds.center_x(),
        bounds.y + ellipse_height / 2.0,
        bounds.width / 2.0,
        ellipse_height / 2.0,
        shape_style_attrs(style)
    ));

    svg
}

/// Render a pipe (horizontal cylinder).
fn render_pipe(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let ellipse_width = bounds.width * 0.15;
    let body_width = bounds.width - ellipse_width;

    let mut svg = String::new();

    // Main body
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="{}" stroke="none"/>"#,
        bounds.x + ellipse_width / 2.0, bounds.y,
        body_width, bounds.height,
        style.background
    ));

    // Left ellipse
    svg.push_str(&format!(
        r#"<ellipse cx="{:.0}" cy="{:.0}" rx="{:.0}" ry="{:.0}" {}/>"#,
        bounds.x + ellipse_width / 2.0,
        bounds.center_y(),
        ellipse_width / 2.0,
        bounds.height / 2.0,
        shape_style_attrs(style)
    ));

    // Top and bottom lines
    svg.push_str(&format!(
        r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}"/>"#,
        bounds.x + ellipse_width / 2.0, bounds.y,
        bounds.x + bounds.width - ellipse_width / 2.0, bounds.y,
        style.stroke, style.stroke_width
    ));
    svg.push_str(&format!(
        r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}"/>"#,
        bounds.x + ellipse_width / 2.0, bounds.y + bounds.height,
        bounds.x + bounds.width - ellipse_width / 2.0, bounds.y + bounds.height,
        style.stroke, style.stroke_width
    ));

    // Right ellipse
    svg.push_str(&format!(
        r#"<ellipse cx="{:.0}" cy="{:.0}" rx="{:.0}" ry="{:.0}" {}/>"#,
        bounds.x + bounds.width - ellipse_width / 2.0,
        bounds.center_y(),
        ellipse_width / 2.0,
        bounds.height / 2.0,
        shape_style_attrs(style)
    ));

    svg
}

/// Render a person shape.
fn render_person(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let mut svg = String::new();

    // Head
    let head_radius = bounds.width * 0.15;
    let head_cx = bounds.center_x();
    let head_cy = bounds.y + head_radius + 10.0;

    svg.push_str(&format!(
        r#"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" {}/>"#,
        head_cx, head_cy, head_radius,
        shape_style_attrs(style)
    ));

    // Body (trapezoid-ish rounded rectangle)
    let body_top = head_cy + head_radius + 10.0;
    let body_height = bounds.y + bounds.height - body_top - 20.0;
    let body_width = bounds.width * 0.7;
    let body_x = bounds.x + (bounds.width - body_width) / 2.0;

    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="10" {}/>"#,
        body_x, body_top, body_width, body_height,
        shape_style_attrs(style)
    ));

    svg
}

/// Render a robot shape.
fn render_robot(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let mut svg = String::new();

    // Head (square with antenna)
    let head_size = bounds.width * 0.35;
    let head_x = bounds.center_x() - head_size / 2.0;
    let head_y = bounds.y + 20.0;

    // Antenna
    svg.push_str(&format!(
        r#"<line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}"/>"#,
        bounds.center_x(), head_y,
        bounds.center_x(), head_y - 15.0,
        style.stroke, style.stroke_width
    ));
    svg.push_str(&format!(
        r#"<circle cx="{:.0}" cy="{:.0}" r="5" fill="{}"/>"#,
        bounds.center_x(), head_y - 18.0, style.background
    ));

    // Head
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="5" {}/>"#,
        head_x, head_y, head_size, head_size,
        shape_style_attrs(style)
    ));

    // Eyes
    let eye_radius = head_size * 0.12;
    let eye_y = head_y + head_size * 0.4;
    svg.push_str(&format!(
        r#"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="{}"/>"#,
        head_x + head_size * 0.3, eye_y, eye_radius, style.color
    ));
    svg.push_str(&format!(
        r#"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="{}"/>"#,
        head_x + head_size * 0.7, eye_y, eye_radius, style.color
    ));

    // Body
    let body_top = head_y + head_size + 10.0;
    let body_height = bounds.y + bounds.height - body_top - 20.0;
    let body_width = bounds.width * 0.6;
    let body_x = bounds.x + (bounds.width - body_width) / 2.0;

    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="5" {}/>"#,
        body_x, body_top, body_width, body_height,
        shape_style_attrs(style)
    ));

    svg
}

/// Render a folder shape.
fn render_folder(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let tab_width = bounds.width * 0.3;
    let tab_height = bounds.height * 0.12;

    let path = format!(
        "M {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} L {:.0} {:.0} Z",
        bounds.x, bounds.y + tab_height,  // Start
        bounds.x + tab_width * 0.8, bounds.y + tab_height,  // To tab start
        bounds.x + tab_width, bounds.y,  // Tab top left
        bounds.x + tab_width * 1.5, bounds.y,  // Tab top right
        bounds.x + tab_width * 1.7, bounds.y + tab_height,  // Tab bottom
        bounds.x + bounds.width, bounds.y + tab_height,  // Top right
        bounds.x + bounds.width, bounds.y + bounds.height,  // Bottom right
        bounds.x, bounds.y + bounds.height,  // Bottom left
    );

    format!(
        r#"<path d="{}" {}/>"#,
        path, shape_style_attrs(style)
    )
}

/// Render a web browser shape.
fn render_web_browser(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let mut svg = String::new();
    let toolbar_height = bounds.height * 0.12;

    // Main window
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="5" {}/>"#,
        bounds.x, bounds.y, bounds.width, bounds.height,
        shape_style_attrs(style)
    ));

    // Toolbar area (lighter)
    svg.push_str(&format!(
        r##"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="#ffffff" fill-opacity="0.3"/>"##,
        bounds.x + 2.0, bounds.y + 2.0, bounds.width - 4.0, toolbar_height
    ));

    // Window buttons (circles)
    let button_y = bounds.y + toolbar_height / 2.0;
    let button_r = toolbar_height * 0.2;
    svg.push_str(&format!(
        r##"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="#ff5f57"/>"##,
        bounds.x + 15.0, button_y, button_r
    ));
    svg.push_str(&format!(
        r##"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="#febc2e"/>"##,
        bounds.x + 35.0, button_y, button_r
    ));
    svg.push_str(&format!(
        r##"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="#28c840"/>"##,
        bounds.x + 55.0, button_y, button_r
    ));

    // URL bar
    svg.push_str(&format!(
        r##"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="3" fill="#ffffff" fill-opacity="0.5"/>"##,
        bounds.x + 75.0, bounds.y + 5.0, bounds.width - 90.0, toolbar_height - 10.0
    ));

    svg
}

/// Render a mobile device shape.
fn render_mobile_device(bounds: &Bounds, style: &ResolvedElementStyle, portrait: bool) -> String {
    let mut svg = String::new();

    let (width, height) = if portrait {
        (bounds.width, bounds.height)
    } else {
        (bounds.height, bounds.width)
    };

    // Adjust bounds for landscape
    let device_bounds = if portrait {
        *bounds
    } else {
        Bounds::new(
            bounds.x + (bounds.width - height) / 2.0,
            bounds.y + (bounds.height - width) / 2.0,
            height,
            width,
        )
    };

    let corner_radius = width * 0.1;
    let bezel = width * 0.05;

    // Device frame
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="{:.0}" {}/>"#,
        device_bounds.x, device_bounds.y, device_bounds.width, device_bounds.height, corner_radius,
        shape_style_attrs(style)
    ));

    // Screen (inner rectangle)
    svg.push_str(&format!(
        r##"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="3" fill="#000000" fill-opacity="0.1"/>"##,
        device_bounds.x + bezel,
        device_bounds.y + bezel * 2.5,
        device_bounds.width - bezel * 2.0,
        device_bounds.height - bezel * 4.0
    ));

    // Home button / notch indicator
    let button_y = device_bounds.y + device_bounds.height - bezel * 1.5;
    svg.push_str(&format!(
        r##"<circle cx="{:.0}" cy="{:.0}" r="{:.0}" fill="#ffffff" fill-opacity="0.3"/>"##,
        device_bounds.center_x(), button_y, bezel * 0.7
    ));

    svg
}

/// Render a component shape (box with component tabs).
fn render_component(bounds: &Bounds, style: &ResolvedElementStyle) -> String {
    let mut svg = String::new();

    let tab_width = 30.0;
    let tab_height = 15.0;
    let tab_offset = 20.0;

    // Main body
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="3" {}/>"#,
        bounds.x + tab_width / 2.0, bounds.y,
        bounds.width - tab_width / 2.0, bounds.height,
        shape_style_attrs(style)
    ));

    // Top tab
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="2" {}/>"#,
        bounds.x, bounds.y + tab_offset,
        tab_width, tab_height,
        shape_style_attrs(style)
    ));

    // Bottom tab
    svg.push_str(&format!(
        r#"<rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" rx="2" {}/>"#,
        bounds.x, bounds.y + tab_offset + tab_height + 15.0,
        tab_width, tab_height,
        shape_style_attrs(style)
    ));

    svg
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::style::Border;

    fn test_style() -> ResolvedElementStyle {
        ResolvedElementStyle {
            shape: Shape::RoundedBox,
            background: "#1168bd".to_string(),
            color: "#ffffff".to_string(),
            stroke: "#000000".to_string(),
            stroke_width: 2,
            font_size: 14,
            border: Border::Solid,
            width: 450,
            height: 300,
            opacity: 100,
            icon: None,
            icon_position: structurizr_core::style::IconPosition::Top,
            show_metadata: true,
            show_description: true,
        }
    }

    fn test_bounds() -> Bounds {
        Bounds::new(100.0, 100.0, 450.0, 300.0)
    }

    #[test]
    fn test_render_rounded_box() {
        let svg = render_rounded_box(&test_bounds(), &test_style());
        assert!(svg.contains("rect"));
        assert!(svg.contains("rx=\"5\""));
        assert!(svg.contains("fill=\"#1168bd\""));
    }

    #[test]
    fn test_render_circle() {
        let svg = render_circle(&test_bounds(), &test_style());
        assert!(svg.contains("circle"));
        assert!(svg.contains("cx=\"325\"")); // center x
    }

    #[test]
    fn test_render_hexagon() {
        let svg = render_hexagon(&test_bounds(), &test_style());
        assert!(svg.contains("polygon"));
        assert!(svg.contains("points="));
    }

    #[test]
    fn test_render_cylinder() {
        let svg = render_cylinder(&test_bounds(), &test_style());
        assert!(svg.contains("ellipse"));
        assert!(svg.contains("rect"));
    }

    #[test]
    fn test_render_person() {
        let svg = render_person(&test_bounds(), &test_style());
        assert!(svg.contains("circle")); // Head
        assert!(svg.contains("rect"));   // Body
    }

    #[test]
    fn test_render_component() {
        let svg = render_component(&test_bounds(), &test_style());
        // Should have main body + 2 tabs = 3 rectangles
        assert_eq!(svg.matches("rect").count(), 3);
    }

    #[test]
    fn test_render_web_browser() {
        let svg = render_web_browser(&test_bounds(), &test_style());
        assert!(svg.contains("rect")); // Window
        assert!(svg.contains("circle")); // Window buttons
    }

    #[test]
    fn test_all_shapes_render() {
        let bounds = test_bounds();
        let style = test_style();

        // Test that all shapes render without panicking
        let shapes = [
            Shape::Box,
            Shape::RoundedBox,
            Shape::Circle,
            Shape::Ellipse,
            Shape::Hexagon,
            Shape::Cylinder,
            Shape::Pipe,
            Shape::Person,
            Shape::Robot,
            Shape::Folder,
            Shape::WebBrowser,
            Shape::MobileDevicePortrait,
            Shape::MobileDeviceLandscape,
            Shape::Component,
        ];

        for shape in shapes {
            let svg = render_shape(shape, &bounds, &style);
            assert!(!svg.is_empty(), "Shape {:?} rendered empty SVG", shape);
        }
    }

    #[test]
    fn test_dashed_border() {
        let mut style = test_style();
        style.border = Border::Dashed;

        let svg = render_rounded_box(&test_bounds(), &style);
        assert!(svg.contains("stroke-dasharray=\"10,5\""));
    }

    #[test]
    fn test_opacity() {
        let mut style = test_style();
        style.opacity = 50;

        let svg = render_rounded_box(&test_bounds(), &style);
        assert!(svg.contains("opacity=\"0.5\""));
    }
}
