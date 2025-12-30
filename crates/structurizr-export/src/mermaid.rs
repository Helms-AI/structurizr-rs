//! Mermaid export for Structurizr views.

use std::collections::HashSet;

use structurizr_core::model::ElementId;
use structurizr_core::view::{
    ComponentView, ContainerView, DeploymentView, DynamicView, SystemContextView,
    SystemLandscapeView,
};
use structurizr_core::Workspace;

use crate::error::Result;

/// Exports views to Mermaid diagram format.
pub struct MermaidExporter;

impl MermaidExporter {
    /// Export a system landscape view to Mermaid.
    pub fn export_system_landscape(workspace: &Workspace, view: &SystemLandscapeView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

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

        output.push_str("```mermaid\n");
        output.push_str("C4Context\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title {}\n", title));
        }

        output.push('\n');

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people (connected if they OR any container/component they relate to is connected)
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "    Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                desc
            ));
        }

        // Add software systems (connected if system OR any of its containers/components is connected)
        for system in &model.software_systems {
            if !candidate_ids.contains(&system.id()) { continue; }
            // System is connected if: itself is connected OR any of its containers/components are
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if !system_connected { continue; }
            let id = system.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = system.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "    System({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                system.name(),
                desc
            ));
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let desc = rel.description.as_deref().unwrap_or("uses");
                output.push_str(&format!(
                    "    Rel({}, {}, \"{}\")\n",
                    sanitize_id(&source),
                    sanitize_id(&dest),
                    desc
                ));
            }
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export a system context view to Mermaid.
    pub fn export_system_context(workspace: &Workspace, view: &SystemContextView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

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

        output.push_str("```mermaid\n");
        output.push_str("C4Context\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title {}\n", title));
        }

        output.push('\n');

        // Find the main system
        let main_system = model.software_systems.iter().find(|s| s.id() == view.software_system_id);

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people (connected if they OR any container/component they relate to is connected)
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "    Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                desc
            ));
        }

        // Add software systems (connected if system OR any of its containers/components is connected)
        for system in &model.software_systems {
            if !candidate_ids.contains(&system.id()) { continue; }
            // System is connected if: itself is connected OR any of its containers/components are
            let system_connected = connected_ids.contains(&system.id()) ||
                system.containers.iter().any(|c| {
                    connected_ids.contains(&c.id()) ||
                    c.components.iter().any(|comp| connected_ids.contains(&comp.id()))
                });
            if !system_connected { continue; }
            let id = system.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = system.properties.description.as_deref().unwrap_or("");
            let is_main = main_system.map(|m| m.id() == system.id()).unwrap_or(false);

            if is_main {
                output.push_str(&format!(
                    "    System({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    desc
                ));
            } else {
                output.push_str(&format!(
                    "    System_Ext({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    desc
                ));
            }
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let desc = rel.description.as_deref().unwrap_or("uses");
                output.push_str(&format!(
                    "    Rel({}, {}, \"{}\")\n",
                    sanitize_id(&source),
                    sanitize_id(&dest),
                    desc
                ));
            }
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export a container view to Mermaid.
    pub fn export_container(workspace: &Workspace, view: &ContainerView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

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

        // Add candidate containers from the main system
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

        output.push_str("```mermaid\n");
        output.push_str("C4Container\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title {}\n", title));
        }

        output.push('\n');

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "    Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                desc
            ));
        }

        // Find the main system and add boundary with containers
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            let system_id = system.properties.id.to_string();
            element_ids.insert(system_id.clone());

            output.push_str(&format!(
                "\n    System_Boundary({}, \"{}\") {{\n",
                sanitize_id(&system_id),
                system.name()
            ));

            for container in &system.containers {
                if !candidate_ids.contains(&container.id()) { continue; }
                if !connected_ids.contains(&container.id()) { continue; }
                let container_id = container.properties.id.to_string();
                element_ids.insert(container_id.clone());
                let tech = container.technology.as_deref().unwrap_or("");
                let desc = container.properties.description.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "        Container({}, \"{}\", \"{}\", \"{}\")\n",
                    sanitize_id(&container_id),
                    container.name(),
                    tech,
                    desc
                ));
            }

            output.push_str("    }\n");
        }

        // Add external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                if !candidate_ids.contains(&system.id()) { continue; }
                if !connected_ids.contains(&system.id()) { continue; }
                let id = system.properties.id.to_string();
                element_ids.insert(id.clone());
                let desc = system.properties.description.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "    System_Ext({}, \"{}\", \"{}\")\n",
                    sanitize_id(&id),
                    system.name(),
                    desc
                ));
            }
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let desc = rel.description.as_deref().unwrap_or("uses");
                if let Some(ref tech) = rel.technology {
                    output.push_str(&format!(
                        "    Rel({}, {}, \"{}\", \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        desc,
                        tech
                    ));
                } else {
                    output.push_str(&format!(
                        "    Rel({}, {}, \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        desc
                    ));
                }
            }
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export a component view to Mermaid.
    pub fn export_component(workspace: &Workspace, view: &ComponentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        // Build set of allowed element IDs if view has explicit elements
        let allowed_ids: Option<HashSet<ElementId>> = if !view.properties.elements.is_empty() {
            Some(view.properties.elements.iter().map(|e| e.id).collect())
        } else {
            None
        };

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

        output.push_str("```mermaid\n");
        output.push_str("C4Component\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title {}\n", title));
        }

        output.push('\n');

        // Step 3: Add elements that are both candidates AND connected within this view
        // Add people
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            let desc = person.properties.description.as_deref().unwrap_or("");
            output.push_str(&format!(
                "    Person({}, \"{}\", \"{}\")\n",
                sanitize_id(&id),
                person.name(),
                desc
            ));
        }

        // Add the container boundary with its components
        if let Some(container) = target_container {
            let container_id = container.properties.id.to_string();
            element_ids.insert(container_id.clone());

            output.push_str(&format!(
                "\n    Container_Boundary({}, \"{}\") {{\n",
                sanitize_id(&container_id),
                container.name()
            ));

            for component in &container.components {
                if !candidate_ids.contains(&component.id()) { continue; }
                if !connected_ids.contains(&component.id()) { continue; }
                let component_id = component.properties.id.to_string();
                element_ids.insert(component_id.clone());
                let tech = component.technology.as_deref().unwrap_or("");
                let desc = component.properties.description.as_deref().unwrap_or("");
                output.push_str(&format!(
                    "        Component({}, \"{}\", \"{}\", \"{}\")\n",
                    sanitize_id(&component_id),
                    component.name(),
                    tech,
                    desc
                ));
            }

            output.push_str("    }\n");
        }

        // Add other containers from the same system
        if let Some(system) = parent_system {
            for container in &system.containers {
                if Some(container.id()) != target_container.map(|c| c.id()) {
                    if !candidate_ids.contains(&container.id()) { continue; }
                    if !connected_ids.contains(&container.id()) { continue; }
                    let id = container.properties.id.to_string();
                    element_ids.insert(id.clone());
                    let tech = container.technology.as_deref().unwrap_or("");
                    let desc = container.properties.description.as_deref().unwrap_or("");
                    output.push_str(&format!(
                        "    Container({}, \"{}\", \"{}\", \"{}\")\n",
                        sanitize_id(&id),
                        container.name(),
                        tech,
                        desc
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
                let desc = rel.description.as_deref().unwrap_or("uses");
                if let Some(ref tech) = rel.technology {
                    output.push_str(&format!(
                        "    Rel({}, {}, \"{}\", \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        desc,
                        tech
                    ));
                } else {
                    output.push_str(&format!(
                        "    Rel({}, {}, \"{}\")\n",
                        sanitize_id(&source),
                        sanitize_id(&dest),
                        desc
                    ));
                }
            }
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export a dynamic view to Mermaid (sequence diagram).
    pub fn export_dynamic(workspace: &Workspace, view: &DynamicView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        output.push_str("```mermaid\n");
        output.push_str("sequenceDiagram\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title: {}\n", title));
        }

        output.push('\n');

        // Collect all participants from the steps
        let mut participants = std::collections::HashSet::new();

        for step in &view.steps {
            participants.insert(step.source_id.to_string());
            participants.insert(step.destination_id.to_string());
        }

        // Declare participants
        for participant_id in &participants {
            let name = find_element_name(model, participant_id);
            output.push_str(&format!(
                "    participant {} as {}\n",
                sanitize_id(participant_id),
                name
            ));
        }

        output.push('\n');

        // Add sequence steps in order (handles "1", "2", "2.1", "2.2" notation)
        let mut sorted_steps: Vec<_> = view.steps.iter().collect();
        sorted_steps.sort_by(|a, b| compare_order_strings(&a.order, &b.order));

        for step in sorted_steps {
            let label = step.description.as_deref().unwrap_or("uses");
            output.push_str(&format!(
                "    {}->>+{}: {}. {}\n",
                sanitize_id(&step.source_id.to_string()),
                sanitize_id(&step.destination_id.to_string()),
                step.order,
                label
            ));
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export a deployment view to Mermaid.
    pub fn export_deployment(workspace: &Workspace, view: &DeploymentView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        output.push_str("```mermaid\n");
        output.push_str("C4Deployment\n");

        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("    title {}\n", title));
        }

        output.push('\n');

        // Filter and render deployment nodes
        let environment = &view.environment;

        for node in &model.deployment_nodes {
            if let Some(ref node_env) = node.environment {
                if node_env != environment {
                    continue;
                }
            }

            render_mermaid_deployment_node(&mut output, node, 1);
        }

        output.push_str("```\n");

        Ok(output)
    }

    /// Export to a simple flowchart format (more widely supported).
    pub fn export_flowchart(workspace: &Workspace) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();
        let mut element_ids: HashSet<String> = HashSet::new();

        output.push_str("```mermaid\n");
        output.push_str("flowchart TB\n");

        // Add people
        for person in &model.people {
            let id = person.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "    {}[\"{}\"]\n",
                sanitize_id(&id),
                person.name()
            ));
        }

        // Add software systems
        for system in &model.software_systems {
            let id = system.properties.id.to_string();
            element_ids.insert(id.clone());
            output.push_str(&format!(
                "    {}[\"{}\"]\n",
                sanitize_id(&id),
                system.name()
            ));
        }

        output.push('\n');

        // Add relationships (only between elements in this view)
        for rel in &model.relationships {
            let source = rel.source_id.to_string();
            let dest = rel.destination_id.to_string();
            if element_ids.contains(&source) && element_ids.contains(&dest) {
                let desc = rel.description.as_deref().unwrap_or("uses");
                output.push_str(&format!(
                    "    {} -->|\"{}\"| {}\n",
                    sanitize_id(&source), desc, sanitize_id(&dest)
                ));
            }
        }

        output.push_str("```\n");

        Ok(output)
    }
}

/// Sanitize an ID for use in Mermaid.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

/// Compare order strings like "1", "2", "2.1", "2.2", "3" for proper sorting.
/// Handles hierarchical numbering for parallel sequences.
fn compare_order_strings(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<u32> = a.split('.').filter_map(|s| s.parse().ok()).collect();
    let b_parts: Vec<u32> = b.split('.').filter_map(|s| s.parse().ok()).collect();
    a_parts.cmp(&b_parts)
}

/// Render a deployment node for Mermaid.
fn render_mermaid_deployment_node(
    output: &mut String,
    node: &structurizr_core::model::DeploymentNode,
    indent: usize,
) {
    let indent_str = "    ".repeat(indent);
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
            "{}    Node({}, \"{}\", \"{}\")\n",
            indent_str,
            sanitize_id(&infra.properties.id.to_string()),
            &infra.properties.name,
            infra_tech
        ));
    }

    // Add container instances
    for instance in &node.container_instances {
        output.push_str(&format!(
            "{}    Container({}, \"{}\")\n",
            indent_str,
            sanitize_id(&instance.id.to_string()),
            instance.container_id
        ));
    }

    // Add software system instances
    for instance in &node.software_system_instances {
        output.push_str(&format!(
            "{}    System({}, \"{}\")\n",
            indent_str,
            sanitize_id(&instance.id.to_string()),
            instance.software_system_id
        ));
    }

    // Recursively add child nodes
    for child in &node.children {
        render_mermaid_deployment_node(output, child, indent + 1);
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

    id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_flowchart() {
        let mut workspace = Workspace::new("Test", "A test");
        let user = workspace.model_mut().add_person("User", "A user");
        let system = workspace.model_mut().add_software_system("System", "A system");
        workspace.model_mut().add_relationship(user, system, "Uses", None);

        let mermaid = MermaidExporter::export_flowchart(&workspace).unwrap();

        assert!(mermaid.contains("```mermaid"));
        assert!(mermaid.contains("flowchart TB"));
        assert!(mermaid.contains("-->"));
        assert!(mermaid.contains("```"));
    }
}
