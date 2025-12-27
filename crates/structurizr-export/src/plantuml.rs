//! PlantUML export for Structurizr views.

use std::collections::HashSet;

use structurizr_core::view::{
    ComponentView, ContainerView, DeploymentView, DynamicView, SystemContextView,
    SystemLandscapeView,
};
use structurizr_core::Workspace;

use crate::error::Result;

/// Exports views to PlantUML format.
pub struct PlantUmlExporter;

impl PlantUmlExporter {
    /// Export a system landscape view to PlantUML.
    pub fn export_system_landscape(workspace: &Workspace, view: &SystemLandscapeView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Add people
        for person in &model.people {
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                person.properties.description.as_deref().unwrap_or("")
            ));
        }

        output.push('\n');

        // Add software systems
        for system in &model.software_systems {
            let id = system.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "System({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                system.name(),
                system.properties.description.as_deref().unwrap_or("")
            ));
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                output.push_str(&format!(
                    "Rel({}, {}, \"{}\")\n",
                    sanitize_id(&source),
                    sanitize_id(&dest),
                    rel.description.as_deref().unwrap_or("uses")
                ));
            }
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a system context view to PlantUML.
    pub fn export_system_context(workspace: &Workspace, view: &SystemContextView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Context.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Find the main system
        let main_system = model.software_systems.iter().find(|s| s.id() == view.software_system_id);

        // Add people
        for person in &model.people {
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                person.properties.description.as_deref().unwrap_or("")
            ));
        }

        output.push('\n');

        // Add software systems
        for system in &model.software_systems {
            let id = system.properties.id.to_string();
            element_ids.insert(id.clone());
            let is_main = main_system.map(|m| m.id() == system.id()).unwrap_or(false);
            if is_main {
                output.push_str(&format!(
                    "System({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    system.properties.description.as_deref().unwrap_or("")
                ));
            } else {
                output.push_str(&format!(
                    "System_Ext({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    system.properties.description.as_deref().unwrap_or("")
                ));
            }
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                output.push_str(&format!(
                    "Rel({}, {}, \"{}\")\n",
                    sanitize_id(&source),
                    sanitize_id(&dest),
                    rel.description.as_deref().unwrap_or("uses")
                ));
            }
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a container view to PlantUML.
    pub fn export_container(workspace: &Workspace, view: &ContainerView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Container.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Add people
        for person in &model.people {
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                person.properties.description.as_deref().unwrap_or("")
            ));
        }

        output.push('\n');

        // Find the main system and add its containers
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            output.push_str(&format!(
                "System_Boundary({}_boundary, \"{}\") {{\n",
                sanitize_id(&system.properties.id.to_string()),
                system.name()
            ));

            for container in &system.containers {
                let id = container.properties.id.to_string();
                element_ids.insert(id.clone());
                let tech = container.technology.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "  Container({}, \"{}\", \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    container.name(),
                    tech,
                    container.properties.description.as_deref().unwrap_or("")
                ));
            }

            output.push_str("}\n\n");
        }

        // Add external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                let id = system.properties.id.to_string();
                element_ids.insert(id.clone());
                output.push_str(&format!(
                    "System_Ext({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    system.properties.description.as_deref().unwrap_or("")
                ));
            }
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let tech = rel.technology.as_deref();
                if let Some(tech) = tech {
                    output.push_str(&format!(
                        "Rel({}, {}, \"{}\", \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        rel.description.as_deref().unwrap_or("uses"),
                        tech
                    ));
                } else {
                    output.push_str(&format!(
                        "Rel({}, {}, \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        rel.description.as_deref().unwrap_or("uses")
                    ));
                }
            }
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a component view to PlantUML.
    pub fn export_component(workspace: &Workspace, view: &ComponentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Component.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Find the container and its parent system
        let mut target_container = None;
        let mut parent_system = None;

        for system in &model.software_systems {
            for container in &system.containers {
                if container.id() == view.container_id {
                    target_container = Some(container);
                    parent_system = Some(system);
                    break;
                }
            }
        }

        // Add people
        for person in &model.people {
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                person.properties.description.as_deref().unwrap_or("")
            ));
        }

        output.push('\n');

        // Add the container boundary with its components
        if let Some(container) = target_container {
            output.push_str(&format!(
                "Container_Boundary({}_boundary, \"{}\") {{\n",
                sanitize_id(&container.properties.id.to_string()),
                container.name()
            ));

            for component in &container.components {
                let id = component.properties.id.to_string();
                element_ids.insert(id.clone());
                let tech = component.technology.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "  Component({}, \"{}\", \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    component.name(),
                    tech,
                    component.properties.description.as_deref().unwrap_or("")
                ));
            }

            output.push_str("}\n\n");
        }

        // Add other containers from the same system
        if let Some(system) = parent_system {
            for container in &system.containers {
                if Some(container.id()) != target_container.map(|c| c.id()) {
                    let id = container.properties.id.to_string();
                    element_ids.insert(id.clone());
                    let tech = container.technology.as_deref().unwrap_or("");
                    output.push_str(&format!(
                        "Container({}, \"{}\", \"{}\", \"{}\")\n",
                        sanitize_id(&id),
                        container.name(),
                        tech,
                        container.properties.description.as_deref().unwrap_or("")
                    ));
                }
            }
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let tech = rel.technology.as_deref();
                if let Some(tech) = tech {
                    output.push_str(&format!(
                        "Rel({}, {}, \"{}\", \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        rel.description.as_deref().unwrap_or("uses"),
                        tech
                    ));
                } else {
                    output.push_str(&format!(
                        "Rel({}, {}, \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        rel.description.as_deref().unwrap_or("uses")
                    ));
                }
            }
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a dynamic view to PlantUML (sequence diagram format).
    pub fn export_dynamic(workspace: &Workspace, view: &DynamicView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Sequence.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Collect all participants from the steps
        let mut participants = HashSet::new();

        for step in &view.steps {
            participants.insert(step.source_id.to_string());
            participants.insert(step.destination_id.to_string());
        }

        // Declare participants
        for participant_id in &participants {
            // Find the element name
            let name = find_element_name(model, participant_id);
            output.push_str(&format!(
                "participant \"{}\" as {}\n",
                name,
                sanitize_id(participant_id)
            ));
        }

        output.push('\n');

        // Add sequence steps in order
        let mut sorted_steps: Vec<_> = view.steps.iter().collect();
        sorted_steps.sort_by_key(|s| s.order);

        for step in sorted_steps {
            let label = step.description.as_deref().unwrap_or("uses");

            output.push_str(&format!(
                "{} -> {}: {}. {}\n",
                sanitize_id(&step.source_id.to_string()),
                sanitize_id(&step.destination_id.to_string()),
                step.order,
                label
            ));
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a deployment view to PlantUML.
    pub fn export_deployment(workspace: &Workspace, view: &DeploymentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        output.push_str("@startuml\n");
        output.push_str("!include https://raw.githubusercontent.com/plantuml-stdlib/C4-PlantUML/master/C4_Deployment.puml\n\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Filter deployment nodes by environment
        let environment = &view.environment;

        // Add deployment nodes
        for node in &model.deployment_nodes {
            // Skip nodes that don't match the environment
            if let Some(ref node_env) = node.environment {
                if node_env != environment {
                    continue;
                }
            }

            render_deployment_node(&mut output, node, 0);
        }

        output.push('\n');

        // Add relationships
        for rel in &model.relationships {
            let tech = rel.technology.as_deref();
            if let Some(tech) = tech {
                output.push_str(&format!(
                    "Rel({}, {}, \"{}\", \"{}\")\n",
                    sanitize_id(&rel.source_id.to_string()),
                    sanitize_id(&rel.destination_id.to_string()),
                    rel.description.as_deref().unwrap_or("uses"),
                    tech
                ));
            } else {
                output.push_str(&format!(
                    "Rel({}, {}, \"{}\")\n",
                    sanitize_id(&rel.source_id.to_string()),
                    sanitize_id(&rel.destination_id.to_string()),
                    rel.description.as_deref().unwrap_or("uses")
                ));
            }
        }

        output.push_str("\n@enduml\n");

        Ok(output)
    }

    /// Export a generic flowchart for the entire workspace.
    pub fn export_flowchart(workspace: &Workspace) -> Result<String> {
        let view = SystemLandscapeView::new("landscape");
        Self::export_system_landscape(workspace, &view)
    }
}

/// Render a deployment node recursively.
fn render_deployment_node(
    output: &mut String,
    node: &structurizr_core::model::DeploymentNode,
    indent: usize,
) {
    let indent_str = "  ".repeat(indent);
    let tech = node.technology.as_deref().unwrap_or("");

    output.push_str(&format!(
        "{}Deployment_Node({}, \"{}\", \"{}\", \"{}\") {{\n",
        indent_str,
        sanitize_id(&node.properties.id.to_string()),
        node.name(),
        tech,
        node.properties.description.as_deref().unwrap_or("")
    ));

    // Add infrastructure nodes
    for infra in &node.infrastructure_nodes {
        let infra_tech = infra.technology.as_deref().unwrap_or("");
        output.push_str(&format!(
            "{}  Deployment_Node({}, \"{}\", \"{}\", \"{}\")\n",
            indent_str,
            sanitize_id(&infra.properties.id.to_string()),
            &infra.properties.name,
            infra_tech,
            infra.properties.description.as_deref().unwrap_or("")
        ));
    }

    // Add container instances
    for instance in &node.container_instances {
        output.push_str(&format!(
            "{}  Container({}, \"{}\")\n",
            indent_str,
            sanitize_id(&instance.id.to_string()),
            instance.container_id
        ));
    }

    // Add software system instances
    for instance in &node.software_system_instances {
        output.push_str(&format!(
            "{}  System({}, \"{}\")\n",
            indent_str,
            sanitize_id(&instance.id.to_string()),
            instance.software_system_id
        ));
    }

    // Recursively add child nodes
    for child in &node.children {
        render_deployment_node(output, child, indent + 1);
    }

    output.push_str(&format!("{}}}\n", indent_str));
}

/// Find the name of an element by its ID.
fn find_element_name(model: &structurizr_core::model::Model, id: &str) -> String {
    // Check people
    for person in &model.people {
        if person.id().to_string() == id {
            return person.name().to_string();
        }
    }

    // Check software systems and their containers/components
    for system in &model.software_systems {
        if system.id().to_string() == id {
            return system.name().to_string();
        }

        for container in &system.containers {
            if container.id().to_string() == id {
                return container.name().to_string();
            }

            for component in &container.components {
                if component.id().to_string() == id {
                    return component.name().to_string();
                }
            }
        }
    }

    // Check deployment nodes
    for node in &model.deployment_nodes {
        if let Some(name) = find_deployment_node_name(node, id) {
            return name;
        }
    }

    id.to_string()
}

/// Find an element name within a deployment node hierarchy.
fn find_deployment_node_name(node: &structurizr_core::model::DeploymentNode, id: &str) -> Option<String> {
    if node.id().to_string() == id {
        return Some(node.name().to_string());
    }

    for infra in &node.infrastructure_nodes {
        if infra.id().to_string() == id {
            return Some(infra.properties.name.clone());
        }
    }

    for child in &node.children {
        if let Some(name) = find_deployment_node_name(child, id) {
            return Some(name);
        }
    }

    None
}

/// Sanitize an ID for use in PlantUML.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_system_landscape() {
        let mut workspace = Workspace::new("Test", "A test");
        workspace.model_mut().add_person("User", "A user");
        workspace.model_mut().add_software_system("System", "A system");

        let view = SystemLandscapeView::new("landscape");
        let puml = PlantUmlExporter::export_system_landscape(&workspace, &view).unwrap();

        assert!(puml.contains("@startuml"));
        assert!(puml.contains("Person"));
        assert!(puml.contains("System"));
        assert!(puml.contains("@enduml"));
    }
}
