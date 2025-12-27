//! DOT/Graphviz export for Structurizr workspaces.
//!
//! This module exports C4 model views to DOT format for use with Graphviz.

use std::collections::HashSet;

use structurizr_core::model::ElementId;
use structurizr_core::view::{
    ContainerView, SystemContextView, SystemLandscapeView, ComponentView,
};
use structurizr_core::Workspace;

use crate::Result;

/// Exports workspaces to DOT/Graphviz format.
pub struct DotExporter;

impl DotExporter {
    /// Export a system landscape view to DOT format.
    pub fn export_system_landscape(workspace: &Workspace, view: &SystemLandscapeView) -> Result<String> {
        let mut dot = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build allowed element set from view.properties.elements if non-empty
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        let model = workspace.model();

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        // Include containers/components as "proxy candidates" so person→container relationships count
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        // Add candidate people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        // Add candidate software systems AND their containers/components as proxy candidates
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

        dot.push_str("digraph SystemLandscape {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people (connected if they OR any container/component they relate to is connected)
        dot.push_str("  // People\n");
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.insert(person.id().to_string());
            let desc = person.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Person]\\n{}\", fillcolor=\"#08427b\", fontcolor=\"white\"];\n",
                person.id(),
                escape_dot(&person.name()),
                escape_dot(desc)
            ));
        }

        dot.push('\n');

        // Add software systems (connected if system OR any of its containers/components is connected)
        dot.push_str("  // Software Systems\n");
        for system in &model.software_systems {
            if !candidate_ids.contains(&system.id()) { continue; }
            // System is connected if: itself is connected OR any of its containers/components are
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if !system_connected { continue; }
            element_ids.insert(system.id().to_string());
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

        // Add relationships (only between elements in this view)
        dot.push_str("  // Relationships\n");
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if !element_ids.contains(&source) || !element_ids.contains(&dest) {
                continue;
            }
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
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build allowed element set from view.properties.elements if non-empty
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        let model = workspace.model();

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        // Include containers/components as "proxy candidates" so person→container relationships count
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        // Add candidate people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        // Add candidate software systems AND their containers/components as proxy candidates
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

        dot.push_str("digraph SystemContext {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        // Find the central system
        let central_system = model.software_systems.iter()
            .find(|s| s.id() == view.software_system_id);

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people (connected if they OR any container/component they relate to is connected)
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.insert(person.id().to_string());
            let desc = person.properties.description.as_deref().unwrap_or("");
            dot.push_str(&format!(
                "  \"{}\" [label=\"{}\\n[Person]\\n{}\", fillcolor=\"#08427b\", fontcolor=\"white\"];\n",
                person.id(),
                escape_dot(&person.name()),
                escape_dot(desc)
            ));
        }

        // Add the central system (highlighted) - connected if itself OR any children are connected
        if let Some(system) = central_system {
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if candidate_ids.contains(&system.id()) && system_connected {
                element_ids.insert(system.id().to_string());
                let desc = system.properties.description.as_deref().unwrap_or("");
                dot.push_str(&format!(
                    "  \"{}\" [label=\"{}\\n[Software System]\\n{}\", fillcolor=\"#1168bd\", fontcolor=\"white\", penwidth=3];\n",
                    system.id(),
                    escape_dot(&system.name()),
                    escape_dot(desc)
                ));
            }
        }

        // Add external systems (connected if system OR any of its containers/components is connected)
        for system in &model.software_systems {
            if Some(system.id()) != central_system.map(|s| s.id()) {
                if !candidate_ids.contains(&system.id()) { continue; }
                // System is connected if: itself is connected OR any of its containers/components are
                let system_connected = connected_ids.contains(&system.id()) ||
                    system.containers.iter().any(|c| {
                        connected_ids.contains(&c.id()) ||
                        c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                    });
                if !system_connected { continue; }
                element_ids.insert(system.id().to_string());
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

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if !element_ids.contains(&source) || !element_ids.contains(&dest) {
                continue;
            }
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
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build allowed element set from view.properties.elements if non-empty
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        let model = workspace.model();

        // Find the system
        let system = model.software_systems.iter()
            .find(|s| s.id() == view.software_system_id);

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        // Add candidate people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        // Add candidate containers from the main system
        if let Some(sys) = system {
            for container in &sys.containers {
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&container.id()) { continue; }
                }
                candidate_ids.insert(container.id());
            }
        }

        // Add candidate external systems
        for sys in &model.software_systems {
            if Some(sys.id()) != system.map(|s| s.id()) {
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&sys.id()) { continue; }
                }
                candidate_ids.insert(sys.id());
            }
        }

        // Step 2: Build connected_ids from relationships where BOTH endpoints are candidates
        let connected_ids: HashSet<ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        dot.push_str("digraph Container {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  compound=true;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.insert(person.id().to_string());
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
                if !candidate_ids.contains(&container.id()) { continue; }
                if !connected_ids.contains(&container.id()) { continue; }
                element_ids.insert(container.id().to_string());
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
                if !candidate_ids.contains(&sys.id()) { continue; }
                if !connected_ids.contains(&sys.id()) { continue; }
                element_ids.insert(sys.id().to_string());
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

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if !element_ids.contains(&source) || !element_ids.contains(&dest) {
                continue;
            }
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
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build allowed element set from view.properties.elements if non-empty
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

        let model = workspace.model();

        // Find the container and its parent system first (needed for candidate collection)
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

        // Step 1: Collect candidate element IDs for this view (respecting allowed_ids)
        let mut candidate_ids: HashSet<ElementId> = HashSet::new();

        // Add candidate people
        for person in &model.people {
            if let Some(ref allowed) = allowed_ids {
                if !allowed.contains(&person.id()) { continue; }
            }
            candidate_ids.insert(person.id());
        }

        // Add candidate components from the target container
        if let Some(container) = target_container {
            for component in &container.components {
                if let Some(ref allowed) = allowed_ids {
                    if !allowed.contains(&component.id()) { continue; }
                }
                candidate_ids.insert(component.id());
            }
        }

        // Add candidate other containers from the same system
        if let Some(system) = parent_system {
            for container in &system.containers {
                if Some(container.id()) != target_container.map(|c| c.id()) {
                    if let Some(ref allowed) = allowed_ids {
                        if !allowed.contains(&container.id()) { continue; }
                    }
                    candidate_ids.insert(container.id());
                }
            }
        }

        // Step 2: Build connected_ids from relationships where BOTH endpoints are candidates
        let connected_ids: HashSet<ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        dot.push_str("digraph Component {\n");
        dot.push_str("  rankdir=TB;\n");
        dot.push_str("  compound=true;\n");
        dot.push_str("  node [shape=box, style=\"rounded,filled\", fontname=\"Arial\"];\n");
        dot.push_str("  edge [fontname=\"Arial\", fontsize=10];\n\n");

        // Step 3: Add elements that are both candidates AND connected within this view
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
                if !candidate_ids.contains(&component.id()) { continue; }
                if !connected_ids.contains(&component.id()) { continue; }
                element_ids.insert(component.id().to_string());
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
                    if !candidate_ids.contains(&container.id()) { continue; }
                    if !connected_ids.contains(&container.id()) { continue; }
                    element_ids.insert(container.id().to_string());
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

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if !element_ids.contains(&source) || !element_ids.contains(&dest) {
                continue;
            }
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

        let user = workspace.model_mut().add_person("User", "A user");
        let system = workspace.model_mut().add_software_system("System", "A system");
        workspace.model_mut().add_relationship(user, system, "Uses", None);

        let view = SystemLandscapeView::new("landscape");
        let dot = DotExporter::export_system_landscape(&workspace, &view).unwrap();

        assert!(dot.contains("digraph SystemLandscape"));
        assert!(dot.contains("User"));
        assert!(dot.contains("System"));
    }
}
