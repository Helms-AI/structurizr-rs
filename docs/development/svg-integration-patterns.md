# SVG Rendering Integration Patterns

This document explains how the SVG rendering system integrates with different parts of structurizr-rs, including the CLI, web server, export formats, and interactive features.

## Table of Contents

1. [Overview](#overview)
2. [CLI Integration](#cli-integration)
3. [Web Server Integration](#web-server-integration)
4. [Export Format Integration](#export-format-integration)
5. [Interactive Features](#interactive-features)
6. [Caching Strategies](#caching-strategies)
7. [Error Handling](#error-handling)
8. [Performance Patterns](#performance-patterns)

## Overview

The SVG rendering system serves as a core component that multiple parts of the application depend on. It provides a unified interface for generating diagrams across different contexts.

### Integration Points

```
SVG Renderer
├── CLI (render command)
├── Web Server (HTTP endpoints)
├── Export System (SVG output)
├── Interactive UI (drag-and-drop)
├── WebSocket Updates (real-time)
└── Cache Layer (performance)
```

## CLI Integration

### Render Command Implementation

The CLI integrates with the renderer through the main binary:

```rust
// src/main.rs
use structurizr_render::SvgRenderer;

#[derive(Parser)]
struct RenderCommand {
    #[arg(short, long)]
    workspace: String,

    #[arg(short, long)]
    output: Option<String>,

    #[arg(long)]
    view_key: Option<String>,
}

impl RenderCommand {
    pub async fn execute(self) -> Result<()> {
        // Load workspace
        let content = fs::read_to_string(&self.workspace)?;
        let workspace = parse_workspace(&content)?;

        // Create renderer
        let renderer = SvgRenderer::default();

        // Render all views or specific view
        if let Some(key) = self.view_key {
            self.render_view(&workspace, &key, &renderer)?;
        } else {
            self.render_all_views(&workspace, &renderer)?;
        }

        Ok(())
    }

    fn render_view(
        &self,
        workspace: &Workspace,
        view_key: &str,
        renderer: &SvgRenderer,
    ) -> Result<()> {
        // Find view
        let view = workspace.views.find_by_key(view_key)
            .ok_or_else(|| anyhow!("View not found: {}", view_key))?;

        // Render based on view type
        let svg = match view {
            View::SystemLandscape(v) => renderer.render_system_landscape(workspace, v)?,
            View::SystemContext(v) => renderer.render_system_context(workspace, v)?,
            View::Container(v) => renderer.render_container(workspace, v)?,
            View::Component(v) => renderer.render_component(workspace, v)?,
            View::Deployment(v) => renderer.render_deployment(workspace, v)?,
            View::Dynamic(v) => renderer.render_dynamic(workspace, v)?,
        };

        // Save to file
        let output_path = self.output_path(view_key);
        fs::write(output_path, svg)?;

        Ok(())
    }

    fn render_all_views(
        &self,
        workspace: &Workspace,
        renderer: &SvgRenderer,
    ) -> Result<()> {
        // Create output directory
        let output_dir = self.output.as_deref().unwrap_or("output");
        fs::create_dir_all(output_dir)?;

        // Render each view
        for (view_type, views) in workspace.views.all_views() {
            for view in views {
                let svg = renderer.render_view(workspace, view)?;
                let filename = format!("{}/{}-{}.svg",
                    output_dir, view_type, view.key());
                fs::write(filename, svg)?;
            }
        }

        Ok(())
    }
}
```

### CLI Options and Configuration

```rust
pub struct CliRenderConfig {
    pub format: OutputFormat,
    pub theme: Option<String>,
    pub size: Option<(i32, i32)>,
    pub layout: LayoutAlgorithm,
}

impl CliRenderConfig {
    pub fn to_renderer(&self) -> SvgRenderer {
        let mut renderer = SvgRenderer::default();

        if let Some((width, height)) = self.size {
            renderer = renderer.with_size(width, height);
        }

        if let Some(theme) = &self.theme {
            renderer = renderer.with_theme(theme.clone());
        }

        renderer
    }
}
```

## Web Server Integration

### HTTP Endpoints

The web server provides multiple endpoints for SVG rendering:

```rust
// crates/structurizr-web/src/handlers.rs
use axum::{extract::*, response::*};
use structurizr_render::SvgRenderer;

pub async fn render_view_svg(
    Path((workspace_id, view_key)): Path<(String, String)>,
    State(state): State<AppState>,
) -> Result<Response, AppError> {
    // Get workspace from cache or load
    let workspace = state.workspace_cache
        .get_or_load(&workspace_id).await?;

    // Find view
    let view = workspace.views.find_by_key(&view_key)
        .ok_or(AppError::ViewNotFound)?;

    // Create renderer with request-specific config
    let renderer = create_renderer(&state.config);

    // Render SVG
    let svg = render_view(&workspace, view, &renderer)?;

    // Return with appropriate headers
    Ok(Response::builder()
        .header("Content-Type", "image/svg+xml")
        .header("Cache-Control", "public, max-age=3600")
        .body(svg)?)
}

fn create_renderer(config: &ServerConfig) -> SvgRenderer {
    SvgRenderer::new()
        .with_size(config.diagram_width, config.diagram_height)
        .with_theme(config.theme.clone())
}
```

### WebSocket Integration

Real-time updates for interactive features:

```rust
// crates/structurizr-web/src/websocket.rs
pub struct WebSocketHandler {
    renderer: SvgRenderer,
    workspace_cache: Arc<WorkspaceCache>,
}

impl WebSocketHandler {
    pub async fn handle_message(
        &self,
        msg: WebSocketMessage,
    ) -> Result<WebSocketResponse> {
        match msg {
            WebSocketMessage::UpdatePosition { view_key, element_id, x, y } => {
                // Update position in workspace
                let workspace = self.workspace_cache.get_mut().await?;
                workspace.update_element_position(&view_key, &element_id, x, y)?;

                // Re-render affected area
                let partial_svg = self.render_partial(&workspace, &view_key)?;

                Ok(WebSocketResponse::PartialUpdate {
                    view_key,
                    svg: partial_svg,
                })
            }

            WebSocketMessage::RequestFullRender { view_key } => {
                let workspace = self.workspace_cache.get().await?;
                let svg = self.render_full(&workspace, &view_key)?;

                Ok(WebSocketResponse::FullRender {
                    view_key,
                    svg,
                })
            }
        }
    }

    fn render_partial(
        &self,
        workspace: &Workspace,
        view_key: &str,
    ) -> Result<String> {
        // Render only changed elements
        // ... implementation
    }
}
```

### REST API Patterns

```rust
pub fn svg_routes() -> Router {
    Router::new()
        // Single view rendering
        .route("/api/workspaces/:id/views/:key/svg", get(render_view_svg))

        // Batch rendering
        .route("/api/workspaces/:id/render-all", post(render_all_views))

        // Export with options
        .route("/api/export/svg", post(export_svg_with_options))

        // Thumbnail generation
        .route("/api/workspaces/:id/views/:key/thumbnail", get(render_thumbnail))
}

async fn export_svg_with_options(
    Json(options): Json<ExportOptions>,
) -> Result<Response> {
    let renderer = SvgRenderer::new()
        .with_size(options.width, options.height)
        .with_background(options.background)
        .with_embed_fonts(options.embed_fonts);

    // ... render and return
}
```

## Export Format Integration

### SVG Export Module

Integration with the export system:

```rust
// crates/structurizr-export/src/svg.rs
use structurizr_render::SvgRenderer;

pub struct SvgExporter {
    renderer: SvgRenderer,
    options: SvgExportOptions,
}

pub struct SvgExportOptions {
    pub embed_styles: bool,
    pub embed_fonts: bool,
    pub include_metadata: bool,
    pub optimize: bool,
}

impl SvgExporter {
    pub fn export_workspace(
        &self,
        workspace: &Workspace,
    ) -> Result<Vec<ExportedFile>> {
        let mut files = Vec::new();

        for view in workspace.views.all_views() {
            let svg = self.export_view(workspace, view)?;
            files.push(ExportedFile {
                name: format!("{}.svg", view.key()),
                content: svg,
                mime_type: "image/svg+xml".to_string(),
            });
        }

        Ok(files)
    }

    fn export_view(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
    ) -> Result<String> {
        let mut svg = self.renderer.render_view(workspace, view)?;

        if self.options.embed_styles {
            svg = self.embed_css_styles(svg)?;
        }

        if self.options.embed_fonts {
            svg = self.embed_fonts(svg)?;
        }

        if self.options.include_metadata {
            svg = self.add_metadata(svg, workspace, view)?;
        }

        if self.options.optimize {
            svg = self.optimize_svg(svg)?;
        }

        Ok(svg)
    }

    fn embed_css_styles(&self, svg: String) -> Result<String> {
        // Embed CSS for standalone SVG
        let css = include_str!("../assets/diagram-styles.css");
        let styled = svg.replace(
            "<svg",
            &format!("<svg><style>{}</style>", css)
        );
        Ok(styled)
    }

    fn optimize_svg(&self, svg: String) -> Result<String> {
        // Remove unnecessary whitespace, optimize paths, etc.
        // ... implementation
    }
}
```

### Multi-Format Export

Coordinating SVG with other formats:

```rust
pub struct UniversalExporter {
    svg_exporter: SvgExporter,
    png_exporter: PngExporter,
    pdf_exporter: PdfExporter,
}

impl UniversalExporter {
    pub async fn export(
        &self,
        workspace: &Workspace,
        formats: Vec<ExportFormat>,
    ) -> Result<ExportBundle> {
        let mut bundle = ExportBundle::new();

        // Generate base SVG first
        let svg_files = self.svg_exporter.export_workspace(workspace)?;

        for format in formats {
            match format {
                ExportFormat::Svg => {
                    bundle.add_files(svg_files.clone());
                }
                ExportFormat::Png => {
                    // Convert SVG to PNG
                    for svg_file in &svg_files {
                        let png = self.svg_to_png(&svg_file.content).await?;
                        bundle.add_file(ExportedFile {
                            name: svg_file.name.replace(".svg", ".png"),
                            content: png,
                            mime_type: "image/png".to_string(),
                        });
                    }
                }
                ExportFormat::Pdf => {
                    // Convert SVG to PDF
                    // ... similar pattern
                }
            }
        }

        Ok(bundle)
    }
}
```

## Interactive Features

### Drag-and-Drop Integration

```rust
// crates/structurizr-web/src/interactive.rs
pub struct InteractiveRenderer {
    base_renderer: SvgRenderer,
    interaction_layer: InteractionLayer,
}

impl InteractiveRenderer {
    pub fn render_interactive(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
    ) -> Result<String> {
        // Generate base SVG
        let mut svg = self.base_renderer.render_view(workspace, view)?;

        // Add interaction attributes
        svg = self.add_draggable_attributes(svg)?;
        svg = self.add_hover_effects(svg)?;
        svg = self.add_click_handlers(svg)?;

        // Add JavaScript for interactivity
        svg = self.embed_interaction_script(svg)?;

        Ok(svg)
    }

    fn add_draggable_attributes(&self, svg: String) -> Result<String> {
        // Add draggable class and data attributes
        let modified = svg.replace(
            "<g class=\"element\"",
            "<g class=\"element draggable\" draggable=\"true\""
        );
        Ok(modified)
    }

    fn embed_interaction_script(&self, svg: String) -> Result<String> {
        let script = r#"
        <script type="text/javascript"><![CDATA[
            // Drag and drop logic
            let draggedElement = null;
            let offset = { x: 0, y: 0 };

            document.querySelectorAll('.draggable').forEach(elem => {
                elem.addEventListener('mousedown', startDrag);
            });

            function startDrag(e) {
                draggedElement = e.currentTarget;
                const rect = draggedElement.getBoundingClientRect();
                offset.x = e.clientX - rect.left;
                offset.y = e.clientY - rect.top;

                document.addEventListener('mousemove', drag);
                document.addEventListener('mouseup', endDrag);
            }

            function drag(e) {
                if (!draggedElement) return;
                const x = e.clientX - offset.x;
                const y = e.clientY - offset.y;
                draggedElement.setAttribute('transform', `translate(${x}, ${y})`);
            }

            function endDrag(e) {
                if (draggedElement) {
                    // Send position update to server
                    sendPositionUpdate(draggedElement);
                }
                draggedElement = null;
                document.removeEventListener('mousemove', drag);
                document.removeEventListener('mouseup', endDrag);
            }

            function sendPositionUpdate(element) {
                // WebSocket or HTTP call to update position
                // ... implementation
            }
        ]]></script>
        "#;

        Ok(svg.replace("</svg>", &format!("{}</svg>", script)))
    }
}
```

### Selection and Highlighting

```rust
pub struct SelectionManager {
    selected_elements: HashSet<String>,
    highlight_style: HighlightStyle,
}

impl SelectionManager {
    pub fn apply_selection(
        &self,
        svg: String,
        selected_ids: &[String],
    ) -> String {
        let mut result = svg;

        for id in selected_ids {
            // Add selection class
            result = result.replace(
                &format!(r#"data-element-id="{}""#, id),
                &format!(r#"data-element-id="{}" class="selected""#, id)
            );
        }

        // Add selection styles
        let styles = self.generate_selection_styles();
        result.replace("<svg", &format!("<svg><style>{}</style>", styles))
    }

    fn generate_selection_styles(&self) -> String {
        format!(r#"
            .selected {{
                filter: drop-shadow(0 0 10px {});
                stroke: {} !important;
                stroke-width: 3 !important;
            }}
            .selected.relationship {{
                stroke-width: 4 !important;
                opacity: 1 !important;
            }}
        "#, self.highlight_style.glow_color, self.highlight_style.stroke_color)
    }
}
```

## Caching Strategies

### Rendered SVG Cache

```rust
use lru::LruCache;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct SvgCache {
    cache: Arc<RwLock<LruCache<CacheKey, CachedSvg>>>,
    renderer: SvgRenderer,
}

#[derive(Hash, PartialEq, Eq)]
struct CacheKey {
    workspace_id: String,
    view_key: String,
    theme: Option<String>,
    size: (i32, i32),
}

struct CachedSvg {
    content: String,
    generated_at: Instant,
    workspace_version: u64,
}

impl SvgCache {
    pub async fn get_or_render(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
        options: &RenderOptions,
    ) -> Result<String> {
        let key = CacheKey {
            workspace_id: workspace.id.clone(),
            view_key: view.key().to_string(),
            theme: options.theme.clone(),
            size: (options.width, options.height),
        };

        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.peek(&key) {
                if cached.workspace_version == workspace.version {
                    return Ok(cached.content.clone());
                }
            }
        }

        // Render new SVG
        let svg = self.renderer
            .with_options(options)
            .render_view(workspace, view)?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.put(key, CachedSvg {
                content: svg.clone(),
                generated_at: Instant::now(),
                workspace_version: workspace.version,
            });
        }

        Ok(svg)
    }

    pub async fn invalidate_workspace(&self, workspace_id: &str) {
        let mut cache = self.cache.write().await;

        // Remove all entries for this workspace
        let keys_to_remove: Vec<_> = cache
            .iter()
            .filter(|(k, _)| k.workspace_id == workspace_id)
            .map(|(k, _)| k.clone())
            .collect();

        for key in keys_to_remove {
            cache.pop(&key);
        }
    }
}
```

### Partial Rendering Cache

```rust
pub struct PartialRenderCache {
    element_cache: HashMap<String, String>,
    relationship_cache: HashMap<String, String>,
}

impl PartialRenderCache {
    pub fn render_element_update(
        &mut self,
        element_id: &str,
        element: &Element,
        style: &ResolvedStyle,
    ) -> String {
        // Generate only the changed element's SVG
        let svg = render_single_element(element, style);
        self.element_cache.insert(element_id.to_string(), svg.clone());
        svg
    }

    pub fn get_cached_elements(&self, ids: &[String]) -> Vec<String> {
        ids.iter()
            .filter_map(|id| self.element_cache.get(id).cloned())
            .collect()
    }
}
```

## Error Handling

### Graceful Degradation

```rust
pub enum RenderError {
    InvalidView(String),
    LayoutFailed(String),
    StyleResolutionFailed(String),
    SvgGenerationFailed(String),
}

impl SvgRenderer {
    pub fn render_with_fallback(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
    ) -> String {
        match self.render_view(workspace, view) {
            Ok(svg) => svg,
            Err(e) => {
                // Log error
                log::error!("Render failed: {:?}", e);

                // Return error placeholder SVG
                self.render_error_placeholder(view, e)
            }
        }
    }

    fn render_error_placeholder(
        &self,
        view: &dyn ViewTrait,
        error: RenderError,
    ) -> String {
        format!(r#"
            <svg viewBox="0 0 800 600" xmlns="http://www.w3.org/2000/svg">
                <rect width="800" height="600" fill="#f0f0f0"/>
                <text x="400" y="280" text-anchor="middle"
                      font-family="Arial" font-size="20" fill="#cc0000">
                    Failed to render view: {}
                </text>
                <text x="400" y="320" text-anchor="middle"
                      font-family="Arial" font-size="14" fill="#666666">
                    Error: {:?}
                </text>
            </svg>
        "#, view.key(), error)
    }
}
```

### Validation and Recovery

```rust
pub struct RenderValidator {
    max_elements: usize,
    max_relationships: usize,
    max_svg_size: usize,
}

impl RenderValidator {
    pub fn validate_before_render(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
    ) -> Result<()> {
        let elements = view.get_elements(workspace);
        let relationships = view.get_relationships(workspace);

        if elements.len() > self.max_elements {
            return Err(anyhow!(
                "Too many elements: {} (max: {})",
                elements.len(),
                self.max_elements
            ));
        }

        if relationships.len() > self.max_relationships {
            return Err(anyhow!(
                "Too many relationships: {} (max: {})",
                relationships.len(),
                self.max_relationships
            ));
        }

        Ok(())
    }

    pub fn validate_after_render(&self, svg: &str) -> Result<()> {
        if svg.len() > self.max_svg_size {
            return Err(anyhow!(
                "SVG too large: {} bytes (max: {})",
                svg.len(),
                self.max_svg_size
            ));
        }

        // Validate SVG structure
        if !svg.starts_with("<svg") || !svg.ends_with("</svg>") {
            return Err(anyhow!("Invalid SVG structure"));
        }

        Ok(())
    }
}
```

## Performance Patterns

### Lazy Rendering

```rust
pub struct LazyRenderer {
    renderer: SvgRenderer,
    render_queue: Arc<Mutex<VecDeque<RenderRequest>>>,
}

impl LazyRenderer {
    pub fn queue_render(&self, request: RenderRequest) -> RenderHandle {
        let handle = RenderHandle::new();

        let mut queue = self.render_queue.lock().unwrap();
        queue.push_back(request.with_handle(handle.clone()));

        handle
    }

    pub async fn process_queue(&self) {
        loop {
            let request = {
                let mut queue = self.render_queue.lock().unwrap();
                queue.pop_front()
            };

            if let Some(req) = request {
                let result = self.renderer.render_view(&req.workspace, &req.view);
                req.handle.complete(result);
            } else {
                // Sleep if queue is empty
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}
```

### Parallel Rendering

```rust
use rayon::prelude::*;

pub struct ParallelRenderer {
    renderer: SvgRenderer,
}

impl ParallelRenderer {
    pub fn render_all_views(
        &self,
        workspace: &Workspace,
    ) -> Vec<Result<RenderedView>> {
        workspace.views.all_views()
            .par_iter()
            .map(|view| {
                let svg = self.renderer.render_view(workspace, view)?;
                Ok(RenderedView {
                    key: view.key().to_string(),
                    svg,
                })
            })
            .collect()
    }
}
```

### Progressive Rendering

```rust
pub struct ProgressiveRenderer {
    renderer: SvgRenderer,
}

impl ProgressiveRenderer {
    pub async fn render_progressive(
        &self,
        workspace: &Workspace,
        view: &dyn ViewTrait,
        progress_callback: impl Fn(RenderProgress),
    ) -> Result<String> {
        // Phase 1: Render basic structure
        progress_callback(RenderProgress::Structure);
        let structure = self.render_structure(workspace, view)?;

        // Phase 2: Add elements
        progress_callback(RenderProgress::Elements);
        let with_elements = self.add_elements(structure, workspace, view)?;

        // Phase 3: Add relationships
        progress_callback(RenderProgress::Relationships);
        let with_relationships = self.add_relationships(with_elements, workspace, view)?;

        // Phase 4: Add labels and styling
        progress_callback(RenderProgress::Styling);
        let final_svg = self.add_styling(with_relationships, workspace, view)?;

        progress_callback(RenderProgress::Complete);
        Ok(final_svg)
    }
}
```

## Related Documentation

- [SVG Rendering Pipeline](svg-rendering-pipeline.md) - Core rendering system
- [Layout Algorithms](layout-algorithms.md) - Layout integration
- [Style System](style-system.md) - Style integration
- [Drag-and-Drop Implementation](drag-drop-implementation.md) - Interactive features
- [Coordinate Systems](coordinate-systems.md) - Coordinate handling