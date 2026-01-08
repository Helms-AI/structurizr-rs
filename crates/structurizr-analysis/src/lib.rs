//! # structurizr-analysis
//!
//! Code analysis engine for generating C4 architecture models from source code.
//!
//! This crate provides:
//! - Language detection from project manifests
//! - Tree-sitter based code parsing
//! - Container and component extraction
//! - Relationship inference
//! - C4 model generation
//!
//! ## Supported Languages
//!
//! Currently supported:
//! - **Rust**: Full support for Cargo workspaces, crate analysis, module/struct/trait extraction
//!
//! Coming soon:
//! - TypeScript/JavaScript (package.json, ES modules)
//! - Python (pyproject.toml, modules)
//! - Go (go.mod, packages)
//!
//! ## Usage
//!
//! ```rust,ignore
//! use structurizr_analysis::{AnalyzerRegistry, AnalyzerConfig, detect_project, generate_dsl};
//!
//! // Detect project type
//! let detected = detect_project("/path/to/project")?;
//!
//! // Get appropriate analyzer
//! let registry = AnalyzerRegistry::with_defaults();
//! let analyzer = registry.get(detected.primary_language).unwrap();
//!
//! // Analyze the project
//! let config = AnalyzerConfig::default();
//! let project = analyzer.analyze("/path/to/project", &config)?;
//!
//! // Generate DSL
//! let dsl = generate_dsl(&project, &GeneratorConfig::default())?;
//! ```

pub mod analyzer;
pub mod detector;
pub mod error;
pub mod generator;
pub mod languages;
pub mod model;

// Re-exports for convenience
pub use analyzer::{AnalyzerConfig, AnalyzerRegistry, LanguageAnalyzer};
pub use detector::{detect_project, DetectedProject, ManifestInfo, ManifestType};
pub use error::{Error, Result};
pub use generator::{generate_dsl, generate_workspace, GeneratorConfig};
pub use model::{
    AnalyzedComponent, AnalyzedContainer, AnalyzedProject, ComponentType, ContainerType,
    ExternalDependency, InferredRelationship, Language,
};

/// Analyze a project and generate a C4 workspace.
///
/// This is the main entry point for analyzing a project. It:
/// 1. Detects the project type and languages
/// 2. Selects the appropriate analyzer
/// 3. Analyzes the codebase
/// 4. Generates a Structurizr workspace
///
/// # Arguments
///
/// * `root` - Path to the project root directory
/// * `analyzer_config` - Configuration for the analyzer
/// * `generator_config` - Configuration for workspace generation
///
/// # Returns
///
/// A `Workspace` ready for serialization to DSL or JSON.
pub fn analyze_and_generate(
    root: impl AsRef<std::path::Path>,
    analyzer_config: &AnalyzerConfig,
    generator_config: &GeneratorConfig,
) -> Result<structurizr_core::Workspace> {
    let root = root.as_ref();

    // Detect project type
    let detected = detect_project(root)?;

    // Get appropriate analyzer
    let registry = AnalyzerRegistry::with_defaults();
    let analyzer = registry
        .get(detected.primary_language)
        .ok_or_else(|| Error::UnsupportedLanguage(detected.primary_language.display_name().to_string()))?;

    // Analyze the project
    let project = analyzer.analyze(root, analyzer_config)?;

    // Generate workspace
    generate_workspace(&project, generator_config)
}

/// Analyze a project and generate DSL.
///
/// Convenience function that combines analysis and DSL generation.
pub fn analyze_and_generate_dsl(
    root: impl AsRef<std::path::Path>,
    analyzer_config: &AnalyzerConfig,
    generator_config: &GeneratorConfig,
) -> Result<String> {
    let root = root.as_ref();

    // Detect project type
    let detected = detect_project(root)?;

    // Get appropriate analyzer
    let registry = AnalyzerRegistry::with_defaults();
    let analyzer = registry
        .get(detected.primary_language)
        .ok_or_else(|| Error::UnsupportedLanguage(detected.primary_language.display_name().to_string()))?;

    // Analyze the project
    let project = analyzer.analyze(root, analyzer_config)?;

    // Generate DSL
    generate_dsl(&project, generator_config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_analyze_and_generate() {
        let dir = tempdir().unwrap();

        // Create a simple Rust project
        let cargo_content = r#"
[package]
name = "my-project"
version = "0.1.0"
description = "A sample project"

[dependencies]
serde = "1.0"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();

        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(
            dir.path().join("src/lib.rs"),
            "pub struct MyStruct;\npub trait MyTrait {}",
        )
        .unwrap();

        let analyzer_config = AnalyzerConfig::default();
        let generator_config = GeneratorConfig::default();

        let workspace = analyze_and_generate(dir.path(), &analyzer_config, &generator_config).unwrap();

        assert_eq!(workspace.name, "my-project");
    }

    #[test]
    fn test_analyze_and_generate_dsl() {
        let dir = tempdir().unwrap();

        let cargo_content = r#"
[package]
name = "dsl-test"
version = "0.1.0"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();

        fs::create_dir(dir.path().join("src")).unwrap();
        fs::write(dir.path().join("src/lib.rs"), "").unwrap();

        let analyzer_config = AnalyzerConfig::default();
        let generator_config = GeneratorConfig::default();

        let dsl = analyze_and_generate_dsl(dir.path(), &analyzer_config, &generator_config).unwrap();

        assert!(dsl.contains("workspace"));
        assert!(dsl.contains("dsl-test"));
    }
}
