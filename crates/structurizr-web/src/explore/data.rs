//! Graph data extraction from workspace model.
//!
//! Converts C4 model elements and relationships into a graph structure
//! suitable for force-directed visualization.

use serde::{Deserialize, Serialize};
use structurizr_core::Workspace;

/// Represents a node in the explore graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    /// Element ID
    pub id: String,
    /// Element name
    pub name: String,
    /// Element type (Person, Software System, Container, Component)
    #[serde(rename = "type")]
    pub node_type: String,
    /// Optional element description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Optional technology (for containers and components)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    /// Parent element ID (for hierarchy - containers have system parent, components have container parent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

/// Represents a link/relationship in the explore graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    /// Source element ID
    pub source: String,
    /// Target element ID
    pub target: String,
    /// Optional relationship label/description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// Extracts graph data from a workspace model.
///
/// Returns a tuple of (nodes, links) representing all elements and relationships
/// in the workspace suitable for force-directed graph visualization.
pub fn extract_graph_data(workspace: &Workspace) -> (Vec<GraphNode>, Vec<GraphLink>) {
    let model = workspace.model();
    let mut nodes = Vec::new();
    let mut links = Vec::new();

    // Extract people as nodes (no parent)
    for person in &model.people {
        nodes.push(GraphNode {
            id: person.id().to_string(),
            name: person.name().to_string(),
            node_type: "Person".to_string(),
            description: person.properties.description.clone(),
            technology: None,
            parent_id: None,
        });
    }

    // Extract software systems and their containers/components
    for system in &model.software_systems {
        let system_id = system.id().to_string();

        // Add system node (no parent)
        nodes.push(GraphNode {
            id: system_id.clone(),
            name: system.name().to_string(),
            node_type: "Software System".to_string(),
            description: system.properties.description.clone(),
            technology: None,
            parent_id: None,
        });

        // Add container nodes (parent is the system)
        for container in &system.containers {
            let container_id = container.id().to_string();

            nodes.push(GraphNode {
                id: container_id.clone(),
                name: container.name().to_string(),
                node_type: "Container".to_string(),
                description: container.properties.description.clone(),
                technology: container.technology.clone(),
                parent_id: Some(system_id.clone()),
            });

            // Add component nodes (parent is the container)
            for component in &container.components {
                nodes.push(GraphNode {
                    id: component.id().to_string(),
                    name: component.name().to_string(),
                    node_type: "Component".to_string(),
                    description: component.properties.description.clone(),
                    technology: component.technology.clone(),
                    parent_id: Some(container_id.clone()),
                });
            }
        }
    }

    // Extract relationships as links
    for relationship in &model.relationships {
        links.push(GraphLink {
            source: relationship.source_id.to_string(),
            target: relationship.destination_id.to_string(),
            label: relationship.description.clone(),
        });
    }

    (nodes, links)
}

/// Helper function to escape JSON strings for safe embedding in HTML.
#[allow(dead_code)]
pub fn escape_json(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '"' => r#"\""#.to_string(),
            '\\' => r"\\".to_string(),
            '\n' => r"\n".to_string(),
            '\r' => r"\r".to_string(),
            '\t' => r"\t".to_string(),
            c if c.is_control() => format!(r"\u{:04x}", c as u32),
            c => c.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::{Model, Person, SoftwareSystem, Relationship};

    #[test]
    fn test_extract_graph_data_empty_workspace() {
        let workspace = Workspace::new("Test", "Test workspace");
        let (nodes, links) = extract_graph_data(&workspace);
        assert_eq!(nodes.len(), 0);
        assert_eq!(links.len(), 0);
    }

    #[test]
    fn test_extract_graph_data_with_elements() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let mut model = workspace.model_mut();

        // Add a person and system
        let person = model.add_person("User", "A user");
        let system = model.add_software_system("System", "A system");

        // Add a relationship
        model.add_relationship(person, system, "Uses", None);

        let (nodes, links) = extract_graph_data(&workspace);

        assert_eq!(nodes.len(), 2);
        assert_eq!(links.len(), 1);

        // Check node types
        assert!(nodes.iter().any(|n| n.node_type == "Person"));
        assert!(nodes.iter().any(|n| n.node_type == "Software System"));

        // Check link
        assert_eq!(links[0].label, Some("Uses".to_string()));
    }

    #[test]
    fn test_escape_json() {
        assert_eq!(escape_json("simple"), "simple");
        assert_eq!(escape_json(r#"has "quotes""#), r#"has \"quotes\""#);
        assert_eq!(escape_json("line\nbreak"), r"line\nbreak");
        assert_eq!(escape_json("tab\there"), r"tab\there");
    }
}