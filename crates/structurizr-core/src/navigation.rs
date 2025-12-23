//! Navigation support for drill-down between C4 diagram views.
//!
//! This module provides data structures and algorithms for navigating
//! between different levels of C4 architecture diagrams:
//! - SystemLandscape/SystemContext → ContainerView
//! - ContainerView → ComponentView

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::model::ElementId;
use crate::view::Views;

/// Type of C4 view for navigation purposes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewType {
    SystemLandscape,
    SystemContext,
    Container,
    Component,
    Dynamic,
    Deployment,
    Filtered,
    Custom,
    Image,
}

impl ViewType {
    /// Get the C4 level indicator (C1, C2, C3, etc.)
    pub fn level_indicator(&self) -> &'static str {
        match self {
            ViewType::SystemLandscape => "C1",
            ViewType::SystemContext => "C1",
            ViewType::Container => "C2",
            ViewType::Component => "C3",
            ViewType::Dynamic => "Dyn",
            ViewType::Deployment => "Dep",
            ViewType::Filtered => "Flt",
            ViewType::Custom => "Cst",
            ViewType::Image => "Img",
        }
    }

    /// Get human-readable name for the view type.
    pub fn display_name(&self) -> &'static str {
        match self {
            ViewType::SystemLandscape => "System Landscape",
            ViewType::SystemContext => "System Context",
            ViewType::Container => "Container",
            ViewType::Component => "Component",
            ViewType::Dynamic => "Dynamic",
            ViewType::Deployment => "Deployment",
            ViewType::Filtered => "Filtered",
            ViewType::Custom => "Custom",
            ViewType::Image => "Image",
        }
    }
}

/// Target information for drilling down from an element.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DrillDownTarget {
    /// The view key to navigate to.
    pub view_key: String,
    /// Title of the target view (for display).
    pub view_title: Option<String>,
    /// Type of the target view.
    pub target_type: ViewType,
}

/// A breadcrumb entry for navigation trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbEntry {
    /// View key for navigation.
    pub view_key: String,
    /// Display title for the breadcrumb.
    pub title: String,
    /// Type of view.
    pub view_type: ViewType,
}

/// Index for efficient drill-down navigation lookups.
///
/// This index is built from the workspace views and provides O(1) lookups
/// for determining:
/// - Whether an element can be drilled into (has a child view)
/// - What view to navigate to when drilling down
/// - Parent view relationships for breadcrumb navigation
#[derive(Debug, Clone, Default)]
pub struct NavigationIndex {
    /// Maps ElementId to its drill-down target (the view showing its children).
    /// - SoftwareSystem ID → ContainerView key
    /// - Container ID → ComponentView key
    drill_targets: HashMap<ElementId, DrillDownTarget>,

    /// Maps view key to its parent view key (for breadcrumb "up" navigation).
    parent_views: HashMap<String, String>,

    /// Maps view key to the element it focuses on.
    /// - ContainerView key → SoftwareSystem ID
    /// - ComponentView key → Container ID
    view_focus_elements: HashMap<String, ElementId>,

    /// Maps view key to its type.
    view_types: HashMap<String, ViewType>,

    /// Maps view key to its title.
    view_titles: HashMap<String, String>,

    /// Set of element IDs that have drill-down views available.
    drillable_elements: HashSet<ElementId>,
}

impl NavigationIndex {
    /// Build a navigation index from workspace views.
    pub fn build(views: &Views) -> Self {
        let mut index = Self::default();

        // Index all view types and titles
        for view in &views.system_landscape_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::SystemLandscape);
            index.view_titles.insert(key, title);
        }

        for view in &views.system_context_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::SystemContext);
            index.view_titles.insert(key.clone(), title);
            index.view_focus_elements.insert(key, view.software_system_id);
        }

        // Index ContainerViews: SoftwareSystem → ContainerView
        for view in &views.container_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());

            index.view_types.insert(key.clone(), ViewType::Container);
            index.view_titles.insert(key.clone(), title.clone());
            index.view_focus_elements.insert(key.clone(), view.software_system_id);

            // This software system can drill down to this container view
            let target = DrillDownTarget {
                view_key: key.clone(),
                view_title: Some(title),
                target_type: ViewType::Container,
            };
            index.drill_targets.insert(view.software_system_id, target);
            index.drillable_elements.insert(view.software_system_id);

            // Find parent SystemContextView for this software system
            for ctx_view in &views.system_context_views {
                if ctx_view.software_system_id == view.software_system_id {
                    index.parent_views.insert(key.clone(), ctx_view.properties.key.clone());
                    break;
                }
            }
        }

        // Index ComponentViews: Container → ComponentView
        for view in &views.component_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());

            index.view_types.insert(key.clone(), ViewType::Component);
            index.view_titles.insert(key.clone(), title.clone());
            index.view_focus_elements.insert(key.clone(), view.container_id);

            // This container can drill down to this component view
            let target = DrillDownTarget {
                view_key: key.clone(),
                view_title: Some(title),
                target_type: ViewType::Component,
            };
            index.drill_targets.insert(view.container_id, target);
            index.drillable_elements.insert(view.container_id);

            // Find parent ContainerView for this container's system
            // Note: We need to find which system owns this container
            // This requires the Model, but for now we link to any ContainerView
            // that shows the same container (approximate parent)
            for cont_view in &views.container_views {
                // If this container view has the container in its elements,
                // it could be the parent. For now, we just use the first
                // container view as parent (will be refined later).
                index.parent_views.insert(key.clone(), cont_view.properties.key.clone());
                break;
            }
        }

        // Index other view types
        for view in &views.dynamic_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::Dynamic);
            index.view_titles.insert(key, title);
        }

        for view in &views.deployment_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::Deployment);
            index.view_titles.insert(key, title);
        }

        for view in &views.filtered_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::Filtered);
            index.view_titles.insert(key, title);
        }

        for view in &views.custom_views {
            let key = view.properties.key.clone();
            let title = view.properties.title.clone()
                .unwrap_or_else(|| view.properties.key.clone());
            index.view_types.insert(key.clone(), ViewType::Custom);
            index.view_titles.insert(key, title);
        }

        index
    }

    /// Check if an element can be drilled into (has a child view).
    pub fn is_drillable(&self, element_id: ElementId) -> bool {
        self.drillable_elements.contains(&element_id)
    }

    /// Get the drill-down target for an element.
    pub fn get_drill_target(&self, element_id: ElementId) -> Option<&DrillDownTarget> {
        self.drill_targets.get(&element_id)
    }

    /// Get the parent view key for a given view.
    pub fn get_parent_view(&self, view_key: &str) -> Option<&String> {
        self.parent_views.get(view_key)
    }

    /// Get the type of a view by its key.
    pub fn get_view_type(&self, view_key: &str) -> Option<ViewType> {
        self.view_types.get(view_key).copied()
    }

    /// Get the title of a view by its key.
    pub fn get_view_title(&self, view_key: &str) -> Option<&String> {
        self.view_titles.get(view_key)
    }

    /// Get the element ID that a view focuses on.
    pub fn get_view_focus_element(&self, view_key: &str) -> Option<ElementId> {
        self.view_focus_elements.get(view_key).copied()
    }

    /// Build breadcrumb trail for a given view.
    pub fn build_breadcrumbs(&self, view_key: &str) -> Vec<BreadcrumbEntry> {
        let mut breadcrumbs = Vec::new();
        let mut current_key = Some(view_key.to_string());

        // Walk up the parent chain
        while let Some(key) = current_key {
            let view_type = self.get_view_type(&key).unwrap_or(ViewType::Custom);
            let title = self.get_view_title(&key)
                .cloned()
                .unwrap_or_else(|| key.clone());

            breadcrumbs.push(BreadcrumbEntry {
                view_key: key.clone(),
                title,
                view_type,
            });

            current_key = self.get_parent_view(&key).cloned();
        }

        // Reverse so root is first
        breadcrumbs.reverse();
        breadcrumbs
    }

    /// Get all drillable element IDs.
    pub fn drillable_element_ids(&self) -> impl Iterator<Item = ElementId> + '_ {
        self.drillable_elements.iter().copied()
    }

    /// Get drill targets as a map (for JSON serialization).
    pub fn drill_targets_map(&self) -> &HashMap<ElementId, DrillDownTarget> {
        &self.drill_targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::{ContainerView, ComponentView, SystemContextView, Views};

    #[test]
    fn test_empty_views() {
        let views = Views::default();
        let index = NavigationIndex::build(&views);

        assert!(!index.is_drillable(ElementId::default()));
        assert!(index.drill_targets_map().is_empty());
    }

    #[test]
    fn test_container_view_makes_system_drillable() {
        let mut views = Views::default();
        let system_id = ElementId::new();

        let container_view = ContainerView::new("containers", system_id);
        views.add_container_view(container_view);

        let index = NavigationIndex::build(&views);

        assert!(index.is_drillable(system_id));
        let target = index.get_drill_target(system_id).unwrap();
        assert_eq!(target.view_key, "containers");
        assert_eq!(target.target_type, ViewType::Container);
    }

    #[test]
    fn test_component_view_makes_container_drillable() {
        let mut views = Views::default();
        let container_id = ElementId::new();

        let component_view = ComponentView::new("components", container_id);
        views.add_component_view(component_view);

        let index = NavigationIndex::build(&views);

        assert!(index.is_drillable(container_id));
        let target = index.get_drill_target(container_id).unwrap();
        assert_eq!(target.view_key, "components");
        assert_eq!(target.target_type, ViewType::Component);
    }

    #[test]
    fn test_breadcrumb_building() {
        let mut views = Views::default();
        let system_id = ElementId::new();
        let container_id = ElementId::new();

        // Create view hierarchy
        let ctx_view = SystemContextView::new("context", system_id);
        views.add_system_context_view(ctx_view);

        let mut cont_view = ContainerView::new("containers", system_id);
        cont_view.properties.title = Some("My Containers".to_string());
        views.add_container_view(cont_view);

        let mut comp_view = ComponentView::new("components", container_id);
        comp_view.properties.title = Some("API Components".to_string());
        views.add_component_view(comp_view);

        let index = NavigationIndex::build(&views);

        // Check breadcrumbs for component view
        let breadcrumbs = index.build_breadcrumbs("components");
        assert!(!breadcrumbs.is_empty());

        // The last entry should be the component view
        let last = breadcrumbs.last().unwrap();
        assert_eq!(last.view_key, "components");
        assert_eq!(last.title, "API Components");
    }

    #[test]
    fn test_view_types() {
        let mut views = Views::default();
        let system_id = ElementId::new();

        views.add_system_context_view(SystemContextView::new("context", system_id));
        views.add_container_view(ContainerView::new("containers", system_id));

        let index = NavigationIndex::build(&views);

        assert_eq!(index.get_view_type("context"), Some(ViewType::SystemContext));
        assert_eq!(index.get_view_type("containers"), Some(ViewType::Container));
        assert_eq!(index.get_view_type("nonexistent"), None);
    }
}
