//! Tests for perspectives functionality.

#[cfg(test)]
mod tests {
    use crate::{Container, Person, Perspective, SoftwareSystem, Workspace};

    #[test]
    fn test_perspective_creation() {
        let p = Perspective::new("Business")
            .with_description("Business view");

        assert_eq!(p.name, "Business");
        assert_eq!(p.description, Some("Business view".to_string()));
    }

    #[test]
    fn test_workspace_perspectives() {
        let mut ws = Workspace::new("Test", "Test workspace");

        assert_eq!(ws.get_perspectives().len(), 0);

        ws.add_perspective(Perspective::new("Business"));
        ws.add_perspective(Perspective::new("Technical"));

        assert_eq!(ws.get_perspectives().len(), 2);
        assert_eq!(ws.get_perspectives()[0].name, "Business");
        assert_eq!(ws.get_perspectives()[1].name, "Technical");
    }

    #[test]
    fn test_element_perspectives() {
        let mut system = SoftwareSystem::new("Test System");

        // Initially no perspectives
        assert!(system.properties.perspectives.is_empty());

        // Add perspectives
        system.properties = system.properties
            .with_perspective("Business")
            .with_perspective("Technical");

        assert_eq!(system.properties.perspectives.len(), 2);
        assert!(system.properties.perspectives.contains(&"Business".to_string()));
        assert!(system.properties.perspectives.contains(&"Technical".to_string()));
    }

    #[test]
    fn test_element_perspectives_batch() {
        let mut container = Container::new("Database");

        container.properties = container.properties
            .with_perspectives(vec!["Business", "Technical", "Security"]);

        assert_eq!(container.properties.perspectives.len(), 3);
    }

    #[test]
    fn test_element_without_perspectives_visible_everywhere() {
        let person = Person::new("User");

        // No perspectives means visible in all perspectives
        assert!(person.properties.perspectives.is_empty());
    }

    #[test]
    fn test_workspace_serialization_with_perspectives() {
        let mut ws = Workspace::new("Test", "Test workspace");

        ws.add_perspective(
            Perspective::new("Business")
                .with_description("Business stakeholders view")
        );

        let mut system = SoftwareSystem::new("System");
        system.properties = system.properties.with_perspective("Business");

        ws.model_mut().software_systems.push(system);

        // Serialize to JSON
        let json = ws.to_json().expect("Failed to serialize");

        // Deserialize back
        let restored = Workspace::from_json(&json).expect("Failed to deserialize");

        assert_eq!(restored.perspectives.len(), 1);
        assert_eq!(restored.perspectives[0].name, "Business");
        assert_eq!(
            restored.perspectives[0].description,
            Some("Business stakeholders view".to_string())
        );

        assert_eq!(restored.model.software_systems.len(), 1);
        assert_eq!(restored.model.software_systems[0].properties.perspectives.len(), 1);
        assert_eq!(
            restored.model.software_systems[0].properties.perspectives[0],
            "Business"
        );
    }

    #[test]
    fn test_empty_perspectives_serialization() {
        let ws = Workspace::new("Test", "Test workspace");

        let json = ws.to_json().expect("Failed to serialize");

        // Empty perspectives array should be omitted in JSON
        assert!(!json.contains("\"perspectives\""));
    }

    #[test]
    fn test_element_properties_with_perspectives() {
        let mut props = crate::ElementProperties::new("Test Element");

        props = props.with_perspectives(vec!["A", "B", "C"]);

        assert_eq!(props.perspectives.len(), 3);
        assert!(props.perspectives.contains(&"A".to_string()));
        assert!(props.perspectives.contains(&"B".to_string()));
        assert!(props.perspectives.contains(&"C".to_string()));
    }
}
