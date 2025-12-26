# ADR 007: Web Server Framework

## Status

Accepted

## Context

We need a web server to:

1. Serve rendered diagrams
2. Display documentation
3. Provide an API for workspace data
4. Support file watching for auto-reload

Several Rust web frameworks were considered:

1. **Axum** - Tower-based, async, modular
2. **Actix-web** - Actor model, high performance
3. **Rocket** - Macro-based, ergonomic
4. **Warp** - Filter-based composition
5. **Hyper** - Low-level HTTP library

## Decision

We chose **Axum** as the web framework for the following reasons:

### Type Safety

Axum uses extractors that are compile-time checked:

```rust
async fn get_diagram(
    Extension(state): Extension<AppState>,
    Path(key): Path<String>,
) -> impl IntoResponse {
    // Type-safe access to state and path parameters
}
```

### Tower Integration

Built on Tower, enabling middleware composition:

```rust
let app = Router::new()
    .route("/", get(index))
    .layer(CorsLayer::new())
    .layer(CompressionLayer::new())
    .layer(TraceLayer::new_for_http());
```

### Tokio Ecosystem

Native async support with Tokio:

```rust
#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

### Simplicity

Minimal boilerplate for common patterns:

```rust
let app = Router::new()
    .route("/", get(index))
    .route("/diagram/:key", get(diagram))
    .route("/api/workspace", get(api_workspace))
    .with_state(state);
```

## Consequences

### Positive

- **Type safety**: Compile-time route checking
- **Performance**: Async throughout, minimal overhead
- **Ecosystem**: Tower middleware compatibility
- **Maintenance**: Active development, good community

### Negative

- **Learning curve**: Tower service model
- **Boilerplate**: Manual extractor definitions
- **Documentation**: Less than Rocket/Actix

### Neutral

- Standard async Rust patterns
- Comparable performance to alternatives

## Implementation Details

### Application State

Shared state using Arc and RwLock:

```rust
#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub workspace: Arc<RwLock<Option<Workspace>>>,
    pub workspace_path: Arc<RwLock<Option<PathBuf>>>,
    pub watcher: Arc<RwLock<FileWatcher>>,
}
```

### Route Handlers

Using extractors for clean handler signatures:

```rust
pub async fn diagram(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let workspace = state.workspace.read().await;
    let ws = workspace.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let svg = render_view(ws, &key)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Html(render_page(&svg)))
}
```

### API Routes

JSON API for programmatic access:

```rust
pub async fn api_workspace(
    State(state): State<AppState>,
) -> Result<Json<Workspace>, StatusCode> {
    let workspace = state.workspace.read().await;
    let ws = workspace.clone().ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json(ws))
}

pub async fn api_diagram_svg(
    State(state): State<AppState>,
    Path(key): Path<String>,
) -> Result<(StatusCode, [(HeaderName, &'static str); 1], String), StatusCode> {
    let workspace = state.workspace.read().await;
    let ws = workspace.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let svg = render_view(ws, &key).map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((
        StatusCode::OK,
        [(CONTENT_TYPE, "image/svg+xml")],
        svg,
    ))
}
```

### File Watching

Integration with notify crate:

```rust
pub struct FileWatcher {
    watcher: Option<RecommendedWatcher>,
}

impl FileWatcher {
    pub fn start(&mut self, path: PathBuf, state: AppState) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = RecommendedWatcher::new(
            move |res| { tx.send(res).ok(); },
            Config::default(),
        )?;

        watcher.watch(&path, RecursiveMode::Recursive)?;
        self.watcher = Some(watcher);

        // Spawn reload task
        tokio::spawn(async move {
            while let Ok(event) = rx.recv() {
                if should_reload(&event) {
                    state.load_workspace().await.ok();
                }
            }
        });

        Ok(())
    }
}
```

### Server Startup

```rust
pub async fn run_server(config: Config) -> Result<()> {
    let state = AppState::new(config.clone());
    state.load_workspace().await?;
    state.start_watcher().await?;

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/diagram/:key", get(handlers::diagram))
        .route("/docs", get(handlers::documentation))
        .route("/api/workspace", get(handlers::api_workspace))
        .route("/api/diagram/:key", get(handlers::api_diagram_svg))
        .with_state(state);

    let addr = config.address();
    let listener = TcpListener::bind(&addr).await?;

    println!("Server running at http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}
```

## Alternatives Considered

### Actix-web

**Pros**: High performance, mature ecosystem
**Cons**: Actor model complexity, macro-heavy

### Rocket

**Pros**: Ergonomic, great DX
**Cons**: Macro-heavy, slower compile times

### Warp

**Pros**: Composable filters, lightweight
**Cons**: Filter chaining can be verbose

### Raw Hyper

**Pros**: Maximum control
**Cons**: Too low-level for our needs

## References

- [Axum Documentation](https://docs.rs/axum)
- [Tower Documentation](https://docs.rs/tower)
- [Tokio Documentation](https://tokio.rs/)
