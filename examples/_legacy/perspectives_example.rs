//! Example demonstrating perspectives support in structurizr-rs.
//!
//! This example shows how to:
//! 1. Add perspectives to elements
//! 2. Create perspectives in the workspace
//! 3. Use perspectives to filter views by stakeholder
//!
//! Run with: cargo run --example perspectives_example

use structurizr_core::{
    Container, Perspective, SoftwareSystem, Workspace,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a workspace
    let mut workspace = Workspace::new(
        "E-Commerce System",
        "Example demonstrating perspectives for different stakeholders"
    );

    // Define perspectives for different stakeholders
    workspace.add_perspective(
        Perspective::new("Business")
            .with_description("Shows business-relevant components and workflows")
    );
    workspace.add_perspective(
        Perspective::new("Technical")
            .with_description("Shows technical implementation details and infrastructure")
    );
    workspace.add_perspective(
        Perspective::new("Security")
            .with_description("Shows security-critical components and data flows")
    );

    // Create model elements with perspectives
    let model = workspace.model_mut();

    // Users are relevant to all perspectives (no perspective specified = visible in all)
    let customer = model.add_person("Customer", "A customer of the e-commerce platform");
    let admin = model.add_person("Admin", "System administrator");

    // Create the main system
    let mut ecommerce_system = SoftwareSystem::new("E-Commerce System")
        .with_description("Handles online shopping");

    // Add perspectives to the system itself
    ecommerce_system.properties = ecommerce_system.properties
        .with_perspectives(vec!["Business", "Technical", "Security"]);

    // Web Application - visible in Business and Technical perspectives
    let mut web_app = Container::new("Web Application")
        .with_description("Provides the user interface")
        .with_technology("React");
    web_app.properties = web_app.properties
        .with_perspectives(vec!["Business", "Technical"]);

    // API Gateway - visible in Technical and Security perspectives
    let mut api_gateway = Container::new("API Gateway")
        .with_description("Routes requests and handles authentication")
        .with_technology("Kong");
    api_gateway.properties = api_gateway.properties
        .with_perspectives(vec!["Technical", "Security"]);

    // Database - visible in all perspectives
    let mut database = Container::new("Database")
        .with_description("Stores product and order data")
        .with_technology("PostgreSQL");
    database.properties = database.properties
        .with_perspectives(vec!["Business", "Technical", "Security"]);

    // Payment Service - visible in Business and Security perspectives
    let mut payment_service = Container::new("Payment Service")
        .with_description("Processes payments securely")
        .with_technology("Rust microservice");
    payment_service.properties = payment_service.properties
        .with_perspectives(vec!["Business", "Security"]);

    // Analytics Engine - visible in Business and Technical perspectives
    let mut analytics = Container::new("Analytics Engine")
        .with_description("Analyzes customer behavior and sales")
        .with_technology("Apache Spark");
    analytics.properties = analytics.properties
        .with_perspectives(vec!["Business", "Technical"]);

    // Add containers to the system
    let web_app_id = ecommerce_system.add_container(web_app);
    let api_gateway_id = ecommerce_system.add_container(api_gateway);
    let database_id = ecommerce_system.add_container(database);
    let payment_service_id = ecommerce_system.add_container(payment_service);
    let analytics_id = ecommerce_system.add_container(analytics);

    let _system_id = model.software_systems.len();
    model.software_systems.push(ecommerce_system);

    // Add relationships (using the IDs we captured earlier)
    // Customer uses web application
    model.add_relationship(
        customer,
        web_app_id,
        "Browses products and places orders",
        Some("HTTPS".to_string())
    );

    // Web app calls API gateway
    model.add_relationship(
        web_app_id,
        api_gateway_id,
        "Makes API calls",
        Some("REST/HTTPS".to_string())
    );

    // API gateway accesses database
    model.add_relationship(
        api_gateway_id,
        database_id,
        "Reads/writes data",
        Some("SQL".to_string())
    );

    // Payment service accesses database
    model.add_relationship(
        payment_service_id,
        database_id,
        "Records transactions",
        Some("SQL".to_string())
    );

    // API gateway calls payment service
    model.add_relationship(
        api_gateway_id,
        payment_service_id,
        "Processes payments",
        Some("gRPC".to_string())
    );

    // Analytics reads from database
    model.add_relationship(
        analytics_id,
        database_id,
        "Analyzes data",
        Some("SQL".to_string())
    );

    // Admin uses analytics
    model.add_relationship(
        admin,
        analytics_id,
        "Views reports",
        Some("HTTPS".to_string())
    );

    // Save the workspace
    workspace.to_json_file("perspectives_example.json")?;

    println!("✓ Created workspace with perspectives support");
    println!("\nPerspectives defined:");
    for perspective in workspace.get_perspectives() {
        println!("  - {}: {}",
            perspective.name,
            perspective.description.as_deref().unwrap_or("No description")
        );
    }

    println!("\nElements by perspective:");

    println!("\n  Business perspective:");
    println!("    - Customer (no specific perspective = visible in all)");
    println!("    - Admin (no specific perspective = visible in all)");
    println!("    - E-Commerce System");
    println!("    - Web Application");
    println!("    - Database");
    println!("    - Payment Service");
    println!("    - Analytics Engine");

    println!("\n  Technical perspective:");
    println!("    - Customer (no specific perspective = visible in all)");
    println!("    - Admin (no specific perspective = visible in all)");
    println!("    - E-Commerce System");
    println!("    - Web Application");
    println!("    - API Gateway");
    println!("    - Database");
    println!("    - Analytics Engine");

    println!("\n  Security perspective:");
    println!("    - Customer (no specific perspective = visible in all)");
    println!("    - Admin (no specific perspective = visible in all)");
    println!("    - E-Commerce System");
    println!("    - API Gateway");
    println!("    - Database");
    println!("    - Payment Service");

    println!("\n✓ Workspace saved to perspectives_example.json");
    println!("\nTo view with perspective filtering:");
    println!("  1. Start the web server: cargo run -- serve");
    println!("  2. Access with perspective filter:");
    println!("     - Business view: http://localhost:8080/view/SystemLandscape/svg?perspective=Business");
    println!("     - Technical view: http://localhost:8080/view/SystemLandscape/svg?perspective=Technical");
    println!("     - Security view: http://localhost:8080/view/SystemLandscape/svg?perspective=Security");
    println!("     - All elements: http://localhost:8080/view/SystemLandscape/svg");

    Ok(())
}
