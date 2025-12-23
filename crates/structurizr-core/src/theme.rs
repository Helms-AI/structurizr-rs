//! Theme fetching and merging for Structurizr workspaces.
//!
//! Themes allow you to define reusable styling that can be applied to workspaces
//! via URL references. This module handles fetching themes from URLs and merging
//! them with workspace styles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::style::{ElementStyle, RelationshipStyle, Styles};
use crate::workspace::Workspace;

/// A fetched theme containing element and relationship styles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub elements: Vec<ElementStyle>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relationships: Vec<RelationshipStyle>,
}

impl Theme {
    /// Create a new empty theme.
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            elements: Vec::new(),
            relationships: Vec::new(),
        }
    }

    /// Merge this theme into a Styles object.
    /// Theme styles are added first, so workspace styles will override them.
    fn merge_into(&self, styles: &mut Styles) {
        // Prepend theme elements (so workspace styles override)
        let mut merged_elements = self.elements.clone();
        merged_elements.extend(styles.elements.drain(..));
        styles.elements = merged_elements;

        // Prepend theme relationships (so workspace styles override)
        let mut merged_relationships = self.relationships.clone();
        merged_relationships.extend(styles.relationships.drain(..));
        styles.relationships = merged_relationships;
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::new()
    }
}

// Theme cache to avoid re-fetching the same theme multiple times.
// Uses Arc<Mutex<>> for thread-safe caching.
lazy_static::lazy_static! {
    static ref THEME_CACHE: Arc<Mutex<HashMap<String, Theme>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Fetch a theme from a URL.
///
/// The theme is fetched via HTTP and parsed as JSON. Results are cached
/// to avoid re-fetching the same theme URL.
///
/// # Arguments
///
/// * `url` - The URL to fetch the theme from
///
/// # Returns
///
/// A `Result` containing the parsed `Theme` or an error.
///
/// # Examples
///
/// ```no_run
/// use structurizr_core::theme::fetch_theme;
///
/// let theme = fetch_theme("https://example.com/theme.json")?;
/// println!("Fetched theme: {:?}", theme.name);
/// # Ok::<(), structurizr_core::Error>(())
/// ```
pub fn fetch_theme(url: &str) -> Result<Theme> {
    // Check cache first
    {
        let cache = THEME_CACHE.lock().map_err(|e| {
            Error::Theme(format!("Failed to acquire theme cache lock: {}", e))
        })?;

        if let Some(cached_theme) = cache.get(url) {
            return Ok(cached_theme.clone());
        }
    }

    // Fetch from URL
    let response = ureq::get(url)
        .call()
        .map_err(|e| Error::Http(format!("Failed to fetch theme from {}: {}", url, e)))?;

    // Check status
    let status = response.status();
    if status != 200 {
        return Err(Error::Http(format!(
            "HTTP error fetching theme from {}: status {}",
            url, status
        )));
    }

    // Parse JSON
    let body = response
        .into_string()
        .map_err(|e| Error::Theme(format!("Failed to read response body from {}: {}", url, e)))?;

    let theme: Theme = serde_json::from_str(&body)
        .map_err(|e| Error::Theme(format!("Failed to parse theme JSON from {}: {}", url, e)))?;

    // Cache the theme
    {
        let mut cache = THEME_CACHE.lock().map_err(|e| {
            Error::Theme(format!("Failed to acquire theme cache lock: {}", e))
        })?;
        cache.insert(url.to_string(), theme.clone());
    }

    Ok(theme)
}

/// Apply all themes referenced in a workspace's styles.
///
/// This function:
/// 1. Fetches all theme URLs specified in the workspace styles
/// 2. Merges theme styles in order (later themes override earlier ones)
/// 3. Ensures workspace styles override all theme styles
///
/// # Arguments
///
/// * `workspace` - The workspace to apply themes to
///
/// # Returns
///
/// A `Result` indicating success or failure.
///
/// # Examples
///
/// ```no_run
/// use structurizr_core::workspace::Workspace;
/// use structurizr_core::theme::apply_themes;
///
/// let mut workspace = Workspace::new("My System", "Description");
/// workspace.styles_mut().add_theme("https://example.com/theme.json");
/// apply_themes(&mut workspace)?;
/// # Ok::<(), structurizr_core::Error>(())
/// ```
pub fn apply_themes(workspace: &mut Workspace) -> Result<()> {
    // Get the theme URLs
    let theme_urls: Vec<String> = workspace
        .styles()
        .map(|s| s.themes.clone())
        .unwrap_or_default();

    if theme_urls.is_empty() {
        return Ok(());
    }

    // Fetch all themes
    let mut themes = Vec::new();
    for url in &theme_urls {
        match fetch_theme(url) {
            Ok(theme) => themes.push(theme),
            Err(e) => {
                // Log the error but continue with other themes
                eprintln!("Warning: Failed to fetch theme from {}: {}", url, e);
            }
        }
    }

    if themes.is_empty() {
        return Ok(());
    }

    // Get or create styles
    let styles = workspace.styles_mut();

    // Save the original workspace styles
    let original_elements = styles.elements.clone();
    let original_relationships = styles.relationships.clone();

    // Clear the current styles
    styles.elements.clear();
    styles.relationships.clear();

    // Apply themes in order (later themes override earlier ones)
    for theme in themes {
        theme.merge_into(styles);
    }

    // Apply original workspace styles last (so they override themes)
    styles.elements.extend(original_elements);
    styles.relationships.extend(original_relationships);

    Ok(())
}

/// Clear the theme cache.
///
/// This is useful for testing or when you want to force re-fetching of themes.
pub fn clear_theme_cache() -> Result<()> {
    let mut cache = THEME_CACHE.lock().map_err(|e| {
        Error::Theme(format!("Failed to acquire theme cache lock: {}", e))
    })?;
    cache.clear();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theme_creation() {
        let theme = Theme::new();
        assert!(theme.name.is_none());
        assert!(theme.elements.is_empty());
        assert!(theme.relationships.is_empty());
    }

    #[test]
    fn test_theme_merge() {
        let mut theme = Theme::new();
        theme.elements.push(ElementStyle {
            tag: "Person".to_string(),
            background: Some("#08427b".to_string()),
            color: Some("#ffffff".to_string()),
            ..Default::default()
        });

        let mut styles = Styles::new();
        styles.add_element_style(ElementStyle {
            tag: "System".to_string(),
            background: Some("#1168bd".to_string()),
            ..Default::default()
        });

        theme.merge_into(&mut styles);

        // Theme element should be first, workspace element second
        assert_eq!(styles.elements.len(), 2);
        assert_eq!(styles.elements[0].tag, "Person");
        assert_eq!(styles.elements[1].tag, "System");
    }

    #[test]
    fn test_theme_override() {
        // Workspace style should override theme style for same tag
        let mut theme = Theme::new();
        theme.elements.push(ElementStyle {
            tag: "Person".to_string(),
            background: Some("#08427b".to_string()),
            ..Default::default()
        });

        let mut styles = Styles::new();
        styles.add_element_style(ElementStyle {
            tag: "Person".to_string(),
            background: Some("#ff0000".to_string()),
            ..Default::default()
        });

        theme.merge_into(&mut styles);

        // Both should be present, with theme first
        assert_eq!(styles.elements.len(), 2);
        assert_eq!(styles.elements[0].tag, "Person");
        assert_eq!(styles.elements[0].background, Some("#08427b".to_string()));
        assert_eq!(styles.elements[1].tag, "Person");
        assert_eq!(styles.elements[1].background, Some("#ff0000".to_string()));

        // When getting effective style, workspace style (later) should win
        let effective = styles.get_element_style(&["Person".to_string()]);
        assert_eq!(effective.background, Some("#ff0000".to_string()));
    }

    #[test]
    fn test_clear_cache() {
        clear_theme_cache().unwrap();
    }
}
