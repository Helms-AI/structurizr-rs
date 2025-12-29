//! View definitions for architecture diagrams.
//!
//! Views are different perspectives of the model:
//! - SystemLandscape: All systems in the enterprise
//! - SystemContext: A system and its external dependencies
//! - Container: Containers within a system
//! - Component: Components within a container
//! - Dynamic: Sequence of interactions
//! - Deployment: Infrastructure and deployments

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

use crate::model::ElementId;

/// Auto-layout direction for diagrams.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum AutoLayoutDirection {
    #[default]
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

/// Auto-layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoLayout {
    pub direction: AutoLayoutDirection,
    #[serde(default = "default_rank_separation")]
    pub rank_separation: u32,
    #[serde(default = "default_node_separation")]
    pub node_separation: u32,
}

fn default_rank_separation() -> u32 {
    150  // Increased for better readability
}

fn default_node_separation() -> u32 {
    150  // Increased for better readability
}

impl Default for AutoLayout {
    fn default() -> Self {
        Self {
            direction: AutoLayoutDirection::default(),
            rank_separation: default_rank_separation(),
            node_separation: default_node_separation(),
        }
    }
}

/// Position of an element in a view.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ElementPosition {
    pub x: i32,
    pub y: i32,
}

/// Layout change for undo/redo history.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutChange {
    Move {
        element_id: ElementId,
        from: ElementPosition,
        to: ElementPosition,
        timestamp: u64,
    },
    BatchMove {
        moves: Vec<(ElementId, ElementPosition, ElementPosition)>,
        timestamp: u64,
    },
}

/// Layout state with history management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutState {
    pub positions: HashMap<ElementId, ElementPosition>,
    #[serde(default)]
    pub history: VecDeque<LayoutChange>,
    #[serde(default)]
    pub redo_stack: VecDeque<LayoutChange>,
    #[serde(skip)]
    pub dirty: bool,
    #[serde(skip)]
    pub last_change_timestamp: Option<u64>,
}

impl LayoutState {
    pub fn new() -> Self {
        Self {
            positions: HashMap::new(),
            history: VecDeque::new(),
            redo_stack: VecDeque::new(),
            dirty: false,
            last_change_timestamp: None,
        }
    }

    pub fn move_element(&mut self, element_id: ElementId, to: ElementPosition) {
        if let Some(&from) = self.positions.get(&element_id) {
            if from.x != to.x || from.y != to.y {
                let timestamp = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64;

                // Check if we should coalesce with the last change
                let should_coalesce = self.should_coalesce(&element_id, timestamp);

                if should_coalesce {
                    // Update the last change instead of creating a new one
                    if let Some(LayoutChange::Move { to: ref mut last_to, .. }) = self.history.back_mut() {
                        *last_to = to;
                    }
                } else {
                    // Create a new change
                    let change = LayoutChange::Move {
                        element_id: element_id.clone(),
                        from,
                        to,
                        timestamp,
                    };
                    self.history.push_back(change);
                    self.redo_stack.clear();
                }

                self.positions.insert(element_id.clone(), to);
                self.dirty = true;
                self.last_change_timestamp = Some(timestamp);
            }
        } else {
            // Element doesn't have a position yet, just set it
            self.positions.insert(element_id, to);
            self.dirty = true;
        }
    }

    fn should_coalesce(&self, element_id: &ElementId, timestamp: u64) -> bool {
        if let Some(last_timestamp) = self.last_change_timestamp {
            if timestamp - last_timestamp < 500 {  // Within 500ms
                if let Some(LayoutChange::Move { element_id: ref last_id, .. }) = self.history.back() {
                    return last_id == element_id;
                }
            }
        }
        false
    }

    pub fn undo(&mut self) -> bool {
        if let Some(change) = self.history.pop_back() {
            match &change {
                LayoutChange::Move { element_id, from, .. } => {
                    self.positions.insert(element_id.clone(), *from);
                }
                LayoutChange::BatchMove { moves, .. } => {
                    for (element_id, from, _) in moves {
                        self.positions.insert(element_id.clone(), *from);
                    }
                }
            }
            self.redo_stack.push_back(change);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some(change) = self.redo_stack.pop_back() {
            match &change {
                LayoutChange::Move { element_id, to, .. } => {
                    self.positions.insert(element_id.clone(), *to);
                }
                LayoutChange::BatchMove { moves, .. } => {
                    for (element_id, _, to) in moves {
                        self.positions.insert(element_id.clone(), *to);
                    }
                }
            }
            self.history.push_back(change);
            self.dirty = true;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}

impl Default for LayoutState {
    fn default() -> Self {
        Self::new()
    }
}

/// Position of a vertex in a relationship.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Vertex {
    pub x: i32,
    pub y: i32,
}

/// View of an element with its position.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementView {
    pub id: ElementId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
}

impl ElementView {
    pub fn new(id: ElementId) -> Self {
        Self { id, x: None, y: None }
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self
    }
}

/// View of a relationship with routing vertices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipView {
    pub id: ElementId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub vertices: Vec<Vertex>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<u32>,
}

impl RelationshipView {
    pub fn new(id: ElementId) -> Self {
        Self {
            id,
            description: None,
            vertices: Vec::new(),
            position: None,
        }
    }
}

/// Base properties shared by all views.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewProperties {
    pub key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ElementView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipView>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_layout: Option<AutoLayout>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    /// Background color for the view (e.g., "#1a1a1a" for dark mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    /// Layout state for drag-and-drop positioning and history.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout_state: Option<LayoutState>,
}

impl ViewProperties {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            title: None,
            description: None,
            elements: Vec::new(),
            relationships: Vec::new(),
            auto_layout: None,
            properties: HashMap::new(),
            background: None,
            layout_state: None,
        }
    }

    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_auto_layout(mut self, auto_layout: AutoLayout) -> Self {
        self.auto_layout = Some(auto_layout);
        self
    }

    pub fn with_background(mut self, background: impl Into<String>) -> Self {
        self.background = Some(background.into());
        self
    }

    pub fn add_element(&mut self, element_id: ElementId) {
        self.elements.push(ElementView::new(element_id));
    }

    pub fn add_relationship(&mut self, relationship_id: ElementId) {
        self.relationships.push(RelationshipView::new(relationship_id));
    }
}

/// System landscape view showing all systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemLandscapeView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    #[serde(default)]
    pub enterprise_boundary_visible: bool,
}

impl SystemLandscapeView {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            properties: ViewProperties::new(key),
            enterprise_boundary_visible: true,
        }
    }
}

/// System context view for a specific software system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemContextView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    pub software_system_id: ElementId,
    #[serde(default)]
    pub enterprise_boundary_visible: bool,
}

impl SystemContextView {
    pub fn new(key: impl Into<String>, software_system_id: ElementId) -> Self {
        Self {
            properties: ViewProperties::new(key),
            software_system_id,
            enterprise_boundary_visible: true,
        }
    }
}

/// Container view for a specific software system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    pub software_system_id: ElementId,
    #[serde(default)]
    pub external_software_system_boundary_visible: bool,
}

impl ContainerView {
    pub fn new(key: impl Into<String>, software_system_id: ElementId) -> Self {
        Self {
            properties: ViewProperties::new(key),
            software_system_id,
            external_software_system_boundary_visible: true,
        }
    }
}

/// Component view for a specific container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    pub container_id: ElementId,
    #[serde(default)]
    pub external_software_system_boundary_visible: bool,
}

impl ComponentView {
    pub fn new(key: impl Into<String>, container_id: ElementId) -> Self {
        Self {
            properties: ViewProperties::new(key),
            container_id,
            external_software_system_boundary_visible: true,
        }
    }
}

/// A step in a dynamic view.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicViewStep {
    pub source_id: ElementId,
    pub destination_id: ElementId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub order: u32,
}

/// Dynamic view showing interactions over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<ElementId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<DynamicViewStep>,
}

impl DynamicView {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            properties: ViewProperties::new(key),
            element_id: None,
            steps: Vec::new(),
        }
    }

    pub fn for_element(key: impl Into<String>, element_id: ElementId) -> Self {
        Self {
            properties: ViewProperties::new(key),
            element_id: Some(element_id),
            steps: Vec::new(),
        }
    }
}

/// Deployment view for a specific environment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub software_system_id: Option<ElementId>,
    pub environment: String,
}

impl DeploymentView {
    pub fn new(key: impl Into<String>, environment: impl Into<String>) -> Self {
        Self {
            properties: ViewProperties::new(key),
            software_system_id: None,
            environment: environment.into(),
        }
    }

    pub fn for_system(
        key: impl Into<String>,
        software_system_id: ElementId,
        environment: impl Into<String>,
    ) -> Self {
        Self {
            properties: ViewProperties::new(key),
            software_system_id: Some(software_system_id),
            environment: environment.into(),
        }
    }
}

/// Filtered view based on another view with tag filtering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilteredView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    pub base_view_key: String,
    pub mode: FilterMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

/// Filter mode for filtered views.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FilterMode {
    #[default]
    Include,
    Exclude,
}

/// Custom view for arbitrary diagrams.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomView {
    #[serde(flatten)]
    pub properties: ViewProperties,
}

impl CustomView {
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            properties: ViewProperties::new(key),
        }
    }
}

/// Image view for embedding images.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageView {
    #[serde(flatten)]
    pub properties: ViewProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
}

/// All views in a workspace.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Views {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub system_landscape_views: Vec<SystemLandscapeView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub system_context_views: Vec<SystemContextView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub container_views: Vec<ContainerView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub component_views: Vec<ComponentView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dynamic_views: Vec<DynamicView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deployment_views: Vec<DeploymentView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filtered_views: Vec<FilteredView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_views: Vec<CustomView>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub image_views: Vec<ImageView>,
}

impl Views {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_system_landscape_view(&mut self, view: SystemLandscapeView) {
        self.system_landscape_views.push(view);
    }

    pub fn add_system_context_view(&mut self, view: SystemContextView) {
        self.system_context_views.push(view);
    }

    pub fn add_container_view(&mut self, view: ContainerView) {
        self.container_views.push(view);
    }

    pub fn add_component_view(&mut self, view: ComponentView) {
        self.component_views.push(view);
    }

    pub fn add_dynamic_view(&mut self, view: DynamicView) {
        self.dynamic_views.push(view);
    }

    pub fn add_deployment_view(&mut self, view: DeploymentView) {
        self.deployment_views.push(view);
    }

    /// Get all view keys.
    pub fn all_keys(&self) -> Vec<&str> {
        let mut keys = Vec::new();
        for v in &self.system_landscape_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.system_context_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.container_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.component_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.dynamic_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.deployment_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.filtered_views {
            keys.push(v.properties.key.as_str());
        }
        for v in &self.custom_views {
            keys.push(v.properties.key.as_str());
        }
        keys
    }
}
