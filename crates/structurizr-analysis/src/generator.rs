//! Model generator - converts analysis results to C4 models.
//!
//! This module takes an `AnalyzedProject` and generates a Structurizr `Workspace`
//! with appropriate C4 model elements and views.

use crate::error::Result;
use crate::model::AnalyzedProject;
use structurizr_core::model::{Component, Container, ElementId, SoftwareSystem};
use structurizr_core::style::{ElementStyle, Shape, Styles, ViewConfiguration};
use structurizr_core::view::{
    AutoLayout, AutoLayoutDirection, ComponentView, ContainerView, SystemContextView, Views,
};
use structurizr_core::Workspace;
use std::collections::HashMap;
use tracing::info;

/// Configuration for model generation.
#[derive(Debug, Clone)]
pub struct GeneratorConfig {
    /// Name for the generated workspace
    pub workspace_name: Option<String>,

    /// Description for the generated workspace
    pub workspace_description: Option<String>,

    /// Whether to generate System Context view
    pub generate_system_context: bool,

    /// Whether to generate Container view
    pub generate_container_view: bool,

    /// Whether to generate Component views
    pub generate_component_views: bool,

    /// Whether to add default C4 styles
    pub add_default_styles: bool,

    /// External system name for the context
    pub external_system_name: Option<String>,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            workspace_name: None,
            workspace_description: None,
            generate_system_context: true,
            generate_container_view: true,
            generate_component_views: true,
            add_default_styles: true,
            external_system_name: None,
        }
    }
}

/// Generate a Structurizr workspace from an analyzed project.
pub fn generate_workspace(project: &AnalyzedProject, config: &GeneratorConfig) -> Result<Workspace> {
    let name = config
        .workspace_name
        .clone()
        .unwrap_or_else(|| project.name.clone());

    let description = config
        .workspace_description
        .clone()
        .or_else(|| project.description.clone())
        .unwrap_or_else(|| format!("Architecture model for {}", project.name));

    info!(
        "Generating workspace '{}' with {} containers",
        name,
        project.containers.len()
    );

    let mut workspace = Workspace::new(&name, &description);

    // Create the main software system with all containers
    let mut main_system = SoftwareSystem::new(&name);
    if let Some(desc) = &project.description {
        main_system = main_system.with_description(desc);
    }

    // Track container and component IDs for relationships
    let mut container_ids: HashMap<String, ElementId> = HashMap::new();
    let mut component_ids: HashMap<String, ElementId> = HashMap::new();

    // Add containers to the system
    for analyzed_container in &project.containers {
        let mut container = Container::new(&analyzed_container.name);

        if let Some(desc) = &analyzed_container.description {
            container = container.with_description(desc);
        }

        container = container.with_technology(&analyzed_container.technology);

        // Add components to this container and track their IDs
        for analyzed_component in &analyzed_container.components {
            let mut component = Component::new(&analyzed_component.name);

            if let Some(desc) = &analyzed_component.description {
                component = component.with_description(desc);
            }

            component = component.with_technology(&analyzed_component.technology);
            let component_id = container.add_component(component);
            component_ids.insert(analyzed_component.id.clone(), component_id);
        }

        let container_id = main_system.add_container(container);
        container_ids.insert(analyzed_container.id.clone(), container_id);
    }

    let system_id = main_system.id();
    workspace.model_mut().software_systems.push(main_system);

    // Add relationships (container-level and component-level)
    for relationship in &project.relationships {
        // First try container relationships
        if let (Some(&source_id), Some(&dest_id)) = (
            container_ids.get(&relationship.source_id),
            container_ids.get(&relationship.destination_id),
        ) {
            let technology = relationship.technology.clone();
            workspace.model_mut().add_relationship(
                source_id,
                dest_id,
                &relationship.description,
                technology,
            );
        }
        // Then try component relationships
        else if let (Some(&source_id), Some(&dest_id)) = (
            component_ids.get(&relationship.source_id),
            component_ids.get(&relationship.destination_id),
        ) {
            let technology = relationship.technology.clone();
            workspace.model_mut().add_relationship(
                source_id,
                dest_id,
                &relationship.description,
                technology,
            );
        }
    }

    // Add external dependencies as external systems
    let external_deps: Vec<_> = project
        .dependencies
        .iter()
        .filter(|d| {
            // Only include significant external dependencies
            matches!(
                d.category,
                crate::model::DependencyCategory::Database
                    | crate::model::DependencyCategory::MessageQueue
            )
        })
        .collect();

    for dep in external_deps {
        let ext_system = SoftwareSystem::new(&dep.name)
            .with_description(dep.purpose.as_deref().unwrap_or(""))
            .external();
        let ext_id = ext_system.id();
        workspace.model_mut().software_systems.push(ext_system);

        // Add relationship from main system to external dependency
        workspace.model_mut().add_relationship(
            system_id,
            ext_id,
            "Uses",
            None,
        );
    }

    // Generate views
    let mut views = Views::new();

    // System Context View
    if config.generate_system_context {
        let mut context_view = SystemContextView::new("SystemContext", system_id);
        context_view.properties.description = Some(format!("System Context for {}", name));
        context_view.properties.auto_layout = Some(AutoLayout {
            direction: AutoLayoutDirection::TopBottom,
            rank_separation: 300,
            node_separation: 300,
        });
        views.add_system_context_view(context_view);
    }

    // Container View
    if config.generate_container_view && !project.containers.is_empty() {
        let mut container_view = ContainerView::new("Containers", system_id);
        container_view.properties.description = Some(format!("Container diagram for {}", name));
        container_view.properties.auto_layout = Some(AutoLayout {
            direction: AutoLayoutDirection::TopBottom,
            rank_separation: 300,
            node_separation: 200,
        });
        views.add_container_view(container_view);
    }

    // Component Views (one per container with components)
    if config.generate_component_views {
        for analyzed_container in &project.containers {
            if analyzed_container.components.is_empty() {
                continue;
            }

            if let Some(&container_id) = container_ids.get(&analyzed_container.id) {
                let view_key = format!("Components_{}", analyzed_container.name.replace(['-', ' '], "_"));
                let mut component_view = ComponentView::new(&view_key, container_id);
                component_view.properties.description = Some(format!(
                    "Component diagram for {}",
                    analyzed_container.name
                ));
                component_view.properties.auto_layout = Some(AutoLayout {
                    direction: AutoLayoutDirection::TopBottom,
                    rank_separation: 200,
                    node_separation: 150,
                });
                views.add_component_view(component_view);
            }
        }
    }

    // Set views on workspace
    *workspace.views_mut() = views;

    // Add styles via ViewConfiguration
    if config.add_default_styles {
        let mut styles = Styles::new();

        // Person style
        let person_style = ElementStyle::new("Person")
            .with_shape(Shape::Person)
            .with_background("#08427b")
            .with_color("#ffffff");
        styles.add_element_style(person_style);

        // Software System style
        let system_style = ElementStyle::new("Software System")
            .with_background("#1168bd")
            .with_color("#ffffff");
        styles.add_element_style(system_style);

        // External system style
        let external_style = ElementStyle::new("External")
            .with_background("#999999")
            .with_color("#ffffff");
        styles.add_element_style(external_style);

        // Container style
        let container_style = ElementStyle::new("Container")
            .with_background("#438dd5")
            .with_color("#ffffff");
        styles.add_element_style(container_style);

        // Component style
        let component_style = ElementStyle::new("Component")
            .with_background("#85bbf0")
            .with_color("#000000");
        styles.add_element_style(component_style);

        // Database style
        let db_style = ElementStyle::new("Database")
            .with_shape(Shape::Cylinder)
            .with_background("#438dd5")
            .with_color("#ffffff");
        styles.add_element_style(db_style);

        workspace.configuration = Some(ViewConfiguration {
            styles,
            branding: None,
            terminology: None,
            default_view: None,
            last_saved_view: None,
        });
    }

    info!(
        "Generated workspace with {} views",
        workspace.views().all_keys().len()
    );

    Ok(workspace)
}

/// Generate DSL from an analyzed project.
pub fn generate_dsl(project: &AnalyzedProject, config: &GeneratorConfig) -> Result<String> {
    let workspace = generate_workspace(project, config)?;

    // Use the DSL serializer
    let dsl = structurizr_dsl::serialize(&workspace);

    Ok(dsl)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AnalyzedContainer, ContainerType, Language};
    use std::path::PathBuf;

    fn create_test_project() -> AnalyzedProject {
        let mut project = AnalyzedProject::new("test-project", "/test");
        project.primary_language = Some(Language::Rust);
        project.description = Some("A test project".to_string());

        project.add_container(AnalyzedContainer {
            id: "api".to_string(),
            name: "API".to_string(),
            description: Some("REST API service".to_string()),
            technology: "Rust, Axum".to_string(),
            container_type: ContainerType::Api,
            path: PathBuf::from("/test/api"),
            components: vec![],
            dependencies: vec![],
            metadata: Default::default(),
        });

        project.add_container(AnalyzedContainer {
            id: "core".to_string(),
            name: "Core".to_string(),
            description: Some("Core business logic".to_string()),
            technology: "Rust".to_string(),
            container_type: ContainerType::Library,
            path: PathBuf::from("/test/core"),
            components: vec![],
            dependencies: vec![],
            metadata: Default::default(),
        });

        project
    }

    #[test]
    fn test_generate_workspace() {
        let project = create_test_project();
        let config = GeneratorConfig::default();

        let workspace = generate_workspace(&project, &config).unwrap();

        assert_eq!(workspace.name, "test-project");
        assert!(!workspace.model().software_systems.is_empty());
    }

    #[test]
    fn test_generate_dsl() {
        let project = create_test_project();
        let config = GeneratorConfig::default();

        let dsl = generate_dsl(&project, &config).unwrap();

        assert!(dsl.contains("workspace"));
        assert!(dsl.contains("test-project"));
        assert!(dsl.contains("API"));
        assert!(dsl.contains("Core"));
    }
}
