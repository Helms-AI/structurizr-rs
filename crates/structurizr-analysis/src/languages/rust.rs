//! Rust language analyzer.
//!
//! Analyzes Rust projects by:
//! - Parsing Cargo.toml for workspace structure
//! - Using tree-sitter-rust to parse source files
//! - Extracting modules and public items as components
//! - Inferring relationships from use statements and impl blocks

use crate::analyzer::{AnalyzerConfig, LanguageAnalyzer};
use crate::error::{Error, Result};
use crate::model::{
    AnalyzedComponent, AnalyzedContainer, AnalyzedProject, ComponentType, ContainerType,
    DependencyCategory, ExternalDependency, InferenceSource, InferredRelationship, Language,
};
use glob::glob;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use tracing::{info, warn};

/// Results from analyzing a single Rust source file.
#[derive(Debug, Default)]
struct FileAnalysisResult {
    /// Extracted components (structs, enums, traits, modules)
    components: Vec<AnalyzedComponent>,
    /// Import relationships from use statements
    imports: Vec<ImportInfo>,
    /// Impl block information for trait relationships
    impl_blocks: Vec<ImplInfo>,
}

/// Information about a use/import statement.
#[derive(Debug, Clone)]
struct ImportInfo {
    /// The module path from the file containing this import
    source_module: String,
    /// The imported path (e.g., "crate::services::UserService")
    imported_path: String,
    /// Whether this is a crate-local import
    is_local: bool,
}

/// Information about an impl block.
#[derive(Debug, Clone)]
struct ImplInfo {
    /// The type being implemented (e.g., "UserService")
    target_type: String,
    /// The trait being implemented, if any (e.g., "Repository")
    trait_name: Option<String>,
    /// The module path where this impl is defined
    module_path: String,
}

/// Result from analyzing a crate including component relationships.
#[derive(Debug)]
struct CrateAnalysisResult {
    container: AnalyzedContainer,
    /// Component-level relationships inferred from imports and impl blocks
    component_relationships: Vec<InferredRelationship>,
}

/// Results from analyzing a source directory.
#[derive(Debug, Default)]
struct DirectoryAnalysisResult {
    components: Vec<AnalyzedComponent>,
    imports: Vec<ImportInfo>,
    impl_blocks: Vec<ImplInfo>,
}

/// Rust language analyzer.
pub struct RustAnalyzer {
    /// Tree-sitter parser
    parser: tree_sitter::Parser,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer.
    pub fn new() -> Self {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .expect("Failed to load Rust grammar");

        Self { parser }
    }

    /// Parse a Cargo.toml file.
    fn parse_cargo_toml(&self, path: &Path) -> Result<CargoManifest> {
        let content = fs::read_to_string(path)?;
        let manifest: CargoManifest = toml::from_str(&content)?;
        Ok(manifest)
    }

    /// Analyze a single crate/package.
    fn analyze_crate(
        &mut self,
        crate_path: &Path,
        config: &AnalyzerConfig,
    ) -> Result<CrateAnalysisResult> {
        let cargo_path = crate_path.join("Cargo.toml");
        if !cargo_path.exists() {
            return Err(Error::Analysis(format!(
                "No Cargo.toml found at: {}",
                crate_path.display()
            )));
        }

        let manifest = self.parse_cargo_toml(&cargo_path)?;
        let package = manifest.package.clone().ok_or_else(|| {
            Error::Analysis(format!(
                "No [package] section in: {}",
                cargo_path.display()
            ))
        })?;

        let name = package.name.clone();
        let description = package.get_description();

        // Determine container type from crate structure
        let container_type = self.infer_container_type(crate_path, &manifest);

        // Build technology string
        let mut technologies = vec!["Rust".to_string()];
        if let Some(deps) = &manifest.dependencies {
            // Check for common frameworks
            if deps.contains_key("axum") || deps.contains_key("actix-web") || deps.contains_key("rocket") {
                technologies.push("Web Framework".to_string());
            }
            if deps.contains_key("sqlx") || deps.contains_key("diesel") || deps.contains_key("rusqlite") {
                technologies.push("Database".to_string());
            }
            if deps.contains_key("tokio") || deps.contains_key("async-std") {
                technologies.push("Async".to_string());
            }
        }

        // Analyze source files for components and relationships
        let src_path = crate_path.join("src");
        let (components, component_relationships) = if src_path.exists() {
            let dir_result = self.analyze_source_directory(&src_path, &name, config)?;
            let relationships = self.infer_component_relationships(
                &name,
                &dir_result.components,
                &dir_result.imports,
                &dir_result.impl_blocks,
            );
            (dir_result.components, relationships)
        } else {
            (Vec::new(), Vec::new())
        };

        // Extract dependencies
        let dependencies = self.extract_dependencies(&manifest);

        // Collect dependency IDs for the container
        let dep_ids: Vec<String> = dependencies.iter().map(|d| d.name.clone()).collect();

        let container = AnalyzedContainer {
            id: name.clone(),
            name: name.clone(),
            description,
            technology: technologies.join(", "),
            container_type,
            path: crate_path.to_path_buf(),
            components,
            dependencies: dep_ids,
            metadata: HashMap::new(),
        };

        Ok(CrateAnalysisResult {
            container,
            component_relationships,
        })
    }

    /// Infer relationships between components from imports and impl blocks.
    fn infer_component_relationships(
        &self,
        container_id: &str,
        components: &[AnalyzedComponent],
        imports: &[ImportInfo],
        impl_blocks: &[ImplInfo],
    ) -> Vec<InferredRelationship> {
        let mut relationships = Vec::new();

        // Build a set of component IDs for matching
        let component_ids: HashSet<&str> = components.iter().map(|c| c.id.as_str()).collect();

        // Process imports to create relationships
        for import in imports {
            // Extract the target component from the import path
            // e.g., "crate::services::UserService" -> "services::UserService"
            let target = import.imported_path
                .trim_start_matches("crate::")
                .trim_start_matches("self::")
                .trim_start_matches("super::");

            // Try to find a matching component
            let target_component = self.find_matching_component(target, components);

            if let Some(dest_id) = target_component {
                // Create a relationship from the source module to the imported component
                let source_id = if import.source_module.is_empty() {
                    container_id.to_string()
                } else {
                    import.source_module.clone()
                };

                // Only add if source and destination are different
                if source_id != dest_id {
                    relationships.push(InferredRelationship {
                        source_id,
                        destination_id: dest_id,
                        description: "Uses".to_string(),
                        technology: Some("Rust import".to_string()),
                        confidence: 0.95,
                        inference_source: InferenceSource::Import,
                    });
                }
            }
        }

        // Process impl blocks to create trait implementation relationships
        for impl_info in impl_blocks {
            if let Some(trait_name) = &impl_info.trait_name {
                // Find the target type component
                let target_component = self.find_matching_component(&impl_info.target_type, components);
                // Find the trait component
                let trait_component = self.find_matching_component(trait_name, components);

                if let (Some(target_id), Some(trait_id)) = (target_component, trait_component) {
                    if target_id != trait_id {
                        relationships.push(InferredRelationship {
                            source_id: target_id,
                            destination_id: trait_id,
                            description: "Implements".to_string(),
                            technology: Some("Rust trait impl".to_string()),
                            confidence: 0.90,
                            inference_source: InferenceSource::Import,
                        });
                    }
                }
            }
        }

        // Deduplicate relationships
        let mut seen: HashSet<(String, String)> = HashSet::new();
        relationships.retain(|r| {
            let key = (r.source_id.clone(), r.destination_id.clone());
            if seen.contains(&key) {
                false
            } else {
                seen.insert(key);
                true
            }
        });

        relationships
    }

    /// Find a component that matches the given import path.
    fn find_matching_component(&self, import_path: &str, components: &[AnalyzedComponent]) -> Option<String> {
        // First try exact match
        for comp in components {
            if comp.id == import_path || comp.name == import_path {
                return Some(comp.id.clone());
            }
        }

        // Try matching just the last segment (type name)
        let type_name = import_path.rsplit("::").next()?;
        for comp in components {
            if comp.name == type_name {
                return Some(comp.id.clone());
            }
        }

        // Try partial path match
        for comp in components {
            if comp.id.ends_with(import_path) || import_path.ends_with(&comp.id) {
                return Some(comp.id.clone());
            }
        }

        None
    }

    /// Infer the container type from crate structure.
    fn infer_container_type(&self, crate_path: &Path, manifest: &CargoManifest) -> ContainerType {
        // Check for binary target
        if crate_path.join("src/main.rs").exists() {
            // Check dependencies for framework hints
            if let Some(deps) = &manifest.dependencies {
                if deps.contains_key("axum")
                    || deps.contains_key("actix-web")
                    || deps.contains_key("rocket")
                    || deps.contains_key("warp")
                {
                    return ContainerType::Api;
                }
                if deps.contains_key("clap") || deps.contains_key("structopt") {
                    return ContainerType::Cli;
                }
            }
            return ContainerType::WebApp;
        }

        // Library crate
        if crate_path.join("src/lib.rs").exists() {
            return ContainerType::Library;
        }

        ContainerType::Other
    }

    /// Analyze a source directory for components and relationships.
    fn analyze_source_directory(
        &mut self,
        src_path: &Path,
        container_name: &str,
        config: &AnalyzerConfig,
    ) -> Result<DirectoryAnalysisResult> {
        let mut result = DirectoryAnalysisResult {
            components: Vec::new(),
            imports: Vec::new(),
            impl_blocks: Vec::new(),
        };

        // Find all Rust source files
        let pattern = src_path.join("**/*.rs");
        let pattern_str = pattern.to_string_lossy();

        for entry in glob(&pattern_str).map_err(|e| Error::Analysis(e.to_string()))? {
            let path = entry.map_err(|e| Error::Analysis(e.to_string()))?;

            // Skip test files unless configured to include
            if !config.include_tests {
                let path_str = path.to_string_lossy();
                if path_str.contains("/tests/")
                    || path_str.contains("_test.rs")
                    || path_str.ends_with("tests.rs")
                {
                    continue;
                }
            }

            // Check exclusion patterns
            let path_str = path.to_string_lossy();
            let mut excluded = false;
            for pattern in &config.exclude_patterns {
                if glob::Pattern::new(pattern)
                    .map(|p| p.matches(&path_str))
                    .unwrap_or(false)
                {
                    excluded = true;
                    break;
                }
            }
            if excluded {
                continue;
            }

            // Analyze the file
            if let Ok(file_result) = self.analyze_rust_file(&path, container_name, config) {
                result.components.extend(file_result.components);
                result.imports.extend(file_result.imports);
                result.impl_blocks.extend(file_result.impl_blocks);
            }
        }

        Ok(result)
    }

    /// Analyze a single Rust source file.
    fn analyze_rust_file(
        &mut self,
        path: &Path,
        _container_name: &str,
        config: &AnalyzerConfig,
    ) -> Result<FileAnalysisResult> {
        let content = fs::read_to_string(path)?;
        let tree = self
            .parser
            .parse(&content, None)
            .ok_or_else(|| Error::TreeSitter("Failed to parse file".to_string()))?;

        let mut result = FileAnalysisResult::default();
        let root = tree.root_node();

        // Extract module path from file path
        let module_path = self.path_to_module(path);

        // Walk the AST
        let mut cursor = root.walk();
        for child in root.children(&mut cursor) {
            match child.kind() {
                // Use declarations (imports) - for relationship detection
                "use_declaration" => {
                    if let Some(import) = self.analyze_use_declaration(&child, &content, &module_path) {
                        result.imports.push(import);
                    }
                }
                // Public module declarations
                "mod_item" => {
                    if let Some(component) = self.analyze_mod_item(&child, &content, path, &module_path, config) {
                        result.components.push(component);
                    }
                }
                // Public struct definitions
                "struct_item" => {
                    if let Some(component) = self.analyze_struct_item(&child, &content, path, &module_path, config) {
                        result.components.push(component);
                    }
                }
                // Public enum definitions
                "enum_item" => {
                    if let Some(component) = self.analyze_enum_item(&child, &content, path, &module_path, config) {
                        result.components.push(component);
                    }
                }
                // Public trait definitions
                "trait_item" => {
                    if let Some(component) = self.analyze_trait_item(&child, &content, path, &module_path, config) {
                        result.components.push(component);
                    }
                }
                // Impl blocks (for understanding trait relationships)
                "impl_item" => {
                    if let Some(impl_info) = self.analyze_impl_item(&child, &content, &module_path) {
                        result.impl_blocks.push(impl_info);
                    }
                }
                _ => {}
            }
        }

        Ok(result)
    }

    /// Analyze a use declaration (import statement).
    fn analyze_use_declaration(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        module_path: &str,
    ) -> Option<ImportInfo> {
        // Find the use_clause child which contains the path
        let mut use_clause = None;
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "use_clause" || child.kind() == "scoped_use_list" || child.kind() == "use_as_clause" {
                    use_clause = Some(child);
                    break;
                }
            }
        }

        let clause = use_clause?;
        let path_text = content[clause.byte_range()].to_string();

        // Determine if this is a local import
        let is_local = path_text.starts_with("crate::")
            || path_text.starts_with("self::")
            || path_text.starts_with("super::");

        // Only track crate-local imports for component relationships
        if is_local {
            Some(ImportInfo {
                source_module: module_path.to_string(),
                imported_path: path_text,
                is_local,
            })
        } else {
            None
        }
    }

    /// Analyze an impl block to extract trait implementation relationships.
    fn analyze_impl_item(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        module_path: &str,
    ) -> Option<ImplInfo> {
        let mut target_type = None;
        let mut trait_name = None;

        // Walk children to find type and trait
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                match child.kind() {
                    // The type being implemented
                    "type_identifier" | "generic_type" => {
                        if target_type.is_none() && trait_name.is_none() {
                            // First type identifier could be the trait (if "for" follows)
                            // Or the target type (if no "for")
                            target_type = Some(content[child.byte_range()].to_string());
                        }
                    }
                    // "for" keyword indicates trait impl
                    "for" => {
                        // The previous type was the trait, next is target
                        if let Some(prev_type) = target_type.take() {
                            trait_name = Some(prev_type);
                        }
                    }
                    _ => {}
                }

                // After "for", look for the target type
                if trait_name.is_some() && target_type.is_none() {
                    if child.kind() == "type_identifier" || child.kind() == "generic_type" {
                        target_type = Some(content[child.byte_range()].to_string());
                    }
                }
            }
        }

        // If we have a target type, create the impl info
        target_type.map(|target| ImplInfo {
            target_type: target,
            trait_name,
            module_path: module_path.to_string(),
        })
    }

    /// Convert file path to module path.
    fn path_to_module(&self, path: &Path) -> String {
        let path_str = path.to_string_lossy();

        // Remove src/ prefix and .rs suffix
        let mut module_path = path_str
            .trim_start_matches("src/")
            .trim_end_matches(".rs")
            .trim_end_matches("/mod")
            .replace('/', "::");

        // Handle lib.rs and main.rs
        if module_path == "lib" || module_path == "main" {
            module_path = String::new();
        }

        module_path
    }

    /// Check if a node has a public visibility modifier.
    fn is_public(&self, node: &tree_sitter::Node, content: &str) -> bool {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "visibility_modifier" {
                    let text = &content[child.byte_range()];
                    return text.starts_with("pub");
                }
            }
        }
        false
    }

    /// Get the name from an identifier child.
    fn get_name(&self, node: &tree_sitter::Node, content: &str) -> Option<String> {
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                if child.kind() == "identifier" || child.kind() == "type_identifier" {
                    return Some(content[child.byte_range()].to_string());
                }
            }
        }
        None
    }

    /// Extract doc comment from preceding nodes.
    fn extract_doc_comment(&self, node: &tree_sitter::Node, content: &str) -> Option<String> {
        let mut doc_lines = Vec::new();

        // Walk preceding siblings to find doc comments
        let mut current = node.prev_sibling();
        while let Some(sibling) = current {
            match sibling.kind() {
                "line_comment" => {
                    let text = &content[sibling.byte_range()];
                    // Check for doc comment (/// or //!)
                    if text.starts_with("///") {
                        // Extract the doc content (skip "/// " or "///")
                        let doc = text.trim_start_matches("///").trim_start();
                        doc_lines.push(doc.to_string());
                    } else {
                        // Regular comment, stop collecting
                        break;
                    }
                }
                "block_comment" => {
                    let text = &content[sibling.byte_range()];
                    // Check for doc comment (/** or /*!)
                    if text.starts_with("/**") || text.starts_with("/*!") {
                        // Extract the doc content
                        let doc = text
                            .trim_start_matches("/**")
                            .trim_start_matches("/*!")
                            .trim_end_matches("*/")
                            .trim();
                        doc_lines.push(doc.to_string());
                    }
                    break;
                }
                // Skip whitespace and attributes, continue looking
                "attribute_item" | "inner_attribute_item" => {
                    current = sibling.prev_sibling();
                    continue;
                }
                _ => {
                    // Any other node type means we've passed the doc comments
                    break;
                }
            }
            current = sibling.prev_sibling();
        }

        // Reverse because we collected in reverse order
        doc_lines.reverse();

        if doc_lines.is_empty() {
            None
        } else {
            // Join multiple doc lines with spaces, taking first sentence/paragraph
            let full_doc = doc_lines.join(" ");
            // Return first sentence or first 200 chars
            let trimmed = if let Some(pos) = full_doc.find(". ") {
                &full_doc[..pos + 1]
            } else if full_doc.len() > 200 {
                &full_doc[..200]
            } else {
                &full_doc
            };
            Some(trimmed.trim().to_string())
        }
    }

    /// Analyze a module declaration.
    fn analyze_mod_item(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        module_path: &str,
        _config: &AnalyzerConfig,
    ) -> Option<AnalyzedComponent> {
        if !self.is_public(node, content) {
            return None;
        }

        let name = self.get_name(node, content)?;
        let full_path = if module_path.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", module_path, name)
        };

        Some(AnalyzedComponent {
            id: full_path.clone(),
            name: name.clone(),
            description: self.extract_doc_comment(node, content),
            technology: "Rust Module".to_string(),
            component_type: ComponentType::Module,
            file_path: path.to_path_buf(),
            line_number: Some(node.start_position().row + 1),
            public_api: Vec::new(),
            dependencies: Vec::new(),
        })
    }

    /// Analyze a struct definition.
    fn analyze_struct_item(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        module_path: &str,
        _config: &AnalyzerConfig,
    ) -> Option<AnalyzedComponent> {
        if !self.is_public(node, content) {
            return None;
        }

        let name = self.get_name(node, content)?;
        let full_path = if module_path.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", module_path, name)
        };

        // Infer component type from name
        let component_type = self.infer_component_type_from_name(&name);

        Some(AnalyzedComponent {
            id: full_path,
            name: name.clone(),
            description: self.extract_doc_comment(node, content),
            technology: "Rust Struct".to_string(),
            component_type,
            file_path: path.to_path_buf(),
            line_number: Some(node.start_position().row + 1),
            public_api: Vec::new(),
            dependencies: Vec::new(),
        })
    }

    /// Analyze an enum definition.
    fn analyze_enum_item(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        module_path: &str,
        _config: &AnalyzerConfig,
    ) -> Option<AnalyzedComponent> {
        if !self.is_public(node, content) {
            return None;
        }

        let name = self.get_name(node, content)?;
        let full_path = if module_path.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", module_path, name)
        };

        Some(AnalyzedComponent {
            id: full_path,
            name: name.clone(),
            description: self.extract_doc_comment(node, content),
            technology: "Rust Enum".to_string(),
            component_type: ComponentType::Class,
            file_path: path.to_path_buf(),
            line_number: Some(node.start_position().row + 1),
            public_api: Vec::new(),
            dependencies: Vec::new(),
        })
    }

    /// Analyze a trait definition.
    fn analyze_trait_item(
        &self,
        node: &tree_sitter::Node,
        content: &str,
        path: &Path,
        module_path: &str,
        _config: &AnalyzerConfig,
    ) -> Option<AnalyzedComponent> {
        if !self.is_public(node, content) {
            return None;
        }

        let name = self.get_name(node, content)?;
        let full_path = if module_path.is_empty() {
            name.clone()
        } else {
            format!("{}::{}", module_path, name)
        };

        Some(AnalyzedComponent {
            id: full_path,
            name: name.clone(),
            description: self.extract_doc_comment(node, content),
            technology: "Rust Trait".to_string(),
            component_type: ComponentType::Service,
            file_path: path.to_path_buf(),
            line_number: Some(node.start_position().row + 1),
            public_api: Vec::new(),
            dependencies: Vec::new(),
        })
    }

    /// Infer component type from struct/type name.
    fn infer_component_type_from_name(&self, name: &str) -> ComponentType {
        let lower = name.to_lowercase();

        if lower.contains("handler") || lower.contains("controller") {
            ComponentType::Handler
        } else if lower.contains("service") {
            ComponentType::Service
        } else if lower.contains("repository") || lower.contains("store") || lower.contains("dao") {
            ComponentType::Repository
        } else if lower.contains("config") || lower.contains("settings") {
            ComponentType::Config
        } else if lower.contains("util") || lower.contains("helper") {
            ComponentType::Utility
        } else {
            ComponentType::Class
        }
    }

    /// Extract external dependencies from Cargo.toml.
    fn extract_dependencies(&self, manifest: &CargoManifest) -> Vec<ExternalDependency> {
        let mut deps = Vec::new();

        if let Some(dependencies) = &manifest.dependencies {
            for (name, value) in dependencies {
                let version = match value {
                    DependencySpec::Simple(v) => Some(v.clone()),
                    DependencySpec::Detailed(d) => d.version.clone(),
                };

                let category = self.categorize_dependency(name);

                deps.push(ExternalDependency {
                    name: name.clone(),
                    version,
                    purpose: Some(self.describe_dependency(name)),
                    category,
                });
            }
        }

        deps
    }

    /// Categorize a dependency by name.
    fn categorize_dependency(&self, name: &str) -> DependencyCategory {
        match name {
            "axum" | "actix-web" | "rocket" | "warp" | "hyper" | "tower" => {
                DependencyCategory::WebFramework
            }
            "sqlx" | "diesel" | "rusqlite" | "postgres" | "mysql" | "mongodb" => {
                DependencyCategory::Database
            }
            "serde" | "serde_json" | "serde_yaml" | "bincode" | "rmp-serde" => {
                DependencyCategory::Serialization
            }
            "tracing" | "log" | "env_logger" | "tracing-subscriber" => {
                DependencyCategory::Logging
            }
            "tokio" | "async-std" | "smol" | "futures" => DependencyCategory::AsyncRuntime,
            "reqwest" | "ureq" | "surf" => DependencyCategory::HttpClient,
            "tokio-test" | "proptest" | "quickcheck" | "criterion" => DependencyCategory::Testing,
            "jsonwebtoken" | "argon2" | "bcrypt" | "oauth2" => DependencyCategory::Security,
            "lapin" | "rdkafka" | "redis" | "amqp" => DependencyCategory::MessageQueue,
            _ => DependencyCategory::Other,
        }
    }

    /// Get a description for a known dependency.
    fn describe_dependency(&self, name: &str) -> String {
        match name {
            "axum" => "Ergonomic and modular web framework".to_string(),
            "tokio" => "Async runtime for Rust".to_string(),
            "serde" => "Serialization/deserialization framework".to_string(),
            "serde_json" => "JSON serialization".to_string(),
            "sqlx" => "Async SQL toolkit".to_string(),
            "tracing" => "Application-level tracing".to_string(),
            "clap" => "Command-line argument parser".to_string(),
            "reqwest" => "HTTP client".to_string(),
            "thiserror" => "Error type derivation".to_string(),
            "anyhow" => "Flexible error handling".to_string(),
            _ => format!("{} dependency", name),
        }
    }
}

impl Default for RustAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl LanguageAnalyzer for RustAnalyzer {
    fn language(&self) -> Language {
        Language::Rust
    }

    fn can_analyze(&self, path: &Path) -> bool {
        path.join("Cargo.toml").exists()
    }

    fn analyze(&self, root: &Path, config: &AnalyzerConfig) -> Result<AnalyzedProject> {
        info!("Analyzing Rust project at: {}", root.display());

        let cargo_path = root.join("Cargo.toml");
        if !cargo_path.exists() {
            return Err(Error::Analysis("No Cargo.toml found".to_string()));
        }

        let manifest = self.parse_cargo_toml(&cargo_path)?;

        // Create a mutable analyzer for parsing
        let mut analyzer = RustAnalyzer::new();

        // Debug logging for Cargo.toml analysis
        info!("Cargo.toml analysis:");
        info!("  - Has [workspace]: {}", manifest.workspace.is_some());
        info!("  - Has [package]: {}", manifest.package.is_some());
        if let Some(ws) = &manifest.workspace {
            info!("  - Workspace members: {:?}", ws.members);
        }

        // Check if this is a workspace with actual members
        let has_workspace_members = manifest
            .workspace
            .as_ref()
            .and_then(|ws| ws.members.as_ref())
            .map_or(false, |m| !m.is_empty());

        // Collect component-level relationships from all crates
        let mut component_relationships: Vec<InferredRelationship> = Vec::new();

        let mut project = if has_workspace_members {
            // True workspace with members - analyze each member
            let workspace = manifest.workspace.as_ref().unwrap();
            let mut project = AnalyzedProject::new(
                root.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("workspace"),
                root,
            );
            project.primary_language = Some(Language::Rust);
            project.languages = vec![Language::Rust];

            for member_pattern in workspace.members.as_ref().unwrap() {
                // Expand glob patterns
                let pattern = root.join(member_pattern);
                let pattern_str = pattern.to_string_lossy();

                for entry in glob(&pattern_str).map_err(|e| Error::Analysis(e.to_string()))? {
                    let member_path = entry.map_err(|e| Error::Analysis(e.to_string()))?;
                    if member_path.join("Cargo.toml").exists() {
                        match analyzer.analyze_crate(&member_path, config) {
                            Ok(result) => {
                                project.add_container(result.container);
                                component_relationships.extend(result.component_relationships);
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to analyze workspace member {}: {}",
                                    member_path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }

            project
        } else if manifest.package.is_some() {
            // Single crate (either no [workspace] or empty [workspace] with [package])
            // This handles cases like tensorflow/rust where [workspace] exists but is empty
            if manifest.workspace.is_some() {
                info!("Empty [workspace] section with [package] - treating as single crate");
            }
            let result = analyzer.analyze_crate(root, config)?;
            let name = result.container.name.clone();

            let mut project = AnalyzedProject::new(&name, root);
            project.primary_language = Some(Language::Rust);
            project.languages = vec![Language::Rust];
            project.description = result.container.description.clone();
            project.add_container(result.container);
            component_relationships.extend(result.component_relationships);

            project
        } else {
            // Virtual workspace with no members and no package - return empty project
            warn!("Empty workspace with no members and no [package] section");
            let mut project = AnalyzedProject::new(
                root.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("workspace"),
                root,
            );
            project.primary_language = Some(Language::Rust);
            project.languages = vec![Language::Rust];
            project
        };

        // Infer relationships between containers
        if config.infer_relationships {
            let mut all_relationships = self.infer_relationships(&project);
            // Add component-level relationships
            all_relationships.extend(component_relationships);
            project.relationships = all_relationships;
        }

        // Extract all unique external dependencies
        let mut all_deps: HashMap<String, ExternalDependency> = HashMap::new();
        for container in &project.containers {
            if let Ok(manifest) = self.parse_cargo_toml(&container.path.join("Cargo.toml")) {
                for dep in self.extract_dependencies(&manifest) {
                    all_deps.entry(dep.name.clone()).or_insert(dep);
                }
            }
        }
        project.dependencies = all_deps.into_values().collect();

        // Count container vs component relationships
        let container_rels = project.relationships.iter()
            .filter(|r| r.inference_source == InferenceSource::Dependency)
            .count();
        let component_rels = project.relationships.len() - container_rels;

        info!(
            "Analysis complete: {} containers, {} components, {} relationships ({} container, {} component)",
            project.containers.len(),
            project.containers.iter().map(|c| c.components.len()).sum::<usize>(),
            project.relationships.len(),
            container_rels,
            component_rels
        );

        Ok(project)
    }
}

impl RustAnalyzer {
    /// Infer relationships between containers based on dependencies.
    fn infer_relationships(&self, project: &AnalyzedProject) -> Vec<InferredRelationship> {
        let mut relationships = Vec::new();

        // Build a set of container IDs
        let container_ids: HashSet<_> = project.containers.iter().map(|c| c.id.clone()).collect();

        // Check each container's dependencies
        for container in &project.containers {
            for dep in &container.dependencies {
                // Check if this dependency is another container
                if container_ids.contains(dep) {
                    relationships.push(InferredRelationship {
                        source_id: container.id.clone(),
                        destination_id: dep.clone(),
                        description: "Uses".to_string(),
                        technology: Some("Rust crate dependency".to_string()),
                        confidence: 0.95,
                        inference_source: InferenceSource::Dependency,
                    });
                }
            }
        }

        relationships
    }
}

/// Cargo.toml manifest structure.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct CargoManifest {
    package: Option<CargoPackage>,
    workspace: Option<CargoWorkspace>,
    dependencies: Option<HashMap<String, DependencySpec>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, DependencySpec>>,
}

/// A value that can be either a string or a workspace reference.
/// Handles both `version = "0.1.0"` and `version.workspace = true`.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)] // Fields used by serde for deserialization matching
enum StringOrWorkspace {
    String(String),
    Workspace { workspace: bool },
}

impl StringOrWorkspace {
    fn as_string(&self) -> Option<&str> {
        match self {
            StringOrWorkspace::String(s) => Some(s),
            StringOrWorkspace::Workspace { .. } => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
struct CargoPackage {
    name: String,
    #[serde(default)]
    version: Option<StringOrWorkspace>,
    #[serde(default)]
    description: Option<StringOrWorkspace>,
    #[serde(default)]
    authors: Option<AuthorsOrWorkspace>,
}

/// Authors can be either a list of strings or a workspace reference.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[allow(dead_code)] // Fields used by serde for deserialization matching
enum AuthorsOrWorkspace {
    Authors(Vec<String>),
    Workspace { workspace: bool },
}

impl CargoPackage {
    fn get_description(&self) -> Option<String> {
        self.description.as_ref().and_then(|d| d.as_string().map(String::from))
    }
}

#[derive(Debug, Deserialize)]
struct CargoWorkspace {
    members: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencySpec {
    Simple(String),
    Detailed(DetailedDependency),
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct DetailedDependency {
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    features: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_rust_analyzer_new() {
        let analyzer = RustAnalyzer::new();
        assert_eq!(analyzer.language(), Language::Rust);
    }

    #[test]
    fn test_can_analyze() {
        let dir = tempdir().unwrap();
        let analyzer = RustAnalyzer::new();

        // Without Cargo.toml
        assert!(!analyzer.can_analyze(dir.path()));

        // With Cargo.toml
        fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
        assert!(analyzer.can_analyze(dir.path()));
    }

    #[test]
    fn test_analyze_simple_crate() {
        let dir = tempdir().unwrap();

        // Create Cargo.toml
        let cargo_content = r#"
[package]
name = "test-crate"
version = "0.1.0"
description = "A test crate"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();

        // Create src/lib.rs
        fs::create_dir(dir.path().join("src")).unwrap();
        let lib_content = r#"
pub mod handlers;

pub struct MyService {
    name: String,
}

pub trait Repository {
    fn find_all(&self);
}
"#;
        fs::write(dir.path().join("src/lib.rs"), lib_content).unwrap();

        let analyzer = RustAnalyzer::new();
        let config = AnalyzerConfig::default();
        let result = analyzer.analyze(dir.path(), &config).unwrap();

        assert_eq!(result.name, "test-crate");
        assert_eq!(result.primary_language, Some(Language::Rust));
        assert_eq!(result.containers.len(), 1);

        let container = &result.containers[0];
        assert_eq!(container.name, "test-crate");
        assert!(container.components.len() >= 2); // At least MyService and Repository
    }

    #[test]
    fn test_infer_component_type() {
        let analyzer = RustAnalyzer::new();

        assert_eq!(
            analyzer.infer_component_type_from_name("UserHandler"),
            ComponentType::Handler
        );
        assert_eq!(
            analyzer.infer_component_type_from_name("AuthService"),
            ComponentType::Service
        );
        assert_eq!(
            analyzer.infer_component_type_from_name("UserRepository"),
            ComponentType::Repository
        );
        assert_eq!(
            analyzer.infer_component_type_from_name("AppConfig"),
            ComponentType::Config
        );
    }

    #[test]
    fn test_categorize_dependency() {
        let analyzer = RustAnalyzer::new();

        assert_eq!(
            analyzer.categorize_dependency("axum"),
            DependencyCategory::WebFramework
        );
        assert_eq!(
            analyzer.categorize_dependency("sqlx"),
            DependencyCategory::Database
        );
        assert_eq!(
            analyzer.categorize_dependency("tokio"),
            DependencyCategory::AsyncRuntime
        );
    }
}
