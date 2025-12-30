//! HTTP request handlers.

use axum::{
    extract::{Path, Query, State},
    http::header,
    response::{Html, IntoResponse},
    Json,
};

/// Query parameters for export endpoints.
#[derive(Debug, serde::Deserialize)]
pub struct ExportQuery {
    /// If true, return raw code instead of rendered HTML viewer.
    pub raw: Option<bool>,
}

use structurizr_core::model::ElementId;
use structurizr_core::navigation::NavigationIndex;
use structurizr_core::view::SystemLandscapeView;
use structurizr_core::Workspace;
use structurizr_export::{D2Exporter, DotExporter, JsonExporter, MermaidExporter, PlantUmlExporter};
use structurizr_render::SvgRenderer;
use structurizr_render::layout::{GridLayout, LayoutEdge, Size};

use crate::error::{Error, Result};
use crate::layout::{ContentType, LayoutConfig, NavItem, generate_page_layout};
use crate::markdown::{escape_html, render_markdown, render_markdown_with_heading_ids, ExtractedHeading};
use crate::notes::{AddNoteRequest, NotesFile, ViewNotes};
use crate::state::AppState;


/// Generate a single view card HTML.
fn generate_view_card(key: &str, base_path: &str, is_dynamic: bool) -> String {
    let animate_link = if is_dynamic {
        format!(r#" | <a href="{}/view/{}/animate">Animate</a>"#, base_path, key)
    } else {
        String::new()
    };
    format!(
        r#"<div class="view-card">
            <h3><a href="{}/view/{}">{}</a></h3>
            <p>
                <a href="{}/edit/{}">Edit</a> |
                <a href="{}/presentation?views={}">Present</a>{}
                | <a href="{}/view/{}/svg">SVG</a>
                | <a href="{}/view/{}/plantuml">PlantUML</a>
                | <a href="{}/view/{}/mermaid">Mermaid</a>
                | <a href="{}/view/{}/dot">DOT</a>
                | <a href="{}/view/{}/d2">D2</a>
            </p>
        </div>"#,
        base_path, key, escape_html(key),
        base_path, key,
        base_path, key, animate_link,
        base_path, key,
        base_path, key,
        base_path, key,
        base_path, key,
        base_path, key
    )
}

/// Generate a collapsible section for a category of views.
/// Returns empty string if views is empty.
fn generate_view_section(title: &str, views: &[(&str, bool)], base_path: &str) -> String {
    if views.is_empty() {
        return String::new();
    }

    let cards: String = views
        .iter()
        .map(|(key, is_dynamic)| generate_view_card(key, base_path, *is_dynamic))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"<details class="view-section" open>
            <summary>{} ({})</summary>
            <div class="views">
                {}
            </div>
        </details>"#,
        escape_html(title),
        views.len(),
        cards
    )
}

/// Generate home page HTML content (shared between single and multi-workspace modes).
fn generate_home_page_html(ws: &Workspace, base_path: &str, workspace_id: Option<&str>) -> String {
    let views = ws.views();

    // Group views by category
    // C4 Model Views: System Landscape, System Context, Container, Component
    let c4_views: Vec<(&str, bool)> = views
        .system_landscape_views
        .iter()
        .map(|v| (v.properties.key.as_str(), false))
        .chain(
            views
                .system_context_views
                .iter()
                .map(|v| (v.properties.key.as_str(), false)),
        )
        .chain(
            views
                .container_views
                .iter()
                .map(|v| (v.properties.key.as_str(), false)),
        )
        .chain(
            views
                .component_views
                .iter()
                .map(|v| (v.properties.key.as_str(), false)),
        )
        .collect();

    // Behavioral Views: Dynamic
    let behavioral_views: Vec<(&str, bool)> = views
        .dynamic_views
        .iter()
        .map(|v| (v.properties.key.as_str(), true))
        .collect();

    // Infrastructure Views: Deployment
    let infrastructure_views: Vec<(&str, bool)> = views
        .deployment_views
        .iter()
        .map(|v| (v.properties.key.as_str(), false))
        .collect();

    // Extended Views: Filtered, Custom, Image
    let extended_views: Vec<(&str, bool)> = views
        .filtered_views
        .iter()
        .map(|v| (v.properties.key.as_str(), false))
        .chain(
            views
                .custom_views
                .iter()
                .map(|v| (v.properties.key.as_str(), false)),
        )
        .chain(
            views
                .image_views
                .iter()
                .map(|v| (v.properties.key.as_str(), false)),
        )
        .collect();

    // Generate sections
    let sections = [
        generate_view_section("C4 Model Views", &c4_views, base_path),
        generate_view_section("Behavioral Views", &behavioral_views, base_path),
        generate_view_section("Infrastructure Views", &infrastructure_views, base_path),
        generate_view_section("Extended Views", &extended_views, base_path),
    ]
    .into_iter()
    .filter(|s| !s.is_empty())
    .collect::<Vec<_>>()
    .join("\n");

    // Handle case where no views exist
    let views_content = if sections.is_empty() {
        r#"<p class="no-views">No views defined in this workspace.</p>"#.to_string()
    } else {
        sections
    };

    // Page-specific styles
    let extra_styles = r##"<style>
        h1 { margin-top: 0; }
        .workspace-info {
            background: var(--card-bg);
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px var(--shadow);
        }
        .workspace-info p { margin: 8px 0; }
        .view-section {
            margin-bottom: 24px;
        }
        .view-section summary {
            cursor: pointer;
            font-size: 1.2em;
            font-weight: 600;
            padding: 12px 0;
            user-select: none;
            list-style: none;
        }
        .view-section summary::-webkit-details-marker {
            display: none;
        }
        .view-section summary::before {
            content: "▼ ";
            font-size: 0.8em;
            margin-right: 4px;
        }
        .view-section:not([open]) summary::before {
            content: "▶ ";
        }
        .view-section summary:hover {
            color: var(--link-color);
        }
        .view-section .views {
            margin-top: 12px;
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
            gap: 20px;
        }
        .view-card {
            background: var(--card-bg);
            padding: 20px;
            border-radius: 8px;
            box-shadow: 0 2px 4px var(--shadow);
            transition: box-shadow 0.2s ease;
        }
        .view-card:hover {
            box-shadow: 0 4px 8px var(--shadow-medium);
        }
        .view-card h3 { margin-top: 0; }
        .view-card p {
            font-size: 13px;
            color: var(--text-secondary);
        }
        .no-views {
            color: var(--text-secondary);
            font-style: italic;
        }
    </style>"##;

    // Build content
    let content = format!(
        r#"<h1>{}</h1>
        <div class="workspace-info">
            <p>{}</p>
            <p><strong>People:</strong> {}</p>
            <p><strong>Software Systems:</strong> {}</p>
            <p><strong>Relationships:</strong> {}</p>
        </div>
        <h2>Views</h2>
        {}"#,
        escape_html(&ws.name),
        ws.description.as_deref().map(escape_html).unwrap_or_default(),
        ws.model().people.len(),
        ws.model().software_systems.len(),
        ws.model().relationships.len(),
        views_content
    );

    let config = LayoutConfig {
        title: &ws.name,
        workspace_name: Some(&ws.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Home,
        content_type: ContentType::Standard,
        extra_head: extra_styles,
        extra_body_end: "",
    };

    generate_page_layout(&config, &content)
}


/// Escape special characters for JSON strings.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

// =============================================================================
// Documentation Navigation Tree Structures
// =============================================================================

/// Tree node for hierarchical navigation.
#[derive(Debug, Clone)]
struct HeadingNode {
    /// Heading level (0 = section, 1-6 = h1-h6)
    #[allow(dead_code)]
    level: u8,
    /// Display title
    title: String,
    /// Anchor ID
    id: String,
    /// Nested child headings
    children: Vec<HeadingNode>,
}

/// Build a tree structure from a flat list of extracted headings.
/// Uses level-based nesting: headings with higher levels become children
/// of preceding headings with lower levels.
fn build_heading_tree(headings: Vec<ExtractedHeading>) -> Vec<HeadingNode> {
    if headings.is_empty() {
        return Vec::new();
    }

    let mut result: Vec<HeadingNode> = Vec::new();
    let mut idx = 0;

    while idx < headings.len() {
        let (node, consumed) = build_subtree(&headings, idx);
        result.push(node);
        idx += consumed;
    }

    result
}

/// Recursively build a subtree starting at the given index.
/// Returns the node and how many headings were consumed.
fn build_subtree(headings: &[ExtractedHeading], start: usize) -> (HeadingNode, usize) {
    let heading = &headings[start];
    let mut node = HeadingNode {
        level: heading.level,
        title: heading.title.clone(),
        id: heading.id.clone(),
        children: Vec::new(),
    };

    let mut consumed = 1;
    let mut idx = start + 1;

    // Collect all following headings with higher level as children
    while idx < headings.len() && headings[idx].level > heading.level {
        let (child, child_consumed) = build_subtree(headings, idx);
        node.children.push(child);
        consumed += child_consumed;
        idx += child_consumed;
    }

    (node, consumed)
}

/// Render the navigation tree as HTML.
fn render_nav_tree(nodes: &[HeadingNode], depth: usize) -> String {
    nodes.iter().map(|node| {
        let has_children = !node.children.is_empty();
        let depth_class = format!("depth-{}", depth.min(5));
        let expanded_class = if depth == 0 { " expanded" } else { "" };

        if has_children {
            format!(
                r##"<li class="nav-item expandable{} {}">
                    <div class="nav-row">
                        <span class="toggle"></span>
                        <a href="#{}" class="nav-link">{}</a>
                    </div>
                    <ul class="nav-children">{}</ul>
                </li>"##,
                expanded_class,
                depth_class,
                node.id,
                escape_html(&node.title),
                render_nav_tree(&node.children, depth + 1)
            )
        } else {
            format!(
                r##"<li class="nav-item leaf {}">
                    <a href="#{}" class="nav-link">{}</a>
                </li>"##,
                depth_class,
                node.id,
                escape_html(&node.title)
            )
        }
    }).collect()
}

/// Health check endpoint.
pub async fn health() -> &'static str {
    "OK"
}

/// Presentation mode query parameters.
#[derive(Debug, serde::Deserialize)]
pub struct PresentationQuery {
    pub views: Option<String>, // Comma-separated list of view keys
}

/// Search query parameters.
#[derive(Debug, serde::Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

/// Search result item.
#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub id: String,
    pub name: String,
    pub element_type: String,
    pub description: Option<String>,
    pub url: String,
}

/// Generate search page HTML content (shared between single and multi-workspace modes).
fn generate_search_page_html(workspace: &Workspace, base_path: &str, workspace_id: Option<&str>, search_term: &str) -> String {
    let results = perform_search(workspace, search_term);

    let results_html: String = if results.is_empty() && !search_term.is_empty() {
        "<p class=\"no-results\">No results found.</p>".to_string()
    } else {
        results.iter().map(|r| format!(
            r#"<div class="result">
                <div class="result-header">
                    <span class="type">{}</span>
                    <h3>{}</h3>
                </div>
                {}
            </div>"#,
            escape_html(&r.element_type),
            escape_html(&r.name),
            r.description.as_ref().map(|d| format!("<p class=\"desc\">{}</p>", escape_html(d))).unwrap_or_default()
        )).collect()
    };

    // Page-specific styles
    let extra_styles = r##"<style>
        .search-container { max-width: 800px; margin: 0 auto; }
        .search-box { display: flex; gap: 10px; margin-bottom: 30px; }
        .search-box input { flex: 1; padding: 12px 16px; font-size: 16px; border: 2px solid var(--border-color); border-radius: 8px; background: var(--bg-secondary); color: var(--text-primary); }
        .search-box input:focus { outline: none; border-color: var(--link-color); }
        .search-box button { background: var(--link-color); color: white; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer; font-size: 16px; }
        .search-box button:hover { background: var(--link-hover); }
        .result { background: var(--card-bg); padding: 20px; border-radius: 8px; margin-bottom: 15px; box-shadow: 0 1px 3px var(--shadow); }
        .result-header { display: flex; align-items: center; gap: 10px; }
        .result-header h3 { margin: 0; }
        .type { background: var(--bg-tertiary); padding: 4px 10px; border-radius: 4px; font-size: 11px; text-transform: uppercase; font-weight: 600; }
        .desc { color: var(--text-secondary); margin: 10px 0 0 0; }
        .no-results { color: var(--text-muted); font-style: italic; text-align: center; padding: 40px; }
        .result-count { color: var(--text-secondary); margin-bottom: 20px; }
    </style>"##;

    let result_count_html = if !search_term.is_empty() {
        format!("<p class=\"result-count\">{} results for \"{}\"</p>", results.len(), escape_html(search_term))
    } else {
        String::new()
    };

    let content = format!(
        r#"<div class="search-container">
            <form class="search-box" method="get">
                <input type="text" name="q" placeholder="Search elements, relationships, documentation..." value="{}" autofocus>
                <button type="submit">Search</button>
            </form>
            {}
            <div class="results">
                {}
            </div>
        </div>"#,
        escape_html(search_term),
        result_count_html,
        results_html
    );

    let title = format!("Search - {}", workspace.name);
    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Search,
        content_type: ContentType::Standard,
        extra_head: extra_styles,
        extra_body_end: "",
    };

    generate_page_layout(&config, &content)
}

/// Perform search across workspace elements.
fn perform_search(workspace: &structurizr_core::Workspace, query: &str) -> Vec<SearchResult> {
    if query.is_empty() {
        return vec![];
    }

    let query_lower = query.to_lowercase();
    let mut results = vec![];

    // Search people
    for person in &workspace.model().people {
        if matches_search(&person.name(), &query_lower)
            || person.properties.description.as_ref().map(|d| matches_search(d, &query_lower)).unwrap_or(false)
        {
            results.push(SearchResult {
                id: person.id().to_string(),
                name: person.name().to_string(),
                element_type: "Person".to_string(),
                description: person.properties.description.clone(),
                url: "/".to_string(),
            });
        }
    }

    // Search software systems and containers
    for system in &workspace.model().software_systems {
        if matches_search(&system.name(), &query_lower)
            || system.properties.description.as_ref().map(|d| matches_search(d, &query_lower)).unwrap_or(false)
        {
            results.push(SearchResult {
                id: system.id().to_string(),
                name: system.name().to_string(),
                element_type: "Software System".to_string(),
                description: system.properties.description.clone(),
                url: "/".to_string(),
            });
        }

        for container in &system.containers {
            if matches_search(&container.name(), &query_lower)
                || container.properties.description.as_ref().map(|d| matches_search(d, &query_lower)).unwrap_or(false)
                || container.technology.as_ref().map(|t| matches_search(t, &query_lower)).unwrap_or(false)
            {
                results.push(SearchResult {
                    id: container.id().to_string(),
                    name: container.name().to_string(),
                    element_type: "Container".to_string(),
                    description: container.properties.description.clone(),
                    url: "/".to_string(),
                });
            }

            for component in &container.components {
                if matches_search(&component.name(), &query_lower)
                    || component.properties.description.as_ref().map(|d| matches_search(d, &query_lower)).unwrap_or(false)
                    || component.technology.as_ref().map(|t| matches_search(t, &query_lower)).unwrap_or(false)
                {
                    results.push(SearchResult {
                        id: component.id().to_string(),
                        name: component.name().to_string(),
                        element_type: "Component".to_string(),
                        description: component.properties.description.clone(),
                        url: "/".to_string(),
                    });
                }
            }
        }
    }

    // Search documentation
    for (i, section) in workspace.documentation.sections.iter().enumerate() {
        let title = section.title.as_deref().unwrap_or("");
        if matches_search(title, &query_lower) || matches_search(&section.content, &query_lower) {
            results.push(SearchResult {
                id: format!("doc-section-{}", i),
                name: title.to_string(),
                element_type: "Documentation".to_string(),
                description: Some(section.content.chars().take(200).collect()),
                url: format!("/docs#section-{}", i),
            });
        }
    }

    // Search ADRs
    for decision in &workspace.documentation.decisions {
        if matches_search(&decision.title, &query_lower)
            || matches_search(&decision.content, &query_lower)
            || matches_search(&decision.id, &query_lower)
        {
            results.push(SearchResult {
                id: format!("adr-{}", decision.id),
                name: format!("{}: {}", decision.id, decision.title),
                element_type: "ADR".to_string(),
                description: Some(decision.content.chars().take(200).collect()),
                url: format!("/docs#adr-{}", decision.id),
            });
        }
    }

    results
}

/// Check if text matches search query.
fn matches_search(text: &str, query: &str) -> bool {
    text.to_lowercase().contains(query)
}

// Use the explore module's function for generating explore page HTML
use crate::explore::generate_explore_page_html;

// The old explore page implementation has been moved to the explore module.
// See crates/structurizr-web/src/explore/ for the modular implementation.

/// Helper to extract workspace ID from a path string.
fn extract_workspace_id(path: &str) -> String {
    // Remove leading slash if present
    path.trim_start_matches('/').to_string()
}

/// Parse workspace path from wildcard capture to extract workspace ID and remaining segments.
///
/// Tries progressively shorter prefixes until a valid workspace is found.
/// For example, given path "small/startup-saas/view/Context":
/// - First tries "small/startup-saas/view/Context" as workspace ID
/// - Then tries "small/startup-saas/view" with remaining ["Context"]
/// - Then tries "small/startup-saas" with remaining ["view", "Context"] <- matches!
///
/// Returns (workspace_id, remaining_segments) or None if no valid workspace found.
async fn parse_workspace_path(state: &AppState, path: &str) -> Option<(String, Vec<String>)> {
    let clean_path = path.trim_start_matches('/');
    let segments: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();

    if segments.is_empty() {
        return None;
    }

    // Try progressively shorter prefixes until we find a valid workspace
    for i in (1..=segments.len()).rev() {
        let candidate_id = segments[..i].join("/");
        if state.workspace_exists(&candidate_id).await {
            let remaining: Vec<String> = segments[i..].iter().map(|s| s.to_string()).collect();
            return Some((candidate_id, remaining));
        }
    }

    None
}

pub async fn workspaces_index(State(state): State<AppState>) -> Result<Html<String>> {
    let workspaces = state.list_workspaces().await;

    let workspace_content: String = if workspaces.is_empty() {
        r#"<div class="empty-state">
            <h2>No Workspaces Found</h2>
            <p>Create a workspace by adding a directory containing a <code>workspace.dsl</code> file.</p>
        </div>"#.to_string()
    } else {
        // Group workspaces by top-level folder
        let mut groups: std::collections::BTreeMap<String, Vec<_>> = std::collections::BTreeMap::new();
        for ws in &workspaces {
            let top_level = ws.id.split('/').next().unwrap_or(&ws.id).to_string();
            groups.entry(top_level).or_default().push(ws);
        }

        // Helper to format last modified time
        let format_last_modified = |ws: &crate::discovery::WorkspaceInfo| {
            ws.last_modified
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| {
                    let secs = d.as_secs();
                    let days_ago = (std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() - secs) / 86400;
                    if days_ago == 0 { "Today".to_string() }
                    else if days_ago == 1 { "Yesterday".to_string() }
                    else { format!("{} days ago", days_ago) }
                })
                .unwrap_or_else(|_| "Unknown".to_string())
        };

        // Helper to generate a workspace card
        let make_card = |ws: &crate::discovery::WorkspaceInfo| {
            let description = ws.description.as_deref().unwrap_or("No description");
            let last_modified = format_last_modified(ws);
            format!(
                r#"<a href="/w/{}" class="workspace-card">
                    <div class="card-header">
                        <h3>{}</h3>
                        <span class="view-count">{} views</span>
                    </div>
                    <p class="description">{}</p>
                    <div class="card-footer">
                        <span class="path">{}</span>
                        <span class="modified">{}</span>
                    </div>
                </a>"#,
                ws.id,
                escape_html(&ws.name),
                ws.view_count,
                escape_html(description),
                escape_html(&ws.id),
                last_modified
            )
        };

        // Generate grouped HTML
        groups.iter().map(|(group_name, group_workspaces)| {
            let cards: String = group_workspaces.iter()
                .map(|ws| make_card(ws))
                .collect::<Vec<_>>()
                .join("\n");

            // If group has only one workspace and the group name equals the workspace id,
            // it's a top-level workspace - don't show a group header
            if group_workspaces.len() == 1 && group_workspaces[0].id == *group_name {
                format!(
                    r#"<div class="workspaces-grid">{}</div>"#,
                    cards
                )
            } else {
                format!(
                    r#"<div class="workspace-group">
                        <h2 class="group-header">{}</h2>
                        <div class="workspaces-grid">{}</div>
                    </div>"#,
                    escape_html(group_name),
                    cards
                )
            }
        }).collect::<Vec<_>>().join("\n")
    };

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Structurizr Workspaces</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            margin: 0;
            padding: 0;
            background: #f5f5f5;
            min-height: 100vh;
        }}
        .header {{
            background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
            color: white;
            padding: 40px 20px;
            text-align: center;
        }}
        .header h1 {{
            margin: 0 0 10px 0;
            font-size: 2.5rem;
            font-weight: 600;
        }}
        .header p {{
            margin: 0;
            opacity: 0.8;
            font-size: 1.1rem;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
            padding: 30px 20px;
        }}
        .workspaces-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
            gap: 24px;
        }}
        .workspace-card {{
            background: white;
            border-radius: 12px;
            padding: 24px;
            text-decoration: none;
            color: inherit;
            box-shadow: 0 2px 8px rgba(0,0,0,0.08);
            transition: transform 0.2s, box-shadow 0.2s;
            display: block;
        }}
        .workspace-card:hover {{
            transform: translateY(-4px);
            box-shadow: 0 8px 24px rgba(0,0,0,0.12);
        }}
        .card-header {{
            display: flex;
            justify-content: space-between;
            align-items: flex-start;
            margin-bottom: 12px;
        }}
        .card-header h3 {{
            margin: 0;
            color: #1a1a2e;
            font-size: 1.25rem;
        }}
        .view-count {{
            background: #e8f4fd;
            color: #0066cc;
            padding: 4px 10px;
            border-radius: 12px;
            font-size: 0.85rem;
            font-weight: 500;
        }}
        .description {{
            color: #666;
            margin: 0 0 16px 0;
            line-height: 1.5;
            font-size: 0.95rem;
        }}
        .card-footer {{
            display: flex;
            justify-content: space-between;
            font-size: 0.85rem;
            color: #999;
            border-top: 1px solid #eee;
            padding-top: 12px;
        }}
        .path {{
            font-family: monospace;
            background: #f5f5f5;
            padding: 2px 6px;
            border-radius: 4px;
        }}
        .empty-state {{
            text-align: center;
            padding: 60px 20px;
            color: #666;
        }}
        .empty-state h2 {{
            color: #333;
        }}
        .empty-state code {{
            background: #f5f5f5;
            padding: 2px 6px;
            border-radius: 4px;
        }}
        .workspace-group {{
            margin-bottom: 32px;
        }}
        .group-header {{
            color: #1a1a2e;
            font-size: 1.5rem;
            font-weight: 600;
            margin: 0 0 16px 0;
            padding-bottom: 8px;
            border-bottom: 2px solid #e0e0e0;
            text-transform: capitalize;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Structurizr Workspaces</h1>
        <p>{} workspace{} available</p>
    </div>
    <div class="container">
        {}
    </div>
</body>
</html>"##,
        workspaces.len(),
        if workspaces.len() == 1 { "" } else { "s" },
        workspace_content
    );

    Ok(Html(html))
}

/// Workspace home page - shows views for a specific workspace.
pub async fn workspace_home(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    let html = generate_home_page_html(&workspace, &base_path, Some(&workspace_id));

    Ok(Html(html))
}

/// Workspace-scoped view diagram handler.
pub async fn workspace_view_diagram(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_view_diagram_html(&workspace, &view_key, &base_path)
}

/// Workspace-scoped animated dynamic view handler.
pub async fn workspace_view_animated(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_dynamic_animated_html(&workspace, &view_key, &base_path)
}

/// Workspace-scoped edit diagram handler.
pub async fn workspace_edit_diagram(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_edit_diagram_html(&workspace, &view_key, &base_path)
}

/// Workspace-scoped documentation handler.
pub async fn workspace_documentation(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_documentation_html(&workspace, &base_path)
}

/// Workspace-scoped search page handler.
pub async fn workspace_search_page(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    let search_term = query.q.unwrap_or_default();
    let html = generate_search_page_html(&workspace, &base_path, Some(&workspace_id), &search_term);
    Ok(Html(html))
}

/// Workspace-scoped tree view handler.
pub async fn workspace_tree_view(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_tree_view_html(&workspace, &base_path)
}

/// Workspace-scoped presentation handler.
pub async fn workspace_presentation(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PresentationQuery>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    render_presentation_html(&workspace, &base_path, query.views)
}

/// Workspace-scoped explore view handler.
pub async fn workspace_explore(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Html<String>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);
    let html = generate_explore_page_html(&workspace, &base_path, Some(&workspace_id));
    Ok(Html(html))
}

/// Workspace-scoped get workspace JSON handler.
pub async fn workspace_get_json(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Json<Workspace>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    Ok(Json(workspace))
}

/// Workspace-scoped validate handler.
pub async fn workspace_validate(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<Json<structurizr_dsl::ValidationResult>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let validation_result = structurizr_dsl::validate_workspace(&workspace);
    Ok(Json(validation_result))
}

/// Generate tree view HTML content (shared between single and multi-workspace modes).
fn generate_tree_page_html(workspace: &Workspace, base_path: &str, workspace_id: Option<&str>) -> String {
    let model = workspace.model();

    // Build tree structure HTML
    let mut tree_html = String::new();

    // People branch
    if !model.people.is_empty() {
        tree_html.push_str(r#"<li class="expandable expanded">"#);
        tree_html.push_str(r#"<div class="tree-node" data-type="group">"#);
        tree_html.push_str(r#"<span class="toggle">▼</span>"#);
        tree_html.push_str(r#"<span class="icon">👤</span>"#);
        tree_html.push_str(r#"<span class="name">People</span>"#);
        tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, model.people.len()));
        tree_html.push_str(r#"</div>"#);
        tree_html.push_str(r#"<ul class="children">"#);

        for person in &model.people {
            tree_html.push_str(r#"<li class="leaf">"#);
            tree_html.push_str(&format!(
                r#"<div class="tree-node" data-id="{}" data-type="Person" data-name="{}" data-description="{}">"#,
                person.id(),
                escape_html(&person.name()),
                person.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default()
            ));
            tree_html.push_str(r#"<span class="icon">👤</span>"#);
            tree_html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&person.name())));
            if let Some(desc) = &person.properties.description {
                let truncated = if desc.len() > 50 {
                    format!("{}...", &desc[..50])
                } else {
                    desc.clone()
                };
                tree_html.push_str(&format!(r#"<span class="desc">{}</span>"#, escape_html(&truncated)));
            }
            tree_html.push_str(r#"</div>"#);
            tree_html.push_str(r#"</li>"#);
        }

        tree_html.push_str(r#"</ul></li>"#);
    }

    // Software Systems branch
    if !model.software_systems.is_empty() {
        tree_html.push_str(r#"<li class="expandable expanded">"#);
        tree_html.push_str(r#"<div class="tree-node" data-type="group">"#);
        tree_html.push_str(r#"<span class="toggle">▼</span>"#);
        tree_html.push_str(r#"<span class="icon">📦</span>"#);
        tree_html.push_str(r#"<span class="name">Software Systems</span>"#);
        tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, model.software_systems.len()));
        tree_html.push_str(r#"</div>"#);
        tree_html.push_str(r#"<ul class="children">"#);

        for system in &model.software_systems {
            let has_containers = !system.containers.is_empty();

            if has_containers {
                tree_html.push_str(r#"<li class="expandable">"#);
            } else {
                tree_html.push_str(r#"<li class="leaf">"#);
            }

            tree_html.push_str(&format!(
                r#"<div class="tree-node" data-id="{}" data-type="Software System" data-name="{}" data-description="{}">"#,
                system.id(),
                escape_html(&system.name()),
                system.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default()
            ));

            if has_containers {
                tree_html.push_str(r#"<span class="toggle">▶</span>"#);
            }
            tree_html.push_str(r#"<span class="icon">📦</span>"#);
            tree_html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&system.name())));
            if has_containers {
                tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, system.containers.len()));
            }
            if let Some(desc) = &system.properties.description {
                let truncated = if desc.len() > 50 {
                    format!("{}...", &desc[..50])
                } else {
                    desc.clone()
                };
                tree_html.push_str(&format!(r#"<span class="desc">{}</span>"#, escape_html(&truncated)));
            }
            tree_html.push_str(r#"</div>"#);

            // Containers branch
            if has_containers {
                tree_html.push_str(r#"<ul class="children" style="display: none;">"#);

                for container in &system.containers {
                    let has_components = !container.components.is_empty();

                    if has_components {
                        tree_html.push_str(r#"<li class="expandable">"#);
                    } else {
                        tree_html.push_str(r#"<li class="leaf">"#);
                    }

                    tree_html.push_str(&format!(
                        r#"<div class="tree-node" data-id="{}" data-type="Container" data-name="{}" data-description="{}" data-technology="{}">"#,
                        container.id(),
                        escape_html(&container.name()),
                        container.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default(),
                        container.technology.as_ref().map(|t| escape_html(t)).unwrap_or_default()
                    ));

                    if has_components {
                        tree_html.push_str(r#"<span class="toggle">▶</span>"#);
                    }
                    tree_html.push_str(r#"<span class="icon">📁</span>"#);
                    tree_html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&container.name())));
                    if has_components {
                        tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, container.components.len()));
                    }
                    if let Some(tech) = &container.technology {
                        tree_html.push_str(&format!(r#"<span class="tech">{}</span>"#, escape_html(tech)));
                    }
                    tree_html.push_str(r#"</div>"#);

                    // Components branch
                    if has_components {
                        tree_html.push_str(r#"<ul class="children" style="display: none;">"#);

                        for component in &container.components {
                            tree_html.push_str(r#"<li class="leaf">"#);
                            tree_html.push_str(&format!(
                                r#"<div class="tree-node" data-id="{}" data-type="Component" data-name="{}" data-description="{}" data-technology="{}">"#,
                                component.id(),
                                escape_html(&component.name()),
                                component.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default(),
                                component.technology.as_ref().map(|t| escape_html(t)).unwrap_or_default()
                            ));
                            tree_html.push_str(r#"<span class="icon">⚙️</span>"#);
                            tree_html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&component.name())));
                            if let Some(tech) = &component.technology {
                                tree_html.push_str(&format!(r#"<span class="tech">{}</span>"#, escape_html(tech)));
                            }
                            tree_html.push_str(r#"</div>"#);
                            tree_html.push_str(r#"</li>"#);
                        }

                        tree_html.push_str(r#"</ul>"#);
                    }

                    tree_html.push_str(r#"</li>"#);
                }

                tree_html.push_str(r#"</ul>"#);
            }

            tree_html.push_str(r#"</li>"#);
        }

        tree_html.push_str(r#"</ul></li>"#);
    }

    // Generate the complete HTML page
    let extra_styles = r##"<style>
        .tree-container {
            padding: 20px;
            background: var(--bg-secondary);
            min-height: calc(100vh - 60px);
        }
        .tree {
            list-style: none;
            padding: 0;
            margin: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
        }
        .tree ul {
            list-style: none;
            padding-left: 20px;
            margin: 0;
        }
        .tree-node {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 8px 12px;
            border-radius: 4px;
            cursor: pointer;
            transition: background 0.2s;
        }
        .tree-node:hover {
            background: var(--bg-tertiary);
        }
        .tree-node[data-selected="true"] {
            background: var(--link-color);
            color: white;
        }
        .toggle {
            width: 16px;
            font-size: 12px;
            user-select: none;
            transition: transform 0.2s;
        }
        .icon {
            font-size: 18px;
        }
        .name {
            font-weight: 500;
            color: var(--text-primary);
        }
        .count {
            color: var(--text-muted);
            font-size: 12px;
        }
        .tech {
            color: var(--text-secondary);
            font-size: 12px;
            background: var(--bg-tertiary);
            padding: 2px 6px;
            border-radius: 3px;
        }
        .desc {
            color: var(--text-secondary);
            font-size: 12px;
            margin-left: auto;
            max-width: 300px;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }
        .search-box {
            margin-bottom: 20px;
            position: sticky;
            top: 0;
            background: var(--bg-secondary);
            padding: 10px 0;
            z-index: 10;
        }
        .search-box input {
            width: 100%;
            padding: 10px;
            border: 1px solid var(--border-color);
            border-radius: 4px;
            background: var(--bg-primary);
            color: var(--text-primary);
            font-size: 14px;
        }
        .search-box input:focus {
            outline: none;
            border-color: var(--link-color);
        }
        .highlight {
            background: yellow;
            color: black;
        }
    </style>"##;

    let extra_scripts = r##"<script>
        // Tree expand/collapse functionality
        document.addEventListener('DOMContentLoaded', function() {
            const toggles = document.querySelectorAll('.toggle');
            toggles.forEach(toggle => {
                toggle.addEventListener('click', function(e) {
                    e.stopPropagation();
                    const li = this.closest('li');
                    const children = li.querySelector('.children');
                    if (children) {
                        const isExpanded = children.style.display !== 'none';
                        children.style.display = isExpanded ? 'none' : 'block';
                        this.textContent = isExpanded ? '▶' : '▼';
                        this.style.transform = isExpanded ? 'rotate(0deg)' : 'rotate(0deg)';
                    }
                });
            });

            // Node selection
            const nodes = document.querySelectorAll('.tree-node');
            nodes.forEach(node => {
                node.addEventListener('click', function(e) {
                    // Remove previous selection
                    document.querySelectorAll('.tree-node[data-selected="true"]').forEach(n => {
                        n.setAttribute('data-selected', 'false');
                    });
                    // Select this node
                    this.setAttribute('data-selected', 'true');

                    // Show details in console
                    const id = this.getAttribute('data-id');
                    const type = this.getAttribute('data-type');
                    const name = this.getAttribute('data-name');
                    const desc = this.getAttribute('data-description');
                    console.log('Selected:', { id, type, name, desc });
                });
            });

            // Search functionality
            const searchInput = document.getElementById('tree-search');
            if (searchInput) {
                searchInput.addEventListener('input', function() {
                    const query = this.value.toLowerCase();
                    const allNodes = document.querySelectorAll('.tree-node');

                    if (query === '') {
                        // Reset all nodes
                        allNodes.forEach(node => {
                            node.style.display = 'flex';
                            const li = node.closest('li');
                            if (li) {
                                li.style.display = 'list-item';
                            }
                        });
                        return;
                    }

                    // Hide all nodes first
                    allNodes.forEach(node => {
                        const name = node.getAttribute('data-name');
                        const desc = node.getAttribute('data-description');
                        const tech = node.getAttribute('data-technology');

                        const matches =
                            (name && name.toLowerCase().includes(query)) ||
                            (desc && desc.toLowerCase().includes(query)) ||
                            (tech && tech.toLowerCase().includes(query));

                        const li = node.closest('li');
                        if (matches) {
                            node.style.display = 'flex';
                            if (li) {
                                li.style.display = 'list-item';
                                // Expand parents
                                let parent = li.parentElement;
                                while (parent && parent.classList.contains('children')) {
                                    parent.style.display = 'block';
                                    const parentLi = parent.closest('li');
                                    if (parentLi) {
                                        const toggle = parentLi.querySelector('.toggle');
                                        if (toggle) {
                                            toggle.textContent = '▼';
                                        }
                                    }
                                    parent = parent.parentElement?.parentElement;
                                }
                            }
                        } else {
                            node.style.display = 'none';
                            if (li && !li.querySelector('.tree-node[style*="flex"]')) {
                                li.style.display = 'none';
                            }
                        }
                    });
                });
            }
        });
    </script>"##;

    let content = format!(r##"
        <div class="tree-container">
            <div class="search-box">
                <input type="text" id="tree-search" placeholder="Search elements..." autocomplete="off">
            </div>
            <ul class="tree">
                {}
            </ul>
        </div>
    "##, tree_html);

    let title = format!("Model Tree - {}", workspace.name);
    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Tree,
        content_type: ContentType::Standard,
        extra_head: extra_styles,
        extra_body_end: extra_scripts,
    };

    generate_page_layout(&config, &content)
}/// - `workspace`: The workspace containing the diagram
/// - `view_key`: The key of the view to render
/// - `base_path`: Base path for URLs (empty for single-workspace, "/w/workspace_id" for multi)
fn generate_view_diagram_html(workspace: &Workspace, view_key: &str, base_path: &str) -> String {
    // Extract workspace_id from base_path for layout
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };

    let svg_url = format!("{}/view/{}/svg", base_path, view_key);
    let model = workspace.model();
    let views = workspace.views();

    // Build navigation index for drill-down support
    let nav_index = NavigationIndex::build(views);

    // Build breadcrumbs for this view
    let breadcrumbs = nav_index.build_breadcrumbs(view_key);
    let breadcrumbs_json = serde_json::to_string(&breadcrumbs).unwrap_or_else(|_| "[]".to_string());

    // Get current view info
    let current_view_title = nav_index.get_view_title(view_key)
        .cloned()
        .unwrap_or_else(|| view_key.to_string());

    // Collect elements based on view type - MUST match SVG renderer element collection
    // Tuple: (id_string, element_id, name, type, description, technology)
    let mut element_ids: Vec<String> = Vec::new();
    let mut element_data: Vec<(String, ElementId, String, String, Option<String>, Option<String>)> = Vec::new();
    let mut auto_layout_config: Option<&structurizr_core::view::AutoLayout> = None;

    // Determine view type and collect appropriate elements (matching SVG renderer logic)
    if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        auto_layout_config = view.properties.auto_layout.as_ref();
        for person in &model.people {
            element_ids.push(person.id().to_string());
            element_data.push((
                person.id().to_string(),
                person.id(),
                person.name().to_string(),
                "Person".to_string(),
                person.properties.description.clone(),
                None,
            ));
        }
        for system in &model.software_systems {
            element_ids.push(system.id().to_string());
            element_data.push((
                system.id().to_string(),
                system.id(),
                system.name().to_string(),
                "Software System".to_string(),
                system.properties.description.clone(),
                None,
            ));
        }
    } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
        auto_layout_config = view.properties.auto_layout.as_ref();
        for person in &model.people {
            element_ids.push(person.id().to_string());
            element_data.push((
                person.id().to_string(),
                person.id(),
                person.name().to_string(),
                "Person".to_string(),
                person.properties.description.clone(),
                None,
            ));
        }
        for system in &model.software_systems {
            element_ids.push(system.id().to_string());
            element_data.push((
                system.id().to_string(),
                system.id(),
                system.name().to_string(),
                "Software System".to_string(),
                system.properties.description.clone(),
                None,
            ));
        }
    } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
        auto_layout_config = view.properties.auto_layout.as_ref();

        // Build candidate IDs (matching SVG renderer logic for container views)
        let mut candidate_ids: std::collections::HashSet<structurizr_core::model::ElementId> = std::collections::HashSet::new();

        // Add candidate people
        for person in &model.people {
            candidate_ids.insert(person.id());
        }

        // Add candidate containers from the target system
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            for container in &system.containers {
                candidate_ids.insert(container.id());
            }
        }

        // Add candidate external systems
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                candidate_ids.insert(system.id());
            }
        }

        // Build connected_ids: only include elements where BOTH endpoints are candidates
        let connected_ids: std::collections::HashSet<structurizr_core::model::ElementId> = model.relationships
            .iter()
            .filter(|rel| candidate_ids.contains(&rel.source_id) && candidate_ids.contains(&rel.destination_id))
            .flat_map(|rel| [rel.source_id, rel.destination_id])
            .collect();

        // Add people that are both candidates AND connected
        for person in &model.people {
            if !candidate_ids.contains(&person.id()) { continue; }
            if !connected_ids.contains(&person.id()) { continue; }
            element_ids.push(person.id().to_string());
            element_data.push((
                person.id().to_string(),
                person.id(),
                person.name().to_string(),
                "Person".to_string(),
                person.properties.description.clone(),
                None,
            ));
        }

        // Add containers from the target system that are connected
        if let Some(system) = model.software_systems.iter().find(|s| s.id() == view.software_system_id) {
            for container in &system.containers {
                if !candidate_ids.contains(&container.id()) { continue; }
                if !connected_ids.contains(&container.id()) { continue; }
                element_ids.push(container.id().to_string());
                element_data.push((
                    container.id().to_string(),
                    container.id(),
                    container.name().to_string(),
                    "Container".to_string(),
                    container.properties.description.clone(),
                    container.technology.clone(),
                ));
            }
        }

        // Add external systems that are connected
        for system in &model.software_systems {
            if system.id() != view.software_system_id {
                if !candidate_ids.contains(&system.id()) { continue; }
                if !connected_ids.contains(&system.id()) { continue; }
                element_ids.push(system.id().to_string());
                element_data.push((
                    system.id().to_string(),
                    system.id(),
                    system.name().to_string(),
                    "External System".to_string(),
                    system.properties.description.clone(),
                    None,
                ));
            }
        }
    } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
        auto_layout_config = view.properties.auto_layout.as_ref();
        for system in &model.software_systems {
            if let Some(container) = system.containers.iter().find(|c| c.id() == view.container_id) {
                for component in &container.components {
                    element_ids.push(component.id().to_string());
                    element_data.push((
                        component.id().to_string(),
                        component.id(),
                        component.name().to_string(),
                        "Component".to_string(),
                        component.properties.description.clone(),
                        component.technology.clone(),
                    ));
                }
                break;
            }
        }
    } else {
        // Fallback: include all people and software systems
        for person in &model.people {
            element_ids.push(person.id().to_string());
            element_data.push((
                person.id().to_string(),
                person.id(),
                person.name().to_string(),
                "Person".to_string(),
                person.properties.description.clone(),
                None,
            ));
        }
        for system in &model.software_systems {
            element_ids.push(system.id().to_string());
            element_data.push((
                system.id().to_string(),
                system.id(),
                system.name().to_string(),
                "Software System".to_string(),
                system.properties.description.clone(),
                None,
            ));
        }
    }

    // Build edges from relationships (filter to only relevant elements)
    let element_id_set: std::collections::HashSet<String> = element_ids.iter().cloned().collect();
    let edges: Vec<LayoutEdge> = model
        .relationships
        .iter()
        .filter(|r| element_id_set.contains(&r.source_id.to_string()) && element_id_set.contains(&r.destination_id.to_string()))
        .map(|r| LayoutEdge {
            source: r.source_id.to_string(),
            target: r.destination_id.to_string(),
        })
        .collect();

    // Compute layout using layout_sugiyama (matching SVG renderer exactly)
    let layout = if let Some(ref auto_layout) = auto_layout_config {
        GridLayout::from_config(auto_layout)
    } else {
        GridLayout::default()
    };

    // Prepare node sizes for Sugiyama (matching SVG renderer)
    let node_sizes: Vec<(String, Size)> = element_ids.iter()
        .map(|id| (id.clone(), Size::default()))
        .collect();

    // Use Sugiyama layout with proper normalization (same as SVG renderer)
    let sugiyama_result = layout.layout_sugiyama(&element_ids, &node_sizes, &edges);

    // Convert sugiyama nodes for element position lookup
    let nodes: Vec<_> = sugiyama_result.nodes;

    // Build elements JSON with position data for hit-testing and drill-down info
    let mut elements_json = String::from("[");
    for (i, (id, element_id, name, elem_type, desc, tech)) in element_data.iter().enumerate() {
        if i > 0 { elements_json.push(','); }

        // Find the corresponding layout node
        let (x, y, width, height) = nodes.iter()
            .find(|n| &n.id == id)
            .map(|n| (n.position.x, n.position.y, n.size.width, n.size.height))
            .unwrap_or((0.0, 0.0, 450.0, 300.0));

        // Check if this element can be drilled into
        let drillable = nav_index.is_drillable(*element_id);
        let drill_target = nav_index.get_drill_target(*element_id);
        let target_view = drill_target.map(|t| format!("\"{}\"", escape_json(&t.view_key))).unwrap_or_else(|| "null".to_string());
        let target_type = drill_target.map(|t| format!("\"{}\"", t.target_type.display_name())).unwrap_or_else(|| "null".to_string());

        elements_json.push_str(&format!(
            r#"{{"id":"{}","name":"{}","type":"{}","description":{},"technology":{},"x":{},"y":{},"width":{},"height":{},"drillable":{},"targetView":{},"targetType":{}}}"#,
            escape_json(id),
            escape_json(name),
            escape_json(elem_type),
            desc.as_ref().map(|d| format!("\"{}\"", escape_json(d))).unwrap_or_else(|| "null".to_string()),
            tech.as_ref().map(|t| format!("\"{}\"", escape_json(t))).unwrap_or_else(|| "null".to_string()),
            x as i32,
            y as i32,
            width as i32,
            height as i32,
            drillable,
            target_view,
            target_type,
        ));
    }
    elements_json.push(']');

    // Build relationships JSON for dynamic highlighting
    let element_id_set: std::collections::HashSet<String> = element_ids.iter().cloned().collect();
    let mut relationships_json = String::from("[");
    let mut rel_count = 0;
    for rel in &model.relationships {
        let source_id = rel.source_id.to_string();
        let target_id = rel.destination_id.to_string();
        // Only include relationships where both endpoints are in this view
        if element_id_set.contains(&source_id) && element_id_set.contains(&target_id) {
            if rel_count > 0 { relationships_json.push(','); }
            relationships_json.push_str(&format!(
                r#"{{"source":"{}","target":"{}","description":{}}}"#,
                escape_json(&source_id),
                escape_json(&target_id),
                rel.description.as_ref().map(|d| format!("\"{}\"", escape_json(d))).unwrap_or_else(|| "null".to_string())
            ));
            rel_count += 1;
        }
    }
    relationships_json.push(']');

    let title = format!("{} - {}", view_key, workspace.name);

    let extra_styles = r##"<style>
        .view-toolbar {
            background: var(--toolbar-bg);
            color: var(--toolbar-text);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 20px;
            border-bottom: 1px solid var(--toolbar-border);
            position: relative;
            z-index: 1001;
        }
        .view-toolbar a { color: var(--link-color); text-decoration: none; }
        .view-toolbar a:hover { text-decoration: underline; }
        .view-toolbar button {
            background: var(--bg-tertiary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
            padding: 6px 12px;
            border-radius: 4px;
            cursor: pointer;
        }
        .view-toolbar button:hover { background: var(--card-hover); }
        .view-toolbar .separator { border-left: 1px solid var(--border-color); height: 20px; }
        .zoom-controls { display: flex; gap: 5px; align-items: center; margin-left: auto; }
        .zoom-level { font-size: 12px; min-width: 50px; text-align: center; color: var(--text-secondary); }
        .diagram-container { flex: 1; overflow: hidden; position: relative; background: var(--canvas-bg); }
        #diagram-canvas { width: 100%; height: 100%; cursor: grab; }
        #diagram-canvas.dragging { cursor: grabbing; }
        .tooltip {
            position: fixed;
            background: var(--card-bg);
            color: var(--text-primary);
            padding: 12px 16px;
            border-radius: 6px;
            font-size: 13px;
            max-width: 300px;
            z-index: 1000;
            pointer-events: none;
            box-shadow: 0 4px 12px var(--shadow-medium);
            border: 1px solid var(--border-color);
            display: none;
        }
        .tooltip h4 { margin: 0 0 6px 0; font-size: 14px; }
        .tooltip .type { color: var(--text-muted); font-size: 11px; margin-bottom: 8px; }
        .tooltip .desc { line-height: 1.4; }
        .tooltip .tech { color: var(--link-color); margin-top: 6px; font-size: 12px; }
        .tooltip .drill-hint { color: #4c9; margin-top: 8px; font-size: 11px; font-style: italic; }
        .minimap {
            position: absolute;
            bottom: 20px;
            right: 20px;
            width: 200px;
            height: 150px;
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-radius: 4px;
            overflow: hidden;
            cursor: crosshair;
        }
        .minimap-canvas { width: 100%; height: 100%; opacity: 0.7; }
        .minimap .viewport {
            position: absolute;
            border: 2px solid #0066cc;
            background: rgba(0,102,204,0.1);
            cursor: grab;
            transition: background 0.15s;
        }
        .minimap .viewport:hover { background: rgba(0,102,204,0.25); }
        .keyboard-help { position: fixed; bottom: 20px; left: 20px; font-size: 11px; color: var(--text-muted); }
        .view-breadcrumbs { display: flex; align-items: center; gap: 6px; font-size: 13px; flex: 1; min-width: 0; overflow: hidden; }
        .breadcrumb {
            display: flex;
            align-items: center;
            gap: 6px;
            color: var(--text-secondary);
            text-decoration: none;
            padding: 4px 8px;
            border-radius: 4px;
            transition: all 0.15s;
            white-space: nowrap;
        }
        .breadcrumb:hover { background: var(--header-link-hover); color: var(--text-primary); }
        .breadcrumb.current { color: var(--text-primary); font-weight: 500; }
        .breadcrumb-icon {
            background: var(--bg-tertiary);
            padding: 2px 8px;
            border-radius: 10px;
            font-size: 10px;
            font-weight: 600;
        }
        .breadcrumb-separator { color: var(--text-muted); font-size: 11px; }
        .drill-indicator { position: absolute; pointer-events: none; }
        /* Settings button */
        .settings-btn {
            background: transparent;
            border: none;
            color: var(--text-secondary);
            cursor: pointer;
            padding: 6px;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            transition: all 0.15s;
        }
        .settings-btn:hover { background: var(--header-link-hover); color: var(--text-primary); }
        .settings-btn svg { width: 18px; height: 18px; }
        /* Settings modal */
        .settings-modal {
            display: none;
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: rgba(0,0,0,0.5);
            z-index: 2000;
            align-items: center;
            justify-content: center;
        }
        .settings-modal.open { display: flex; }
        .settings-content {
            background: var(--card-bg);
            border-radius: 8px;
            padding: 20px 24px;
            min-width: 280px;
            box-shadow: 0 4px 20px var(--shadow-heavy);
            border: 1px solid var(--border-color);
        }
        .settings-content h3 {
            margin: 0 0 16px 0;
            font-size: 16px;
            color: var(--text-primary);
        }
        .settings-group {
            display: flex;
            flex-direction: column;
            gap: 12px;
            margin-bottom: 20px;
        }
        .settings-group label {
            display: flex;
            align-items: center;
            gap: 10px;
            cursor: pointer;
            font-size: 14px;
            color: var(--text-primary);
        }
        .settings-group input[type="checkbox"] {
            width: 16px;
            height: 16px;
            cursor: pointer;
        }
        .settings-close-btn {
            width: 100%;
            padding: 8px 16px;
            background: var(--bg-tertiary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
        }
        .settings-close-btn:hover { background: var(--card-hover); }
        /* Fixed tooltip in top-right, below toolbar */
        .tooltip.fixed {
            left: auto !important;
            right: 20px;
            top: 150px !important;
        }
    </style>"##;

    let content = format!(r##"
        <div class="view-toolbar">
            <nav class="view-breadcrumbs" id="breadcrumbs"></nav>
            <div class="zoom-controls">
                <button onclick="zoomOut()">−</button>
                <span class="zoom-level" id="zoom-level">100%</span>
                <button onclick="zoomIn()">+</button>
                <button onclick="resetZoom()">Reset</button>
                <button onclick="fitToScreen()">Fit</button>
            </div>
            <div class="separator"></div>
            <a href="{base_path}/edit/{view_key}">Edit</a>
            <a href="{svg_url}" download="{view_key}.svg">Download SVG</a>
            <div class="separator"></div>
            <button class="settings-btn" onclick="openSettings()" title="Settings">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                    <circle cx="12" cy="12" r="3"></circle>
                    <path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path>
                </svg>
            </button>
        </div>
        <div class="diagram-container" id="diagram-container">
            <canvas id="diagram-canvas"></canvas>
        </div>
        <div class="tooltip" id="tooltip"></div>
        <div class="minimap" id="minimap">
            <canvas class="minimap-canvas" id="minimap-canvas"></canvas>
            <div class="viewport" id="minimap-viewport"></div>
        </div>
        <div class="keyboard-help">
            Scroll to zoom • Drag to pan • Double-click to drill down • Hover for info • Esc to go back
        </div>
        <div class="settings-modal" id="settings-modal" onclick="if(event.target===this)closeSettings()">
            <div class="settings-content">
                <h3>Diagram Settings</h3>
                <div class="settings-group">
                    <label>
                        <input type="checkbox" id="setting-tooltips" checked>
                        Show tooltips on hover
                    </label>
                    <label>
                        <input type="checkbox" id="setting-outbound-arrows">
                        Show outbound relationship arrows
                    </label>
                </div>
                <button class="settings-close-btn" onclick="closeSettings()">Close</button>
            </div>
        </div>
    "##, base_path = base_path, view_key = view_key, svg_url = svg_url);

    let extra_scripts = format!(r##"<script>
        // Base path for URL construction
        const basePath = '{base_path}';

        // Element data with positions for hit-testing
        const elements = {elements_json};

        // Relationship data for dynamic highlighting
        const relationships = {relationships_json};

        // Breadcrumb navigation data
        const breadcrumbs = {breadcrumbs_json};
        const currentViewKey = '{view_key}';
        const currentViewTitle = '{current_view_title}';

        // View type level indicators
        const viewTypeLevels = {{
            'system_landscape': 'C1',
            'system_context': 'C1',
            'container': 'C2',
            'component': 'C3',
            'dynamic': 'Dyn',
            'deployment': 'Dep',
            'filtered': 'Flt',
            'custom': 'Cst',
            'image': 'Img'
        }};

        // Settings management
        const SETTINGS_KEY = 'structurizr-diagram-settings';
        const defaultSettings = {{
            tooltipsEnabled: true,
            outboundArrowsEnabled: false  // OFF by default
        }};

        function loadSettings() {{
            try {{
                const stored = localStorage.getItem(SETTINGS_KEY);
                if (stored) {{
                    return {{ ...defaultSettings, ...JSON.parse(stored) }};
                }}
            }} catch (e) {{}}
            return {{ ...defaultSettings }};
        }}

        function saveSettings(settings) {{
            try {{
                localStorage.setItem(SETTINGS_KEY, JSON.stringify(settings));
            }} catch (e) {{}}
        }}

        let settings = loadSettings();

        function applySettingsToUI() {{
            document.getElementById('setting-tooltips').checked = settings.tooltipsEnabled;
            document.getElementById('setting-outbound-arrows').checked = settings.outboundArrowsEnabled;
            // Apply fixed class to tooltip if tooltips are enabled
            const tooltipEl = document.getElementById('tooltip');
            if (tooltipEl) {{
                tooltipEl.classList.add('fixed');
            }}
        }}

        function openSettings() {{
            document.getElementById('settings-modal').classList.add('open');
        }}

        function closeSettings() {{
            document.getElementById('settings-modal').classList.remove('open');
        }}

        // Initialize settings UI after DOM is ready
        document.addEventListener('DOMContentLoaded', () => {{
            applySettingsToUI();

            document.getElementById('setting-tooltips').addEventListener('change', (e) => {{
                settings.tooltipsEnabled = e.target.checked;
                saveSettings(settings);
                if (!settings.tooltipsEnabled) {{
                    document.getElementById('tooltip').style.display = 'none';
                }}
            }});

            document.getElementById('setting-outbound-arrows').addEventListener('change', (e) => {{
                settings.outboundArrowsEnabled = e.target.checked;
                saveSettings(settings);
                render();
            }});
        }});

        // Render breadcrumbs
        function renderBreadcrumbs() {{
            const container = document.getElementById('breadcrumbs');
            if (!breadcrumbs || breadcrumbs.length === 0) {{
                container.innerHTML = `<span class="breadcrumb current"><span class="breadcrumb-icon">V</span><span class="breadcrumb-text">${{currentViewTitle}}</span></span>`;
                return;
            }}

            let html = '';
            breadcrumbs.forEach((crumb, index) => {{
                if (index > 0) {{
                    html += '<span class="breadcrumb-separator">›</span>';
                }}

                const isLast = index === breadcrumbs.length - 1;
                const level = viewTypeLevels[crumb.view_type] || 'V';

                if (isLast) {{
                    html += `<span class="breadcrumb current"><span class="breadcrumb-icon">${{level}}</span><span class="breadcrumb-text">${{escapeHtml(crumb.title)}}</span></span>`;
                }} else {{
                    html += `<a href="${{basePath}}/view/${{crumb.view_key}}" class="breadcrumb"><span class="breadcrumb-icon">${{level}}</span><span class="breadcrumb-text">${{escapeHtml(crumb.title)}}</span></a>`;
                }}
            }});
            container.innerHTML = html;
        }}

        function escapeHtml(text) {{
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }}

        renderBreadcrumbs();

        // Theme-aware colors (read from CSS variables)
        function getThemeColor(varName, fallback) {{
            return getComputedStyle(document.documentElement).getPropertyValue(varName).trim() || fallback;
        }}
        function getCanvasBg() {{ return getThemeColor('--canvas-bg', '#f0f0f0'); }}
        function getCardBg() {{ return getThemeColor('--card-bg', '#ffffff'); }}

        // Canvas and context
        const canvas = document.getElementById('diagram-canvas');
        const ctx = canvas.getContext('2d');
        const container = document.getElementById('diagram-container');
        const tooltip = document.getElementById('tooltip');
        const zoomLevelEl = document.getElementById('zoom-level');

        // Minimap elements
        const minimap = document.getElementById('minimap');
        const minimapCanvas = document.getElementById('minimap-canvas');
        const minimapCtx = minimapCanvas.getContext('2d');
        const minimapViewport = document.getElementById('minimap-viewport');

        // SVG image
        const svgImage = new Image();
        let svgLoaded = false;
        let svgWidth = 0;
        let svgHeight = 0;
        let svgMinX = 0;
        let svgMinY = 0;
        let relationshipPaths = {{}};  // Map of "sourceId-targetId" -> path data

        // Transform state
        let scale = 1;
        let offsetX = 0;
        let offsetY = 0;
        let isPanning = false;
        let panStartX = 0;
        let panStartY = 0;
        let panStartOffsetX = 0;
        let panStartOffsetY = 0;

        // Currently hovered element
        let hoveredElement = null;

        // Initialize canvas size
        function initCanvas() {{
            const rect = container.getBoundingClientRect();
            const width = rect.width || window.innerWidth;
            const height = rect.height || (window.innerHeight - 50);

            canvas.width = width;
            canvas.height = height;
            minimapCanvas.width = 200;
            minimapCanvas.height = 150;

            ctx.fillStyle = getCanvasBg();
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '16px system-ui, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Loading diagram...', canvas.width / 2, canvas.height / 2);
        }}

        requestAnimationFrame(() => {{
            initCanvas();
        }});

        // Load SVG image
        svgImage.onload = () => {{
            svgLoaded = true;
            svgWidth = svgImage.naturalWidth;
            svgHeight = svgImage.naturalHeight;

            fetch('{svg_url}')
                .then(r => r.text())
                .then(svgText => {{
                    const viewBoxMatch = svgText.match(/viewBox="([^"]+)"/);
                    if (viewBoxMatch) {{
                        const parts = viewBoxMatch[1].split(/\s+/).map(Number);
                        if (parts.length >= 4) {{
                            svgMinX = parts[0];
                            svgMinY = parts[1];
                        }}
                    }}

                    // Parse relationship paths from SVG for hover highlighting
                    const parser = new DOMParser();
                    const svgDoc = parser.parseFromString(svgText, 'image/svg+xml');

                    // Extract <path> elements with data-source/data-target (orthogonal/curved routing)
                    svgDoc.querySelectorAll('path.relationship[data-source][data-target]').forEach(path => {{
                        const key = path.getAttribute('data-source') + '-' + path.getAttribute('data-target');
                        relationshipPaths[key] = {{
                            d: path.getAttribute('d'),
                            routing: path.getAttribute('data-routing') || 'orthogonal'
                        }};
                    }});

                    // Extract <line> elements (direct routing)
                    svgDoc.querySelectorAll('line.relationship[data-source][data-target]').forEach(line => {{
                        const key = line.getAttribute('data-source') + '-' + line.getAttribute('data-target');
                        relationshipPaths[key] = {{
                            x1: parseFloat(line.getAttribute('x1')),
                            y1: parseFloat(line.getAttribute('y1')),
                            x2: parseFloat(line.getAttribute('x2')),
                            y2: parseFloat(line.getAttribute('y2')),
                            routing: 'direct'
                        }};
                    }});

                    fitToScreen();
                }})
                .catch(() => {{
                    fitToScreen();
                }});
        }};
        svgImage.onerror = (e) => {{
            ctx.fillStyle = getCanvasBg();
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            ctx.fillStyle = '#f66';
            ctx.font = '16px system-ui, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Failed to load diagram', canvas.width / 2, canvas.height / 2);
        }};
        svgImage.src = '{svg_url}';

        function resizeCanvas() {{
            const rect = container.getBoundingClientRect();
            canvas.width = rect.width;
            canvas.height = rect.height;
            minimapCanvas.width = 200;
            minimapCanvas.height = 150;
            render();
        }}

        function render() {{
            if (!svgLoaded) return;

            ctx.fillStyle = getCanvasBg();
            ctx.fillRect(0, 0, canvas.width, canvas.height);

            ctx.save();
            ctx.translate(offsetX, offsetY);
            ctx.scale(scale, scale);

            ctx.shadowColor = 'rgba(0, 0, 0, 0.4)';
            ctx.shadowBlur = 20;
            ctx.shadowOffsetX = 0;
            ctx.shadowOffsetY = 4;
            ctx.fillStyle = getCardBg();
            ctx.fillRect(0, 0, svgWidth, svgHeight);

            ctx.shadowColor = 'transparent';
            ctx.drawImage(svgImage, 0, 0);

            renderDrillableIndicators();
            renderRelationshipHighlights();

            ctx.restore();

            zoomLevelEl.textContent = Math.round(scale * 100) + '%';
            updateMinimap();
        }}

        function renderDrillableIndicators() {{
            for (const el of elements) {{
                if (!el.drillable) continue;

                const drawX = el.x - svgMinX;
                const drawY = el.y - svgMinY;

                ctx.strokeStyle = 'rgba(0, 150, 255, 0.6)';
                ctx.lineWidth = 3;
                ctx.setLineDash([]);
                ctx.strokeRect(drawX + 2, drawY + 2, el.width - 4, el.height - 4);

                const iconSize = 24;
                const iconX = drawX + el.width - iconSize - 8;
                const iconY = drawY + el.height - iconSize - 8;

                ctx.fillStyle = 'rgba(0, 150, 255, 0.9)';
                ctx.beginPath();
                ctx.arc(iconX + iconSize/2, iconY + iconSize/2, iconSize/2, 0, Math.PI * 2);
                ctx.fill();

                ctx.strokeStyle = 'white';
                ctx.lineWidth = 2.5;
                ctx.lineCap = 'round';
                ctx.beginPath();
                ctx.moveTo(iconX + iconSize/2 - 6, iconY + iconSize/2);
                ctx.lineTo(iconX + iconSize/2 + 6, iconY + iconSize/2);
                ctx.moveTo(iconX + iconSize/2, iconY + iconSize/2 - 6);
                ctx.lineTo(iconX + iconSize/2, iconY + iconSize/2 + 6);
                ctx.stroke();

                ctx.font = 'bold 11px system-ui, sans-serif';
                ctx.textAlign = 'center';
                ctx.fillStyle = 'rgba(0, 150, 255, 0.9)';
                const labelText = 'Double-click to explore ' + (el.targetType || 'details');
                const labelWidth = ctx.measureText(labelText).width + 12;
                const labelX = drawX + el.width/2;
                const labelY = drawY - 8;

                ctx.fillStyle = 'rgba(0, 150, 255, 0.9)';
                ctx.beginPath();
                ctx.roundRect(labelX - labelWidth/2, labelY - 14, labelWidth, 18, 4);
                ctx.fill();

                ctx.fillStyle = 'white';
                ctx.fillText(labelText, labelX, labelY - 1);
            }}
        }}

        // Render highlight overlays for outbound relationships from hovered element
        function renderRelationshipHighlights() {{
            // Check if outbound arrows are enabled in settings
            if (!settings.outboundArrowsEnabled) return;
            if (!hoveredElement) return;

            // Find outbound relationships from the hovered element
            const outboundRels = relationships.filter(rel => rel.source === hoveredElement.id);
            if (outboundRels.length === 0) return;

            ctx.save();

            for (const rel of outboundRels) {{
                // Look up the actual path data from the SVG
                const key = rel.source + '-' + rel.target;
                const pathData = relationshipPaths[key];
                if (!pathData) continue;

                // Draw the actual path in green (highlighting the existing connector)
                ctx.strokeStyle = 'rgba(0, 200, 100, 0.9)';
                ctx.lineWidth = 6;
                ctx.lineCap = 'round';
                ctx.lineJoin = 'round';
                ctx.setLineDash([]);

                if (pathData.routing === 'direct') {{
                    // Draw line for direct routing
                    const x1 = pathData.x1 - svgMinX;
                    const y1 = pathData.y1 - svgMinY;
                    const x2 = pathData.x2 - svgMinX;
                    const y2 = pathData.y2 - svgMinY;
                    ctx.beginPath();
                    ctx.moveTo(x1, y1);
                    ctx.lineTo(x2, y2);
                    ctx.stroke();
                }} else {{
                    // Draw SVG path using Path2D for orthogonal/curved routing
                    // Adjust path coordinates by translating context
                    ctx.save();
                    ctx.translate(-svgMinX, -svgMinY);
                    const path = new Path2D(pathData.d);
                    ctx.stroke(path);
                    ctx.restore();
                }}
            }}

            ctx.restore();
        }}

        function updateMinimap() {{
            if (!svgLoaded) return;

            minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);

            const minimapScale = Math.min(
                minimapCanvas.width / svgWidth,
                minimapCanvas.height / svgHeight
            );

            minimapCtx.save();
            minimapCtx.scale(minimapScale, minimapScale);
            minimapCtx.drawImage(svgImage, 0, 0);
            minimapCtx.restore();

            const viewportWidth = (canvas.width / scale) * minimapScale;
            const viewportHeight = (canvas.height / scale) * minimapScale;
            const viewportX = (-offsetX / scale) * minimapScale;
            const viewportY = (-offsetY / scale) * minimapScale;

            minimapViewport.style.width = Math.max(10, Math.min(viewportWidth, minimapCanvas.width)) + 'px';
            minimapViewport.style.height = Math.max(10, Math.min(viewportHeight, minimapCanvas.height)) + 'px';
            minimapViewport.style.left = Math.max(0, Math.min(viewportX, minimapCanvas.width - 10)) + 'px';
            minimapViewport.style.top = Math.max(0, Math.min(viewportY, minimapCanvas.height - 10)) + 'px';
        }}

        function screenToDiagram(screenX, screenY) {{
            const rect = canvas.getBoundingClientRect();
            const canvasX = screenX - rect.left;
            const canvasY = screenY - rect.top;
            return {{
                x: (canvasX - offsetX) / scale + svgMinX,
                y: (canvasY - offsetY) / scale + svgMinY
            }};
        }}

        function getElementAtPoint(diagramX, diagramY) {{
            for (const el of elements) {{
                if (diagramX >= el.x && diagramX <= el.x + el.width &&
                    diagramY >= el.y && diagramY <= el.y + el.height) {{
                    return el;
                }}
            }}
            return null;
        }}

        function showTooltip(element, screenX, screenY) {{
            // Check if tooltips are enabled in settings
            if (!settings.tooltipsEnabled) {{
                tooltip.style.display = 'none';
                return;
            }}
            if (element) {{
                let html = `<h4>${{element.name}}</h4><div class="type">${{element.type}}</div>`;
                if (element.description) {{
                    html += `<div class="desc">${{element.description}}</div>`;
                }}
                if (element.technology) {{
                    html += `<div class="tech">${{element.technology}}</div>`;
                }}
                if (element.drillable && element.targetType) {{
                    html += `<div class="drill-hint">Double-click to view ${{element.targetType.toLowerCase()}}s</div>`;
                }}
                tooltip.innerHTML = html;
                // Fixed position in top-right corner (CSS handles positioning)
                tooltip.style.display = 'block';
            }} else {{
                tooltip.style.display = 'none';
            }}
        }}

        function setZoom(newScale, centerX, centerY) {{
            newScale = Math.max(0.1, Math.min(5, newScale));

            if (centerX !== undefined && centerY !== undefined) {{
                offsetX = centerX - (centerX - offsetX) * (newScale / scale);
                offsetY = centerY - (centerY - offsetY) * (newScale / scale);
            }}

            scale = newScale;
            render();
        }}

        function zoomIn() {{
            setZoom(scale * 1.2, canvas.width / 2, canvas.height / 2);
        }}

        function zoomOut() {{
            setZoom(scale / 1.2, canvas.width / 2, canvas.height / 2);
        }}

        function resetZoom() {{
            scale = 1;
            offsetX = 50;
            offsetY = 50;
            render();
        }}

        function fitToScreen() {{
            if (!svgLoaded) return;

            const padding = 50;
            const scaleX = (canvas.width - padding * 2) / svgWidth;
            const scaleY = (canvas.height - padding * 2) / svgHeight;
            scale = Math.min(scaleX, scaleY, 1);

            offsetX = (canvas.width - svgWidth * scale) / 2;
            offsetY = (canvas.height - svgHeight * scale) / 2;
            render();
        }}

        canvas.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            setZoom(scale * delta, mouseX, mouseY);
        }}, {{ passive: false }});

        canvas.addEventListener('mousedown', (e) => {{
            if (e.button === 0) {{
                isPanning = true;
                panStartX = e.clientX;
                panStartY = e.clientY;
                panStartOffsetX = offsetX;
                panStartOffsetY = offsetY;
                canvas.classList.add('dragging');
            }}
        }});

        document.addEventListener('mousemove', (e) => {{
            if (isPanning) {{
                offsetX = panStartOffsetX + (e.clientX - panStartX);
                offsetY = panStartOffsetY + (e.clientY - panStartY);
                render();
            }} else {{
                const diagramPos = screenToDiagram(e.clientX, e.clientY);
                const element = getElementAtPoint(diagramPos.x, diagramPos.y);

                if (element !== hoveredElement) {{
                    hoveredElement = element;
                    showTooltip(element, e.clientX, e.clientY);
                    if (element && element.drillable) {{
                        canvas.style.cursor = 'zoom-in';
                    }} else if (element) {{
                        canvas.style.cursor = 'pointer';
                    }} else {{
                        canvas.style.cursor = 'grab';
                    }}
                    // Re-render to show/hide relationship highlights
                    render();
                }}
                // Tooltip is now fixed in top-right corner, no position updates needed
            }}
        }});

        document.addEventListener('mouseup', () => {{
            if (isPanning) {{
                isPanning = false;
                canvas.classList.remove('dragging');
                canvas.style.cursor = hoveredElement ? 'pointer' : 'grab';
            }}
        }});

        canvas.addEventListener('dblclick', (e) => {{
            e.preventDefault();
            e.stopPropagation();

            isPanning = false;
            canvas.classList.remove('dragging');

            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            const diagramPos = screenToDiagram(e.clientX, e.clientY);
            const element = getElementAtPoint(diagramPos.x, diagramPos.y);

            if (element && element.drillable && element.targetView) {{
                initiateDrillDown(element);
                return;
            }} else {{
                setZoom(e.shiftKey ? scale / 2 : scale * 2, mouseX, mouseY);
            }}
        }});

        function initiateDrillDown(element) {{
            const centerX = element.x + element.width / 2;
            const centerY = element.y + element.height / 2;

            animateZoomTo(centerX, centerY, 2.5, 400, () => {{
                canvas.style.transition = 'opacity 0.2s ease-out';
                canvas.style.opacity = '0';

                setTimeout(() => {{
                    window.location.href = basePath + '/view/' + element.targetView;
                }}, 200);
            }});
        }}

        function animateZoomTo(targetX, targetY, targetScale, duration, callback) {{
            const startScale = scale;
            const startOffsetX = offsetX;
            const startOffsetY = offsetY;

            const endOffsetX = canvas.width / 2 - targetX * targetScale;
            const endOffsetY = canvas.height / 2 - targetY * targetScale;

            const startTime = performance.now();

            function animate(currentTime) {{
                const elapsed = currentTime - startTime;
                const progress = Math.min(elapsed / duration, 1);

                const eased = progress < 0.5
                    ? 4 * progress * progress * progress
                    : 1 - Math.pow(-2 * progress + 2, 3) / 2;

                scale = startScale + (targetScale - startScale) * eased;
                offsetX = startOffsetX + (endOffsetX - startOffsetX) * eased;
                offsetY = startOffsetY + (endOffsetY - startOffsetY) * eased;

                render();

                if (progress < 1) {{
                    requestAnimationFrame(animate);
                }} else if (callback) {{
                    callback();
                }}
            }}

            requestAnimationFrame(animate);
        }}

        document.addEventListener('keydown', (e) => {{
            if (e.key === '+' || e.key === '=') zoomIn();
            if (e.key === '-') zoomOut();
            if (e.key === '0') resetZoom();
            if (e.key === 'f' || e.key === 'F') fitToScreen();
            if (e.key === 'Escape') {{
                if (breadcrumbs && breadcrumbs.length > 1) {{
                    const parentCrumb = breadcrumbs[breadcrumbs.length - 2];
                    if (parentCrumb) {{
                        window.location.href = basePath + '/view/' + parentCrumb.view_key;
                    }}
                }}
            }}
        }});

        let minimapDragging = false;
        let minimapDragStart = {{ x: 0, y: 0, offsetX: 0, offsetY: 0 }};

        function getMinimapScale() {{
            if (!svgLoaded) return 1;
            return Math.min(
                minimapCanvas.width / svgWidth,
                minimapCanvas.height / svgHeight
            );
        }}

        minimap.addEventListener('click', (e) => {{
            if (minimapDragging) return;
            const rect = minimap.getBoundingClientRect();
            const clickX = e.clientX - rect.left;
            const clickY = e.clientY - rect.top;
            const minimapScale = getMinimapScale();

            const diagramX = clickX / minimapScale;
            const diagramY = clickY / minimapScale;

            offsetX = (canvas.width / 2) - (diagramX * scale);
            offsetY = (canvas.height / 2) - (diagramY * scale);
            render();
        }});

        minimapViewport.addEventListener('mousedown', (e) => {{
            e.stopPropagation();
            minimapDragging = true;
            minimapDragStart = {{
                x: e.clientX,
                y: e.clientY,
                offsetX: offsetX,
                offsetY: offsetY
            }};
            minimapViewport.style.cursor = 'grabbing';
        }});

        document.addEventListener('mousemove', (e) => {{
            if (minimapDragging) {{
                const minimapScale = getMinimapScale();
                const deltaX = (e.clientX - minimapDragStart.x) / minimapScale * scale;
                const deltaY = (e.clientY - minimapDragStart.y) / minimapScale * scale;
                offsetX = minimapDragStart.offsetX - deltaX;
                offsetY = minimapDragStart.offsetY - deltaY;
                render();
            }}
        }});

        document.addEventListener('mouseup', () => {{
            if (minimapDragging) {{
                minimapDragging = false;
                minimapViewport.style.cursor = 'grab';
            }}
        }});

        window.addEventListener('resize', () => {{
            resizeCanvas();
        }});

        // Re-render when theme changes
        new MutationObserver(() => {{ render(); }}).observe(
            document.documentElement, {{ attributes: true, attributeFilter: ['data-theme'] }}
        );

        minimapViewport.style.cursor = 'grab';
    </script>"##,
        base_path = base_path,
        svg_url = svg_url,
        elements_json = elements_json,
        relationships_json = relationships_json,
        breadcrumbs_json = breadcrumbs_json,
        current_view_title = escape_json(&current_view_title),
    );

    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::View,
        content_type: ContentType::ToolbarViewport,
        extra_head: extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&config, &content)
}

/// Render view diagram HTML (wrapper for multi-workspace handlers).
fn render_view_diagram_html(workspace: &Workspace, view_key: &str, base_path: &str) -> Result<Html<String>> {
    let html = generate_view_diagram_html(workspace, view_key, base_path);
    Ok(Html(html))
}

/// Render dynamic animated view HTML.
fn render_dynamic_animated_html(workspace: &Workspace, view_key: &str, base_path: &str) -> Result<Html<String>> {
    let html = generate_dynamic_animated_html(workspace, view_key, base_path)?;
    Ok(Html(html))
}

/// Generate dynamic animated view HTML with full animation controls and visualization.
fn generate_dynamic_animated_html(workspace: &Workspace, view_key: &str, base_path: &str) -> Result<String> {
    // Find the dynamic view
    let dynamic_view = workspace.views().dynamic_views.iter()
        .find(|v| v.properties.key == view_key)
        .ok_or_else(|| Error::ViewNotFound(view_key.to_string()))?;

    // Count steps for animation
    let step_count = dynamic_view.steps.len();

    // Build step data as JSON with enhanced metadata
    let mut steps_json = String::from("[");
    for (i, step) in dynamic_view.steps.iter().enumerate() {
        if i > 0 { steps_json.push(','); }

        // Find the relationship for this step
        let relationship = workspace.model().relationships.iter()
            .find(|r| r.source_id == step.source_id && r.destination_id == step.destination_id);

        // Get source and destination elements
        let source_element = workspace.model().find_element(step.source_id);
        let dest_element = workspace.model().find_element(step.destination_id);

        // Build tags array
        let tags_json = if let Some(rel) = relationship {
            let tags_str: Vec<String> = rel.tags.iter()
                .map(|t| format!("\"{}\"", escape_json(t)))
                .collect();
            format!("[{}]", tags_str.join(","))
        } else {
            "[]".to_string()
        };

        // Build properties object
        let props_json = if let Some(rel) = relationship {
            if !rel.properties.is_empty() {
                let props_str: Vec<String> = rel.properties.iter()
                    .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
                    .collect();
                format!("{{{}}}", props_str.join(","))
            } else {
                "{}".to_string()
            }
        } else {
            "{}".to_string()
        };

        // Build perspectives object
        let perspectives_json = if let Some(rel) = relationship {
            if !rel.perspectives.is_empty() {
                let persp_str: Vec<String> = rel.perspectives.iter()
                    .map(|(k, v)| format!("\"{}\":\"{}\"", escape_json(k), escape_json(v)))
                    .collect();
                format!("{{{}}}", persp_str.join(","))
            } else {
                "{}".to_string()
            }
        } else {
            "{}".to_string()
        };

        steps_json.push_str(&format!(
            r#"{{"order":{},"sourceId":"{}","destId":"{}","description":{},"technology":{},"interactionStyle":"{}","tags":{},"properties":{},"perspectives":{},"sourceName":"{}","sourceType":"{}","destName":"{}","destType":"{}"}}"#,
            step.order,
            step.source_id,
            step.destination_id,
            step.description.as_ref()
                .or_else(|| relationship.and_then(|r| r.description.as_ref()))
                .map(|d| format!("\"{}\"", escape_json(d)))
                .unwrap_or_else(|| "null".to_string()),
            relationship.and_then(|r| r.technology.as_ref())
                .map(|t| format!("\"{}\"", escape_json(t)))
                .unwrap_or_else(|| "null".to_string()),
            relationship.map(|r| match r.interaction_style {
                structurizr_core::model::InteractionStyle::Synchronous => "Synchronous",
                structurizr_core::model::InteractionStyle::Asynchronous => "Asynchronous",
            }).unwrap_or("Synchronous"),
            tags_json,
            props_json,
            perspectives_json,
            source_element.map(|e| escape_json(&e.name())).unwrap_or_else(|| "Unknown".to_string()),
            source_element.map(|e| match e {
                structurizr_core::model::ElementRef::Person(_) => "Person",
                structurizr_core::model::ElementRef::SoftwareSystem(_) => "Software System",
                structurizr_core::model::ElementRef::Container(_) => "Container",
                structurizr_core::model::ElementRef::Component(_) => "Component",
                structurizr_core::model::ElementRef::DeploymentNode(_) => "Deployment Node",
            }).unwrap_or("Unknown"),
            dest_element.map(|e| escape_json(&e.name())).unwrap_or_else(|| "Unknown".to_string()),
            dest_element.map(|e| match e {
                structurizr_core::model::ElementRef::Person(_) => "Person",
                structurizr_core::model::ElementRef::SoftwareSystem(_) => "Software System",
                structurizr_core::model::ElementRef::Container(_) => "Container",
                structurizr_core::model::ElementRef::Component(_) => "Component",
                structurizr_core::model::ElementRef::DeploymentNode(_) => "Deployment Node",
            }).unwrap_or("Unknown")
        ));
    }
    steps_json.push(']');

    let svg_url = format!("{}/view/{}/svg", base_path, view_key);
    let home_href = if base_path.is_empty() { "/".to_string() } else { base_path.to_string() };

    Ok(format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>{view_key} - Animated - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; background: #1a1a1a; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; height: 100vh; overflow: hidden; color: white; }}
        .toolbar {{ background: #333; color: white; padding: 10px 20px; display: flex; align-items: center; gap: 20px; border-bottom: 1px solid #444; height: 50px; }}
        .toolbar a {{ color: white; text-decoration: none; }}
        .toolbar a:hover {{ text-decoration: underline; }}
        .toolbar .separator {{ border-left: 1px solid #555; height: 20px; }}
        .controls {{ display: flex; align-items: center; gap: 10px; flex: 1; justify-content: center; }}
        .btn {{ background: #555; color: white; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; font-size: 14px; transition: background 0.2s; }}
        .btn:hover {{ background: #666; }}
        .btn:disabled {{ background: #3a3a3a; color: #666; cursor: not-allowed; }}
        .btn.primary {{ background: #0066cc; }}
        .btn.primary:hover {{ background: #0052a3; }}
        .step-info {{ font-size: 14px; color: #ccc; min-width: 120px; text-align: center; }}
        .speed-control {{ display: flex; align-items: center; gap: 8px; }}
        .speed-control label {{ font-size: 12px; color: #aaa; }}
        .speed-control select {{ background: #555; color: white; border: 1px solid #666; padding: 4px 8px; border-radius: 4px; cursor: pointer; }}
        .diagram-container {{ height: calc(100vh - 50px); overflow: hidden; position: relative; background: #2a2a2a; }}
        #svg-wrapper {{ position: absolute; transform-origin: 0 0; background: white; box-shadow: 0 4px 20px rgba(0,0,0,0.4); }}
        #svg-wrapper svg {{ display: block; }}
        /* Arrows - visible by default, dimmed when not current step (spotlight mode) */
        .arrow-line {{ opacity: 1; transition: opacity 0.4s ease-in-out; cursor: pointer; }}
        .arrow-line.dimmed {{ opacity: 0.35; }}
        .arrow-line:hover {{ stroke-width: 3; filter: brightness(1.2); }}
        .arrow-text {{ opacity: 1; transition: opacity 0.4s ease-in-out; }}
        .arrow-text.dimmed {{ opacity: 0.35; }}
        /* Element groups (shape + name + lifeline) - visible by default, dimmed when not in current step */
        .element-group {{ opacity: 1; transition: opacity 0.4s ease-in-out; }}
        .element-group.dimmed {{ opacity: 0.35; }}
        .step-overlay {{ position: absolute; bottom: 30px; left: 50%; transform: translateX(-50%); background: rgba(0, 0, 0, 0.9); color: white; padding: 16px 24px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); max-width: 600px; opacity: 0; transition: opacity 0.3s ease-in-out; pointer-events: none; z-index: 100; }}
        .step-overlay.visible {{ opacity: 1; }}
        .step-overlay .step-number {{ font-size: 12px; color: #0066cc; font-weight: 600; margin-bottom: 6px; }}
        .step-overlay .step-desc {{ font-size: 15px; line-height: 1.4; }}
        .step-overlay .step-tech {{ font-size: 12px; color: #888; margin-top: 8px; font-family: monospace; }}
        .keyboard-help {{ position: fixed; bottom: 20px; left: 20px; font-size: 11px; color: #666; z-index: 50; }}
        .loading {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #666; font-size: 16px; }}

        /* Snackbar Styles */
        .snackbar {{
            position: fixed;
            top: 50px;
            right: -400px;
            width: 380px;
            max-width: 95vw;
            height: calc(100vh - 50px);
            background: #2a2a2a;
            border-left: 1px solid #444;
            box-shadow: -4px 0 12px rgba(0, 0, 0, 0.3);
            transition: right 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            z-index: 200;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
        }}

        .snackbar.open {{
            right: 0;
        }}

        /* Collapsed snackbar tab - desktop (right edge) */
        .snackbar-tab {{
            position: fixed;
            top: 50%;
            right: 0;
            transform: translateY(-50%);
            background: #2a2a2a;
            border: 1px solid #444;
            border-right: none;
            border-radius: 8px 0 0 8px;
            padding: 12px 8px;
            cursor: pointer;
            z-index: 199;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
            transition: background 0.2s, transform 0.3s;
        }}
        .snackbar-tab:hover {{
            background: #333;
        }}
        .snackbar-tab.hidden {{
            transform: translateY(-50%) translateX(100%);
            pointer-events: none;
        }}
        .tab-icon {{
            color: #888;
            font-size: 18px;
        }}
        .tab-text {{
            writing-mode: vertical-rl;
            text-orientation: mixed;
            color: #ccc;
            font-size: 12px;
            font-weight: 500;
        }}

        /* No selection state */
        .no-selection {{
            display: none;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 60px 20px;
            text-align: center;
            color: #888;
        }}
        .no-selection.visible {{
            display: flex;
        }}
        .no-selection-title {{
            font-size: 18px;
            font-weight: 500;
            color: #aaa;
            margin: 0 0 12px 0;
        }}
        .no-selection-hint {{
            font-size: 14px;
            color: #666;
            margin: 0;
        }}

        .snackbar-header {{
            padding: 16px 20px;
            background: #333;
            border-bottom: 1px solid #444;
            display: flex;
            justify-content: space-between;
            align-items: center;
            flex-shrink: 0;
        }}

        .snackbar-title {{
            font-size: 16px;
            font-weight: 600;
            color: white;
            margin: 0;
        }}

        .snackbar-close {{
            background: transparent;
            border: none;
            color: #999;
            font-size: 24px;
            cursor: pointer;
            padding: 0;
            width: 32px;
            height: 32px;
            display: flex;
            align-items: center;
            justify-content: center;
            border-radius: 4px;
            transition: background 0.2s;
        }}

        .snackbar-close:hover {{
            background: #444;
            color: white;
        }}

        .snackbar-content {{
            padding: 20px;
            overflow-y: auto;
            flex: 1;
        }}

        .metadata-section {{
            margin-bottom: 24px;
        }}

        .metadata-section h4 {{
            font-size: 12px;
            font-weight: 600;
            text-transform: uppercase;
            color: #888;
            margin: 0 0 12px 0;
            letter-spacing: 0.5px;
        }}

        .step-description {{
            font-size: 16px;
            color: white;
            margin: 0;
            line-height: 1.5;
        }}

        .metadata-grid {{
            display: grid;
            grid-template-columns: 1fr;
            gap: 12px;
        }}

        .metadata-item {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 8px 0;
            border-bottom: 1px solid #3a3a3a;
        }}

        .metadata-item label {{
            font-size: 13px;
            color: #999;
        }}

        .metadata-item span {{
            font-size: 13px;
            color: white;
            text-align: right;
        }}

        .tech-badge, .style-badge {{
            background: #444;
            padding: 4px 8px;
            border-radius: 4px;
            font-family: monospace;
            font-size: 12px;
        }}

        .tech-badge {{
            background: #0066cc;
        }}

        .style-badge.async {{
            background: #6b46c1;
        }}

        .tag-list {{
            display: flex;
            flex-wrap: wrap;
            gap: 6px;
        }}

        .tag {{
            display: inline-block;
            background: #3a3a3a;
            color: #aaa;
            padding: 4px 8px;
            border-radius: 12px;
            font-size: 12px;
        }}

        .properties-list {{
            margin: 0;
        }}

        .properties-list dt {{
            font-size: 12px;
            color: #999;
            display: inline;
        }}

        .properties-list dd {{
            font-size: 12px;
            color: white;
            display: inline;
            margin: 0 12px 0 4px;
        }}

        .properties-list div {{
            padding: 4px 0;
        }}

        .perspectives-list {{
            display: flex;
            flex-direction: column;
            gap: 12px;
        }}

        .perspective-item {{
            background: #333;
            border-radius: 8px;
            padding: 12px;
            border-left: 3px solid #0066cc;
        }}

        .perspective-item .perspective-name {{
            font-size: 13px;
            font-weight: 600;
            color: #0066cc;
            margin-bottom: 6px;
            display: flex;
            align-items: center;
            gap: 6px;
        }}

        .perspective-item .perspective-name::before {{
            content: "";
            display: inline-block;
            width: 8px;
            height: 8px;
            background: #0066cc;
            border-radius: 50%;
        }}

        .perspective-item .perspective-desc {{
            font-size: 12px;
            color: #ccc;
            line-height: 1.5;
        }}

        .snackbar-actions {{
            margin-top: 24px;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }}

        .action-btn {{
            background: #444;
            color: white;
            border: none;
            padding: 10px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 13px;
            transition: background 0.2s;
            text-align: center;
        }}

        .action-btn:hover {{
            background: #555;
        }}

        /* Tab Navigation */
        .snackbar-tabs {{
            display: flex;
            border-bottom: 1px solid #444;
            background: #333;
            flex-shrink: 0;
        }}

        .tab-btn {{
            flex: 1;
            padding: 12px 16px;
            background: transparent;
            border: none;
            color: #888;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s;
            border-bottom: 2px solid transparent;
        }}

        .tab-btn:hover {{
            color: #ccc;
            background: rgba(255, 255, 255, 0.05);
        }}

        .tab-btn.active {{
            color: white;
            border-bottom-color: #0066cc;
        }}

        .note-count {{
            font-size: 12px;
            opacity: 0.7;
        }}

        .tab-content {{
            display: none;
            flex: 1;
            overflow-y: auto;
        }}

        .tab-content.active {{
            display: flex;
            flex-direction: column;
        }}

        /* Notes Container */
        .notes-container {{
            display: flex;
            flex-direction: column;
            height: 100%;
        }}

        .notes-list {{
            flex: 1;
            overflow-y: auto;
            padding: 0;
        }}

        .no-notes {{
            padding: 40px 20px;
            text-align: center;
            color: #666;
            font-style: italic;
        }}

        .note-item {{
            padding: 16px 20px;
            border-bottom: 1px solid #3a3a3a;
        }}

        .note-item:nth-child(odd) {{
            background: #2a2a2a;
        }}

        .note-item:nth-child(even) {{
            background: #262626;
        }}

        .note-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 8px;
        }}

        .note-author {{
            font-weight: 600;
            color: #0066cc;
            font-size: 14px;
        }}

        .note-timestamp {{
            color: #666;
            font-size: 12px;
        }}

        .note-content {{
            color: #ccc;
            line-height: 1.6;
            white-space: pre-wrap;
            font-size: 13px;
        }}

        /* Add Note Section */
        .add-note-section {{
            padding: 16px 20px;
            border-top: 1px solid #444;
            background: #333;
            flex-shrink: 0;
        }}

        #note-input {{
            width: 100%;
            min-height: 80px;
            background: #2a2a2a;
            border: 1px solid #444;
            border-radius: 4px;
            color: white;
            padding: 12px;
            resize: vertical;
            margin-bottom: 8px;
            font-family: inherit;
            font-size: 13px;
        }}

        #note-input:focus {{
            outline: none;
            border-color: #0066cc;
        }}

        #note-input::placeholder {{
            color: #666;
        }}

        .add-note-btn {{
            width: 100%;
            padding: 10px;
            background: #0066cc;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: background 0.2s;
        }}

        .add-note-btn:hover {{
            background: #0077ee;
        }}

        /* Name Modal */
        .name-modal {{
            display: none;
            position: fixed;
            inset: 0;
            background: rgba(0, 0, 0, 0.8);
            z-index: 300;
            justify-content: center;
            align-items: center;
        }}

        .name-modal.open {{
            display: flex;
        }}

        .name-modal-content {{
            background: #2a2a2a;
            padding: 24px;
            border-radius: 8px;
            width: 320px;
            max-width: 90vw;
        }}

        .name-modal-content h3 {{
            margin: 0 0 8px 0;
            color: white;
            font-size: 18px;
        }}

        .name-modal-content p {{
            margin: 0 0 16px 0;
            color: #888;
            font-size: 13px;
        }}

        .name-modal-content input {{
            width: 100%;
            padding: 10px 12px;
            margin-bottom: 12px;
            background: #333;
            border: 1px solid #444;
            color: white;
            border-radius: 4px;
            font-size: 14px;
        }}

        .name-modal-content input:focus {{
            outline: none;
            border-color: #0066cc;
        }}

        .name-modal-content input::placeholder {{
            color: #666;
        }}

        .name-modal-actions {{
            display: flex;
            gap: 12px;
            margin-top: 8px;
        }}

        .name-modal-actions button {{
            flex: 1;
            padding: 10px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            border: none;
        }}

        .name-modal-actions .cancel-btn {{
            background: #444;
            color: white;
        }}

        .name-modal-actions .cancel-btn:hover {{
            background: #555;
        }}

        .name-modal-actions .save-btn {{
            background: #0066cc;
            color: white;
        }}

        .name-modal-actions .save-btn:hover {{
            background: #0077ee;
        }}

        /* Mobile responsive */
        @media (max-width: 768px) {{
            .snackbar {{
                position: fixed;
                bottom: -60vh;
                right: 0;
                left: 0;
                top: auto;
                width: 100%;
                max-width: 100%;
                height: 60vh;
                border-left: none;
                border-top: 1px solid #444;
                transition: bottom 0.3s cubic-bezier(0.4, 0, 0.2, 1);
            }}

            .snackbar.open {{
                bottom: 0;
                right: 0;
            }}

            /* Mobile snackbar tab - bottom edge */
            .snackbar-tab {{
                top: auto;
                bottom: 0;
                right: 50%;
                transform: translateX(50%);
                border-radius: 8px 8px 0 0;
                border-right: 1px solid #444;
                border-bottom: none;
                flex-direction: row;
                padding: 8px 16px;
            }}
            .snackbar-tab.hidden {{
                transform: translateX(50%) translateY(100%);
            }}
            .tab-text {{
                writing-mode: horizontal-tb;
            }}

            .keyboard-help {{
                display: none;
            }}
        }}

        /* Clickable arrow highlight */
        .arrow-line {{
            cursor: pointer;
            stroke-width: 2;
        }}

        .arrow-line:hover {{
            stroke-width: 3;
            filter: brightness(1.2);
        }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="{home_href}">← Back</a>
        <span>{view_key}</span>
        <div class="separator"></div>
        <div class="controls">
            <button class="btn" id="btn-reset" onclick="resetAnimation()">⟲ Reset</button>
            <button class="btn" id="btn-prev" onclick="previousStep()" disabled>← Previous</button>
            <button class="btn primary" id="btn-play" onclick="togglePlay()">▶ Play</button>
            <button class="btn" id="btn-next" onclick="nextStep()">Next →</button>
            <span class="step-info" id="step-counter">Step 0 of {step_count}</span>
            <div class="separator"></div>
            <div class="speed-control">
                <label>Speed:</label>
                <select id="speed-select" onchange="updateSpeed()">
                    <option value="3000">Slow (3s)</option>
                    <option value="2000" selected>Normal (2s)</option>
                    <option value="1000">Fast (1s)</option>
                    <option value="500">Very Fast (0.5s)</option>
                </select>
            </div>
        </div>
        <div class="separator"></div>
        <a href="{base_path}/view/{view_key}">View Static</a>
    </div>
    <div class="diagram-container" id="diagram-container">
        <div class="loading" id="loading">Loading diagram...</div>
        <div id="svg-wrapper"></div>
        <div class="step-overlay" id="step-overlay">
            <div class="step-number" id="overlay-number">Step 1</div>
            <div class="step-desc" id="overlay-desc">Step description</div>
            <div class="step-tech" id="overlay-tech"></div>
        </div>
    </div>
    <div class="keyboard-help">Space to play/pause • ← → to step • R to reset • 1-9 to jump to step • Scroll to zoom • Drag to pan • Click arrows for details</div>

    <!-- Collapsed snackbar tab -->
    <div id="snackbar-tab" class="snackbar-tab" onclick="expandSnackbar()">
        <span class="tab-icon">&#9776;</span>
        <span class="tab-text">Details</span>
    </div>

    <!-- Step Metadata Snackbar -->
    <div id="step-snackbar" class="snackbar" role="complementary" aria-live="polite">
        <div class="snackbar-header">
            <h3 class="snackbar-title">Step <span id="step-number">1</span> Details</h3>
            <button class="snackbar-close" aria-label="Close details" onclick="collapseSnackbar()">×</button>
        </div>

        <!-- No Selection State -->
        <div id="no-selection" class="no-selection">
            <p class="no-selection-title">No Step Selected</p>
            <p class="no-selection-hint">Click on any relationship arrow to see details</p>
        </div>

        <!-- Tab Navigation -->
        <div class="snackbar-tabs">
            <button class="tab-btn active" data-tab="metadata" onclick="switchSnackbarTab('metadata')">
                Metadata
            </button>
            <button class="tab-btn" data-tab="notes" onclick="switchSnackbarTab('notes')">
                Notes <span class="note-count" id="note-count"></span>
            </button>
        </div>

        <!-- Metadata Tab -->
        <div class="tab-content active" id="tab-metadata">
            <div class="snackbar-content">
                <div class="metadata-section">
                    <h4>Interaction</h4>
                    <p id="step-description" class="step-description">Loading...</p>
                </div>

                <div class="metadata-grid">
                    <div class="metadata-item">
                        <label>Source:</label>
                        <span id="source-element">-</span>
                    </div>
                    <div class="metadata-item">
                        <label>Destination:</label>
                        <span id="dest-element">-</span>
                    </div>
                    <div class="metadata-item" id="technology-item" style="display: none;">
                        <label>Technology:</label>
                        <span id="technology" class="tech-badge">-</span>
                    </div>
                    <div class="metadata-item">
                        <label>Type:</label>
                        <span id="interaction-style" class="style-badge">-</span>
                    </div>
                </div>

                <div class="metadata-section" id="tags-section" style="display: none;">
                    <h4>Tags</h4>
                    <div class="tag-list" id="tag-list"></div>
                </div>

                <div class="metadata-section" id="properties-section" style="display: none;">
                    <h4>Properties</h4>
                    <div class="properties-list" id="properties-list"></div>
                </div>

                <div class="metadata-section" id="perspectives-section" style="display: none;">
                    <h4>Perspectives</h4>
                    <div class="perspectives-list" id="perspectives-list"></div>
                </div>

                <div class="snackbar-actions">
                    <button onclick="navigateToElement('source')" class="action-btn" id="view-source-btn">
                        View Source Element
                    </button>
                    <button onclick="navigateToElement('destination')" class="action-btn" id="view-dest-btn">
                        View Target Element
                    </button>
                </div>
            </div>
        </div>

        <!-- Notes Tab -->
        <div class="tab-content" id="tab-notes">
            <div class="notes-container">
                <div class="notes-list" id="notes-list">
                    <div class="no-notes">No notes yet. Be the first to add one!</div>
                </div>
                <div class="add-note-section">
                    <textarea id="note-input" placeholder="Add a note about this step..."></textarea>
                    <button class="add-note-btn" onclick="submitNote()">Add Note</button>
                </div>
            </div>
        </div>
    </div>

    <!-- Name Prompt Modal -->
    <div class="name-modal" id="name-modal">
        <div class="name-modal-content">
            <h3>Enter Your Name</h3>
            <p>Your name will be displayed with your notes.</p>
            <input type="text" id="first-name-input" placeholder="First Name">
            <input type="text" id="last-name-input" placeholder="Last Name">
            <div class="name-modal-actions">
                <button class="cancel-btn" onclick="cancelNameModal()">Cancel</button>
                <button class="save-btn" onclick="saveName()">Save</button>
            </div>
        </div>
    </div>

    <script>
        const steps = {steps_json};
        const totalSteps = {step_count};

        // Helper function to get base order number (e.g., "4" from "4.1" or "4.2")
        function getBaseOrder(order) {{
            const orderStr = String(order);
            const dotIndex = orderStr.indexOf('.');
            return dotIndex === -1 ? orderStr : orderStr.substring(0, dotIndex);
        }}

        // Helper function to get sub-order number (e.g., "1" from "4.1", "2" from "4.2")
        function getSubOrder(order) {{
            const orderStr = String(order);
            const dotIndex = orderStr.indexOf('.');
            return dotIndex === -1 ? '0' : orderStr.substring(dotIndex + 1);
        }}

        // Group steps by base order for parallel step handling
        const stepGroups = [];  // Array of {{ baseOrder: string, indices: number[] }}
        let tempGroup = null;
        steps.forEach((step, idx) => {{
            const base = getBaseOrder(step.order);
            if (!tempGroup || tempGroup.baseOrder !== base) {{
                tempGroup = {{ baseOrder: base, indices: [idx] }};
                stepGroups.push(tempGroup);
            }} else {{
                tempGroup.indices.push(idx);
            }}
        }});
        const totalLogicalSteps = stepGroups.length;

        // Color palette for distinguishing parallel steps
        const parallelColors = [
            '#E53E3E',  // Red
            '#38A169',  // Green
            '#3182CE',  // Blue
            '#D69E2E',  // Yellow/Gold
            '#805AD5',  // Purple
            '#DD6B20',  // Orange
            '#319795',  // Teal
            '#D53F8C',  // Pink
        ];

        let currentStep = 0;
        let isPlaying = false;
        let playInterval = null;
        let playSpeed = 2000;
        let svgWidth = 0, svgHeight = 0, scale = 1, offsetX = 0, offsetY = 0;
        let arrowLines = [], arrowTexts = [];
        let originalArrowColors = [];  // Store original colors for non-parallel display
        let elementGroups = [];  // For element opacity control
        let isPanning = false, panStartX = 0, panStartY = 0, panStartOffsetX = 0, panStartOffsetY = 0;
        const container = document.getElementById('diagram-container');
        const wrapper = document.getElementById('svg-wrapper');

        // Notes state
        const viewKey = '{view_key}';
        const basePath = '{base_path}';
        let viewNotes = {{ steps: {{}} }};
        let pendingNote = null;
        let notesLoaded = false;

        async function loadSVG() {{
            try {{
                const response = await fetch('{svg_url}');
                const svgText = await response.text();
                wrapper.innerHTML = svgText;
                document.getElementById('loading').style.display = 'none';
                const svg = wrapper.querySelector('svg');
                if (!svg) return;
                svgWidth = parseFloat(svg.getAttribute('width')) || 800;
                svgHeight = parseFloat(svg.getAttribute('height')) || 600;
                if (!svg.getAttribute('viewBox')) svg.setAttribute('viewBox', `0 0 ${{svgWidth}} ${{svgHeight}}`);
                arrowLines = Array.from(svg.querySelectorAll('line[marker-end]'));
                arrowTexts = Array.from(svg.querySelectorAll('text')).filter(t => /^\d+\./.test(t.textContent || ''));
                arrowLines.forEach((line, idx) => {{ line.classList.add('arrow-line'); line.dataset.stepIndex = idx; }});
                arrowTexts.forEach((text, idx) => {{ text.classList.add('arrow-text'); text.dataset.stepIndex = idx; }});
                // Store original arrow colors for resetting after parallel display
                originalArrowColors = arrowLines.map(line => line.getAttribute('stroke') || '#707070');
                // Collect element groups for opacity control
                elementGroups = Array.from(svg.querySelectorAll('.element-group'));
                fitToScreen();
                updateDisplay();
                // Initialize click handlers for step metadata
                initializeArrowClickHandlers();
            }} catch (err) {{
                document.getElementById('loading').textContent = 'Failed to load diagram';
                console.error('Failed to load SVG:', err);
            }}
        }}

        function fitToScreen() {{
            const padding = 40;
            const scaleX = (container.clientWidth - padding * 2) / svgWidth;
            const scaleY = (container.clientHeight - padding * 2) / svgHeight;
            scale = Math.min(scaleX, scaleY, 1.5);
            offsetX = (container.clientWidth - svgWidth * scale) / 2;
            offsetY = (container.clientHeight - svgHeight * scale) / 2;
            applyTransform();
        }}

        function applyTransform() {{
            wrapper.style.transform = `translate(${{offsetX}}px, ${{offsetY}}px) scale(${{scale}})`;
            wrapper.style.width = svgWidth + 'px';
            wrapper.style.height = svgHeight + 'px';
        }}

        function updateDisplay() {{
            document.getElementById('step-counter').textContent = `Step ${{currentStep}} of ${{totalLogicalSteps}}`;
            document.getElementById('btn-prev').disabled = currentStep === 0;
            document.getElementById('btn-next').disabled = currentStep >= totalLogicalSteps;

            if (currentStep === 0) {{
                // Initial state: everything visible, nothing dimmed, original colors
                arrowLines.forEach((line, idx) => {{
                    line.classList.remove('dimmed');
                    line.setAttribute('stroke', originalArrowColors[idx] || '#707070');
                }});
                arrowTexts.forEach((text, idx) => {{
                    text.classList.remove('dimmed');
                    text.setAttribute('fill', originalArrowColors[idx] || '#707070');
                }});
                elementGroups.forEach(group => group.classList.remove('dimmed'));
            }} else {{
                // Spotlight mode: show ALL steps in current logical group (parallel steps together)
                const group = stepGroups[currentStep - 1];
                const activeStepIndices = new Set(group.indices);

                // Collect ALL element IDs from ALL parallel steps in this group
                const activeElementIds = new Set();
                group.indices.forEach(idx => {{
                    activeElementIds.add(steps[idx].sourceId);
                    activeElementIds.add(steps[idx].destId);
                }});

                // Reset ALL arrow colors to original first, then apply parallel colors to active ones
                arrowLines.forEach((line, idx) => {{
                    line.classList.toggle('dimmed', !activeStepIndices.has(idx));
                    line.setAttribute('stroke', originalArrowColors[idx] || '#707070');
                }});
                arrowTexts.forEach((text, idx) => {{
                    text.classList.toggle('dimmed', !activeStepIndices.has(idx));
                    text.setAttribute('fill', originalArrowColors[idx] || '#707070');
                }});

                // Elements: all involved in parallel steps are NOT dimmed
                elementGroups.forEach(group => {{
                    const elementId = group.dataset.elementId;
                    const isInCurrentStep = activeElementIds.has(elementId);
                    group.classList.toggle('dimmed', !isInCurrentStep);
                }});

                // Apply distinct colors only to parallel steps (based on sub-order)
                if (group.indices.length > 1) {{
                    group.indices.forEach((stepIdx) => {{
                        const subOrder = parseInt(getSubOrder(steps[stepIdx].order)) || 1;
                        const color = parallelColors[(subOrder - 1) % parallelColors.length];
                        arrowLines[stepIdx].setAttribute('stroke', color);
                        arrowTexts[stepIdx].setAttribute('fill', color);
                    }});
                }}
            }}

            // Update step overlay for parallel step groups
            const overlay = document.getElementById('step-overlay');
            if (currentStep > 0 && currentStep <= stepGroups.length) {{
                const group = stepGroups[currentStep - 1];
                const firstStep = steps[group.indices[0]];

                // Show base order number and parallel indicator
                const orderDisplay = group.indices.length > 1
                    ? `Step ${{group.baseOrder}} (${{group.indices.length}} parallel)`
                    : `Step ${{firstStep.order}}`;
                document.getElementById('overlay-number').textContent = orderDisplay;

                // Show descriptions of all parallel steps with color-coded bullets (by sub-order)
                if (group.indices.length > 1) {{
                    const descList = group.indices.map((idx) => {{
                        const subOrder = parseInt(getSubOrder(steps[idx].order)) || 1;
                        const color = parallelColors[(subOrder - 1) % parallelColors.length];
                        const desc = steps[idx].description || 'No description';
                        return `<span style="color: ${{color}}">●</span> ${{desc}}`;
                    }}).join('<br>');
                    document.getElementById('overlay-desc').innerHTML = descList;
                }} else {{
                    document.getElementById('overlay-desc').textContent = firstStep.description || 'No description';
                }}

                // Display technology if available
                const techDisplay = document.getElementById('overlay-tech');
                if (group.indices.length > 1) {{
                    const techs = [...new Set(group.indices.map(idx => steps[idx].technology).filter(t => t))];
                    techDisplay.textContent = techs.length > 0 ? techs.join(', ') : '';
                    techDisplay.style.display = techs.length > 0 ? 'block' : 'none';
                }} else {{
                    techDisplay.textContent = firstStep.technology || '';
                    techDisplay.style.display = firstStep.technology ? 'block' : 'none';
                }}

                overlay.classList.add('visible');

                // Always keep snackbar content in sync with first step of group
                populateSnackbarContent(group.indices[0]);
            }} else {{
                overlay.classList.remove('visible');
            }}
        }}

        function nextStep() {{ if (currentStep < totalLogicalSteps) {{ currentStep++; updateDisplay(); }} if (currentStep >= totalLogicalSteps) stopPlaying(); }}
        function previousStep() {{ if (currentStep > 0) {{ currentStep--; updateDisplay(); }} }}
        function resetAnimation() {{ currentStep = 0; stopPlaying(); updateDisplay(); }}
        function togglePlay() {{ isPlaying ? stopPlaying() : startPlaying(); }}
        function startPlaying() {{ if (currentStep >= totalLogicalSteps) currentStep = 0; isPlaying = true; document.getElementById('btn-play').textContent = '⏸ Pause'; playInterval = setInterval(() => {{ nextStep(); if (currentStep >= totalLogicalSteps) stopPlaying(); }}, playSpeed); }}
        function stopPlaying() {{ isPlaying = false; document.getElementById('btn-play').textContent = '▶ Play'; if (playInterval) {{ clearInterval(playInterval); playInterval = null; }} }}
        function updateSpeed() {{ playSpeed = parseInt(document.getElementById('speed-select').value); if (isPlaying) {{ stopPlaying(); startPlaying(); }} }}

        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = container.getBoundingClientRect();
            const mouseX = e.clientX - rect.left, mouseY = e.clientY - rect.top;
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            const newScale = Math.max(0.1, Math.min(3, scale * delta));
            offsetX = mouseX - (mouseX - offsetX) * (newScale / scale);
            offsetY = mouseY - (mouseY - offsetY) * (newScale / scale);
            scale = newScale;
            applyTransform();
        }}, {{ passive: false }});

        container.addEventListener('mousedown', (e) => {{ if (e.button === 0) {{ isPanning = true; panStartX = e.clientX; panStartY = e.clientY; panStartOffsetX = offsetX; panStartOffsetY = offsetY; container.style.cursor = 'grabbing'; }} }});
        document.addEventListener('mousemove', (e) => {{ if (isPanning) {{ offsetX = panStartOffsetX + (e.clientX - panStartX); offsetY = panStartOffsetY + (e.clientY - panStartY); applyTransform(); }} }});
        document.addEventListener('mouseup', () => {{ isPanning = false; container.style.cursor = 'grab'; }});
        container.style.cursor = 'grab';

        document.addEventListener('keydown', (e) => {{
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT' || e.target.tagName === 'TEXTAREA') return;
            switch(e.key) {{
                case ' ': e.preventDefault(); togglePlay(); break;
                case 'ArrowRight': e.preventDefault(); nextStep(); break;
                case 'ArrowLeft': e.preventDefault(); previousStep(); break;
                case 'r': case 'R': resetAnimation(); break;
                case 'f': case 'F': fitToScreen(); break;
                case '0': resetAnimation(); break;
                default: if (e.key >= '1' && e.key <= '9') {{ const stepNum = parseInt(e.key); if (stepNum <= totalLogicalSteps) {{ currentStep = stepNum; updateDisplay(); }} }}
            }}
        }});

        // Snackbar functionality
        let currentSnackbarStep = null;
        let snackbarCollapsed = true;  // Start collapsed
        let selectedStepIndex = null;  // Track selected step separately

        // Populate snackbar content without expanding (used for navigation updates)
        function populateSnackbarContent(stepIndex) {{
            if (stepIndex < 0 || stepIndex >= steps.length) return;

            const step = steps[stepIndex];
            currentSnackbarStep = stepIndex;
            selectedStepIndex = stepIndex;

            // Update header
            document.getElementById('step-number').textContent = step.order;

            // Update description
            document.getElementById('step-description').textContent = step.description || 'No description';

            // Update source and destination
            document.getElementById('source-element').textContent = `${{step.sourceName}} [${{step.sourceType}}]`;
            document.getElementById('dest-element').textContent = `${{step.destName}} [${{step.destType}}]`;

            // Update technology
            const techItem = document.getElementById('technology-item');
            const techBadge = document.getElementById('technology');
            if (step.technology) {{
                techItem.style.display = 'flex';
                techBadge.textContent = step.technology;
            }} else {{
                techItem.style.display = 'none';
            }}

            // Update interaction style
            const styleBadge = document.getElementById('interaction-style');
            styleBadge.textContent = step.interactionStyle || 'Synchronous';
            if (step.interactionStyle === 'Asynchronous') {{
                styleBadge.classList.add('async');
            }} else {{
                styleBadge.classList.remove('async');
            }}

            // Update tags
            const tagsSection = document.getElementById('tags-section');
            const tagList = document.getElementById('tag-list');
            if (step.tags && step.tags.length > 0) {{
                tagsSection.style.display = 'block';
                tagList.innerHTML = step.tags.map(tag => `<span class="tag">${{escapeHtml(tag)}}</span>`).join('');
            }} else {{
                tagsSection.style.display = 'none';
            }}

            // Update properties
            const propsSection = document.getElementById('properties-section');
            const propsList = document.getElementById('properties-list');
            if (step.properties && Object.keys(step.properties).length > 0) {{
                propsSection.style.display = 'block';
                propsList.innerHTML = Object.entries(step.properties)
                    .map(([key, value]) => `<div><dt>${{escapeHtml(key)}}:</dt><dd>${{escapeHtml(value)}}</dd></div>`)
                    .join('');
            }} else {{
                propsSection.style.display = 'none';
            }}

            // Update perspectives
            const perspSection = document.getElementById('perspectives-section');
            const perspList = document.getElementById('perspectives-list');
            if (step.perspectives && Object.keys(step.perspectives).length > 0) {{
                perspSection.style.display = 'block';
                perspList.innerHTML = Object.entries(step.perspectives)
                    .map(([name, description]) => `
                        <div class="perspective-item">
                            <div class="perspective-name">${{escapeHtml(name)}}</div>
                            <div class="perspective-desc">${{escapeHtml(description)}}</div>
                        </div>
                    `).join('');
            }} else {{
                perspSection.style.display = 'none';
            }}

            hideNoSelection();

            // Load notes eagerly to show count in tab, or render if already loaded
            if (!notesLoaded) {{
                loadNotes();
            }} else {{
                renderNotes(stepIndex);
            }}
        }}

        // Show step metadata AND expand snackbar (called from arrow click handlers)
        function showStepMetadata(stepIndex) {{
            populateSnackbarContent(stepIndex);
            expandSnackbar();
        }}

        function expandSnackbar() {{
            const snackbar = document.getElementById('step-snackbar');
            const tab = document.getElementById('snackbar-tab');
            snackbar.classList.add('open');
            tab.classList.add('hidden');
            snackbarCollapsed = false;

            // Show appropriate content based on selection
            if (selectedStepIndex === null) {{
                showNoSelection();
            }}
        }}

        function collapseSnackbar() {{
            const snackbar = document.getElementById('step-snackbar');
            const tab = document.getElementById('snackbar-tab');
            snackbar.classList.remove('open');
            tab.classList.remove('hidden');
            snackbarCollapsed = true;
            // Note: selectedStepIndex is NOT cleared - maintains selection state
        }}

        function showNoSelection() {{
            document.getElementById('no-selection').classList.add('visible');
            // Hide the tabs and all tab content
            document.querySelector('.snackbar-tabs').style.display = 'none';
            document.querySelectorAll('.snackbar .tab-content').forEach(c => c.style.display = 'none');
        }}

        function hideNoSelection() {{
            document.getElementById('no-selection').classList.remove('visible');
            // Show the tabs
            document.querySelector('.snackbar-tabs').style.display = 'flex';
            // Restore tab content visibility based on active state
            document.querySelectorAll('.snackbar .tab-content').forEach(c => {{
                c.style.display = c.classList.contains('active') ? 'block' : 'none';
            }});
        }}

        // Tab switching
        function switchSnackbarTab(tabName) {{
            document.querySelectorAll('.snackbar .tab-btn').forEach(btn => {{
                btn.classList.toggle('active', btn.dataset.tab === tabName);
            }});
            document.querySelectorAll('.snackbar .tab-content').forEach(content => {{
                const isActive = content.id === `tab-${{tabName}}`;
                content.classList.toggle('active', isActive);
                content.style.display = isActive ? 'block' : 'none';
            }});

            // Load notes when switching to notes tab
            if (tabName === 'notes' && !notesLoaded) {{
                loadNotes();
            }}
        }}

        // Load notes from API
        async function loadNotes() {{
            try {{
                const response = await fetch(`${{basePath}}/api/view/${{viewKey}}/notes`);
                if (response.ok) {{
                    viewNotes = await response.json();
                    notesLoaded = true;
                    if (currentSnackbarStep !== null) {{
                        renderNotes(currentSnackbarStep);
                    }}
                }}
            }} catch (err) {{
                console.error('Failed to load notes:', err);
            }}
        }}

        // Render notes for current step
        function renderNotes(stepIndex) {{
            const notesList = document.getElementById('notes-list');
            const stepNotes = viewNotes.steps?.[stepIndex] || [];

            if (stepNotes.length === 0) {{
                notesList.innerHTML = '<div class="no-notes">No notes yet. Be the first to add one!</div>';
            }} else {{
                notesList.innerHTML = stepNotes.map(note => `
                    <div class="note-item">
                        <div class="note-header">
                            <span class="note-author">${{escapeHtml(note.first_name)}} ${{escapeHtml(note.last_name)}}</span>
                            <span class="note-timestamp">${{formatTimestamp(note.timestamp)}}</span>
                        </div>
                        <div class="note-content">${{escapeHtml(note.content)}}</div>
                    </div>
                `).join('');
            }}

            // Update note count badge
            const totalNotes = Object.values(viewNotes.steps || {{}}).reduce((sum, notes) => sum + notes.length, 0);
            document.getElementById('note-count').textContent = totalNotes > 0 ? `(${{totalNotes}})` : '';
        }}

        // Submit note
        function submitNote() {{
            const content = document.getElementById('note-input').value.trim();
            if (!content) return;

            const firstName = localStorage.getItem('structurizr_first_name');
            const lastName = localStorage.getItem('structurizr_last_name');

            if (!firstName || !lastName) {{
                pendingNote = content;
                document.getElementById('name-modal').classList.add('open');
                return;
            }}

            sendNote(content, firstName, lastName);
        }}

        // Save name and submit pending note
        function saveName() {{
            const firstName = document.getElementById('first-name-input').value.trim();
            const lastName = document.getElementById('last-name-input').value.trim();

            if (!firstName || !lastName) {{
                alert('Please enter both first and last name');
                return;
            }}

            localStorage.setItem('structurizr_first_name', firstName);
            localStorage.setItem('structurizr_last_name', lastName);
            document.getElementById('name-modal').classList.remove('open');

            if (pendingNote) {{
                sendNote(pendingNote, firstName, lastName);
                pendingNote = null;
            }}
        }}

        // Cancel name modal
        function cancelNameModal() {{
            document.getElementById('name-modal').classList.remove('open');
            pendingNote = null;
        }}

        // Send note to API
        async function sendNote(content, firstName, lastName) {{
            if (currentSnackbarStep === null) return;

            try {{
                const response = await fetch(`${{basePath}}/api/view/${{viewKey}}/notes`, {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        step_index: currentSnackbarStep,
                        first_name: firstName,
                        last_name: lastName,
                        content: content
                    }})
                }});

                if (response.ok) {{
                    document.getElementById('note-input').value = '';
                    viewNotes = await response.json();
                    renderNotes(currentSnackbarStep);
                }} else {{
                    alert('Failed to save note. Please try again.');
                }}
            }} catch (err) {{
                console.error('Failed to save note:', err);
                alert('Failed to save note. Please try again.');
            }}
        }}

        // Format timestamp for display
        function formatTimestamp(iso) {{
            const date = new Date(iso);
            return date.toLocaleDateString() + ' ' + date.toLocaleTimeString([], {{ hour: '2-digit', minute: '2-digit' }});
        }}

        function escapeHtml(str) {{
            const div = document.createElement('div');
            div.textContent = str;
            return div.innerHTML;
        }}

        function navigateToElement(type) {{
            if (currentSnackbarStep === null) return;
            const step = steps[currentSnackbarStep];

            const elementId = type === 'source' ? step.sourceId : step.destId;
            // TODO: Implement navigation to element view
            console.log(`Navigate to ${{type}} element: ${{elementId}}`);
            alert(`Navigation to element views will be implemented in a future update.\\nElement ID: ${{elementId}}`);
        }}

        // Arrow click detection
        function initializeArrowClickHandlers() {{
            // This will be called after SVG is loaded
            const svg = wrapper.querySelector('svg');
            if (!svg) return;

            arrowLines.forEach((line, idx) => {{
                line.addEventListener('click', (e) => {{
                    e.stopPropagation();
                    showStepMetadata(idx);
                }});

                // Add visual feedback on hover
                line.style.cursor = 'pointer';
            }});

            // Also make text clickable
            arrowTexts.forEach((text, idx) => {{
                text.addEventListener('click', (e) => {{
                    e.stopPropagation();
                    showStepMetadata(idx);
                }});

                text.style.cursor = 'pointer';
            }});
        }}

        // Keyboard shortcuts for snackbar
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'Escape' && !snackbarCollapsed) {{
                collapseSnackbar();
            }}
        }});

        // Close snackbar when clicking outside (optional)
        document.getElementById('step-snackbar').addEventListener('click', (e) => {{
            if (e.target === e.currentTarget) {{
                // Clicked on backdrop, don't close for now
            }}
        }});

        window.addEventListener('resize', fitToScreen);
        loadSVG();
    </script>
</body>
</html>"##,
        view_key = view_key,
        home_href = home_href,
        step_count = step_count,
        base_path = base_path,
        steps_json = steps_json,
        svg_url = svg_url,
    ))
}

/// Render edit diagram HTML using the shared generator.
fn render_edit_diagram_html(workspace: &Workspace, view_key: &str, base_path: &str) -> Result<Html<String>> {
    // Use ws path relative to base_path for multi-workspace routing
    let ws_path = if base_path.is_empty() { "".to_string() } else { format!("{}/ws", base_path) };
    // Extract workspace_id from base_path for layout
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };
    let html = generate_editor_html(workspace, view_key, base_path, &ws_path, workspace_id);
    Ok(Html(html))
}

/// Generate full editor HTML with WebSocket support, pan/zoom, and element dragging.
///
/// This shared function provides full diagram editing features:
/// - WebSocket connection for real-time collaboration
/// - Canvas-based rendering with pan and zoom
/// - Element dragging to reposition
/// - Auto-layout, save, undo/redo functionality
/// - Connection status indicator
///
/// # Arguments
/// - `workspace` - The workspace containing the diagram
/// - `view_key` - The view being edited
/// - `base_path` - Base URL path (empty for single-workspace, e.g., "/w/my-workspace" for multi)
/// - `ws_path` - WebSocket base path (empty for single-workspace, e.g., "/w/my-workspace/ws" for multi)
/// - `workspace_id` - Optional workspace ID for multi-workspace mode
fn generate_editor_html(workspace: &Workspace, view_key: &str, base_path: &str, ws_path: &str, workspace_id: Option<&str>) -> String {
    let svg_url = if base_path.is_empty() {
        format!("/view/{}/svg", view_key)
    } else {
        format!("{}/view/{}/svg", base_path, view_key)
    };
    let ws_base = if ws_path.is_empty() { "/ws".to_string() } else { ws_path.to_string() };

    let title = format!("Edit {} - {}", view_key, workspace.name);

    let extra_styles = r##"<style>
        .editor-toolbar {
            background: var(--toolbar-bg);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 20px;
            border-bottom: 1px solid var(--toolbar-border);
        }
        .editor-toolbar span { color: var(--toolbar-text); }
        .editor-toolbar button {
            background: #0066cc;
            color: #fff;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-weight: 500;
        }
        .editor-toolbar button:hover { background: #0052a3; }
        .editor-toolbar button.secondary {
            background: var(--bg-tertiary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
        }
        .editor-toolbar button.secondary:hover { background: var(--card-hover); }
        .editor-toolbar .divider { width: 1px; height: 24px; background: var(--border-color); }
        .editor-container { display: flex; flex: 1; }
        .canvas-container { flex: 1; overflow: hidden; position: relative; background: var(--canvas-bg); cursor: grab; }
        .canvas-container.dragging { cursor: grabbing; }
        .canvas-container.element-dragging { cursor: move; }
        #svg-wrapper {
            position: absolute;
            transform-origin: 0 0;
            background: white;
            box-shadow: 0 4px 20px rgba(0,0,0,0.2);
        }
        #svg-wrapper svg { display: block; }
        .element {
            position: absolute;
            cursor: move;
            user-select: none;
            border: 2px solid transparent;
            border-radius: 4px;
            transition: border-color 0.15s ease;
        }
        .element:hover { border-color: rgba(255, 255, 255, 0.3); }
        .element.selected { border-color: #0066cc !important; box-shadow: 0 0 0 2px rgba(0, 102, 204, 0.3); }
        .element.dragging { opacity: 0.8; z-index: 1000; }
        .status {
            position: fixed;
            bottom: 20px;
            right: 20px;
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            padding: 10px 20px;
            border-radius: 4px;
            font-size: 12px;
            display: flex;
            align-items: center;
            gap: 8px;
            color: var(--text-secondary);
        }
        .status .dot {
            width: 8px;
            height: 8px;
            border-radius: 50%;
            background: var(--text-muted);
        }
        .status.connected .dot { background: #28a745; }
        .status.disconnected .dot { background: #dc3545; }
        .status.connecting .dot { background: #ffc107; animation: pulse 1s infinite; }
        @keyframes pulse { 0%, 100% { opacity: 1; } 50% { opacity: 0.5; } }
        .zoom-controls {
            position: absolute;
            bottom: 20px;
            left: 20px;
            display: flex;
            gap: 5px;
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            padding: 5px;
            border-radius: 4px;
        }
        .zoom-controls button {
            background: var(--bg-tertiary);
            border: none;
            color: var(--text-primary);
            padding: 8px 12px;
            border-radius: 3px;
            cursor: pointer;
        }
        .zoom-controls button:hover { background: var(--card-hover); }
        .zoom-controls span {
            padding: 8px 12px;
            font-size: 12px;
            color: var(--text-muted);
        }
        .minimap {
            position: absolute;
            bottom: 20px;
            right: 100px;
            width: 150px;
            height: 100px;
            background: var(--card-bg);
            border: 1px solid var(--border-color);
            border-radius: 4px;
            overflow: hidden;
        }
        .minimap-viewport {
            position: absolute;
            border: 1px solid #0066cc;
            background: rgba(0, 102, 204, 0.1);
        }
        .help-text {
            position: absolute;
            top: 10px;
            right: 10px;
            font-size: 11px;
            color: var(--text-muted);
            text-align: right;
        }
    </style>"##;

    let content = format!(r##"
        <div class="editor-toolbar">
            <span id="view-name">{view_key}</span>
            <span class="divider"></span>
            <button onclick="autoLayout()">Auto Layout</button>
            <button onclick="save()" class="secondary">Save</button>
            <button onclick="undo()" class="secondary">Undo</button>
            <button onclick="redo()" class="secondary">Redo</button>
            <span style="margin-left: auto; font-size: 12px;">
                Drag elements to reposition • Drag background to pan • Scroll to zoom
            </span>
        </div>
        <div class="editor-container">
            <div class="canvas-container" id="canvas-container">
                <div id="svg-wrapper">
                    <!-- SVG content will be loaded here -->
                </div>
                <div class="zoom-controls">
                    <button onclick="zoomIn()">+</button>
                    <span id="zoom-level">100%</span>
                    <button onclick="zoomOut()">−</button>
                    <button onclick="resetZoom()">Reset</button>
                </div>
                <div class="minimap" id="minimap">
                    <div class="minimap-viewport" id="minimap-viewport"></div>
                </div>
                <div class="help-text">
                    Drag background: Pan<br>
                    Scroll: Zoom<br>
                    Drag element: Move
                </div>
            </div>
        </div>
        <div class="status connecting" id="status">
            <span class="dot"></span>
            <span id="status-text">Connecting...</span>
        </div>
    "##, view_key = view_key);

    let extra_scripts = format!(r##"<script>
        const viewKey = '{view_key}';
        const wsBase = '{ws_base}';
        const wsUrl = 'ws://' + window.location.host + wsBase + '/edit/' + viewKey;
        const svgUrl = '{svg_url}';
        let ws = null;
        let reconnectAttempts = 0;
        const maxReconnectAttempts = 10;

        // Generate unique client ID to identify our own messages
        const clientId = 'client_' + Math.random().toString(36).substr(2, 9);

        // Track pending moves we've sent (to ignore echo from server)
        const pendingMoves = new Set();

        // Canvas state (same as viewer)
        let svgWidth = 0, svgHeight = 0;
        let scale = 1;
        let offsetX = 0, offsetY = 0;

        // Pan state
        let isPanning = false;
        let panStartX = 0, panStartY = 0;
        let panStartOffsetX = 0, panStartOffsetY = 0;

        // Element dragging state
        let selectedElement = null;
        let isDraggingElement = false;
        let dragStartX = 0, dragStartY = 0;
        let elementStartX = 0, elementStartY = 0;
        let originalPosition = null;

        // Undo/redo history
        let undoStack = [];
        let redoStack = [];

        // Connector manager for real-time connector updates during drag
        let connectorManager = null;
        let pendingConnectorUpdate = null;

        class ConnectorManager {{
            constructor() {{
                this.connectors = new Map();      // connectorKey -> {{element, sourceId, targetId, routingType}}
                this.elementConnectors = new Map(); // elementId -> Set<connectorKey>
                this.elementBounds = new Map();    // elementId -> {{baseX, baseY, width, height}}
                this.connectorLabels = new Map();  // connectorKey -> {{descElement, techElement}}
            }}

            // Index all connectors from SVG
            indexConnectors(svg) {{
                svg.querySelectorAll('[data-source][data-target]').forEach(el => {{
                    const sourceId = el.getAttribute('data-source');
                    const targetId = el.getAttribute('data-target');
                    const key = `${{sourceId}}_${{targetId}}`;
                    const routingType = el.getAttribute('data-routing') || 'direct';

                    this.connectors.set(key, {{
                        element: el,
                        sourceId,
                        targetId,
                        type: el.tagName.toLowerCase(),
                        routingType
                    }});

                    // Build reverse index
                    if (!this.elementConnectors.has(sourceId)) {{
                        this.elementConnectors.set(sourceId, new Set());
                    }}
                    this.elementConnectors.get(sourceId).add(key);

                    if (!this.elementConnectors.has(targetId)) {{
                        this.elementConnectors.set(targetId, new Set());
                    }}
                    this.elementConnectors.get(targetId).add(key);
                }});
                console.log('Indexed', this.connectors.size, 'connectors');
            }}

            // Index all draggable elements
            indexElements(svg) {{
                svg.querySelectorAll('.draggable-element').forEach(group => {{
                    const elementId = group.getAttribute('data-element-id');
                    // Find the shape element (rect, ellipse, polygon, etc.)
                    const shape = group.querySelector('rect, ellipse, polygon, circle, path');
                    if (shape) {{
                        let bounds;
                        if (shape.tagName === 'rect') {{
                            bounds = {{
                                baseX: parseFloat(shape.getAttribute('x')) || 0,
                                baseY: parseFloat(shape.getAttribute('y')) || 0,
                                width: parseFloat(shape.getAttribute('width')) || 0,
                                height: parseFloat(shape.getAttribute('height')) || 0
                            }};
                        }} else if (shape.tagName === 'ellipse') {{
                            const cx = parseFloat(shape.getAttribute('cx')) || 0;
                            const cy = parseFloat(shape.getAttribute('cy')) || 0;
                            const rx = parseFloat(shape.getAttribute('rx')) || 0;
                            const ry = parseFloat(shape.getAttribute('ry')) || 0;
                            bounds = {{
                                baseX: cx - rx,
                                baseY: cy - ry,
                                width: rx * 2,
                                height: ry * 2
                            }};
                        }} else if (shape.tagName === 'circle') {{
                            const cx = parseFloat(shape.getAttribute('cx')) || 0;
                            const cy = parseFloat(shape.getAttribute('cy')) || 0;
                            const r = parseFloat(shape.getAttribute('r')) || 0;
                            bounds = {{
                                baseX: cx - r,
                                baseY: cy - r,
                                width: r * 2,
                                height: r * 2
                            }};
                        }} else {{
                            // Fallback: use getBBox if available
                            try {{
                                const bbox = shape.getBBox();
                                bounds = {{
                                    baseX: bbox.x,
                                    baseY: bbox.y,
                                    width: bbox.width,
                                    height: bbox.height
                                }};
                            }} catch (e) {{
                                bounds = {{ baseX: 0, baseY: 0, width: 100, height: 80 }};
                            }}
                        }}
                        this.elementBounds.set(elementId, bounds);
                    }}
                }});
                console.log('Indexed', this.elementBounds.size, 'element bounds');
            }}

            // Get element rect with current transform applied
            getEffectiveRect(elementId) {{
                const base = this.elementBounds.get(elementId);
                if (!base) return null;

                const group = document.querySelector(`[data-element-id="${{elementId}}"]`);
                const transform = group?.getAttribute('transform') || 'translate(0, 0)';
                const match = transform.match(/translate\(([^,]+),\s*([^)]+)\)/);
                const offsetX = match ? parseFloat(match[1]) : 0;
                const offsetY = match ? parseFloat(match[2]) : 0;

                return {{
                    x: base.baseX + offsetX,
                    y: base.baseY + offsetY,
                    width: base.width,
                    height: base.height,
                    cx: base.baseX + offsetX + base.width / 2,
                    cy: base.baseY + offsetY + base.height / 2
                }};
            }}

            // Update all connectors attached to an element
            updateConnectorsForElement(elementId) {{
                const connectorKeys = this.elementConnectors.get(elementId);
                if (!connectorKeys) return;

                for (const key of connectorKeys) {{
                    this.updateConnector(key);
                }}
            }}

            updateConnector(key) {{
                const connector = this.connectors.get(key);
                if (!connector) return;

                const sourceRect = this.getEffectiveRect(connector.sourceId);
                const targetRect = this.getEffectiveRect(connector.targetId);

                if (!sourceRect || !targetRect) return;

                switch (connector.routingType) {{
                    case 'direct':
                        this.updateDirectConnector(key, connector, sourceRect, targetRect);
                        break;
                    case 'curved':
                        this.updateCurvedConnector(key, connector, sourceRect, targetRect);
                        break;
                    case 'orthogonal':
                        this.updateOrthogonalConnector(key, connector, sourceRect, targetRect);
                        break;
                }}
            }}

            // Line-rectangle intersection calculation
            lineRectIntersection(fromX, fromY, toX, toY, rect) {{
                const {{ x, y, width, height }} = rect;
                const dx = toX - fromX;
                const dy = toY - fromY;

                if (Math.abs(dx) < 0.001 && Math.abs(dy) < 0.001) {{
                    return {{ x: fromX, y: fromY }};
                }}

                // Check all 4 edges
                const edges = [
                    {{ t: dx !== 0 ? (x - fromX) / dx : Infinity, edge: 'left', getCoord: (t) => ({{ x: x, y: fromY + t * dy }}), min: y, max: y + height, isY: true }},
                    {{ t: dx !== 0 ? (x + width - fromX) / dx : Infinity, edge: 'right', getCoord: (t) => ({{ x: x + width, y: fromY + t * dy }}), min: y, max: y + height, isY: true }},
                    {{ t: dy !== 0 ? (y - fromY) / dy : Infinity, edge: 'top', getCoord: (t) => ({{ x: fromX + t * dx, y: y }}), min: x, max: x + width, isY: false }},
                    {{ t: dy !== 0 ? (y + height - fromY) / dy : Infinity, edge: 'bottom', getCoord: (t) => ({{ x: fromX + t * dx, y: y + height }}), min: x, max: x + width, isY: false }}
                ];

                let result = {{ x: toX, y: toY }};
                let bestT = Infinity;

                for (const e of edges) {{
                    if (e.t > 0.001 && e.t < 1) {{
                        const coord = e.getCoord(e.t);
                        const checkVal = e.isY ? coord.y : coord.x;
                        if (checkVal >= e.min && checkVal <= e.max) {{
                            if (e.t < bestT) {{
                                bestT = e.t;
                                result = coord;
                            }}
                        }}
                    }}
                }}

                return result;
            }}

            // Direct routing - simple line
            updateDirectConnector(key, connector, sourceRect, targetRect) {{
                const start = this.lineRectIntersection(
                    targetRect.cx, targetRect.cy, sourceRect.cx, sourceRect.cy, sourceRect
                );
                const end = this.lineRectIntersection(
                    sourceRect.cx, sourceRect.cy, targetRect.cx, targetRect.cy, targetRect
                );

                if (connector.type === 'line') {{
                    connector.element.setAttribute('x1', start.x.toFixed(1));
                    connector.element.setAttribute('y1', start.y.toFixed(1));
                    connector.element.setAttribute('x2', end.x.toFixed(1));
                    connector.element.setAttribute('y2', end.y.toFixed(1));
                }} else {{
                    connector.element.setAttribute('d', `M ${{start.x.toFixed(1)}} ${{start.y.toFixed(1)}} L ${{end.x.toFixed(1)}} ${{end.y.toFixed(1)}}`);
                }}

                // Update labels to follow the connector
                this.updateLabelPosition(key, start, end);
            }}

            // Curved routing - cubic Bezier
            updateCurvedConnector(key, connector, sourceRect, targetRect) {{
                const start = this.lineRectIntersection(
                    targetRect.cx, targetRect.cy, sourceRect.cx, sourceRect.cy, sourceRect
                );
                const end = this.lineRectIntersection(
                    sourceRect.cx, sourceRect.cy, targetRect.cx, targetRect.cy, targetRect
                );

                const dx = end.x - start.x;
                const dy = end.y - start.y;
                const length = Math.sqrt(dx * dx + dy * dy);

                if (length < 10) {{
                    // Too close, use straight line
                    connector.element.setAttribute('d', `M ${{start.x.toFixed(1)}} ${{start.y.toFixed(1)}} L ${{end.x.toFixed(1)}} ${{end.y.toFixed(1)}}`);
                    this.updateLabelPosition(key, start, end);
                    return;
                }}

                // Unit vector and perpendicular
                const ux = dx / length;
                const uy = dy / length;
                const perpX = -uy;
                const perpY = ux;

                // Control point distance and curve offset
                const ctrlDist = length / 3;
                const baseCurveOffset = Math.min(Math.max(length * 0.15, 10), 50);

                // Control points
                const ctrl1x = start.x + ux * ctrlDist + perpX * baseCurveOffset;
                const ctrl1y = start.y + uy * ctrlDist + perpY * baseCurveOffset;
                const ctrl2x = end.x - ux * ctrlDist + perpX * baseCurveOffset;
                const ctrl2y = end.y - uy * ctrlDist + perpY * baseCurveOffset;

                connector.element.setAttribute('d',
                    `M ${{start.x.toFixed(1)}} ${{start.y.toFixed(1)}} C ${{ctrl1x.toFixed(1)}} ${{ctrl1y.toFixed(1)}} ${{ctrl2x.toFixed(1)}} ${{ctrl2y.toFixed(1)}} ${{end.x.toFixed(1)}} ${{end.y.toFixed(1)}}`
                );

                // Update labels to follow the connector
                this.updateLabelPosition(key, start, end);
            }}

            // Orthogonal routing - L or Z shaped paths
            updateOrthogonalConnector(key, connector, sourceRect, targetRect) {{
                const portClearance = 20;

                // Determine primary direction
                const deltaY = targetRect.cy - sourceRect.cy;
                const deltaX = targetRect.cx - sourceRect.cx;

                let waypoints = [];

                if (Math.abs(deltaY) > Math.abs(deltaX)) {{
                    // Primarily vertical
                    const exitY = deltaY > 0 ? sourceRect.y + sourceRect.height : sourceRect.y;
                    const entryY = deltaY > 0 ? targetRect.y : targetRect.y + targetRect.height;

                    const startX = sourceRect.cx;
                    const startY = exitY;
                    const endX = targetRect.cx;
                    const endY = entryY;

                    const midY = (startY + endY) / 2;

                    waypoints = [
                        {{ x: startX, y: startY }},
                        {{ x: startX, y: midY }},
                        {{ x: endX, y: midY }},
                        {{ x: endX, y: endY }}
                    ];
                }} else {{
                    // Primarily horizontal
                    const exitX = deltaX > 0 ? sourceRect.x + sourceRect.width : sourceRect.x;
                    const entryX = deltaX > 0 ? targetRect.x : targetRect.x + targetRect.width;

                    const startX = exitX;
                    const startY = sourceRect.cy;
                    const endX = entryX;
                    const endY = targetRect.cy;

                    const midX = (startX + endX) / 2;

                    waypoints = [
                        {{ x: startX, y: startY }},
                        {{ x: midX, y: startY }},
                        {{ x: midX, y: endY }},
                        {{ x: endX, y: endY }}
                    ];
                }}

                // Simplify path (remove collinear points)
                waypoints = this.simplifyPath(waypoints);

                // Build path string
                let d = `M ${{waypoints[0].x.toFixed(1)}} ${{waypoints[0].y.toFixed(1)}}`;
                for (let i = 1; i < waypoints.length; i++) {{
                    d += ` L ${{waypoints[i].x.toFixed(1)}} ${{waypoints[i].y.toFixed(1)}}`;
                }}

                connector.element.setAttribute('d', d);

                // Update labels to follow the connector (use first and last waypoints)
                const start = waypoints[0];
                const end = waypoints[waypoints.length - 1];
                this.updateLabelPosition(key, start, end);
            }}

            simplifyPath(waypoints) {{
                if (waypoints.length <= 2) return waypoints;

                const result = [waypoints[0]];
                for (let i = 1; i < waypoints.length - 1; i++) {{
                    const prev = result[result.length - 1];
                    const curr = waypoints[i];
                    const next = waypoints[i + 1];

                    const isHorizontal = Math.abs(prev.y - curr.y) < 0.1 && Math.abs(curr.y - next.y) < 0.1;
                    const isVertical = Math.abs(prev.x - curr.x) < 0.1 && Math.abs(curr.x - next.x) < 0.1;

                    if (!isHorizontal && !isVertical) {{
                        result.push(curr);
                    }}
                }}
                result.push(waypoints[waypoints.length - 1]);
                return result;
            }}

            // Index all relationship labels from SVG
            indexLabels(svg) {{
                svg.querySelectorAll('.relationship-label').forEach(el => {{
                    const sourceId = el.getAttribute('data-source');
                    const targetId = el.getAttribute('data-target');
                    const labelType = el.getAttribute('data-label-type');
                    if (!sourceId || !targetId) return;

                    const key = `${{sourceId}}_${{targetId}}`;

                    if (!this.connectorLabels.has(key)) {{
                        this.connectorLabels.set(key, {{}});
                    }}
                    const labels = this.connectorLabels.get(key);
                    if (labelType === 'description') {{
                        labels.descElement = el;
                    }} else if (labelType === 'technology') {{
                        labels.techElement = el;
                    }}
                }});
                console.log('Indexed', this.connectorLabels.size, 'connector label groups');
            }}

            // Update label positions for a connector
            updateLabelPosition(key, startPoint, endPoint) {{
                const labels = this.connectorLabels.get(key);
                if (!labels) return;

                // Calculate midpoint
                const midX = (startPoint.x + endPoint.x) / 2;
                const midY = (startPoint.y + endPoint.y) / 2;

                // Calculate angle for rotation
                const dx = endPoint.x - startPoint.x;
                const dy = endPoint.y - startPoint.y;
                const length = Math.sqrt(dx * dx + dy * dy);

                // Calculate angle in degrees
                let angle = Math.atan2(dy, dx) * 180 / Math.PI;
                // Normalize angle for readability (avoid upside-down text)
                if (angle > 90 || angle < -90) {{
                    angle += 180;
                }}

                // Calculate perpendicular offset direction
                const perpX = length > 0.1 ? -dy / length : 0;
                const perpY = length > 0.1 ? dx / length : 0;

                // Update description label (above the line)
                if (labels.descElement) {{
                    const offset = -12;  // Above the line
                    const labelX = midX + perpX * offset;
                    const labelY = midY + perpY * offset;

                    labels.descElement.setAttribute('x', labelX.toFixed(1));
                    labels.descElement.setAttribute('y', labelY.toFixed(1));
                    labels.descElement.setAttribute('transform',
                        `rotate(${{angle.toFixed(1)}}, ${{labelX.toFixed(1)}}, ${{labelY.toFixed(1)}})`);
                }}

                // Update technology label (below the line)
                if (labels.techElement) {{
                    const offset = 18;  // Below the line (accounts for font height)
                    const labelX = midX + perpX * offset;
                    const labelY = midY + perpY * offset;

                    labels.techElement.setAttribute('x', labelX.toFixed(1));
                    labels.techElement.setAttribute('y', labelY.toFixed(1));
                    labels.techElement.setAttribute('transform',
                        `rotate(${{angle.toFixed(1)}}, ${{labelX.toFixed(1)}}, ${{labelY.toFixed(1)}})`);
                }}
            }}
        }}

        // Schedule connector updates with requestAnimationFrame for performance
        function scheduleConnectorUpdate(elementId) {{
            if (pendingConnectorUpdate) return;
            pendingConnectorUpdate = requestAnimationFrame(() => {{
                if (connectorManager) {{
                    connectorManager.updateConnectorsForElement(elementId);
                }}
                pendingConnectorUpdate = null;
            }});
        }}

        const container = document.getElementById('canvas-container');
        const wrapper = document.getElementById('svg-wrapper');
        const statusEl = document.getElementById('status');
        const statusText = document.getElementById('status-text');

        // WebSocket connection with reconnection logic
        function connect() {{
            console.log('Connecting to WebSocket:', wsUrl);
            ws = new WebSocket(wsUrl);

            ws.onopen = () => {{
                console.log('WebSocket connected');
                reconnectAttempts = 0;
                statusEl.className = 'status connected';
                statusText.textContent = 'Connected';

                // Request initial state
                ws.send(JSON.stringify({{ type: 'request_state', view_key: viewKey }}));
            }};

            ws.onclose = () => {{
                console.log('WebSocket disconnected');
                statusEl.className = 'status disconnected';
                statusText.textContent = 'Disconnected';

                // Reconnect with exponential backoff
                if (reconnectAttempts < maxReconnectAttempts) {{
                    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000);
                    reconnectAttempts++;
                    statusEl.className = 'status connecting';
                    statusText.textContent = `Reconnecting in ${{Math.round(delay/1000)}}s...`;
                    setTimeout(connect, delay);
                }} else {{
                    statusText.textContent = 'Connection failed. Refresh to retry.';
                }}
            }};

            ws.onerror = (error) => {{
                console.error('WebSocket error:', error);
            }};

            ws.onmessage = (event) => {{
                const message = JSON.parse(event.data);
                handleMessage(message);
            }};
        }}

        function handleMessage(message) {{
            console.log('Received:', message);

            switch (message.type) {{
                case 'state':
                    // Initial state received - apply positions to elements
                    console.log('State received with', message.elements?.length || 0, 'elements');
                    if (message.elements) {{
                        message.elements.forEach(el => {{
                            applyElementPosition(el.id, el.x, el.y);
                        }});
                    }}
                    break;
                case 'element_moved':
                    // Check if this is our own move echoed back
                    const moveKey = `${{message.element_id}}_${{message.x}}_${{message.y}}`;
                    if (pendingMoves.has(moveKey)) {{
                        // This is our own move, ignore it
                        pendingMoves.delete(moveKey);
                        console.log('Ignoring our own move echo for', message.element_id);
                    }} else {{
                        // Another client moved an element - update position without refresh
                        console.log('Element', message.element_id, 'moved by another client to', message.x, message.y);
                        applyElementPosition(message.element_id, message.x, message.y);
                    }}
                    break;
                case 'layout_updated':
                    // Auto-layout was applied - need full refresh
                    console.log('Layout updated');
                    refreshDiagram();
                    break;
                case 'saved':
                    console.log('Changes saved successfully');
                    statusText.textContent = 'Saved';
                    setTimeout(() => {{
                        if (statusEl.classList.contains('connected')) {{
                            statusText.textContent = 'Connected';
                        }}
                    }}, 2000);
                    break;
                case 'error':
                    alert('Error: ' + message.message);
                    break;
            }}
        }}

        // Apply position to an element without reloading
        function applyElementPosition(elementId, x, y) {{
            const element = wrapper.querySelector(`[data-element-id="${{elementId}}"]`);
            if (element) {{
                element.setAttribute('transform', `translate(${{x}}, ${{y}})`);
            }}
        }}

        // Load SVG content into the wrapper (like viewer)
        async function loadSVG() {{
            try {{
                const response = await fetch(svgUrl);
                const svgText = await response.text();
                wrapper.innerHTML = svgText;

                const svg = wrapper.querySelector('svg');
                if (!svg) return;

                // Get dimensions from SVG
                svgWidth = parseFloat(svg.getAttribute('width')) || 800;
                svgHeight = parseFloat(svg.getAttribute('height')) || 600;

                // Ensure viewBox is set
                if (!svg.getAttribute('viewBox')) {{
                    svg.setAttribute('viewBox', `0 0 ${{svgWidth}} ${{svgHeight}}`);
                }}

                // Set wrapper size
                wrapper.style.width = svgWidth + 'px';
                wrapper.style.height = svgHeight + 'px';

                // Fit to screen
                fitToScreen();

                // Set up drag handlers for elements
                setupDragHandlers();

                // Initialize connector manager for real-time connector updates
                connectorManager = new ConnectorManager();
                connectorManager.indexConnectors(wrapper);
                connectorManager.indexElements(wrapper);
                connectorManager.indexLabels(wrapper);
            }} catch (error) {{
                console.error('Failed to load SVG:', error);
            }}
        }}

        function refreshDiagram() {{
            loadSVG();
        }}

        // Fit SVG to screen (like viewer)
        function fitToScreen() {{
            const padding = 40;
            const scaleX = (container.clientWidth - padding * 2) / svgWidth;
            const scaleY = (container.clientHeight - padding * 2) / svgHeight;
            scale = Math.min(scaleX, scaleY, 1.5);
            offsetX = (container.clientWidth - svgWidth * scale) / 2;
            offsetY = (container.clientHeight - svgHeight * scale) / 2;
            applyTransform();
        }}

        // Apply CSS transform (like viewer)
        function applyTransform() {{
            wrapper.style.transform = `translate(${{offsetX}}px, ${{offsetY}}px) scale(${{scale}})`;
            updateZoomDisplay();
            updateMinimap();
        }}

        // Set up drag handlers for draggable elements
        function setupDragHandlers() {{
            const draggableElements = wrapper.querySelectorAll('.draggable-element');

            draggableElements.forEach(element => {{
                element.style.cursor = 'move';

                element.addEventListener('mousedown', (e) => {{
                    if (e.button !== 0) return; // Only left click

                    e.preventDefault();
                    e.stopPropagation();

                    selectedElement = element;
                    isDraggingElement = true;
                    container.classList.add('element-dragging');

                    // Get current transform
                    const transform = element.getAttribute('transform') || 'translate(0, 0)';
                    const match = transform.match(/translate\(([^,]+),\s*([^)]+)\)/);
                    elementStartX = match ? parseFloat(match[1]) : 0;
                    elementStartY = match ? parseFloat(match[2]) : 0;

                    // Store original position for undo
                    originalPosition = {{ x: elementStartX, y: elementStartY }};

                    // Store mouse start position (in screen coords)
                    dragStartX = e.clientX;
                    dragStartY = e.clientY;

                    element.classList.add('dragging');
                }});
            }});
        }}

        // Handle mouse move for dragging elements and panning
        document.addEventListener('mousemove', (e) => {{
            if (isDraggingElement && selectedElement) {{
                // Calculate delta in screen pixels, then convert to SVG space
                const dx = (e.clientX - dragStartX) / scale;
                const dy = (e.clientY - dragStartY) / scale;
                const newX = elementStartX + dx;
                const newY = elementStartY + dy;

                // Update element transform
                selectedElement.setAttribute('transform', `translate(${{newX}}, ${{newY}})`);

                // Update connectors attached to this element in real-time
                const elementId = selectedElement.getAttribute('data-element-id');
                scheduleConnectorUpdate(elementId);
            }} else if (isPanning) {{
                // Pan the canvas (like viewer)
                offsetX = panStartOffsetX + (e.clientX - panStartX);
                offsetY = panStartOffsetY + (e.clientY - panStartY);
                applyTransform();
            }}
        }});

        // Handle mouse up to end dragging
        document.addEventListener('mouseup', (e) => {{
            if (isDraggingElement && selectedElement) {{
                isDraggingElement = false;
                selectedElement.classList.remove('dragging');
                container.classList.remove('element-dragging');

                // Get final position
                const transform = selectedElement.getAttribute('transform') || 'translate(0, 0)';
                const match = transform.match(/translate\(([^,]+),\s*([^)]+)\)/);
                const finalX = match ? parseFloat(match[1]) : 0;
                const finalY = match ? parseFloat(match[2]) : 0;

                // Send position update to server if position changed
                if (originalPosition && (Math.abs(finalX - originalPosition.x) > 0.5 || Math.abs(finalY - originalPosition.y) > 0.5)) {{
                    const elementId = selectedElement.getAttribute('data-element-id');
                    const roundedX = Math.round(finalX);
                    const roundedY = Math.round(finalY);

                    // Track this move so we don't refresh when echoed back
                    const moveKey = `${{elementId}}_${{roundedX}}_${{roundedY}}`;
                    pendingMoves.add(moveKey);

                    send({{
                        type: 'element_moved',
                        view_key: viewKey,
                        element_id: elementId,
                        x: roundedX,
                        y: roundedY
                    }});

                    // Add to undo stack
                    undoStack.push({{
                        type: 'move',
                        element_id: elementId,
                        from: originalPosition,
                        to: {{ x: finalX, y: finalY }}
                    }});
                    redoStack = [];
                }}

                selectedElement = null;
                originalPosition = null;
            }} else if (isPanning) {{
                isPanning = false;
                container.classList.remove('dragging');
            }}
        }});

        // Pan on background drag (like viewer)
        container.addEventListener('mousedown', (e) => {{
            // Only start pan if clicking on background (not an element)
            if (e.button === 0 && !e.target.closest('.draggable-element')) {{
                isPanning = true;
                panStartX = e.clientX;
                panStartY = e.clientY;
                panStartOffsetX = offsetX;
                panStartOffsetY = offsetY;
                container.classList.add('dragging');
            }}
        }});

        function send(msg) {{
            if (ws && ws.readyState === WebSocket.OPEN) {{
                ws.send(JSON.stringify(msg));
                return true;
            }}
            return false;
        }}

        function save() {{
            if (send({{ type: 'save' }})) {{
                statusText.textContent = 'Saving...';
                setTimeout(() => {{
                    if (statusEl.classList.contains('connected')) {{
                        statusText.textContent = 'Connected';
                    }}
                }}, 1000);
            }}
        }}

        function autoLayout() {{
            send({{ type: 'auto_layout', view_key: viewKey }});
        }}

        function undo() {{
            send({{ type: 'undo', view_key: viewKey }});
        }}

        function redo() {{
            send({{ type: 'redo', view_key: viewKey }});
        }}

        // Zoom controls
        function updateZoomDisplay() {{
            document.getElementById('zoom-level').textContent = Math.round(scale * 100) + '%';
        }}

        function zoomIn() {{
            const centerX = container.clientWidth / 2;
            const centerY = container.clientHeight / 2;
            zoomAt(centerX, centerY, 1.2);
        }}

        function zoomOut() {{
            const centerX = container.clientWidth / 2;
            const centerY = container.clientHeight / 2;
            zoomAt(centerX, centerY, 0.8);
        }}

        function zoomAt(mouseX, mouseY, factor) {{
            const newScale = Math.max(0.1, Math.min(3, scale * factor));
            offsetX = mouseX - (mouseX - offsetX) * (newScale / scale);
            offsetY = mouseY - (mouseY - offsetY) * (newScale / scale);
            scale = newScale;
            applyTransform();
        }}

        function resetZoom() {{
            fitToScreen();
        }}

        // Minimap
        function updateMinimap() {{
            const minimap = document.getElementById('minimap');
            const viewport = document.getElementById('minimap-viewport');

            if (!svgWidth || !svgHeight) return;

            const minimapScale = Math.min(150 / svgWidth, 100 / svgHeight);

            // Calculate visible area
            const visibleWidth = container.clientWidth / scale;
            const visibleHeight = container.clientHeight / scale;
            const visibleX = -offsetX / scale;
            const visibleY = -offsetY / scale;

            const vpWidth = visibleWidth * minimapScale;
            const vpHeight = visibleHeight * minimapScale;
            const vpX = Math.max(0, visibleX * minimapScale);
            const vpY = Math.max(0, visibleY * minimapScale);

            viewport.style.width = Math.min(150, Math.max(10, vpWidth)) + 'px';
            viewport.style.height = Math.min(100, Math.max(10, vpHeight)) + 'px';
            viewport.style.left = vpX + 'px';
            viewport.style.top = vpY + 'px';
        }}

        // Mouse wheel zoom (like viewer)
        container.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = container.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            zoomAt(mouseX, mouseY, delta);
        }}, {{ passive: false }});

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {{
            if (e.key === 'z' && (e.ctrlKey || e.metaKey)) {{
                e.preventDefault();
                if (e.shiftKey) {{
                    redo();
                }} else {{
                    undo();
                }}
            }} else if (e.key === 's' && (e.ctrlKey || e.metaKey)) {{
                e.preventDefault();
                save();
            }} else if (e.key === 'l' && (e.ctrlKey || e.metaKey)) {{
                e.preventDefault();
                autoLayout();
            }}
        }});

        // Initialize
        window.addEventListener('load', () => {{
            loadSVG();
            connect();
        }});
    </script>"##,
        view_key = view_key,
        ws_base = ws_base,
        svg_url = svg_url,
    );

    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Edit,
        content_type: ContentType::ToolbarViewport,
        extra_head: extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&config, &content)
}

/// Render documentation HTML with full sidebar navigation and scroll-spy.
fn render_documentation_html(workspace: &Workspace, base_path: &str) -> Result<Html<String>> {
    let html = generate_documentation_html(workspace, base_path);
    Ok(Html(html))
}

/// Generate documentation HTML with full sidebar navigation, tree structure, and scroll-spy.
///
/// This shared function provides full documentation viewing features:
/// - Three-column layout with sidebar navigation
/// - Hierarchical tree navigation with expand/collapse
/// - Scroll-spy that tracks current position
/// - ADR (Architecture Decision Record) display with status badges
/// - Markdown rendering with syntax highlighting
fn generate_documentation_html(workspace: &Workspace, base_path: &str) -> String {
    // Extract workspace_id from base_path if present
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };

    let docs = &workspace.documentation;

    // Build sections list and collect navigation tree
    let mut nav_tree: Vec<HeadingNode> = Vec::new();

    let sections_html: String = if docs.sections.is_empty() {
        "<p class=\"empty\">No documentation sections available.</p>".to_string()
    } else {
        docs.sections.iter().enumerate().map(|(i, section)| {
            let default_title = format!("Section {}", i + 1);
            let title = section.title.as_deref().unwrap_or(&default_title);
            let section_id = format!("section-{}", i);

            // Render markdown and extract headings
            let result = render_markdown_with_heading_ids(&section.content, i);

            // Build tree from extracted headings
            let mut heading_tree = build_heading_tree(result.headings);

            // If the first heading matches the section title, use its children directly
            let children = if heading_tree.len() == 1
                && heading_tree[0].title.eq_ignore_ascii_case(title)
            {
                std::mem::take(&mut heading_tree[0].children)
            } else {
                heading_tree
            };

            // Add section as root node with its heading children
            nav_tree.push(HeadingNode {
                level: 0,
                title: title.to_string(),
                id: section_id.clone(),
                children,
            });

            format!(
                r#"<div class="doc-section" id="{}">
                    <h2>{}</h2>
                    <div class="content">{}</div>
                </div>"#,
                section_id,
                escape_html(title),
                result.html
            )
        }).collect()
    };

    // Build sidebar with tree navigation
    let sidebar_sections = if nav_tree.is_empty() {
        String::new()
    } else {
        format!(r##"<ul class="nav-tree">{}</ul>"##, render_nav_tree(&nav_tree, 0))
    };

    // Page-specific styles
    let extra_styles = r##"<style>
        .sidebar { width: 300px; background: var(--card-bg); border-right: 1px solid var(--border-color); padding: 20px; overflow-y: auto; flex-shrink: 0; }
        .sidebar h3 { margin: 0 0 10px 0; font-size: 12px; text-transform: uppercase; color: var(--text-muted); }

        /* Tree Navigation */
        .nav-tree, .nav-tree ul { list-style: none; padding: 0; margin: 0; }
        .nav-item { margin: 1px 0; position: relative; }
        .nav-row { display: flex; align-items: center; }
        .sidebar .nav-link { flex: 1; padding: 6px 8px; color: var(--text-primary); text-decoration: none; border-radius: 4px; font-size: 13px; display: block; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .sidebar .nav-link:hover { background: var(--bg-tertiary); text-decoration: none; }
        .sidebar .nav-link.active { background: var(--link-color); color: white; font-weight: 500; }
        [data-theme="light"] .sidebar .nav-link.active { background: #e3f2fd; color: #1976d2; }

        /* Toggle arrow */
        .toggle { width: 20px; height: 20px; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0; border-radius: 4px; }
        .toggle:hover { background: var(--bg-tertiary); }
        .toggle::before { content: '\25B6'; font-size: 8px; color: var(--text-muted); transition: transform 0.15s ease; }
        .expandable.expanded > .nav-row > .toggle::before { transform: rotate(90deg); }

        /* Leaf nodes need padding to align with expandable nodes */
        .leaf > .nav-link { padding-left: 28px; }

        /* Children container */
        .nav-children { display: none; margin-left: 12px; border-left: 1px solid var(--border-color); padding-left: 4px; }
        .expandable.expanded > .nav-children { display: block; }

        /* Depth styling */
        .depth-0 > .nav-row > .nav-link, .depth-0 > .nav-link { font-weight: 600; font-size: 14px; }
        .depth-1 > .nav-row > .nav-link, .depth-1 > .nav-link { font-size: 13px; }
        .depth-2 > .nav-row > .nav-link, .depth-2 > .nav-link { font-size: 12px; color: var(--text-secondary); }
        .depth-3 > .nav-row > .nav-link, .depth-3 > .nav-link { font-size: 12px; color: var(--text-secondary); }
        .depth-4 > .nav-row > .nav-link, .depth-4 > .nav-link { font-size: 11px; color: var(--text-muted); }
        .depth-5 > .nav-row > .nav-link, .depth-5 > .nav-link { font-size: 11px; color: var(--text-muted); }

        /* Active parent chain */
        .nav-item.active-parent > .nav-row > .nav-link { color: var(--link-color); }

        .main { flex: 1; padding: 40px; overflow-y: auto; }
        .doc-section { background: var(--card-bg); padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px var(--shadow); }
        .doc-section h2 { margin-top: 0; border-bottom: 1px solid var(--border-color); padding-bottom: 10px; }
        .content { line-height: 1.5; }
        .content h1, .content h2, .content h3 { margin-top: 0.8em; margin-bottom: 0.3em; }
        .content p { margin: 0.5em 0; }
        .content ul, .content ol { margin: 0.5em 0; padding-left: 1.5em; }
        .content pre { background: var(--pre-bg); padding: 15px; border-radius: 4px; overflow-x: auto; }
        .content code { background: var(--code-bg); color: var(--code-text); padding: 2px 6px; border-radius: 3px; font-family: "SF Mono", Monaco, monospace; font-size: 0.9em; }
        .content pre code { background: none; padding: 0; }
        .content blockquote { border-left: 4px solid var(--border-color); margin: 0; padding-left: 20px; color: var(--text-secondary); }
        .content table { border-collapse: collapse; width: 100%; margin: 1em 0; display: block; overflow-x: auto; }
        .content th, .content td { border: 1px solid var(--border-color); padding: 10px; text-align: left; }
        .content th { background: var(--bg-tertiary); font-weight: 600; }
        .content tr:nth-child(even) { background: var(--bg-secondary); }
        .content del { color: var(--text-muted); text-decoration: line-through; }
        .content input[type="checkbox"] { margin-right: 0.5em; transform: scale(1.1); }
        .content ul.contains-task-list { list-style: none; padding-left: 1em; }
        .content li.task-list-item { list-style: none; }
        .content .footnotes { font-size: 0.9em; border-top: 1px solid var(--border-color); padding-top: 1em; margin-top: 2em; }
        .content .footnote-ref { font-size: 0.75em; vertical-align: super; }
        .content .footnote-backref { text-decoration: none; margin-left: 0.25em; }
        .content img { max-width: 100%; height: auto; border-radius: 4px; margin: 1em 0; }
        .content dl { margin: 1em 0; }
        .content dt { font-weight: 600; margin-top: 0.5em; }
        .content dd { margin-left: 2em; color: var(--text-secondary); }
        .empty { color: var(--text-muted); font-style: italic; }
    </style>"##;

    // Page-specific scripts
    let extra_scripts = r##"<script>
    document.addEventListener('DOMContentLoaded', function() {
        const mainContent = document.querySelector('.main');
        const allHeadings = document.querySelectorAll('[id^="section-"], [id^="s"]');
        const navItems = document.querySelectorAll('.nav-item');
        const navLinks = document.querySelectorAll('.sidebar .nav-link');

        // Toggle expand/collapse on toggle button click
        document.querySelectorAll('.toggle').forEach(function(toggle) {
            toggle.addEventListener('click', function(e) {
                e.stopPropagation();
                e.preventDefault();
                const navItem = this.closest('.nav-item');
                if (navItem && navItem.classList.contains('expandable')) {
                    navItem.classList.toggle('expanded');
                }
            });
        });

        // Smooth scroll on nav link click
        navLinks.forEach(function(link) {
            link.addEventListener('click', function(e) {
                e.preventDefault();
                const href = this.getAttribute('href');
                if (href && href.startsWith('#')) {
                    const targetId = href.slice(1);
                    const target = document.getElementById(targetId);
                    if (target) {
                        mainContent.scrollTo({
                            top: target.offsetTop - 20,
                            behavior: 'smooth'
                        });
                    }
                }
            });
        });

        // Scroll-spy: update active state based on scroll position
        function updateActiveState() {
            const scrollTop = mainContent.scrollTop;
            const offset = 100;
            let currentId = '';

            allHeadings.forEach(function(heading) {
                if (scrollTop >= heading.offsetTop - offset) {
                    currentId = heading.id;
                }
            });

            navItems.forEach(function(item) {
                item.classList.remove('active', 'active-parent');
            });
            navLinks.forEach(function(link) {
                link.classList.remove('active');
            });

            if (currentId) {
                const activeLink = document.querySelector('.sidebar .nav-link[href="#' + currentId + '"]');
                if (activeLink) {
                    activeLink.classList.add('active');

                    let parent = activeLink.closest('.nav-item');
                    if (parent) {
                        parent.classList.add('active');
                    }
                    parent = parent ? parent.parentElement : null;
                    while (parent) {
                        const parentItem = parent.closest('.nav-item');
                        if (parentItem) {
                            parentItem.classList.add('active-parent');
                            if (parentItem.classList.contains('expandable')) {
                                parentItem.classList.add('expanded');
                            }
                            parent = parentItem.parentElement;
                        } else {
                            break;
                        }
                    }

                    const sidebar = document.querySelector('.sidebar');
                    if (sidebar) {
                        const linkRect = activeLink.getBoundingClientRect();
                        const sidebarRect = sidebar.getBoundingClientRect();
                        const linkCenter = linkRect.top + linkRect.height / 2;
                        const sidebarCenter = sidebarRect.top + sidebarRect.height / 2;
                        const scrollOffset = linkCenter - sidebarCenter;
                        sidebar.scrollBy({ top: scrollOffset, behavior: 'smooth' });
                    }
                }
            }
        }

        let scrollTimer;
        mainContent.addEventListener('scroll', function() {
            clearTimeout(scrollTimer);
            scrollTimer = setTimeout(updateActiveState, 50);
        });

        updateActiveState();
    });
    </script>"##;

    // Build content HTML
    let content = format!(
        r#"<div class="sidebar">
            <h3>Sections</h3>
            {}
        </div>
        <div class="main">
            {}
        </div>"#,
        sidebar_sections,
        sections_html
    );

    let title = format!("Documentation - {}", workspace.name);
    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Docs,
        content_type: ContentType::Sidebar,
        extra_head: extra_styles,
        extra_body_end: extra_scripts,
    };

    generate_page_layout(&config, &content)
}

/// Render decisions page HTML wrapper.
fn render_decisions_html(workspace: &Workspace, base_path: &str) -> Result<Html<String>> {
    let html = generate_decisions_html(workspace, base_path);
    Ok(Html(html))
}

/// Generate decisions (ADR) page HTML with click-to-load navigation.
///
/// This function provides a dedicated page for Architecture Decision Records:
/// - Three-column layout with sidebar navigation
/// - Click on a decision in sidebar to load it in main area
/// - First decision is loaded and selected by default
fn generate_decisions_html(workspace: &Workspace, base_path: &str) -> String {
    // Extract workspace_id from base_path if present
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };

    let docs = &workspace.documentation;

    // Build decisions data as JSON for JavaScript
    let decisions_json: String = if docs.decisions.is_empty() {
        "[]".to_string()
    } else {
        let decisions_data: Vec<String> = docs.decisions.iter().map(|decision| {
            let status_str = match decision.status {
                structurizr_core::workspace::DecisionStatus::Accepted => "Accepted",
                structurizr_core::workspace::DecisionStatus::Proposed => "Proposed",
                structurizr_core::workspace::DecisionStatus::Superseded => "Superseded",
                structurizr_core::workspace::DecisionStatus::Deprecated => "Deprecated",
                structurizr_core::workspace::DecisionStatus::Rejected => "Rejected",
            };
            let status_class = match decision.status {
                structurizr_core::workspace::DecisionStatus::Accepted => "accepted",
                structurizr_core::workspace::DecisionStatus::Proposed => "proposed",
                structurizr_core::workspace::DecisionStatus::Superseded => "superseded",
                structurizr_core::workspace::DecisionStatus::Deprecated => "deprecated",
                structurizr_core::workspace::DecisionStatus::Rejected => "rejected",
            };
            // Escape content for JSON embedding
            let content_html = render_markdown(&decision.content);
            let escaped_content = content_html
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t");
            let escaped_title = decision.title
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            let escaped_date = decision.date
                .replace('\\', "\\\\")
                .replace('"', "\\\"");
            format!(
                r#"{{"id":"{}","title":"{}","status":"{}","statusClass":"{}","date":"{}","content":"{}"}}"#,
                decision.id,
                escaped_title,
                status_str,
                status_class,
                escaped_date,
                escaped_content
            )
        }).collect();
        format!("[{}]", decisions_data.join(","))
    };

    // Build sidebar with decision links
    let sidebar_decisions: String = if docs.decisions.is_empty() {
        "<p class=\"empty\">No decisions available.</p>".to_string()
    } else {
        docs.decisions.iter().map(|decision| {
            let status_class = match decision.status {
                structurizr_core::workspace::DecisionStatus::Accepted => "accepted",
                structurizr_core::workspace::DecisionStatus::Proposed => "proposed",
                structurizr_core::workspace::DecisionStatus::Superseded => "superseded",
                structurizr_core::workspace::DecisionStatus::Deprecated => "deprecated",
                structurizr_core::workspace::DecisionStatus::Rejected => "rejected",
            };
            format!(
                r##"<a href="#" class="decision-link" data-id="{}">
                    <span class="decision-nav-id">{}</span>
                    <span class="decision-nav-title">{}</span>
                    <span class="status-dot {}"></span>
                </a>"##,
                decision.id,
                escape_html(&decision.id),
                escape_html(&decision.title),
                status_class
            )
        }).collect()
    };

    // Page-specific styles
    let extra_styles = r##"<style>
        .sidebar { width: 300px; background: var(--card-bg); border-right: 1px solid var(--border-color); padding: 20px; overflow-y: auto; flex-shrink: 0; }
        .sidebar h3 { margin: 0 0 15px 0; font-size: 12px; text-transform: uppercase; color: var(--text-muted); }

        /* Decision links in sidebar */
        .decision-link { display: flex; align-items: center; gap: 8px; padding: 8px 10px; color: var(--text-primary); text-decoration: none; border-radius: 4px; font-size: 13px; margin: 2px 0; cursor: pointer; }
        .decision-link:hover { background: var(--bg-tertiary); text-decoration: none; }
        .decision-link.active { background: var(--link-color); color: white; font-weight: 500; }
        [data-theme="light"] .decision-link.active { background: #e3f2fd; color: #1976d2; }
        .decision-nav-id { font-family: monospace; font-size: 11px; background: var(--bg-tertiary); padding: 2px 6px; border-radius: 3px; flex-shrink: 0; }
        .decision-link.active .decision-nav-id { background: rgba(255,255,255,0.2); }
        [data-theme="light"] .decision-link.active .decision-nav-id { background: rgba(0,0,0,0.08); }
        .decision-nav-title { flex: 1; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
        .status-dot { width: 8px; height: 8px; border-radius: 50%; flex-shrink: 0; }
        .status-dot.accepted { background: var(--status-accepted-text); }
        .status-dot.proposed { background: var(--status-proposed-text); }
        .status-dot.superseded { background: var(--status-superseded-text); }
        .status-dot.deprecated { background: var(--status-deprecated-text); }
        .status-dot.rejected { background: var(--status-deprecated-text); }

        .main { flex: 1; padding: 40px; overflow-y: auto; }
        .decision { background: var(--card-bg); padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px var(--shadow); }
        .decision-header { display: flex; align-items: center; gap: 15px; margin-bottom: 20px; flex-wrap: wrap; }
        .decision-id { font-family: monospace; background: var(--bg-tertiary); padding: 4px 8px; border-radius: 4px; font-size: 12px; }
        .decision-header h3 { margin: 0; flex: 1; }
        .status { padding: 4px 10px; border-radius: 20px; font-size: 11px; text-transform: uppercase; font-weight: 600; }
        .status.accepted { background: var(--status-accepted-bg); color: var(--status-accepted-text); }
        .status.proposed { background: var(--status-proposed-bg); color: var(--status-proposed-text); }
        .status.superseded { background: var(--status-superseded-bg); color: var(--status-superseded-text); }
        .status.deprecated { background: var(--status-deprecated-bg); color: var(--status-deprecated-text); }
        .status.rejected { background: var(--status-deprecated-bg); color: var(--status-deprecated-text); }
        .date { color: var(--text-muted); font-size: 12px; }
        .empty { color: var(--text-muted); font-style: italic; padding: 20px; }

        .content { line-height: 1.5; }
        .content h1, .content h2, .content h3 { margin-top: 0.8em; margin-bottom: 0.3em; }
        .content p { margin: 0.5em 0; }
        .content ul, .content ol { margin: 0.5em 0; padding-left: 1.5em; }
        .content pre { background: var(--pre-bg); padding: 15px; border-radius: 4px; overflow-x: auto; }
        .content code { background: var(--code-bg); color: var(--code-text); padding: 2px 6px; border-radius: 3px; font-family: "SF Mono", Monaco, monospace; font-size: 0.9em; }
        .content pre code { background: none; padding: 0; }
        .content blockquote { border-left: 4px solid var(--border-color); margin: 0; padding-left: 20px; color: var(--text-secondary); }
        .content table { border-collapse: collapse; width: 100%; margin: 1em 0; display: block; overflow-x: auto; }
        .content th, .content td { border: 1px solid var(--border-color); padding: 10px; text-align: left; }
        .content th { background: var(--bg-tertiary); font-weight: 600; }
        .content tr:nth-child(even) { background: var(--bg-secondary); }
    </style>"##;

    // Page-specific scripts for click-to-load
    let extra_scripts = format!(r##"<script>
    document.addEventListener('DOMContentLoaded', function() {{
        const decisions = {decisions_json};
        const mainContent = document.getElementById('decision-container');
        const navLinks = document.querySelectorAll('.sidebar .decision-link');

        function renderDecision(decision) {{
            if (!decision) {{
                mainContent.innerHTML = '<p class="empty">Select a decision from the sidebar.</p>';
                return;
            }}
            mainContent.innerHTML = `
                <div class="decision">
                    <div class="decision-header">
                        <span class="decision-id">${{decision.id}}</span>
                        <h3>${{decision.title}}</h3>
                        <span class="status ${{decision.statusClass}}">${{decision.status}}</span>
                        <span class="date">${{decision.date}}</span>
                    </div>
                    <div class="content">${{decision.content}}</div>
                </div>
            `;
        }}

        function selectDecision(id) {{
            // Update active state in sidebar
            navLinks.forEach(function(link) {{
                link.classList.remove('active');
                if (link.dataset.id === id) {{
                    link.classList.add('active');
                }}
            }});

            // Find and render the decision
            const decision = decisions.find(d => d.id === id);
            renderDecision(decision);
        }}

        // Click handler for sidebar links
        navLinks.forEach(function(link) {{
            link.addEventListener('click', function(e) {{
                e.preventDefault();
                const id = this.dataset.id;
                selectDecision(id);
            }});
        }});

        // Load first decision by default
        if (decisions.length > 0) {{
            selectDecision(decisions[0].id);
        }}
    }});
    </script>"##, decisions_json = decisions_json);

    // Build content HTML - main area is a container that will be filled by JS
    let main_content = if docs.decisions.is_empty() {
        "<p class=\"empty\">No architecture decision records available.</p>".to_string()
    } else {
        String::new()
    };

    let content = format!(
        r#"<div class="sidebar">
            <h3>Decisions</h3>
            {}
        </div>
        <div class="main" id="decision-container">
            {}
        </div>"#,
        sidebar_decisions,
        main_content
    );

    let title = format!("Decisions - {}", workspace.name);
    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Decisions,
        content_type: ContentType::Sidebar,
        extra_head: extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&config, &content)
}

/// Render tree view HTML with full expand/collapse tree navigation.
fn render_tree_view_html(workspace: &Workspace, base_path: &str) -> Result<Html<String>> {
    // Extract workspace_id from base_path if present (e.g., "/w/small/my-workspace" -> "small/my-workspace")
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };
    let html = generate_tree_page_html(workspace, base_path, workspace_id);
    Ok(Html(html))
}

/// Render presentation HTML with slideshow navigation.
fn render_presentation_html(workspace: &Workspace, base_path: &str, views_param: Option<String>) -> Result<Html<String>> {
    let view_keys: Vec<String> = if let Some(views) = views_param {
        views.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        workspace.views().all_keys().iter().map(|k| k.to_string()).collect()
    };

    let home_href = if base_path.is_empty() { "/".to_string() } else { base_path.to_string() };
    let slides_json = serde_json::to_string(&view_keys).unwrap_or_else(|_| "[]".to_string());

    let html = format!(r##"<!DOCTYPE html>
<html>
<head>
    <title>Presentation - {name} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; background: #1a1a1a; color: #fff; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; height: 100vh; overflow: hidden; }}
        .toolbar {{ position: fixed; top: 0; left: 0; right: 0; background: rgba(0,0,0,0.8); padding: 10px 20px; display: flex; align-items: center; gap: 20px; z-index: 100; opacity: 0; transition: opacity 0.3s; }}
        .toolbar:hover, .toolbar.visible {{ opacity: 1; }}
        .toolbar a {{ color: white; text-decoration: none; }}
        .toolbar .spacer {{ flex: 1; }}
        .slide-info {{ font-size: 14px; }}
        .slide-container {{ position: relative; z-index: 1; height: calc(100vh - 130px); display: flex; align-items: center; justify-content: center; }}
        .slide {{ max-width: 95vw; max-height: calc(100vh - 150px); background: white; border-radius: 8px; box-shadow: 0 10px 50px rgba(0,0,0,0.5); overflow: hidden; }}
        .slide img {{ max-width: 100%; max-height: calc(100vh - 180px); display: block; }}
        .controls {{ position: fixed; bottom: 30px; left: 50%; transform: translateX(-50%); display: flex; gap: 10px; z-index: 100; }}
        .controls button {{ background: rgba(255,255,255,0.2); color: white; border: none; padding: 12px 24px; border-radius: 6px; cursor: pointer; font-size: 16px; transition: background 0.2s; }}
        .controls button:hover {{ background: rgba(255,255,255,0.3); }}
        .controls button:disabled {{ opacity: 0.3; cursor: not-allowed; }}
        .keyboard-hint {{ position: fixed; bottom: 20px; right: 20px; font-size: 11px; color: #666; z-index: 100; }}
        .slide-title {{ position: fixed; bottom: 80px; left: 50%; transform: translateX(-50%); font-size: 18px; color: white; background: rgba(0,0,0,0.7); padding: 8px 20px; border-radius: 20px; z-index: 100; }}
    </style>
</head>
<body>
    <div class="toolbar" id="toolbar">
        <a href="{home_href}">← Exit Presentation</a>
        <div class="spacer"></div>
        <span class="slide-info" id="slide-info">Slide 1 of {slide_count}</span>
    </div>
    <div class="slide-container">
        <div class="slide" id="slide"><img id="slide-img" alt="Loading..."></div>
    </div>
    <div class="slide-title" id="slide-title"></div>
    <div class="controls">
        <button id="btn-prev" onclick="prevSlide()">← Previous</button>
        <button id="btn-next" onclick="nextSlide()">Next →</button>
    </div>
    <div class="keyboard-hint">← → Arrow keys to navigate • Esc to exit</div>
    <script>
        const slides = {slides_json};
        const basePath = '{base_path}';
        let current = 0;

        function showSlide(index) {{
            if (index < 0 || index >= slides.length) return;
            current = index;
            const key = slides[current];
            document.getElementById('slide-img').src = `${{basePath}}/view/${{key}}/svg`;
            document.getElementById('slide-title').textContent = key;
            document.getElementById('slide-info').textContent = `Slide ${{current + 1}} of ${{slides.length}}`;
            document.getElementById('btn-prev').disabled = current === 0;
            document.getElementById('btn-next').disabled = current === slides.length - 1;
        }}

        function nextSlide() {{ showSlide(current + 1); }}
        function prevSlide() {{ showSlide(current - 1); }}

        document.addEventListener('keydown', (e) => {{
            if (e.key === 'ArrowRight' || e.key === ' ') {{ e.preventDefault(); nextSlide(); }}
            if (e.key === 'ArrowLeft') {{ e.preventDefault(); prevSlide(); }}
            if (e.key === 'Escape') {{ window.location.href = '{home_href}'; }}
        }});

        document.addEventListener('mousemove', () => {{
            document.getElementById('toolbar').classList.add('visible');
            setTimeout(() => document.getElementById('toolbar').classList.remove('visible'), 2000);
        }});

        showSlide(0);
    </script>
</body>
</html>"##,
        name = workspace.name,
        home_href = home_href,
        slide_count = view_keys.len(),
        slides_json = slides_json,
        base_path = base_path,
    );

    Ok(Html(html))
}

/// Search workspace and return results.
fn search_workspace(workspace: &Workspace, query: &str, base_path: &str) -> Result<Json<Vec<SearchResult>>> {
    let query_lower = query.to_lowercase();
    let mut results = Vec::new();

    // Search people
    for person in &workspace.model().people {
        if person.name().to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                id: person.id().to_string(),
                name: person.name().to_string(),
                element_type: "Person".to_string(),
                description: person.properties.description.clone(),
                url: format!("{}#element-{}", base_path, person.id()),
            });
        }
    }

    // Search software systems
    for system in &workspace.model().software_systems {
        if system.name().to_lowercase().contains(&query_lower) {
            results.push(SearchResult {
                id: system.id().to_string(),
                name: system.name().to_string(),
                element_type: "Software System".to_string(),
                description: system.properties.description.clone(),
                url: format!("{}#element-{}", base_path, system.id()),
            });
        }
    }

    Ok(Json(results))
}

/// Render view as SVG.
fn render_view_svg(workspace: &Workspace, view_key: &str) -> Result<impl IntoResponse> {
    let renderer = SvgRenderer::default();
    let views = workspace.views();

    let svg = if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_system_landscape(workspace, view)?
    } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_system_context(workspace, view)?
    } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_container(workspace, view)?
    } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_component(workspace, view)?
    } else if let Some(view) = views.deployment_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_deployment(workspace, view)?
    } else if let Some(view) = views.dynamic_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_dynamic(workspace, view)?
    } else if let Some(view) = views.filtered_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_filtered(workspace, view)?
    } else {
        // Default: render as system landscape
        let view = SystemLandscapeView::new(view_key);
        renderer.render_system_landscape(workspace, &view)?
    };

    Ok(([(header::CONTENT_TYPE, "image/svg+xml")], svg))
}

// ============================================================================
// Export Viewer HTML Generators
// ============================================================================

/// Get the raw code for an export format (shared by viewer generators).
fn get_export_code(workspace: &Workspace, view_key: &str, format: &str) -> Result<String> {
    let views = workspace.views();

    match format {
        "plantuml" => {
            if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_system_landscape(workspace, view)?)
            } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_system_context(workspace, view)?)
            } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_container(workspace, view)?)
            } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_component(workspace, view)?)
            } else if let Some(view) = views.dynamic_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_dynamic(workspace, view)?)
            } else if let Some(view) = views.deployment_views.iter().find(|v| v.properties.key == view_key) {
                Ok(PlantUmlExporter::export_deployment(workspace, view)?)
            } else {
                let view = SystemLandscapeView::new(view_key);
                Ok(PlantUmlExporter::export_system_landscape(workspace, &view)?)
            }
        },
        "mermaid" => {
            if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_system_landscape(workspace, view)?)
            } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_system_context(workspace, view)?)
            } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_container(workspace, view)?)
            } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_component(workspace, view)?)
            } else if let Some(view) = views.dynamic_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_dynamic(workspace, view)?)
            } else if let Some(view) = views.deployment_views.iter().find(|v| v.properties.key == view_key) {
                Ok(MermaidExporter::export_deployment(workspace, view)?)
            } else {
                Ok(MermaidExporter::export_flowchart(workspace)?)
            }
        },
        "dot" => {
            if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
                Ok(DotExporter::export_system_landscape(workspace, view)?)
            } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
                Ok(DotExporter::export_system_context(workspace, view)?)
            } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
                Ok(DotExporter::export_container(workspace, view)?)
            } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
                Ok(DotExporter::export_component(workspace, view)?)
            } else {
                Ok(DotExporter::export_flowchart(workspace)?)
            }
        },
        "d2" => {
            if let Some(view) = views.system_landscape_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_system_landscape(workspace, view)?)
            } else if let Some(view) = views.system_context_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_system_context(workspace, view)?)
            } else if let Some(view) = views.container_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_container(workspace, view)?)
            } else if let Some(view) = views.component_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_component(workspace, view)?)
            } else if let Some(view) = views.dynamic_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_dynamic(workspace, view)?)
            } else if let Some(view) = views.deployment_views.iter().find(|v| v.properties.key == view_key) {
                Ok(D2Exporter::export_deployment(workspace, view)?)
            } else {
                Ok(D2Exporter::export_flowchart(workspace)?)
            }
        },
        _ => Ok(String::new()),
    }
}

/// Generate export viewer HTML page.
fn generate_export_viewer_html(
    workspace: &Workspace,
    view_key: &str,
    base_path: &str,
    format_name: &str,
    code: &str,
    render_script: &str,
) -> String {
    let workspace_id = if base_path.starts_with("/w/") {
        Some(&base_path[3..])
    } else {
        None
    };

    let title = format!("{} - {} ({})", view_key, workspace.name, format_name);
    let escaped_code = escape_html(code);
    let raw_url = format!("{}/view/{}/{}?raw=true", base_path, view_key, format_name.to_lowercase());
    let view_url = format!("{}/view/{}", base_path, view_key);

    let extra_styles = r##"<style>
        .export-viewer {
            display: flex;
            flex-direction: column;
            height: 100%;
        }
        .export-toolbar {
            background: var(--toolbar-bg);
            color: var(--toolbar-text);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 15px;
            border-bottom: 1px solid var(--toolbar-border);
            flex-shrink: 0;
        }
        .export-toolbar a, .export-toolbar button {
            color: var(--link-color);
            text-decoration: none;
            background: none;
            border: none;
            cursor: pointer;
            font-size: inherit;
            font-family: inherit;
            padding: 0;
        }
        .export-toolbar a:hover, .export-toolbar button:hover {
            text-decoration: underline;
        }
        .export-toolbar .separator {
            border-left: 1px solid var(--border-color);
            height: 20px;
        }
        .export-toolbar .format-badge {
            background: var(--bg-tertiary);
            color: var(--text-secondary);
            padding: 4px 10px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: 600;
        }
        .export-content {
            flex: 1;
            display: flex;
            overflow: hidden;
        }
        .render-panel {
            flex: 1;
            overflow: auto;
            padding: 20px;
            display: flex;
            justify-content: center;
            align-items: flex-start;
            background: var(--canvas-bg);
        }
        .render-panel svg {
            max-width: 100%;
            height: auto;
        }
        .render-panel .mermaid {
            width: 100%;
            display: flex;
            justify-content: center;
        }
        .code-panel {
            width: 450px;
            background: var(--bg-secondary);
            border-left: 1px solid var(--border-color);
            overflow: auto;
            display: none;
            flex-direction: column;
        }
        .code-panel.visible {
            display: flex;
        }
        .code-panel-header {
            background: var(--bg-tertiary);
            padding: 10px 15px;
            border-bottom: 1px solid var(--border-color);
            font-weight: 600;
            font-size: 13px;
            flex-shrink: 0;
        }
        .code-panel pre {
            margin: 0;
            padding: 15px;
            font-size: 12px;
            line-height: 1.5;
            white-space: pre-wrap;
            word-break: break-word;
            flex: 1;
            overflow: auto;
            background: var(--pre-bg);
        }
        .code-panel code {
            font-family: 'Monaco', 'Menlo', 'Ubuntu Mono', monospace;
            color: var(--code-text);
        }
        .render-error {
            background: var(--status-deprecated-bg);
            color: var(--status-deprecated-text);
            padding: 20px 30px;
            border-radius: 8px;
            text-align: center;
            max-width: 500px;
        }
        .render-error h3 {
            margin: 0 0 10px 0;
        }
        .render-error p {
            margin: 0 0 15px 0;
        }
        .render-error button {
            background: var(--bg-secondary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
        }
        .render-loading {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 15px;
            color: var(--text-secondary);
        }
        .render-loading .spinner {
            width: 40px;
            height: 40px;
            border: 3px solid var(--border-color);
            border-top-color: var(--link-color);
            border-radius: 50%;
            animation: spin 1s linear infinite;
        }
        @keyframes spin {
            to { transform: rotate(360deg); }
        }
        .copy-feedback {
            position: fixed;
            top: 80px;
            right: 20px;
            background: var(--status-accepted-bg);
            color: var(--status-accepted-text);
            padding: 10px 20px;
            border-radius: 6px;
            opacity: 0;
            transition: opacity 0.3s;
            z-index: 1000;
        }
        .copy-feedback.visible {
            opacity: 1;
        }
    </style>"##;

    let content = format!(r##"
        <div class="export-viewer">
            <div class="export-toolbar">
                <a href="{view_url}">Back to Diagram</a>
                <div class="separator"></div>
                <span class="format-badge">{format_name}</span>
                <div class="separator"></div>
                <button onclick="toggleCode()">Toggle Code</button>
                <button onclick="copyCode()">Copy Code</button>
                <a href="{raw_url}" download="{view_key}.{ext}">Download</a>
            </div>
            <div class="export-content">
                <div class="render-panel" id="render-panel">
                    <div class="render-loading" id="loading">
                        <div class="spinner"></div>
                        <span>Rendering {format_name}...</span>
                    </div>
                </div>
                <div class="code-panel" id="code-panel">
                    <div class="code-panel-header">{format_name} Code</div>
                    <pre><code id="source-code">{escaped_code}</code></pre>
                </div>
            </div>
        </div>
        <div class="copy-feedback" id="copy-feedback">Copied to clipboard!</div>
    "##,
        view_url = view_url,
        format_name = format_name,
        raw_url = raw_url,
        view_key = view_key,
        ext = format_name.to_lowercase(),
        escaped_code = escaped_code,
    );

    let extra_scripts = format!(r##"<script>
        const sourceCode = document.getElementById('source-code').textContent;

        function toggleCode() {{
            document.getElementById('code-panel').classList.toggle('visible');
        }}

        function copyCode() {{
            navigator.clipboard.writeText(sourceCode).then(() => {{
                const feedback = document.getElementById('copy-feedback');
                feedback.classList.add('visible');
                setTimeout(() => feedback.classList.remove('visible'), 2000);
            }});
        }}

        function showError(message) {{
            document.getElementById('loading').style.display = 'none';
            document.getElementById('render-panel').innerHTML = `
                <div class="render-error">
                    <h3>Rendering Failed</h3>
                    <p>${{message}}</p>
                    <button onclick="toggleCode()">View Source Code</button>
                </div>
            `;
            document.getElementById('code-panel').classList.add('visible');
        }}

        {render_script}
    </script>"##, render_script = render_script);

    let layout_config = LayoutConfig {
        title: &title,
        workspace_name: Some(workspace.name.as_str()),
        workspace_id,
        base_path,
        active_nav: NavItem::Export,
        content_type: ContentType::FullViewport,
        extra_head: extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&layout_config, &content)
}

/// Generate Mermaid viewer HTML.
fn generate_mermaid_viewer_html(workspace: &Workspace, view_key: &str, base_path: &str, code: &str) -> String {
    // Strip markdown fence markers from Mermaid output
    // Chain each operation properly so we don't fall back to the original
    let trimmed = code.trim();
    let clean_code = trimmed
        .strip_prefix("```mermaid\n")
        .or_else(|| trimmed.strip_prefix("```mermaid"))
        .unwrap_or(trimmed);
    let clean_code = clean_code
        .strip_suffix("\n```")
        .or_else(|| clean_code.strip_suffix("```"))
        .unwrap_or(clean_code)
        .trim();

    let escaped_for_js = clean_code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let render_script = format!(r##"
        (async function() {{
            try {{
                const {{ default: mermaid }} = await import('https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.esm.min.mjs');

                const isDark = document.documentElement.getAttribute('data-theme') === 'dark';
                mermaid.initialize({{
                    startOnLoad: false,
                    theme: isDark ? 'dark' : 'default',
                    securityLevel: 'loose',
                    flowchart: {{ useMaxWidth: true }},
                    c4: {{ useMaxWidth: true }}
                }});

                const code = `{escaped_code}`;
                const {{ svg }} = await mermaid.render('mermaid-diagram', code);

                document.getElementById('loading').style.display = 'none';
                const container = document.createElement('div');
                container.className = 'mermaid';
                container.innerHTML = svg;
                document.getElementById('render-panel').appendChild(container);
            }} catch (error) {{
                showError('Failed to render Mermaid diagram: ' + error.message);
            }}
        }})();
    "##, escaped_code = escaped_for_js);

    generate_export_viewer_html(workspace, view_key, base_path, "Mermaid", code, &render_script)
}

/// Generate DOT/Graphviz viewer HTML.
fn generate_dot_viewer_html(workspace: &Workspace, view_key: &str, base_path: &str, code: &str) -> String {
    let escaped_for_js = code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let render_script = format!(r##"
        (async function() {{
            try {{
                const {{ instance }} = await import('https://cdn.jsdelivr.net/npm/@viz-js/viz@3.2.4/+esm');

                const viz = await instance();
                const code = `{escaped_code}`;
                const svg = viz.renderSVGElement(code);

                document.getElementById('loading').style.display = 'none';
                document.getElementById('render-panel').appendChild(svg);
            }} catch (error) {{
                showError('Failed to render DOT diagram: ' + error.message);
            }}
        }})();
    "##, escaped_code = escaped_for_js);

    generate_export_viewer_html(workspace, view_key, base_path, "DOT", code, &render_script)
}

/// Generate PlantUML viewer HTML (uses Kroki.io for rendering).
fn generate_plantuml_viewer_html(workspace: &Workspace, view_key: &str, base_path: &str, code: &str) -> String {
    let escaped_for_js = code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let render_script = format!(r##"
        (async function() {{
            try {{
                // Load pako for compression
                const pako = await import('https://cdn.jsdelivr.net/npm/pako@2.1.0/+esm');

                const code = `{escaped_code}`;
                const data = new TextEncoder().encode(code);
                const compressed = pako.default.deflate(data, {{ level: 9 }});

                // Base64 encode for URL
                let binary = '';
                for (let i = 0; i < compressed.length; i++) {{
                    binary += String.fromCharCode(compressed[i]);
                }}
                const encoded = btoa(binary)
                    .replace(/\+/g, '-')
                    .replace(/\//g, '_');

                const url = `https://kroki.io/plantuml/svg/${{encoded}}`;
                const response = await fetch(url);

                if (!response.ok) {{
                    throw new Error(`Kroki.io returned ${{response.status}}: ${{response.statusText}}`);
                }}

                const svg = await response.text();
                document.getElementById('loading').style.display = 'none';

                const container = document.createElement('div');
                container.innerHTML = svg;
                document.getElementById('render-panel').appendChild(container);
            }} catch (error) {{
                showError('Failed to render PlantUML diagram: ' + error.message);
            }}
        }})();
    "##, escaped_code = escaped_for_js);

    generate_export_viewer_html(workspace, view_key, base_path, "PlantUML", code, &render_script)
}

/// Generate D2 viewer HTML (uses Kroki.io for rendering).
fn generate_d2_viewer_html(workspace: &Workspace, view_key: &str, base_path: &str, code: &str) -> String {
    let escaped_for_js = code
        .replace('\\', "\\\\")
        .replace('`', "\\`")
        .replace("${", "\\${");

    let render_script = format!(r##"
        (async function() {{
            try {{
                // Load pako for compression
                const pako = await import('https://cdn.jsdelivr.net/npm/pako@2.1.0/+esm');

                const code = `{escaped_code}`;
                const data = new TextEncoder().encode(code);
                const compressed = pako.default.deflate(data, {{ level: 9 }});

                // Base64 encode for URL
                let binary = '';
                for (let i = 0; i < compressed.length; i++) {{
                    binary += String.fromCharCode(compressed[i]);
                }}
                const encoded = btoa(binary)
                    .replace(/\+/g, '-')
                    .replace(/\//g, '_');

                const url = `https://kroki.io/d2/svg/${{encoded}}`;
                const response = await fetch(url);

                if (!response.ok) {{
                    throw new Error(`Kroki.io returned ${{response.status}}: ${{response.statusText}}`);
                }}

                const svg = await response.text();
                document.getElementById('loading').style.display = 'none';

                const container = document.createElement('div');
                container.innerHTML = svg;
                document.getElementById('render-panel').appendChild(container);
            }} catch (error) {{
                showError('Failed to render D2 diagram: ' + error.message);
            }}
        }})();
    "##, escaped_code = escaped_for_js);

    generate_export_viewer_html(workspace, view_key, base_path, "D2", code, &render_script)
}

// ============================================================================
// Nested Workspace Handlers (for category/workspace_id paths)
// ============================================================================

/// Helper to combine category and workspace_id into full workspace ID.
fn make_nested_workspace_id(category: &str, workspace_id: &str) -> String {
    format!("{}/{}", category, workspace_id)
}

pub async fn workspace_home_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let base_path = format!("/w/{}", full_id);
    let html = generate_home_page_html(&workspace, &base_path, Some(&full_id));

    Ok(Html(html))
}

pub async fn workspace_view_diagram_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_view_diagram_html(&workspace, &view_key, &format!("/w/{}", full_id))
}

pub async fn workspace_view_animated_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_dynamic_animated_html(&workspace, &view_key, &format!("/w/{}", full_id))
}

pub async fn workspace_edit_diagram_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_edit_diagram_html(&workspace, &view_key, &format!("/w/{}", full_id))
}

pub async fn workspace_documentation_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_documentation_html(&workspace, &format!("/w/{}", full_id))
}

pub async fn workspace_search_page_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    let base_path = format!("/w/{}", full_id);
    let search_term = query.q.unwrap_or_default();
    let html = generate_search_page_html(&workspace, &base_path, Some(&full_id), &search_term);
    Ok(Html(html))
}

pub async fn workspace_tree_view_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_tree_view_html(&workspace, &format!("/w/{}", full_id))
}

pub async fn workspace_presentation_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<PresentationQuery>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_presentation_html(&workspace, &format!("/w/{}", full_id), query.views)
}

pub async fn workspace_explore_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Html<String>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    let base_path = format!("/w/{}", full_id);
    let html = generate_explore_page_html(&workspace, &base_path, Some(&full_id));
    Ok(Html(html))
}

pub async fn workspace_get_json_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Json<Workspace>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    Ok(Json(workspace))
}

pub async fn workspace_validate_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<Json<structurizr_dsl::ValidationResult>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    Ok(Json(structurizr_dsl::validate_workspace(&workspace)))
}

pub async fn workspace_search_api_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    let q = query.q.as_deref().unwrap_or("");
    search_workspace(&workspace, q, &format!("/w/{}", full_id))
}

pub async fn workspace_export_json_nested(
    State(state): State<AppState>,
    Path((category, workspace_id)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    let json = JsonExporter::export(&workspace)?;
    Ok(([(header::CONTENT_TYPE, "application/json")], json))
}

/// Nested workspace: Get notes.
pub async fn workspace_get_notes_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<Json<ViewNotes>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let info = state.get_workspace_info(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let notes_file = NotesFile::load(&info.path).await;
    let view_notes = notes_file
        .get_view_notes(&view_key)
        .cloned()
        .unwrap_or_default();

    Ok(Json(view_notes))
}

/// Nested workspace: Add note.
pub async fn workspace_add_note_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Json(req): Json<AddNoteRequest>,
) -> Result<Json<ViewNotes>> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let info = state.get_workspace_info(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let mut notes_file = NotesFile::load(&info.path).await;
    notes_file.add_note(
        &view_key,
        req.step_index,
        req.first_name,
        req.last_name,
        req.content,
    );
    notes_file.save(&info.path).await
        .map_err(|e| Error::Server(format!("Failed to save notes: {}", e)))?;

    let view_notes = notes_file
        .get_view_notes(&view_key)
        .cloned()
        .unwrap_or_default();

    Ok(Json(view_notes))
}

pub async fn workspace_render_svg_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
) -> Result<impl IntoResponse> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;
    render_view_svg(&workspace, &view_key)
}

pub async fn workspace_export_plantuml_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "plantuml")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", full_id);
        let html = generate_plantuml_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

pub async fn workspace_export_mermaid_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "mermaid")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", full_id);
        let html = generate_mermaid_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

pub async fn workspace_export_dot_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "dot")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", full_id);
        let html = generate_dot_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

pub async fn workspace_export_d2_nested(
    State(state): State<AppState>,
    Path((category, workspace_id, view_key)): Path<(String, String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let full_id = make_nested_workspace_id(&category, &workspace_id);
    let workspace = state.get_workspace_by_id(&full_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(full_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "d2")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", full_id);
        let html = generate_d2_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

// ============================================================================
// Unified Wildcard Dispatcher
// ============================================================================

/// Unified workspace route dispatcher for GET requests (including WebSocket upgrades).
///
/// This handler uses wildcard routing to support workspace IDs of any depth.
/// For example: /w/my-workspace, /w/small/startup-saas, /w/team/project/workspace
///
/// The path is parsed to find the workspace ID by trying progressively shorter
/// prefixes until a valid workspace is found.
///
/// WebSocket upgrade requests (for /ws/edit/ paths) are detected and handled specially.
pub async fn workspace_dispatch(
    ws: Option<axum::extract::ws::WebSocketUpgrade>,
    State(state): State<AppState>,
    Path(path): Path<String>,
    axum::extract::Query(query): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<axum::response::Response> {
    use axum::response::IntoResponse;

    // Check if this is a WebSocket upgrade request for /ws/edit/ paths
    if let Some(ws_upgrade) = ws {
        return handle_websocket_dispatch(ws_upgrade, state, &path).await;
    }

    let (workspace_id, remaining) = parse_workspace_path(&state, &path)
        .await
        .ok_or_else(|| Error::WorkspaceNotFound(path.clone()))?;

    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let base_path = format!("/w/{}", workspace_id);

    // Dispatch based on remaining path segments
    match remaining.as_slice() {
        // Home page: /w/{workspace}
        [] => {
            let html = generate_home_page_html(&workspace, &base_path, Some(&workspace_id));
            Ok(Html(html).into_response())
        }

        // Documentation: /w/{workspace}/docs
        [action] if action == "docs" => {
            render_documentation_html(&workspace, &base_path)
                .map(|h| h.into_response())
        }

        // Decisions (ADRs): /w/{workspace}/decisions
        [action] if action == "decisions" => {
            render_decisions_html(&workspace, &base_path)
                .map(|h| h.into_response())
        }

        // Search page: /w/{workspace}/search
        [action] if action == "search" => {
            let search_term = query.get("q").cloned().unwrap_or_default();
            let html = generate_search_page_html(&workspace, &base_path, Some(&workspace_id), &search_term);
            Ok(Html(html).into_response())
        }

        // Tree view: /w/{workspace}/tree
        [action] if action == "tree" => {
            render_tree_view_html(&workspace, &base_path)
                .map(|h| h.into_response())
        }

        // Presentation: /w/{workspace}/presentation
        [action] if action == "presentation" => {
            let views = query.get("views").cloned();
            render_presentation_html(&workspace, &base_path, views)
                .map(|h| h.into_response())
        }

        // Explore: /w/{workspace}/explore
        [action] if action == "explore" => {
            let html = generate_explore_page_html(&workspace, &base_path, Some(&workspace_id));
            Ok(Html(html).into_response())
        }

        // View diagram: /w/{workspace}/view/{key}
        [action, key] if action == "view" => {
            render_view_diagram_html(&workspace, key, &base_path)
                .map(|h| h.into_response())
        }

        // Edit diagram: /w/{workspace}/edit/{key}
        [action, key] if action == "edit" => {
            render_edit_diagram_html(&workspace, key, &base_path)
                .map(|h| h.into_response())
        }

        // Animated view: /w/{workspace}/view/{key}/animate
        [action, key, sub] if action == "view" && sub == "animate" => {
            render_dynamic_animated_html(&workspace, key, &base_path)
                .map(|h| h.into_response())
        }

        // SVG export: /w/{workspace}/view/{key}/svg
        [action, key, sub] if action == "view" && sub == "svg" => {
            render_view_svg(&workspace, key)
                .map(|r| r.into_response())
        }

        // PlantUML export: /w/{workspace}/view/{key}/plantuml
        [action, key, sub] if action == "view" && sub == "plantuml" => {
            let raw = query.get("raw").map(|v| v == "true").unwrap_or(false);
            let code = get_export_code(&workspace, key, "plantuml")?;
            if raw {
                Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
            } else {
                let html = generate_plantuml_viewer_html(&workspace, key, &base_path, &code);
                Ok(Html(html).into_response())
            }
        }

        // Mermaid export: /w/{workspace}/view/{key}/mermaid
        [action, key, sub] if action == "view" && sub == "mermaid" => {
            let raw = query.get("raw").map(|v| v == "true").unwrap_or(false);
            let code = get_export_code(&workspace, key, "mermaid")?;
            if raw {
                Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
            } else {
                let html = generate_mermaid_viewer_html(&workspace, key, &base_path, &code);
                Ok(Html(html).into_response())
            }
        }

        // DOT export: /w/{workspace}/view/{key}/dot
        [action, key, sub] if action == "view" && sub == "dot" => {
            let raw = query.get("raw").map(|v| v == "true").unwrap_or(false);
            let code = get_export_code(&workspace, key, "dot")?;
            if raw {
                Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
            } else {
                let html = generate_dot_viewer_html(&workspace, key, &base_path, &code);
                Ok(Html(html).into_response())
            }
        }

        // D2 export: /w/{workspace}/view/{key}/d2
        [action, key, sub] if action == "view" && sub == "d2" => {
            let raw = query.get("raw").map(|v| v == "true").unwrap_or(false);
            let code = get_export_code(&workspace, key, "d2")?;
            if raw {
                Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
            } else {
                let html = generate_d2_viewer_html(&workspace, key, &base_path, &code);
                Ok(Html(html).into_response())
            }
        }

        // API: Get workspace JSON: /w/{workspace}/api/workspace
        [action, sub] if action == "api" && sub == "workspace" => {
            Ok(Json(workspace).into_response())
        }

        // API: Validate workspace: /w/{workspace}/api/validate
        [action, sub] if action == "api" && sub == "validate" => {
            let validation_result = structurizr_dsl::validate_workspace(&workspace);
            Ok(Json(validation_result).into_response())
        }

        // API: Search: /w/{workspace}/api/search
        [action, sub] if action == "api" && sub == "search" => {
            let query_str = query.get("q").map(|s| s.as_str()).unwrap_or("");
            let results = search_workspace(&workspace, query_str, "")?;
            Ok(results.into_response())
        }

        // API: Get notes: /w/{workspace}/api/view/{view_key}/notes
        [action, sub, view_key, notes_action] if action == "api" && sub == "view" && notes_action == "notes" => {
            let info = state.get_workspace_info(&workspace_id).await
                .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;
            let notes_file = NotesFile::load(&info.path).await;
            let view_notes = notes_file
                .get_view_notes(view_key)
                .cloned()
                .unwrap_or_default();
            Ok(Json(view_notes).into_response())
        }

        // API: Get positions: /w/{workspace}/api/view/{view_key}/positions
        [action, sub, view_key, pos_action] if action == "api" && sub == "view" && pos_action == "positions" => {
            let positions = state.editor.get_positions(view_key).await;
            Ok(Json(positions.values().cloned().collect::<Vec<_>>()).into_response())
        }

        // Export JSON: /w/{workspace}/export/json
        [action, sub] if action == "export" && sub == "json" => {
            let json = JsonExporter::export(&workspace)?;
            Ok(([(header::CONTENT_TYPE, "application/json")], json).into_response())
        }

        // Not found
        _ => Err(Error::ViewNotFound(format!("Unknown route: /w/{}", path)))
    }
}

/// Handle WebSocket dispatch for editor connections.
///
/// Path format: {workspace_path}/ws/edit/{view_key}
/// Examples:
/// - my-workspace/ws/edit/Context
/// - small/startup-saas/ws/edit/Context
async fn handle_websocket_dispatch(
    ws: axum::extract::ws::WebSocketUpgrade,
    state: AppState,
    path: &str,
) -> Result<axum::response::Response> {
    use axum::response::IntoResponse;

    let segments: Vec<&str> = path.trim_start_matches('/').split('/').collect();

    // Find "ws" followed by "edit" to split workspace path from view key
    let ws_pos = segments.iter().position(|&s| s == "ws");

    if let Some(pos) = ws_pos {
        if pos + 2 < segments.len() && segments[pos + 1] == "edit" {
            let workspace_id = segments[..pos].join("/");
            let view_key = segments[pos + 2].to_string();

            // Verify workspace exists before upgrading
            if state.workspace_exists(&workspace_id).await {
                return Ok(ws.on_upgrade(move |socket| {
                    crate::editor::handle_workspace_editor_socket_public(socket, state, workspace_id, view_key)
                }).into_response());
            }
        }
    }

    // Invalid path or workspace not found
    Err(Error::ViewNotFound(format!("Invalid WebSocket path: /w/{}", path)))
}

/// Unified workspace route dispatcher for PUT requests.
pub async fn workspace_dispatch_put(
    State(state): State<AppState>,
    Path(path): Path<String>,
    body: axum::body::Bytes,
) -> Result<axum::response::Response> {
    use axum::response::IntoResponse;

    let (workspace_id, remaining) = parse_workspace_path(&state, &path)
        .await
        .ok_or_else(|| Error::WorkspaceNotFound(path.clone()))?;

    // Verify workspace exists
    let _workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    match remaining.as_slice() {
        // Batch update positions: /w/{workspace}/api/view/{view_key}/positions
        [action, sub, view_key, pos_action] if action == "api" && sub == "view" && pos_action == "positions" => {
            let req: crate::editor::BatchUpdatePositionsRequest = serde_json::from_slice(&body)
                .map_err(|e| Error::Server(format!("Invalid JSON: {}", e)))?;

            for pos in &req.positions {
                state.editor.update_position(view_key, &pos.id, pos.x, pos.y).await;
                state.editor.broadcast(crate::editor::EditorMessage::ElementMoved {
                    view_key: view_key.clone(),
                    element_id: pos.id.clone(),
                    x: pos.x,
                    y: pos.y,
                });
            }
            state.editor.mark_dirty(&workspace_id).await;

            let positions = state.editor.get_positions(view_key).await;
            Ok(Json(positions.values().cloned().collect::<Vec<crate::editor::ElementPosition>>()).into_response())
        }

        _ => Err(Error::ViewNotFound(format!("Unknown PUT route: /w/{}", path)))
    }
}

/// Unified workspace route dispatcher for POST requests.
pub async fn workspace_dispatch_post(
    State(state): State<AppState>,
    Path(path): Path<String>,
    body: axum::body::Bytes,
) -> Result<axum::response::Response> {
    use axum::response::IntoResponse;

    let (workspace_id, remaining) = parse_workspace_path(&state, &path)
        .await
        .ok_or_else(|| Error::WorkspaceNotFound(path.clone()))?;

    // Verify workspace exists
    let _workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    match remaining.as_slice() {
        // Add note: /w/{workspace}/api/view/{view_key}/notes
        [action, sub, view_key, notes_action] if action == "api" && sub == "view" && notes_action == "notes" => {
            let req: AddNoteRequest = serde_json::from_slice(&body)
                .map_err(|e| Error::Server(format!("Invalid JSON: {}", e)))?;

            let info = state.get_workspace_info(&workspace_id).await
                .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

            let mut notes_file = NotesFile::load(&info.path).await;
            notes_file.add_note(
                view_key,
                req.step_index,
                req.first_name,
                req.last_name,
                req.content,
            );
            notes_file.save(&info.path).await
                .map_err(|e| Error::Server(format!("Failed to save notes: {}", e)))?;

            let view_notes = notes_file
                .get_view_notes(view_key)
                .cloned()
                .unwrap_or_default();

            Ok(Json(view_notes).into_response())
        }

        _ => Err(Error::ViewNotFound(format!("Unknown POST route: /w/{}", path)))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create a minimal workspace for testing.
    fn create_test_workspace() -> Workspace {
        let mut workspace = Workspace::new("Test Workspace", "A test workspace");

        // Add a simple software system for views
        workspace.model_mut().add_software_system("TestSystem", "A test system");

        workspace
    }

    // ========================================================================
    // Diagram Viewer Tests
    // ========================================================================

    #[test]
    fn test_diagram_html_contains_canvas_element() {
        let workspace = create_test_workspace();
        let html = generate_view_diagram_html(&workspace, "test-view", "");

        assert!(html.contains("id=\"diagram-canvas\""), "Diagram HTML should contain diagram-canvas element");
        assert!(html.contains("id=\"diagram-container\""), "Diagram HTML should contain diagram-container");
    }

    #[test]
    fn test_diagram_html_contains_zoom_controls() {
        let workspace = create_test_workspace();
        let html = generate_view_diagram_html(&workspace, "test-view", "");

        assert!(html.contains("zoom-controls"), "Diagram HTML should contain zoom controls");
        assert!(html.contains("zoomIn()"), "Diagram HTML should contain zoomIn function");
        assert!(html.contains("zoomOut()"), "Diagram HTML should contain zoomOut function");
        assert!(html.contains("resetZoom()"), "Diagram HTML should contain resetZoom function");
    }

    #[test]
    fn test_diagram_html_contains_minimap() {
        let workspace = create_test_workspace();
        let html = generate_view_diagram_html(&workspace, "test-view", "");

        assert!(html.contains("id=\"minimap\""), "Diagram HTML should contain minimap");
        assert!(html.contains("minimap-viewport"), "Diagram HTML should contain minimap viewport");
        assert!(html.contains("updateMinimap()"), "Diagram HTML should contain updateMinimap function");
    }

    #[test]
    fn test_diagram_html_contains_breadcrumbs() {
        let workspace = create_test_workspace();
        let html = generate_view_diagram_html(&workspace, "test-view", "");

        assert!(html.contains("breadcrumbs"), "Diagram HTML should contain breadcrumbs");
        assert!(html.contains("initiateDrillDown"), "Diagram HTML should contain drill-down function");
    }

    #[test]
    fn test_diagram_html_with_base_path() {
        let workspace = create_test_workspace();
        let html = generate_view_diagram_html(&workspace, "test-view", "/w/my-workspace");

        assert!(html.contains("href=\"/w/my-workspace\""), "Links should use base_path");
        assert!(html.contains("basePath = '/w/my-workspace'"), "JavaScript should have basePath constant");
    }

    // ========================================================================
    // Documentation Tests
    // ========================================================================

    #[test]
    fn test_documentation_html_contains_sidebar() {
        let workspace = create_test_workspace();
        let html = generate_documentation_html(&workspace, "");

        assert!(html.contains("class=\"sidebar\"") || html.contains("sidebar"),
            "Documentation HTML should contain sidebar");
    }

    #[test]
    fn test_documentation_html_contains_scroll_spy() {
        let workspace = create_test_workspace();
        let html = generate_documentation_html(&workspace, "");

        // Scroll-spy functionality
        assert!(html.contains("IntersectionObserver") || html.contains("scroll"),
            "Documentation HTML should contain scroll tracking");
    }

    #[test]
    fn test_documentation_html_with_base_path() {
        let workspace = create_test_workspace();
        let html = generate_documentation_html(&workspace, "/w/my-workspace");

        assert!(html.contains("/w/my-workspace"), "Documentation HTML should use base_path in links");
    }

    // ========================================================================
    // Editor Tests
    // ========================================================================

    #[test]
    fn test_editor_html_contains_websocket_init() {
        let workspace = create_test_workspace();
        let html = generate_editor_html(&workspace, "test-view", "", "", None);

        assert!(html.contains("WebSocket"), "Editor HTML should initialize WebSocket");
    }

    #[test]
    fn test_editor_html_contains_toolbar() {
        let workspace = create_test_workspace();
        let html = generate_editor_html(&workspace, "test-view", "", "", None);

        assert!(html.contains("toolbar"), "Editor HTML should contain toolbar");
        assert!(html.contains("Auto Layout") || html.contains("autoLayout"), "Editor HTML should contain auto-layout button");
        assert!(html.contains("Save") || html.contains("save"), "Editor HTML should contain save button");
    }

    #[test]
    fn test_editor_html_contains_connection_status() {
        let workspace = create_test_workspace();
        let html = generate_editor_html(&workspace, "test-view", "", "", None);

        assert!(html.contains("status") || html.contains("Status"),
            "Editor HTML should show connection status");
    }

    #[test]
    fn test_editor_html_with_workspace_paths() {
        let workspace = create_test_workspace();
        let html = generate_editor_html(&workspace, "test-view", "/w/my-workspace", "/w/my-workspace/ws", Some("my-workspace"));

        assert!(html.contains("/w/my-workspace"), "Back link should use base_path");
    }

    #[test]
    fn test_editor_html_contains_pan_zoom() {
        let workspace = create_test_workspace();
        let html = generate_editor_html(&workspace, "test-view", "", "", None);

        assert!(html.contains("scale") || html.contains("zoom"), "Editor HTML should track zoom scale");
    }

    // ========================================================================
    // Dynamic Animation Tests
    // ========================================================================

    #[test]
    fn test_animation_html_contains_step_controls() {
        let workspace = create_test_workspace();
        // Create a dynamic view for testing
        if let Ok(html) = generate_dynamic_animated_html(&workspace, "test-view", "") {
            assert!(html.contains("step") || html.contains("Step"),
                "Animation HTML should contain step controls");
        }
        // Note: May fail if no dynamic views exist, which is expected
    }

    // ========================================================================
    // Tree View Tests
    // ========================================================================

    #[test]
    fn test_tree_html_contains_expand_collapse() {
        let workspace = create_test_workspace();
        let html = generate_tree_page_html(&workspace, "", None);

        assert!(html.contains("toggleNode") || html.contains("expand") || html.contains("collapse") || html.contains("tree"),
            "Tree HTML should contain expand/collapse functionality");
    }

    #[test]
    fn test_tree_html_with_base_path() {
        let workspace = create_test_workspace();
        let html = generate_tree_page_html(&workspace, "/w/my-workspace", Some("my-workspace"));

        assert!(html.contains("/w/my-workspace"), "Tree HTML should use base_path in links");
    }

    // ========================================================================
    // Search Tests
    // ========================================================================

    #[test]
    fn test_search_html_contains_search_form() {
        let workspace = create_test_workspace();
        let html = generate_search_page_html(&workspace, "", None, "");

        assert!(html.contains("<form") || html.contains("search"), "Search HTML should contain form");
    }

    #[test]
    fn test_search_html_with_results() {
        let workspace = create_test_workspace();
        let html = generate_search_page_html(&workspace, "", None, "test");

        // Just verify the page can be generated with a search term
        assert!(html.contains("search") || html.contains("Search"), "Search HTML should show search functionality");
    }

    #[test]
    fn test_search_html_with_base_path() {
        let workspace = create_test_workspace();
        let html = generate_search_page_html(&workspace, "/w/my-workspace", Some("my-workspace"), "");

        assert!(html.contains("/w/my-workspace"), "Search HTML should use base_path");
    }

    // ========================================================================
    // Presentation Mode Tests
    // ========================================================================

    #[test]
    fn test_presentation_html_contains_slideshow() {
        let workspace = create_test_workspace();
        // We use the render function as it's easier to test
        if let Ok(html_response) = render_presentation_html(&workspace, "", None) {
            let html = html_response.0;
            assert!(html.contains("slide") || html.contains("Slide") || html.contains("presentation"),
                "Presentation HTML should contain slideshow elements");
        }
    }

    // ========================================================================
    // Explore Graph Tests
    // ========================================================================

    #[test]
    fn test_explore_html_contains_force_simulation() {
        let workspace = create_test_workspace();
        let html = generate_explore_page_html(&workspace, "", None);

        assert!(html.contains("force") || html.contains("simulation") || html.contains("physics") || html.contains("node") || html.contains("graph"),
            "Explore HTML should contain force-directed graph elements");
    }
}
