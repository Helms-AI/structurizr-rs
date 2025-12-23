//! Ilograph export for Structurizr views.
//!
//! Ilograph uses YAML format for architecture diagrams with resources and perspectives.
//!
//! # Overview
//!
//! This module exports Structurizr workspaces to Ilograph YAML format, which can be
//! used to create interactive architecture diagrams at https://app.ilograph.com/.
//!
//! # Format
//!
//! Ilograph diagrams consist of:
//! - **Resources**: The elements in your architecture (people, systems, containers, components)
//! - **Perspectives**: Different views of your architecture showing specific relationships
//!
//! # Mapping
//!
//! The exporter maps C4 model elements to Ilograph format as follows:
//! - Person -> Resource with `style: actor`
//! - SoftwareSystem -> Resource with `style: box`
//! - Container -> Nested resource (child) of its parent system
//! - Component -> Nested resource (child) of its parent container
//! - View -> Perspective with filtered relationships
//!
//! # Example
//!
//! ```rust
//! use structurizr_core::{Workspace, view::SystemLandscapeView};
//! use structurizr_export::IlographExporter;
//!
//! let mut workspace = Workspace::new("My System", "Example workspace");
//! let user = workspace.model_mut().add_person("User", "A user");
//! let system = workspace.model_mut().add_software_system("System", "A system");
//! workspace.model_mut().add_relationship(user, system, "Uses", None);
//!
//! let view = SystemLandscapeView::new("landscape");
//! let yaml = IlographExporter::export_system_landscape(&workspace, &view).unwrap();
//! ```

use serde::{Deserialize, Serialize};
use structurizr_core::model::{ElementId, Model};
use structurizr_core::view::{
    ComponentView, ContainerView, SystemContextView, SystemLandscapeView,
};
use structurizr_core::Workspace;

use crate::error::Result;

/// A resource in an Ilograph diagram (node/actor).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlographResource {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<Vec<IlographResource>>,
}

/// A relation in an Ilograph perspective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlographRelation {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// A perspective in an Ilograph diagram (view).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlographPerspective {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub relations: Vec<IlographRelation>,
}

/// Root Ilograph document structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IlographDocument {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub resources: Vec<IlographResource>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub perspectives: Vec<IlographPerspective>,
}

/// Exports views to Ilograph YAML format.
pub struct IlographExporter;

impl IlographExporter {
    /// Export the entire workspace to Ilograph format.
    pub fn export_workspace(workspace: &Workspace) -> Result<String> {
        let model = workspace.model();
        let views = workspace.views();

        let mut resources = Vec::new();
        let mut perspectives = Vec::new();

        // Build resources from model elements
        Self::build_resources(model, &mut resources);

        // Build perspectives from views
        for view in &views.system_landscape_views {
            if let Ok(perspective) = Self::build_perspective_from_system_landscape(workspace, view) {
                perspectives.push(perspective);
            }
        }

        for view in &views.system_context_views {
            if let Ok(perspective) = Self::build_perspective_from_system_context(workspace, view) {
                perspectives.push(perspective);
            }
        }

        for view in &views.container_views {
            if let Ok(perspective) = Self::build_perspective_from_container(workspace, view) {
                perspectives.push(perspective);
            }
        }

        for view in &views.component_views {
            if let Ok(perspective) = Self::build_perspective_from_component(workspace, view) {
                perspectives.push(perspective);
            }
        }

        let document = IlographDocument {
            resources,
            perspectives,
        };

        let yaml = serde_yaml::to_string(&document)
            .map_err(|e| crate::error::Error::Export(format!("YAML serialization error: {}", e)))?;

        Ok(yaml)
    }

    /// Export a system landscape view to Ilograph format.
    pub fn export_system_landscape(workspace: &Workspace, view: &SystemLandscapeView) -> Result<String> {
        let model = workspace.model();

        let mut resources = Vec::new();
        Self::build_resources(model, &mut resources);

        let perspective = Self::build_perspective_from_system_landscape(workspace, view)?;

        let document = IlographDocument {
            resources,
            perspectives: vec![perspective],
        };

        let yaml = serde_yaml::to_string(&document)
            .map_err(|e| crate::error::Error::Export(format!("YAML serialization error: {}", e)))?;

        Ok(yaml)
    }

    /// Export a system context view to Ilograph format.
    pub fn export_system_context(workspace: &Workspace, view: &SystemContextView) -> Result<String> {
        let model = workspace.model();

        let mut resources = Vec::new();
        Self::build_resources(model, &mut resources);

        let perspective = Self::build_perspective_from_system_context(workspace, view)?;

        let document = IlographDocument {
            resources,
            perspectives: vec![perspective],
        };

        let yaml = serde_yaml::to_string(&document)
            .map_err(|e| crate::error::Error::Export(format!("YAML serialization error: {}", e)))?;

        Ok(yaml)
    }

    /// Export a container view to Ilograph format.
    pub fn export_container(workspace: &Workspace, view: &ContainerView) -> Result<String> {
        let model = workspace.model();

        let mut resources = Vec::new();
        Self::build_resources(model, &mut resources);

        let perspective = Self::build_perspective_from_container(workspace, view)?;

        let document = IlographDocument {
            resources,
            perspectives: vec![perspective],
        };

        let yaml = serde_yaml::to_string(&document)
            .map_err(|e| crate::error::Error::Export(format!("YAML serialization error: {}", e)))?;

        Ok(yaml)
    }

    /// Export a component view to Ilograph format.
    pub fn export_component(workspace: &Workspace, view: &ComponentView) -> Result<String> {
        let model = workspace.model();

        let mut resources = Vec::new();
        Self::build_resources(model, &mut resources);

        let perspective = Self::build_perspective_from_component(workspace, view)?;

        let document = IlographDocument {
            resources,
            perspectives: vec![perspective],
        };

        let yaml = serde_yaml::to_string(&document)
            .map_err(|e| crate::error::Error::Export(format!("YAML serialization error: {}", e)))?;

        Ok(yaml)
    }

    // Helper methods

    /// Build all resources from the model.
    fn build_resources(model: &Model, resources: &mut Vec<IlographResource>) {
        // Add people as actors
        for person in &model.people {
            resources.push(IlographResource {
                id: sanitize_id(&person.properties.id.to_string()),
                name: person.name().to_string(),
                subtitle: person.properties.description.clone(),
                style: Some("actor".to_string()),
                children: None,
            });
        }

        // Add software systems with their containers as children
        for system in &model.software_systems {
            let mut system_resource = IlographResource {
                id: sanitize_id(&system.properties.id.to_string()),
                name: system.name().to_string(),
                subtitle: system.properties.description.clone(),
                style: Some("box".to_string()),
                children: None,
            };

            // Add containers as children
            if !system.containers.is_empty() {
                let mut children = Vec::new();
                for container in &system.containers {
                    let mut container_resource = IlographResource {
                        id: sanitize_id(&container.properties.id.to_string()),
                        name: container.name().to_string(),
                        subtitle: container.properties.description.clone()
                            .or_else(|| container.technology.clone()),
                        style: Some("box".to_string()),
                        children: None,
                    };

                    // Add components as nested children
                    if !container.components.is_empty() {
                        let mut component_children = Vec::new();
                        for component in &container.components {
                            component_children.push(IlographResource {
                                id: sanitize_id(&component.properties.id.to_string()),
                                name: component.name().to_string(),
                                subtitle: component.properties.description.clone()
                                    .or_else(|| component.technology.clone()),
                                style: Some("box".to_string()),
                                children: None,
                            });
                        }
                        container_resource.children = Some(component_children);
                    }

                    children.push(container_resource);
                }
                system_resource.children = Some(children);
            }

            resources.push(system_resource);
        }
    }

    /// Build a perspective from a system landscape view.
    fn build_perspective_from_system_landscape(
        workspace: &Workspace,
        view: &SystemLandscapeView,
    ) -> Result<IlographPerspective> {
        let model = workspace.model();
        let name = view.properties.title.clone()
            .unwrap_or_else(|| view.properties.key.clone());

        let mut relations = Vec::new();

        // Add all relationships from the model
        for rel in &model.relationships {
            relations.push(IlographRelation {
                from: sanitize_id(&rel.source_id.to_string()),
                to: sanitize_id(&rel.destination_id.to_string()),
                label: rel.description.clone(),
            });
        }

        Ok(IlographPerspective {
            name,
            description: view.properties.description.clone(),
            relations,
        })
    }

    /// Build a perspective from a system context view.
    fn build_perspective_from_system_context(
        workspace: &Workspace,
        view: &SystemContextView,
    ) -> Result<IlographPerspective> {
        let model = workspace.model();
        let name = view.properties.title.clone()
            .unwrap_or_else(|| view.properties.key.clone());

        let mut relations = Vec::new();

        // Filter relationships to only those relevant to the view
        let view_element_ids: std::collections::HashSet<ElementId> = view
            .properties
            .elements
            .iter()
            .map(|e| e.id)
            .collect();

        for rel in &model.relationships {
            if view_element_ids.contains(&rel.source_id)
                || view_element_ids.contains(&rel.destination_id) {
                relations.push(IlographRelation {
                    from: sanitize_id(&rel.source_id.to_string()),
                    to: sanitize_id(&rel.destination_id.to_string()),
                    label: rel.description.clone(),
                });
            }
        }

        Ok(IlographPerspective {
            name,
            description: view.properties.description.clone(),
            relations,
        })
    }

    /// Build a perspective from a container view.
    fn build_perspective_from_container(
        workspace: &Workspace,
        view: &ContainerView,
    ) -> Result<IlographPerspective> {
        let model = workspace.model();
        let name = view.properties.title.clone()
            .unwrap_or_else(|| view.properties.key.clone());

        let mut relations = Vec::new();

        // Filter relationships to only those relevant to the view
        let view_element_ids: std::collections::HashSet<ElementId> = view
            .properties
            .elements
            .iter()
            .map(|e| e.id)
            .collect();

        for rel in &model.relationships {
            if view_element_ids.contains(&rel.source_id)
                || view_element_ids.contains(&rel.destination_id) {
                let mut relation = IlographRelation {
                    from: sanitize_id(&rel.source_id.to_string()),
                    to: sanitize_id(&rel.destination_id.to_string()),
                    label: rel.description.clone(),
                };

                // Add technology to label if present
                if let Some(ref tech) = rel.technology {
                    relation.label = match relation.label {
                        Some(desc) => Some(format!("{} [{}]", desc, tech)),
                        None => Some(tech.clone()),
                    };
                }

                relations.push(relation);
            }
        }

        Ok(IlographPerspective {
            name,
            description: view.properties.description.clone(),
            relations,
        })
    }

    /// Build a perspective from a component view.
    fn build_perspective_from_component(
        workspace: &Workspace,
        view: &ComponentView,
    ) -> Result<IlographPerspective> {
        let model = workspace.model();
        let name = view.properties.title.clone()
            .unwrap_or_else(|| view.properties.key.clone());

        let mut relations = Vec::new();

        // Filter relationships to only those relevant to the view
        let view_element_ids: std::collections::HashSet<ElementId> = view
            .properties
            .elements
            .iter()
            .map(|e| e.id)
            .collect();

        for rel in &model.relationships {
            if view_element_ids.contains(&rel.source_id)
                || view_element_ids.contains(&rel.destination_id) {
                let mut relation = IlographRelation {
                    from: sanitize_id(&rel.source_id.to_string()),
                    to: sanitize_id(&rel.destination_id.to_string()),
                    label: rel.description.clone(),
                };

                // Add technology to label if present
                if let Some(ref tech) = rel.technology {
                    relation.label = match relation.label {
                        Some(desc) => Some(format!("{} [{}]", desc, tech)),
                        None => Some(tech.clone()),
                    };
                }

                relations.push(relation);
            }
        }

        Ok(IlographPerspective {
            name,
            description: view.properties.description.clone(),
            relations,
        })
    }
}

/// Sanitize an ID for use in Ilograph YAML.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_workspace() {
        let mut workspace = Workspace::new("Test System", "A test workspace");
        let user = workspace.model_mut().add_person("User", "A person");
        let system = workspace.model_mut().add_software_system("System", "A system");
        workspace.model_mut().add_relationship(user, system, "Uses", None);

        // Add a view
        let view = SystemLandscapeView::new("landscape");
        workspace.views_mut().add_system_landscape_view(view);

        let yaml = IlographExporter::export_workspace(&workspace).unwrap();

        assert!(yaml.contains("resources:"));
        assert!(yaml.contains("perspectives:"));
        assert!(yaml.contains("User"));
        assert!(yaml.contains("System"));
        assert!(yaml.contains("actor"));
    }

    #[test]
    fn test_export_system_landscape() {
        let mut workspace = Workspace::new("Test", "A test");
        let user = workspace.model_mut().add_person("User", "A user");
        let system = workspace.model_mut().add_software_system("My System", "A system");
        workspace.model_mut().add_relationship(user, system, "Uses", None);

        let view = SystemLandscapeView::new("landscape");
        let yaml = IlographExporter::export_system_landscape(&workspace, &view).unwrap();

        assert!(yaml.contains("resources:"));
        assert!(yaml.contains("User"));
        assert!(yaml.contains("My System"));
        assert!(yaml.contains("style: actor"));
        assert!(yaml.contains("style: box"));
    }

    #[test]
    fn test_sanitize_id() {
        assert_eq!(sanitize_id("abc-123"), "abc_123");
        assert_eq!(sanitize_id("test@example.com"), "test_example_com");
        assert_eq!(sanitize_id("valid_id"), "valid_id");
    }
}
