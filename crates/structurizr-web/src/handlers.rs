//! HTTP request handlers.

use axum::{
    extract::{Path, State},
    http::header,
    response::{Html, IntoResponse},
    Json,
};

use structurizr_core::view::SystemLandscapeView;
use structurizr_core::Workspace;
use structurizr_export::{D2Exporter, DotExporter, JsonExporter, MermaidExporter, PlantUmlExporter};
use structurizr_render::SvgRenderer;
use structurizr_render::layout::{GridLayout, LayoutEdge};

use crate::error::{Error, Result};
use crate::state::AppState;

/// Home page handler.
pub async fn index(State(state): State<AppState>) -> Result<Html<String>> {
    let workspace = state.get_workspace().await;

    let html = if let Some(ws) = workspace {
        let views = ws.views();
        let view_list: Vec<String> = views.all_keys().iter().map(|k| k.to_string()).collect();

        // Check which views are dynamic views
        let dynamic_view_keys: std::collections::HashSet<String> = views.dynamic_views.iter()
            .map(|v| v.properties.key.clone())
            .collect();

        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{} - Structurizr</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; margin: 0; padding: 20px; background: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        h1 {{ color: #333; }}
        .workspace-info {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .views {{ display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }}
        .view-card {{ background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .view-card h3 {{ margin-top: 0; }}
        .view-card a {{ color: #0066cc; text-decoration: none; }}
        .view-card a:hover {{ text-decoration: underline; }}
        .nav {{ margin-bottom: 20px; }}
        .nav a {{ margin-right: 15px; color: #0066cc; text-decoration: none; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="nav">
            <a href="/">Home</a>
            <a href="/tree">Tree View</a>
            <a href="/docs">Documentation</a>
            <a href="/search">Search</a>
            <a href="/explore">Explore Graph</a>            <a href="/presentation">Presentation Mode</a>
            <a href="/api/workspace">API</a>
            <a href="/export/json">Export JSON</a>
        </div>
        <h1>{}</h1>
        <div class="workspace-info">
            <p>{}</p>
            <p><strong>People:</strong> {}</p>
            <p><strong>Software Systems:</strong> {}</p>
            <p><strong>Relationships:</strong> {}</p>
        </div>
        <h2>Views</h2>
        <div class="views">
            {}
        </div>
    </div>
</body>
</html>"#,
            ws.name,
            ws.name,
            ws.description.as_deref().unwrap_or(""),
            ws.model().people.len(),
            ws.model().software_systems.len(),
            ws.model().relationships.len(),
            view_list.iter().map(|v| {
                let animate_link = if dynamic_view_keys.contains(v) {
                    format!(r#" | <a href="/view/{}/animate">Animate</a>"#, v)
                } else {
                    String::new()
                };
                format!(
                    r#"<div class="view-card"><h3><a href="/view/{}">{}</a></h3><p><a href="/edit/{}">Edit</a> | <a href="/presentation?views={}">Present</a>{} | <a href="/view/{}/svg">SVG</a> | <a href="/view/{}/plantuml">PlantUML</a> | <a href="/view/{}/mermaid">Mermaid</a> | <a href="/view/{}/dot">DOT</a> | <a href="/view/{}/d2">D2</a></p></div>"#,
                    v, v, v, v, animate_link, v, v, v, v, v
                )
            }).collect::<Vec<_>>().join("\n            ")
        )
    } else {
        r#"<!DOCTYPE html>
<html>
<head><title>Structurizr</title></head>
<body>
    <h1>No workspace loaded</h1>
    <p>Create a workspace.dsl file in the data directory.</p>
</body>
</html>"#.to_string()
    };

    Ok(Html(html))
}

/// Get workspace as JSON.
pub async fn get_workspace(State(state): State<AppState>) -> Result<Json<structurizr_core::Workspace>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    Ok(Json(workspace))
}

/// Validate workspace and return issues.
pub async fn validate_workspace(State(state): State<AppState>) -> Result<Json<structurizr_dsl::ValidationResult>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let validation_result = structurizr_dsl::validate_workspace(&workspace);
    Ok(Json(validation_result))
}

/// Export workspace as JSON.
pub async fn export_json(State(state): State<AppState>) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let json = JsonExporter::export(&workspace)?;

    Ok((
        [(header::CONTENT_TYPE, "application/json")],
        json
    ))
}

/// View a diagram.
pub async fn view_diagram(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let svg_url = format!("/view/{}/svg", view_key);
    let model = workspace.model();

    // Collect elements and compute layout to get positions
    let mut element_ids: Vec<String> = Vec::new();
    let mut element_data: Vec<(String, String, String, Option<String>, Option<String>)> = Vec::new(); // (id, name, type, description, technology)

    for person in &model.people {
        element_ids.push(person.id().to_string());
        element_data.push((
            person.id().to_string(),
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
            system.name().to_string(),
            "Software System".to_string(),
            system.properties.description.clone(),
            None,
        ));

        for container in &system.containers {
            element_ids.push(container.id().to_string());
            element_data.push((
                container.id().to_string(),
                container.name().to_string(),
                "Container".to_string(),
                container.properties.description.clone(),
                container.technology.clone(),
            ));
        }
    }

    // Build edges from relationships
    let edges: Vec<LayoutEdge> = model
        .relationships
        .iter()
        .map(|r| LayoutEdge {
            source: r.source_id.to_string(),
            target: r.destination_id.to_string(),
        })
        .collect();

    // Compute layout
    let layout = GridLayout::default();
    let nodes = layout.layout(&element_ids, &edges);

    // Build elements JSON with position data for hit-testing
    let mut elements_json = String::from("[");
    for (i, (id, name, elem_type, desc, tech)) in element_data.iter().enumerate() {
        if i > 0 { elements_json.push(','); }

        // Find the corresponding layout node
        let (x, y, width, height) = nodes.iter()
            .find(|n| &n.id == id)
            .map(|n| (n.position.x, n.position.y, n.size.width, n.size.height))
            .unwrap_or((0.0, 0.0, 450.0, 300.0));

        elements_json.push_str(&format!(
            r#"{{"id":"{}","name":"{}","type":"{}","description":{},"technology":{},"x":{},"y":{},"width":{},"height":{}}}"#,
            escape_json(id),
            escape_json(name),
            escape_json(elem_type),
            desc.as_ref().map(|d| format!("\"{}\"", escape_json(d))).unwrap_or_else(|| "null".to_string()),
            tech.as_ref().map(|t| format!("\"{}\"", escape_json(t))).unwrap_or_else(|| "null".to_string()),
            x as i32,
            y as i32,
            width as i32,
            height as i32,
        ));
    }
    elements_json.push(']');

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>{} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; background: #1a1a1a; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; height: 100vh; overflow: hidden; }}
        .toolbar {{ background: #333; color: white; padding: 10px 20px; display: flex; align-items: center; gap: 20px; border-bottom: 1px solid #444; }}
        .toolbar a {{ color: white; text-decoration: none; }}
        .toolbar a:hover {{ text-decoration: underline; }}
        .toolbar button {{ background: #555; color: white; border: none; padding: 6px 12px; border-radius: 4px; cursor: pointer; }}
        .toolbar button:hover {{ background: #666; }}
        .toolbar .separator {{ border-left: 1px solid #555; height: 20px; }}
        .zoom-controls {{ display: flex; gap: 5px; align-items: center; }}
        .zoom-level {{ font-size: 12px; min-width: 50px; text-align: center; }}
        .diagram-container {{ height: calc(100vh - 50px); overflow: hidden; position: relative; background: #2a2a2a; }}
        #diagram-canvas {{ width: 100%; height: 100%; cursor: grab; }}
        #diagram-canvas.dragging {{ cursor: grabbing; }}
        .tooltip {{ position: fixed; background: #333; color: white; padding: 12px 16px; border-radius: 6px; font-size: 13px; max-width: 300px; z-index: 1000; pointer-events: none; box-shadow: 0 4px 12px rgba(0,0,0,0.3); display: none; }}
        .tooltip h4 {{ margin: 0 0 6px 0; font-size: 14px; }}
        .tooltip .type {{ color: #888; font-size: 11px; margin-bottom: 8px; }}
        .tooltip .desc {{ line-height: 1.4; }}
        .tooltip .tech {{ color: #6af; margin-top: 6px; font-size: 12px; }}
        .minimap {{ position: absolute; bottom: 20px; right: 20px; width: 200px; height: 150px; background: #333; border: 1px solid #555; border-radius: 4px; overflow: hidden; cursor: crosshair; }}
        .minimap-canvas {{ width: 100%; height: 100%; opacity: 0.7; }}
        .minimap .viewport {{ position: absolute; border: 2px solid #0066cc; background: rgba(0,102,204,0.1); cursor: grab; transition: background 0.15s; }}
        .minimap .viewport:hover {{ background: rgba(0,102,204,0.25); }}
        .keyboard-help {{ position: fixed; bottom: 20px; left: 20px; font-size: 11px; color: #666; }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="/">← Back</a>
        <span>{}</span>
        <div class="separator"></div>
        <div class="zoom-controls">
            <button onclick="zoomOut()">−</button>
            <span class="zoom-level" id="zoom-level">100%</span>
            <button onclick="zoomIn()">+</button>
            <button onclick="resetZoom()">Reset</button>
            <button onclick="fitToScreen()">Fit</button>
        </div>
        <div class="separator"></div>
        <a href="/edit/{}">Edit</a>
        <a href="{}" download="{}.svg">Download SVG</a>
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
        Scroll to zoom • Drag to pan • Double-click to zoom in • Hover elements for info
    </div>

    <script>
        // Element data with positions for hit-testing
        const elements = {};

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
            console.log('Container rect:', rect.width, 'x', rect.height);

            // Ensure we have valid dimensions
            const width = rect.width || window.innerWidth;
            const height = rect.height || (window.innerHeight - 50);

            canvas.width = width;
            canvas.height = height;
            minimapCanvas.width = 200;
            minimapCanvas.height = 150;

            // Draw loading state
            ctx.fillStyle = '#2a2a2a';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            ctx.fillStyle = '#666';
            ctx.font = '16px system-ui, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Loading diagram...', canvas.width / 2, canvas.height / 2);
        }}

        // Use requestAnimationFrame to ensure layout is complete
        requestAnimationFrame(() => {{
            initCanvas();
        }});

        // Load SVG image
        svgImage.onload = () => {{
            svgLoaded = true;
            svgWidth = svgImage.naturalWidth;
            svgHeight = svgImage.naturalHeight;
            console.log('SVG loaded:', svgWidth, 'x', svgHeight);

            // Fit diagram to screen
            fitToScreen();
        }};
        svgImage.onerror = (e) => {{
            console.error('Failed to load SVG:', e);
            ctx.fillStyle = '#2a2a2a';
            ctx.fillRect(0, 0, canvas.width, canvas.height);
            ctx.fillStyle = '#f66';
            ctx.font = '16px system-ui, sans-serif';
            ctx.textAlign = 'center';
            ctx.fillText('Failed to load diagram', canvas.width / 2, canvas.height / 2);
        }};
        svgImage.src = '{}';

        function resizeCanvas() {{
            const rect = container.getBoundingClientRect();
            canvas.width = rect.width;
            canvas.height = rect.height;

            // Set minimap canvas size
            minimapCanvas.width = 200;
            minimapCanvas.height = 150;

            render();
        }}

        function render() {{
            if (!svgLoaded) return;

            // Clear canvas with dark background
            ctx.fillStyle = '#2a2a2a';
            ctx.fillRect(0, 0, canvas.width, canvas.height);

            // Draw white background for diagram area
            ctx.save();
            ctx.translate(offsetX, offsetY);
            ctx.scale(scale, scale);

            // White background with shadow
            ctx.shadowColor = 'rgba(0, 0, 0, 0.4)';
            ctx.shadowBlur = 20;
            ctx.shadowOffsetX = 0;
            ctx.shadowOffsetY = 4;
            ctx.fillStyle = 'white';
            ctx.fillRect(0, 0, svgWidth, svgHeight);

            // Reset shadow and draw SVG
            ctx.shadowColor = 'transparent';
            ctx.drawImage(svgImage, 0, 0);
            ctx.restore();

            // Update zoom level display
            zoomLevelEl.textContent = Math.round(scale * 100) + '%';

            // Update minimap
            updateMinimap();
        }}

        function updateMinimap() {{
            if (!svgLoaded) return;

            // Clear minimap
            minimapCtx.clearRect(0, 0, minimapCanvas.width, minimapCanvas.height);

            // Calculate scale to fit SVG in minimap
            const minimapScale = Math.min(
                minimapCanvas.width / svgWidth,
                minimapCanvas.height / svgHeight
            );

            // Draw SVG in minimap
            minimapCtx.save();
            minimapCtx.scale(minimapScale, minimapScale);
            minimapCtx.drawImage(svgImage, 0, 0);
            minimapCtx.restore();

            // Calculate viewport rectangle
            const viewportWidth = (canvas.width / scale) * minimapScale;
            const viewportHeight = (canvas.height / scale) * minimapScale;
            const viewportX = (-offsetX / scale) * minimapScale;
            const viewportY = (-offsetY / scale) * minimapScale;

            minimapViewport.style.width = Math.max(10, Math.min(viewportWidth, minimapCanvas.width)) + 'px';
            minimapViewport.style.height = Math.max(10, Math.min(viewportHeight, minimapCanvas.height)) + 'px';
            minimapViewport.style.left = Math.max(0, Math.min(viewportX, minimapCanvas.width - 10)) + 'px';
            minimapViewport.style.top = Math.max(0, Math.min(viewportY, minimapCanvas.height - 10)) + 'px';
        }}

        // Convert screen coordinates to diagram coordinates
        function screenToDiagram(screenX, screenY) {{
            const rect = canvas.getBoundingClientRect();
            const canvasX = screenX - rect.left;
            const canvasY = screenY - rect.top;
            return {{
                x: (canvasX - offsetX) / scale,
                y: (canvasY - offsetY) / scale
            }};
        }}

        // Find element at diagram coordinates
        function getElementAtPoint(diagramX, diagramY) {{
            for (const el of elements) {{
                if (diagramX >= el.x && diagramX <= el.x + el.width &&
                    diagramY >= el.y && diagramY <= el.y + el.height) {{
                    return el;
                }}
            }}
            return null;
        }}

        // Show/hide tooltip
        function showTooltip(element, screenX, screenY) {{
            if (element) {{
                let html = `<h4>${{element.name}}</h4><div class="type">${{element.type}}</div>`;
                if (element.description) {{
                    html += `<div class="desc">${{element.description}}</div>`;
                }}
                if (element.technology) {{
                    html += `<div class="tech">${{element.technology}}</div>`;
                }}
                tooltip.innerHTML = html;
                tooltip.style.left = (screenX + 15) + 'px';
                tooltip.style.top = (screenY + 15) + 'px';
                tooltip.style.display = 'block';
            }} else {{
                tooltip.style.display = 'none';
            }}
        }}

        // Zoom functions
        function setZoom(newScale, centerX, centerY) {{
            newScale = Math.max(0.1, Math.min(5, newScale));

            if (centerX !== undefined && centerY !== undefined) {{
                // Zoom toward the specified point
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

        // Mouse wheel zoom
        canvas.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            setZoom(scale * delta, mouseX, mouseY);
        }}, {{ passive: false }});

        // Pan with mouse drag
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
                // Hit-testing for hover tooltips
                const diagramPos = screenToDiagram(e.clientX, e.clientY);
                const element = getElementAtPoint(diagramPos.x, diagramPos.y);

                if (element !== hoveredElement) {{
                    hoveredElement = element;
                    showTooltip(element, e.clientX, e.clientY);
                    canvas.style.cursor = element ? 'pointer' : 'grab';
                }} else if (element) {{
                    // Update tooltip position
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

        // Double-click zoom
        canvas.addEventListener('dblclick', (e) => {{
            const rect = canvas.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;
            setZoom(e.shiftKey ? scale / 2 : scale * 2, mouseX, mouseY);
        }});

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {{
            if (e.key === '+' || e.key === '=') zoomIn();
            if (e.key === '-') zoomOut();
            if (e.key === '0') resetZoom();
            if (e.key === 'f' || e.key === 'F') fitToScreen();
        }});

        // Minimap interaction
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

            // Convert minimap click to diagram position
            const diagramX = clickX / minimapScale;
            const diagramY = clickY / minimapScale;

            // Center viewport on clicked position
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

        // Handle window resize
        window.addEventListener('resize', () => {{
            resizeCanvas();
        }});

        // Initialize minimap viewport cursor
        minimapViewport.style.cursor = 'grab';
    </script>
</body>
</html>"##,
        view_key,
        view_key,
        view_key,
        svg_url,
        view_key,
        elements_json,
        svg_url
    );

    Ok(Html(html))
}

/// Escape special characters for JSON strings.
fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Documentation viewer page.
pub async fn documentation(
    State(state): State<AppState>,
) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let docs = &workspace.documentation;

    // Build sections list
    let sections_html: String = if docs.sections.is_empty() {
        "<p class=\"empty\">No documentation sections available.</p>".to_string()
    } else {
        docs.sections.iter().enumerate().map(|(i, section)| {
            let default_title = format!("Section {}", i + 1);
            let title = section.title.as_deref().unwrap_or(&default_title);
            format!(
                r#"<div class="doc-section" id="section-{}">
                    <h2>{}</h2>
                    <div class="content">{}</div>
                </div>"#,
                i,
                escape_html(title),
                render_markdown(&section.content)
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

    // Build sidebar
    let sidebar_sections: String = docs.sections.iter().enumerate().map(|(i, section)| {
        let default_title = format!("Section {}", i + 1);
        let title = section.title.as_deref().unwrap_or(&default_title);
        format!(r##"<a href="#section-{}">{}</a>"##, i, escape_html(title))
    }).collect();

    let sidebar_decisions: String = docs.decisions.iter().map(|decision| {
        format!(r##"<a href="#adr-{}">{}: {}</a>"##, decision.id, decision.id, escape_html(&decision.title))
    }).collect();

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Documentation - {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; }}
        .header {{ background: #333; color: white; padding: 15px 20px; display: flex; align-items: center; gap: 20px; }}
        .header a {{ color: white; text-decoration: none; }}
        .header h1 {{ margin: 0; font-size: 18px; }}
        .container {{ display: flex; min-height: calc(100vh - 54px); }}
        .sidebar {{ width: 280px; background: white; border-right: 1px solid #ddd; padding: 20px; overflow-y: auto; }}
        .sidebar h3 {{ margin: 0 0 10px 0; font-size: 12px; text-transform: uppercase; color: #888; }}
        .sidebar a {{ display: block; padding: 8px 12px; color: #333; text-decoration: none; border-radius: 4px; margin-bottom: 2px; font-size: 14px; }}
        .sidebar a:hover {{ background: #f0f0f0; }}
        .main {{ flex: 1; padding: 40px; max-width: 900px; }}
        .doc-section {{ background: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .doc-section h2 {{ margin-top: 0; border-bottom: 1px solid #eee; padding-bottom: 10px; }}
        .content {{ line-height: 1.7; }}
        .content h1, .content h2, .content h3 {{ margin-top: 1.5em; }}
        .content pre {{ background: #f5f5f5; padding: 15px; border-radius: 4px; overflow-x: auto; }}
        .content code {{ background: #f0f0f0; padding: 2px 6px; border-radius: 3px; font-family: "SF Mono", Monaco, monospace; font-size: 0.9em; }}
        .content pre code {{ background: none; padding: 0; }}
        .content blockquote {{ border-left: 4px solid #ddd; margin: 0; padding-left: 20px; color: #666; }}
        .content table {{ border-collapse: collapse; width: 100%; margin: 1em 0; }}
        .content th, .content td {{ border: 1px solid #ddd; padding: 10px; text-align: left; }}
        .content th {{ background: #f5f5f5; }}
        .decisions-section {{ margin-top: 40px; }}
        .decision {{ background: white; padding: 30px; border-radius: 8px; margin-bottom: 20px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .decision-header {{ display: flex; align-items: center; gap: 15px; margin-bottom: 20px; flex-wrap: wrap; }}
        .decision-id {{ font-family: monospace; background: #f0f0f0; padding: 4px 8px; border-radius: 4px; font-size: 12px; }}
        .decision-header h3 {{ margin: 0; flex: 1; }}
        .status {{ padding: 4px 10px; border-radius: 20px; font-size: 11px; text-transform: uppercase; font-weight: 600; }}
        .status.accepted {{ background: #d4edda; color: #155724; }}
        .status.proposed {{ background: #fff3cd; color: #856404; }}
        .status.superseded {{ background: #e2e3e5; color: #383d41; }}
        .status.deprecated {{ background: #f8d7da; color: #721c24; }}
        .status.rejected {{ background: #f8d7da; color: #721c24; }}
        .date {{ color: #888; font-size: 12px; }}
        .empty {{ color: #888; font-style: italic; }}
    </style>
</head>
<body>
    <div class="header">
        <a href="/">← Back</a>
        <h1>Documentation: {}</h1>
    </div>
    <div class="container">
        <div class="sidebar">
            <h3>Sections</h3>
            {}
            {}
        </div>
        <div class="main">
            {}
            {}
        </div>
    </div>
</body>
</html>"##,
        workspace.name,
        workspace.name,
        sidebar_sections,
        if !docs.decisions.is_empty() { format!("<h3 style=\"margin-top: 20px;\">ADRs</h3>{}", sidebar_decisions) } else { String::new() },
        sections_html,
        decisions_html
    );

    Ok(Html(html))
}

/// Simple Markdown to HTML renderer.
fn render_markdown(md: &str) -> String {
    let mut html = String::new();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in md.lines() {
        // Code blocks
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>\n");
                in_code_block = false;
            } else {
                let code_lang = line[3..].trim();
                html.push_str(&format!("<pre><code class=\"language-{}\">", code_lang));
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&escape_html(line));
            html.push('\n');
            continue;
        }

        // Close list if needed
        if in_list && !line.starts_with("- ") && !line.starts_with("* ") && !line.starts_with("1.") {
            html.push_str("</ul>\n");
            in_list = false;
        }

        // Headers
        if line.starts_with("### ") {
            html.push_str(&format!("<h3>{}</h3>\n", escape_html(&line[4..])));
        } else if line.starts_with("## ") {
            html.push_str(&format!("<h2>{}</h2>\n", escape_html(&line[3..])));
        } else if line.starts_with("# ") {
            html.push_str(&format!("<h1>{}</h1>\n", escape_html(&line[2..])));
        }
        // Lists
        else if line.starts_with("- ") || line.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>\n");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>\n", render_inline(&line[2..])));
        }
        // Blockquotes
        else if line.starts_with("> ") {
            html.push_str(&format!("<blockquote>{}</blockquote>\n", render_inline(&line[2..])));
        }
        // Horizontal rule
        else if line == "---" || line == "***" || line == "___" {
            html.push_str("<hr>\n");
        }
        // Empty line
        else if line.trim().is_empty() {
            html.push_str("<br>\n");
        }
        // Paragraph
        else {
            html.push_str(&format!("<p>{}</p>\n", render_inline(line)));
        }
    }

    if in_list {
        html.push_str("</ul>\n");
    }
    if in_code_block {
        html.push_str("</code></pre>\n");
    }

    html
}

/// Render inline Markdown elements.
fn render_inline(text: &str) -> String {
    let mut result = escape_html(text);

    // Bold
    let bold_re = regex_lite::Regex::new(r"\*\*(.+?)\*\*").unwrap();
    result = bold_re.replace_all(&result, "<strong>$1</strong>").to_string();

    // Italic
    let italic_re = regex_lite::Regex::new(r"\*(.+?)\*").unwrap();
    result = italic_re.replace_all(&result, "<em>$1</em>").to_string();

    // Inline code
    let code_re = regex_lite::Regex::new(r"`(.+?)`").unwrap();
    result = code_re.replace_all(&result, "<code>$1</code>").to_string();

    // Links
    let link_re = regex_lite::Regex::new(r"\[(.+?)\]\((.+?)\)").unwrap();
    result = link_re.replace_all(&result, "<a href=\"$2\">$1</a>").to_string();

    result
}

/// Escape HTML special characters.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Helper function to check if an element should be visible for a given perspective.
/// Elements with no perspectives specified are visible in all perspectives.
/// Elements with perspectives specified are only visible when the requested perspective matches.
fn element_matches_perspective(element_perspectives: &[String], requested_perspective: Option<&str>) -> bool {
    match requested_perspective {
        None => true, // No perspective filter, show all elements
        Some(perspective) => {
            // If element has no perspectives, it's visible in all perspectives
            if element_perspectives.is_empty() {
                true
            } else {
                // Element has perspectives, check if requested perspective is in the list
                element_perspectives.iter().any(|p| p == perspective)
            }
        }
    }
}

/// Filter workspace by perspective.
/// Returns a cloned workspace with only elements that match the perspective.
fn filter_workspace_by_perspective(workspace: &Workspace, perspective: Option<&str>) -> Workspace {
    if perspective.is_none() {
        // No filtering needed
        return workspace.clone();
    }

    let mut filtered = workspace.clone();

    // Filter people
    filtered.model.people.retain(|p| {
        element_matches_perspective(&p.properties.perspectives, perspective)
    });

    // Filter software systems and their containers/components
    filtered.model.software_systems.retain_mut(|sys| {
        // First filter containers
        sys.containers.retain_mut(|container| {
            // Filter components
            container.components.retain(|comp| {
                element_matches_perspective(&comp.properties.perspectives, perspective)
            });

            element_matches_perspective(&container.properties.perspectives, perspective)
        });

        element_matches_perspective(&sys.properties.perspectives, perspective)
    });

    // Filter deployment nodes recursively
    fn filter_deployment_nodes(
        nodes: &mut Vec<structurizr_core::DeploymentNode>,
        perspective: Option<&str>,
    ) {
        nodes.retain_mut(|node| {
            // Filter children recursively
            filter_deployment_nodes(&mut node.children, perspective);

            // Filter infrastructure nodes
            node.infrastructure_nodes.retain(|infra| {
                element_matches_perspective(&infra.properties.perspectives, perspective)
            });

            element_matches_perspective(&node.properties.perspectives, perspective)
        });
    }

    filter_deployment_nodes(&mut filtered.model.deployment_nodes, perspective);

    // Filter relationships - keep only those where both source and destination still exist
    let all_element_ids: std::collections::HashSet<_> = {
        let mut ids = std::collections::HashSet::new();

        for person in &filtered.model.people {
            ids.insert(person.id());
        }

        for system in &filtered.model.software_systems {
            ids.insert(system.id());
            for container in &system.containers {
                ids.insert(container.id());
                for component in &container.components {
                    ids.insert(component.id());
                }
            }
        }

        fn collect_deployment_ids(
            nodes: &[structurizr_core::DeploymentNode],
            ids: &mut std::collections::HashSet<structurizr_core::ElementId>,
        ) {
            for node in nodes {
                ids.insert(node.id());
                collect_deployment_ids(&node.children, ids);
                for infra in &node.infrastructure_nodes {
                    ids.insert(infra.id());
                }
            }
        }

        collect_deployment_ids(&filtered.model.deployment_nodes, &mut ids);

        ids
    };

    filtered.model.relationships.retain(|rel| {
        all_element_ids.contains(&rel.source_id) && all_element_ids.contains(&rel.destination_id)
    });

    filtered
}

/// Perspective query parameters.
#[derive(Debug, serde::Deserialize)]
pub struct PerspectiveQuery {
    pub perspective: Option<String>,
}

/// Render view as SVG.
pub async fn render_svg(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
    axum::extract::Query(query): axum::extract::Query<PerspectiveQuery>,
) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    // Apply perspective filter if requested
    let filtered_workspace = filter_workspace_by_perspective(
        &workspace,
        query.perspective.as_deref()
    );

    let renderer = SvgRenderer::default();

    // Try to find the view by key
    let svg = if let Some(view) = filtered_workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_system_landscape(&filtered_workspace, view)?
    } else if let Some(view) = filtered_workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_system_context(&filtered_workspace, view)?
    } else if let Some(view) = filtered_workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        renderer.render_container(&filtered_workspace, view)?
    } else {
        // Default: render a system landscape view
        let view = SystemLandscapeView::new(&view_key);
        renderer.render_system_landscape(&filtered_workspace, &view)?
    };

    Ok((
        [(header::CONTENT_TYPE, "image/svg+xml")],
        svg
    ))
}

/// Export view as PlantUML.
pub async fn export_plantuml(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let puml = if let Some(view) = workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_system_landscape(&workspace, view)?
    } else if let Some(view) = workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_system_context(&workspace, view)?
    } else if let Some(view) = workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_container(&workspace, view)?
    } else if let Some(view) = workspace.views().component_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_component(&workspace, view)?
    } else if let Some(view) = workspace.views().dynamic_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_dynamic(&workspace, view)?
    } else if let Some(view) = workspace.views().deployment_views.iter().find(|v| v.properties.key == view_key) {
        PlantUmlExporter::export_deployment(&workspace, view)?
    } else {
        PlantUmlExporter::export_flowchart(&workspace)?
    };

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        puml
    ))
}

/// Export view as Mermaid.
pub async fn export_mermaid(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let mermaid = if let Some(view) = workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_system_landscape(&workspace, view)?
    } else if let Some(view) = workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_system_context(&workspace, view)?
    } else if let Some(view) = workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_container(&workspace, view)?
    } else if let Some(view) = workspace.views().component_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_component(&workspace, view)?
    } else if let Some(view) = workspace.views().dynamic_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_dynamic(&workspace, view)?
    } else if let Some(view) = workspace.views().deployment_views.iter().find(|v| v.properties.key == view_key) {
        MermaidExporter::export_deployment(&workspace, view)?
    } else {
        MermaidExporter::export_flowchart(&workspace)?
    };

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        mermaid
    ))
}

/// Export view as DOT/Graphviz.
pub async fn export_dot(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let dot = if let Some(view) = workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        DotExporter::export_system_landscape(&workspace, view)?
    } else if let Some(view) = workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        DotExporter::export_system_context(&workspace, view)?
    } else if let Some(view) = workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        DotExporter::export_container(&workspace, view)?
    } else if let Some(view) = workspace.views().component_views.iter().find(|v| v.properties.key == view_key) {
        DotExporter::export_component(&workspace, view)?
    } else {
        DotExporter::export_flowchart(&workspace)?
    };

    Ok((
        [(header::CONTENT_TYPE, "text/vnd.graphviz; charset=utf-8")],
        dot
    ))
}

/// Export view as D2.
pub async fn export_d2(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<impl IntoResponse> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let d2 = if let Some(view) = workspace.views().system_landscape_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_system_landscape(&workspace, view)?
    } else if let Some(view) = workspace.views().system_context_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_system_context(&workspace, view)?
    } else if let Some(view) = workspace.views().container_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_container(&workspace, view)?
    } else if let Some(view) = workspace.views().component_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_component(&workspace, view)?
    } else if let Some(view) = workspace.views().dynamic_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_dynamic(&workspace, view)?
    } else if let Some(view) = workspace.views().deployment_views.iter().find(|v| v.properties.key == view_key) {
        D2Exporter::export_deployment(&workspace, view)?
    } else {
        D2Exporter::export_flowchart(&workspace)?
    };

    Ok((
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        d2
    ))
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

/// Presentation mode handler - full-screen slideshow of diagrams.
pub async fn presentation_mode(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<PresentationQuery>,
) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    // Get view keys - either from query params or all views
    let view_keys: Vec<String> = if let Some(views_param) = query.views {
        views_param.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        workspace.views().all_keys().iter().map(|k| k.to_string()).collect()
    };

    if view_keys.is_empty() {
        return Ok(Html(
            r#"<!DOCTYPE html>
<html>
<head><title>Presentation Mode - No Views</title></head>
<body style="background: #000; color: #fff; display: flex; align-items: center; justify-content: center; height: 100vh; font-family: sans-serif;">
    <div style="text-align: center;">
        <h1>No Views Available</h1>
        <p>Create some views in your workspace to use presentation mode.</p>
        <p><a href="/" style="color: #0066cc;">← Back to Home</a></p>
    </div>
</body>
</html>"#.to_string()
        ));
    }

    // Build slides data - for each view, get its SVG URL and title
    let slides_json: String = view_keys.iter().enumerate().map(|(i, key)| {
        let title = escape_json(key);
        let svg_url = format!("/view/{}/svg", key);
        if i > 0 {
            format!(r#",{{"title":"{}","svg":"{}"}}"#, title, svg_url)
        } else {
            format!(r#"{{"title":"{}","svg":"{}"}}"#, title, svg_url)
        }
    }).collect();

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Presentation Mode - {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            background: #000;
            color: #fff;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            overflow: hidden;
            height: 100vh;
            width: 100vw;
        }}

        .presentation-container {{
            position: relative;
            height: 100vh;
            width: 100vw;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
        }}

        .slide {{
            display: none;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100%;
            width: 100%;
            padding: 40px;
            opacity: 0;
            transition: opacity 0.3s ease-in-out;
        }}

        .slide.active {{
            display: flex;
            opacity: 1;
        }}

        .slide-content {{
            max-width: 90%;
            max-height: 85%;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
        }}

        .slide-content img {{
            max-width: 100%;
            max-height: 100%;
            object-fit: contain;
            background: #fff;
            box-shadow: 0 10px 50px rgba(0,0,0,0.5);
            border-radius: 4px;
        }}

        .slide-title {{
            margin-top: 30px;
            font-size: 24px;
            font-weight: 500;
            color: #ccc;
            text-align: center;
        }}

        .slide-counter {{
            position: fixed;
            bottom: 30px;
            right: 30px;
            font-size: 18px;
            color: #666;
            background: rgba(255,255,255,0.1);
            padding: 10px 20px;
            border-radius: 20px;
            backdrop-filter: blur(10px);
        }}

        .controls {{
            position: fixed;
            bottom: 30px;
            left: 30px;
            display: flex;
            gap: 10px;
            opacity: 0;
            transition: opacity 0.3s ease;
        }}

        body:hover .controls {{
            opacity: 1;
        }}

        .control-btn {{
            background: rgba(255,255,255,0.1);
            border: 1px solid rgba(255,255,255,0.2);
            color: #fff;
            padding: 10px 20px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            backdrop-filter: blur(10px);
            transition: all 0.2s ease;
        }}

        .control-btn:hover {{
            background: rgba(255,255,255,0.2);
            border-color: rgba(255,255,255,0.3);
        }}

        .control-btn:active {{
            transform: scale(0.95);
        }}

        .help-overlay {{
            position: fixed;
            top: 30px;
            left: 50%;
            transform: translateX(-50%);
            background: rgba(0,0,0,0.8);
            color: #fff;
            padding: 20px 30px;
            border-radius: 8px;
            backdrop-filter: blur(10px);
            opacity: 0;
            transition: opacity 0.3s ease;
            pointer-events: none;
            z-index: 1000;
        }}

        .help-overlay.show {{
            opacity: 1;
        }}

        .help-overlay h3 {{
            margin-bottom: 15px;
            font-size: 16px;
        }}

        .help-overlay ul {{
            list-style: none;
            padding: 0;
        }}

        .help-overlay li {{
            margin: 8px 0;
            font-size: 14px;
            color: #ccc;
        }}

        .help-overlay kbd {{
            background: rgba(255,255,255,0.1);
            padding: 3px 8px;
            border-radius: 3px;
            font-family: monospace;
            font-size: 12px;
            border: 1px solid rgba(255,255,255,0.2);
        }}

        .loading {{
            position: fixed;
            top: 50%;
            left: 50%;
            transform: translate(-50%, -50%);
            font-size: 20px;
            color: #666;
        }}

        .exit-button {{
            position: fixed;
            top: 30px;
            right: 30px;
            background: rgba(255,255,255,0.1);
            border: 1px solid rgba(255,255,255,0.2);
            color: #fff;
            padding: 10px 20px;
            border-radius: 6px;
            cursor: pointer;
            font-size: 14px;
            backdrop-filter: blur(10px);
            transition: all 0.2s ease;
            opacity: 0;
        }}

        body:hover .exit-button {{
            opacity: 1;
        }}

        .exit-button:hover {{
            background: rgba(255,255,255,0.2);
            border-color: rgba(255,255,255,0.3);
        }}
    </style>
</head>
<body>
    <div class="presentation-container">
        <div class="loading" id="loading">Loading presentation...</div>
        <div id="slides-container"></div>
        <div class="slide-counter" id="counter">1 / 1</div>
        <button class="exit-button" onclick="exitPresentation()">Exit (Esc)</button>
    </div>

    <div class="controls">
        <button class="control-btn" onclick="prevSlide()">← Previous</button>
        <button class="control-btn" onclick="nextSlide()">Next →</button>
        <button class="control-btn" onclick="toggleHelp()">Help (?)</button>
        <button class="control-btn" onclick="toggleFullscreen()">Fullscreen (F)</button>
    </div>

    <div class="help-overlay" id="help">
        <h3>Keyboard Shortcuts</h3>
        <ul>
            <li><kbd>→</kbd> <kbd>Space</kbd> <kbd>Enter</kbd> - Next slide</li>
            <li><kbd>←</kbd> <kbd>Backspace</kbd> - Previous slide</li>
            <li><kbd>Home</kbd> - First slide</li>
            <li><kbd>End</kbd> - Last slide</li>
            <li><kbd>F</kbd> - Toggle fullscreen</li>
            <li><kbd>?</kbd> - Toggle this help</li>
            <li><kbd>Esc</kbd> - Exit presentation</li>
        </ul>
    </div>

    <script>
        const slides = [{}];
        let currentSlide = 0;
        let imagesLoaded = 0;
        let helpVisible = false;

        // Preload all images
        function preloadImages() {{
            const promises = slides.map((slide, index) => {{
                return new Promise((resolve, reject) => {{
                    const img = new Image();
                    img.onload = () => {{
                        imagesLoaded++;
                        updateLoadingProgress();
                        resolve({{ index, img }});
                    }};
                    img.onerror = () => reject(new Error(`Failed to load slide ${{index + 1}}`));
                    img.src = slide.svg;
                }});
            }});

            Promise.all(promises)
                .then(loadedImages => {{
                    renderSlides(loadedImages);
                    document.getElementById('loading').style.display = 'none';
                    showSlide(0);
                }})
                .catch(err => {{
                    console.error('Error loading images:', err);
                    document.getElementById('loading').textContent = 'Error loading slides';
                }});
        }}

        function updateLoadingProgress() {{
            const progress = Math.round((imagesLoaded / slides.length) * 100);
            document.getElementById('loading').textContent = `Loading presentation... ${{progress}}%`;
        }}

        function renderSlides(loadedImages) {{
            const container = document.getElementById('slides-container');
            loadedImages.forEach(({{ index, img }}) => {{
                const slideDiv = document.createElement('div');
                slideDiv.className = 'slide';
                slideDiv.id = `slide-${{index}}`;

                const content = document.createElement('div');
                content.className = 'slide-content';

                const imgClone = img.cloneNode();
                content.appendChild(imgClone);

                const title = document.createElement('div');
                title.className = 'slide-title';
                title.textContent = slides[index].title;
                content.appendChild(title);

                slideDiv.appendChild(content);
                container.appendChild(slideDiv);
            }});
        }}

        function showSlide(index) {{
            // Hide all slides
            document.querySelectorAll('.slide').forEach(slide => {{
                slide.classList.remove('active');
            }});

            // Show current slide
            const slide = document.getElementById(`slide-${{index}}`);
            if (slide) {{
                slide.classList.add('active');
                currentSlide = index;
                updateCounter();
            }}
        }}

        function updateCounter() {{
            const counter = document.getElementById('counter');
            counter.textContent = `${{currentSlide + 1}} / ${{slides.length}}`;
        }}

        function nextSlide() {{
            if (currentSlide < slides.length - 1) {{
                showSlide(currentSlide + 1);
            }}
        }}

        function prevSlide() {{
            if (currentSlide > 0) {{
                showSlide(currentSlide - 1);
            }}
        }}

        function firstSlide() {{
            showSlide(0);
        }}

        function lastSlide() {{
            showSlide(slides.length - 1);
        }}

        function toggleFullscreen() {{
            if (!document.fullscreenElement) {{
                document.documentElement.requestFullscreen().catch(err => {{
                    console.error('Error entering fullscreen:', err);
                }});
            }} else {{
                if (document.exitFullscreen) {{
                    document.exitFullscreen();
                }}
            }}
        }}

        function toggleHelp() {{
            helpVisible = !helpVisible;
            const helpOverlay = document.getElementById('help');
            if (helpVisible) {{
                helpOverlay.classList.add('show');
            }} else {{
                helpOverlay.classList.remove('show');
            }}
        }}

        function exitPresentation() {{
            window.location.href = '/';
        }}

        // Keyboard navigation
        document.addEventListener('keydown', (e) => {{
            switch(e.key) {{
                case 'ArrowRight':
                case ' ':
                case 'Enter':
                    e.preventDefault();
                    nextSlide();
                    break;
                case 'ArrowLeft':
                case 'Backspace':
                    e.preventDefault();
                    prevSlide();
                    break;
                case 'Home':
                    e.preventDefault();
                    firstSlide();
                    break;
                case 'End':
                    e.preventDefault();
                    lastSlide();
                    break;
                case 'f':
                case 'F':
                    e.preventDefault();
                    toggleFullscreen();
                    break;
                case '?':
                    e.preventDefault();
                    toggleHelp();
                    break;
                case 'Escape':
                    e.preventDefault();
                    if (document.fullscreenElement) {{
                        document.exitFullscreen();
                    }} else {{
                        exitPresentation();
                    }}
                    break;
            }}
        }});

        // Auto-hide help after 5 seconds on first show
        let helpTimeout;
        function toggleHelp() {{
            helpVisible = !helpVisible;
            const helpOverlay = document.getElementById('help');
            if (helpVisible) {{
                helpOverlay.classList.add('show');
                clearTimeout(helpTimeout);
                helpTimeout = setTimeout(() => {{
                    helpVisible = false;
                    helpOverlay.classList.remove('show');
                }}, 5000);
            }} else {{
                helpOverlay.classList.remove('show');
            }}
        }}

        // Show help briefly on load
        window.addEventListener('load', () => {{
            setTimeout(() => {{
                toggleHelp();
            }}, 500);
        }});

        // Initialize
        preloadImages();
    </script>
</body>
</html>"##,
        workspace.name,
        slides_json
    );

    Ok(Html(html))
}

/// Interactive diagram editor page.
pub async fn edit_diagram(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<Html<String>> {
    let _workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let ws_host = state.config.address();
    let svg_url = format!("/view/{}/svg", view_key);

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Edit {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #1a1a1a; color: #fff; height: 100vh; overflow: hidden; }}
        .toolbar {{ background: #333; padding: 10px 20px; display: flex; align-items: center; gap: 20px; border-bottom: 1px solid #444; }}
        .toolbar a {{ color: #fff; text-decoration: none; }}
        .toolbar button {{ background: #0066cc; color: #fff; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; }}
        .toolbar button:hover {{ background: #0052a3; }}
        .toolbar button.secondary {{ background: #555; }}
        .toolbar button.secondary:hover {{ background: #666; }}
        .editor-container {{ display: flex; height: calc(100vh - 50px); }}
        .canvas-container {{ flex: 1; overflow: auto; position: relative; background: #2a2a2a; }}
        #canvas {{
            position: absolute;
            cursor: grab;
            transform-origin: 0 0;
        }}
        #canvas.dragging {{ cursor: grabbing; }}
        .element {{
            position: absolute;
            cursor: move;
            user-select: none;
        }}
        .element:hover {{ filter: brightness(1.1); }}
        .element.selected {{ outline: 3px solid #0066cc; outline-offset: 2px; }}
        .status {{ position: fixed; bottom: 20px; right: 20px; background: #333; padding: 10px 20px; border-radius: 4px; }}
        .status.connected {{ background: #28a745; }}
        .status.disconnected {{ background: #dc3545; }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="/">← Back</a>
        <span id="view-name">{}</span>
        <button onclick="autoLayout()">Auto Layout</button>
        <button onclick="save()" class="secondary">Save</button>
        <button onclick="undo()" class="secondary">Undo</button>
        <button onclick="redo()" class="secondary">Redo</button>
        <span style="margin-left: auto; font-size: 12px; color: #888;">
            Drag elements to reposition • Click to select
        </span>
    </div>
    <div class="editor-container">
        <div class="canvas-container" id="canvas-container">
            <div id="canvas">
                <img src="{}" alt="{}" id="diagram-svg" style="pointer-events: none;">
            </div>
        </div>
    </div>
    <div class="status disconnected" id="status">Connecting...</div>

    <script>
        const viewKey = '{}';
        const wsUrl = 'ws://{}/ws/edit/' + viewKey;
        let ws = null;
        let selectedElements = [];
        let isDragging = false;
        let dragStart = {{ x: 0, y: 0 }};
        let elementStart = {{ x: 0, y: 0 }};
        let currentElement = null;

        // Connect to WebSocket
        function connect() {{
            ws = new WebSocket(wsUrl);

            ws.onopen = () => {{
                console.log('WebSocket connected');
                document.getElementById('status').className = 'status connected';
                document.getElementById('status').textContent = 'Connected';

                // Request initial state
                ws.send(JSON.stringify({{ type: 'request_state', view_key: viewKey }}));
            }};

            ws.onclose = () => {{
                console.log('WebSocket disconnected');
                document.getElementById('status').className = 'status disconnected';
                document.getElementById('status').textContent = 'Disconnected';
                // Reconnect after 2 seconds
                setTimeout(connect, 2000);
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
                    // Initial state received
                    console.log('State received with', message.elements.length, 'elements');
                    break;
                case 'element_moved':
                    // Another client moved an element
                    console.log('Element', message.element_id, 'moved to', message.x, message.y);
                    break;
                case 'error':
                    alert('Error: ' + message.message);
                    break;
            }}
        }}

        function save() {{
            if (ws && ws.readyState === WebSocket.OPEN) {{
                ws.send(JSON.stringify({{ type: 'save' }}));
            }}
        }}

        function autoLayout() {{
            if (ws && ws.readyState === WebSocket.OPEN) {{
                ws.send(JSON.stringify({{ type: 'auto_layout', view_key: viewKey }}));
            }}
        }}

        function undo() {{
            if (ws && ws.readyState === WebSocket.OPEN) {{
                ws.send(JSON.stringify({{ type: 'undo', view_key: viewKey }}));
            }}
        }}

        function redo() {{
            if (ws && ws.readyState === WebSocket.OPEN) {{
                ws.send(JSON.stringify({{ type: 'redo', view_key: viewKey }}));
            }}
        }}

        // Pan and zoom setup
        let scale = 1;
        let translateX = 0;
        let translateY = 0;
        let isPanning = false;
        let panStart = {{ x: 0, y: 0 }};

        const canvasContainer = document.getElementById('canvas-container');
        const canvas = document.getElementById('canvas');

        canvasContainer.addEventListener('wheel', (e) => {{
            e.preventDefault();
            const delta = e.deltaY > 0 ? 0.9 : 1.1;
            const newScale = Math.max(0.1, Math.min(5, scale * delta));

            const rect = canvasContainer.getBoundingClientRect();
            const mouseX = e.clientX - rect.left;
            const mouseY = e.clientY - rect.top;

            translateX = mouseX - (mouseX - translateX) * (newScale / scale);
            translateY = mouseY - (mouseY - translateY) * (newScale / scale);
            scale = newScale;

            updateCanvasTransform();
        }});

        canvasContainer.addEventListener('mousedown', (e) => {{
            if (e.button === 1 || (e.button === 0 && e.shiftKey)) {{
                isPanning = true;
                panStart = {{ x: e.clientX - translateX, y: e.clientY - translateY }};
                canvas.classList.add('dragging');
            }}
        }});

        document.addEventListener('mousemove', (e) => {{
            if (isPanning) {{
                translateX = e.clientX - panStart.x;
                translateY = e.clientY - panStart.y;
                updateCanvasTransform();
            }}
        }});

        document.addEventListener('mouseup', () => {{
            isPanning = false;
            canvas.classList.remove('dragging');
        }});

        function updateCanvasTransform() {{
            canvas.style.transform = `translate(${{translateX}}px, ${{translateY}}px) scale(${{scale}})`;
        }}

        // Initialize
        connect();
    </script>
</body>
</html>"##,
        view_key,
        view_key,
        svg_url,
        view_key,
        view_key,
        ws_host
    );

    Ok(Html(html))
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

/// Search page.
pub async fn search_page(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let search_term = query.q.unwrap_or_default();
    let results = perform_search(&workspace, &search_term);

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

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Search - {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; }}
        .header {{ background: #333; color: white; padding: 15px 20px; display: flex; align-items: center; gap: 20px; }}
        .header a {{ color: white; text-decoration: none; }}
        .header h1 {{ margin: 0; font-size: 18px; }}
        .search-container {{ max-width: 800px; margin: 40px auto; padding: 0 20px; }}
        .search-box {{ display: flex; gap: 10px; margin-bottom: 30px; }}
        .search-box input {{ flex: 1; padding: 12px 16px; font-size: 16px; border: 2px solid #ddd; border-radius: 8px; }}
        .search-box input:focus {{ outline: none; border-color: #0066cc; }}
        .search-box button {{ background: #0066cc; color: white; border: none; padding: 12px 24px; border-radius: 8px; cursor: pointer; font-size: 16px; }}
        .search-box button:hover {{ background: #0052a3; }}
        .result {{ background: white; padding: 20px; border-radius: 8px; margin-bottom: 15px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .result-header {{ display: flex; align-items: center; gap: 10px; }}
        .result-header h3 {{ margin: 0; }}
        .type {{ background: #e8e8e8; padding: 4px 10px; border-radius: 4px; font-size: 11px; text-transform: uppercase; font-weight: 600; }}
        .desc {{ color: #666; margin: 10px 0 0 0; }}
        .no-results {{ color: #888; font-style: italic; text-align: center; padding: 40px; }}
        .result-count {{ color: #666; margin-bottom: 20px; }}
    </style>
</head>
<body>
    <div class="header">
        <a href="/">← Back</a>
        <h1>Search</h1>
    </div>
    <div class="search-container">
        <form class="search-box" method="get">
            <input type="text" name="q" placeholder="Search elements, relationships, documentation..." value="{}" autofocus>
            <button type="submit">Search</button>
        </form>
        {}
        <div class="results">
            {}
        </div>
    </div>
</body>
</html>"##,
        workspace.name,
        escape_html(&search_term),
        if !search_term.is_empty() { format!("<p class=\"result-count\">{} results for \"{}\"</p>", results.len(), escape_html(&search_term)) } else { String::new() },
        results_html
    );

    Ok(Html(html))
}

/// Search API endpoint.
pub async fn search_api(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<SearchQuery>,
) -> Result<Json<Vec<SearchResult>>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    let search_term = query.q.unwrap_or_default();
    let results = perform_search(&workspace, &search_term);

    Ok(Json(results))
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

/// Force-directed graph exploration view.
pub async fn explore_view(State(state): State<AppState>) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

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

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Explore - {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #1a1a1a; color: #fff; height: 100vh; overflow: hidden; }}
        .toolbar {{ background: #333; padding: 10px 20px; display: flex; align-items: center; gap: 20px; border-bottom: 1px solid #444; }}
        .toolbar a {{ color: #fff; text-decoration: none; }}
        .toolbar a:hover {{ text-decoration: underline; }}
        .toolbar button {{ background: #555; color: #fff; border: none; padding: 8px 16px; border-radius: 4px; cursor: pointer; }}
        .toolbar button:hover {{ background: #666; }}
        .toolbar .separator {{ border-left: 1px solid #555; height: 20px; }}
        .canvas-container {{ height: calc(100vh - 50px); position: relative; }}
        svg {{ width: 100%; height: 100%; cursor: grab; }}
        svg.dragging {{ cursor: grabbing; }}
        .node {{ cursor: pointer; }}
        .node circle {{ transition: r 0.2s, fill 0.2s; }}
        .node:hover circle {{ r: 35; }}
        .node text {{ pointer-events: none; user-select: none; fill: #fff; }}
        .link {{ stroke: #666; stroke-width: 1.5; fill: none; }}
        .link-label {{ fill: #999; font-size: 10px; pointer-events: none; user-select: none; }}
        .tooltip {{ position: fixed; background: #333; color: white; padding: 12px 16px; border-radius: 6px; font-size: 13px; max-width: 300px; z-index: 1000; pointer-events: none; box-shadow: 0 4px 12px rgba(0,0,0,0.5); display: none; }}
        .tooltip h4 {{ margin: 0 0 6px 0; font-size: 14px; }}
        .tooltip .type {{ color: #888; font-size: 11px; margin-bottom: 8px; }}
        .tooltip .desc {{ line-height: 1.4; }}
        .tooltip .tech {{ color: #6af; margin-top: 6px; font-size: 12px; }}
        .info {{ position: fixed; bottom: 20px; left: 20px; background: #333; padding: 12px 16px; border-radius: 6px; font-size: 12px; color: #999; }}
        .controls {{ display: flex; gap: 10px; align-items: center; }}
        .controls label {{ font-size: 12px; color: #ccc; display: flex; align-items: center; gap: 5px; }}
        .controls input[type="range"] {{ width: 100px; }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="/">← Back</a>
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
        <div style="margin-top: 4px; font-size: 11px; color: #666;">Drag nodes • Scroll to zoom • Drag canvas to pan</div>
    </div>

    <script>
        const nodes = {{}};
        const links = {{}};

        // Create SVG and groups
        const svg = document.getElementById('canvas');
        const width = window.innerWidth;
        const height = window.innerHeight - 50;

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
            nodes: {},
            links: {}
        }};

        // Create node elements
        data.nodes.forEach(node => {{
            node.x = width / 2 + (Math.random() - 0.5) * 200;
            node.y = height / 2 + (Math.random() - 0.5) * 200;
            node.vx = 0;
            node.vy = 0;

            const g = document.createElementNS('http://www.w3.org/2000/svg', 'g');
            g.setAttribute('class', 'node');
            g.setAttribute('data-id', node.id);

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

            g.appendChild(circle);
            g.appendChild(text);
            nodeGroup.appendChild(g);

            node.element = g;
            nodes[node.id] = node;

            // Drag handlers
            let isDragging = false;
            let dragStart = {{ x: 0, y: 0 }};

            g.addEventListener('mousedown', (e) => {{
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
            g.addEventListener('mouseenter', (e) => {{
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

            g.addEventListener('mousemove', (e) => {{
                const tooltip = document.getElementById('tooltip');
                tooltip.style.left = (e.clientX + 15) + 'px';
                tooltip.style.top = (e.clientY + 15) + 'px';
            }});

            g.addEventListener('mouseleave', () => {{
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
            const newWidth = window.innerWidth;
            const newHeight = window.innerHeight - 50;
            svg.setAttribute('viewBox', `0 0 ${{newWidth}} ${{newHeight}}`);
        }});

        // Start simulation
        tick();
    </script>
</body>
</html>"##,
        workspace.name,
        nodes_json,
        links_json
    );

    Ok(Html(html))
}

/// Animated dynamic view handler.
pub async fn view_dynamic_animated(
    State(state): State<AppState>,
    Path(view_key): Path<String>,
) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

    // Find the dynamic view
    let dynamic_view = workspace.views().dynamic_views.iter()
        .find(|v| v.properties.key == view_key)
        .ok_or_else(|| Error::WorkspaceNotFound(format!("Dynamic view '{}' not found", view_key)))?;

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

    let svg_url = format!("/view/{}/svg", view_key);

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>{} - Animated - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; background: #1a1a1a; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; height: 100vh; overflow: hidden; color: white; }}

        .toolbar {{
            background: #333;
            color: white;
            padding: 10px 20px;
            display: flex;
            align-items: center;
            gap: 20px;
            border-bottom: 1px solid #444;
        }}
        .toolbar a {{ color: white; text-decoration: none; }}
        .toolbar a:hover {{ text-decoration: underline; }}
        .toolbar .separator {{ border-left: 1px solid #555; height: 20px; }}

        .controls {{
            display: flex;
            align-items: center;
            gap: 10px;
            flex: 1;
            justify-content: center;
        }}

        .btn {{
            background: #555;
            color: white;
            border: none;
            padding: 8px 16px;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: background 0.2s;
        }}
        .btn:hover {{ background: #666; }}
        .btn:disabled {{
            background: #3a3a3a;
            color: #666;
            cursor: not-allowed;
        }}
        .btn.primary {{ background: #0066cc; }}
        .btn.primary:hover {{ background: #0052a3; }}
        .btn.primary:disabled {{ background: #003d7a; }}

        .step-info {{
            font-size: 14px;
            color: #ccc;
            min-width: 120px;
            text-align: center;
        }}

        .speed-control {{
            display: flex;
            align-items: center;
            gap: 8px;
        }}
        .speed-control label {{
            font-size: 12px;
            color: #aaa;
        }}
        .speed-control select {{
            background: #555;
            color: white;
            border: 1px solid #666;
            padding: 4px 8px;
            border-radius: 4px;
            cursor: pointer;
        }}

        .diagram-container {{
            height: calc(100vh - 50px);
            overflow: hidden;
            position: relative;
            background: #2a2a2a;
            display: flex;
            align-items: center;
            justify-content: center;
        }}

        #svg-container {{
            position: relative;
            background: white;
            box-shadow: 0 4px 20px rgba(0,0,0,0.4);
        }}

        /* Animation classes */
        .step-arrow {{
            opacity: 0;
            transition: opacity 0.5s ease-in-out;
        }}
        .step-arrow.visible {{
            opacity: 1;
        }}

        .step-element {{
            transition: filter 0.3s ease-in-out;
        }}
        .step-element.active {{
            filter: drop-shadow(0 0 10px #0066cc) brightness(1.2);
        }}

        .step-overlay {{
            position: absolute;
            bottom: 30px;
            left: 50%;
            transform: translateX(-50%);
            background: rgba(0, 0, 0, 0.85);
            color: white;
            padding: 20px 30px;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.3);
            max-width: 600px;
            opacity: 0;
            transition: opacity 0.3s ease-in-out;
            pointer-events: none;
        }}
        .step-overlay.visible {{
            opacity: 1;
        }}
        .step-overlay .step-number {{
            font-size: 12px;
            color: #0066cc;
            font-weight: 600;
            margin-bottom: 8px;
        }}
        .step-overlay .step-desc {{
            font-size: 16px;
            line-height: 1.4;
        }}

        .keyboard-help {{
            position: fixed;
            bottom: 20px;
            left: 20px;
            font-size: 11px;
            color: #666;
        }}
    </style>
</head>
<body>
    <div class="toolbar">
        <a href="/">← Back</a>
        <span>{}</span>
        <div class="separator"></div>

        <div class="controls">
            <button class="btn" id="btn-reset" onclick="resetAnimation()">⟲ Reset</button>
            <button class="btn" id="btn-prev" onclick="previousStep()" disabled>← Previous</button>
            <button class="btn primary" id="btn-play" onclick="togglePlay()">▶ Play</button>
            <button class="btn" id="btn-next" onclick="nextStep()">Next →</button>
            <span class="step-info" id="step-counter">Step 0 of {}</span>

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
        <a href="/view/{}">View Static</a>
    </div>

    <div class="diagram-container">
        <div id="svg-container"></div>
        <div class="step-overlay" id="step-overlay">
            <div class="step-number" id="overlay-number">Step 1</div>
            <div class="step-desc" id="overlay-desc">Step description</div>
        </div>
    </div>

    <div class="keyboard-help">
        Space to play/pause • ← → to step • R to reset • 1-9 to jump to step
    </div>

    <script>
        // Animation state
        const steps = {};
        const totalSteps = {};
        let currentStep = 0;
        let isPlaying = false;
        let playInterval = null;
        let playSpeed = 2000; // milliseconds

        // Elements
        let svgElements = [];
        let arrowElements = [];

        // Load SVG and initialize
        async function loadSVG() {{
            try {{
                const response = await fetch('{}');
                const svgText = await response.text();
                const container = document.getElementById('svg-container');
                container.innerHTML = svgText;

                // Wait for SVG to be inserted
                await new Promise(resolve => setTimeout(resolve, 100));

                const svg = container.querySelector('svg');
                if (!svg) {{
                    console.error('SVG not found in response');
                    return;
                }}

                // Find and tag all arrows (lines with markers)
                const lines = svg.querySelectorAll('line[marker-end]');
                arrowElements = Array.from(lines);

                // Hide all arrows initially
                arrowElements.forEach((arrow, idx) => {{
                    arrow.classList.add('step-arrow');
                    arrow.style.opacity = '0';
                    arrow.dataset.stepIndex = idx;
                }});

                // Tag elements by finding their closest g or rect containers
                // This is a simplified approach - in production you'd want more precise element tracking
                const rects = svg.querySelectorAll('rect:not([id*="marker"])');
                svgElements = Array.from(rects).filter(r => {{
                    const width = parseFloat(r.getAttribute('width') || '0');
                    const height = parseFloat(r.getAttribute('height') || '0');
                    return width > 50 && height > 50; // Filter out small decorative rects
                }});

                svgElements.forEach(el => {{
                    el.classList.add('step-element');
                }});

                updateDisplay();
            }} catch (err) {{
                console.error('Error loading SVG:', err);
            }}
        }}

        function updateDisplay() {{
            // Update step counter
            document.getElementById('step-counter').textContent = `Step ${{currentStep}} of ${{totalSteps}}`;

            // Update button states
            document.getElementById('btn-prev').disabled = currentStep === 0;
            document.getElementById('btn-next').disabled = currentStep >= totalSteps;

            // Show/hide arrows based on current step
            arrowElements.forEach((arrow, idx) => {{
                if (idx < currentStep) {{
                    arrow.style.opacity = '1';
                    arrow.classList.add('visible');
                }} else {{
                    arrow.style.opacity = '0';
                    arrow.classList.remove('visible');
                }}
            }});

            // Highlight active elements
            svgElements.forEach(el => {{
                el.classList.remove('active');
            }});

            // Show step overlay
            const overlay = document.getElementById('step-overlay');
            if (currentStep > 0 && currentStep <= steps.length) {{
                const step = steps[currentStep - 1];
                document.getElementById('overlay-number').textContent = `Step ${{step.order}}`;
                document.getElementById('overlay-desc').textContent = step.description || 'No description';
                overlay.classList.add('visible');

                // Highlight active elements (simplified - would need proper element ID matching)
                if (currentStep - 1 < svgElements.length) {{
                    svgElements[Math.min(currentStep - 1, svgElements.length - 1)]?.classList.add('active');
                }}
            }} else {{
                overlay.classList.remove('visible');
            }}
        }}

        function nextStep() {{
            if (currentStep < totalSteps) {{
                currentStep++;
                updateDisplay();
            }}
            if (currentStep >= totalSteps) {{
                stopPlaying();
            }}
        }}

        function previousStep() {{
            if (currentStep > 0) {{
                currentStep--;
                updateDisplay();
            }}
        }}

        function resetAnimation() {{
            currentStep = 0;
            stopPlaying();
            updateDisplay();
        }}

        function togglePlay() {{
            if (isPlaying) {{
                stopPlaying();
            }} else {{
                startPlaying();
            }}
        }}

        function startPlaying() {{
            if (currentStep >= totalSteps) {{
                resetAnimation();
            }}
            isPlaying = true;
            document.getElementById('btn-play').textContent = '⏸ Pause';
            document.getElementById('btn-play').classList.add('primary');

            playInterval = setInterval(() => {{
                nextStep();
                if (currentStep >= totalSteps) {{
                    stopPlaying();
                }}
            }}, playSpeed);
        }}

        function stopPlaying() {{
            isPlaying = false;
            document.getElementById('btn-play').textContent = '▶ Play';
            if (playInterval) {{
                clearInterval(playInterval);
                playInterval = null;
            }}
        }}

        function updateSpeed() {{
            const select = document.getElementById('speed-select');
            playSpeed = parseInt(select.value);

            // Restart playing if currently playing
            if (isPlaying) {{
                stopPlaying();
                startPlaying();
            }}
        }}

        // Keyboard shortcuts
        document.addEventListener('keydown', (e) => {{
            if (e.target.tagName === 'INPUT' || e.target.tagName === 'SELECT') return;

            switch(e.key) {{
                case ' ':
                    e.preventDefault();
                    togglePlay();
                    break;
                case 'ArrowRight':
                    e.preventDefault();
                    nextStep();
                    break;
                case 'ArrowLeft':
                    e.preventDefault();
                    previousStep();
                    break;
                case 'r':
                case 'R':
                    resetAnimation();
                    break;
                case '0':
                    resetAnimation();
                    break;
                default:
                    // Jump to step number (1-9)
                    if (e.key >= '1' && e.key <= '9') {{
                        const stepNum = parseInt(e.key);
                        if (stepNum <= totalSteps) {{
                            currentStep = stepNum;
                            updateDisplay();
                        }}
                    }}
            }}
        }});

        // Initialize
        loadSVG();
    </script>
</body>
</html>"##,
        view_key,
        view_key,
        step_count,
        view_key,
        svg_url,
        steps_json,
        step_count
    );

    Ok(Html(html))
}

/// Tree view handler - hierarchical explorer of workspace elements.
pub async fn tree_view(State(state): State<AppState>) -> Result<Html<String>> {
    let workspace = state.get_workspace().await
        .ok_or_else(|| Error::WorkspaceNotFound("No workspace loaded".to_string()))?;

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

    let html = format!(
        r##"<!DOCTYPE html>
<html>
<head>
    <title>Tree View - {} - Structurizr</title>
    <style>
        * {{ box-sizing: border-box; }}
        body {{ margin: 0; padding: 0; font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif; background: #f5f5f5; height: 100vh; overflow: hidden; }}
        .header {{ background: #333; color: white; padding: 15px 20px; display: flex; align-items: center; gap: 20px; }}
        .header a {{ color: white; text-decoration: none; }}
        .header h1 {{ margin: 0; font-size: 18px; }}
        .container {{ display: flex; height: calc(100vh - 54px); }}

        .tree-panel {{ width: 500px; background: white; border-right: 1px solid #ddd; display: flex; flex-direction: column; }}
        .search-box {{ padding: 15px; border-bottom: 1px solid #ddd; }}
        .search-box input {{ width: 100%; padding: 10px; font-size: 14px; border: 2px solid #ddd; border-radius: 6px; }}
        .search-box input:focus {{ outline: none; border-color: #0066cc; }}
        .tree-container {{ flex: 1; overflow-y: auto; padding: 10px; }}

        ul.tree {{ list-style: none; padding: 0; margin: 0; }}
        ul.tree ul {{ padding-left: 20px; }}

        .tree li {{ margin: 2px 0; }}
        .tree-node {{ display: flex; align-items: center; gap: 6px; padding: 6px 8px; border-radius: 4px; cursor: pointer; user-select: none; }}
        .tree-node:hover {{ background: #f0f0f0; }}
        .tree-node.selected {{ background: #e3f2fd; border-left: 3px solid #0066cc; }}

        .toggle {{ width: 16px; text-align: center; font-size: 12px; color: #666; }}
        .icon {{ font-size: 16px; }}
        .name {{ font-weight: 500; font-size: 14px; color: #333; }}
        .count {{ font-size: 11px; color: #888; }}
        .desc {{ font-size: 12px; color: #666; margin-left: auto; max-width: 200px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
        .tech {{ font-size: 11px; color: #0066cc; font-family: monospace; }}

        .children {{ display: none; }}
        .expandable.expanded > .children {{ display: block; }}

        .detail-panel {{ flex: 1; padding: 30px; overflow-y: auto; background: #fafafa; }}
        .detail-content {{ max-width: 800px; background: white; padding: 30px; border-radius: 8px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }}
        .detail-content h2 {{ margin-top: 0; display: flex; align-items: center; gap: 10px; }}
        .detail-content .type-badge {{ background: #e8e8e8; padding: 4px 12px; border-radius: 4px; font-size: 11px; text-transform: uppercase; font-weight: 600; }}
        .detail-content .property {{ margin: 15px 0; }}
        .detail-content .property-label {{ font-weight: 600; color: #666; font-size: 12px; text-transform: uppercase; margin-bottom: 5px; }}
        .detail-content .property-value {{ font-size: 14px; color: #333; line-height: 1.6; }}
        .empty-state {{ color: #888; font-style: italic; text-align: center; padding: 100px 20px; }}
        .no-results {{ color: #888; font-style: italic; padding: 20px; text-align: center; }}
        .stats {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; margin: 20px 0; }}
        .stat-card {{ background: #f5f5f5; padding: 15px; border-radius: 6px; text-align: center; }}
        .stat-card .value {{ font-size: 24px; font-weight: bold; color: #0066cc; }}
        .stat-card .label {{ font-size: 12px; color: #666; margin-top: 5px; }}
    </style>
</head>
<body>
    <div class="header">
        <a href="/">← Back</a>
        <h1>Tree View: {}</h1>
    </div>
    <div class="container">
        <div class="tree-panel">
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
        </div>
    </div>

    <script>
        // Tree expand/collapse functionality
        document.addEventListener('click', (e) => {{
            const toggle = e.target.closest('.toggle');
            if (toggle) {{
                e.stopPropagation();
                const li = toggle.closest('li');
                li.classList.toggle('expanded');
                toggle.textContent = li.classList.contains('expanded') ? '▼' : '▶';
            }}
        }});

        // Node selection and detail display
        document.addEventListener('click', (e) => {{
            const node = e.target.closest('.tree-node');
            if (node && node.dataset.id) {{
                // Remove previous selection
                document.querySelectorAll('.tree-node.selected').forEach(n => n.classList.remove('selected'));

                // Add new selection
                node.classList.add('selected');

                // Show details
                showDetails(node);
            }}
        }});

        function showDetails(node) {{
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
                    <span class="icon">${{getIcon(type)}}</span>
                    ${{escapeHtml(name)}}
                    <span class="type-badge">${{type}}</span>
                </h2>
                <div class="property">
                    <div class="property-label">ID</div>
                    <div class="property-value"><code>${{id}}</code></div>
                </div>
                <div class="property">
                    <div class="property-label">Description</div>
                    <div class="property-value">${{escapeHtml(description)}}</div>
                </div>
            `;

            if (technology) {{
                html += `
                    <div class="property">
                        <div class="property-label">Technology</div>
                        <div class="property-value">${{escapeHtml(technology)}}</div>
                    </div>
                `;
            }}

            detailContent.innerHTML = html;
        }}

        function getIcon(type) {{
            const icons = {{
                'Person': '👤',
                'Software System': '📦',
                'Container': '🗄️',
                'Component': '⚙️',
                'Deployment Node': '🖥️'
            }};
            return icons[type] || '📄';
        }}

        function escapeHtml(text) {{
            const div = document.createElement('div');
            div.textContent = text;
            return div.innerHTML;
        }}

        // Search functionality
        const searchInput = document.getElementById('search-input');
        const tree = document.getElementById('tree');
        const noResults = document.getElementById('no-results');

        searchInput.addEventListener('input', (e) => {{
            const query = e.target.value.toLowerCase().trim();

            if (!query) {{
                // Show all nodes
                document.querySelectorAll('.tree li, .tree-node').forEach(el => {{
                    el.style.display = '';
                }});
                noResults.style.display = 'none';
                tree.style.display = '';
                return;
            }}

            let hasResults = false;

            // Filter nodes
            document.querySelectorAll('.tree-node').forEach(node => {{
                const name = (node.dataset.name || '').toLowerCase();
                const description = (node.dataset.description || '').toLowerCase();
                const technology = (node.dataset.technology || '').toLowerCase();
                const type = (node.dataset.type || '').toLowerCase();

                const matches = name.includes(query) ||
                               description.includes(query) ||
                               technology.includes(query) ||
                               type.includes(query);

                const li = node.closest('li');

                if (matches) {{
                    hasResults = true;
                    li.style.display = '';
                    node.style.display = '';

                    // Expand parent nodes
                    let parent = li.parentElement?.closest('li');
                    while (parent) {{
                        parent.style.display = '';
                        parent.classList.add('expanded');
                        const toggle = parent.querySelector(':scope > .tree-node > .toggle');
                        if (toggle) toggle.textContent = '▼';
                        parent = parent.parentElement?.closest('li');
                    }}
                }} else if (node.dataset.type !== 'group') {{
                    li.style.display = 'none';
                }}
            }});

            // Handle group nodes visibility
            document.querySelectorAll('[data-type="group"]').forEach(groupNode => {{
                const li = groupNode.closest('li');
                const visibleChildren = li.querySelectorAll('.children > li[style=""], .children > li:not([style])');
                if (visibleChildren.length === 0) {{
                    li.style.display = 'none';
                }} else {{
                    li.style.display = '';
                    li.classList.add('expanded');
                    const toggle = groupNode.querySelector('.toggle');
                    if (toggle) toggle.textContent = '▼';
                }}
            }});

            if (hasResults) {{
                noResults.style.display = 'none';
                tree.style.display = '';
            }} else {{
                noResults.style.display = 'block';
                tree.style.display = 'none';
            }}
        }});
    </script>
</body>
</html>"##,
        workspace.name,
        workspace.name,
        tree_html,
        model.people.len(),
        model.software_systems.len(),
        model.relationships.len()
    );

    Ok(Html(html))
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
