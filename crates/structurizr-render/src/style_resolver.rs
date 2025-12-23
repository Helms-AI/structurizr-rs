//! Style resolution for diagram elements.
//!
//! This module resolves styles from element tags using the workspace's style definitions.
//! It provides a `ResolvedStyle` struct that contains all the computed style values
//! with appropriate defaults.

use structurizr_core::style::{Border, ElementStyle, LineStyle, RelationshipStyle, Routing, Shape, Styles};

/// Default colors matching Structurizr's color scheme.
pub mod defaults {
    pub const PERSON_BACKGROUND: &str = "#08427b";
    pub const SOFTWARE_SYSTEM_BACKGROUND: &str = "#1168bd";
    pub const CONTAINER_BACKGROUND: &str = "#438dd5";
    pub const COMPONENT_BACKGROUND: &str = "#85bbf0";
    pub const EXTERNAL_BACKGROUND: &str = "#999999";
    pub const DEPLOYMENT_NODE_BACKGROUND: &str = "#ffffff";
    pub const INFRASTRUCTURE_NODE_BACKGROUND: &str = "#ffffff";

    pub const TEXT_COLOR: &str = "#ffffff";
    pub const DARK_TEXT_COLOR: &str = "#000000";
    pub const RELATIONSHIP_COLOR: &str = "#707070";

    pub const STROKE_COLOR: &str = "#000000";
    pub const DEFAULT_STROKE_WIDTH: u32 = 2;
    pub const DEFAULT_FONT_SIZE: u32 = 14;
    pub const DEFAULT_RELATIONSHIP_THICKNESS: u32 = 2;

    pub const DEFAULT_WIDTH: u32 = 450;
    pub const DEFAULT_HEIGHT: u32 = 300;
    pub const PERSON_WIDTH: u32 = 400;
    pub const PERSON_HEIGHT: u32 = 400;
}

/// Type of element for default style resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementKind {
    Person,
    SoftwareSystem,
    Container,
    Component,
    DeploymentNode,
    InfrastructureNode,
    /// External/unspecified system
    External,
}

impl ElementKind {
    /// Get the default background color for this element kind.
    pub fn default_background(&self) -> &'static str {
        match self {
            ElementKind::Person => defaults::PERSON_BACKGROUND,
            ElementKind::SoftwareSystem => defaults::SOFTWARE_SYSTEM_BACKGROUND,
            ElementKind::Container => defaults::CONTAINER_BACKGROUND,
            ElementKind::Component => defaults::COMPONENT_BACKGROUND,
            ElementKind::External => defaults::EXTERNAL_BACKGROUND,
            ElementKind::DeploymentNode => defaults::DEPLOYMENT_NODE_BACKGROUND,
            ElementKind::InfrastructureNode => defaults::INFRASTRUCTURE_NODE_BACKGROUND,
        }
    }

    /// Get the default text color for this element kind.
    pub fn default_text_color(&self) -> &'static str {
        match self {
            ElementKind::DeploymentNode | ElementKind::InfrastructureNode => defaults::DARK_TEXT_COLOR,
            _ => defaults::TEXT_COLOR,
        }
    }

    /// Get the default shape for this element kind.
    pub fn default_shape(&self) -> Shape {
        match self {
            ElementKind::Person => Shape::Person,
            ElementKind::Component => Shape::Component,
            _ => Shape::RoundedBox,
        }
    }

    /// Get the default dimensions for this element kind.
    pub fn default_dimensions(&self) -> (u32, u32) {
        match self {
            ElementKind::Person => (defaults::PERSON_WIDTH, defaults::PERSON_HEIGHT),
            _ => (defaults::DEFAULT_WIDTH, defaults::DEFAULT_HEIGHT),
        }
    }
}

/// Fully resolved style for an element with all defaults applied.
#[derive(Debug, Clone)]
pub struct ResolvedElementStyle {
    pub shape: Shape,
    pub background: String,
    pub color: String,
    pub stroke: String,
    pub stroke_width: u32,
    pub font_size: u32,
    pub border: Border,
    pub width: u32,
    pub height: u32,
    pub opacity: u32,
    pub icon: Option<String>,
    pub show_metadata: bool,
    pub show_description: bool,
}

impl ResolvedElementStyle {
    /// Create a resolved style with defaults for the given element kind.
    pub fn with_defaults(kind: ElementKind) -> Self {
        let (width, height) = kind.default_dimensions();
        Self {
            shape: kind.default_shape(),
            background: kind.default_background().to_string(),
            color: kind.default_text_color().to_string(),
            stroke: defaults::STROKE_COLOR.to_string(),
            stroke_width: defaults::DEFAULT_STROKE_WIDTH,
            font_size: defaults::DEFAULT_FONT_SIZE,
            border: Border::Solid,
            width,
            height,
            opacity: 100,
            icon: None,
            show_metadata: true,
            show_description: true,
        }
    }

    /// Apply an element style on top of current values.
    pub fn apply(&mut self, style: &ElementStyle) {
        if let Some(shape) = style.shape {
            self.shape = shape;
        }
        if let Some(ref background) = style.background {
            self.background = background.clone();
        }
        if let Some(ref color) = style.color {
            self.color = color.clone();
        }
        if let Some(ref stroke) = style.stroke {
            self.stroke = stroke.clone();
        }
        if let Some(stroke_width) = style.stroke_width {
            self.stroke_width = stroke_width;
        }
        if let Some(font_size) = style.font_size {
            self.font_size = font_size;
        }
        if let Some(border) = style.border {
            self.border = border;
        }
        if let Some(width) = style.width {
            self.width = width;
        }
        if let Some(height) = style.height {
            self.height = height;
        }
        if let Some(opacity) = style.opacity {
            self.opacity = opacity;
        }
        if style.icon.is_some() {
            self.icon = style.icon.clone();
        }
        if let Some(metadata) = style.metadata {
            self.show_metadata = metadata;
        }
        if let Some(description) = style.description {
            self.show_description = description;
        }
    }
}

/// Fully resolved style for a relationship.
#[derive(Debug, Clone)]
pub struct ResolvedRelationshipStyle {
    pub thickness: u32,
    pub color: String,
    pub style: LineStyle,
    pub routing: Routing,
    pub font_size: u32,
    pub opacity: u32,
    pub dashed: bool,
}

impl Default for ResolvedRelationshipStyle {
    fn default() -> Self {
        Self {
            thickness: defaults::DEFAULT_RELATIONSHIP_THICKNESS,
            color: defaults::RELATIONSHIP_COLOR.to_string(),
            style: LineStyle::Solid,
            routing: Routing::Direct,
            font_size: 12,
            opacity: 100,
            dashed: false,
        }
    }
}

impl ResolvedRelationshipStyle {
    /// Apply a relationship style on top of current values.
    pub fn apply(&mut self, style: &RelationshipStyle) {
        if let Some(thickness) = style.thickness {
            self.thickness = thickness;
        }
        if let Some(ref color) = style.color {
            self.color = color.clone();
        }
        if let Some(line_style) = style.style {
            self.style = line_style;
            self.dashed = matches!(line_style, LineStyle::Dashed | LineStyle::Dotted);
        }
        if let Some(routing) = style.routing {
            self.routing = routing;
        }
        if let Some(font_size) = style.font_size {
            self.font_size = font_size;
        }
        if let Some(opacity) = style.opacity {
            self.opacity = opacity;
        }
    }

    /// Get the SVG stroke-dasharray value for this style.
    pub fn stroke_dasharray(&self) -> Option<&'static str> {
        match self.style {
            LineStyle::Solid => None,
            LineStyle::Dashed => Some("10,5"),
            LineStyle::Dotted => Some("2,2"),
        }
    }
}

/// Resolves styles for elements and relationships.
pub struct StyleResolver<'a> {
    styles: Option<&'a Styles>,
}

impl<'a> StyleResolver<'a> {
    /// Create a new style resolver with the given styles.
    pub fn new(styles: Option<&'a Styles>) -> Self {
        Self { styles }
    }

    /// Resolve the style for an element with the given tags.
    pub fn resolve_element(&self, tags: &[String], kind: ElementKind) -> ResolvedElementStyle {
        let mut resolved = ResolvedElementStyle::with_defaults(kind);

        if let Some(styles) = self.styles {
            // Get the merged style from all matching tags
            let merged = styles.get_element_style(tags);
            resolved.apply(&merged);
        }

        resolved
    }

    /// Resolve the style for a relationship with the given tags.
    pub fn resolve_relationship(&self, tags: &[String]) -> ResolvedRelationshipStyle {
        let mut resolved = ResolvedRelationshipStyle::default();

        if let Some(styles) = self.styles {
            let merged = styles.get_relationship_style(tags);
            resolved.apply(&merged);
        }

        resolved
    }
}

/// Convert a Border to SVG stroke-dasharray value.
pub fn border_dasharray(border: Border) -> Option<&'static str> {
    match border {
        Border::Solid => None,
        Border::Dashed => Some("10,5"),
        Border::Dotted => Some("2,2"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_kind_defaults() {
        assert_eq!(ElementKind::Person.default_background(), "#08427b");
        assert_eq!(ElementKind::SoftwareSystem.default_background(), "#1168bd");
        assert_eq!(ElementKind::Container.default_background(), "#438dd5");
        assert_eq!(ElementKind::Component.default_background(), "#85bbf0");
        assert_eq!(ElementKind::External.default_background(), "#999999");
    }

    #[test]
    fn test_resolved_style_defaults() {
        let style = ResolvedElementStyle::with_defaults(ElementKind::Person);
        assert_eq!(style.shape, Shape::Person);
        assert_eq!(style.background, "#08427b");
        assert_eq!(style.color, "#ffffff");
    }

    #[test]
    fn test_style_application() {
        let mut resolved = ResolvedElementStyle::with_defaults(ElementKind::SoftwareSystem);

        let custom_style = ElementStyle {
            tag: "Custom".to_string(),
            background: Some("#ff0000".to_string()),
            shape: Some(Shape::Hexagon),
            ..Default::default()
        };

        resolved.apply(&custom_style);

        assert_eq!(resolved.background, "#ff0000");
        assert_eq!(resolved.shape, Shape::Hexagon);
        // Other properties should retain defaults
        assert_eq!(resolved.color, "#ffffff");
    }

    #[test]
    fn test_resolver_with_styles() {
        let mut styles = Styles::new();
        styles.add_element_style(
            ElementStyle::new("Person")
                .with_background("#123456")
        );

        let resolver = StyleResolver::new(Some(&styles));
        let tags = vec!["Element".to_string(), "Person".to_string()];

        let resolved = resolver.resolve_element(&tags, ElementKind::Person);
        assert_eq!(resolved.background, "#123456");
    }

    #[test]
    fn test_resolver_without_styles() {
        let resolver = StyleResolver::new(None);
        let tags = vec!["Element".to_string(), "Person".to_string()];

        let resolved = resolver.resolve_element(&tags, ElementKind::Person);
        // Should use defaults
        assert_eq!(resolved.background, "#08427b");
    }

    #[test]
    fn test_relationship_style_defaults() {
        let style = ResolvedRelationshipStyle::default();
        assert_eq!(style.thickness, 2);
        assert_eq!(style.color, "#707070");
        assert!(!style.dashed);
    }

    #[test]
    fn test_relationship_dasharray() {
        let mut style = ResolvedRelationshipStyle::default();
        assert!(style.stroke_dasharray().is_none());

        style.style = LineStyle::Dashed;
        style.dashed = true;
        assert_eq!(style.stroke_dasharray(), Some("10,5"));

        style.style = LineStyle::Dotted;
        assert_eq!(style.stroke_dasharray(), Some("2,2"));
    }
}
