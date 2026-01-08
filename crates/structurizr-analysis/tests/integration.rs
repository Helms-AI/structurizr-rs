//! Integration tests for the structurizr-analysis crate.

use std::path::PathBuf;
use structurizr_analysis::{analyze_and_generate_dsl, AnalyzerConfig, GeneratorConfig, detect_project};

/// Get the path to the structurizr-rs root directory.
fn get_project_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().to_path_buf()
}

#[test]
fn test_detect_structurizr_rs() {
    let root = get_project_root();
    let detected = detect_project(&root).expect("Should detect project");

    assert_eq!(detected.primary_language, structurizr_analysis::Language::Rust);
    assert!(detected.is_workspace, "Should detect workspace");
    // Note: detector finds root manifest only, not workspace members
    assert!(!detected.manifests.is_empty(), "Should find at least one manifest");
    assert!(!detected.workspace_members.is_empty(), "Should find workspace members");
}

#[test]
fn test_analyze_structurizr_rs() {
    let root = get_project_root();

    let analyzer_config = AnalyzerConfig {
        max_depth: 5, // Limit depth for faster testing
        include_tests: false,
        ..Default::default()
    };

    let generator_config = GeneratorConfig::default();

    let dsl = analyze_and_generate_dsl(&root, &analyzer_config, &generator_config)
        .expect("Should generate DSL");

    // Verify basic DSL structure
    assert!(dsl.contains("workspace"), "DSL should contain workspace");
    assert!(dsl.contains("structurizr-rs"), "DSL should contain project name");
    assert!(dsl.contains("model"), "DSL should contain model section");
    assert!(dsl.contains("views"), "DSL should contain views section");

    // Should find the main crates as containers
    assert!(dsl.contains("structurizr-core") || dsl.contains("core"),
            "DSL should mention core crate");

    println!("Generated DSL for structurizr-rs:");
    println!("Length: {} bytes", dsl.len());
    println!("---");
    // Print first 1500 chars for inspection
    let preview = if dsl.len() > 1500 { &dsl[..1500] } else { &dsl };
    println!("{}", preview);
    if dsl.len() > 1500 {
        println!("... (truncated, total {} bytes)", dsl.len());
    }
}

#[test]
fn test_analyze_single_crate() {
    let root = get_project_root();
    let core_crate = root.join("crates").join("structurizr-core");

    if !core_crate.exists() {
        println!("Skipping test: core crate not found at {:?}", core_crate);
        return;
    }

    let analyzer_config = AnalyzerConfig::default();
    let generator_config = GeneratorConfig::default();

    let dsl = analyze_and_generate_dsl(&core_crate, &analyzer_config, &generator_config)
        .expect("Should generate DSL for core crate");

    assert!(dsl.contains("structurizr-core"), "DSL should contain crate name");

    println!("Generated DSL for structurizr-core:");
    println!("Length: {} bytes", dsl.len());
}
