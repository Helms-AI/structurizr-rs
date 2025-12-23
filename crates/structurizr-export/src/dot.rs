//! DOT/Graphviz export for Structurizr workspaces.
//!
//! This module exports C4 model views to DOT format for use with Graphviz.

use structurizr_core::view::{
    ContainerView, SystemContextView, SystemLandscapeView, ComponentView,
};
use structurizr_core::Workspace;

use crate::Result;

/// Exports workspaces to DOT/Graphviz format.
pub struct DotExporter;

impl DotExporter {
    /// Export a system landscape view to DOT format.
    pub fn export_system_landscape(workspace: &Workspace, _view: &SystemLandscapeView) -> Result<String> {
        let mut dot = String::new();

        dot.push_str("digraph SystemLandscape {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        let model = workspace.model();

        // Add people
        dot.push_str("  // People\n");
        for person in &model.people {
            let desc = person.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Person]\\n{}\", fillcolor=\"#08427b\", fontcolor=\"white\"];\n",
                person.id(),
                escape_dot(&person.name()),
                escape_dot(desc)
            ));
        }

        dot.push('\n');

        // Add software systems
        dot.push_str("  // Software Systems\n");
        for system in &model.software_systems {
            let desc = system.properties.description.as_deref().unwrap_or("");
            let color = if system.location == structurizr_core::model::Location::External {
                "#999999"
            } else {
                "#1168bd"
            };
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Software System]\\n{}\", fillcolor=\"{}\", fontcolor=\"white\"];\n",
                system.id(),
                escape_dot(&system.name()),
                escape_dot(desc),
                color
            ));
        }

        dot.push('\n');

        // Add relationships
        dot.push_str("  // Relationships\n");
        for rel in &model.relationships {
            let desc = rel.description.as_deref().unwrap_or("");
            let tech = rel.technology.as_ref()
                .map(|t| format!("\\n[{}]", t))
                .unwrap_or_default();

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}{}\"];\n",
                rel.source_id,
                rel.destination_id,
                escape_dot(desc),
                tech
            ));
        }

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Export a system context view to DOT format.
    pub fn export_system_context(workspace: &Workspace, view: &SystemContextView) -> Result<String> {
        let mut dot = String::new();

        dot.push_str("digraph SystemContext {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        let model = workspace.model();

        // Find the central system
        let central_system = model.software_systems.iter()
            .find(|s| s.id() == view.software_system_id);

        // Add people
        for person in &model.people {
            let desc = person.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Person]\\n{}\", fillcolor=\"#08427b\", fontcolor=\"white\"];\n",
                person.id(),
                escape_dot(&person.name()),
                escape_dot(desc)
            ));
        }

        // Add the central system (highlighted)
        if let Some(system) = central_system {
            let desc = system.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Software System]\\n{}\", fillcolor=\"#1168bd\", fontcolor=\"white\", penwidth=3];\n",
                system.id(),
                escape_dot(&system.name()),
                escape_dot(desc)
            ));
        }

        // Add external systems
        for system in &model.software_systems {
            if Some(system.id()) != central_system.map(|s| s.id()) {
                let desc = system.properties.description.as_deref().unwrap_or("");
                dot.push_str(&format!(
                    "  \"{}\" [label=\"{}\\n[Software System]\\n{}\", fillcolor=\"#999999\", fontcolor=\"white\"];\n",
                    system.id(),
                    escape_dot(&system.name()),
                    escape_dot(desc)
                ));
            }
        }

        dot.push('\n');

        // Add relationships
        for rel in &model.relationships {
            let desc = rel.description.as_deref().unwrap_or("");
            let tech = rel.technology.as_ref()
                .map(|t| format!("\\n[{}]", t))
                .unwrap_or_default();

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}{}\"];\n",
                rel.source_id,
                rel.destination_id,
                escape_dot(desc),
                tech
            ));
        }

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Export a container view to DOT format.
    pub fn export_container(workspace: &Workspace, view: &ContainerView) -> Result<String> {
        let mut dot = String::new();

        dot.push_str("digraph Container {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  compound=true;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        let model = workspace.model();

        // Find the system
        let system = model.software_systems.iter()
            .find(|s| s.id() == view.software_system_id);

        // Add people
        for person in &model.people {
            let desc = person.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Person]\\n{}\", fillcolor=\"#08427b\", fontcolor=\"white\"];\n",
                person.id(),
                escape_dot(&person.name()),
                escape_dot(desc)
            ));
        }

        // Add system boundary with containers
        if let Some(sys) = system {
            dot.push_str(&format!(
                "\n  subgraph \"cluster_{}\" {{\n",
                sys.id()
            ));
            dot.push_str(&format!(
                "    label=\"{}\\n[Software System]\";\n",
                escape_dot(&sys.name())
            ));
            dot.push_str("    style=dashed;\n");
            dot.push_str("    color=\"#1168bd\";\n\n");

            for container in &sys.containers {
                let desc = container.properties.description.as_deref().unwrap_or("");
                let tech = container.technology.as_ref()
                    .map(|t| format!("\\n[{}]", t))
                    .unwrap_or_default();

                dot.push_str(&format!(
                    "    \"{}\" [label=\"{}\\n[Container{}]\\n{}\", fillcolor=\"#438dd5\", fontcolor=\"white\"];\n",
                    container.id(),
                    escape_dot(&container.name()),
                    tech,
                    escape_dot(desc)
                ));
            }

            dot.push_str("  }\n\n");
        }

        // Add external systems
        for sys in &model.software_systems {
            if Some(sys.id()) != system.map(|s| s.id()) {
                let desc = sys.properties.description.as_deref().unwrap_or("");
                dot.push_str(&format!(
                    "  \"{}\" [label=\"{}\\n[Software System]\\n{}\", fillcolor=\"#999999\", fontcolor=\"white\"];\n",
                    sys.id(),
                    escape_dot(&sys.name()),
                    escape_dot(desc)
                ));
            }
        }

        dot.push('\n');

        // Add relationships
        for rel in &model.relationships {
            let desc = rel.description.as_deref().unwrap_or("");
            let tech = rel.technology.as_ref()
                .map(|t| format!("\\n[{}]", t))
                .unwrap_or_default();

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}{}\"];\n",
                rel.source_id,
                rel.destination_id,
                escape_dot(desc),
                tech
            ));
        }

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Export a component view to DOT format.
    pub fn export_component(workspace: &Workspace, view: &ComponentView) -> Result<String> {
        let mut dot = String::new();

        dot.push_str("digraph Component {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  compound=true;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        let model = workspace.model();

        // Find the container
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
            dot.push_str(&format!(
                "  subgraph \"cluster_{}\" {{\n",
                container.id()
            ));
            dot.push_str(&format!(
                "    label=\"{}\\n[Container]\";\n",
                escape_dot(&container.name())
            ));
            dot.push_str("    style=dashed;\n");
            dot.push_str("    color=\"#438dd5\";\n\n");

            for component in &container.components {
                let desc = component.properties.description.as_deref().unwrap_or("");
                let tech = component.technology.as_ref()
                    .map(|t| format!("\\n[{}]", t))
                    .unwrap_or_default();

                dot.push_str(&format!(
                    "    \"{}\" [label=\"{}\\n[Component{}]\\n{}\", fillcolor=\"#85bbf0\", fontcolor=\"black\"];\n",
                    component.id(),
                    escape_dot(&component.name()),
                    tech,
                    escape_dot(desc)
                ));
            }

            dot.push_str("  }\n\n");
        }

        // Add other containers from the same system
        if let Some(system) = parent_system {
            for container in &system.containers {
                if Some(container.id()) != target_container.map(|c| c.id()) {
                    let desc = container.properties.description.as_deref().unwrap_or("");
                    let tech = container.technology.as_ref()
                        .map(|t| format!("\\n[{}]", t))
                        .unwrap_or_default();

                    dot.push_str(&format!(
                        "  \"{}\" [label=\"{}\\n[Container{}]\\n{}\", fillcolor=\"#438dd5\", fontcolor=\"white\"];\n",
                        container.id(),
                        escape_dot(&container.name()),
                        tech,
                        escape_dot(desc)
                    ));
                }
            }
        }

        dot.push('\n');

        // Add relationships
        for rel in &model.relationships {
            let desc = rel.description.as_deref().unwrap_or("");
            let tech = rel.technology.as_ref()
                .map(|t| format!("\\n[{}]", t))
                .unwrap_or_default();

            dot.push_str(&format!(
                "  \"{}\" -> \"{}\" [label=\"{}{}\"];\n",
                rel.source_id,
                rel.destination_id,
                escape_dot(desc),
                tech
            ));
        }

        dot.push_str("}\n");

        Ok(dot)
    }

    /// Export a generic flowchart for the entire workspace.
    pub fn export_flowchart(workspace: &Workspace) -> Result<String> {
        let view = SystemLandscapeView::new("landscape");
        Self::export_system_landscape(workspace, &view)
    }
}

/// Escape special characters for DOT format.
fn escape_dot(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_system_landscape() {
        let mut workspace = Workspace::new("Test", "Test workspace");

        workspace.model_mut().add_person("User", "A user");
        workspace.model_mut().add_software_system("System", "A system");

        let view = SystemLandscapeView::new("landscape");
        let dot = DotExporter::export_system_landscape(&workspace, &view).unwrap();

        assert!(dot.contains("digraph SystemLandscape"));
        assert!(dot.contains("User"));
        assert!(dot.contains("System"));
    }
}
