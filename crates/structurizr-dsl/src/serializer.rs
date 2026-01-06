//! DSL Serializer for structurizr-rs
//!
//! This module provides functionality to serialize a Workspace back to
//! Structurizr DSL format. The output is compatible with the official
//! Structurizr Java tooling.

use std::fmt::Write;

use structurizr_core::model::{Component, Container, Person, Relationship, SoftwareSystem};
use structurizr_core::style::{ElementStyle, LineStyle, RelationshipStyle, Shape};
use structurizr_core::view::{
    ComponentView, ContainerView, DeploymentView, DynamicView, SystemContextView,
    SystemLandscapeView,
};
use structurizr_core::Workspace;

/// Options for DSL serialization
#[derive(Debug, Clone, Default)]
pub struct SerializerOptions {
    /// Include comments in output
    pub include_comments: bool,
    /// Indentation string (default: 4 spaces)
    pub indent: String,
}

impl SerializerOptions {
    pub fn new() -> Self {
        Self {
            include_comments: true,
            indent: "    ".to_string(),
        }
    }
}

/// Serializes a Workspace to Structurizr DSL format
pub struct DslSerializer {
    options: SerializerOptions,
}

impl DslSerializer {
    pub fn new() -> Self {
        Self {
            options: SerializerOptions::new(),
        }
    }

    pub fn with_options(options: SerializerOptions) -> Self {
        Self { options }
    }

    /// Serialize a workspace to DSL string
    pub fn serialize(&self, workspace: &Workspace) -> String {
        let mut output = String::new();

        // Workspace declaration
        self.write_workspace_header(&mut output, workspace);

        // Model block
        self.write_model(&mut output, workspace);

        // Views block
        self.write_views(&mut output, workspace);

        // Close workspace
        output.push_str("}\n");

        output
    }

    fn write_workspace_header(&self, output: &mut String, workspace: &Workspace) {
        let name = escape_string(&workspace.name);
        let desc = workspace.description.as_deref().unwrap_or("");

        if desc.is_empty() {
            writeln!(output, "workspace \"{}\" {{", name).unwrap();
        } else {
            writeln!(
                output,
                "workspace \"{}\" \"{}\" {{",
                name,
                escape_string(desc)
            )
            .unwrap();
        }
        output.push('\n');
    }

    fn write_model(&self, output: &mut String, workspace: &Workspace) {
        let indent = &self.options.indent;

        writeln!(output, "{}model {{", indent).unwrap();

        // Write people
        for person in &workspace.model.people {
            self.write_person(output, person, 2);
        }

        if !workspace.model.people.is_empty() && !workspace.model.software_systems.is_empty() {
            output.push('\n');
        }

        // Write software systems
        for system in &workspace.model.software_systems {
            self.write_software_system(output, system, 2);
        }

        // Write relationships
        if !workspace.model.relationships.is_empty() {
            output.push('\n');
            for rel in &workspace.model.relationships {
                self.write_relationship(output, rel, workspace, 2);
            }
        }

        writeln!(output, "{}}}", indent).unwrap();
        output.push('\n');
    }

    fn write_person(&self, output: &mut String, person: &Person, depth: usize) {
        let indent = self.indent(depth);
        let id = self.generate_identifier(person.name());
        let name = escape_string(person.name());
        let desc = person.properties.description.as_deref().unwrap_or("");
        let tags = &person.properties.tags;

        // Check if we need a block (has tags beyond default)
        let custom_tags: Vec<_> = tags
            .iter()
            .filter(|t| *t != "Element" && *t != "Person")
            .collect();

        if custom_tags.is_empty() {
            if desc.is_empty() {
                writeln!(output, "{}{} = person \"{}\"", indent, id, name).unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} = person \"{}\" \"{}\"",
                    indent,
                    id,
                    name,
                    escape_string(desc)
                )
                .unwrap();
            }
        } else {
            if desc.is_empty() {
                writeln!(output, "{}{} = person \"{}\" {{", indent, id, name).unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} = person \"{}\" \"{}\" {{",
                    indent,
                    id,
                    name,
                    escape_string(desc)
                )
                .unwrap();
            }
            self.write_tags(output, &custom_tags, depth + 1);
            writeln!(output, "{}}}", indent).unwrap();
        }
    }

    fn write_software_system(
        &self,
        output: &mut String,
        system: &SoftwareSystem,
        depth: usize,
    ) {
        let indent = self.indent(depth);
        let id = self.generate_identifier(system.name());
        let name = escape_string(system.name());
        let desc = system.properties.description.as_deref().unwrap_or("");
        let tags = &system.properties.tags;

        let custom_tags: Vec<_> = tags
            .iter()
            .filter(|t| *t != "Element" && *t != "Software System")
            .collect();

        let has_content = !system.containers.is_empty() || !custom_tags.is_empty();

        if has_content {
            if desc.is_empty() {
                writeln!(output, "{}{} = softwareSystem \"{}\" {{", indent, id, name).unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} = softwareSystem \"{}\" \"{}\" {{",
                    indent,
                    id,
                    name,
                    escape_string(desc)
                )
                .unwrap();
            }

            if !custom_tags.is_empty() {
                self.write_tags(output, &custom_tags, depth + 1);
            }

            for container in &system.containers {
                self.write_container(output, container, depth + 1);
            }

            writeln!(output, "{}}}", indent).unwrap();
        } else {
            if desc.is_empty() {
                writeln!(output, "{}{} = softwareSystem \"{}\"", indent, id, name).unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} = softwareSystem \"{}\" \"{}\"",
                    indent,
                    id,
                    name,
                    escape_string(desc)
                )
                .unwrap();
            }
        }
    }

    fn write_container(&self, output: &mut String, container: &Container, depth: usize) {
        let indent = self.indent(depth);
        let id = self.generate_identifier(container.name());
        let name = escape_string(container.name());
        let desc = container.properties.description.as_deref().unwrap_or("");
        let tech = container.technology.as_deref().unwrap_or("");
        let tags = &container.properties.tags;

        let custom_tags: Vec<_> = tags
            .iter()
            .filter(|t| *t != "Element" && *t != "Container")
            .collect();

        let has_content = !container.components.is_empty() || !custom_tags.is_empty();

        if has_content {
            if tech.is_empty() {
                if desc.is_empty() {
                    writeln!(output, "{}{} = container \"{}\" {{", indent, id, name).unwrap();
                } else {
                    writeln!(
                        output,
                        "{}{} = container \"{}\" \"{}\" {{",
                        indent,
                        id,
                        name,
                        escape_string(desc)
                    )
                    .unwrap();
                }
            } else {
                writeln!(
                    output,
                    "{}{} = container \"{}\" \"{}\" \"{}\" {{",
                    indent,
                    id,
                    name,
                    escape_string(desc),
                    escape_string(tech)
                )
                .unwrap();
            }

            if !custom_tags.is_empty() {
                self.write_tags(output, &custom_tags, depth + 1);
            }

            for component in &container.components {
                self.write_component(output, component, depth + 1);
            }

            writeln!(output, "{}}}", indent).unwrap();
        } else {
            if tech.is_empty() {
                if desc.is_empty() {
                    writeln!(output, "{}{} = container \"{}\"", indent, id, name).unwrap();
                } else {
                    writeln!(
                        output,
                        "{}{} = container \"{}\" \"{}\"",
                        indent,
                        id,
                        name,
                        escape_string(desc)
                    )
                    .unwrap();
                }
            } else {
                writeln!(
                    output,
                    "{}{} = container \"{}\" \"{}\" \"{}\"",
                    indent,
                    id,
                    name,
                    escape_string(desc),
                    escape_string(tech)
                )
                .unwrap();
            }
        }
    }

    fn write_component(&self, output: &mut String, component: &Component, depth: usize) {
        let indent = self.indent(depth);
        let id = self.generate_identifier(component.name());
        let name = escape_string(component.name());
        let desc = component.properties.description.as_deref().unwrap_or("");
        let tech = component.technology.as_deref().unwrap_or("");
        let tags = &component.properties.tags;

        let custom_tags: Vec<_> = tags
            .iter()
            .filter(|t| *t != "Element" && *t != "Component")
            .collect();

        if !custom_tags.is_empty() {
            if tech.is_empty() {
                if desc.is_empty() {
                    writeln!(output, "{}{} = component \"{}\" {{", indent, id, name).unwrap();
                } else {
                    writeln!(
                        output,
                        "{}{} = component \"{}\" \"{}\" {{",
                        indent,
                        id,
                        name,
                        escape_string(desc)
                    )
                    .unwrap();
                }
            } else {
                writeln!(
                    output,
                    "{}{} = component \"{}\" \"{}\" \"{}\" {{",
                    indent,
                    id,
                    name,
                    escape_string(desc),
                    escape_string(tech)
                )
                .unwrap();
            }
            self.write_tags(output, &custom_tags, depth + 1);
            writeln!(output, "{}}}", indent).unwrap();
        } else {
            if tech.is_empty() {
                if desc.is_empty() {
                    writeln!(output, "{}{} = component \"{}\"", indent, id, name).unwrap();
                } else {
                    writeln!(
                        output,
                        "{}{} = component \"{}\" \"{}\"",
                        indent,
                        id,
                        name,
                        escape_string(desc)
                    )
                    .unwrap();
                }
            } else {
                writeln!(
                    output,
                    "{}{} = component \"{}\" \"{}\" \"{}\"",
                    indent,
                    id,
                    name,
                    escape_string(desc),
                    escape_string(tech)
                )
                .unwrap();
            }
        }
    }

    fn write_relationship(
        &self,
        output: &mut String,
        rel: &Relationship,
        workspace: &Workspace,
        depth: usize,
    ) {
        let indent = self.indent(depth);

        // Find element names from IDs
        let source_name = self.find_element_name(rel.source_id, workspace);
        let dest_name = self.find_element_name(rel.destination_id, workspace);

        let source_id = self.generate_identifier(&source_name);
        let dest_id = self.generate_identifier(&dest_name);

        let desc = rel.description.as_deref().unwrap_or("");
        let tech = rel.technology.as_deref();

        if let Some(tech) = tech {
            if desc.is_empty() {
                writeln!(
                    output,
                    "{}{} -> {} \"\" \"{}\"",
                    indent,
                    source_id,
                    dest_id,
                    escape_string(tech)
                )
                .unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} -> {} \"{}\" \"{}\"",
                    indent,
                    source_id,
                    dest_id,
                    escape_string(desc),
                    escape_string(tech)
                )
                .unwrap();
            }
        } else {
            if desc.is_empty() {
                writeln!(output, "{}{} -> {}", indent, source_id, dest_id).unwrap();
            } else {
                writeln!(
                    output,
                    "{}{} -> {} \"{}\"",
                    indent,
                    source_id,
                    dest_id,
                    escape_string(desc)
                )
                .unwrap();
            }
        }
    }

    fn write_tags(&self, output: &mut String, tags: &[&String], depth: usize) {
        let indent = self.indent(depth);
        let tags_str = tags
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(output, "{}tags {}", indent, tags_str).unwrap();
    }

    fn write_views(&self, output: &mut String, workspace: &Workspace) {
        let indent = &self.options.indent;

        writeln!(output, "{}views {{", indent).unwrap();

        // System Landscape Views
        for view in &workspace.views.system_landscape_views {
            self.write_system_landscape_view(output, view, 2);
        }

        // System Context Views
        for view in &workspace.views.system_context_views {
            self.write_system_context_view(output, view, workspace, 2);
        }

        // Container Views
        for view in &workspace.views.container_views {
            self.write_container_view(output, view, workspace, 2);
        }

        // Component Views
        for view in &workspace.views.component_views {
            self.write_component_view(output, view, workspace, 2);
        }

        // Dynamic Views
        for view in &workspace.views.dynamic_views {
            self.write_dynamic_view(output, view, 2);
        }

        // Deployment Views
        for view in &workspace.views.deployment_views {
            self.write_deployment_view(output, view, 2);
        }

        // Styles (from workspace.configuration)
        if let Some(config) = &workspace.configuration {
            if !config.styles.elements.is_empty() || !config.styles.relationships.is_empty() {
                output.push('\n');
                self.write_styles(output, &config.styles.elements, &config.styles.relationships, 2);
            }
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_system_landscape_view(
        &self,
        output: &mut String,
        view: &SystemLandscapeView,
        depth: usize,
    ) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");

        if title.is_empty() {
            writeln!(output, "{}systemLandscape \"{}\" {{", indent, key).unwrap();
        } else {
            writeln!(
                output,
                "{}systemLandscape \"{}\" \"{}\" {{",
                indent,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        self.write_view_content(output, view.properties.auto_layout.is_some(), depth + 1);

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_system_context_view(
        &self,
        output: &mut String,
        view: &SystemContextView,
        workspace: &Workspace,
        depth: usize,
    ) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");

        // Find the system name from ID
        let system_name = self.find_element_name(view.software_system_id, workspace);
        let system_id = self.generate_identifier(&system_name);

        if title.is_empty() {
            writeln!(output, "{}systemContext {} \"{}\" {{", indent, system_id, key).unwrap();
        } else {
            writeln!(
                output,
                "{}systemContext {} \"{}\" \"{}\" {{",
                indent,
                system_id,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        self.write_view_content(output, view.properties.auto_layout.is_some(), depth + 1);

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_container_view(
        &self,
        output: &mut String,
        view: &ContainerView,
        workspace: &Workspace,
        depth: usize,
    ) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");

        let system_name = self.find_element_name(view.software_system_id, workspace);
        let system_id = self.generate_identifier(&system_name);

        if title.is_empty() {
            writeln!(output, "{}container {} \"{}\" {{", indent, system_id, key).unwrap();
        } else {
            writeln!(
                output,
                "{}container {} \"{}\" \"{}\" {{",
                indent,
                system_id,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        self.write_view_content(output, view.properties.auto_layout.is_some(), depth + 1);

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_component_view(
        &self,
        output: &mut String,
        view: &ComponentView,
        workspace: &Workspace,
        depth: usize,
    ) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");

        let container_name = self.find_element_name(view.container_id, workspace);
        let container_id = self.generate_identifier(&container_name);

        if title.is_empty() {
            writeln!(output, "{}component {} \"{}\" {{", indent, container_id, key).unwrap();
        } else {
            writeln!(
                output,
                "{}component {} \"{}\" \"{}\" {{",
                indent,
                container_id,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        self.write_view_content(output, view.properties.auto_layout.is_some(), depth + 1);

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_dynamic_view(&self, output: &mut String, view: &DynamicView, depth: usize) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");

        if title.is_empty() {
            writeln!(output, "{}dynamic * \"{}\" {{", indent, key).unwrap();
        } else {
            writeln!(
                output,
                "{}dynamic * \"{}\" \"{}\" {{",
                indent,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        // Write autoLayout if present
        if view.properties.auto_layout.is_some() {
            let inner_indent = self.indent(depth + 1);
            writeln!(output, "{}autoLayout", inner_indent).unwrap();
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_deployment_view(&self, output: &mut String, view: &DeploymentView, depth: usize) {
        let indent = self.indent(depth);
        let key = &view.properties.key;
        let title = view.properties.title.as_deref().unwrap_or("");
        let env = &view.environment;

        if title.is_empty() {
            writeln!(output, "{}deployment * \"{}\" \"{}\" {{", indent, env, key).unwrap();
        } else {
            writeln!(
                output,
                "{}deployment * \"{}\" \"{}\" \"{}\" {{",
                indent,
                env,
                key,
                escape_string(title)
            )
            .unwrap();
        }

        self.write_view_content(output, view.properties.auto_layout.is_some(), depth + 1);

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_view_content(&self, output: &mut String, has_auto_layout: bool, depth: usize) {
        let indent = self.indent(depth);

        // Default include all
        writeln!(output, "{}include *", indent).unwrap();

        // Write autoLayout
        if has_auto_layout {
            writeln!(output, "{}autoLayout", indent).unwrap();
        }
    }

    fn write_styles(
        &self,
        output: &mut String,
        element_styles: &[ElementStyle],
        relationship_styles: &[RelationshipStyle],
        depth: usize,
    ) {
        let indent = self.indent(depth);

        writeln!(output, "{}styles {{", indent).unwrap();

        for style in element_styles {
            self.write_element_style(output, style, depth + 1);
        }

        for style in relationship_styles {
            self.write_relationship_style(output, style, depth + 1);
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_element_style(&self, output: &mut String, style: &ElementStyle, depth: usize) {
        let indent = self.indent(depth);

        writeln!(output, "{}element \"{}\" {{", indent, style.tag).unwrap();

        let inner_indent = self.indent(depth + 1);

        if let Some(shape) = &style.shape {
            writeln!(output, "{}shape {}", inner_indent, shape_to_dsl(shape)).unwrap();
        }
        if let Some(bg) = &style.background {
            writeln!(output, "{}background \"{}\"", inner_indent, bg).unwrap();
        }
        if let Some(color) = &style.color {
            writeln!(output, "{}color \"{}\"", inner_indent, color).unwrap();
        }
        if let Some(stroke) = &style.stroke {
            writeln!(output, "{}stroke \"{}\"", inner_indent, stroke).unwrap();
        }
        if let Some(font_size) = style.font_size {
            writeln!(output, "{}fontSize {}", inner_indent, font_size).unwrap();
        }
        if let Some(icon) = &style.icon {
            writeln!(output, "{}icon \"{}\"", inner_indent, icon).unwrap();
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    fn write_relationship_style(
        &self,
        output: &mut String,
        style: &RelationshipStyle,
        depth: usize,
    ) {
        let indent = self.indent(depth);

        writeln!(output, "{}relationship \"{}\" {{", indent, style.tag).unwrap();

        let inner_indent = self.indent(depth + 1);

        if let Some(color) = &style.color {
            writeln!(output, "{}color \"{}\"", inner_indent, color).unwrap();
        }
        if let Some(thickness) = style.thickness {
            writeln!(output, "{}thickness {}", inner_indent, thickness).unwrap();
        }
        if let Some(line_style) = &style.style {
            if *line_style == LineStyle::Dashed {
                writeln!(output, "{}dashed true", inner_indent).unwrap();
            }
        }

        writeln!(output, "{}}}", indent).unwrap();
    }

    // Helper methods

    fn indent(&self, depth: usize) -> String {
        self.options.indent.repeat(depth)
    }

    fn generate_identifier(&self, name: &str) -> String {
        // Convert name to a valid DSL identifier
        // - Start with lowercase letter
        // - Use camelCase
        // - Remove special characters
        let mut result = String::new();
        let mut capitalize_next = false;

        for c in name.chars() {
            if c.is_alphanumeric() {
                if result.is_empty() {
                    result.push(c.to_ascii_lowercase());
                } else if capitalize_next {
                    result.push(c.to_ascii_uppercase());
                    capitalize_next = false;
                } else {
                    result.push(c);
                }
            } else if c == ' ' || c == '-' || c == '_' {
                capitalize_next = true;
            }
        }

        if result.is_empty() {
            "element".to_string()
        } else {
            result
        }
    }

    fn find_element_name(
        &self,
        id: structurizr_core::model::ElementId,
        workspace: &Workspace,
    ) -> String {
        // Search in people
        for person in &workspace.model.people {
            if person.id() == id {
                return person.name().to_string();
            }
        }

        // Search in software systems and their children
        for system in &workspace.model.software_systems {
            if system.id() == id {
                return system.name().to_string();
            }
            for container in &system.containers {
                if container.id() == id {
                    return container.name().to_string();
                }
                for component in &container.components {
                    if component.id() == id {
                        return component.name().to_string();
                    }
                }
            }
        }

        // Fallback to ID string
        format!("element_{}", id)
    }
}

impl Default for DslSerializer {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert a Shape enum to DSL string
fn shape_to_dsl(shape: &Shape) -> &'static str {
    match shape {
        Shape::Box => "Box",
        Shape::RoundedBox => "RoundedBox",
        Shape::Circle => "Circle",
        Shape::Ellipse => "Ellipse",
        Shape::Hexagon => "Hexagon",
        Shape::Cylinder => "Cylinder",
        Shape::Pipe => "Pipe",
        Shape::Person => "Person",
        Shape::Robot => "Robot",
        Shape::Folder => "Folder",
        Shape::WebBrowser => "WebBrowser",
        Shape::MobileDevicePortrait => "MobileDevicePortrait",
        Shape::MobileDeviceLandscape => "MobileDeviceLandscape",
        Shape::Component => "Component",
    }
}

/// Escape special characters in a string for DSL output
fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Convenience function to serialize a workspace to DSL
pub fn serialize(workspace: &Workspace) -> String {
    DslSerializer::new().serialize(workspace)
}

/// Convenience function to serialize a workspace with custom options
pub fn serialize_with_options(workspace: &Workspace, options: SerializerOptions) -> String {
    DslSerializer::with_options(options).serialize(workspace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::model::{Container, Person, SoftwareSystem};

    #[test]
    fn test_generate_identifier() {
        let serializer = DslSerializer::new();
        assert_eq!(serializer.generate_identifier("My System"), "mySystem");
        assert_eq!(serializer.generate_identifier("API Gateway"), "aPIGateway");
        assert_eq!(serializer.generate_identifier("user-service"), "userService");
        assert_eq!(
            serializer.generate_identifier("Test_Component"),
            "testComponent"
        );
    }

    #[test]
    fn test_escape_string() {
        assert_eq!(escape_string("Hello World"), "Hello World");
        assert_eq!(escape_string("Say \"Hello\""), "Say \\\"Hello\\\"");
        assert_eq!(escape_string("Line1\nLine2"), "Line1\\nLine2");
    }

    #[test]
    fn test_serialize_empty_workspace() {
        let workspace = Workspace::new("Test Workspace", "Test description");
        let dsl = serialize(&workspace);

        assert!(dsl.contains("workspace \"Test Workspace\" \"Test description\""));
        assert!(dsl.contains("model {"));
        assert!(dsl.contains("views {"));
    }

    #[test]
    fn test_serialize_workspace_with_person() {
        let mut workspace = Workspace::new("Test", "");
        let person = Person::new("User").with_description("A user");
        workspace.model.people.push(person);

        let dsl = serialize(&workspace);

        assert!(dsl.contains("person \"User\""));
        assert!(dsl.contains("\"A user\""));
    }

    #[test]
    fn test_serialize_workspace_with_system() {
        let mut workspace = Workspace::new("Test", "");
        let system = SoftwareSystem::new("My System").with_description("A system");
        workspace.model.software_systems.push(system);

        let dsl = serialize(&workspace);

        assert!(dsl.contains("softwareSystem \"My System\""));
        assert!(dsl.contains("\"A system\""));
    }

    #[test]
    fn test_serialize_workspace_with_container() {
        let mut workspace = Workspace::new("Test", "");
        let mut system = SoftwareSystem::new("My System");
        let container = Container::new("API").with_technology("Rust");
        system.containers.push(container);
        workspace.model.software_systems.push(system);

        let dsl = serialize(&workspace);

        assert!(dsl.contains("container \"API\""));
        assert!(dsl.contains("\"Rust\""));
    }

    #[test]
    fn test_roundtrip_basic() {
        // Create a workspace
        let mut workspace = Workspace::new("Roundtrip Test", "Testing round-trip");

        let user = Person::new("User").with_description("An end user");
        let system = SoftwareSystem::new("System").with_description("Main system");

        let user_id = user.id();
        let system_id = system.id();

        workspace.model.people.push(user);
        workspace.model.software_systems.push(system);

        // Add a relationship
        let rel = structurizr_core::model::Relationship::new(user_id, system_id)
            .with_description("Uses");
        workspace.model.relationships.push(rel);

        // Serialize
        let dsl = serialize(&workspace);

        // Verify the DSL contains expected elements
        assert!(dsl.contains("workspace \"Roundtrip Test\" \"Testing round-trip\""));
        assert!(dsl.contains("person \"User\""));
        assert!(dsl.contains("softwareSystem \"System\""));
        assert!(dsl.contains("user -> system \"Uses\""));
    }
}
