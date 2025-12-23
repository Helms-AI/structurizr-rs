//! Workspace validation and inspections.
//!
//! This module provides validation and inspection capabilities for Structurizr workspaces,
//! detecting common issues such as empty descriptions, orphan elements, and missing technologies.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use structurizr_core::{ElementId, Workspace};

/// Severity level of a validation issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Critical error that should be fixed immediately.
    Error,
    /// Warning that should be addressed but doesn't prevent usage.
    Warning,
    /// Informational suggestion for improvement.
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Error => write!(f, "ERROR"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// A validation issue found in the workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Severity of the issue.
    pub severity: Severity,
    /// Human-readable message describing the issue.
    pub message: String,
    /// ID of the element related to this issue (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    /// Name of the element related to this issue (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub element_name: Option<String>,
    /// Type of check that generated this issue.
    pub check_type: String,
}

impl ValidationIssue {
    /// Create a new validation issue.
    pub fn new(
        severity: Severity,
        check_type: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            message: message.into(),
            element_id: None,
            element_name: None,
            check_type: check_type.into(),
        }
    }

    /// Add element information to the issue.
    pub fn with_element(mut self, id: ElementId, name: impl Into<String>) -> Self {
        self.element_id = Some(id.to_string());
        self.element_name = Some(name.into());
        self
    }
}

/// Result of validation containing all issues found.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// All issues found during validation.
    pub issues: Vec<ValidationIssue>,
    /// Number of errors.
    pub error_count: usize,
    /// Number of warnings.
    pub warning_count: usize,
    /// Number of info messages.
    pub info_count: usize,
}

impl ValidationResult {
    /// Create a new empty validation result.
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            error_count: 0,
            warning_count: 0,
            info_count: 0,
        }
    }

    /// Add an issue to the result.
    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            Severity::Info => self.info_count += 1,
        }
        self.issues.push(issue);
    }

    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.error_count == 0
    }

    /// Check if there are any issues at all.
    pub fn has_issues(&self) -> bool {
        !self.issues.is_empty()
    }

    /// Get all issues of a specific severity.
    pub fn issues_by_severity(&self, severity: Severity) -> Vec<&ValidationIssue> {
        self.issues
            .iter()
            .filter(|i| i.severity == severity)
            .collect()
    }
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration for validation checks.
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Check for empty descriptions.
    pub check_empty_descriptions: bool,
    /// Check for orphan elements (no relationships).
    pub check_orphan_elements: bool,
    /// Check for missing technology on containers/components.
    pub check_missing_technology: bool,
    /// Check for duplicate names at the same level.
    pub check_duplicate_names: bool,
    /// Check for circular relationships (self-references).
    pub check_circular_relationships: bool,
    /// Check for elements not included in any view.
    pub check_unused_elements: bool,
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            check_empty_descriptions: true,
            check_orphan_elements: true,
            check_missing_technology: true,
            check_duplicate_names: true,
            check_circular_relationships: true,
            check_unused_elements: true,
        }
    }
}

impl ValidationConfig {
    /// Create a new configuration with all checks enabled.
    pub fn all() -> Self {
        Self::default()
    }

    /// Create a new configuration with all checks disabled.
    pub fn none() -> Self {
        Self {
            check_empty_descriptions: false,
            check_orphan_elements: false,
            check_missing_technology: false,
            check_duplicate_names: false,
            check_circular_relationships: false,
            check_unused_elements: false,
        }
    }
}

/// Validate a workspace with default configuration.
pub fn validate_workspace(workspace: &Workspace) -> ValidationResult {
    validate_workspace_with_config(workspace, &ValidationConfig::default())
}

/// Validate a workspace with custom configuration.
pub fn validate_workspace_with_config(
    workspace: &Workspace,
    config: &ValidationConfig,
) -> ValidationResult {
    let mut result = ValidationResult::new();

    if config.check_empty_descriptions {
        check_empty_descriptions(workspace, &mut result);
    }

    if config.check_orphan_elements {
        check_orphan_elements(workspace, &mut result);
    }

    if config.check_missing_technology {
        check_missing_technology(workspace, &mut result);
    }

    if config.check_duplicate_names {
        check_duplicate_names(workspace, &mut result);
    }

    if config.check_circular_relationships {
        check_circular_relationships(workspace, &mut result);
    }

    if config.check_unused_elements {
        check_unused_elements(workspace, &mut result);
    }

    result
}

/// Check for elements with empty or missing descriptions.
fn check_empty_descriptions(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();

    // Check people
    for person in &model.people {
        if person.properties.description.is_none()
            || person
                .properties
                .description
                .as_ref()
                .map_or(false, |d| d.trim().is_empty())
        {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Warning,
                    "empty_description",
                    format!("Person '{}' has no description", person.name()),
                )
                .with_element(person.id(), person.name()),
            );
        }
    }

    // Check software systems and their containers/components
    for system in &model.software_systems {
        if system.properties.description.is_none()
            || system
                .properties
                .description
                .as_ref()
                .map_or(false, |d| d.trim().is_empty())
        {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Warning,
                    "empty_description",
                    format!("Software System '{}' has no description", system.name()),
                )
                .with_element(system.id(), system.name()),
            );
        }

        for container in &system.containers {
            if container.properties.description.is_none()
                || container
                    .properties
                    .description
                    .as_ref()
                    .map_or(false, |d| d.trim().is_empty())
            {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Warning,
                        "empty_description",
                        format!(
                            "Container '{}' in system '{}' has no description",
                            container.name(),
                            system.name()
                        ),
                    )
                    .with_element(container.id(), container.name()),
                );
            }

            for component in &container.components {
                if component.properties.description.is_none()
                    || component
                        .properties
                        .description
                        .as_ref()
                        .map_or(false, |d| d.trim().is_empty())
                {
                    result.add_issue(
                        ValidationIssue::new(
                            Severity::Warning,
                            "empty_description",
                            format!(
                                "Component '{}' in container '{}' has no description",
                                component.name(),
                                container.name()
                            ),
                        )
                        .with_element(component.id(), component.name()),
                    );
                }
            }
        }
    }

    // Check deployment nodes
    fn check_deployment_node(
        node: &structurizr_core::DeploymentNode,
        result: &mut ValidationResult,
    ) {
        if node.properties.description.is_none()
            || node
                .properties
                .description
                .as_ref()
                .map_or(false, |d| d.trim().is_empty())
        {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Info,
                    "empty_description",
                    format!("Deployment Node '{}' has no description", node.name()),
                )
                .with_element(node.id(), node.name()),
            );
        }

        for child in &node.children {
            check_deployment_node(child, result);
        }
    }

    for node in &model.deployment_nodes {
        check_deployment_node(node, result);
    }
}

/// Check for orphan elements that have no relationships.
fn check_orphan_elements(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();

    // Build a set of all element IDs that appear in relationships
    let mut connected_elements = HashSet::new();
    for rel in &model.relationships {
        connected_elements.insert(rel.source_id);
        connected_elements.insert(rel.destination_id);
    }

    // Check people
    for person in &model.people {
        if !connected_elements.contains(&person.id()) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Info,
                    "orphan_element",
                    format!(
                        "Person '{}' has no relationships (neither incoming nor outgoing)",
                        person.name()
                    ),
                )
                .with_element(person.id(), person.name()),
            );
        }
    }

    // Check software systems
    for system in &model.software_systems {
        if !connected_elements.contains(&system.id()) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Info,
                    "orphan_element",
                    format!(
                        "Software System '{}' has no relationships (neither incoming nor outgoing)",
                        system.name()
                    ),
                )
                .with_element(system.id(), system.name()),
            );
        }

        // Check containers
        for container in &system.containers {
            if !connected_elements.contains(&container.id()) {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Info,
                        "orphan_element",
                        format!(
                            "Container '{}' in system '{}' has no relationships",
                            container.name(),
                            system.name()
                        ),
                    )
                    .with_element(container.id(), container.name()),
                );
            }

            // Check components
            for component in &container.components {
                if !connected_elements.contains(&component.id()) {
                    result.add_issue(
                        ValidationIssue::new(
                            Severity::Info,
                            "orphan_element",
                            format!(
                                "Component '{}' in container '{}' has no relationships",
                                component.name(),
                                container.name()
                            ),
                        )
                        .with_element(component.id(), component.name()),
                    );
                }
            }
        }
    }
}

/// Check for containers and components without technology specified.
fn check_missing_technology(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();

    for system in &model.software_systems {
        for container in &system.containers {
            if container.technology.is_none()
                || container.technology.as_ref().map_or(false, |t| t.trim().is_empty())
            {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Warning,
                        "missing_technology",
                        format!(
                            "Container '{}' in system '{}' has no technology specified",
                            container.name(),
                            system.name()
                        ),
                    )
                    .with_element(container.id(), container.name()),
                );
            }

            for component in &container.components {
                if component.technology.is_none()
                    || component.technology.as_ref().map_or(false, |t| t.trim().is_empty())
                {
                    result.add_issue(
                        ValidationIssue::new(
                            Severity::Info,
                            "missing_technology",
                            format!(
                                "Component '{}' in container '{}' has no technology specified",
                                component.name(),
                                container.name()
                            ),
                        )
                        .with_element(component.id(), component.name()),
                    );
                }
            }
        }
    }
}

/// Check for duplicate names at the same level.
fn check_duplicate_names(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();

    // Check people
    let mut person_names = HashMap::new();
    for person in &model.people {
        let name = person.name().to_lowercase();
        if let Some(first_id) = person_names.get(&name) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Error,
                    "duplicate_name",
                    format!(
                        "Duplicate person name '{}' (conflicts with element ID {})",
                        person.name(),
                        first_id
                    ),
                )
                .with_element(person.id(), person.name()),
            );
        } else {
            person_names.insert(name, person.id());
        }
    }

    // Check software systems
    let mut system_names = HashMap::new();
    for system in &model.software_systems {
        let name = system.name().to_lowercase();
        if let Some(first_id) = system_names.get(&name) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Error,
                    "duplicate_name",
                    format!(
                        "Duplicate software system name '{}' (conflicts with element ID {})",
                        system.name(),
                        first_id
                    ),
                )
                .with_element(system.id(), system.name()),
            );
        } else {
            system_names.insert(name, system.id());
        }

        // Check containers within each system
        let mut container_names = HashMap::new();
        for container in &system.containers {
            let name = container.name().to_lowercase();
            if let Some(first_id) = container_names.get(&name) {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Error,
                        "duplicate_name",
                        format!(
                            "Duplicate container name '{}' in system '{}' (conflicts with element ID {})",
                            container.name(),
                            system.name(),
                            first_id
                        ),
                    )
                    .with_element(container.id(), container.name()),
                );
            } else {
                container_names.insert(name, container.id());
            }

            // Check components within each container
            let mut component_names = HashMap::new();
            for component in &container.components {
                let name = component.name().to_lowercase();
                if let Some(first_id) = component_names.get(&name) {
                    result.add_issue(
                        ValidationIssue::new(
                            Severity::Error,
                            "duplicate_name",
                            format!(
                                "Duplicate component name '{}' in container '{}' (conflicts with element ID {})",
                                component.name(),
                                container.name(),
                                first_id
                            ),
                        )
                        .with_element(component.id(), component.name()),
                    );
                } else {
                    component_names.insert(name, component.id());
                }
            }
        }
    }
}

/// Check for circular relationships (element relating to itself).
fn check_circular_relationships(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();

    for rel in &model.relationships {
        if rel.source_id == rel.destination_id {
            // Find the element to get its name
            if let Some(element) = model.find_element(rel.source_id) {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Warning,
                        "circular_relationship",
                        format!(
                            "Circular relationship: '{}' has a relationship to itself",
                            element.name()
                        ),
                    )
                    .with_element(rel.source_id, element.name()),
                );
            } else {
                result.add_issue(ValidationIssue::new(
                    Severity::Warning,
                    "circular_relationship",
                    format!(
                        "Circular relationship: element {} has a relationship to itself",
                        rel.source_id
                    ),
                ));
            }
        }
    }
}

/// Check for elements not included in any view.
fn check_unused_elements(workspace: &Workspace, result: &mut ValidationResult) {
    let model = workspace.model();
    let views = workspace.views();

    // Collect all element IDs that appear in any view
    let mut used_elements = HashSet::new();

    // System landscape views include all elements by default
    for view in &views.system_landscape_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
    }

    // System context views
    for view in &views.system_context_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
        // Also include the system itself
        used_elements.insert(view.software_system_id);
    }

    // Container views
    for view in &views.container_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
        // Also include the system itself
        used_elements.insert(view.software_system_id);
    }

    // Component views
    for view in &views.component_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
        // Also include the container itself
        used_elements.insert(view.container_id);
    }

    // Dynamic views
    for view in &views.dynamic_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
    }

    // Deployment views
    for view in &views.deployment_views {
        for element in &view.properties.elements {
            used_elements.insert(element.id);
        }
    }

    // Now check which elements are not used
    // Check people
    for person in &model.people {
        if !used_elements.contains(&person.id()) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Info,
                    "unused_element",
                    format!("Person '{}' is not included in any view", person.name()),
                )
                .with_element(person.id(), person.name()),
            );
        }
    }

    // Check software systems
    for system in &model.software_systems {
        if !used_elements.contains(&system.id()) {
            result.add_issue(
                ValidationIssue::new(
                    Severity::Info,
                    "unused_element",
                    format!(
                        "Software System '{}' is not included in any view",
                        system.name()
                    ),
                )
                .with_element(system.id(), system.name()),
            );
        }

        // Check containers
        for container in &system.containers {
            if !used_elements.contains(&container.id()) {
                result.add_issue(
                    ValidationIssue::new(
                        Severity::Info,
                        "unused_element",
                        format!(
                            "Container '{}' in system '{}' is not included in any view",
                            container.name(),
                            system.name()
                        ),
                    )
                    .with_element(container.id(), container.name()),
                );
            }

            // Check components
            for component in &container.components {
                if !used_elements.contains(&component.id()) {
                    result.add_issue(
                        ValidationIssue::new(
                            Severity::Info,
                            "unused_element",
                            format!(
                                "Component '{}' in container '{}' is not included in any view",
                                component.name(),
                                container.name()
                            ),
                        )
                        .with_element(component.id(), component.name()),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::{Container, Person, SoftwareSystem, Workspace};

    #[test]
    fn test_empty_workspace_has_no_issues() {
        let workspace = Workspace::new("Test", "Test workspace");
        let result = validate_workspace(&workspace);
        assert_eq!(result.issues.len(), 0);
    }

    #[test]
    fn test_detect_empty_descriptions() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let person = Person::new("User");
        workspace.model_mut().people.push(person);

        let result = validate_workspace(&workspace);
        assert!(result.warning_count > 0);
        assert!(result
            .issues
            .iter()
            .any(|i| i.check_type == "empty_description"));
    }

    #[test]
    fn test_detect_orphan_elements() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let person = Person::new("User").with_description("A user");
        workspace.model_mut().people.push(person);

        let result = validate_workspace(&workspace);
        assert!(result
            .issues
            .iter()
            .any(|i| i.check_type == "orphan_element"));
    }

    #[test]
    fn test_detect_missing_technology() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let mut system = SoftwareSystem::new("System").with_description("A system");
        let container = Container::new("Container").with_description("A container");
        system.containers.push(container);
        workspace.model_mut().software_systems.push(system);

        let result = validate_workspace(&workspace);
        assert!(result
            .issues
            .iter()
            .any(|i| i.check_type == "missing_technology"));
    }

    #[test]
    fn test_detect_duplicate_names() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        workspace
            .model_mut()
            .people
            .push(Person::new("User").with_description("First"));
        workspace
            .model_mut()
            .people
            .push(Person::new("User").with_description("Second"));

        let result = validate_workspace(&workspace);
        assert!(result.error_count > 0);
        assert!(result
            .issues
            .iter()
            .any(|i| i.check_type == "duplicate_name"));
    }

    #[test]
    fn test_detect_circular_relationships() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let person_id = workspace
            .model_mut()
            .add_person("User", "A user of the system");

        // Create a circular relationship
        workspace
            .model_mut()
            .add_relationship(person_id, person_id, "talks to", None);

        let result = validate_workspace(&workspace);
        assert!(result.warning_count > 0);
        assert!(result
            .issues
            .iter()
            .any(|i| i.check_type == "circular_relationship"));
    }

    #[test]
    fn test_validation_config() {
        let mut workspace = Workspace::new("Test", "Test workspace");
        let person = Person::new("User"); // No description
        workspace.model_mut().people.push(person);

        // Validate with empty descriptions check disabled
        let config = ValidationConfig {
            check_empty_descriptions: false,
            ..ValidationConfig::default()
        };
        let result = validate_workspace_with_config(&workspace, &config);
        assert_eq!(result.warning_count, 0);

        // Validate with empty descriptions check enabled
        let config = ValidationConfig {
            check_empty_descriptions: true,
            ..ValidationConfig::default()
        };
        let result = validate_workspace_with_config(&workspace, &config);
        assert!(result.warning_count > 0);
    }
}
