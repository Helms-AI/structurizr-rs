//! D2 export for Structurizr views.
//!
//! D2 is a modern declarative diagramming language: https://d2lang.com

use structurizr_core::view::{
    ComponentView, ContainerView, DeploymentView, DynamicView, SystemContextView,
    SystemLandscapeView,
};
use structurizr_core::Workspace;

use crate::error::Result;

/// Exports views to D2 diagram format.
pub struct D2Exporter;

impl D2Exporter {
    /// Export a system landscape view to D2.
    pub fn export_system_landscape(workspace: &Workspace, view: &SystemLandscapeView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        // Add title
        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        // Add direction
        output.push_str("direction: down\n\n");

        // Add people with person shape
        for person in &model.people {
            let id = sanitize_id(&person.id().to_string());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  shape: person\n",
                id,
                person.name()
            ));
            if !desc.is_empty() {
                output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
            }
            output.push_str("  style.fill: \"#08427b\"\n  style.font-color: white\n}\n\n");
        }

        // Add software systems
        for system in &model.software_systems {
            let id = sanitize_id(&system.id().to_string());
            let desc = system.properties.description.as_deref().unwrap_or("");
            let color = if system.location == structurizr_core::model::Location::External {
                "#999999"
            } else {
                "#1168bd"
            };
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  shape: rectangle\n",
                id,
                system.name()
            ));
            if !desc.is_empty() {
                output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
            }
            output.push_str(&format!("  style.fill: \"{}\"\n  style.font-color: white\n  style.border-radius: 8\n}}\n\n", color));
        }

        // Add relationships
        for rel in &model.relationships {
            let source = sanitize_id(&rel.source_id.to_string());
            let target = sanitize_id(&rel.destination_id.to_string());
            let desc = rel.description.as_deref().unwrap_or("uses");
            let tech = rel.technology.as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "{} -> {}: \"{}{}\"\n",
                source, target, desc, tech
            ));
        }

        Ok(output)
    }

    /// Export a system context view to D2.
    pub fn export_system_context(workspace: &Workspace, view: &SystemContextView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        output.push_str("direction: down\n\n");

        // Find the central system
        let central_system = model.software_systems.iter()
            .find(|s| s.id() == view.software_system_id);

        // Add people
        for person in &model.people {
            let id = sanitize_id(&person.id().to_string());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  shape: person\n",
                id,
                person.name()
            ));
            if !desc.is_empty() {
                output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
            }
            output.push_str("  style.fill: \"#08427b\"\n  style.font-color: white\n}\n\n");
        }

        // Add software systems
        for system in &model.software_systems {
            let id = sanitize_id(&system.id().to_string());
            let desc = system.properties.description.as_deref().unwrap_or("");
            let is_central = central_system.map(|c| c.id() == system.id()).unwrap_or(false);
            let color = if is_central { "#1168bd" } else { "#999999" };
            let stroke_width = if is_central { 3 } else { 1 };

            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  shape: rectangle\n",
                id,
                system.name()
            ));
            if !desc.is_empty() {
                output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
            }
            output.push_str(&format!(
                "  style.fill: \"{}\"\n  style.font-color: white\n  style.border-radius: 8\n  style.stroke-width: {}\n}}\n\n",
                color, stroke_width
            ));
        }

        // Add relationships
        for rel in &model.relationships {
            let source = sanitize_id(&rel.source_id.to_string());
            let target = sanitize_id(&rel.destination_id.to_string());
            let desc = rel.description.as_deref().unwrap_or("uses");
            let tech = rel.technology.as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "{} -> {}: \"{}{}\"\n",
                source, target, desc, tech
            ));
        }

        Ok(output)
    }

    /// Export a container view to D2.
    pub fn export_container(workspace: &Workspace, view: &ContainerView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        output.push_str("direction: down\n\n");

        // Add people
        for person in &model.people {
            let id = sanitize_id(&person.id().to_string());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  shape: person\n",
                id,
                person.name()
            ));
            if !desc.is_empty() {
                output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
            }
            output.push_str("  style.fill: \"#08427b\"\n  style.font-color: white\n}\n\n");
        }

        // Find the system and add containers as a group
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            let sys_id = sanitize_id(&system.id().to_string());
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  style.stroke-dash: 5\n  style.stroke: \"#1168bd\"\n\n",
                sys_id,
                system.name()
            ));

            for container in &system.containers {
                let container_id = sanitize_id(&container.id().to_string());
                let desc = container.properties.description.as_deref().unwrap_or("");
                let tech = container.technology.as_deref().unwrap_or("");

                output.push_str(&format!(
                    "  {}: {{\n    label: \"{}\"\n    shape: rectangle\n",
                    container_id,
                    container.name()
                ));
                if !tech.is_empty() {
                    output.push_str(&format!("    near: bottom-center\n"));
                }
                if !desc.is_empty() {
                    output.push_str(&format!("    tooltip: \"{}\"\n", escape_d2(desc)));
                }
                output.push_str("    style.fill: \"#438dd5\"\n    style.font-color: white\n    style.border-radius: 8\n  }\n\n");
            }

            output.push_str("}\n\n");
        }

        // Add external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                let id = sanitize_id(&system.id().to_string());
                let desc = system.properties.description.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "{}: {{\n  label: \"{}\"\n  shape: rectangle\n",
                    id,
                    system.name()
                ));
                if !desc.is_empty() {
                    output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
                }
                output.push_str("  style.fill: \"#999999\"\n  style.font-color: white\n  style.border-radius: 8\n}\n\n");
            }
        }

        // Add relationships
        for rel in &model.relationships {
            let source = sanitize_id(&rel.source_id.to_string());
            let target = sanitize_id(&rel.destination_id.to_string());
            let desc = rel.description.as_deref().unwrap_or("uses");
            let tech = rel.technology.as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "{} -> {}: \"{}{}\"\n",
                source, target, desc, tech
            ));
        }

        Ok(output)
    }

    /// Export a component view to D2.
    pub fn export_component(workspace: &Workspace, view: &ComponentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        output.push_str("direction: down\n\n");

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

        // Add container boundary with components
        if let Some(container) = target_container {
            let container_id = sanitize_id(&container.id().to_string());
            output.push_str(&format!(
                "{}: {{\n  label: \"{}\"\n  style.stroke-dash: 5\n  style.stroke: \"#438dd5\"\n\n",
                container_id,
                container.name()
            ));

            for component in &container.components {
                let comp_id = sanitize_id(&component.id().to_string());
                let desc = component.properties.description.as_deref().unwrap_or("");
                let tech = component.technology.as_deref().unwrap_or("");

                output.push_str(&format!(
                    "  {}: {{\n    label: \"{}\"\n    shape: rectangle\n",
                    comp_id,
                    component.name()
                ));
                if !tech.is_empty() {
                    output.push_str(&format!("    # technology: {}\n", tech));
                }
                if !desc.is_empty() {
                    output.push_str(&format!("    tooltip: \"{}\"\n", escape_d2(desc)));
                }
                output.push_str("    style.fill: \"#85bbf0\"\n    style.font-color: black\n    style.border-radius: 8\n  }\n\n");
            }

            output.push_str("}\n\n");
        }

        // Add other containers from the same system
        if let Some(system) = parent_system {
            for container in &system.containers {
                if Some(container.id()) != target_container.map(|c| c.id()) {
                    let id = sanitize_id(&container.id().to_string());
                    let desc = container.properties.description.as_deref().unwrap_or("");
                    output.push_str(&format!(
                        "{}: {{\n  label: \"{}\"\n  shape: rectangle\n",
                        id,
                        container.name()
                    ));
                    if !desc.is_empty() {
                        output.push_str(&format!("  tooltip: \"{}\"\n", escape_d2(desc)));
                    }
                    output.push_str("  style.fill: \"#438dd5\"\n  style.font-color: white\n  style.border-radius: 8\n}\n\n");
                }
            }
        }

        // Add relationships
        for rel in &model.relationships {
            let source = sanitize_id(&rel.source_id.to_string());
            let target = sanitize_id(&rel.destination_id.to_string());
            let desc = rel.description.as_deref().unwrap_or("uses");
            let tech = rel.technology.as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "{} -> {}: \"{}{}\"\n",
                source, target, desc, tech
            ));
        }

        Ok(output)
    }

    /// Export a dynamic view to D2 (sequence diagram).
    pub fn export_dynamic(workspace: &Workspace, view: &DynamicView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        // D2 supports sequence diagrams natively
        output.push_str("shape: sequence_diagram\n\n");

        // Collect all participants
        let mut participants = std::collections::HashSet::new();
        for step in &view.steps {
            participants.insert(step.source_id.to_string());
            participants.insert(step.destination_id.to_string());
        }

        // Declare participants
        for participant_id in &participants {
            let name = find_element_name(model, participant_id);
            let id = sanitize_id(participant_id);
            output.push_str(&format!("{}: {}\n", id, name));
        }

        output.push('\n');

        // Add sequence steps in order
        let mut sorted_steps: Vec<_> = view.steps.iter().collect();
        sorted_steps.sort_by_key(|s| s.order);

        for step in sorted_steps {
            let source = sanitize_id(&step.source_id.to_string());
            let target = sanitize_id(&step.destination_id.to_string());
            let label = step.description.as_deref().unwrap_or("uses");

            output.push_str(&format!(
                "{} -> {}: {}. {}\n",
                source, target, step.order, label
            ));
        }

        Ok(output)
    }

    /// Export a deployment view to D2.
    pub fn export_deployment(workspace: &Workspace, view: &DeploymentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("# {}\n\n", title));
        }

        output.push_str("direction: down\n\n");

        // Filter and render deployment nodes
        let environment = &view.environment;

        for node in &model.deployment_nodes {
            if let Some(ref node_env) = node.environment {
                if node_env != environment {
                    continue;
                }
            }

            render_d2_deployment_node(&mut output, node, 0);
        }

        // Add relationships
        output.push('\n');
        for rel in &model.relationships {
            let source = sanitize_id(&rel.source_id.to_string());
            let target = sanitize_id(&rel.destination_id.to_string());
            let desc = rel.description.as_deref().unwrap_or("uses");
            let tech = rel.technology.as_ref()
                .map(|t| format!(" [{}]", t))
                .unwrap_or_default();

            output.push_str(&format!(
                "{} -> {}: \"{}{}\"\n",
                source, target, desc, tech
            ));
        }

        Ok(output)
    }

    /// Export a generic flowchart for the entire workspace.
    pub fn export_flowchart(workspace: &Workspace) -> Result<String> {
        let view = SystemLandscapeView::new("landscape");
        Self::export_system_landscape(workspace, &view)
    }
}

/// Render a deployment node for D2.
fn render_d2_deployment_node(
    output: &mut String,
    node: &structurizr_core::model::DeploymentNode,
    indent: usize,
) {
    let indent_str = "  ".repeat(indent);
    let id = sanitize_id(&node.properties.id.to_string());
    let tech = node.technology.as_deref().unwrap_or("");

    output.push_str(&format!(
        "{}{}: {{\n{}  label: \"{}\"\n",
        indent_str,
        id,
        indent_str,
        node.name()
    ));

    if !tech.is_empty() {
        output.push_str(&format!("{}  # technology: {}\n", indent_str, tech));
    }

    output.push_str(&format!(
        "{}  style.stroke-dash: 5\n{}  style.stroke: \"#666666\"\n\n",
        indent_str, indent_str
    ));

    // Add infrastructure nodes
    for infra in &node.infrastructure_nodes {
        let infra_id = sanitize_id(&infra.properties.id.to_string());
        let infra_tech = infra.technology.as_deref().unwrap_or("");
        output.push_str(&format!(
            "{}  {}: {{\n{}    label: \"{}\"\n{}    shape: rectangle\n",
            indent_str, infra_id, indent_str, &infra.properties.name, indent_str
        ));
        if !infra_tech.is_empty() {
            output.push_str(&format!("{}    # technology: {}\n", indent_str, infra_tech));
        }
        output.push_str(&format!("{}    style.fill: \"#666666\"\n{}    style.font-color: white\n{}  }}\n\n", indent_str, indent_str, indent_str));
    }

    // Add container instances
    for instance in &node.container_instances {
        let inst_id = sanitize_id(&instance.id.to_string());
        output.push_str(&format!(
            "{}  {}: {{\n{}    label: \"{}\"\n{}    shape: rectangle\n{}    style.fill: \"#438dd5\"\n{}    style.font-color: white\n{}  }}\n\n",
            indent_str, inst_id, indent_str, instance.container_id, indent_str, indent_str, indent_str, indent_str
        ));
    }

    // Add software system instances
    for instance in &node.software_system_instances {
        let inst_id = sanitize_id(&instance.id.to_string());
        output.push_str(&format!(
            "{}  {}: {{\n{}    label: \"{}\"\n{}    shape: rectangle\n{}    style.fill: \"#1168bd\"\n{}    style.font-color: white\n{}  }}\n\n",
            indent_str, inst_id, indent_str, instance.software_system_id, indent_str, indent_str, indent_str, indent_str
        ));
    }

    // Recursively add child nodes
    for child in &node.children {
        render_d2_deployment_node(output, child, indent + 1);
    }

    output.push_str(&format!("{}}}\n\n", indent_str));
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

    id.to_string()
}

/// Sanitize an ID for use in D2.
fn sanitize_id(id: &str) -> String {
    let s: String = id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect();

    // D2 IDs can't start with a number, prefix with underscore if needed
    if s.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
        format!("_{}", s)
    } else {
        s
    }
}

/// Escape special characters for D2 strings.
fn escape_d2(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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
        let d2 = D2Exporter::export_system_landscape(&workspace, &view).unwrap();

        assert!(d2.contains("direction: down"));
        assert!(d2.contains("shape: person"));
        assert!(d2.contains("shape: rectangle"));
    }
}
