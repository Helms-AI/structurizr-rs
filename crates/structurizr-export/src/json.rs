//! JSON export for Structurizr workspaces.

use structurizr_core::Workspace;
use crate::error::Result;

/// Exports workspaces to Structurizr JSON format.
pub struct JsonExporter;

impl JsonExporter {
    /// Export a workspace to JSON string.
    pub fn export(workspace: &Workspace) -> Result<String> {
        let json = serde_json::to_string_pretty(workspace)?;
        Ok(json)
    }

    /// Export a workspace to minified JSON string.
    pub fn export_minified(workspace: &Workspace) -> Result<String> {
        let json = serde_json::to_string(workspace)?;
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_workspace() {
        let workspace = Workspace::new("Test", "A test workspace");
        let json = JsonExporter::export(&workspace).unwrap();

        assert!(json.contains("\"name\": \"Test\""));
    }
}
