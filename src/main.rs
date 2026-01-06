//! Structurizr CLI - A Rust implementation of Structurizr Lite.
//!
//! This CLI tool provides commands to:
//! - Start a web server for viewing diagrams
//! - Parse and validate DSL files
//! - Export diagrams to various formats

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use structurizr_core::Workspace;
use structurizr_dsl::parse_with_base_path;
use structurizr_export::{JsonExporter, PlantUmlExporter, MermaidExporter};
use structurizr_render::SvgRenderer;
use structurizr_web::{Server, state::Config};

/// Structurizr - Software architecture visualization tool
#[derive(Parser)]
#[command(name = "structurizr")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the web server
    Serve {
        /// Directory containing workspaces
        #[arg(long, default_value = "workspaces")]
        workspaces_dir: PathBuf,

        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Host to bind to
        #[arg(long, default_value = "127.0.0.1")]
        host: String,
    },

    /// Parse and validate a DSL file
    Validate {
        /// Path to workspace.dsl file
        #[arg(default_value = "workspace.dsl")]
        file: PathBuf,
    },

    /// Export workspace to various formats
    Export {
        /// Path to workspace file (DSL or JSON)
        #[arg(short, long, default_value = "workspace.dsl")]
        workspace: PathBuf,

        /// Output format (json, plantuml, mermaid)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Output file (defaults to stdout)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Render a view to SVG
    Render {
        /// Path to workspace file (DSL or JSON)
        #[arg(short, long, default_value = "workspace.dsl")]
        workspace: PathBuf,

        /// View key to render (or "all" for all views)
        #[arg(long, default_value = "all")]
        view: String,

        /// Output directory for SVG files
        #[arg(short, long, default_value = ".")]
        output: PathBuf,
    },

    /// Create a new workspace
    Init {
        /// Name of the workspace
        #[arg(default_value = "My System")]
        name: String,

        /// Output file
        #[arg(short, long, default_value = "workspace.dsl")]
        output: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("structurizr={}", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    match cli.command {
        Commands::Serve { workspaces_dir, port, host } => {
            if !workspaces_dir.exists() {
                eprintln!("Error: Workspaces directory '{}' does not exist.", workspaces_dir.display());
                eprintln!("Create the directory and add workspace subdirectories, or use --workspaces-dir to specify a different path.");
                std::process::exit(1);
            }

            println!("Starting Structurizr server...");
            println!("  Workspaces directory: {}", workspaces_dir.display());
            println!("  URL: http://{}:{}", host, port);
            println!();

            let config = Config::new(&workspaces_dir)
                .with_port(port)
                .with_host(&host);

            Server::new(config).run().await?;
        }

        Commands::Validate { file } => {
            println!("Validating {}...", file.display());
            println!();

            let content = std::fs::read_to_string(&file)?;
            let base_path = file.parent();
            match parse_with_base_path(&content, base_path) {
                Ok(workspace) => {
                    println!("✓ DSL Parsing: Valid workspace: {}", workspace.name);
                    println!("  People: {}", workspace.model().people.len());
                    println!("  Software Systems: {}", workspace.model().software_systems.len());
                    println!("  Relationships: {}", workspace.model().relationships.len());
                    println!("  Views: {}", workspace.views().all_keys().len());
                    println!();

                    // Run workspace validation
                    println!("Running workspace inspections...");
                    let validation_result = structurizr_dsl::validate_workspace(&workspace);

                    if validation_result.has_issues() {
                        println!();
                        println!("Issues found:");
                        println!(
                            "  Errors: {}, Warnings: {}, Info: {}",
                            validation_result.error_count,
                            validation_result.warning_count,
                            validation_result.info_count
                        );
                        println!();

                        // Group issues by severity
                        for severity in &[structurizr_dsl::Severity::Error, structurizr_dsl::Severity::Warning, structurizr_dsl::Severity::Info] {
                            let issues = validation_result.issues_by_severity(*severity);
                            if !issues.is_empty() {
                                for issue in issues {
                                    let element_info = if let Some(name) = &issue.element_name {
                                        format!(" [{}]", name)
                                    } else {
                                        String::new()
                                    };
                                    println!("  [{}] {}{}", issue.severity, issue.message, element_info);
                                }
                            }
                        }

                        println!();
                        if validation_result.is_valid() {
                            println!("✓ Validation passed (warnings and info only)");
                        } else {
                            eprintln!("✗ Validation failed with {} error(s)", validation_result.error_count);
                            std::process::exit(1);
                        }
                    } else {
                        println!("✓ No issues found - workspace is clean!");
                    }
                }
                Err(e) => {
                    eprintln!("✗ DSL Parsing failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        Commands::Export { workspace, format, output } => {
            let ws = load_workspace(&workspace)?;

            let exported = match format.to_lowercase().as_str() {
                "json" => JsonExporter::export(&ws)?,
                "plantuml" => {
                    let view = structurizr_core::view::SystemLandscapeView::new("landscape");
                    PlantUmlExporter::export_system_landscape(&ws, &view)?
                }
                "mermaid" => MermaidExporter::export_flowchart(&ws)?,
                _ => {
                    eprintln!("Unknown format: {}. Supported: json, plantuml, mermaid", format);
                    std::process::exit(1);
                }
            };

            if let Some(output_path) = output {
                std::fs::write(&output_path, &exported)?;
                println!("Exported to {}", output_path.display());
            } else {
                println!("{}", exported);
            }
        }

        Commands::Render { workspace, view, output } => {
            let ws = load_workspace(&workspace)?;
            let renderer = SvgRenderer::default();

            std::fs::create_dir_all(&output)?;

            if view == "all" {
                // Render all views
                for key in ws.views().all_keys() {
                    let svg = render_view(&ws, &renderer, key)?;
                    let output_file = output.join(format!("{}.svg", key));
                    std::fs::write(&output_file, &svg)?;
                    println!("Rendered {} -> {}", key, output_file.display());
                }

                // If no views, render a default landscape
                if ws.views().all_keys().is_empty() {
                    let landscape = structurizr_core::view::SystemLandscapeView::new("landscape");
                    let svg = renderer.render_system_landscape(&ws, &landscape)?;
                    let output_file = output.join("landscape.svg");
                    std::fs::write(&output_file, &svg)?;
                    println!("Rendered landscape -> {}", output_file.display());
                }
            } else {
                let svg = render_view(&ws, &renderer, &view)?;
                let output_file = output.join(format!("{}.svg", view));
                std::fs::write(&output_file, &svg)?;
                println!("Rendered {} -> {}", view, output_file.display());
            }
        }

        Commands::Init { name, output } => {
            let dsl = format!(
                r##"workspace "{}" "Description of the system" {{

    model {{
        user = person "User" "A user of the system"
        system = softwareSystem "{}" "Description of the software system" {{
            webapp = container "Web Application" "Provides functionality via web browser" "React"
            api = container "API" "Provides REST API" "Rust/Axum"
            db = container "Database" "Stores data" "PostgreSQL"
        }}

        user -> webapp "Uses"
        webapp -> api "Makes API calls to" "HTTPS/JSON"
        api -> db "Reads from and writes to" "SQL"
    }}

    views {{
        systemContext system "SystemContext" "System Context diagram" {{
            include *
            autoLayout
        }}

        container system "Containers" "Container diagram" {{
            include *
            autoLayout
        }}

        styles {{
            element "Person" {{
                shape Person
                background "#08427b"
                color "#ffffff"
            }}
            element "Software System" {{
                background "#1168bd"
                color "#ffffff"
            }}
            element "Container" {{
                background "#438dd5"
                color "#ffffff"
            }}
        }}
    }}
}}
"##,
                name, name
            );

            std::fs::write(&output, &dsl)?;
            println!("Created {}", output.display());
            println!();
            println!("Next steps:");
            println!("  1. Edit {} to define your architecture", output.display());
            println!("  2. Run 'structurizr serve' to view your diagrams");
        }
    }

    Ok(())
}

fn load_workspace(path: &PathBuf) -> anyhow::Result<Workspace> {
    let content = std::fs::read_to_string(path)?;
    let base_path = path.parent();

    if path.extension().and_then(|e| e.to_str()) == Some("json") {
        Ok(Workspace::from_json(&content)?)
    } else {
        Ok(parse_with_base_path(&content, base_path)?)
    }
}

fn render_view(
    workspace: &Workspace,
    renderer: &SvgRenderer,
    view_key: &str,
) -> anyhow::Result<String> {
    // Try to find the view by key across all view types
    if let Some(view) = workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_system_landscape(workspace, view)?);
    }

    if let Some(view) = workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_system_context(workspace, view)?);
    }

    if let Some(view) = workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_container(workspace, view)?);
    }

    if let Some(view) = workspace.views().component_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_component(workspace, view)?);
    }

    if let Some(view) = workspace.views().deployment_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_deployment(workspace, view)?);
    }

    if let Some(view) = workspace.views().dynamic_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_dynamic(workspace, view)?);
    }

    if let Some(view) = workspace.views().filtered_views.iter().find(|v| v.properties.key == view_key) {
        return Ok(renderer.render_filtered(workspace, view)?);
    }

    // Default: render as system landscape
    let view = structurizr_core::view::SystemLandscapeView::new(view_key);
    Ok(renderer.render_system_landscape(workspace, &view)?)
}
