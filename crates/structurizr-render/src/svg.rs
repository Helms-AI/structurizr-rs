//! SVG rendering for Structurizr diagrams.

use std::collections::{HashMap, HashSet};

/// Convert a color string (e.g., "#707070") to a valid SVG marker ID.
/// Removes the '#' prefix and adds a marker prefix.
fn color_to_marker_id(color: &str) -> String {
    let clean_color = color.trim_start_matches('#');
    format!("arrow-{}", clean_color)
}

use structurizr_core::model::{ElementId, DeploymentNode};
use structurizr_core::style::{IconPosition, Routing, Styles};
use structurizr_core::view::{
    ComponentView, ContainerView, CustomView, DeploymentView, DynamicView,
    FilteredView, FilterMode, ImageView, SystemContextView, SystemLandscapeView,
    ViewProperties,
};
use structurizr_core::Workspace;

use crate::error::Result;
use crate::layout::{GridLayout, LayoutEdge, LayoutNode, Position, Size};
use crate::routing::orthogonal::{OrthogonalConfig, OrthogonalRouter};
use crate::routing::{route_curved, EdgePath};
use crate::shapes::{render_shape, Bounds};
use crate::style_resolver::{ElementKind, StyleResolver};

/// Extract explicit positions from view properties' element views.
/// Returns a HashMap of element_id -> (x, y) for elements with positions set.
fn extract_explicit_positions(view_props: &ViewProperties) -> HashMap<String, (i32, i32)> {
    let mut positions = HashMap::new();
    for element_view in &view_props.elements {
        if let (Some(x), Some(y)) = (element_view.x, element_view.y) {
            positions.insert(element_view.id.to_string(), (x, y));
        }
    }
    positions
}

/// Apply explicit positions to layout nodes, overriding computed positions.
/// Elements with explicit positions use those; others keep their computed positions.
fn apply_explicit_positions(nodes: &mut Vec<LayoutNode>, explicit_positions: &HashMap<String, (i32, i32)>) {
    for node in nodes.iter_mut() {
        if let Some(&(x, y)) = explicit_positions.get(&node.id) {
            node.position = Position { x: x as f64, y: y as f64 };
        }
    }
}

/// Tracks the bounding box of all content in the SVG.
/// Used to calculate dynamic viewBox dimensions.
#[derive(Debug, Clone, Copy)]
pub struct ContentBounds {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

impl ContentBounds {
    /// Create new content bounds with initial values at infinity.
    pub fn new() -> Self {
        Self {
            min_x: f64::INFINITY,
            min_y: f64::INFINITY,
            max_x: f64::NEG_INFINITY,
            max_y: f64::NEG_INFINITY,
        }
    }

    /// Expand bounds to include a rectangle.
    pub fn expand(&mut self, x: f64, y: f64, width: f64, height: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x + width);
        self.max_y = self.max_y.max(y + height);
    }

    /// Expand bounds to include a point.
    pub fn expand_point(&mut self, x: f64, y: f64) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    /// Add padding around the bounds.
    pub fn with_padding(self, padding: f64) -> Self {
        Self {
            min_x: self.min_x - padding,
            min_y: self.min_y - padding,
            max_x: self.max_x + padding,
            max_y: self.max_y + padding,
        }
    }

    /// Get the width of the bounds.
    pub fn width(&self) -> f64 {
        self.max_x - self.min_x
    }

    /// Get the height of the bounds.
    pub fn height(&self) -> f64 {
        self.max_y - self.min_y
    }

    /// Check if the bounds are valid (have been expanded at least once).
    pub fn is_valid(&self) -> bool {
        self.min_x.is_finite() && self.min_y.is_finite()
            && self.max_x.is_finite() && self.max_y.is_finite()
    }

    /// Generate a viewBox string for SVG.
    pub fn to_viewbox(&self) -> String {
        format!("{:.0} {:.0} {:.0} {:.0}",
            self.min_x, self.min_y,
            self.width(), self.height())
    }
}

impl Default for ContentBounds {
    fn default() -> Self {
        Self::new()
    }
}

/// Tracks text label bounding boxes for collision detection.
#[derive(Debug, Clone)]
struct TextBounds {
    x: f64,      // Center X
    y: f64,      // Center Y
    width: f64,  // Estimated text width
    height: f64, // Based on font size
}

/// Configuration for SVG rendering.
#[derive(Debug, Clone)]
pub struct RenderConfig {
    /// Background color for the diagram (default: "#ffffff" for light mode).
    pub background_color: String,
}

impl RenderConfig {
    /// Create a new render config with default settings.
    pub fn new() -> Self {
        Self {
            background_color: "#ffffff".to_string(),
        }
    }

    /// Create a render config for dark mode.
    pub fn dark_mode() -> Self {
        Self {
            background_color: "#1a1a1a".to_string(),
        }
    }

    /// Create a render config from view properties.
    pub fn from_view_properties(props: &ViewProperties) -> Self {
        Self {
            background_color: props.background.clone().unwrap_or_else(|| "#ffffff".to_string()),
        }
    }

    /// Check if dark mode is enabled (background is dark).
    pub fn is_dark_mode(&self) -> bool {
        // Consider it dark mode if background starts with #0, #1, or #2
        self.background_color.starts_with("#0")
            || self.background_color.starts_with("#1")
            || self.background_color.starts_with("#2")
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl TextBounds {
    /// Create a new text bounds from center position and dimensions.
    fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    /// Check if this text bounds overlaps with another.
    fn overlaps(&self, other: &TextBounds) -> bool {
        let half_w1 = self.width / 2.0;
        let half_h1 = self.height / 2.0;
        let half_w2 = other.width / 2.0;
        let half_h2 = other.height / 2.0;

        // Add a small padding (5px) between labels for better readability
        let padding = 5.0;

        (self.x - half_w1 - padding < other.x + half_w2) &&
        (self.x + half_w1 + padding > other.x - half_w2) &&
        (self.y - half_h1 - padding < other.y + half_h2) &&
        (self.y + half_h1 + padding > other.y - half_h2)
    }

    /// Estimate the width of text based on character count and font size.
    /// Uses average character width for Arial/sans-serif (~0.55 of font size).
    fn estimate_width(text: &str, font_size: u32) -> f64 {
        text.len() as f64 * font_size as f64 * 0.55
    }
}

/// Information about a label to be placed on a relationship line.
#[derive(Debug, Clone)]
struct LabelPlacement {
    text: String,
    line_start: (f64, f64),
    line_end: (f64, f64),
    font_size: u32,
    is_technology: bool,
}

/// Calculate the angle of a line in degrees, normalized so text is always readable.
/// Returns an angle between -90 and 90 degrees (text never upside down).
fn calculate_line_angle(x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;

    // Calculate angle in radians, then convert to degrees
    let angle_rad = dy.atan2(dx);
    let mut angle_deg = angle_rad.to_degrees();

    // Normalize angle so text is never upside down
    // Keep angle between -90 and 90 degrees
    if angle_deg > 90.0 {
        angle_deg -= 180.0;
    } else if angle_deg < -90.0 {
        angle_deg += 180.0;
    }

    angle_deg
}

/// Find a non-overlapping position for a label along its relationship line.
fn find_non_overlapping_position(
    label: &LabelPlacement,
    placed_labels: &[TextBounds],
) -> (f64, f64) {
    let (x1, y1) = label.line_start;
    let (x2, y2) = label.line_end;

    // Calculate line direction and length
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len = (dx * dx + dy * dy).sqrt();

    // Handle zero-length lines
    if len < 1.0 {
        return ((x1 + x2) / 2.0, (y1 + y2) / 2.0);
    }

    // Perpendicular unit vector (for offset direction)
    let perp_x = -dy / len;
    let perp_y = dx / len;

    // Base offset: description labels above, technology labels below
    let base_offset = if label.is_technology { 15.0 } else { -15.0 };

    let text_width = TextBounds::estimate_width(&label.text, label.font_size);
    let text_height = label.font_size as f64 * 1.4;

    // Try different positions along the line (midpoint first, then other positions)
    let line_positions = [0.5, 0.35, 0.65, 0.25, 0.75, 0.15, 0.85];

    for &t in &line_positions {
        // Position along the line
        let base_x = x1 + dx * t;
        let base_y = y1 + dy * t;

        // Try increasing perpendicular offsets
        for multiplier in [1.0, 1.5, 2.0, 2.5, 3.0] {
            let offset = base_offset * multiplier;
            let try_x = base_x + perp_x * offset;
            let try_y = base_y + perp_y * offset;

            let bounds = TextBounds::new(try_x, try_y, text_width, text_height);

            if !placed_labels.iter().any(|p| bounds.overlaps(p)) {
                return (try_x, try_y);
            }
        }

        // Also try the opposite side of the line
        for multiplier in [1.0, 1.5, 2.0, 2.5, 3.0] {
            let offset = -base_offset * multiplier;
            let try_x = base_x + perp_x * offset;
            let try_y = base_y + perp_y * offset;

            let bounds = TextBounds::new(try_x, try_y, text_width, text_height);

            if !placed_labels.iter().any(|p| bounds.overlaps(p)) {
                return (try_x, try_y);
            }
        }
    }

    // Fallback: use midpoint with base offset (may overlap, but better than nothing)
    let mid_x = (x1 + x2) / 2.0;
    let mid_y = (y1 + y2) / 2.0;
    (mid_x + perp_x * base_offset, mid_y + perp_y * base_offset)
}

/// Calculate where a line from (x1, y1) to (x2, y2) intersects a rectangle boundary.
/// Returns the intersection point on the rectangle edge closest to (x1, y1).
/// The rectangle is defined by its top-left corner (rect_x, rect_y) and dimensions.
fn line_rect_intersection(
    x1: f64, y1: f64,  // Start point (outside rectangle)
    x2: f64, y2: f64,  // End point (center of rectangle)
    rect_x: f64, rect_y: f64, rect_w: f64, rect_h: f64,
) -> (f64, f64) {
    // Rectangle edges
    let left = rect_x;
    let right = rect_x + rect_w;
    let top = rect_y;
    let bottom = rect_y + rect_h;

    // Direction vector from x1,y1 to x2,y2
    let dx = x2 - x1;
    let dy = y2 - y1;

    // Avoid division by zero
    if dx.abs() < 0.0001 && dy.abs() < 0.0001 {
        return (x2, y2);
    }

    let mut t_min = f64::MAX;
    let mut intersection = (x2, y2);

    // Check intersection with each edge
    // We want the intersection closest to x2,y2 (i.e., largest t value < 1)

    // Left edge (x = left)
    if dx.abs() > 0.0001 {
        let t = (left - x1) / dx;
        if t > 0.0 && t < 1.0 {
            let y = y1 + t * dy;
            if y >= top && y <= bottom {
                let dist = (1.0 - t).abs();
                if dist < t_min {
                    t_min = dist;
                    intersection = (left, y);
                }
            }
        }
    }

    // Right edge (x = right)
    if dx.abs() > 0.0001 {
        let t = (right - x1) / dx;
        if t > 0.0 && t < 1.0 {
            let y = y1 + t * dy;
            if y >= top && y <= bottom {
                let dist = (1.0 - t).abs();
                if dist < t_min {
                    t_min = dist;
                    intersection = (right, y);
                }
            }
        }
    }

    // Top edge (y = top)
    if dy.abs() > 0.0001 {
        let t = (top - y1) / dy;
        if t > 0.0 && t < 1.0 {
            let x = x1 + t * dx;
            if x >= left && x <= right {
                let dist = (1.0 - t).abs();
                if dist < t_min {
                    t_min = dist;
                    intersection = (x, top);
                }
            }
        }
    }

    // Bottom edge (y = bottom)
    if dy.abs() > 0.0001 {
        let t = (bottom - y1) / dy;
        if t > 0.0 && t < 1.0 {
            let x = x1 + t * dx;
            if x >= left && x <= right {
                let dist = (1.0 - t).abs();
                if dist < t_min {
                    // t_min = dist; // Not needed since this is the last check
                    intersection = (x, bottom);
                }
            }
        }
    }

    intersection
}

/// Calculate intersection point with offset for distributing multiple edges.
/// The offset moves the intersection point along the rectangle edge.
fn line_rect_intersection_with_offset(
    x1: f64, y1: f64,  // Start point (outside rectangle)
    x2: f64, y2: f64,  // End point (center of rectangle)
    rect_x: f64, rect_y: f64, rect_w: f64, rect_h: f64,
    edge_idx: usize,
    total_edges: usize,
) -> (f64, f64) {
    // Get the base intersection point
    let (base_x, base_y) = line_rect_intersection(x1, y1, x2, y2, rect_x, rect_y, rect_w, rect_h);

    // If only one edge, no offset needed
    if total_edges <= 1 {
        return (base_x, base_y);
    }

    // Rectangle edges
    let left = rect_x;
    let right = rect_x + rect_w;
    let top = rect_y;
    let bottom = rect_y + rect_h;

    // Determine which edge the intersection is on and apply offset
    let tolerance = 1.0;
    let margin = 0.2;  // 20% margin on each side
    let usable = 1.0 - 2.0 * margin;

    // Calculate offset position (0.0 to 1.0 along the edge)
    let offset_ratio = if total_edges == 1 {
        0.5
    } else {
        margin + usable * (edge_idx as f64 / (total_edges - 1) as f64)
    };

    // Apply offset based on which edge the intersection is on
    if (base_x - left).abs() < tolerance {
        // Left edge - offset vertically
        let edge_y = top + rect_h * offset_ratio;
        (left, edge_y)
    } else if (base_x - right).abs() < tolerance {
        // Right edge - offset vertically
        let edge_y = top + rect_h * offset_ratio;
        (right, edge_y)
    } else if (base_y - top).abs() < tolerance {
        // Top edge - offset horizontally
        let edge_x = left + rect_w * offset_ratio;
        (edge_x, top)
    } else if (base_y - bottom).abs() < tolerance {
        // Bottom edge - offset horizontally
        let edge_x = left + rect_w * offset_ratio;
        (edge_x, bottom)
    } else {
        // Fallback to base intersection
        (base_x, base_y)
    }
}

/// SVG renderer for Structurizr diagrams.
pub struct SvgRenderer {
    width: u32,
    height: u32,
    #[allow(dead_code)]
    padding: u32,
    show_legend: bool,
}

impl Default for SvgRenderer {
    fn default() -> Self {
        Self {
            width: 2000,
            height: 1500,
            padding: 50,
            show_legend: false,
        }
    }
}

impl SvgRenderer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            padding: 50,
            show_legend: false,
        }
    }

    /// Enable or disable the legend in rendered diagrams.
    pub fn with_legend(mut self, show_legend: bool) -> Self {
        self.show_legend = show_legend;
        self
    }

    /// Enable the legend in rendered diagrams.
    pub fn enable_legend(&mut self) {
        self.show_legend = true;
    }

    /// Disable the legend in rendered diagrams.
    pub fn disable_legend(&mut self) {
        self.show_legend = false;
    }

    /// Render a system landscape view to SVG.
    pub fn render_system_landscape(
        &self,
        workspace: &Workspace,
        view: &SystemLandscapeView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        // Collect all elements to render
        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        // Include containers/components as "proxy candidates" so person→container relationships count
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        for system in &model.software_systems {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&system.id()) { continue; }
            }
            candidate_ids.insert(system.id());
            // Also add containers and components as proxy candidates
            // This allows person→container relationships to count for connectivity
            for container in &system.containers {
                candidate_ids.insert(container.id());
                for component in &container.components {
                    candidate_ids.insert(component.id());
                }
            }
        }

        // Step 2: Build connected_ids from relationships where BOTH endpoints are candidates
        let connected_ids: HashSet<ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        // Step 3: Add elements that are both candidates AND connected within this view
        // (Only add people and systems - containers/components were just proxy candidates)
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.push(person.id().to_string());
            elements_info.push(ElementInfo {
                id: person.id(),
                name: person.name().to_string(),
                description: person.properties.description.clone(),
                element_type: ElementType::Person,
                technology: None,
                tags: person.properties.tags.clone(),
            });
        }

        for system in &model.software_systems {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&system.id()) { continue; }
            }
            // System is connected if: itself is connected OR any of its containers/components are
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if !system_connected { continue; }
            element_ids.push(system.id().to_string());
            elements_info.push(ElementInfo {
                id: system.id(),
                name: system.name().to_string(),
                description: system.properties.description.clone(),
                element_type: ElementType::SoftwareSystem,
                technology: None,
                tags: system.properties.tags.clone(),
            });
        }

        // Build edges from relationships
        let edges: Vec<LayoutEdge> = model
            .relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        // Compute layout
        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &model.relationships, &model.groups, styles)
    }

    /// Render a system context view to SVG.
    pub fn render_system_context(
        &self,
        workspace: &Workspace,
        view: &SystemContextView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        // Find the main system
        let main_system = model
            .software_systems
            .iter()
            .find(|s| s.id() == view.software_system_id);

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        // Include containers/components as "proxy candidates" so person→container relationships count
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        for system in &model.software_systems {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&system.id()) { continue; }
            }
            candidate_ids.insert(system.id());
            // Also add containers and components as proxy candidates
            // This allows person→container relationships to count for connectivity
            for container in &system.containers {
                candidate_ids.insert(container.id());
                for component in &container.components {
                    candidate_ids.insert(component.id());
                }
            }
        }

        // Step 2: Build connected_ids from relationships where BOTH endpoints are candidates
        let connected_ids: HashSet<ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        // Step 3: Add elements that are both candidates AND connected within this view
        // (Only add people and systems - containers/components were just proxy candidates)
        // Add all people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.push(person.id().to_string());
            elements_info.push(ElementInfo {
                id: person.id(),
                name: person.name().to_string(),
                description: person.properties.description.clone(),
                element_type: ElementType::Person,
                technology: None,
                tags: person.properties.tags.clone(),
            });
        }

        // Add all software systems
        for system in &model.software_systems {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&system.id()) { continue; }
            }
            // System is connected if: itself is connected OR any of its containers/components are
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if !system_connected { continue; }
            element_ids.push(system.id().to_string());
            let is_main = main_system.map(|m| m.id() == system.id()).unwrap_or(false);
            elements_info.push(ElementInfo {
                id: system.id(),
                name: system.name().to_string(),
                description: system.properties.description.clone(),
                element_type: if is_main {
                    ElementType::SoftwareSystem
                } else {
                    ElementType::ExternalSystem
                },
                technology: None,
                tags: system.properties.tags.clone(),
            });
        }

        let edges: Vec<LayoutEdge> = model
            .relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        // The active element for SystemContextView is the main software system
        let active_element_id = main_system.map(|s| s.id());
        self.render_svg_with_active_element(&nodes, &elements_info, &model.relationships, &model.groups, styles, active_element_id)
    }

    /// Render a container view to SVG.
    pub fn render_container(
        &self,
        workspace: &Workspace,
        view: &ContainerView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        // Add candidate people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        // Add candidate containers from the target system
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            for container in &system.containers {
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&container.id()) { continue; }
                }
                candidate_ids.insert(container.id());
            }
        }

        // Add candidate external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&system.id()) { continue; }
                }
                candidate_ids.insert(system.id());
            }
        }

        // Step 2: Build connected_ids from relationships where BOTH endpoints are candidates
        let connected_ids: HashSet<ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.push(person.id().to_string());
            elements_info.push(ElementInfo {
                id: person.id(),
                name: person.name().to_string(),
                description: person.properties.description.clone(),
                element_type: ElementType::Person,
                technology: None,
                tags: person.properties.tags.clone(),
            });
        }

        // Add containers from the target system
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            for container in &system.containers {
                if !candidate_ids.contains(&container.id()) { continue; }
                if !connected_ids.contains(&container.id()) { continue; }
                element_ids.push(container.id().to_string());
                elements_info.push(ElementInfo {
                    id: container.id(),
                    name: container.name().to_string(),
                    description: container.properties.description.clone(),
                    element_type: ElementType::Container,
                    technology: container.technology.clone(),
                    tags: container.properties.tags.clone(),
                });
            }
        }

        // Add external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                if !candidate_ids.contains(&system.id()) { continue; }
                if !connected_ids.contains(&system.id()) { continue; }
                element_ids.push(system.id().to_string());
                elements_info.push(ElementInfo {
                    id: system.id(),
                    name: system.name().to_string(),
                    description: system.properties.description.clone(),
                    element_type: ElementType::ExternalSystem,
                    technology: None,
                    tags: system.properties.tags.clone(),
                });
            }
        }

        // Build set of element IDs for filtering relationships (from elements_info which has proper ElementId)
        let element_id_set: HashSet<ElementId> = elements_info.iter()
            .map(|e| e.id)
            .collect();

        // Filter relationships to only include those where BOTH endpoints are in the view
        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| element_id_set.contains(&r.source_id) && element_id_set.contains(&r.destination_id))
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        // The active element for ContainerView is the scoped software system
        let active_element_id = Some(view.software_system_id);
        self.render_svg_with_active_element(&nodes, &elements_info, &filtered_relationships, &model.groups, styles, active_element_id)
    }

    /// Render a component view to SVG.
    pub fn render_component(
        &self,
        workspace: &Workspace,
        view: &ComponentView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        // Find the container
        let mut target_container = None;

        for system in &model.software_systems {
            for container in &system.containers {
                if container.id() == view.container_id {
                    target_container = Some(container);
                    break;
                }
            }
            if target_container.is_some() {
                break;
            }
        }

        // For component views, include ALL components in the container
        // (unlike container views, we don't filter by connectivity - all components are relevant)
        if let Some(container) = target_container {
            for component in &container.components {
                // Only filter if view has explicit elements specified
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&component.id()) { continue; }
                }
                element_ids.push(component.id().to_string());
                elements_info.push(ElementInfo {
                    id: component.id(),
                    name: component.name().to_string(),
                    description: component.properties.description.clone(),
                    element_type: ElementType::Component,
                    technology: component.technology.clone(),
                    tags: component.properties.tags.clone(),
                });
            }
        }

        // Build set of element IDs for filtering relationships
        let element_id_set: HashSet<ElementId> = elements_info.iter()
            .map(|e| e.id)
            .collect();

        // Filter relationships to only include those where BOTH endpoints are in the view
        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| element_id_set.contains(&r.source_id) && element_id_set.contains(&r.destination_id))
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        // The active element for ComponentView is the scoped container
        let active_element_id = Some(view.container_id);
        self.render_svg_with_active_element(&nodes, &elements_info, &filtered_relationships, &model.groups, styles, active_element_id)
    }

    /// Render a deployment view to SVG.
    pub fn render_deployment(
        &self,
        workspace: &Workspace,
        view: &DeploymentView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Recursively collect deployment nodes
        fn collect_deployment_nodes(
            nodes: &[DeploymentNode],
            environment: &str,
            element_ids: &mut Vec<String>,
            elements_info: &mut Vec<ElementInfo>,
        ) {
            for node in nodes {
                // Check if this node matches the environment
                let matches_env = node.environment.as_ref()
                    .map(|e| e == environment)
                    .unwrap_or(true);

                if matches_env {
                    element_ids.push(node.id().to_string());
                    elements_info.push(ElementInfo {
                        id: node.id(),
                        name: node.name().to_string(),
                        description: node.properties.description.clone(),
                        element_type: ElementType::DeploymentNode,
                        technology: node.technology.clone(),
                        tags: node.properties.tags.clone(),
                    });

                    // Add infrastructure nodes
                    for infra in &node.infrastructure_nodes {
                        element_ids.push(infra.id().to_string());
                        elements_info.push(ElementInfo {
                            id: infra.id(),
                            name: infra.properties.name.clone(),
                            description: infra.properties.description.clone(),
                            element_type: ElementType::InfrastructureNode,
                            technology: infra.technology.clone(),
                            tags: infra.properties.tags.clone(),
                        });
                    }

                    // Recurse into children
                    collect_deployment_nodes(&node.children, environment, element_ids, elements_info);
                }
            }
        }

        collect_deployment_nodes(&model.deployment_nodes, &view.environment, &mut element_ids, &mut elements_info);

        let edges: Vec<LayoutEdge> = model
            .relationships
            .iter()
            .filter(|r| {
                element_ids.contains(&r.source_id.to_string()) ||
                element_ids.contains(&r.destination_id.to_string())
            })
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &model.relationships, &model.groups, styles)
    }

    /// Render a dynamic view to SVG.
    /// Dynamic views show interactions/sequences between elements.
    pub fn render_dynamic(
        &self,
        workspace: &Workspace,
        view: &DynamicView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();
        let style_resolver = StyleResolver::new(styles);

        // Collect unique elements from steps
        let mut element_ids: Vec<ElementId> = Vec::new();
        for step in &view.steps {
            if !element_ids.contains(&step.source_id) {
                element_ids.push(step.source_id);
            }
            if !element_ids.contains(&step.destination_id) {
                element_ids.push(step.destination_id);
            }
        }

        // Position elements horizontally (sequence diagram style)
        let element_width = 200.0;
        let element_height = 100.0;
        let element_spacing = 250.0;
        let start_x = 100.0;
        let start_y = 50.0;
        let step_spacing = 60.0;
        let step_start_y = start_y + element_height + 40.0;

        // Calculate content bounds
        let padding = self.padding as f64;
        let num_elements = element_ids.len().max(1);
        let num_steps = view.steps.len().max(1);

        let content_width = start_x + ((num_elements - 1) as f64 * element_spacing) + element_width + padding;
        let content_height = step_start_y + ((num_steps - 1) as f64 * step_spacing) + padding * 2.0;

        let bounds = ContentBounds {
            min_x: 0.0,
            min_y: 0.0,
            max_x: content_width.max(self.width as f64),
            max_y: content_height.max(400.0), // Minimum height for dynamic views
        };

        let view_width = bounds.width();
        let view_height = bounds.height();

        let mut svg = String::new();

        // SVG header with dynamic viewBox
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}" width="{:.0}" height="{:.0}">"#,
            bounds.to_viewbox(), view_width, view_height
        ));
        svg.push('\n');

        // Defs for markers - JointJS-compatible filled triangles
        let default_rel_style = style_resolver.resolve_relationship(&["Relationship".to_string()]);
        let marker_id = color_to_marker_id(&default_rel_style.color);
        svg.push_str(&format!(
            r##"  <defs>
    <marker id="{}" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto" markerUnits="strokeWidth">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="{}"/>
    </marker>
  </defs>
"##,
            marker_id, default_rel_style.color
        ));

        // Background - transparent to allow CSS theming
        svg.push_str(&format!(
            r##"  <rect width="{:.0}" height="{:.0}" fill="transparent"/>"##,
            view_width, view_height
        ));
        svg.push('\n');

        // Map element IDs to positions
        let mut element_positions: std::collections::HashMap<ElementId, (f64, f64)> = std::collections::HashMap::new();
        for (idx, id) in element_ids.iter().enumerate() {
            let x = start_x + (idx as f64) * element_spacing;
            element_positions.insert(*id, (x, start_y));
        }

        // Render elements as boxes at the top (wrapped in groups for JS targeting)
        for id in &element_ids {
            if let Some(&(x, y)) = element_positions.get(id) {
                // Find element info
                let (name, element_type, tags) = self.find_element_info(model, *id);
                let kind = element_type.to_element_kind();
                let resolved_style = style_resolver.resolve_element(&tags, kind);

                // Start element group with data-element-id for JavaScript targeting
                svg.push_str(&format!(
                    r#"  <g class="element-group" data-element-id="{}">"#,
                    id
                ));
                svg.push('\n');

                let bounds = Bounds::new(x, y, element_width, element_height);
                svg.push_str("    ");
                svg.push_str(&render_shape(resolved_style.shape, &bounds, &resolved_style));
                svg.push('\n');

                // Element name
                svg.push_str(&format!(
                    r#"    <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="{}">{}</text>"#,
                    x + element_width / 2.0, y + element_height / 2.0 + 5.0, resolved_style.color, escape_xml(&name)
                ));
                svg.push('\n');

                // Lifeline (vertical dashed line) - extend to bottom of content
                let lifeline_x = x + element_width / 2.0;
                let lifeline_start = y + element_height;
                let lifeline_end = view_height - padding;
                svg.push_str(&format!(
                    r##"    <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#999999" stroke-width="1" stroke-dasharray="5,5"/>"##,
                    lifeline_x, lifeline_start, lifeline_x, lifeline_end
                ));
                svg.push('\n');

                // Close element group
                svg.push_str("  </g>\n");
            }
        }

        // Render interaction arrows (steps) with data attributes for JS targeting
        for (idx, step) in view.steps.iter().enumerate() {
            if let (Some(&(src_x, _)), Some(&(dst_x, _))) =
                (element_positions.get(&step.source_id), element_positions.get(&step.destination_id))
            {
                let y = step_start_y + (idx as f64) * step_spacing;
                let src_center = src_x + element_width / 2.0;
                let dst_center = dst_x + element_width / 2.0;

                // Arrow line with data attributes for step index and element IDs
                svg.push_str(&format!(
                    r#"  <line data-step-index="{}" data-source-id="{}" data-dest-id="{}" x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="2" marker-end="url(#{})"/>"#,
                    idx, step.source_id, step.destination_id,
                    src_center, y, dst_center, y, default_rel_style.color, marker_id
                ));
                svg.push('\n');

                // Step number and description with data attribute
                let label_x = (src_center + dst_center) / 2.0;
                let label = format!("{}. {}", step.order, step.description.as_deref().unwrap_or(""));
                svg.push_str(&format!(
                    r#"  <text data-step-index="{}" x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="12" fill="{}">{}</text>"#,
                    idx, label_x, y - 8.0, default_rel_style.color, escape_xml(&label)
                ));
                svg.push('\n');

                // Technology (if available) - render below the arrow with padding
                let technology = model.relationships.iter()
                    .find(|r| r.source_id == step.source_id && r.destination_id == step.destination_id)
                    .and_then(|r| r.technology.as_ref());
                if let Some(tech) = technology {
                    svg.push_str(&format!(
                        r##"  <text data-step-index="{}" x="{:.0}" y="{:.0}" text-anchor="middle" font-family="monospace" font-size="10" fill="#888888">{}</text>"##,
                        idx, label_x, y + 18.0, escape_xml(tech)
                    ));
                    svg.push('\n');
                }
            }
        }

        svg.push_str("</svg>\n");

        Ok(svg)
    }

    /// Render a filtered view to SVG.
    /// This applies tag-based filtering to an existing view.
    pub fn render_filtered(
        &self,
        workspace: &Workspace,
        view: &FilteredView,
    ) -> Result<String> {
        // Find the base view and render it with filtering applied
        let views = workspace.views();

        // Try to find the base view in different view collections
        if let Some(base_view) = views.system_landscape_views.iter()
            .find(|v| v.properties.key == view.base_view_key)
        {
            return self.render_filtered_landscape(workspace, base_view, view);
        }

        if let Some(base_view) = views.system_context_views.iter()
            .find(|v| v.properties.key == view.base_view_key)
        {
            return self.render_filtered_context(workspace, base_view, view);
        }

        if let Some(base_view) = views.container_views.iter()
            .find(|v| v.properties.key == view.base_view_key)
        {
            return self.render_filtered_container(workspace, base_view, view);
        }

        // If base view not found, return an error
        Err(crate::error::Error::ViewNotFound(view.base_view_key.clone()))
    }

    /// Helper method to render a filtered system landscape view.
    fn render_filtered_landscape(
        &self,
        workspace: &Workspace,
        base_view: &SystemLandscapeView,
        filter: &FilteredView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        for person in &model.people {
            if self.should_include_element(&person.properties.tags, filter) {
                element_ids.push(person.id().to_string());
                elements_info.push(ElementInfo {
                    id: person.id(),
                    name: person.name().to_string(),
                    description: person.properties.description.clone(),
                    element_type: ElementType::Person,
                    technology: None,
                    tags: person.properties.tags.clone(),
                });
            }
        }

        for system in &model.software_systems {
            if self.should_include_element(&system.properties.tags, filter) {
                element_ids.push(system.id().to_string());
                elements_info.push(ElementInfo {
                    id: system.id(),
                    name: system.name().to_string(),
                    description: system.properties.description.clone(),
                    element_type: ElementType::SoftwareSystem,
                    technology: None,
                    tags: system.properties.tags.clone(),
                });
            }
        }

        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| self.should_include_relationship(&r.tags, filter))
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = base_view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from base view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&base_view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &filtered_relationships, &model.groups, styles)
    }

    /// Helper method to render a filtered system context view.
    fn render_filtered_context(
        &self,
        workspace: &Workspace,
        base_view: &SystemContextView,
        filter: &FilteredView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        let main_system = model
            .software_systems
            .iter()
            .find(|s| s.id() == base_view.software_system_id);

        for person in &model.people {
            if self.should_include_element(&person.properties.tags, filter) {
                element_ids.push(person.id().to_string());
                elements_info.push(ElementInfo {
                    id: person.id(),
                    name: person.name().to_string(),
                    description: person.properties.description.clone(),
                    element_type: ElementType::Person,
                    technology: None,
                    tags: person.properties.tags.clone(),
                });
            }
        }

        for system in &model.software_systems {
            if self.should_include_element(&system.properties.tags, filter) {
                let is_main = main_system.map(|m| m.id() == system.id()).unwrap_or(false);
                element_ids.push(system.id().to_string());
                elements_info.push(ElementInfo {
                    id: system.id(),
                    name: system.name().to_string(),
                    description: system.properties.description.clone(),
                    element_type: if is_main {
                        ElementType::SoftwareSystem
                    } else {
                        ElementType::ExternalSystem
                    },
                    technology: None,
                    tags: system.properties.tags.clone(),
                });
            }
        }

        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| self.should_include_relationship(&r.tags, filter))
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = base_view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from base view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&base_view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &filtered_relationships, &model.groups, styles)
    }

    /// Helper method to render a filtered container view.
    fn render_filtered_container(
        &self,
        workspace: &Workspace,
        base_view: &ContainerView,
        filter: &FilteredView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        for person in &model.people {
            if self.should_include_element(&person.properties.tags, filter) {
                element_ids.push(person.id().to_string());
                elements_info.push(ElementInfo {
                    id: person.id(),
                    name: person.name().to_string(),
                    description: person.properties.description.clone(),
                    element_type: ElementType::Person,
                    technology: None,
                    tags: person.properties.tags.clone(),
                });
            }
        }

        if let Some(system) = model.software_systems.iter().find(|s| s.id() == base_view.software_system_id) {
            for container in &system.containers {
                if self.should_include_element(&container.properties.tags, filter) {
                    element_ids.push(container.id().to_string());
                    elements_info.push(ElementInfo {
                        id: container.id(),
                        name: container.name().to_string(),
                        description: container.properties.description.clone(),
                        element_type: ElementType::Container,
                        technology: container.technology.clone(),
                        tags: container.properties.tags.clone(),
                    });
                }
            }
        }

        for system in &model.software_systems {
            if system.id() != base_view.software_system_id {
                if self.should_include_element(&system.properties.tags, filter) {
                    element_ids.push(system.id().to_string());
                    elements_info.push(ElementInfo {
                        id: system.id(),
                        name: system.name().to_string(),
                        description: system.properties.description.clone(),
                        element_type: ElementType::ExternalSystem,
                        technology: None,
                        tags: system.properties.tags.clone(),
                    });
                }
            }
        }

        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| self.should_include_relationship(&r.tags, filter))
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = base_view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply explicit positions from base view properties (e.g., from drag-and-drop)
        let explicit_positions = extract_explicit_positions(&base_view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &filtered_relationships, &model.groups, styles)
    }

    /// Check if an element should be included based on the filter.
    fn should_include_element(&self, element_tags: &[String], filter: &FilteredView) -> bool {
        let has_matching_tag = filter.tags.iter().any(|ft| element_tags.contains(ft));

        match filter.mode {
            FilterMode::Include => has_matching_tag || filter.tags.is_empty(),
            FilterMode::Exclude => !has_matching_tag,
        }
    }

    /// Check if a relationship should be included based on the filter.
    fn should_include_relationship(&self, rel_tags: &[String], filter: &FilteredView) -> bool {
        let has_matching_tag = filter.tags.iter().any(|ft| rel_tags.contains(ft));

        match filter.mode {
            FilterMode::Include => has_matching_tag || filter.tags.is_empty(),
            FilterMode::Exclude => !has_matching_tag,
        }
    }

    /// Render a custom view to SVG.
    /// Custom views show user-defined elements without type constraints.
    pub fn render_custom(
        &self,
        workspace: &Workspace,
        view: &CustomView,
    ) -> Result<String> {
        let model = workspace.model();
        let styles = workspace.styles();

        let mut element_ids: Vec<String> = Vec::new();
        let mut elements_info: Vec<ElementInfo> = Vec::new();

        // Custom views explicitly list the elements to include
        for element_view in &view.properties.elements {
            let element_id = element_view.id;

            // Find the element in the model
            if let Some((name, element_type, tags, technology)) = self.find_element_details(model, element_id) {
                element_ids.push(element_id.to_string());
                elements_info.push(ElementInfo {
                    id: element_id,
                    name,
                    description: None, // Custom views typically don't show descriptions
                    element_type,
                    technology,
                    tags,
                });
            }
        }

        // Filter relationships to only include those between visible elements
        let visible_ids: std::collections::HashSet<_> = element_ids.iter().collect();
        let filtered_relationships: Vec<_> = model
            .relationships
            .iter()
            .filter(|r| {
                visible_ids.contains(&r.source_id.to_string()) &&
                visible_ids.contains(&r.destination_id.to_string())
            })
            .cloned()
            .collect();

        let edges: Vec<LayoutEdge> = filtered_relationships
            .iter()
            .map(|r| LayoutEdge {
                source: r.source_id.to_string(),
                target: r.destination_id.to_string(),
            })
            .collect();

        let layout = if let Some(ref auto_layout) = view.properties.auto_layout {
            GridLayout::from_config(auto_layout)
        } else {
            GridLayout::default()
        };

        // Prepare node sizes for Sugiyama
        let node_sizes: Vec<(String, Size)> = element_ids.iter()
            .map(|id| (id.clone(), Size::default()))
            .collect();

        // Use Sugiyama with proper normalization
        let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

        // Convert sugiyama::LayoutNode to layout::LayoutNode
        let mut nodes: Vec<LayoutNode> = sugiyama_result.nodes
            .into_iter()
            .map(|n| LayoutNode {
                id: n.id,
                position: crate::layout::Position { x: n.position.x, y: n.position.y },
                size: Size { width: n.size.width, height: n.size.height },
                rank: n.rank,
            })
            .collect();

        // Apply any explicit positions from the view (from saved .positions.json)
        let explicit_positions = extract_explicit_positions(&view.properties);
        apply_explicit_positions(&mut nodes, &explicit_positions);

        self.render_svg(&nodes, &elements_info, &filtered_relationships, &model.groups, styles)
    }

    /// Render an image view to SVG.
    /// Image views embed external images (PNG, JPEG, SVG) in the diagram.
    pub fn render_image(
        &self,
        _workspace: &Workspace,
        view: &ImageView,
    ) -> Result<String> {
        let mut svg = String::new();

        // SVG header
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
            self.width, self.height, self.width, self.height
        ));
        svg.push('\n');

        // Background - transparent to allow CSS theming
        svg.push_str(&format!(
            r##"  <rect width="{}" height="{}" fill="transparent"/>"##,
            self.width, self.height
        ));
        svg.push('\n');

        // If there's content (base64 encoded image), embed it
        if let Some(ref content) = view.content {
            let content_type = view.content_type.as_deref().unwrap_or("image/png");

            // Determine if content is already a data URI or just base64
            let data_uri = if content.starts_with("data:") {
                content.clone()
            } else {
                format!("data:{};base64,{}", content_type, content)
            };

            // Center the image in the SVG
            let image_margin = 50.0;
            let image_x = image_margin;
            let image_y = image_margin;
            let image_width = self.width as f64 - (2.0 * image_margin);
            let image_height = self.height as f64 - (2.0 * image_margin);

            svg.push_str(&format!(
                r#"  <image x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" href="{}"/>"#,
                image_x, image_y, image_width, image_height, escape_xml(&data_uri)
            ));
            svg.push('\n');
        } else {
            // No content - show placeholder text
            svg.push_str(&format!(
                r##"  <text x="{}" y="{}" text-anchor="middle" font-family="Arial, sans-serif" font-size="24" fill="#999999">No image content</text>"##,
                self.width / 2, self.height / 2
            ));
            svg.push('\n');
        }

        // Add title if present
        if let Some(ref title) = view.properties.title {
            svg.push_str(&format!(
                r##"  <text x="{}" y="30" text-anchor="middle" font-family="Arial, sans-serif" font-size="18" font-weight="bold" fill="#333333">{}</text>"##,
                self.width / 2, escape_xml(title)
            ));
            svg.push('\n');
        }

        svg.push_str("</svg>\n");

        Ok(svg)
    }

    /// Find detailed element information including technology.
    fn find_element_details(&self, model: &structurizr_core::model::Model, id: ElementId) -> Option<(String, ElementType, Vec<String>, Option<String>)> {
        // Check people
        for person in &model.people {
            if person.id() == id {
                return Some((
                    person.name().to_string(),
                    ElementType::Person,
                    person.properties.tags.clone(),
                    None,
                ));
            }
        }

        // Check software systems and their children
        for system in &model.software_systems {
            if system.id() == id {
                return Some((
                    system.name().to_string(),
                    ElementType::SoftwareSystem,
                    system.properties.tags.clone(),
                    None,
                ));
            }

            for container in &system.containers {
                if container.id() == id {
                    return Some((
                        container.name().to_string(),
                        ElementType::Container,
                        container.properties.tags.clone(),
                        container.technology.clone(),
                    ));
                }

                for component in &container.components {
                    if component.id() == id {
                        return Some((
                            component.name().to_string(),
                            ElementType::Component,
                            component.properties.tags.clone(),
                            component.technology.clone(),
                        ));
                    }
                }
            }
        }

        // Check deployment nodes
        fn search_deployment_nodes(nodes: &[DeploymentNode], id: ElementId) -> Option<(String, ElementType, Vec<String>, Option<String>)> {
            for node in nodes {
                if node.id() == id {
                    return Some((
                        node.name().to_string(),
                        ElementType::DeploymentNode,
                        node.properties.tags.clone(),
                        node.technology.clone(),
                    ));
                }

                // Check infrastructure nodes
                for infra in &node.infrastructure_nodes {
                    if infra.id() == id {
                        return Some((
                            infra.properties.name.clone(),
                            ElementType::InfrastructureNode,
                            infra.properties.tags.clone(),
                            infra.technology.clone(),
                        ));
                    }
                }

                // Recurse into children
                if let Some(result) = search_deployment_nodes(&node.children, id) {
                    return Some(result);
                }
            }
            None
        }

        search_deployment_nodes(&model.deployment_nodes, id)
    }

    /// Find element information by ID.
    fn find_element_info(&self, model: &structurizr_core::model::Model, id: ElementId) -> (String, ElementType, Vec<String>) {
        // Check people
        for person in &model.people {
            if person.id() == id {
                return (person.name().to_string(), ElementType::Person, person.properties.tags.clone());
            }
        }

        // Check software systems and containers
        for system in &model.software_systems {
            if system.id() == id {
                return (system.name().to_string(), ElementType::SoftwareSystem, system.properties.tags.clone());
            }
            for container in &system.containers {
                if container.id() == id {
                    return (container.name().to_string(), ElementType::Container, container.properties.tags.clone());
                }
                for component in &container.components {
                    if component.id() == id {
                        return (component.name().to_string(), ElementType::Component, component.properties.tags.clone());
                    }
                }
            }
        }

        // Default
        ("Unknown".to_string(), ElementType::SoftwareSystem, vec![])
    }

    fn render_svg(
        &self,
        nodes: &[LayoutNode],
        elements: &[ElementInfo],
        relationships: &[structurizr_core::model::Relationship],
        groups: &[structurizr_core::model::Group],
        styles: Option<&Styles>,
    ) -> Result<String> {
        self.render_svg_with_options(nodes, elements, relationships, groups, styles, self.show_legend, None)
    }

    fn render_svg_with_active_element(
        &self,
        nodes: &[LayoutNode],
        elements: &[ElementInfo],
        relationships: &[structurizr_core::model::Relationship],
        groups: &[structurizr_core::model::Group],
        styles: Option<&Styles>,
        active_element_id: Option<ElementId>,
    ) -> Result<String> {
        self.render_svg_with_options(nodes, elements, relationships, groups, styles, self.show_legend, active_element_id)
    }

    fn render_svg_with_options(
        &self,
        nodes: &[LayoutNode],
        elements: &[ElementInfo],
        relationships: &[structurizr_core::model::Relationship],
        groups: &[structurizr_core::model::Group],
        styles: Option<&Styles>,
        show_legend: bool,
        active_element_id: Option<ElementId>,
    ) -> Result<String> {
        let mut svg = String::new();
        let style_resolver = StyleResolver::new(styles);

        // Build a map of element ID to position
        let node_map: std::collections::HashMap<_, _> = nodes
            .iter()
            .map(|n| (n.id.clone(), n))
            .collect();

        // Calculate content bounds from all nodes
        let mut bounds = ContentBounds::new();
        for node in nodes {
            bounds.expand(
                node.position.x,
                node.position.y,
                node.size.width,
                node.size.height,
            );
        }

        // Add padding and ensure minimum size
        let padding = self.padding as f64;
        let mut bounds = if bounds.is_valid() {
            bounds.with_padding(padding)
        } else {
            // Fallback to default dimensions if no nodes
            ContentBounds {
                min_x: 0.0,
                min_y: 0.0,
                max_x: self.width as f64,
                max_y: self.height as f64,
            }
        };

        // Use calculated bounds for viewBox, ensuring minimum dimensions
        let view_width = bounds.width().max(100.0);
        let view_height = bounds.height().max(100.0);

        // SVG header with dynamic viewBox and ARIA attributes for accessibility
        // role="img" indicates this is an image, aria-label provides description for screen readers
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{}" width="{:.0}" height="{:.0}" role="img" aria-label="Architecture diagram showing software elements and their relationships. Bold lines indicate outbound relationships from the primary element.">"#,
            bounds.to_viewbox(), view_width, view_height
        ));
        svg.push('\n');

        // Collect all unique relationship colors for arrow markers
        let mut marker_colors: std::collections::HashSet<String> = std::collections::HashSet::new();
        let default_rel_style = style_resolver.resolve_relationship(&["Relationship".to_string()]);
        marker_colors.insert(default_rel_style.color.clone());

        for rel in relationships {
            let rel_style = style_resolver.resolve_relationship(&rel.tags);
            marker_colors.insert(rel_style.color.clone());
        }

        // Defs for markers (arrows) - JointJS-compatible filled triangles
        // Create a marker for each unique color used in relationships
        svg.push_str("  <defs>\n");
        for color in &marker_colors {
            let marker_id = color_to_marker_id(color);
            // JointJS-style arrow: filled triangle, cleaner design
            // Dimensions: 10x10, refX=9 positions arrow tip at line end
            svg.push_str(&format!(
                r##"    <marker id="{}" markerWidth="10" markerHeight="10" refX="9" refY="5" orient="auto" markerUnits="strokeWidth">
      <path d="M 0 0 L 10 5 L 0 10 z" fill="{}"/>
    </marker>
"##,
                marker_id, color
            ));

            // Bold version of arrow marker for outbound relationships (1.5x size)
            let bold_marker_id = format!("arrow-bold-{}", color.trim_start_matches('#'));
            svg.push_str(&format!(
                r##"    <marker id="{}" markerWidth="15" markerHeight="15" refX="13" refY="7.5" orient="auto" markerUnits="strokeWidth">
      <path d="M 0 0 L 15 7.5 L 0 15 z" fill="{}"/>
    </marker>
"##,
                bold_marker_id, color
            ));
        }
        svg.push_str("  </defs>\n");

        // Background - transparent to allow CSS theming (covers the full viewBox area)
        svg.push_str(&format!(
            r##"  <rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="transparent"/>"##,
            bounds.min_x, bounds.min_y, view_width, view_height
        ));
        svg.push('\n');

        // Render groups first (so they appear behind everything)
        self.render_groups(&mut svg, &node_map, groups);

        // Collect relationship line data and labels for collision-aware rendering
        struct RelLineData {
            x1: f64,
            y1: f64,
            x2: f64,
            y2: f64,
            source_id: String,
            target_id: String,
            description: Option<String>,
            technology: Option<String>,
            font_size: u32,
            color: String,
            is_orthogonal: bool,  // Track if path uses orthogonal routing (for horizontal labels)
            is_outbound_from_active: bool,  // Track if relationship is outbound from the active/scope element
        }

        // Bold styling constants for outbound relationships
        const OUTBOUND_STROKE_MULTIPLIER: f64 = 2.0;  // 2x thickness for bold
        const OUTBOUND_OPACITY: f64 = 1.0;  // Full opacity for outbound
        const NORMAL_OPACITY: f64 = 0.6;  // Reduced opacity for other relationships

        let mut rel_lines: Vec<RelLineData> = Vec::new();

        // Create orthogonal router for use when needed
        let orthogonal_config = OrthogonalConfig::for_direction(
            structurizr_core::view::AutoLayoutDirection::TopBottom
        );
        let orthogonal_router = OrthogonalRouter::new(orthogonal_config);

        // Pre-compute edge indices for port distribution
        // This allows multiple connectors to the same node to attach at different points
        let mut edges_by_source: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
        let mut edges_by_target: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for rel in relationships {
            let src = rel.source_id.to_string();
            let tgt = rel.destination_id.to_string();
            // Only count edges where both nodes are in the view
            if node_map.contains_key(&src) && node_map.contains_key(&tgt) {
                edges_by_source.entry(src.clone()).or_default().push(tgt.clone());
                edges_by_target.entry(tgt).or_default().push(src);
            }
        }

        // First pass: collect relationship data and render lines/paths
        for rel in relationships {
            let source_id = rel.source_id.to_string();
            let target_id = rel.destination_id.to_string();

            if let (Some(source_node), Some(target_node)) =
                (node_map.get(&source_id), node_map.get(&target_id))
            {
                // Resolve relationship style
                let rel_style = style_resolver.resolve_relationship(&rel.tags);

                // Check if this relationship is outbound from the active/scope element
                let is_outbound_from_active = active_element_id
                    .map(|active_id| rel.source_id == active_id)
                    .unwrap_or(false);

                // Calculate effective stroke width and opacity based on outbound status
                let effective_thickness = if is_outbound_from_active {
                    (rel_style.thickness as f64 * OUTBOUND_STROKE_MULTIPLIER) as u32
                } else {
                    rel_style.thickness
                };
                let effective_opacity = if is_outbound_from_active {
                    OUTBOUND_OPACITY
                } else if active_element_id.is_some() {
                    NORMAL_OPACITY  // Dim non-outbound relationships when there's an active element
                } else {
                    rel_style.opacity as f64 / 100.0
                };

                // CSS class for dynamic highlighting (Phase 2)
                let css_class = if is_outbound_from_active {
                    "relationship relationship-outbound"
                } else {
                    "relationship"
                };

                // Check which routing style is requested
                let use_orthogonal = matches!(rel_style.routing, Routing::Orthogonal);
                let use_curved = matches!(rel_style.routing, Routing::Curved);

                // Get edge indices for proper port distribution
                let src_edges = edges_by_source.get(&source_id);
                let tgt_edges = edges_by_target.get(&target_id);
                let edge_idx_at_source = src_edges
                    .and_then(|edges| edges.iter().position(|t| t == &target_id))
                    .unwrap_or(0);
                let edge_idx_at_target = tgt_edges
                    .and_then(|edges| edges.iter().position(|s| s == &source_id))
                    .unwrap_or(0);
                let total_source_edges = src_edges.map(|e| e.len()).unwrap_or(1);
                let total_target_edges = tgt_edges.map(|e| e.len()).unwrap_or(1);

                let (x1, y1, x2, y2) = if use_orthogonal {
                    // Orthogonal routing - use the router with proper port distribution
                    let path = orthogonal_router.route_edge(
                        source_node, target_node,
                        edge_idx_at_source, total_source_edges,
                        edge_idx_at_target, total_target_edges,
                    );

                    // Get waypoints from the path
                    if let EdgePath::Orthogonal { waypoints } = &path {
                        // Build SVG path data
                        let path_data = path.to_svg_path();

                        // Build ARIA label for accessibility
                        let aria_label = if is_outbound_from_active {
                            format!("Outbound relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                        } else {
                            format!("Relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                        };

                        // Build path attributes with bold styling for outbound relationships
                        let mut path_attrs = format!(
                            r#"d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{:.2}" class="{}" data-source="{}" data-target="{}" data-routing="orthogonal" role="graphics-symbol" aria-label="{}""#,
                            path_data, rel_style.color, effective_thickness, effective_opacity, css_class, source_id, target_id, escape_xml(&aria_label)
                        );

                        // Add dash pattern if needed
                        if let Some(dasharray) = rel_style.stroke_dasharray() {
                            path_attrs.push_str(&format!(r#" stroke-dasharray="{}""#, dasharray));
                        }

                        // Draw path with color-matched arrow marker
                        // Use larger arrow marker for outbound relationships
                        let marker_id = if is_outbound_from_active {
                            format!("arrow-bold-{}", color_to_marker_id(&rel_style.color).trim_start_matches("arrow-"))
                        } else {
                            color_to_marker_id(&rel_style.color)
                        };
                        svg.push_str(&format!(
                            r##"  <path {} marker-end="url(#{})"/>"##,
                            path_attrs, marker_id
                        ));
                        svg.push('\n');

                        // Get label position from path (reserved for future label positioning)
                        let (_label_x, _label_y, _angle) = path.label_position();

                        // Use first and last waypoints for label data
                        let start = waypoints.first().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
                        let end = waypoints.last().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));

                        // Expand bounds to include all waypoints
                        for wp in waypoints {
                            bounds.expand_point(wp.x, wp.y);
                        }

                        (start.0, start.1, end.0, end.1)
                    } else {
                        // Fallback to direct if somehow we got a different path type
                        let src_cx = source_node.position.x + source_node.size.width / 2.0;
                        let src_cy = source_node.position.y + source_node.size.height / 2.0;
                        let tgt_cx = target_node.position.x + target_node.size.width / 2.0;
                        let tgt_cy = target_node.position.y + target_node.size.height / 2.0;
                        (src_cx, src_cy, tgt_cx, tgt_cy)
                    }
                } else if use_curved {
                    // Curved routing - smooth Bezier curves
                    let path = route_curved(
                        source_node, target_node,
                        edge_idx_at_source, total_source_edges,
                        edge_idx_at_target, total_target_edges,
                    );

                    // Get control points from the path
                    if let EdgePath::Curved { control_points } = &path {
                        // Build SVG path data
                        let path_data = path.to_svg_path();

                        // Build ARIA label for accessibility
                        let aria_label = if is_outbound_from_active {
                            format!("Outbound relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                        } else {
                            format!("Relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                        };

                        // Build path attributes with bold styling for outbound relationships
                        let mut path_attrs = format!(
                            r#"d="{}" fill="none" stroke="{}" stroke-width="{}" opacity="{:.2}" class="{}" data-source="{}" data-target="{}" data-routing="curved" role="graphics-symbol" aria-label="{}""#,
                            path_data, rel_style.color, effective_thickness, effective_opacity, css_class, source_id, target_id, escape_xml(&aria_label)
                        );

                        // Add dash pattern if needed
                        if let Some(dasharray) = rel_style.stroke_dasharray() {
                            path_attrs.push_str(&format!(r#" stroke-dasharray="{}""#, dasharray));
                        }

                        // Draw path with color-matched arrow marker
                        // Use larger arrow marker for outbound relationships
                        let marker_id = if is_outbound_from_active {
                            format!("arrow-bold-{}", color_to_marker_id(&rel_style.color).trim_start_matches("arrow-"))
                        } else {
                            color_to_marker_id(&rel_style.color)
                        };
                        svg.push_str(&format!(
                            r##"  <path {} marker-end="url(#{})"/>"##,
                            path_attrs, marker_id
                        ));
                        svg.push('\n');

                        // Use first and last control points for label data
                        let start = control_points.first().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));
                        let end = control_points.last().map(|p| (p.x, p.y)).unwrap_or((0.0, 0.0));

                        // Expand bounds to include all control points
                        for cp in control_points {
                            bounds.expand_point(cp.x, cp.y);
                        }

                        (start.0, start.1, end.0, end.1)
                    } else {
                        // Fallback to direct if somehow we got a different path type
                        let src_cx = source_node.position.x + source_node.size.width / 2.0;
                        let src_cy = source_node.position.y + source_node.size.height / 2.0;
                        let tgt_cx = target_node.position.x + target_node.size.width / 2.0;
                        let tgt_cy = target_node.position.y + target_node.size.height / 2.0;
                        (src_cx, src_cy, tgt_cx, tgt_cy)
                    }
                } else {
                    // Direct routing (default) - with edge distribution
                    // Calculate center points
                    let src_cx = source_node.position.x + source_node.size.width / 2.0;
                    let src_cy = source_node.position.y + source_node.size.height / 2.0;
                    let tgt_cx = target_node.position.x + target_node.size.width / 2.0;
                    let tgt_cy = target_node.position.y + target_node.size.height / 2.0;

                    // Calculate intersection points with element boundaries
                    // Use offset version to distribute multiple edges along the boundary
                    let (x1, y1) = line_rect_intersection_with_offset(
                        tgt_cx, tgt_cy, src_cx, src_cy,
                        source_node.position.x, source_node.position.y,
                        source_node.size.width, source_node.size.height,
                        edge_idx_at_source, total_source_edges,
                    );
                    let (x2, y2) = line_rect_intersection_with_offset(
                        src_cx, src_cy, tgt_cx, tgt_cy,
                        target_node.position.x, target_node.position.y,
                        target_node.size.width, target_node.size.height,
                        edge_idx_at_target, total_target_edges,
                    );

                    // Build ARIA label for accessibility
                    let aria_label = if is_outbound_from_active {
                        format!("Outbound relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                    } else {
                        format!("Relationship: {}", rel.description.as_deref().unwrap_or("connected to"))
                    };

                    // Build line attributes with bold styling for outbound relationships
                    let mut line_attrs = format!(
                        r#"x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="{}" opacity="{:.2}" class="{}" data-source="{}" data-target="{}" data-routing="direct" role="graphics-symbol" aria-label="{}""#,
                        x1, y1, x2, y2, rel_style.color, effective_thickness, effective_opacity, css_class, source_id, target_id, escape_xml(&aria_label)
                    );

                    // Add dash pattern if needed
                    if let Some(dasharray) = rel_style.stroke_dasharray() {
                        line_attrs.push_str(&format!(r#" stroke-dasharray="{}""#, dasharray));
                    }

                    // Draw line with color-matched arrow marker
                    // Use larger arrow marker for outbound relationships
                    let marker_id = if is_outbound_from_active {
                        format!("arrow-bold-{}", color_to_marker_id(&rel_style.color).trim_start_matches("arrow-"))
                    } else {
                        color_to_marker_id(&rel_style.color)
                    };
                    svg.push_str(&format!(
                        r##"  <line {} marker-end="url(#{})"/>"##,
                        line_attrs, marker_id
                    ));
                    svg.push('\n');

                    (x1, y1, x2, y2)
                };

                // Store data for label rendering
                // Both orthogonal and curved paths use horizontal labels for better readability
                rel_lines.push(RelLineData {
                    x1,
                    y1,
                    x2,
                    y2,
                    source_id: source_id.clone(),
                    target_id: target_id.clone(),
                    description: rel.description.clone(),
                    technology: rel.technology.clone(),
                    font_size: rel_style.font_size,
                    color: rel_style.color.clone(),
                    is_orthogonal: use_orthogonal || use_curved,
                    is_outbound_from_active,
                });
            }
        }

        // Second pass: render labels with collision avoidance
        let mut placed_labels: Vec<TextBounds> = Vec::new();

        for rel_data in &rel_lines {
            // Calculate the angle for this relationship line
            // Orthogonal paths use horizontal (0°) labels for better readability
            let angle = if rel_data.is_orthogonal {
                0.0
            } else {
                calculate_line_angle(rel_data.x1, rel_data.y1, rel_data.x2, rel_data.y2)
            };

            // Render description label with collision avoidance
            if let Some(ref desc) = rel_data.description {
                let label = LabelPlacement {
                    text: desc.clone(),
                    line_start: (rel_data.x1, rel_data.y1),
                    line_end: (rel_data.x2, rel_data.y2),
                    font_size: rel_data.font_size,
                    is_technology: false,
                };

                let (label_x, label_y) = find_non_overlapping_position(&label, &placed_labels);

                // Use bold font weight for outbound relationship labels to match bold lines
                let font_weight = if rel_data.is_outbound_from_active { "bold" } else { "normal" };

                svg.push_str(&format!(
                    r#"  <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" font-weight="{}" fill="{}" transform="rotate({:.1}, {:.0}, {:.0})" class="relationship-label" data-source="{}" data-target="{}" data-label-type="description">{}</text>"#,
                    label_x, label_y, rel_data.font_size, font_weight, rel_data.color, angle, label_x, label_y, rel_data.source_id, rel_data.target_id, escape_xml(desc)
                ));
                svg.push('\n');

                // Record this label's bounds
                let text_width = TextBounds::estimate_width(desc, rel_data.font_size);
                let text_height = rel_data.font_size as f64 * 1.4;
                placed_labels.push(TextBounds::new(label_x, label_y, text_width, text_height));
            }

            // Render technology label with collision avoidance
            if let Some(ref tech) = rel_data.technology {
                let tech_text = format!("[{}]", tech);
                let tech_font_size = rel_data.font_size.saturating_sub(2);

                let label = LabelPlacement {
                    text: tech_text.clone(),
                    line_start: (rel_data.x1, rel_data.y1),
                    line_end: (rel_data.x2, rel_data.y2),
                    font_size: tech_font_size,
                    is_technology: true,
                };

                let (label_x, label_y) = find_non_overlapping_position(&label, &placed_labels);

                // Use bold font weight for outbound relationship labels to match bold lines
                let font_weight = if rel_data.is_outbound_from_active { "bold" } else { "normal" };

                svg.push_str(&format!(
                    r#"  <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" font-weight="{}" fill="{}" transform="rotate({:.1}, {:.0}, {:.0})" class="relationship-label" data-source="{}" data-target="{}" data-label-type="technology">{}</text>"#,
                    label_x, label_y, tech_font_size, font_weight, rel_data.color, angle, label_x, label_y, rel_data.source_id, rel_data.target_id, escape_xml(&tech_text)
                ));
                svg.push('\n');

                // Record this label's bounds
                let text_width = TextBounds::estimate_width(&tech_text, tech_font_size);
                let text_height = tech_font_size as f64 * 1.4;
                placed_labels.push(TextBounds::new(label_x, label_y, text_width, text_height));
            }
        }

        // Render elements
        for element in elements {
            if let Some(node) = node_map.get(&element.id.to_string()) {
                self.render_element(&mut svg, node, element, &style_resolver);
            }
        }

        // Render legend if requested
        if show_legend {
            svg.push_str(&self.render_legend(elements, relationships, &style_resolver, &bounds));
        }

        svg.push_str("</svg>\n");

        Ok(svg)
    }

    /// Render a legend showing unique element types and relationship styles.
    fn render_legend(
        &self,
        elements: &[ElementInfo],
        relationships: &[structurizr_core::model::Relationship],
        style_resolver: &StyleResolver,
        bounds: &ContentBounds,
    ) -> String {
        // Collect unique element types
        let mut unique_elements: Vec<LegendElementEntry> = Vec::new();
        let mut seen_element_types: std::collections::HashSet<String> = std::collections::HashSet::new();

        for element in elements {
            let key = format!("{:?}", element.element_type);
            if !seen_element_types.contains(&key) {
                seen_element_types.insert(key);
                let element_kind = element.element_type.to_element_kind();
                let resolved_style = style_resolver.resolve_element(&element.tags, element_kind);

                let label = match element.element_type {
                    ElementType::Person => "Person",
                    ElementType::SoftwareSystem => "Software System",
                    ElementType::Container => "Container",
                    ElementType::Component => "Component",
                    ElementType::ExternalSystem => "External System",
                    ElementType::DeploymentNode => "Deployment Node",
                    ElementType::InfrastructureNode => "Infrastructure Node",
                };

                unique_elements.push(LegendElementEntry {
                    label: label.to_string(),
                    shape: resolved_style.shape,
                    background: resolved_style.background.clone(),
                    stroke: resolved_style.stroke.clone(),
                    stroke_width: resolved_style.stroke_width,
                    border: resolved_style.border,
                });
            }
        }

        // Collect unique relationship styles
        let mut unique_relationships: Vec<LegendRelationshipEntry> = Vec::new();
        let mut seen_rel_styles: std::collections::HashSet<String> = std::collections::HashSet::new();

        for rel in relationships {
            let resolved_style = style_resolver.resolve_relationship(&rel.tags);

            // Create a unique key based on visual properties
            let key = format!(
                "{:?}_{}_{}",
                resolved_style.style,
                resolved_style.color,
                resolved_style.thickness
            );

            if !seen_rel_styles.contains(&key) {
                seen_rel_styles.insert(key);

                // Determine label from tags (skip generic "Relationship" tag)
                let label = rel.tags
                    .iter()
                    .find(|t| *t != "Relationship")
                    .map(|s| s.as_str())
                    .unwrap_or(match resolved_style.style {
                        structurizr_core::style::LineStyle::Solid => "Relationship",
                        structurizr_core::style::LineStyle::Dashed => "Async Relationship",
                        structurizr_core::style::LineStyle::Dotted => "Dotted Relationship",
                    });

                unique_relationships.push(LegendRelationshipEntry {
                    label: label.to_string(),
                    style: resolved_style.style,
                    color: resolved_style.color.clone(),
                    thickness: resolved_style.thickness,
                });
            }
        }

        // If nothing to show, return empty
        if unique_elements.is_empty() && unique_relationships.is_empty() {
            return String::new();
        }

        // Calculate legend dimensions
        let legend_padding = 15.0;
        let legend_item_height = 30.0;
        let legend_icon_size = 20.0;
        let legend_text_offset = legend_icon_size + 10.0;

        // +1 for the visual encoding section (bold outbound line explanation)
        // +45 for extra spacing: title(30) + separator(15 gap + line)
        let num_items = unique_elements.len() + unique_relationships.len() + 1;
        let legend_height = (num_items as f64 * legend_item_height) + (legend_padding * 2.0) + 45.0;
        let legend_width = 250.0;

        let legend_x = bounds.min_x + 20.0;
        let legend_y = bounds.max_y - legend_height - 20.0;

        let mut legend_svg = String::new();

        // Legend background and border
        legend_svg.push_str(&format!(
            r##"  <rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="#ffffff" stroke="#cccccc" stroke-width="2" rx="5"/>"##,
            legend_x, legend_y, legend_width, legend_height
        ));
        legend_svg.push('\n');

        // Legend title
        legend_svg.push_str(&format!(
            r##"  <text x="{:.0}" y="{:.0}" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#333333">Legend</text>"##,
            legend_x + legend_padding, legend_y + legend_padding + 12.0
        ));
        legend_svg.push('\n');

        // Separator line
        legend_svg.push_str(&format!(
            r##"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#cccccc" stroke-width="1"/>"##,
            legend_x + legend_padding, legend_y + legend_padding + 20.0,
            legend_x + legend_width - legend_padding, legend_y + legend_padding + 20.0
        ));
        legend_svg.push('\n');

        let mut current_y = legend_y + legend_padding + 30.0;

        // Render element type entries
        for entry in &unique_elements {
            let icon_x = legend_x + legend_padding;
            let icon_y = current_y + 5.0;

            // Render small icon for this element type
            let icon_bounds = Bounds::new(icon_x, icon_y, legend_icon_size, legend_icon_size);
            let icon_style = crate::style_resolver::ResolvedElementStyle {
                shape: entry.shape,
                background: entry.background.clone(),
                color: "#ffffff".to_string(),
                stroke: entry.stroke.clone(),
                stroke_width: entry.stroke_width,
                font_size: 8,
                border: entry.border,
                width: legend_icon_size as u32,
                height: legend_icon_size as u32,
                opacity: 100,
                icon: None,
                icon_position: structurizr_core::style::IconPosition::Top,
                show_metadata: false,
                show_description: false,
            };

            legend_svg.push_str("  ");
            legend_svg.push_str(&render_shape(entry.shape, &icon_bounds, &icon_style));
            legend_svg.push('\n');

            // Render label
            legend_svg.push_str(&format!(
                r##"  <text x="{:.0}" y="{:.0}" font-family="Arial, sans-serif" font-size="12" fill="#333333">{}</text>"##,
                icon_x + legend_text_offset, current_y + 17.0, escape_xml(&entry.label)
            ));
            legend_svg.push('\n');

            current_y += legend_item_height;
        }

        // Render relationship entries
        for entry in &unique_relationships {
            let line_x1 = legend_x + legend_padding;
            let line_x2 = line_x1 + legend_icon_size;
            let line_y = current_y + legend_icon_size / 2.0 + 5.0;

            // Render line sample
            let mut line_attrs = format!(
                r##"stroke="{}" stroke-width="{}""##,
                entry.color, entry.thickness
            );

            if let Some(dasharray) = match entry.style {
                structurizr_core::style::LineStyle::Solid => None,
                structurizr_core::style::LineStyle::Dashed => Some("5,5"),  // JointJS standard
                structurizr_core::style::LineStyle::Dotted => Some("2,2"),
            } {
                line_attrs.push_str(&format!(r##" stroke-dasharray="{}""##, dasharray));
            }

            // Use color-matched marker for legend
            let marker_id = color_to_marker_id(&entry.color);
            legend_svg.push_str(&format!(
                r##"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" {} marker-end="url(#{})"/>"##,
                line_x1, line_y, line_x2, line_y, line_attrs, marker_id
            ));
            legend_svg.push('\n');

            // Render label
            legend_svg.push_str(&format!(
                r##"  <text x="{:.0}" y="{:.0}" font-family="Arial, sans-serif" font-size="12" fill="#333333">{}</text>"##,
                line_x1 + legend_text_offset, current_y + 17.0, escape_xml(&entry.label)
            ));
            legend_svg.push('\n');

            current_y += legend_item_height;
        }

        // Add visual encoding explanation for bold outbound relationships
        // This helps users understand that bold lines = outbound from primary element
        current_y += 5.0; // Add a small gap before the encoding section

        // Separator line before encoding section
        legend_svg.push_str(&format!(
            r##"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="#cccccc" stroke-width="1"/>"##,
            legend_x + legend_padding, current_y,
            legend_x + legend_width - legend_padding, current_y
        ));
        legend_svg.push('\n');
        current_y += 10.0;

        // Bold line example (outbound relationships)
        let line_x1 = legend_x + legend_padding;
        let line_x2 = line_x1 + legend_icon_size;
        let line_y = current_y + legend_icon_size / 2.0;

        // Use first relationship color or default for the bold example
        let example_color = unique_relationships.first()
            .map(|r| r.color.as_str())
            .unwrap_or("#707070");
        let bold_marker_id = format!("arrow-bold-{}", example_color.trim_start_matches('#'));

        legend_svg.push_str(&format!(
            r##"  <line x1="{:.0}" y1="{:.0}" x2="{:.0}" y2="{:.0}" stroke="{}" stroke-width="4" marker-end="url(#{})" role="graphics-symbol" aria-label="Bold line indicating outbound relationship from primary element"/>"##,
            line_x1, line_y, line_x2, line_y, example_color, bold_marker_id
        ));
        legend_svg.push('\n');

        // Label for bold lines
        legend_svg.push_str(&format!(
            r##"  <text x="{:.0}" y="{:.0}" font-family="Arial, sans-serif" font-size="12" fill="#333333">Outbound (from focus)</text>"##,
            line_x1 + legend_text_offset, current_y + 14.0
        ));
        legend_svg.push('\n');

        legend_svg
    }

    /// Render groups as dashed rectangles behind grouped elements.
    fn render_groups(
        &self,
        svg: &mut String,
        node_map: &std::collections::HashMap<String, &LayoutNode>,
        groups: &[structurizr_core::model::Group],
    ) {
        for group in groups {
            // Skip empty groups
            if group.element_ids.is_empty() {
                continue;
            }

            // Calculate bounding box for all elements in the group
            let mut min_x = f64::MAX;
            let mut min_y = f64::MAX;
            let mut max_x = f64::MIN;
            let mut max_y = f64::MIN;

            let mut found_any = false;
            for element_id in &group.element_ids {
                if let Some(node) = node_map.get(&element_id.to_string()) {
                    found_any = true;
                    min_x = min_x.min(node.position.x);
                    min_y = min_y.min(node.position.y);
                    max_x = max_x.max(node.position.x + node.size.width);
                    max_y = max_y.max(node.position.y + node.size.height);
                }
            }

            // Skip if no elements were found in the layout
            if !found_any {
                continue;
            }

            // Add padding around the group
            let padding = 20.0;
            let group_x = min_x - padding;
            let group_y = min_y - padding;
            let group_width = (max_x - min_x) + (padding * 2.0);
            let group_height = (max_y - min_y) + (padding * 2.0);

            // Render group boundary rectangle
            svg.push_str(&format!(
                r##"  <rect x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" fill="#f0f0f0" fill-opacity="0.5" stroke="#999999" stroke-width="2" stroke-dasharray="10,5" rx="10"/>"##,
                group_x, group_y, group_width, group_height
            ));
            svg.push('\n');

            // Render group label at top-left corner
            let label_x = group_x + 10.0;
            let label_y = group_y + 20.0;
            svg.push_str(&format!(
                r##"  <text x="{:.0}" y="{:.0}" font-family="Arial, sans-serif" font-size="14" font-weight="bold" fill="#666666">{}</text>"##,
                label_x, label_y, escape_xml(&group.name)
            ));
            svg.push('\n');
        }
    }

    fn render_element(
        &self,
        svg: &mut String,
        node: &LayoutNode,
        element: &ElementInfo,
        style_resolver: &StyleResolver,
    ) {
        let x = node.position.x;
        let y = node.position.y;
        let width = node.size.width;
        let height = node.size.height;

        // Resolve the style for this element based on its tags
        let element_kind = element.element_type.to_element_kind();
        let resolved_style = style_resolver.resolve_element(&element.tags, element_kind);

        // Create bounds for shape rendering
        let bounds = Bounds::new(x, y, width, height);

        // Start a draggable group for this element
        svg.push_str(&format!(
            r##"  <g class="draggable-element" data-element-id="{}" transform="translate(0, 0)">
"##,
            element.id.to_string()
        ));

        // Render the shape using the resolved style
        svg.push_str("    ");
        svg.push_str(&render_shape(resolved_style.shape, &bounds, &resolved_style));
        svg.push('\n');

        // Render icon if present
        let icon_text_offset = if let Some(ref icon_url) = resolved_style.icon {
            let (icon_svg, text_offset) = render_icon(
                icon_url,
                resolved_style.icon_position,
                &bounds,
                resolved_style.shape,
            );
            svg.push_str("    ");
            svg.push_str(&icon_svg);
            svg.push('\n');
            text_offset
        } else {
            0.0
        };

        // Calculate text position based on shape (with icon offset)
        let text_y = match resolved_style.shape {
            structurizr_core::style::Shape::Person => y + height - 80.0,
            structurizr_core::style::Shape::Robot => y + height - 60.0,
            structurizr_core::style::Shape::Cylinder => y + height * 0.35,
            structurizr_core::style::Shape::WebBrowser => y + height * 0.25,
            structurizr_core::style::Shape::MobileDevicePortrait |
            structurizr_core::style::Shape::MobileDeviceLandscape => y + height * 0.35,
            structurizr_core::style::Shape::Folder => y + height * 0.35,
            _ => y + 40.0 + icon_text_offset,
        };

        let text_color = &resolved_style.color;
        let font_size = resolved_style.font_size;

        // Name
        svg.push_str(&format!(
            r#"    <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" font-weight="bold" fill="{}">{}</text>"#,
            x + width / 2.0, text_y, font_size + 2, text_color, escape_xml(&element.name)
        ));
        svg.push('\n');

        // Type label (only if metadata is enabled)
        if resolved_style.show_metadata {
            let type_label = match element.element_type {
                ElementType::Person => "[Person]",
                ElementType::SoftwareSystem => "[Software System]",
                ElementType::Container => "[Container]",
                ElementType::Component => "[Component]",
                ElementType::ExternalSystem => "[External System]",
                ElementType::DeploymentNode => "[Deployment Node]",
                ElementType::InfrastructureNode => "[Infrastructure Node]",
            };
            svg.push_str(&format!(
                r#"    <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" fill="{}">{}</text>"#,
                x + width / 2.0, text_y + 18.0, font_size - 3, text_color, type_label
            ));
            svg.push('\n');

            // Technology (if any)
            if let Some(ref tech) = element.technology {
                svg.push_str(&format!(
                    r#"    <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" fill="{}">[{}]</text>"#,
                    x + width / 2.0, text_y + 34.0, font_size - 3, text_color, escape_xml(tech)
                ));
                svg.push('\n');
            }
        }

        // Description (wrapped) - only if description is enabled
        if resolved_style.show_description {
            if let Some(ref desc) = element.description {
                let desc_y = if element.technology.is_some() && resolved_style.show_metadata {
                    text_y + 52.0
                } else if resolved_style.show_metadata {
                    text_y + 38.0
                } else {
                    text_y + 22.0
                };

                // Simple word wrap
                let max_chars = 40;
                let lines = wrap_text(desc, max_chars);
                for (i, line) in lines.iter().take(3).enumerate() {
                    svg.push_str(&format!(
                        r#"    <text x="{:.0}" y="{:.0}" text-anchor="middle" font-family="Arial, sans-serif" font-size="{}" fill="{}">{}</text>"#,
                        x + width / 2.0, desc_y + (i as f64 * 16.0), font_size - 2, text_color, escape_xml(line)
                    ));
                    svg.push('\n');
                }
            }
        }

        // Close the draggable group
        svg.push_str("  </g>\n");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
enum ElementType {
    Person,
    SoftwareSystem,
    Container,
    Component,
    ExternalSystem,
    DeploymentNode,
    InfrastructureNode,
}

impl ElementType {
    /// Convert to ElementKind for style resolution.
    fn to_element_kind(&self) -> ElementKind {
        match self {
            ElementType::Person => ElementKind::Person,
            ElementType::SoftwareSystem => ElementKind::SoftwareSystem,
            ElementType::Container => ElementKind::Container,
            ElementType::Component => ElementKind::Component,
            ElementType::ExternalSystem => ElementKind::External,
            ElementType::DeploymentNode => ElementKind::DeploymentNode,
            ElementType::InfrastructureNode => ElementKind::InfrastructureNode,
        }
    }
}

#[derive(Debug, Clone)]
struct ElementInfo {
    id: ElementId,
    name: String,
    description: Option<String>,
    element_type: ElementType,
    technology: Option<String>,
    /// Tags for style resolution
    tags: Vec<String>,
}

/// Entry in the legend for an element type.
#[derive(Debug, Clone)]
struct LegendElementEntry {
    label: String,
    shape: structurizr_core::style::Shape,
    background: String,
    stroke: String,
    stroke_width: u32,
    border: structurizr_core::style::Border,
}

/// Entry in the legend for a relationship style.
#[derive(Debug, Clone)]
struct LegendRelationshipEntry {
    label: String,
    style: structurizr_core::style::LineStyle,
    color: String,
    thickness: u32,
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line = word.to_string();
        } else if current_line.len() + 1 + word.len() <= max_chars {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}

/// Render an icon image within an element.
///
/// Returns the SVG `<image>` element and the vertical offset for text positioning.
fn render_icon(
    icon_url: &str,
    position: IconPosition,
    bounds: &Bounds,
    shape: structurizr_core::style::Shape,
) -> (String, f64) {
    // Icon size is proportional to element size, max 48px
    let icon_size = 48.0_f64.min(bounds.width * 0.25).min(bounds.height * 0.25);

    let (icon_x, icon_y) = match position {
        IconPosition::Top => {
            // Center horizontally, position at top with some padding
            let padding = match shape {
                structurizr_core::style::Shape::Person |
                structurizr_core::style::Shape::Robot => bounds.height * 0.15,
                structurizr_core::style::Shape::Cylinder => bounds.height * 0.2,
                _ => 10.0,
            };
            (
                bounds.x + (bounds.width - icon_size) / 2.0,
                bounds.y + padding,
            )
        }
        IconPosition::Bottom => {
            // Center horizontally, position at bottom
            (
                bounds.x + (bounds.width - icon_size) / 2.0,
                bounds.y + bounds.height - icon_size - 10.0,
            )
        }
        IconPosition::Left => {
            // Left side, vertically centered
            (
                bounds.x + 10.0,
                bounds.y + (bounds.height - icon_size) / 2.0,
            )
        }
    };

    // Calculate text offset based on icon position
    let text_offset = match position {
        IconPosition::Top => icon_size + 15.0,
        IconPosition::Bottom => 0.0,
        IconPosition::Left => 0.0,
    };

    let svg = format!(
        r#"<image x="{:.0}" y="{:.0}" width="{:.0}" height="{:.0}" href="{}" preserveAspectRatio="xMidYMid meet"/>"#,
        icon_x, icon_y, icon_size, icon_size, escape_xml(icon_url)
    );

    (svg, text_offset)
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::Workspace;

    #[test]
    fn test_render_empty_workspace() {
        let workspace = Workspace::new("Test", "Test workspace");
        let view = SystemLandscapeView::new("landscape");

        let renderer = SvgRenderer::default();
        let svg = renderer.render_system_landscape(&workspace, &view).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_with_legend() {
        let mut workspace = Workspace::new("Test", "Test workspace with legend");
        let model = workspace.model_mut();

        // Add a person
        let person_id = model.add_person("User", "A user of the system");

        // Add a software system
        let system_id = model.add_software_system("System", "The main system");

        // Add a relationship
        model.add_relationship(person_id, system_id, "Uses", None);

        let view = SystemLandscapeView::new("landscape");

        // Render without legend
        let renderer = SvgRenderer::default();
        let svg_without_legend = renderer.render_system_landscape(&workspace, &view).unwrap();
        assert!(!svg_without_legend.contains("Legend"));

        // Render with legend
        let renderer_with_legend = SvgRenderer::default().with_legend(true);
        let svg_with_legend = renderer_with_legend.render_system_landscape(&workspace, &view).unwrap();
        assert!(svg_with_legend.contains("Legend"));
        assert!(svg_with_legend.contains("Person"));
        assert!(svg_with_legend.contains("Software System"));
    }

    #[test]
    fn test_legend_toggle() {
        let mut renderer = SvgRenderer::default();
        assert!(!renderer.show_legend);

        renderer.enable_legend();
        assert!(renderer.show_legend);

        renderer.disable_legend();
        assert!(!renderer.show_legend);
    }

    #[test]
    fn test_render_custom_view() {
        let mut workspace = Workspace::new("Test", "Test custom view");
        let model = workspace.model_mut();

        // Add elements
        let person_id = model.add_person("User", "A user");
        let system_id = model.add_software_system("System", "The system");
        model.add_relationship(person_id, system_id, "Uses", None);

        // Create custom view with explicit elements
        let mut custom_view = CustomView::new("custom");
        custom_view.properties.add_element(person_id);
        custom_view.properties.add_element(system_id);

        let renderer = SvgRenderer::default();
        let svg = renderer.render_custom(&workspace, &custom_view).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn test_render_image_view() {
        let workspace = Workspace::new("Test", "Test image view");

        // Create image view without content
        let mut image_view = ImageView {
            properties: structurizr_core::view::ViewProperties::new("image"),
            element_id: None,
            content: None,
            content_type: None,
        };
        image_view.properties.title = Some("Test Image".to_string());

        let renderer = SvgRenderer::default();
        let svg = renderer.render_image(&workspace, &image_view).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("No image content"));
        assert!(svg.contains("Test Image"));
    }

    #[test]
    fn test_render_image_view_with_content() {
        let workspace = Workspace::new("Test", "Test image view with content");

        // Create image view with base64 content
        let image_view = ImageView {
            properties: structurizr_core::view::ViewProperties::new("image"),
            element_id: None,
            content: Some("iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==".to_string()),
            content_type: Some("image/png".to_string()),
        };

        let renderer = SvgRenderer::default();
        let svg = renderer.render_image(&workspace, &image_view).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("<image"));
        assert!(svg.contains("data:image/png;base64,"));
        assert!(!svg.contains("No image content"));
    }

    #[test]
    fn test_render_filtered_view() {
        use structurizr_core::view::{FilterMode, SystemLandscapeView};

        let mut workspace = Workspace::new("Test", "Test filtered view");
        let model = workspace.model_mut();

        // Add elements
        let person_id = model.add_person("User", "A user");
        let system1_id = model.add_software_system("System1", "First system");
        let system2_id = model.add_software_system("System2", "Second system");

        // Add tags manually to the model elements
        if let Some(person) = model.people.iter_mut().find(|p| p.id() == person_id) {
            person.properties.tags.push("External".to_string());
        }
        if let Some(system1) = model.software_systems.iter_mut().find(|s| s.id() == system1_id) {
            system1.properties.tags.push("Internal".to_string());
        }
        if let Some(system2) = model.software_systems.iter_mut().find(|s| s.id() == system2_id) {
            system2.properties.tags.push("External".to_string());
        }

        model.add_relationship(person_id, system1_id, "Uses", None);

        // Create base view
        let base_view = SystemLandscapeView::new("landscape");
        let views = workspace.views_mut();
        views.add_system_landscape_view(base_view);

        // Create filtered view to show only "Internal" tagged elements
        let filtered_view = FilteredView {
            properties: structurizr_core::view::ViewProperties::new("filtered"),
            base_view_key: "landscape".to_string(),
            mode: FilterMode::Include,
            tags: vec!["Internal".to_string()],
        };

        let renderer = SvgRenderer::default();
        let svg = renderer.render_filtered(&workspace, &filtered_view).unwrap();

        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }
}
