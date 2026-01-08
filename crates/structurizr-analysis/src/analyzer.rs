//! Core analyzer trait and configuration.

use crate::error::Result;
use crate::model::{AnalyzedProject, Language};
use std::path::Path;

/// Configuration for the analyzer.
#[derive(Debug, Clone)]
pub struct AnalyzerConfig {
    /// Maximum depth to recurse into directories.
    pub max_depth: usize,

    /// Include test files in analysis.
    pub include_tests: bool,

    /// Glob patterns for files to include.
    pub include_patterns: Vec<String>,

    /// Glob patterns for files to exclude.
    pub exclude_patterns: Vec<String>,

    /// Whether to extract doc comments.
    pub extract_docs: bool,

    /// Whether to infer relationships.
    pub infer_relationships: bool,

    /// Minimum confidence for inferred relationships.
    pub min_relationship_confidence: f32,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            max_depth: 10,
            include_tests: false,
            include_patterns: vec![],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/vendor/**".to_string(),
                "**/.git/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/__pycache__/**".to_string(),
                "**/.venv/**".to_string(),
            ],
            extract_docs: true,
            infer_relationships: true,
            min_relationship_confidence: 0.5,
        }
    }
}

/// Trait for language-specific analyzers.
pub trait LanguageAnalyzer: Send + Sync {
    /// Get the language this analyzer handles.
    fn language(&self) -> Language;

    /// Check if this analyzer can handle the given file.
    fn can_analyze(&self, path: &Path) -> bool;

    /// Analyze a project directory.
    fn analyze(&self, root: &Path, config: &AnalyzerConfig) -> Result<AnalyzedProject>;
}

/// Registry of language analyzers.
pub struct AnalyzerRegistry {
    analyzers: Vec<Box<dyn LanguageAnalyzer>>,
}

impl AnalyzerRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            analyzers: Vec::new(),
        }
    }

    /// Create a registry with all default analyzers.
    pub fn with_defaults() -> Self {
        let mut registry = Self::new();

        // Register the Rust analyzer
        registry.register(Box::new(crate::languages::rust::RustAnalyzer::new()));

        registry
    }

    /// Register an analyzer.
    pub fn register(&mut self, analyzer: Box<dyn LanguageAnalyzer>) {
        self.analyzers.push(analyzer);
    }

    /// Get an analyzer for a specific language.
    pub fn get(&self, language: Language) -> Option<&dyn LanguageAnalyzer> {
        self.analyzers
            .iter()
            .find(|a| a.language() == language)
            .map(|a| a.as_ref())
    }

    /// List all registered analyzers.
    pub fn list(&self) -> impl Iterator<Item = &dyn LanguageAnalyzer> {
        self.analyzers.iter().map(|a| a.as_ref())
    }
}

impl Default for AnalyzerRegistry {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyzerConfig::default();
        assert_eq!(config.max_depth, 10);
        assert!(!config.include_tests);
        assert!(!config.exclude_patterns.is_empty());
    }

    #[test]
    fn test_analyzer_registry() {
        let registry = AnalyzerRegistry::with_defaults();
        assert!(registry.get(Language::Rust).is_some());
    }
}
