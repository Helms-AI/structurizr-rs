//! C4 Model elements and relationships.
//!
//! This module contains the core abstractions of the C4 model:
//! - Person
//! - SoftwareSystem
//! - Container
//! - Component
//! - DeploymentNode
//! - Relationships between elements

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for model elements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ElementId(Uuid);

impl ElementId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create an ElementId from a name using deterministic UUID v5.
    pub fn from_name(name: &str) -> Self {
        let namespace = Uuid::NAMESPACE_OID;
        Self(Uuid::new_v5(&namespace, name.as_bytes()))
    }
}

impl Default for ElementId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for ElementId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<Uuid> for ElementId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

/// Location of an element (internal or external to the system).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Location {
    #[default]
    Internal,
    External,
}

/// Type of interaction for relationships.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InteractionStyle {
    #[default]
    Synchronous,
    Asynchronous,
}

/// Base properties shared by all model elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementProperties {
    pub id: ElementId,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub perspectives: Vec<String>,
}

impl ElementProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: ElementId::new(),
            name: name.into(),
            description: None,
            tags: Vec::new(),
            properties: HashMap::new(),
            url: None,
            perspectives: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tags.extend(tags.into_iter().map(Into::into));
        self
    }

    pub fn with_perspective(mut self, perspective: impl Into<String>) -> Self {
        self.perspectives.push(perspective.into());
        self
    }

    pub fn with_perspectives(mut self, perspectives: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.perspectives.extend(perspectives.into_iter().map(Into::into));
        self
    }
}

/// A person represents a human user of your software system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Person {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default)]
    pub location: Location,
}

impl Person {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Person".to_string());
        Self {
            properties: props,
            location: Location::default(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.properties.description = Some(description.into());
        self
    }

    pub fn external(mut self) -> Self {
        self.location = Location::External;
        self
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// A software system is the highest level of abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareSystem {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default)]
    pub location: Location,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containers: Vec<Container>,
}

impl SoftwareSystem {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Software System".to_string());
        Self {
            properties: props,
            location: Location::default(),
            containers: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.properties.description = Some(description.into());
        self
    }

    pub fn external(mut self) -> Self {
        self.location = Location::External;
        self
    }

    pub fn add_container(&mut self, container: Container) -> ElementId {
        let id = container.id();
        self.containers.push(container);
        id
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// A container represents a deployable unit (application, database, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Container {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<Component>,
}

impl Container {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Container".to_string());
        Self {
            properties: props,
            technology: None,
            components: Vec::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.properties.description = Some(description.into());
        self
    }

    pub fn with_technology(mut self, technology: impl Into<String>) -> Self {
        self.technology = Some(technology.into());
        self
    }

    pub fn add_component(&mut self, component: Component) -> ElementId {
        let id = component.id();
        self.components.push(component);
        id
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// A component is a grouping of related functionality within a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

impl Component {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Component".to_string());
        Self {
            properties: props,
            technology: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.properties.description = Some(description.into());
        self
    }

    pub fn with_technology(mut self, technology: impl Into<String>) -> Self {
        self.technology = Some(technology.into());
        self
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// A deployment node represents infrastructure (servers, VMs, containers, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentNode {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Number of instances. Can be a number or range like "2-10".
    #[serde(default = "default_instances")]
    pub instances: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<DeploymentNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub infrastructure_nodes: Vec<InfrastructureNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub container_instances: Vec<ContainerInstance>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub software_system_instances: Vec<SoftwareSystemInstance>,
}

fn default_instances() -> String {
    "1".to_string()
}

impl DeploymentNode {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Deployment Node".to_string());
        Self {
            properties: props,
            technology: None,
            environment: None,
            instances: "1".to_string(),
            children: Vec::new(),
            infrastructure_nodes: Vec::new(),
            container_instances: Vec::new(),
            software_system_instances: Vec::new(),
        }
    }

    pub fn with_instances(mut self, instances: impl Into<String>) -> Self {
        self.instances = instances.into();
        self
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// An infrastructure node represents infrastructure elements (DNS, load balancers, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfrastructureNode {
    #[serde(flatten)]
    pub properties: ElementProperties,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
}

impl InfrastructureNode {
    pub fn new(name: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        props.tags.push("Element".to_string());
        props.tags.push("Infrastructure Node".to_string());
        Self {
            properties: props,
            technology: None,
        }
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }
}

/// A health check for a container or software system instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub name: String,
    pub url: String,
    /// Polling interval in seconds (default: 60)
    #[serde(default = "default_health_check_interval")]
    pub interval: u32,
    /// Timeout in milliseconds (default: 0 means no timeout)
    #[serde(default)]
    pub timeout: u32,
    /// Optional HTTP headers for the health check request
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,
}

fn default_health_check_interval() -> u32 {
    60
}

impl HealthCheck {
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            interval: 60,
            timeout: 0,
            headers: HashMap::new(),
        }
    }

    pub fn with_interval(mut self, interval: u32) -> Self {
        self.interval = interval;
        self
    }

    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// An instance of a container deployed to a deployment node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerInstance {
    pub id: ElementId,
    pub container_id: ElementId,
    #[serde(default)]
    pub instance_id: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub health_checks: Vec<HealthCheck>,
}

/// An instance of a software system deployed to a deployment node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareSystemInstance {
    pub id: ElementId,
    pub software_system_id: ElementId,
    #[serde(default)]
    pub instance_id: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub health_checks: Vec<HealthCheck>,
}

/// A relationship between two model elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    pub id: ElementId,
    pub source_id: ElementId,
    pub destination_id: ElementId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub technology: Option<String>,
    #[serde(default)]
    pub interaction_style: InteractionStyle,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub properties: HashMap<String, String>,
    /// Perspectives provide different stakeholder viewpoints on this relationship.
    /// Keys are perspective names (e.g., "Security", "Performance"), values are descriptions.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub perspectives: HashMap<String, String>,
}

impl Relationship {
    pub fn new(source_id: ElementId, destination_id: ElementId) -> Self {
        Self {
            id: ElementId::new(),
            source_id,
            destination_id,
            description: None,
            technology: None,
            interaction_style: InteractionStyle::default(),
            tags: vec!["Relationship".to_string()],
            properties: HashMap::new(),
            perspectives: HashMap::new(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_technology(mut self, technology: impl Into<String>) -> Self {
        self.technology = Some(technology.into());
        self
    }

    pub fn asynchronous(mut self) -> Self {
        self.interaction_style = InteractionStyle::Asynchronous;
        self
    }
}

/// A custom element type for modeling non-standard architectural elements.
/// This allows for flexible modeling beyond the standard C4 model types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomElement {
    #[serde(flatten)]
    pub properties: ElementProperties,
    /// The metadata type of this custom element (e.g., "Database", "Queue", etc.)
    pub metadata: String,
    #[serde(default)]
    pub location: Location,
}

impl CustomElement {
    pub fn new(name: impl Into<String>, metadata: impl Into<String>) -> Self {
        let mut props = ElementProperties::new(name);
        let metadata_str = metadata.into();
        props.tags.push("Element".to_string());
        props.tags.push(format!("Custom:{}", metadata_str));
        Self {
            properties: props,
            metadata: metadata_str,
            location: Location::default(),
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.properties.description = Some(description.into());
        self
    }

    pub fn external(mut self) -> Self {
        self.location = Location::External;
        self
    }

    pub fn id(&self) -> ElementId {
        self.properties.id
    }

    pub fn name(&self) -> &str {
        &self.properties.name
    }
}

/// A group for organizing elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub element_ids: Vec<ElementId>,
}

impl Group {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            element_ids: Vec::new(),
        }
    }
}

/// The model contains all elements and relationships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Model {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub people: Vec<Person>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub software_systems: Vec<SoftwareSystem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub custom_elements: Vec<CustomElement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deployment_nodes: Vec<DeploymentNode>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<Relationship>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<Group>,
}

impl Model {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_person(&mut self, name: impl Into<String>, description: impl Into<String>) -> ElementId {
        let person = Person::new(name).with_description(description);
        let id = person.id();
        self.people.push(person);
        id
    }

    pub fn add_software_system(&mut self, name: impl Into<String>, description: impl Into<String>) -> ElementId {
        let system = SoftwareSystem::new(name).with_description(description);
        let id = system.id();
        self.software_systems.push(system);
        id
    }

    pub fn add_custom_element(&mut self, name: impl Into<String>, metadata: impl Into<String>, description: impl Into<String>) -> ElementId {
        let element = CustomElement::new(name, metadata).with_description(description);
        let id = element.id();
        self.custom_elements.push(element);
        id
    }

    pub fn add_relationship(
        &mut self,
        source_id: ElementId,
        destination_id: ElementId,
        description: impl Into<String>,
        technology: Option<String>,
    ) -> ElementId {
        let mut rel = Relationship::new(source_id, destination_id)
            .with_description(description);
        if let Some(tech) = technology {
            rel = rel.with_technology(tech);
        }
        let id = rel.id;
        self.relationships.push(rel);
        id
    }

    /// Add a relationship with full metadata including tags, properties, and perspectives.
    pub fn add_relationship_with_metadata(
        &mut self,
        source_id: ElementId,
        destination_id: ElementId,
        description: impl Into<String>,
        technology: Option<String>,
        tags: Vec<String>,
        properties: HashMap<String, String>,
        perspectives: HashMap<String, String>,
    ) -> ElementId {
        let mut rel = Relationship::new(source_id, destination_id)
            .with_description(description);
        if let Some(tech) = technology {
            rel = rel.with_technology(tech);
        }
        // Add tags (append to default "Relationship" tag)
        rel.tags.extend(tags);
        rel.properties = properties;
        rel.perspectives = perspectives;
        let id = rel.id;
        self.relationships.push(rel);
        id
    }

    /// Find an element by its ID across all element types.
    pub fn find_element(&self, id: ElementId) -> Option<ElementRef<'_>> {
        // Check people
        for person in &self.people {
            if person.id() == id {
                return Some(ElementRef::Person(person));
            }
        }

        // Check software systems and their containers/components
        for system in &self.software_systems {
            if system.id() == id {
                return Some(ElementRef::SoftwareSystem(system));
            }
            for container in &system.containers {
                if container.id() == id {
                    return Some(ElementRef::Container(container));
                }
                for component in &container.components {
                    if component.id() == id {
                        return Some(ElementRef::Component(component));
                    }
                }
            }
        }

        // Check deployment nodes
        fn find_in_deployment_nodes(nodes: &[DeploymentNode], id: ElementId) -> Option<ElementRef<'_>> {
            for node in nodes {
                if node.id() == id {
                    return Some(ElementRef::DeploymentNode(node));
                }
                if let Some(found) = find_in_deployment_nodes(&node.children, id) {
                    return Some(found);
                }
            }
            None
        }

        find_in_deployment_nodes(&self.deployment_nodes, id)
    }
}

/// A reference to any model element.
#[derive(Debug, Clone, Copy)]
pub enum ElementRef<'a> {
    Person(&'a Person),
    SoftwareSystem(&'a SoftwareSystem),
    Container(&'a Container),
    Component(&'a Component),
    DeploymentNode(&'a DeploymentNode),
}

impl<'a> ElementRef<'a> {
    pub fn id(&self) -> ElementId {
        match self {
            ElementRef::Person(p) => p.id(),
            ElementRef::SoftwareSystem(s) => s.id(),
            ElementRef::Container(c) => c.id(),
            ElementRef::Component(c) => c.id(),
            ElementRef::DeploymentNode(d) => d.id(),
        }
    }

    pub fn name(&self) -> &str {
        match self {
            ElementRef::Person(p) => p.name(),
            ElementRef::SoftwareSystem(s) => s.name(),
            ElementRef::Container(c) => c.name(),
            ElementRef::Component(c) => c.name(),
            ElementRef::DeploymentNode(d) => d.name(),
        }
    }
}
