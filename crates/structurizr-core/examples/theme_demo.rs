//! Example demonstrating theme fetching and application.
//!
//! This example shows how to:
//! 1. Create a workspace with styles
//! 2. Add theme URLs
//! 3. Apply themes to merge styles
//!
//! Run with: cargo run --example theme_demo

use structurizr_core::{ElementStyle, Workspace};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new workspace
    let mut workspace = Workspace::new("My System", "A demonstration of theme support");

    // Add some elements to the model
    let user_id = workspace.model_mut().add_person("User", "A user of the system");
    let system_id =
        workspace
            .model_mut()
            .add_software_system("System", "The software system");
    workspace
        .model_mut()
        .add_relationship(user_id, system_id, "Uses", None);

    // Add a custom element style (this will override theme styles)
    workspace
        .styles_mut()
        .add_element_style(ElementStyle::new("User").with_background("#FF0000"));

    // Add a theme URL (in real usage, this would be a valid URL to a JSON theme)
    // For this demo, we'll just show the structure without actually fetching
    workspace
        .styles_mut()
        .add_theme("https://static.structurizr.com/themes/default/theme.json");

    // Print workspace info
    println!("Workspace: {}", workspace.name);
    println!("Description: {:?}", workspace.description);
    println!("\nModel:");
    println!("  People: {}", workspace.model().people.len());
    println!(
        "  Software Systems: {}",
        workspace.model().software_systems.len()
    );
    println!("  Relationships: {}", workspace.model().relationships.len());
    println!("\nStyles:");
    if let Some(styles) = workspace.styles() {
        println!("  Element styles: {}", styles.elements.len());
        println!("  Relationship styles: {}", styles.relationships.len());
        println!("  Theme URLs: {:?}", styles.themes);
    }

    // Note: In a real application, you would call apply_themes() to fetch and merge themes:
    // apply_themes(&mut workspace)?;
    //
    // This would:
    // 1. Fetch the JSON theme from the URL
    // 2. Parse it into ElementStyle and RelationshipStyle objects
    // 3. Merge theme styles with workspace styles (workspace styles override themes)
    // 4. Cache the theme to avoid re-fetching

    println!("\nNote: Theme fetching is disabled in this demo.");
    println!("In production, call apply_themes(&mut workspace) to fetch and apply themes.");

    Ok(())
}
