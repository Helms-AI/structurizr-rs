//! Shared layout infrastructure for web pages.
//!
//! Provides a standardized layout system with consistent navigation header,
//! breadcrumbs, and dark/light mode theming across all workspace pages.

use crate::markdown::escape_html;

/// Configuration for page layout generation.
pub struct LayoutConfig<'a> {
    /// Page title (shown in browser tab)
    pub title: &'a str,
    /// Workspace name (for display in breadcrumb)
    pub workspace_name: Option<&'a str>,
    /// Workspace ID (for multi-workspace breadcrumb links). None for single-workspace mode.
    pub workspace_id: Option<&'a str>,
    /// Base path for navigation links ("" for single-workspace, "/w/{id}" for multi)
    pub base_path: &'a str,
    /// Currently active navigation item
    pub active_nav: NavItem,
    /// Content container type
    pub content_type: ContentType,
    /// Additional content for <head> (page-specific styles)
    pub extra_head: &'a str,
    /// Additional content before </body> (page-specific scripts)
    pub extra_body_end: &'a str,
}

/// Navigation items for the header.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum NavItem {
    Home,
    Docs,
    Decisions,
    Tree,
    Search,
    Explore,
    Presentation,
    Api,
    Export,
    View,
    Edit,
    None,
}

/// Content container types for different page layouts.
#[derive(Clone, Copy)]
pub enum ContentType {
    /// Standard padded container for content pages (max-width: 1200px)
    Standard,
    /// Full-height sidebar layout (documentation)
    Sidebar,
    /// Full-viewport for canvas-based pages (diagrams, explore)
    FullViewport,
    /// Full-viewport with toolbar area (edit, explore with controls)
    ToolbarViewport,
}

/// Generate CSS variables for theme support.
fn generate_theme_css() -> &'static str {
    r##"
        :root {
            /* Light theme (default) */
            --bg-primary: #f5f5f5;
            --bg-secondary: #ffffff;
            --bg-tertiary: #e8e8e8;
            --text-primary: #333333;
            --text-secondary: #666666;
            --text-muted: #888888;
            --border-color: #dddddd;
            --link-color: #0066cc;
            --link-hover: #0052a3;
            --shadow: rgba(0, 0, 0, 0.1);
            --shadow-medium: rgba(0, 0, 0, 0.15);

            /* Header */
            --header-bg: #333333;
            --header-text: #ffffff;
            --header-link-hover: rgba(255, 255, 255, 0.1);
            --header-link-active: rgba(255, 255, 255, 0.15);

            /* Canvas/interactive specific */
            --canvas-bg: #f0f0f0;
            --canvas-border: #cccccc;
            --toolbar-bg: #ffffff;
            --toolbar-border: #dddddd;
            --toolbar-text: #333333;

            /* Cards and containers */
            --card-bg: #ffffff;
            --card-hover: #fafafa;

            /* Status colors */
            --status-accepted-bg: #d4edda;
            --status-accepted-text: #155724;
            --status-proposed-bg: #fff3cd;
            --status-proposed-text: #856404;
            --status-superseded-bg: #e2e3e5;
            --status-superseded-text: #383d41;
            --status-deprecated-bg: #f8d7da;
            --status-deprecated-text: #721c24;

            /* Code */
            --code-bg: #f0f0f0;
            --code-text: #333333;
            --pre-bg: #f5f5f5;
        }

        [data-theme="dark"] {
            /* Dark theme */
            --bg-primary: #1a1a1a;
            --bg-secondary: #2a2a2a;
            --bg-tertiary: #333333;
            --text-primary: #ffffff;
            --text-secondary: #cccccc;
            --text-muted: #888888;
            --border-color: #444444;
            --link-color: #6cb6ff;
            --link-hover: #88c8ff;
            --shadow: rgba(0, 0, 0, 0.3);
            --shadow-medium: rgba(0, 0, 0, 0.4);

            /* Header */
            --header-bg: #2a2a2a;
            --header-text: #ffffff;
            --header-link-hover: rgba(255, 255, 255, 0.1);
            --header-link-active: rgba(255, 255, 255, 0.15);

            /* Canvas/interactive specific */
            --canvas-bg: #1a1a1a;
            --canvas-border: #444444;
            --toolbar-bg: #2a2a2a;
            --toolbar-border: #444444;
            --toolbar-text: #ffffff;

            /* Cards and containers */
            --card-bg: #2a2a2a;
            --card-hover: #333333;

            /* Status colors - slightly muted for dark mode */
            --status-accepted-bg: #1e4620;
            --status-accepted-text: #a3d9a5;
            --status-proposed-bg: #4a3f1a;
            --status-proposed-text: #ffd666;
            --status-superseded-bg: #3a3a3a;
            --status-superseded-text: #b0b0b0;
            --status-deprecated-bg: #4a1a1a;
            --status-deprecated-text: #ff9999;

            /* Code */
            --code-bg: #333333;
            --code-text: #e0e0e0;
            --pre-bg: #2a2a2a;
        }
    "##
}

/// Generate JavaScript for theme toggle and localStorage persistence.
fn generate_theme_js() -> &'static str {
    r##"<script>
    (function() {
        // Initialize theme from localStorage or system preference
        function getPreferredTheme() {
            try {
                const stored = localStorage.getItem('structurizr-theme');
                if (stored === 'dark' || stored === 'light') return stored;
            } catch (e) {
                // localStorage not available
            }
            return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
        }

        function setTheme(theme) {
            document.documentElement.setAttribute('data-theme', theme);
            try {
                localStorage.setItem('structurizr-theme', theme);
            } catch (e) {
                // localStorage not available
            }
            updateIcon(theme);
        }

        function updateIcon(theme) {
            const icon = document.querySelector('.theme-icon');
            if (icon) {
                // Sun for light mode, Moon for dark mode
                icon.textContent = theme === 'dark' ? '\u263E' : '\u2600';
            }
        }

        // Apply theme immediately to prevent flash
        const initialTheme = getPreferredTheme();
        document.documentElement.setAttribute('data-theme', initialTheme);

        // Set up icon once DOM is ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', function() {
                updateIcon(initialTheme);
            });
        } else {
            updateIcon(initialTheme);
        }

        // Global toggle function
        window.toggleTheme = function() {
            const current = document.documentElement.getAttribute('data-theme') || 'light';
            setTheme(current === 'dark' ? 'light' : 'dark');
        };

        // Listen for system preference changes
        try {
            window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', function(e) {
                // Only auto-switch if user hasn't set a preference
                try {
                    if (!localStorage.getItem('structurizr-theme')) {
                        setTheme(e.matches ? 'dark' : 'light');
                    }
                } catch (err) {
                    setTheme(e.matches ? 'dark' : 'light');
                }
            });
        } catch (e) {
            // matchMedia not fully supported
        }
    })();
    </script>"##
}

/// Generate breadcrumb HTML based on workspace context.
fn generate_breadcrumb(config: &LayoutConfig) -> String {
    match (config.workspace_id, config.workspace_name) {
        (Some(id), Some(_name)) => {
            // Multi-workspace mode: "All Workspaces > {workspace-id}"
            // The workspace name links back to the workspace index page
            // Replace "/" in workspace ID with " > " for display
            let display_id = id.replace("/", " > ");
            format!(
                r#"<div class="breadcrumb">
                    <a href="/">All Workspaces</a>
                    <span class="separator">&gt;</span>
                    <a href="/w/{}" class="workspace-link">{}</a>
                </div>"#,
                escape_html(id),
                escape_html(&display_id),
            )
        }
        (Some(id), None) => {
            // Multi-workspace mode without name
            // Replace "/" in workspace ID with " > " for display
            let display_id = id.replace("/", " > ");
            format!(
                r#"<div class="breadcrumb">
                    <a href="/">All Workspaces</a>
                    <span class="separator">&gt;</span>
                    <a href="/w/{}" class="workspace-link">{}</a>
                </div>"#,
                escape_html(id),
                escape_html(&display_id),
            )
        }
        (None, Some(name)) => {
            // Single-workspace mode: just show workspace name (no link needed, already on home)
            format!(
                r#"<div class="breadcrumb">
                    <a href="/" class="workspace-link">{}</a>
                </div>"#,
                escape_html(name)
            )
        }
        (None, None) => String::new(),
    }
}

/// Generate navigation links HTML.
fn generate_navigation(config: &LayoutConfig) -> String {
    let base = config.base_path;
    let home_href = if base.is_empty() {
        "/".to_string()
    } else {
        base.to_string()
    };

    let nav_items: Vec<(NavItem, String, &str)> = vec![
        (NavItem::Home, home_href, "Home"),
        (NavItem::Tree, format!("{}/tree", base), "Tree"),
        (NavItem::Docs, format!("{}/docs", base), "Docs"),
        (NavItem::Decisions, format!("{}/decisions", base), "Decisions"),
        (NavItem::Search, format!("{}/search", base), "Search"),
        (NavItem::Explore, format!("{}/explore", base), "Explore"),
        (NavItem::Presentation, format!("{}/presentation", base), "Presentation"),
        (NavItem::Api, format!("{}/api/workspace", base), "API"),
        (NavItem::Export, format!("{}/export/json", base), "Export"),
    ];

    nav_items
        .iter()
        .map(|(item, href, label)| {
            let active = if *item == config.active_nav {
                " active"
            } else {
                ""
            };
            // Fix href for single-workspace (avoid "//tree")
            let actual_href = if href.starts_with("//") {
                &href[1..]
            } else {
                href.as_str()
            };
            format!(
                r#"<a href="{}" class="nav-link{}">{}</a>"#,
                actual_href, active, label
            )
        })
        .collect()
}

/// Generate theme toggle button HTML.
fn generate_theme_toggle() -> &'static str {
    r#"<button class="theme-toggle" onclick="toggleTheme()" title="Toggle dark/light mode" aria-label="Toggle theme">
        <span class="theme-icon">&#9728;</span>
    </button>"#
}

/// Generate the base CSS for the layout.
fn generate_base_css() -> &'static str {
    r##"
        * { box-sizing: border-box; }

        body {
            margin: 0;
            padding: 0;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            background: var(--bg-primary);
            color: var(--text-primary);
            transition: background-color 0.2s ease, color 0.2s ease;
        }

        a {
            color: var(--link-color);
            text-decoration: none;
        }

        a:hover {
            color: var(--link-hover);
            text-decoration: underline;
        }

        /* Header styles */
        .app-header {
            background: var(--header-bg);
            color: var(--header-text);
            padding: 0 20px;
            position: sticky;
            top: 0;
            z-index: 100;
        }

        .header-top {
            display: flex;
            align-items: center;
            justify-content: space-between;
            height: 40px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }

        .header-bottom {
            display: flex;
            align-items: center;
            height: 40px;
        }

        .breadcrumb {
            font-size: 14px;
            display: flex;
            align-items: center;
        }

        .breadcrumb a {
            color: var(--link-color);
            text-decoration: none;
        }

        .breadcrumb a:hover {
            text-decoration: underline;
        }

        .breadcrumb .separator {
            color: var(--text-muted);
            margin: 0 8px;
        }

        .breadcrumb .workspace-link {
            color: var(--header-text);
            font-weight: 500;
        }

        .breadcrumb .workspace-link:hover {
            color: var(--link-color);
        }

        .nav-links {
            display: flex;
            gap: 4px;
        }

        .nav-links a {
            color: var(--header-text);
            text-decoration: none;
            padding: 8px 12px;
            border-radius: 6px;
            font-size: 14px;
            transition: background 0.15s ease;
        }

        .nav-links a:hover {
            background: var(--header-link-hover);
            text-decoration: none;
        }

        .nav-links a.active {
            background: var(--header-link-active);
            font-weight: 500;
        }

        .header-right {
            display: flex;
            align-items: center;
            gap: 12px;
        }

        /* Theme toggle button */
        .theme-toggle {
            background: transparent;
            border: 1px solid rgba(255, 255, 255, 0.2);
            color: var(--header-text);
            padding: 6px 10px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 16px;
            transition: background 0.15s ease;
            line-height: 1;
        }

        .theme-toggle:hover {
            background: var(--header-link-hover);
        }

        .theme-icon {
            display: inline-block;
        }

        /* Container types */
        .layout-content.standard {
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
        }

        .layout-content.sidebar-layout {
            display: flex;
            height: calc(100vh - 80px);
        }

        .layout-content.full-viewport {
            height: calc(100vh - 80px);
            overflow: hidden;
        }

        .layout-content.toolbar-viewport {
            height: calc(100vh - 80px);
            display: flex;
            flex-direction: column;
        }
    "##
}

/// Generate the complete page wrapper with header, navigation, and theme toggle.
pub fn generate_page_layout(config: &LayoutConfig, content: &str) -> String {
    let breadcrumb_html = generate_breadcrumb(config);
    let nav_html = generate_navigation(config);
    let theme_toggle_html = generate_theme_toggle();
    let theme_css = generate_theme_css();
    let theme_js = generate_theme_js();
    let base_css = generate_base_css();

    let container_class = match config.content_type {
        ContentType::Standard => "layout-content standard",
        ContentType::Sidebar => "layout-content sidebar-layout",
        ContentType::FullViewport => "layout-content full-viewport",
        ContentType::ToolbarViewport => "layout-content toolbar-viewport",
    };

    format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>{title} - Structurizr</title>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <style>
        {theme_css}
        {base_css}
    </style>
    {extra_head}
</head>
<body>
    <header class="app-header">
        <div class="header-top">
            {breadcrumb_html}
            <div class="header-right">
                {theme_toggle_html}
            </div>
        </div>
        <div class="header-bottom">
            <nav class="nav-links">
                {nav_html}
            </nav>
        </div>
    </header>
    <main class="{container_class}">
        {content}
    </main>
    {theme_js}
    {extra_body_end}
</body>
</html>"##,
        title = escape_html(config.title),
        theme_css = theme_css,
        base_css = base_css,
        extra_head = config.extra_head,
        breadcrumb_html = breadcrumb_html,
        theme_toggle_html = theme_toggle_html,
        nav_html = nav_html,
        container_class = container_class,
        content = content,
        theme_js = theme_js,
        extra_body_end = config.extra_body_end,
    )
}
