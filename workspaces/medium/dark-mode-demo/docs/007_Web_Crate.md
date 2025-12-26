# structurizr-web Crate

The `structurizr-web` crate provides an Axum-based web server for viewing and navigating architecture documentation. It includes file watching for auto-reload and a complete web UI.

## Module Overview

```
structurizr-web/
├── src/
│   ├── lib.rs          # Public API
│   ├── server.rs       # Axum server setup
│   ├── handlers.rs     # HTTP request handlers
│   ├── state.rs        # Application state
│   ├── watcher.rs      # File system watcher
│   └── editor.rs       # Editor state management
```

## Server Architecture

### Application State

```rust
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub workspace: Arc<RwLock<Option<Workspace>>>,
    pub workspace_path: Arc<RwLock<Option<PathBuf>>>,
    pub editor: EditorState,
    pub watcher: Arc<RwLock<FileWatcher>>,
}

pub struct Config {
    pub data_dir: PathBuf,
    pub port: u16,
    pub host: String,
    pub auto_save_interval: u64,
    pub auto_refresh_interval: u64,
}
```

### Server Setup

```rust
pub async fn run_server(config: Config) -> Result<()> {
    let state = AppState::new(config.clone());

    // Load workspace
    state.load_workspace().await?;

    // Start file watcher
    state.start_watcher().await?;

    // Build router
    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/diagram/:key", get(handlers::diagram))
        .route("/docs", get(handlers::documentation))
        .route("/api/workspace", get(handlers::api_workspace))
        .route("/api/diagram/:key", get(handlers::api_diagram_svg))
        .layer(Extension(state));

    // Start server
    let addr = config.address().parse()?;
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
```

## Route Handlers

### Index Handler

The index page shows available views:

```rust
pub async fn index(
    Extension(state): Extension<AppState>,
) -> Result<Html<String>, StatusCode> {
    let workspace = state.get_workspace().await
        .ok_or(StatusCode::NOT_FOUND)?;

    let html = render_index(&workspace);
    Ok(Html(html))
}
```

### Diagram Handler

Renders a specific view as SVG:

```rust
pub async fn diagram(
    Extension(state): Extension<AppState>,
    Path(key): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let workspace = state.get_workspace().await
        .ok_or(StatusCode::NOT_FOUND)?;

    let svg = render_view(&workspace, &key)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let html = render_diagram_page(&workspace, &key, &svg);
    Ok(Html(html))
}
```

### Documentation Handler

Renders documentation sections:

```rust
pub async fn documentation(
    Extension(state): Extension<AppState>,
) -> Result<Html<String>, StatusCode> {
    let workspace = state.get_workspace().await
        .ok_or(StatusCode::NOT_FOUND)?;

    let html = render_documentation(&workspace);
    Ok(Html(html))
}
```

### API Handlers

```rust
// Return workspace as JSON
pub async fn api_workspace(
    Extension(state): Extension<AppState>,
) -> Result<Json<Workspace>, StatusCode> {
    let workspace = state.get_workspace().await
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(workspace))
}

// Return diagram as raw SVG
pub async fn api_diagram_svg(
    Extension(state): Extension<AppState>,
    Path(key): Path<String>,
) -> Result<(StatusCode, [(HeaderName, &'static str); 1], String), StatusCode> {
    let workspace = state.get_workspace().await
        .ok_or(StatusCode::NOT_FOUND)?;

    let svg = render_view(&workspace, &key)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((
        StatusCode::OK,
        [(header::CONTENT_TYPE, "image/svg+xml")],
        svg,
    ))
}
```

## File Watching

The file watcher monitors the workspace directory for changes:

```rust
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
    rx: Option<Receiver<notify::Result<Event>>>,
}

impl FileWatcher {
    pub fn start(
        &mut self,
        path: PathBuf,
        state: AppState,
    ) -> Result<()> {
        let (tx, rx) = channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| tx.send(res).unwrap(),
            Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;

        self.watcher = Some(watcher);
        self.rx = Some(rx);

        // Spawn reload task
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if let Ok(event) = event {
                    if should_reload(&event) {
                        state.load_workspace().await.ok();
                    }
                }
            }
        });

        Ok(())
    }
}
```

## Documentation Loading

Documentation is loaded from the `!docs` directive path:

```rust
async fn load_documentation(
    workspace: &mut Workspace,
    data_dir: &Path,
) -> Result<()> {
    if let Some(docs_path) = workspace.get_property("structurizr.docs") {
        let docs_dir = data_dir.join(docs_path);

        if docs_dir.exists() {
            let mut entries = tokio::fs::read_dir(&docs_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension() == Some("md".as_ref()) {
                    let content = tokio::fs::read_to_string(&path).await?;
                    let title = extract_title(&path);

                    workspace.documentation.sections.push(DocumentationSection {
                        title: Some(title),
                        content,
                        format: DocumentationFormat::Markdown,
                        order: extract_order(&path),
                    });
                }
            }

            // Sort by order (filename prefix)
            workspace.documentation.sections.sort_by_key(|s| s.order);
        }
    }

    Ok(())
}
```

## Web UI

### Index Page

```html
<!DOCTYPE html>
<html>
<head>
    <title>Workspace - Structurizr</title>
    <style>/* styles */</style>
</head>
<body>
    <div class="header">
        <h1>{{ workspace.name }}</h1>
    </div>
    <div class="views">
        {% for view in views %}
        <a href="/diagram/{{ view.key }}">
            <div class="view-card">
                <h3>{{ view.title }}</h3>
                <p>{{ view.description }}</p>
            </div>
        </a>
        {% endfor %}
    </div>
    <div class="nav">
        <a href="/docs">Documentation</a>
    </div>
</body>
</html>
```

### Diagram Page

```html
<!DOCTYPE html>
<html>
<head>
    <title>{{ view.title }} - Structurizr</title>
</head>
<body>
    <div class="header">
        <a href="/">← Back</a>
        <h1>{{ view.title }}</h1>
    </div>
    <div class="diagram">
        {{ svg | safe }}
    </div>
</body>
</html>
```

## Starting the Server

```rust
use structurizr_web::{Config, run_server};

#[tokio::main]
async fn main() -> Result<()> {
    let config = Config {
        data_dir: PathBuf::from("./workspace"),
        port: 8080,
        host: "127.0.0.1".to_string(),
        auto_save_interval: 5000,
        auto_refresh_interval: 0,
    };

    run_server(config).await
}
```

## CLI Integration

```bash
# Start server with default settings
structurizr serve

# Custom data directory and port
structurizr serve --data-dir ./my-workspace --port 3000

# Bind to all interfaces
structurizr serve --host 0.0.0.0 --port 8080
```
