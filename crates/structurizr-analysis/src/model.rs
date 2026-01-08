//! Intermediate representation for code analysis results.
//!
//! This module defines the data structures that represent the analyzed
//! architecture before it's converted to a C4 model.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Analyzed project structure.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalyzedProject {
    /// Project name (from manifest or directory name)
    pub name: String,

    /// Project description
    pub description: Option<String>,

    /// Root directory path
    pub root_path: PathBuf,

    /// Detected primary language
    pub primary_language: Option<Language>,

    /// All detected languages
    pub languages: Vec<Language>,

    /// Discovered containers (services, libraries, apps)
    pub containers: Vec<AnalyzedContainer>,

    /// External dependencies
    pub dependencies: Vec<ExternalDependency>,

    /// Inferred relationships between containers
    pub relationships: Vec<InferredRelationship>,
}

/// Supported programming languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Rust,
    TypeScript,
    JavaScript,
    Python,
    Go,
    Java,
    CSharp,
    Ruby,
    Unknown,
}

impl Language {
    /// Get the language from a file extension.
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "ts" | "tsx" => Some(Language::TypeScript),
            "js" | "jsx" | "mjs" | "cjs" => Some(Language::JavaScript),
            "py" | "pyi" => Some(Language::Python),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "cs" => Some(Language::CSharp),
            "rb" => Some(Language::Ruby),
            _ => None,
        }
    }

    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::TypeScript => "TypeScript",
            Language::JavaScript => "JavaScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::CSharp => "C#",
            Language::Ruby => "Ruby",
            Language::Unknown => "Unknown",
        }
    }
}

/// An analyzed container (service, library, or application).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedContainer {
    /// Unique identifier (usually the path or package name)
    pub id: String,

    /// Display name
    pub name: String,

    /// Description (from manifest or comments)
    pub description: Option<String>,

    /// Technology stack (e.g., "Rust", "Axum", "PostgreSQL")
    pub technology: String,

    /// Container type
    pub container_type: ContainerType,

    /// Path relative to project root
    pub path: PathBuf,

    /// Components within this container
    pub components: Vec<AnalyzedComponent>,

    /// Direct dependencies (other containers or external)
    pub dependencies: Vec<String>,

    /// Metadata from manifest files
    pub metadata: HashMap<String, String>,
}

/// Type of container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerType {
    /// Web application or service
    WebApp,
    /// API service
    Api,
    /// Background worker or job processor
    Worker,
    /// Library or shared code
    Library,
    /// CLI application
    Cli,
    /// Database
    Database,
    /// Message queue
    MessageQueue,
    /// Unknown/other
    Other,
}

impl ContainerType {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ContainerType::WebApp => "Web Application",
            ContainerType::Api => "API",
            ContainerType::Worker => "Worker",
            ContainerType::Library => "Library",
            ContainerType::Cli => "CLI",
            ContainerType::Database => "Database",
            ContainerType::MessageQueue => "Message Queue",
            ContainerType::Other => "Container",
        }
    }
}

/// An analyzed component within a container.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedComponent {
    /// Unique identifier
    pub id: String,

    /// Display name
    pub name: String,

    /// Description (from doc comments)
    pub description: Option<String>,

    /// Technology (e.g., "Rust Module", "React Component")
    pub technology: String,

    /// Component type
    pub component_type: ComponentType,

    /// Source file path
    pub file_path: PathBuf,

    /// Line number where defined
    pub line_number: Option<usize>,

    /// Public API elements
    pub public_api: Vec<PublicApiElement>,

    /// Dependencies on other components
    pub dependencies: Vec<String>,
}

/// Type of component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    /// Module or namespace
    Module,
    /// Class or struct
    Class,
    /// Service or controller
    Service,
    /// Repository or data access
    Repository,
    /// Handler or controller action
    Handler,
    /// Utility or helper
    Utility,
    /// Configuration
    Config,
    /// Unknown/other
    Other,
}

impl ComponentType {
    /// Get the display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::Module => "Module",
            ComponentType::Class => "Class",
            ComponentType::Service => "Service",
            ComponentType::Repository => "Repository",
            ComponentType::Handler => "Handler",
            ComponentType::Utility => "Utility",
            ComponentType::Config => "Configuration",
            ComponentType::Other => "Component",
        }
    }
}

/// A public API element (function, method, type).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicApiElement {
    /// Element name
    pub name: String,

    /// Element type (function, struct, trait, etc.)
    pub element_type: String,

    /// Documentation
    pub doc: Option<String>,
}

/// An external dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    /// Dependency name
    pub name: String,

    /// Version constraint
    pub version: Option<String>,

    /// Inferred purpose
    pub purpose: Option<String>,

    /// Category
    pub category: DependencyCategory,
}

/// Category of external dependency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyCategory {
    /// Web framework
    WebFramework,
    /// Database driver
    Database,
    /// Serialization
    Serialization,
    /// Logging/tracing
    Logging,
    /// Testing
    Testing,
    /// Build/dev tool
    DevTool,
    /// Authentication/security
    Security,
    /// Async runtime
    AsyncRuntime,
    /// HTTP client
    HttpClient,
    /// Message queue
    MessageQueue,
    /// Other/unknown
    Other,
}

/// An inferred relationship between elements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferredRelationship {
    /// Source element ID
    pub source_id: String,

    /// Destination element ID
    pub destination_id: String,

    /// Relationship description
    pub description: String,

    /// Technology/protocol used
    pub technology: Option<String>,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,

    /// How the relationship was inferred
    pub inference_source: InferenceSource,
}

/// How a relationship was inferred.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InferenceSource {
    /// From import/use statements
    Import,
    /// From function calls
    FunctionCall,
    /// From dependency declarations
    Dependency,
    /// From configuration files
    Config,
    /// From naming conventions
    NamingConvention,
    /// Manual/explicit
    Manual,
}

impl AnalyzedProject {
    /// Create a new empty project.
    pub fn new(name: impl Into<String>, root_path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            root_path: root_path.into(),
            ..Default::default()
        }
    }

    /// Add a container to the project.
    pub fn add_container(&mut self, container: AnalyzedContainer) {
        self.containers.push(container);
    }

    /// Add an external dependency.
    pub fn add_dependency(&mut self, dep: ExternalDependency) {
        self.dependencies.push(dep);
    }

    /// Add a relationship.
    pub fn add_relationship(&mut self, rel: InferredRelationship) {
        self.relationships.push(rel);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(Language::from_extension("rs"), Some(Language::Rust));
        assert_eq!(Language::from_extension("ts"), Some(Language::TypeScript));
        assert_eq!(Language::from_extension("py"), Some(Language::Python));
        assert_eq!(Language::from_extension("unknown"), None);
    }

    #[test]
    fn test_analyzed_project_new() {
        let project = AnalyzedProject::new("test-project", "/path/to/project");
        assert_eq!(project.name, "test-project");
        assert!(project.containers.is_empty());
    }
}
