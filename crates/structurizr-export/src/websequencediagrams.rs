//! WebSequenceDiagrams export for Structurizr dynamic views.
//!
//! Exports DynamicView as WebSequenceDiagrams format (websequencediagrams.com).

use structurizr_core::model::InteractionStyle;
use structurizr_core::view::DynamicView;
use structurizr_core::Workspace;

use crate::error::Result;

/// Exports dynamic views to WebSequenceDiagrams text format.
pub struct WebSequenceDiagramsExporter;

impl WebSequenceDiagramsExporter {
    /// Export a dynamic view to WebSequenceDiagrams format.
    ///
    /// # Format
    /// - Title from view properties if present
    /// - Each step converted to: `source->destination: description`
    /// - Uses `->` for synchronous calls, `-->` for asynchronous
    /// - Sanitizes participant names (replaces spaces with underscores)
    /// - Steps sorted by order field
    ///
    /// # Example output
    /// ```text
    /// title Order Process
    ///
    /// User->WebApp: Submit Order
    /// WebApp->Database: Save Order
    /// Database-->WebApp: Confirmation
    /// WebApp-->User: Order Confirmed
    /// ```
    pub fn export_dynamic(workspace: &Workspace, view: &DynamicView) -> Result<String> {
        let model = workspace.model();
        let mut output = String::new();

        // Add title if present
        if let Some(ref title) = view.properties.title {
            output.push_str(&format!("title {}\n\n", title));
        }

        // Sort steps by order
        let mut sorted_steps: Vec<_> = view.steps.iter().collect();
        sorted_steps.sort_by_key(|s| s.order);

        // Generate sequence messages for each step
        for step in sorted_steps {
            // Find source and destination names
            let source_name = find_element_name(model, &step.source_id.to_string());
            let dest_name = find_element_name(model, &step.destination_id.to_string());

            // Sanitize names (replace spaces with underscores)
            let source = sanitize_participant_name(&source_name);
            let dest = sanitize_participant_name(&dest_name);

            // Get the relationship to determine if it's async
            let relationship = model.relationships.iter().find(|r| {
                r.source_id == step.source_id && r.destination_id == step.destination_id
            });

            // Determine arrow type based on interaction style or technology
            let arrow = if let Some(rel) = relationship {
                match rel.interaction_style {
                    InteractionStyle::Asynchronous => "-->",
                    InteractionStyle::Synchronous => {
                        // Also check technology field for "async"
                        if let Some(ref tech) = rel.technology {
                            if tech.to_lowercase().contains("async") {
                                "-->"
                            } else {
                                "->"
                            }
                        } else {
                            "->"
                        }
                    }
                }
            } else {
                "->"
            };

            // Get description
            let description = step
                .description
                .as_deref()
                .unwrap_or("uses");

            output.push_str(&format!("{}{}{}: {}\n", source, arrow, dest, description));
        }

        Ok(output)
    }
}

/// Sanitize a participant name for WebSequenceDiagrams.
/// Replaces spaces with underscores and removes special characters.
fn sanitize_participant_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else if c.is_whitespace() {
                '_'
            } else {
                '_'
            }
        })
        .collect()
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

    // If not found, return the ID
    id.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use structurizr_core::view::DynamicViewStep;
    use structurizr_core::Workspace;

    #[test]
    fn test_export_dynamic_with_title() {
        let mut workspace = Workspace::new("Test", "A test");
        let user = workspace.model_mut().add_person("User", "A user");
        let webapp = workspace.model_mut().add_software_system("Web App", "A web application");
        let database = workspace.model_mut().add_software_system("Database", "A database");

        // Add relationships
        workspace
            .model_mut()
            .add_relationship(user, webapp, "Submit Order", None);
        workspace
            .model_mut()
            .add_relationship(webapp, database, "Save Order", None);

        // Create dynamic view
        let mut view = DynamicView::new("order-process");
        view.properties.title = Some("Order Process".to_string());

        // Add steps
        view.steps.push(DynamicViewStep {
            source_id: user,
            destination_id: webapp,
            description: Some("Submit Order".to_string()),
            order: 1,
        });
        view.steps.push(DynamicViewStep {
            source_id: webapp,
            destination_id: database,
            description: Some("Save Order".to_string()),
            order: 2,
        });

        let output = WebSequenceDiagramsExporter::export_dynamic(&workspace, &view).unwrap();

        assert!(output.contains("title Order Process"));
        assert!(output.contains("User->Web_App: Submit Order"));
        assert!(output.contains("Web_App->Database: Save Order"));
    }

    #[test]
    fn test_export_dynamic_without_title() {
        let mut workspace = Workspace::new("Test", "A test");
        let user = workspace.model_mut().add_person("User", "A user");
        let system = workspace.model_mut().add_software_system("System", "A system");

        workspace
            .model_mut()
            .add_relationship(user, system, "Uses", None);

        let mut view = DynamicView::new("interaction");
        view.steps.push(DynamicViewStep {
            source_id: user,
            destination_id: system,
            description: Some("Uses".to_string()),
            order: 1,
        });

        let output = WebSequenceDiagramsExporter::export_dynamic(&workspace, &view).unwrap();

        assert!(!output.contains("title"));
        assert!(output.contains("User->System: Uses"));
    }

    #[test]
    fn test_async_relationship_detection() {
        let mut workspace = Workspace::new("Test", "A test");
        let user = workspace.model_mut().add_person("User", "A user");
        let system = workspace.model_mut().add_software_system("System", "A system");

        // Add async relationship using technology hint
        workspace
            .model_mut()
            .add_relationship(user, system, "Sends message", Some("Async Queue".to_string()));

        let mut view = DynamicView::new("async-test");
        view.steps.push(DynamicViewStep {
            source_id: user,
            destination_id: system,
            description: Some("Sends message".to_string()),
            order: 1,
        });

        let output = WebSequenceDiagramsExporter::export_dynamic(&workspace, &view).unwrap();

        // Should use async arrow
        assert!(output.contains("User-->System: Sends message"));
    }

    #[test]
    fn test_sanitize_participant_name() {
        assert_eq!(sanitize_participant_name("User"), "User");
        assert_eq!(sanitize_participant_name("Web App"), "Web_App");
        assert_eq!(sanitize_participant_name("API Gateway"), "API_Gateway");
        assert_eq!(sanitize_participant_name("User-Service"), "User_Service");
        assert_eq!(
            sanitize_participant_name("System@Backend"),
            "System_Backend"
        );
    }

    #[test]
    fn test_step_ordering() {
        let mut workspace = Workspace::new("Test", "A test");
        let a = workspace.model_mut().add_person("A", "A");
        let b = workspace.model_mut().add_person("B", "B");
        let c = workspace.model_mut().add_person("C", "C");

        workspace.model_mut().add_relationship(a, b, "First", None);
        workspace.model_mut().add_relationship(b, c, "Second", None);

        let mut view = DynamicView::new("ordering");

        // Add steps out of order
        view.steps.push(DynamicViewStep {
            source_id: b,
            destination_id: c,
            description: Some("Second".to_string()),
            order: 2,
        });
        view.steps.push(DynamicViewStep {
            source_id: a,
            destination_id: b,
            description: Some("First".to_string()),
            order: 1,
        });

        let output = WebSequenceDiagramsExporter::export_dynamic(&workspace, &view).unwrap();

        // Should be ordered correctly
        let lines: Vec<&str> = output.lines().collect();
        assert!(lines[0].contains("A->B: First"));
        assert!(lines[1].contains("B->C: Second"));
    }
}
