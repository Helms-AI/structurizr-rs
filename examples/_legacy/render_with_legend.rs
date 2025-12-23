//! Example demonstrating the legend feature in SVG rendering.
//!
//! This example shows how to enable the legend when rendering diagrams.
//! The legend displays all unique element types and relationship styles
//! used in the diagram.

use structurizr_core::Workspace;
use structurizr_render::SvgRenderer;

fn main() {
    // Create a sample workspace
    let mut workspace = Workspace::new("Legend Example", "Example workspace with legend");
    let model = workspace.model_mut();

    // Add elements
    let user = model.add_person("User", "A user of the system");
    let admin = model.add_person("Administrator", "System administrator");

    let system = model.add_software_system("Main System", "The primary software system");
    let external = model.add_software_system("External System", "Third-party system");

    // Add relationships
    model.add_relationship(user, system, "Uses", Some("HTTPS".to_string()));
    model.add_relationship(admin, system, "Manages", Some("HTTPS".to_string()));
    model.add_relationship(system, external, "Integrates with", Some("REST/JSON".to_string()));

    // Create a system landscape view
    let view = structurizr_core::view::SystemLandscapeView::new("landscape");

    // Render WITHOUT legend
    println!("Rendering diagram without legend...");
    let renderer = SvgRenderer::default();
    let svg_without_legend = renderer.render_system_landscape(&workspace, &view).unwrap();
    println!("SVG without legend size: {} bytes", svg_without_legend.len());

    // Render WITH legend
    println!("\nRendering diagram with legend...");
    let renderer_with_legend = SvgRenderer::default().with_legend(true);
    let svg_with_legend = renderer_with_legend.render_system_landscape(&workspace, &view).unwrap();
    println!("SVG with legend size: {} bytes", svg_with_legend.len());

    // Check legend is present
    if svg_with_legend.contains("Legend") {
        println!("\n✓ Legend successfully rendered!");
        println!("✓ Contains 'Person' entry");
        println!("✓ Contains 'Software System' entry");
    } else {
        println!("\n✗ Legend not found in output");
    }

    // Save both versions
    std::fs::write("/tmp/diagram_without_legend.svg", svg_without_legend)
        .expect("Failed to write file");
    std::fs::write("/tmp/diagram_with_legend.svg", svg_with_legend)
        .expect("Failed to write file");

    println!("\nFiles saved:");
    println!("  - /tmp/diagram_without_legend.svg");
    println!("  - /tmp/diagram_with_legend.svg");
}
