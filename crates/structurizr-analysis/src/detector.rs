//! Language detection based on project manifest files.
//!
//! Detects programming languages by looking for characteristic files:
//! - Rust: Cargo.toml
//! - JavaScript/TypeScript: package.json
//! - Python: pyproject.toml, setup.py, requirements.txt
//! - Go: go.mod
//! - Java: pom.xml, build.gradle
//! - Ruby: Gemfile

use crate::error::Result;
use crate::model::Language;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use tracing::{debug, info};

/// Manifest file detection result.
#[derive(Debug, Clone)]
pub struct DetectedProject {
    /// Primary language detected
    pub primary_language: Language,

    /// All languages detected
    pub languages: Vec<Language>,

    /// Project name from manifest
    pub name: Option<String>,

    /// Project description from manifest
    pub description: Option<String>,

    /// Detected manifest files
    pub manifests: Vec<ManifestInfo>,

    /// Whether this is a monorepo/workspace
    pub is_workspace: bool,

    /// Workspace members (for monorepos)
    pub workspace_members: Vec<String>,
}

/// Information about a detected manifest file.
#[derive(Debug, Clone)]
pub struct ManifestInfo {
    /// Path to the manifest file
    pub path: String,

    /// Type of manifest
    pub manifest_type: ManifestType,

    /// Language this manifest is for
    pub language: Language,
}

/// Type of manifest file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestType {
    CargoToml,
    PackageJson,
    PyProjectToml,
    SetupPy,
    RequirementsTxt,
    GoMod,
    PomXml,
    BuildGradle,
    Gemfile,
}

/// Detect the project languages and metadata from a directory.
pub fn detect_project(root: &Path) -> Result<DetectedProject> {
    info!("Detecting project type in: {}", root.display());

    let mut languages = Vec::new();
    let mut manifests = Vec::new();
    let mut name = None;
    let mut description = None;
    let mut is_workspace = false;
    let mut workspace_members = Vec::new();

    // Check for Rust (Cargo.toml)
    let cargo_path = root.join("Cargo.toml");
    if cargo_path.exists() {
        debug!("Found Cargo.toml");
        languages.push(Language::Rust);
        manifests.push(ManifestInfo {
            path: "Cargo.toml".to_string(),
            manifest_type: ManifestType::CargoToml,
            language: Language::Rust,
        });

        if let Ok(info) = parse_cargo_toml(&cargo_path) {
            if name.is_none() {
                name = info.name;
            }
            if description.is_none() {
                description = info.description;
            }
            if info.is_workspace {
                is_workspace = true;
                workspace_members = info.workspace_members;
            }
        }
    }

    // Check for JavaScript/TypeScript (package.json)
    let package_json_path = root.join("package.json");
    if package_json_path.exists() {
        debug!("Found package.json");

        // Check for TypeScript
        let tsconfig = root.join("tsconfig.json");
        if tsconfig.exists() {
            languages.push(Language::TypeScript);
            manifests.push(ManifestInfo {
                path: "package.json".to_string(),
                manifest_type: ManifestType::PackageJson,
                language: Language::TypeScript,
            });
        } else {
            languages.push(Language::JavaScript);
            manifests.push(ManifestInfo {
                path: "package.json".to_string(),
                manifest_type: ManifestType::PackageJson,
                language: Language::JavaScript,
            });
        }

        if let Ok(info) = parse_package_json(&package_json_path) {
            if name.is_none() {
                name = info.name;
            }
            if description.is_none() {
                description = info.description;
            }
            if info.is_workspace {
                is_workspace = true;
                workspace_members.extend(info.workspace_members);
            }
        }
    }

    // Check for Python (pyproject.toml, setup.py, requirements.txt)
    let pyproject_path = root.join("pyproject.toml");
    let setup_py_path = root.join("setup.py");
    let requirements_path = root.join("requirements.txt");

    if pyproject_path.exists() {
        debug!("Found pyproject.toml");
        languages.push(Language::Python);
        manifests.push(ManifestInfo {
            path: "pyproject.toml".to_string(),
            manifest_type: ManifestType::PyProjectToml,
            language: Language::Python,
        });

        if let Ok(info) = parse_pyproject_toml(&pyproject_path) {
            if name.is_none() {
                name = info.name;
            }
            if description.is_none() {
                description = info.description;
            }
        }
    } else if setup_py_path.exists() {
        debug!("Found setup.py");
        languages.push(Language::Python);
        manifests.push(ManifestInfo {
            path: "setup.py".to_string(),
            manifest_type: ManifestType::SetupPy,
            language: Language::Python,
        });
    } else if requirements_path.exists() {
        debug!("Found requirements.txt");
        languages.push(Language::Python);
        manifests.push(ManifestInfo {
            path: "requirements.txt".to_string(),
            manifest_type: ManifestType::RequirementsTxt,
            language: Language::Python,
        });
    }

    // Check for Go (go.mod)
    let go_mod_path = root.join("go.mod");
    if go_mod_path.exists() {
        debug!("Found go.mod");
        languages.push(Language::Go);
        manifests.push(ManifestInfo {
            path: "go.mod".to_string(),
            manifest_type: ManifestType::GoMod,
            language: Language::Go,
        });

        if let Ok(info) = parse_go_mod(&go_mod_path) {
            if name.is_none() {
                name = info.name;
            }
        }
    }

    // Check for Java (pom.xml, build.gradle)
    let pom_path = root.join("pom.xml");
    let gradle_path = root.join("build.gradle");

    if pom_path.exists() {
        debug!("Found pom.xml");
        languages.push(Language::Java);
        manifests.push(ManifestInfo {
            path: "pom.xml".to_string(),
            manifest_type: ManifestType::PomXml,
            language: Language::Java,
        });
    } else if gradle_path.exists() {
        debug!("Found build.gradle");
        languages.push(Language::Java);
        manifests.push(ManifestInfo {
            path: "build.gradle".to_string(),
            manifest_type: ManifestType::BuildGradle,
            language: Language::Java,
        });
    }

    // Check for Ruby (Gemfile)
    let gemfile_path = root.join("Gemfile");
    if gemfile_path.exists() {
        debug!("Found Gemfile");
        languages.push(Language::Ruby);
        manifests.push(ManifestInfo {
            path: "Gemfile".to_string(),
            manifest_type: ManifestType::Gemfile,
            language: Language::Ruby,
        });
    }

    // Determine primary language (first detected or Unknown)
    let primary_language = languages.first().copied().unwrap_or(Language::Unknown);

    // Use directory name if no name was found
    if name.is_none() {
        name = root.file_name().and_then(|n| n.to_str()).map(String::from);
    }

    info!(
        "Detected project: {:?}, languages: {:?}, workspace: {}",
        name, languages, is_workspace
    );

    Ok(DetectedProject {
        primary_language,
        languages,
        name,
        description,
        manifests,
        is_workspace,
        workspace_members,
    })
}

/// Parsed info from a manifest.
struct ManifestMetadata {
    name: Option<String>,
    description: Option<String>,
    is_workspace: bool,
    workspace_members: Vec<String>,
}

/// Parse Cargo.toml for project metadata.
fn parse_cargo_toml(path: &Path) -> Result<ManifestMetadata> {
    /// A value that can be either a string or a workspace reference.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrWorkspace {
        String(String),
        #[allow(dead_code)]
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

    #[derive(Deserialize)]
    struct CargoToml {
        package: Option<CargoPackage>,
        workspace: Option<CargoWorkspace>,
    }

    #[derive(Deserialize)]
    struct CargoPackage {
        name: Option<String>,
        #[serde(default)]
        description: Option<StringOrWorkspace>,
    }

    #[derive(Deserialize)]
    struct CargoWorkspace {
        members: Option<Vec<String>>,
    }

    let content = fs::read_to_string(path)?;
    let cargo: CargoToml = toml::from_str(&content)?;

    let is_workspace = cargo.workspace.is_some();
    let workspace_members = cargo
        .workspace
        .and_then(|w| w.members)
        .unwrap_or_default();

    Ok(ManifestMetadata {
        name: cargo.package.as_ref().and_then(|p| p.name.clone()),
        description: cargo.package.and_then(|p| p.description.and_then(|d| d.as_string().map(String::from))),
        is_workspace,
        workspace_members,
    })
}

/// Parse package.json for project metadata.
fn parse_package_json(path: &Path) -> Result<ManifestMetadata> {
    #[derive(Deserialize)]
    struct PackageJson {
        name: Option<String>,
        description: Option<String>,
        workspaces: Option<WorkspacesField>,
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum WorkspacesField {
        Array(Vec<String>),
        Object { packages: Option<Vec<String>> },
    }

    let content = fs::read_to_string(path)?;
    let package: PackageJson = serde_json::from_str(&content)?;

    let (is_workspace, workspace_members) = match package.workspaces {
        Some(WorkspacesField::Array(members)) => (true, members),
        Some(WorkspacesField::Object { packages }) => (true, packages.unwrap_or_default()),
        None => (false, Vec::new()),
    };

    Ok(ManifestMetadata {
        name: package.name,
        description: package.description,
        is_workspace,
        workspace_members,
    })
}

/// Parse pyproject.toml for project metadata.
fn parse_pyproject_toml(path: &Path) -> Result<ManifestMetadata> {
    #[derive(Deserialize)]
    struct PyProjectToml {
        project: Option<PyProject>,
        tool: Option<PyTool>,
    }

    #[derive(Deserialize)]
    struct PyProject {
        name: Option<String>,
        description: Option<String>,
    }

    #[derive(Deserialize)]
    struct PyTool {
        poetry: Option<PoetryConfig>,
    }

    #[derive(Deserialize)]
    struct PoetryConfig {
        name: Option<String>,
        description: Option<String>,
    }

    let content = fs::read_to_string(path)?;
    let pyproject: PyProjectToml = toml::from_str(&content)?;

    // Try [project] first, then [tool.poetry]
    let (name, description) = if let Some(proj) = pyproject.project {
        (proj.name, proj.description)
    } else if let Some(tool) = pyproject.tool {
        if let Some(poetry) = tool.poetry {
            (poetry.name, poetry.description)
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    Ok(ManifestMetadata {
        name,
        description,
        is_workspace: false,
        workspace_members: Vec::new(),
    })
}

/// Parse go.mod for module name.
fn parse_go_mod(path: &Path) -> Result<ManifestMetadata> {
    let content = fs::read_to_string(path)?;

    // Parse "module github.com/owner/repo" line
    let name = content
        .lines()
        .find(|line| line.starts_with("module "))
        .map(|line| {
            line.strip_prefix("module ")
                .unwrap_or("")
                .trim()
                .to_string()
        });

    Ok(ManifestMetadata {
        name,
        description: None,
        is_workspace: false,
        workspace_members: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_rust_project() {
        let dir = tempdir().unwrap();
        let cargo_content = r#"
[package]
name = "test-project"
description = "A test project"
version = "0.1.0"
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let result = detect_project(dir.path()).unwrap();

        assert_eq!(result.primary_language, Language::Rust);
        assert_eq!(result.name, Some("test-project".to_string()));
        assert_eq!(result.description, Some("A test project".to_string()));
    }

    #[test]
    fn test_detect_rust_workspace() {
        let dir = tempdir().unwrap();
        let cargo_content = r#"
[workspace]
members = ["crates/core", "crates/web"]
"#;
        fs::write(dir.path().join("Cargo.toml"), cargo_content).unwrap();

        let result = detect_project(dir.path()).unwrap();

        assert_eq!(result.primary_language, Language::Rust);
        assert!(result.is_workspace);
        assert_eq!(result.workspace_members.len(), 2);
    }

    #[test]
    fn test_detect_typescript_project() {
        let dir = tempdir().unwrap();
        let package_json = r#"{"name": "my-app", "description": "A TypeScript app"}"#;
        fs::write(dir.path().join("package.json"), package_json).unwrap();
        fs::write(dir.path().join("tsconfig.json"), "{}").unwrap();

        let result = detect_project(dir.path()).unwrap();

        assert_eq!(result.primary_language, Language::TypeScript);
        assert_eq!(result.name, Some("my-app".to_string()));
    }

    #[test]
    fn test_detect_python_project() {
        let dir = tempdir().unwrap();
        let pyproject = r#"
[project]
name = "my-python-app"
description = "A Python application"
"#;
        fs::write(dir.path().join("pyproject.toml"), pyproject).unwrap();

        let result = detect_project(dir.path()).unwrap();

        assert_eq!(result.primary_language, Language::Python);
        assert_eq!(result.name, Some("my-python-app".to_string()));
    }
}
