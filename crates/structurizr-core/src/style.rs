//! Styling for elements and relationships in diagrams.

use serde::{Deserialize, Serialize};

/// Shape of an element in a diagram.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Shape {
    #[default]
    Box,
    RoundedBox,
    Circle,
    Ellipse,
    Hexagon,
    Cylinder,
    Pipe,
    Person,
    Robot,
    Folder,
    WebBrowser,
    MobileDevicePortrait,
    MobileDeviceLandscape,
    Component,
}

/// Border style for elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Border {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Line style for relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

/// Routing style for relationship lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Routing {
    #[default]
    Direct,
    Orthogonal,
    Curved,
}

/// Style for elements with a specific tag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ElementStyle {
    pub tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape: Option<Shape>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<Border>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<bool>,
}

impl ElementStyle {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            ..Default::default()
        }
    }

    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn with_background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_stroke(mut self, stroke: impl Into<String>) -> Self {
        self.stroke = Some(stroke.into());
        self
    }

    pub fn with_font_size(mut self, size: u32) -> Self {
        self.font_size = Some(size);
        self
    }

    pub fn with_border(mut self, border: Border) -> Self {
        self.border = Some(border);
        self
    }
}

/// Style for relationships with a specific tag.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipStyle {
    pub tag: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<LineStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub routing: Option<Routing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<u32>,
}

impl RelationshipStyle {
    pub fn new(tag: impl Into<String>) -> Self {
        Self {
            tag: tag.into(),
            ..Default::default()
        }
    }

    pub fn with_thickness(mut self, thickness: u32) -> Self {
        self.thickness = Some(thickness);
        self
    }

    pub fn with_color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn with_style(mut self, style: LineStyle) -> Self {
        self.style = Some(style);
        self
    }

    pub fn with_routing(mut self, routing: Routing) -> Self {
        self.routing = Some(routing);
        self
    }

    pub fn dashed(mut self) -> Self {
        self.style = Some(LineStyle::Dashed);
        self
    }
}

/// A theme URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub url: String,
}

/// Configuration for diagram styling.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Styles {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ElementStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub themes: Vec<String>,
}

impl Styles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_element_style(&mut self, style: ElementStyle) {
        self.elements.push(style);
    }

    pub fn add_relationship_style(&mut self, style: RelationshipStyle) {
        self.relationships.push(style);
    }

    pub fn add_theme(&mut self, url: impl Into<String>) {
        self.themes.push(url.into());
    }

    /// Get the effective style for an element with given tags.
    pub fn get_element_style(&self, tags: &[String]) -> ElementStyle {
        let mut result = ElementStyle::default();

        for style in &self.elements {
            if tags.iter().any(|t| t == &style.tag) {
                // Merge style properties (later styles override earlier ones)
                if style.shape.is_some() {
                    result.shape = style.shape;
                }
                if style.icon.is_some() {
                    result.icon = style.icon.clone();
                }
                if style.width.is_some() {
                    result.width = style.width;
                }
                if style.height.is_some() {
                    result.height = style.height;
                }
                if style.background.is_some() {
                    result.background = style.background.clone();
                }
                if style.color.is_some() {
                    result.color = style.color.clone();
                }
                if style.stroke.is_some() {
                    result.stroke = style.stroke.clone();
                }
                if style.stroke_width.is_some() {
                    result.stroke_width = style.stroke_width;
                }
                if style.font_size.is_some() {
                    result.font_size = style.font_size;
                }
                if style.border.is_some() {
                    result.border = style.border;
                }
                if style.opacity.is_some() {
                    result.opacity = style.opacity;
                }
            }
        }

        result
    }

    /// Get the effective style for a relationship with given tags.
    pub fn get_relationship_style(&self, tags: &[String]) -> RelationshipStyle {
        let mut result = RelationshipStyle::default();

        for style in &self.relationships {
            if tags.iter().any(|t| t == &style.tag) {
                if style.thickness.is_some() {
                    result.thickness = style.thickness;
                }
                if style.color.is_some() {
                    result.color = style.color.clone();
                }
                if style.style.is_some() {
                    result.style = style.style;
                }
                if style.routing.is_some() {
                    result.routing = style.routing;
                }
                if style.font_size.is_some() {
                    result.font_size = style.font_size;
                }
                if style.width.is_some() {
                    result.width = style.width;
                }
                if style.position.is_some() {
                    result.position = style.position;
                }
                if style.opacity.is_some() {
                    result.opacity = style.opacity;
                }
            }
        }

        result
    }
}

/// Branding configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Branding {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font: Option<Font>,
}

/// Font configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Font {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

/// Terminology customization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Terminology {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enterprise: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub person: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub software_system: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub container: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deployment_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub infrastructure_node: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relationship: Option<String>,
}

/// Configuration for views.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ViewConfiguration {
    #[serde(default)]
    pub styles: Styles,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branding: Option<Branding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminology: Option<Terminology>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_view: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_saved_view: Option<String>,
}
