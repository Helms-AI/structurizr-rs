# structurizr-core Crate

The `structurizr-core` crate provides the fundamental types for representing C4 architecture models. It has no dependencies on other structurizr crates and defines the data structures used throughout the system.

## Module Overview

```
structurizr-core/
├── src/
│   ├── lib.rs          # Public exports
│   ├── model.rs        # C4 model elements
│   ├── view.rs         # View definitions
│   ├── style.rs        # Styling types
│   ├── workspace.rs    # Workspace container
│   └── error.rs        # Error types
```

## Model Elements

### Person

Represents a human user of the system:

```rust
pub struct Person {
    pub id: ElementId,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub location: Option<Location>,
}
```

### Software System

Represents a software system:

```rust
pub struct SoftwareSystem {
    pub id: ElementId,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub location: Option<Location>,
    pub containers: Vec<Container>,
}
```

### Container

Represents a deployable unit within a system:

```rust
pub struct Container {
    pub id: ElementId,
    pub name: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
    pub components: Vec<Component>,
}
```

### Component

Represents a structural building block within a container:

```rust
pub struct Component {
    pub id: ElementId,
    pub name: String,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}
```

### Relationship

Represents a connection between elements:

```rust
pub struct Relationship {
    pub id: RelationshipId,
    pub source_id: ElementId,
    pub destination_id: ElementId,
    pub description: Option<String>,
    pub technology: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}
```

## Element IDs

Elements are identified by UUIDs generated from their names:

```rust
pub struct ElementId(Uuid);

impl ElementId {
    pub fn from_name(name: &str) -> Self {
        // Uses UUID v5 with a namespace for deterministic IDs
        let uuid = Uuid::new_v5(&STRUCTURIZR_NAMESPACE, name.as_bytes());
        Self(uuid)
    }
}
```

## Views

### View Types

The crate supports multiple view types:

```rust
pub enum ViewType {
    SystemLandscape,
    SystemContext,
    Container,
    Component,
    Dynamic,
    Deployment,
}
```

### View Structure

All views share a common structure:

```rust
pub struct SystemContextView {
    pub key: String,
    pub description: Option<String>,
    pub software_system_id: ElementId,
    pub elements: Vec<ElementView>,
    pub relationships: Vec<RelationshipView>,
    pub auto_layout: Option<AutoLayout>,
    pub properties: ViewProperties,
}
```

### Element Positioning

Elements in views can be positioned explicitly or auto-layouted:

```rust
pub struct ElementView {
    pub id: ElementId,
    pub x: Option<i32>,
    pub y: Option<i32>,
}
```

## Styles

### Element Styles

```rust
pub struct ElementStyle {
    pub tag: String,
    pub shape: Option<Shape>,
    pub icon: Option<String>,
    pub icon_position: Option<IconPosition>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub background: Option<String>,
    pub color: Option<String>,
    pub stroke: Option<String>,
    pub stroke_width: Option<u32>,
    pub font_size: Option<u32>,
    pub border: Option<Border>,
    pub opacity: Option<u32>,
    pub metadata: Option<bool>,
    pub description: Option<bool>,
}
```

### Shapes

```rust
pub enum Shape {
    Box,
    RoundedBox,
    Circle,
    Ellipse,
    Hexagon,
    Cylinder,
    Pipe,
    Person,
    Robot,
    Folder,
    WebBrowser,
    MobileDevicePortrait,
    MobileDeviceLandscape,
    Component,
}
```

### Relationship Styles

```rust
pub struct RelationshipStyle {
    pub tag: String,
    pub thickness: Option<u32>,
    pub color: Option<String>,
    pub style: Option<LineStyle>,
    pub routing: Option<Routing>,
    pub font_size: Option<u32>,
    pub width: Option<u32>,
    pub position: Option<u32>,
    pub opacity: Option<u32>,
}
```

## Workspace

The `Workspace` struct is the top-level container:

```rust
pub struct Workspace {
    pub id: Option<i64>,
    pub name: String,
    pub description: Option<String>,
    pub model: Model,
    pub views: Views,
    pub documentation: Documentation,
    pub configuration: Option<ViewConfiguration>,
    pub properties: HashMap<String, String>,
}
```

### Documentation

```rust
pub struct Documentation {
    pub sections: Vec<DocumentationSection>,
    pub decisions: Vec<Decision>,
}

pub struct DocumentationSection {
    pub title: Option<String>,
    pub content: String,
    pub format: DocumentationFormat,
    pub order: u32,
}
```

## Error Handling

The crate defines a `Result` type alias:

```rust
pub type Result<T> = std::result::Result<T, Error>;

pub enum Error {
    ElementNotFound(ElementId),
    DuplicateElement(String),
    InvalidRelationship(String),
    SerializationError(String),
    IoError(std::io::Error),
}
```

## JSON Serialization

Workspaces can be serialized to/from JSON:

```rust
impl Workspace {
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| Error::SerializationError(e.to_string()))
    }
}
```

## Usage Example

```rust
use structurizr_core::{Workspace, Model, Views};

let mut workspace = Workspace::new("My System", "Description");

// Add model elements
let user_id = workspace.model_mut().add_person("User", "A user");
let system_id = workspace.model_mut().add_software_system("System", "Main system");

// Add relationships
workspace.model_mut().add_relationship(user_id, system_id, "Uses", None);

// Serialize to JSON
let json = workspace.to_json()?;
```
