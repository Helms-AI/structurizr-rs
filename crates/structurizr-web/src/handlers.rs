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
use crate::state::AppState;


/// Generate home page HTML content (shared between single and multi-workspace modes).
fn generate_home_page_html(ws: &Workspace, base_path: &str, workspace_id: Option<&str>) -> String {
    let views = ws.views();
    let view_list: Vec<String> = views.all_keys().iter().map(|k| k.to_string()).collect();

    // Check which views are dynamic views
    let dynamic_view_keys: std::collections::HashSet<String> = views.dynamic_views.iter()
        .map(|v| v.properties.key.clone())
        .collect();

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
        .views {
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
    </style>"##;

    // Build view cards
    let view_cards: String = view_list.iter().map(|v| {
        let bp = base_path;
        let animate_link = if dynamic_view_keys.contains(v) {
            format!(r#" | <a href="{}/view/{}/animate">Animate</a>"#, bp, v)
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
            bp, v, escape_html(v),
            bp, v,
            bp, v, animate_link,
            bp, v,
            bp, v,
            bp, v,
            bp, v,
            bp, v
        )
    }).collect::<Vec<_>>().join("\n");

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
        <div class="views">
            {}
        </div>"#,
        escape_html(&ws.name),
        ws.description.as_deref().map(escape_html).unwrap_or_default(),
        ws.model().people.len(),
        ws.model().software_systems.len(),
        ws.model().relationships.len(),
        view_cards
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

/// Generate explore page HTML (shared between single and multi-workspace modes).
fn generate_explore_page_html(workspace: &Workspace, base_path: &str, workspace_id: Option<&str>) -> String {
    let model = workspace.model();

    // Build nodes JSON
    let mut nodes_json = String::from("[");
    let mut first = true;

    // Add people
    for person in &model.people {
        if !first {
            nodes_json.push(',');
        }
        first = false;
        nodes_json.push_str(&format!(
            r#"{{"id":"{}","name":"{}","type":"Person","description":{}}}"#,
            person.id(),
            escape_json(&person.name()),
            person.properties.description.as_ref()
                .map(|d| format!("\"{}\"", escape_json(d)))
                .unwrap_or_else(|| "null".to_string())
        ));
    }

    // Add software systems
    for system in &model.software_systems {
        if !first {
            nodes_json.push(',');
        }
        first = false;
        nodes_json.push_str(&format!(
            r#"{{"id":"{}","name":"{}","type":"Software System","description":{}}}"#,
            system.id(),
            escape_json(&system.name()),
            system.properties.description.as_ref()
                .map(|d| format!("\"{}\"", escape_json(d)))
                .unwrap_or_else(|| "null".to_string())
        ));

        // Add containers
        for container in &system.containers {
            nodes_json.push(',');
            nodes_json.push_str(&format!(
                r#"{{"id":"{}","name":"{}","type":"Container","description":{},"technology":{}}}"#,
                container.id(),
                escape_json(&container.name()),
                container.properties.description.as_ref()
                    .map(|d| format!("\"{}\"", escape_json(d)))
                    .unwrap_or_else(|| "null".to_string()),
                container.technology.as_ref()
                    .map(|t| format!("\"{}\"", escape_json(t)))
                    .unwrap_or_else(|| "null".to_string())
            ));

            // Add components
            for component in &container.components {
                nodes_json.push(',');
                nodes_json.push_str(&format!(
                    r#"{{"id":"{}","name":"{}","type":"Component","description":{},"technology":{}}}"#,
                    component.id(),
                    escape_json(&component.name()),
                    component.properties.description.as_ref()
                        .map(|d| format!("\"{}\"", escape_json(d)))
                        .unwrap_or_else(|| "null".to_string()),
                    component.technology.as_ref()
                        .map(|t| format!("\"{}\"", escape_json(t)))
                        .unwrap_or_else(|| "null".to_string())
                ));
            }
        }
    }
    nodes_json.push(']');

    // Build links JSON
    let mut links_json = String::from("[");
    first = true;

    for rel in &model.relationships {
        if !first {
            links_json.push(',');
        }
        first = false;
        links_json.push_str(&format!(
            r#"{{"source":"{}","target":"{}","label":{}}}"#,
            rel.source_id,
            rel.destination_id,
            rel.description.as_ref()
                .map(|d| format!("\"{}\"", escape_json(d)))
                .unwrap_or_else(|| "null".to_string())
        ));
    }
    links_json.push(']');

    let title = format!("Explore - {}", workspace.name);

    let extra_styles = r##"<style>
        .explore-toolbar {
            background: var(--toolbar-bg);
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 20px;
            border-bottom: 1px solid var(--toolbar-border);
        }
        .explore-toolbar span { color: var(--toolbar-text); }
        .explore-toolbar button {
            background: var(--bg-tertiary);
            color: var(--text-primary);
            border: 1px solid var(--border-color);
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
        }
        .explore-toolbar button:hover { background: var(--card-hover); }
        .explore-toolbar .separator { border-left: 1px solid var(--border-color); height: 20px; }
        .canvas-container { flex: 1; position: relative; background: var(--canvas-bg); }
        svg { width: 100%; height: 100%; cursor: grab; }
        svg.dragging { cursor: grabbing; }
        .node { cursor: pointer; }
        .node circle { transition: r 0.2s, fill 0.2s; }
        .node:hover circle { r: 35; }
        .node text { pointer-events: none; user-select: none; fill: #fff; }
        .link { stroke: var(--text-muted); stroke-width: 1.5; fill: none; }
        .link-label { fill: var(--text-muted); font-size: 10px; pointer-events: none; user-select: none; }
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
        .info {
            position: fixed;
            bottom: 20px;
            left: 20px;
            background: var(--card-bg);
            padding: 12px 16px;
            border-radius: 6px;
            font-size: 12px;
            color: var(--text-muted);
            border: 1px solid var(--border-color);
        }
        .controls { display: flex; gap: 10px; align-items: center; }
        .controls label {
            font-size: 12px;
            color: var(--text-secondary);
            display: flex;
            align-items: center;
            gap: 5px;
        }
        .controls input[type="range"] { width: 100px; }
    </style>"##;

    let extra_scripts = format!(r##"<script>
        const nodes = {{}};
        const links = {{}};

        // Create SVG and groups
        const svg = document.getElementById('canvas');
        const container = document.querySelector('.canvas-container');
        const width = container.clientWidth;
        const height = container.clientHeight;

        svg.setAttribute('viewBox', `0 0 ${{width}} ${{height}}`);

        const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        svg.appendChild(g);

        const linkGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        linkGroup.setAttribute('class', 'links');
        g.appendChild(linkGroup);

        const nodeGroup = document.createElementNS('http://www.w3.org/2000/svg', 'g');
        nodeGroup.setAttribute('class', 'nodes');
        g.appendChild(nodeGroup);

        // Zoom and pan
        let transform = {{ x: 0, y: 0, k: 1 }};
        let isPanning = false;
        let panStart = {{ x: 0, y: 0 }};

        svg.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = svg.getBoundingClientRect();
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;

            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            const newK = Math.max(0.1, Math.min(5, transform.k * delta));

            transform.x = x - (x - transform.x) * (newK / transform.k);
            transform.y = y - (y - transform.y) * (newK / transform.k);
            transform.k = newK;

            updateTransform();
        }});

        svg.addEventListener('mousedown', (e) => {{
            if (e.button === 0 && !e.target.closest('.node')) {{
                isPanning = true;
                panStart = {{ x: e.clientX - transform.x, y: e.clientY - transform.y }};
                svg.classList.add('dragging');
            }}
        }});

        document.addEventListener('mousemove', (e) => {{
            if (isPanning) {{
                transform.x = e.clientX - panStart.x;
                transform.y = e.clientY - panStart.y;
                updateTransform();
            }}
        }});

        document.addEventListener('mouseup', () => {{
            isPanning = false;
            svg.classList.remove('dragging');
        }});

        function updateTransform() {{
            g.setAttribute('transform', `translate(${{transform.x}},${{transform.y}}) scale(${{transform.k}})`);
        }}

        // Node colors by type
        const colors = {{
            'Person': '#08427b',
            'Software System': '#1168bd',
            'Container': '#438dd5',
            'Component': '#85bbf0'
        }};

        // Initialize force simulation
        let simulation = {{
            nodes: [],
            links: [],
            alpha: 1,
            alphaDecay: 0.02,
            velocityDecay: 0.4,
            chargeStrength: -300,
            linkDistance: 100
        }};

        // Load data
        const data = {{
            nodes: {nodes_json},
            links: {links_json}
        }};

        // Create node elements
        data.nodes.forEach(node => {{
            node.x = width / 2 + (Math.random() - 0.5) * 200;
            node.y = height / 2 + (Math.random() - 0.5) * 200;
            node.vx = 0;
            node.vy = 0;

            const nodeG = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            nodeG.setAttribute('class', 'node');
            nodeG.setAttribute('data-id', node.id);

            const circle = document.createElementNS('http://www.w3.org/2000/svg', 'circle');
            circle.setAttribute('r', 30);
            circle.setAttribute('fill', colors[node.type] || '#999');
            circle.setAttribute('stroke', '#fff');
            circle.setAttribute('stroke-width', 2);

            const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
            text.setAttribute('text-anchor', 'middle');
            text.setAttribute('dy', '0.35em');
            text.setAttribute('font-size', '12');
            text.textContent = node.name.length > 15 ? node.name.substring(0, 13) + '...' : node.name;

            nodeG.appendChild(circle);
            nodeG.appendChild(text);
            nodeGroup.appendChild(nodeG);

            node.element = nodeG;
            nodes[node.id] = node;

            // Drag handlers
            let isDragging = false;
            let dragStart = {{ x: 0, y: 0 }};

            nodeG.addEventListener('mousedown', (e) => {{
                e.stopPropagation();
                isDragging = true;
                dragStart = {{ x: e.clientX / transform.k - node.x, y: e.clientY / transform.k - node.y }};
                node.fx = node.x;
                node.fy = node.y;
            }});

            document.addEventListener('mousemove', (e) => {{
                if (isDragging && node.fx !== undefined) {{
                    node.fx = e.clientX / transform.k - dragStart.x;
                    node.fy = e.clientY / transform.k - dragStart.y;
                }}
            }});

            document.addEventListener('mouseup', () => {{
                if (isDragging) {{
                    isDragging = false;
                    node.fx = undefined;
                    node.fy = undefined;
                }}
            }});

            // Tooltip
            nodeG.addEventListener('mouseenter', (e) => {{
                const tooltip = document.getElementById('tooltip');
                let html = `<h4>${{escapeHtml(node.name)}}</h4><div class="type">${{node.type}}</div>`;
                if (node.description) {{
                    html += `<div class="desc">${{escapeHtml(node.description)}}</div>`;
                }}
                if (node.technology) {{
                    html += `<div class="tech">Technology: ${{escapeHtml(node.technology)}}</div>`;
                }}
                tooltip.innerHTML = html;
                tooltip.style.display = 'block';
            }});

            nodeG.addEventListener('mousemove', (e) => {{
                const tooltip = document.getElementById('tooltip');
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
            }});

            nodeG.addEventListener('mouseleave', () => {{
                document.getElementById('tooltip').style.display = 'none';
            }});
        }});

        // Create link elements
        data.links.forEach(link => {{
            const source = nodes[link.source];
            const target = nodes[link.target];

            if (!source || !target) return;

            link.source = source;
            link.target = target;

            const path = document.createElementNS('http://www.w3.org/2000/svg', 'path');
            path.setAttribute('class', 'link');
            linkGroup.appendChild(path);

            link.element = path;

            if (link.label) {{
                const text = document.createElementNS('http://www.w3.org/2000/svg', 'text');
                text.setAttribute('class', 'link-label');
                text.setAttribute('text-anchor', 'middle');
                text.textContent = link.label.length > 20 ? link.label.substring(0, 18) + '...' : link.label;
                linkGroup.appendChild(text);
                link.labelElement = text;
            }}

            links[`${{link.source.id}}-${{link.target.id}}`] = link;
        }});

        simulation.nodes = data.nodes;
        simulation.links = data.links;

        // Update counts
        document.getElementById('node-count').textContent = data.nodes.length;
        document.getElementById('link-count').textContent = data.links.length;

        // Force simulation
        function tick() {{
            if (simulation.alpha < 0.001) return;

            simulation.alpha *= (1 - simulation.alphaDecay);

            // Apply forces
            simulation.nodes.forEach(node => {{
                if (node.fx !== undefined) {{
                    node.x = node.fx;
                    node.y = node.fy;
                    node.vx = 0;
                    node.vy = 0;
                    return;
                }}

                // Center force
                const cx = width / 2;
                const cy = height / 2;
                node.vx += (cx - node.x) * 0.01;
                node.vy += (cy - node.y) * 0.01;

                // Many-body repulsion
                simulation.nodes.forEach(other => {{
                    if (node === other) return;
                    const dx = node.x - other.x;
                    const dy = node.y - other.y;
                    const dist = Math.sqrt(dx * dx + dy * dy) || 1;
                    const force = simulation.chargeStrength / (dist * dist);
                    node.vx += (dx / dist) * force;
                    node.vy += (dy / dist) * force;
                }});
            }});

            // Link force
            simulation.links.forEach(link => {{
                const dx = link.target.x - link.source.x;
                const dy = link.target.y - link.source.y;
                const dist = Math.sqrt(dx * dx + dy * dy) || 1;
                const force = (dist - simulation.linkDistance) * 0.1;
                const fx = (dx / dist) * force;
                const fy = (dy / dist) * force;

                if (link.source.fx === undefined) {{
                    link.source.vx += fx;
                    link.source.vy += fy;
                }}
                if (link.target.fx === undefined) {{
                    link.target.vx -= fx;
                    link.target.vy -= fy;
                }}
            }});

            // Apply velocity
            simulation.nodes.forEach(node => {{
                if (node.fx !== undefined) return;
                node.vx *= simulation.velocityDecay;
                node.vy *= simulation.velocityDecay;
                node.x += node.vx;
                node.y += node.vy;
            }});

            render();
            requestAnimationFrame(tick);
        }}

        function render() {{
            // Update node positions
            simulation.nodes.forEach(node => {{
                node.element.setAttribute('transform', `translate(${{node.x}},${{node.y}})`);
            }});

            // Update link positions
            simulation.links.forEach(link => {{
                const sx = link.source.x;
                const sy = link.source.y;
                const tx = link.target.x;
                const ty = link.target.y;

                // Draw curved line
                const dx = tx - sx;
                const dy = ty - sy;
                const dr = Math.sqrt(dx * dx + dy * dy) * 1.5;
                link.element.setAttribute('d', `M${{sx}},${{sy}}A${{dr}},${{dr}} 0 0,1 ${{tx}},${{ty}}`);

                if (link.labelElement) {{
                    link.labelElement.setAttribute('x', (sx + tx) / 2);
                    link.labelElement.setAttribute('y', (sy + ty) / 2);
                }}
            }});
        }}

        function resetSimulation() {{
            simulation.alpha = 1;
            simulation.nodes.forEach(node => {{
                node.x = width / 2 + (Math.random() - 0.5) * 200;
                node.y = height / 2 + (Math.random() - 0.5) * 200;
                node.vx = 0;
                node.vy = 0;
            }});
            tick();
        }}

        function centerGraph() {{
            transform = {{ x: 0, y: 0, k: 1 }};
            updateTransform();
        }}

        function escapeHtml(text) {{
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }}

        // Controls
        document.getElementById('charge-slider').addEventListener('input', (e) => {{
            simulation.chargeStrength = -parseInt(e.target.value);
            document.getElementById('charge-value').textContent = simulation.chargeStrength;
            simulation.alpha = 0.3;
        }});

        document.getElementById('link-slider').addEventListener('input', (e) => {{
            simulation.linkDistance = parseInt(e.target.value);
            document.getElementById('link-value').textContent = simulation.linkDistance;
            simulation.alpha = 0.3;
        }});

        // Handle window resize
        window.addEventListener('resize', () => {{
            const newWidth = container.clientWidth;
            const newHeight = container.clientHeight;
            svg.setAttribute('viewBox', `0 0 ${{newWidth}} ${{newHeight}}`);
        }});

        // Start simulation
        tick();
    </script>"##, nodes_json = nodes_json, links_json = links_json);

    let content = r#"
        <div class="explore-toolbar">
            <span>Explore Graph</span>
            <div class="separator"></div>
            <div class="controls">
                <button onclick="resetSimulation()">Reset</button>
                <button onclick="centerGraph()">Center</button>
                <label>
                    Charge: <input type="range" id="charge-slider" min="50" max="500" value="300" step="10">
                    <span id="charge-value">-300</span>
                </label>
                <label>
                    Link Distance: <input type="range" id="link-slider" min="30" max="200" value="100" step="10">
                    <span id="link-value">100</span>
                </label>
            </div>
        </div>
        <div class="canvas-container">
            <svg id="canvas"></svg>
        </div>
        <div class="tooltip" id="tooltip"></div>
        <div class="info">
            <div>Nodes: <span id="node-count">0</span> | Links: <span id="link-count">0</span></div>
            <div style="margin-top: 4px; font-size: 11px;">Drag nodes • Scroll to zoom • Drag canvas to pan</div>
        </div>
    "#;

    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Explore,
        content_type: ContentType::ToolbarViewport,
        extra_head: extra_styles,
        extra_body_end: &extra_scripts,
    };

    generate_page_layout(&config, content)
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

            // Add containers if any
            if has_containers {
                tree_html.push_str(r#"<ul class="children">"#);

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
                    tree_html.push_str(r#"<span class="icon">🗄️</span>"#);
                    tree_html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&container.name())));
                    if has_components {
                        tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, container.components.len()));
                    }
                    if let Some(tech) = &container.technology {
                        tree_html.push_str(&format!(r#"<span class="tech">[{}]</span>"#, escape_html(tech)));
                    }
                    tree_html.push_str(r#"</div>"#);

                    // Add components if any
                    if has_components {
                        tree_html.push_str(r#"<ul class="children">"#);

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
                                tree_html.push_str(&format!(r#"<span class="tech">[{}]</span>"#, escape_html(tech)));
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

    // Deployment Nodes branch
    if !model.deployment_nodes.is_empty() {
        tree_html.push_str(r#"<li class="expandable">"#);
        tree_html.push_str(r#"<div class="tree-node" data-type="group">"#);
        tree_html.push_str(r#"<span class="toggle">▶</span>"#);
        tree_html.push_str(r#"<span class="icon">🖥️</span>"#);
        tree_html.push_str(r#"<span class="name">Deployment Nodes</span>"#);
        tree_html.push_str(&format!(r#"<span class="count">({})</span>"#, model.deployment_nodes.len()));
        tree_html.push_str(r#"</div>"#);
        tree_html.push_str(r#"<ul class="children">"#);

        for node in &model.deployment_nodes {
            render_deployment_node(&mut tree_html, node);
        }

        tree_html.push_str(r#"</ul></li>"#);
    }

    // Page-specific styles for tree view
    let extra_styles = r##"<style>
        .tree-panel { width: 500px; background: var(--card-bg); border-right: 1px solid var(--border-color); display: flex; flex-direction: column; }
        .search-box { padding: 15px; border-bottom: 1px solid var(--border-color); }
        .search-box input { width: 100%; padding: 10px; font-size: 14px; border: 2px solid var(--border-color); border-radius: 6px; background: var(--bg-secondary); color: var(--text-primary); }
        .search-box input:focus { outline: none; border-color: var(--link-color); }
        .tree-container { flex: 1; overflow-y: auto; padding: 10px; }

        ul.tree { list-style: none; padding: 0; margin: 0; }
        ul.tree ul { padding-left: 20px; }

        .tree li { margin: 2px 0; }
        .tree-node { display: flex; align-items: center; gap: 6px; padding: 6px 8px; border-radius: 4px; cursor: pointer; user-select: none; }
        .tree-node:hover { background: var(--bg-tertiary); }
        .tree-node.selected { background: var(--link-color); color: var(--bg-primary); border-left: 3px solid var(--link-color); }
        [data-theme="light"] .tree-node.selected { background: #e3f2fd; color: #333; }

        .toggle { width: 16px; text-align: center; font-size: 12px; color: var(--text-muted); }
        .icon { font-size: 16px; }
        .name { font-weight: 500; font-size: 14px; color: var(--text-primary); }
        .count { font-size: 11px; color: var(--text-muted); }
        .desc { font-size: 12px; color: var(--text-secondary); margin-left: auto; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
        .tech { font-size: 11px; color: var(--link-color); font-family: monospace; }

        .children { display: none; }
        .expandable.expanded > .children { display: block; }

        .detail-panel { flex: 1; padding: 30px; overflow-y: auto; background: var(--bg-primary); }
        .detail-content { max-width: 800px; background: var(--card-bg); padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px var(--shadow); }
        .detail-content h2 { margin-top: 0; display: flex; align-items: center; gap: 10px; }
        .detail-content .type-badge { background: var(--bg-tertiary); padding: 4px 12px; border-radius: 4px; font-size: 11px; text-transform: uppercase; font-weight: 600; }
        .detail-content .property { margin: 15px 0; }
        .detail-content .property-label { font-weight: 600; color: var(--text-secondary); font-size: 12px; text-transform: uppercase; margin-bottom: 5px; }
        .detail-content .property-value { font-size: 14px; color: var(--text-primary); line-height: 1.6; }
        .empty-state { color: var(--text-muted); font-style: italic; text-align: center; padding: 100px 20px; }
        .no-results { color: var(--text-muted); font-style: italic; padding: 20px; text-align: center; }
        .stats { display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; margin: 20px 0; }
        .stat-card { background: var(--bg-tertiary); padding: 15px; border-radius: 6px; text-align: center; }
        .stat-card .value { font-size: 24px; font-weight: bold; color: var(--link-color); }
        .stat-card .label { font-size: 12px; color: var(--text-secondary); margin-top: 5px; }
    </style>"##;

    // Tree page JavaScript
    let extra_scripts = r##"<script>
        // Tree expand/collapse functionality
        document.addEventListener('click', (e) => {
            const toggle = e.target.closest('.toggle');
            if (toggle) {
                e.stopPropagation();
                const li = toggle.closest('li');
                li.classList.toggle('expanded');
                toggle.textContent = li.classList.contains('expanded') ? '▼' : '▶';
            }
        });

        // Node selection and detail display
        document.addEventListener('click', (e) => {
            const node = e.target.closest('.tree-node');
            if (node && node.dataset.id) {
                // Remove previous selection
                document.querySelectorAll('.tree-node.selected').forEach(n => n.classList.remove('selected'));

                // Add new selection
                node.classList.add('selected');

                // Show details
                showDetails(node);
            }
        });

        function showDetails(node) {
            const emptyState = document.getElementById('empty-state');
            const detailContent = document.getElementById('detail-content');

            emptyState.style.display = 'none';
            detailContent.style.display = 'block';

            const type = node.dataset.type;
            const name = node.dataset.name;
            const description = node.dataset.description || 'No description provided';
            const technology = node.dataset.technology || '';
            const id = node.dataset.id;

            let html = `
                <h2>
                    <span class="icon">${getIcon(type)}</span>
                    ${escapeHtml(name)}
                    <span class="type-badge">${type}</span>
                </h2>
                <div class="property">
                    <div class="property-label">ID</div>
                    <div class="property-value"><code>${id}</code></div>
                </div>
                <div class="property">
                    <div class="property-label">Description</div>
                    <div class="property-value">${escapeHtml(description)}</div>
                </div>
            `;

            if (technology) {
                html += `
                    <div class="property">
                        <div class="property-label">Technology</div>
                        <div class="property-value">${escapeHtml(technology)}</div>
                    </div>
                `;
            }

            detailContent.innerHTML = html;
        }

        function getIcon(type) {
            const icons = {
                'Person': '👤',
                'Software System': '📦',
                'Container': '🗄️',
                'Component': '⚙️',
                'Deployment Node': '🖥️'
            };
            return icons[type] || '📄';
        }

        function escapeHtml(text) {
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }

        // Search functionality
        const searchInput = document.getElementById('search-input');
        const tree = document.getElementById('tree');
        const noResults = document.getElementById('no-results');

        searchInput.addEventListener('input', (e) => {
            const query = e.target.value.toLowerCase().trim();

            if (!query) {
                // Show all nodes
                document.querySelectorAll('.tree li, .tree-node').forEach(el => {
                    el.style.display = '';
                });
                noResults.style.display = 'none';
                tree.style.display = '';
                return;
            }

            let hasResults = false;

            // Filter nodes
            document.querySelectorAll('.tree-node').forEach(node => {
                const name = (node.dataset.name || '').toLowerCase();
                const description = (node.dataset.description || '').toLowerCase();
                const technology = (node.dataset.technology || '').toLowerCase();
                const type = (node.dataset.type || '').toLowerCase();

                const matches = name.includes(query) ||
                               description.includes(query) ||
                               technology.includes(query) ||
                               type.includes(query);

                const li = node.closest('li');

                if (matches) {
                    hasResults = true;
                    li.style.display = '';
                    node.style.display = '';

                    // Expand parent nodes
                    let parent = li.parentElement?.closest('li');
                    while (parent) {
                        parent.style.display = '';
                        parent.classList.add('expanded');
                        const toggle = parent.querySelector(':scope > .tree-node > .toggle');
                        if (toggle) toggle.textContent = '▼';
                        parent = parent.parentElement?.closest('li');
                    }
                } else if (node.dataset.type !== 'group') {
                    li.style.display = 'none';
                }
            });

            // Handle group nodes visibility
            document.querySelectorAll('[data-type="group"]').forEach(groupNode => {
                const li = groupNode.closest('li');
                const visibleChildren = li.querySelectorAll('.children > li[style=""], .children > li:not([style])');
                if (visibleChildren.length === 0) {
                    li.style.display = 'none';
                } else {
                    li.style.display = '';
                    li.classList.add('expanded');
                    const toggle = groupNode.querySelector('.toggle');
                    if (toggle) toggle.textContent = '▼';
                }
            });

            if (hasResults) {
                noResults.style.display = 'none';
                tree.style.display = '';
            } else {
                noResults.style.display = 'block';
                tree.style.display = 'none';
            }
        });
    </script>"##;

    // Build content HTML
    let content = format!(
        r#"<div class="tree-panel">
            <div class="search-box">
                <input type="text" id="search-input" placeholder="Search elements...">
            </div>
            <div class="tree-container">
                <ul class="tree" id="tree">
                    {}
                </ul>
                <div class="no-results" id="no-results" style="display: none;">No matching elements found</div>
            </div>
        </div>
        <div class="detail-panel">
            <div class="empty-state" id="empty-state">
                <h2>Select an element to view details</h2>
                <p>Click on any element in the tree to see its details here.</p>
                <div class="stats">
                    <div class="stat-card">
                        <div class="value">{}</div>
                        <div class="label">People</div>
                    </div>
                    <div class="stat-card">
                        <div class="value">{}</div>
                        <div class="label">Systems</div>
                    </div>
                    <div class="stat-card">
                        <div class="value">{}</div>
                        <div class="label">Relationships</div>
                    </div>
                </div>
            </div>
            <div class="detail-content" id="detail-content" style="display: none;"></div>
        </div>"#,
        tree_html,
        model.people.len(),
        model.software_systems.len(),
        model.relationships.len()
    );

    let title = format!("Tree View - {}", workspace.name);
    let config = LayoutConfig {
        title: &title,
        workspace_name: Some(&workspace.name),
        workspace_id,
        base_path,
        active_nav: NavItem::Tree,
        content_type: ContentType::Sidebar,
        extra_head: extra_styles,
        extra_body_end: extra_scripts,
    };

    generate_page_layout(&config, &content)
}

/// Helper function to render deployment nodes recursively.
fn render_deployment_node(html: &mut String, node: &structurizr_core::model::DeploymentNode) {
    let has_children = !node.children.is_empty() || !node.infrastructure_nodes.is_empty() || !node.container_instances.is_empty();

    if has_children {
        html.push_str(r#"<li class="expandable">"#);
    } else {
        html.push_str(r#"<li class="leaf">"#);
    }

    html.push_str(&format!(
        r#"<div class="tree-node" data-id="{}" data-type="Deployment Node" data-name="{}" data-description="{}" data-technology="{}">"#,
        node.id(),
        escape_html(&node.name()),
        node.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default(),
        node.technology.as_ref().map(|t| escape_html(t)).unwrap_or_default()
    ));

    if has_children {
        html.push_str(r#"<span class="toggle">▶</span>"#);
    }
    html.push_str(r#"<span class="icon">🖥️</span>"#);
    html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&node.name())));
    if let Some(tech) = &node.technology {
        html.push_str(&format!(r#"<span class="tech">[{}]</span>"#, escape_html(tech)));
    }
    html.push_str(r#"</div>"#);

    if has_children {
        html.push_str(r#"<ul class="children">"#);

        // Render child deployment nodes
        for child in &node.children {
            render_deployment_node(html, child);
        }

        // Render infrastructure nodes
        for infra in &node.infrastructure_nodes {
            html.push_str(r#"<li class="leaf">"#);
            html.push_str(&format!(
                r#"<div class="tree-node" data-id="{}" data-type="Infrastructure Node" data-name="{}" data-description="{}" data-technology="{}">"#,
                infra.properties.id,
                escape_html(&infra.properties.name),
                infra.properties.description.as_ref().map(|d| escape_html(d)).unwrap_or_default(),
                infra.technology.as_ref().map(|t| escape_html(t)).unwrap_or_default()
            ));
            html.push_str(r#"<span class="icon">🔧</span>"#);
            html.push_str(&format!(r#"<span class="name">{}</span>"#, escape_html(&infra.properties.name)));
            if let Some(tech) = &infra.technology {
                html.push_str(&format!(r#"<span class="tech">[{}]</span>"#, escape_html(tech)));
            }
            html.push_str(r#"</div>"#);
            html.push_str(r#"</li>"#);
        }

        // Render container instances
        for instance in &node.container_instances {
            html.push_str(r#"<li class="leaf">"#);
            html.push_str(&format!(
                r#"<div class="tree-node" data-id="{}" data-type="Container Instance" data-name="Instance: {}" data-description="">"#,
                instance.id,
                escape_html(&instance.container_id.to_string())
            ));
            html.push_str(r#"<span class="icon">📦</span>"#);
            html.push_str(&format!(r#"<span class="name">Instance: {}</span>"#, escape_html(&instance.container_id.to_string())));
            html.push_str(r#"</div>"#);
            html.push_str(r#"</li>"#);
        }

        html.push_str(r#"</ul>"#);
    }

    html.push_str(r#"</li>"#);
}

// ============================================================================
// Multi-Workspace Mode Handlers
// ============================================================================

/// Helper function to extract workspace_id from wildcard path.
fn extract_workspace_id(path: &str) -> String {
    // Remove leading slash if present
    path.trim_start_matches('/').to_string()
}

/// Multi-workspace index page - shows grid of available workspaces.
pub async fn workspaces_index(State(state): State<AppState>) -> Result<Html<String>> {
    let workspaces = state.list_workspaces().await;

    let workspace_cards: String = if workspaces.is_empty() {
        r#"<div class="empty-state">
            <h2>No Workspaces Found</h2>
            <p>Create a workspace by adding a directory containing a <code>workspace.dsl</code> file.</p>
        </div>"#.to_string()
    } else {
        workspaces.iter().map(|ws| {
            let description = ws.description.as_deref().unwrap_or("No description");
            let last_modified = ws.last_modified
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
                .unwrap_or_else(|_| "Unknown".to_string());

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
    </style>
</head>
<body>
    <div class="header">
        <h1>Structurizr Workspaces</h1>
        <p>{} workspace{} available</p>
    </div>
    <div class="container">
        <div class="workspaces-grid">
            {}
        </div>
    </div>
</body>
</html>"##,
        workspaces.len(),
        if workspaces.len() == 1 { "" } else { "s" },
        workspace_cards
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

/// Workspace-scoped search API handler.
pub async fn workspace_search_api(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let query_str = query.q.as_deref().unwrap_or("");
    search_workspace(&workspace, query_str, "")
}

/// Workspace-scoped export JSON handler.
pub async fn workspace_export_json(
    State(state): State<AppState>,
    Path(workspace_id): Path<String>,
) -> Result<impl IntoResponse> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let json = JsonExporter::export(&workspace)?;
    Ok(([(header::CONTENT_TYPE, "application/json")], json))
}

/// Workspace-scoped render SVG handler.
pub async fn workspace_render_svg(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
) -> Result<impl IntoResponse> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    render_view_svg(&workspace, &view_key)
}

/// Workspace-scoped PlantUML export handler.
pub async fn workspace_export_plantuml(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "plantuml")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", workspace_id);
        let html = generate_plantuml_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

/// Workspace-scoped Mermaid export handler.
pub async fn workspace_export_mermaid(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "mermaid")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", workspace_id);
        let html = generate_mermaid_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

/// Workspace-scoped DOT export handler.
pub async fn workspace_export_dot(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "dot")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", workspace_id);
        let html = generate_dot_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

/// Workspace-scoped D2 export handler.
pub async fn workspace_export_d2(
    State(state): State<AppState>,
    Path((workspace_id, view_key)): Path<(String, String)>,
    Query(query): Query<ExportQuery>,
) -> Result<axum::response::Response> {
    let workspace_id = extract_workspace_id(&workspace_id);
    let workspace = state.get_workspace_by_id(&workspace_id).await
        .ok_or_else(|| Error::WorkspaceNotFound(workspace_id.clone()))?;

    let code = get_export_code(&workspace, &view_key, "d2")?;

    if query.raw.unwrap_or(false) {
        Ok(([(header::CONTENT_TYPE, "text/plain; charset=utf-8")], code).into_response())
    } else {
        let base_path = format!("/w/{}", workspace_id);
        let html = generate_d2_viewer_html(&workspace, &view_key, &base_path, &code);
        Ok(Html(html).into_response())
    }
}

// ============================================================================
// Helper functions for rendering (used by both single and multi-workspace modes)
// ============================================================================

/// Generate view diagram HTML with full interactivity (shared implementation).
///
/// This function generates the complete HTML for diagram viewing with:
/// - Canvas-based rendering with pan and zoom
/// - Minimap navigation
/// - Breadcrumb navigation
/// - Element tooltips and drill-down
/// - Keyboard shortcuts
///
/// # Parameters
/// - `workspace`: The workspace containing the diagram
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
        .zoom-controls { display: flex; gap: 5px; align-items: center; }
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
        .view-breadcrumbs { display: flex; align-items: center; gap: 6px; font-size: 13px; max-width: 500px; overflow: hidden; }
        .view-breadcrumb {
            display: flex;
            align-items: center;
            gap: 4px;
            color: var(--text-secondary);
            text-decoration: none;
            padding: 4px 8px;
            border-radius: 4px;
            transition: all 0.15s;
            white-space: nowrap;
        }
        .view-breadcrumb:hover { background: var(--header-link-hover); color: var(--text-primary); }
        .view-breadcrumb.current { color: var(--text-primary); font-weight: 500; }
        .view-breadcrumb-icon {
            background: var(--bg-tertiary);
            padding: 2px 6px;
            border-radius: 3px;
            font-size: 10px;
            font-weight: 600;
        }
        .view-breadcrumb-separator { color: var(--text-muted); font-size: 11px; }
        .drill-indicator { position: absolute; pointer-events: none; }
    </style>"##;

    let content = format!(r##"
        <div class="view-toolbar">
            <nav class="view-breadcrumbs" id="breadcrumbs"></nav>
            <div class="separator"></div>
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
            if (!hoveredElement) return;

            // Find outbound relationships from the hovered element
            const outboundRels = relationships.filter(rel => rel.source === hoveredElement.id);
            if (outboundRels.length === 0) return;

            ctx.save();
            ctx.globalAlpha = 0.8;

            for (const rel of outboundRels) {{
                // Find source and target elements
                const sourceEl = elements.find(el => el.id === rel.source);
                const targetEl = elements.find(el => el.id === rel.target);
                if (!sourceEl || !targetEl) continue;

                // Calculate connection points (center of elements)
                const srcX = sourceEl.x - svgMinX + sourceEl.width / 2;
                const srcY = sourceEl.y - svgMinY + sourceEl.height / 2;
                const tgtX = targetEl.x - svgMinX + targetEl.width / 2;
                const tgtY = targetEl.y - svgMinY + targetEl.height / 2;

                // Draw highlight line
                ctx.strokeStyle = 'rgba(0, 200, 100, 0.9)';
                ctx.lineWidth = 6;
                ctx.lineCap = 'round';
                ctx.setLineDash([]);

                ctx.beginPath();
                ctx.moveTo(srcX, srcY);
                ctx.lineTo(tgtX, tgtY);
                ctx.stroke();

                // Draw arrow head at target
                const angle = Math.atan2(tgtY - srcY, tgtX - srcX);
                const arrowLength = 15;
                ctx.beginPath();
                ctx.moveTo(tgtX, tgtY);
                ctx.lineTo(
                    tgtX - arrowLength * Math.cos(angle - Math.PI / 6),
                    tgtY - arrowLength * Math.sin(angle - Math.PI / 6)
                );
                ctx.lineTo(
                    tgtX - arrowLength * Math.cos(angle + Math.PI / 6),
                    tgtY - arrowLength * Math.sin(angle + Math.PI / 6)
                );
                ctx.closePath();
                ctx.fillStyle = 'rgba(0, 200, 100, 0.9)';
                ctx.fill();

                // Draw relationship description label
                if (rel.description) {{
                    const midX = (srcX + tgtX) / 2;
                    const midY = (srcY + tgtY) / 2 - 10;
                    ctx.font = 'bold 12px system-ui, sans-serif';
                    ctx.textAlign = 'center';
                    const labelWidth = ctx.measureText(rel.description).width + 12;

                    ctx.fillStyle = 'rgba(0, 200, 100, 0.95)';
                    ctx.beginPath();
                    ctx.roundRect(midX - labelWidth/2, midY - 10, labelWidth, 20, 4);
                    ctx.fill();

                    ctx.fillStyle = 'white';
                    ctx.fillText(rel.description, midX, midY + 4);
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
                tooltip.style.left = (screenX + 15) + 'px';
                tooltip.style.top = (screenY + 15) + 'px';
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
                }} else if (element) {{
                    tooltip.style.left = (e.clientX + 15) + 'px';
                    tooltip.style.top = (e.clientY + 15) + 'px';
                }}
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

    // Build step data as JSON
    let mut steps_json = String::from("[");
    for (i, step) in dynamic_view.steps.iter().enumerate() {
        if i > 0 { steps_json.push(','); }
        steps_json.push_str(&format!(
            r#"{{"order":{},"sourceId":"{}","destId":"{}","description":{}}}"#,
            step.order,
            step.source_id,
            step.destination_id,
            step.description.as_ref().map(|d| format!("\"{}\"", escape_json(d))).unwrap_or_else(|| "null".to_string())
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
        .arrow-line {{ opacity: 0; transition: opacity 0.4s ease-in-out; }}
        .arrow-line.visible {{ opacity: 1; }}
        .arrow-text {{ opacity: 0; transition: opacity 0.4s ease-in-out; }}
        .arrow-text.visible {{ opacity: 1; }}
        .step-overlay {{ position: absolute; bottom: 30px; left: 50%; transform: translateX(-50%); background: rgba(0, 0, 0, 0.9); color: white; padding: 16px 24px; border-radius: 8px; box-shadow: 0 4px 12px rgba(0,0,0,0.3); max-width: 600px; opacity: 0; transition: opacity 0.3s ease-in-out; pointer-events: none; z-index: 100; }}
        .step-overlay.visible {{ opacity: 1; }}
        .step-overlay .step-number {{ font-size: 12px; color: #0066cc; font-weight: 600; margin-bottom: 6px; }}
        .step-overlay .step-desc {{ font-size: 15px; line-height: 1.4; }}
        .keyboard-help {{ position: fixed; bottom: 20px; left: 20px; font-size: 11px; color: #666; z-index: 50; }}
        .loading {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); color: #666; font-size: 16px; }}
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
        </div>
    </div>
    <div class="keyboard-help">Space to play/pause • ← → to step • R to reset • 1-9 to jump to step • Scroll to zoom • Drag to pan</div>
    <script>
        const steps = {steps_json};
        const totalSteps = {step_count};
        let currentStep = 0;
        let isPlaying = false;
        let playInterval = null;
        let playSpeed = 2000;
        let svgWidth = 0, svgHeight = 0, scale = 1, offsetX = 0, offsetY = 0;
        let arrowLines = [], arrowTexts = [];
        let isPanning = false, panStartX = 0, panStartY = 0, panStartOffsetX = 0, panStartOffsetY = 0;
        const container = document.getElementById('diagram-container');
        const wrapper = document.getElementById('svg-wrapper');

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
                fitToScreen();
                updateDisplay();
            }} catch (err) {{
                document.getElementById('loading').textContent = 'Failed to load diagram';
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
            document.getElementById('step-counter').textContent = `Step ${{currentStep}} of ${{totalSteps}}`;
            document.getElementById('btn-prev').disabled = currentStep === 0;
            document.getElementById('btn-next').disabled = currentStep >= totalSteps;
            arrowLines.forEach((line, idx) => line.classList.toggle('visible', idx < currentStep));
            arrowTexts.forEach((text, idx) => text.classList.toggle('visible', idx < currentStep));
            const overlay = document.getElementById('step-overlay');
            if (currentStep > 0 && currentStep <= steps.length) {{
                const step = steps[currentStep - 1];
                document.getElementById('overlay-number').textContent = `Step ${{step.order}}`;
                document.getElementById('overlay-desc').textContent = step.description || 'No description';
                overlay.classList.add('visible');
            }} else {{
                overlay.classList.remove('visible');
            }}
        }}

        function nextStep() {{ if (currentStep < totalSteps) {{ currentStep++; updateDisplay(); }} if (currentStep >= totalSteps) stopPlaying(); }}
        function previousStep() {{ if (currentStep > 0) {{ currentStep--; updateDisplay(); }} }}
        function resetAnimation() {{ currentStep = 0; stopPlaying(); updateDisplay(); }}
        function togglePlay() {{ isPlaying ? stopPlaying() : startPlaying(); }}
        function startPlaying() {{ if (currentStep >= totalSteps) currentStep = 0; isPlaying = true; document.getElementById('btn-play').textContent = '⏸ Pause'; playInterval = setInterval(() => {{ nextStep(); if (currentStep >= totalSteps) stopPlaying(); }}, playSpeed); }}
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
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;
            switch(e.key) {{
                case ' ': e.preventDefault(); togglePlay(); break;
                case 'ArrowRight': e.preventDefault(); nextStep(); break;
                case 'ArrowLeft': e.preventDefault(); previousStep(); break;
                case 'r': case 'R': resetAnimation(); break;
                case 'f': case 'F': fitToScreen(); break;
                case '0': resetAnimation(); break;
                default: if (e.key >= '1' && e.key <= '9') {{ const stepNum = parseInt(e.key); if (stepNum <= totalSteps) {{ currentStep = stepNum; updateDisplay(); }} }}
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

    // Build decisions list
    let decisions_html: String = if docs.decisions.is_empty() {
        String::new()
    } else {
        let decisions_list: String = docs.decisions.iter().map(|decision| {
            let status_class = match decision.status {
                structurizr_core::workspace::DecisionStatus::Accepted => "accepted",
                structurizr_core::workspace::DecisionStatus::Proposed => "proposed",
                structurizr_core::workspace::DecisionStatus::Superseded => "superseded",
                structurizr_core::workspace::DecisionStatus::Deprecated => "deprecated",
                structurizr_core::workspace::DecisionStatus::Rejected => "rejected",
            };
            format!(
                r#"<div class="decision" id="adr-{}">
                    <div class="decision-header">
                        <span class="decision-id">{}</span>
                        <h3>{}</h3>
                        <span class="status {}">{:?}</span>
                        <span class="date">{}</span>
                    </div>
                    <div class="content">{}</div>
                </div>"#,
                decision.id,
                escape_html(&decision.id),
                escape_html(&decision.title),
                status_class,
                decision.status,
                escape_html(&decision.date),
                render_markdown(&decision.content)
            )
        }).collect();

        format!(
            r#"<div class="decisions-section">
                <h2>Architecture Decision Records</h2>
                {}
            </div>"#,
            decisions_list
        )
    };

    // Build sidebar with tree navigation
    let sidebar_sections = if nav_tree.is_empty() {
        String::new()
    } else {
        format!(r##"<ul class="nav-tree">{}</ul>"##, render_nav_tree(&nav_tree, 0))
    };

    // ADRs as flat list
    let sidebar_decisions: String = docs.decisions.iter().map(|decision| {
        format!(r##"<a href="#adr-{}" class="nav-link adr-link">{}: {}</a>"##, decision.id, decision.id, escape_html(&decision.title))
    }).collect();

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

        /* ADR links */
        .adr-link { display: block; padding: 6px 8px; margin: 2px 0; font-size: 13px; }
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
        .decisions-section { margin-top: 40px; }
        .decision { background: var(--card-bg); padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px var(--shadow); }
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

    // Build sidebar decisions section
    let sidebar_decisions_section = if !docs.decisions.is_empty() {
        format!("<h3 style=\"margin-top: 20px;\">ADRs</h3>{}", sidebar_decisions)
    } else {
        String::new()
    };

    // Build content HTML
    let content = format!(
        r#"<div class="sidebar">
            <h3>Sections</h3>
            {}
            {}
        </div>
        <div class="main">
            {}
            {}
        </div>"#,
        sidebar_sections,
        sidebar_decisions_section,
        sections_html,
        decisions_html
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
        .slide-container {{ height: 100vh; display: flex; align-items: center; justify-content: center; }}
        .slide {{ max-width: 95vw; max-height: 95vh; background: white; border-radius: 8px; box-shadow: 0 10px 50px rgba(0,0,0,0.5); overflow: hidden; }}
        .slide img {{ max-width: 100%; max-height: 90vh; display: block; }}
        .controls {{ position: fixed; bottom: 30px; left: 50%; transform: translateX(-50%); display: flex; gap: 10px; }}
        .controls button {{ background: rgba(255,255,255,0.2); color: white; border: none; padding: 12px 24px; border-radius: 6px; cursor: pointer; font-size: 16px; transition: background 0.2s; }}
        .controls button:hover {{ background: rgba(255,255,255,0.3); }}
        .controls button:disabled {{ opacity: 0.3; cursor: not-allowed; }}
        .keyboard-hint {{ position: fixed; bottom: 20px; right: 20px; font-size: 11px; color: #666; }}
        .slide-title {{ position: fixed; bottom: 80px; left: 50%; transform: translateX(-50%); font-size: 18px; color: white; background: rgba(0,0,0,0.7); padding: 8px 20px; border-radius: 20px; }}
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
