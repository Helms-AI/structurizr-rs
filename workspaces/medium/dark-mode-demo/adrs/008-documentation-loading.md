# ADR 008: Documentation Loading

## Status

Accepted

## Context

The Structurizr DSL supports documentation through:

1. `!docs "path"` - Documentation directory
2. `!adrs "path"` - Architecture Decision Records directory

We need to load these files and integrate them into the workspace for rendering in the web UI.

Options considered:

1. **Parse-time loading** - Load docs during DSL parsing
2. **Lazy loading** - Load docs on first access
3. **Post-parse loading** - Load after workspace is parsed
4. **External service** - Separate documentation server

## Decision

We chose **post-parse loading** where documentation is loaded after the DSL is parsed, during workspace initialization in the web server.

### Flow

```
DSL → Parser → Workspace (with !docs property) → load_documentation() → Workspace (with sections)
```

### Implementation

```rust
async fn load_workspace(&self) -> Result<()> {
    let content = tokio::fs::read_to_string(&dsl_path).await?;
    let mut workspace = structurizr_dsl::parse_with_base_path(&content, Some(data_dir))?;

    // Load documentation after parsing
    load_documentation(&mut workspace, data_dir).await?;

    *self.workspace.write().await = Some(workspace);
    Ok(())
}
```

## Consequences

### Positive

- **Separation of concerns**: Parser focuses on DSL syntax
- **Flexibility**: Docs loaded only when needed
- **File watching**: Can detect doc changes independently
- **Error handling**: Doc errors don't break parsing

### Negative

- **Two-phase loading**: More complex initialization
- **Property dependency**: Relies on workspace properties
- **Ordering**: Must load docs in filename order

### Neutral

- Standard Rust async file I/O
- Markdown format for documentation

## Implementation Details

### Property Storage

The parser stores directive paths as workspace properties:

```rust
// In parser.rs
for directive in &ast.directives {
    match directive {
        Directive::Docs(path) => {
            workspace.set_property("structurizr.docs", path);
        }
        Directive::Adrs(path) => {
            workspace.set_property("structurizr.adrs", path);
        }
        _ => {}
    }
}
```

### Documentation Loading

```rust
async fn load_documentation(
    workspace: &mut Workspace,
    data_dir: &Path,
) -> Result<()> {
    // Load docs
    if let Some(docs_path) = workspace.get_property("structurizr.docs").cloned() {
        let docs_dir = data_dir.join(&docs_path);

        if docs_dir.exists() {
            let mut entries = collect_entries(&docs_dir).await?;
            entries.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

            for (order, entry) in entries.iter().enumerate() {
                let content = tokio::fs::read_to_string(entry.path()).await?;
                let title = extract_title(entry.path());

                workspace.documentation.sections.push(DocumentationSection {
                    title: Some(title),
                    content,
                    format: DocumentationFormat::Markdown,
                    order: order as u32 + 1,
                });
            }
        }
    }

    // Similar logic for ADRs...
    Ok(())
}
```

### Title Extraction

Extract title from filename using naming convention:

```rust
fn extract_title(path: &Path) -> String {
    let filename = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Untitled");

    // Handle numeric prefix: "001_Overview" -> "Overview"
    let title = filename
        .trim_start_matches(|c: char| c.is_ascii_digit() || c == '_' || c == '-')
        .replace('_', " ")
        .replace('-', " ");

    titlecase(&title)
}
```

### ADR Processing

Parse ADR metadata from content:

```rust
fn parse_adr_status(content: &str) -> DecisionStatus {
    for line in content.lines() {
        if line.trim().to_lowercase().starts_with("## status") {
            // Look at next non-empty line
            // ...
        }
    }
    DecisionStatus::Proposed
}

fn extract_adr_title(filename: &str, content: &str) -> String {
    // First try # heading in content
    for line in content.lines() {
        if line.starts_with("# ") {
            return line[2..].trim().to_string();
        }
    }
    // Fallback to filename
    extract_title_from_filename(filename)
}
```

### File Ordering

Files are sorted by filename to respect numeric prefixes:

```
docs/
├── 001_Overview.md      → Order 1
├── 002_Architecture.md  → Order 2
├── 003_API.md           → Order 3
└── index.md             → Order 4 (sorted after numbers)
```

### Web Rendering

Documentation rendered in `/docs` handler:

```rust
async fn documentation(State(state): State<AppState>) -> impl IntoResponse {
    let workspace = state.workspace.read().await;
    let ws = workspace.as_ref().ok_or(StatusCode::NOT_FOUND)?;

    let html = render_documentation_page(
        &ws.name,
        &ws.documentation.sections,
        &ws.documentation.decisions,
    );

    Ok(Html(html))
}
```

## DSL Usage

```dsl
workspace "My System" {
    !docs "docs"
    !adrs "adrs"

    model { ... }
    views { ... }
}
```

Directory structure:

```
workspace/
├── workspace.dsl
├── docs/
│   ├── 001_Overview.md
│   ├── 002_Architecture.md
│   └── 003_API_Reference.md
└── adrs/
    ├── 001-use-rust.md
    └── 002-database-choice.md
```

## Alternatives Considered

### Parse-Time Loading

**Pros**: Single pass, simpler mental model
**Cons**: Couples parser to file system, harder testing

### Lazy Loading

**Pros**: Faster initial load
**Cons**: Complex caching, inconsistent behavior

### External Service

**Pros**: Separation, scalability
**Cons**: Deployment complexity, overkill for our needs

## References

- [Structurizr Documentation](https://docs.structurizr.com/dsl/docs)
- [ADR Format](https://adr.github.io/)
