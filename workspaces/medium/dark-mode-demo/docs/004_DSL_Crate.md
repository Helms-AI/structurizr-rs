# structurizr-dsl Crate

The `structurizr-dsl` crate implements a parser for the Structurizr DSL (Domain Specific Language). It provides a complete lexer and recursive descent parser that transforms DSL text into a `Workspace` structure.

## Module Overview

```
structurizr-dsl/
├── src/
│   ├── lib.rs          # Public API
│   ├── lexer.rs        # Tokenization
│   ├── parser.rs       # Recursive descent parser
│   ├── ast.rs          # Abstract Syntax Tree nodes
│   └── error.rs        # Parser errors
```

## Lexer

The lexer converts DSL text into a stream of tokens.

### Token Types

```rust
pub enum TokenKind {
    // Keywords
    Workspace,
    Model,
    Views,
    Styles,
    Person,
    SoftwareSystem,
    Container,
    Component,
    Include,
    Exclude,
    AutoLayout,

    // Literals
    String(String),
    Identifier(String),
    Number(i64),

    // Operators
    Arrow,           // ->
    Equals,          // =
    Star,            // *

    // Delimiters
    OpenBrace,       // {
    CloseBrace,      // }
    OpenParen,       // (
    CloseParen,      // )

    // Special
    Directive(String), // !docs, !adrs, etc.
    Comment,
    Whitespace,
    Eof,
}
```

### Lexer Implementation

```rust
pub struct Lexer<'a> {
    input: &'a str,
    position: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        while let Some(token) = self.next_token()? {
            if !matches!(token.kind, TokenKind::Whitespace | TokenKind::Comment) {
                tokens.push(token);
            }
        }
        tokens.push(Token::new(TokenKind::Eof, self.line, self.column));
        Ok(tokens)
    }
}
```

## Parser

The parser uses recursive descent to build an AST from tokens.

### Public API

```rust
/// Parse DSL string into a Workspace
pub fn parse(input: &str) -> Result<Workspace> {
    parse_with_base_path(input, None)
}

/// Parse with base path for !include directives
pub fn parse_with_base_path(
    input: &str,
    base_path: Option<&Path>
) -> Result<Workspace> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_workspace()?;

    // Process includes if base path provided
    let ast = if let Some(base) = base_path {
        process_includes(ast, base)?
    } else {
        ast
    };

    build_workspace(ast)
}
```

### Parser Structure

```rust
pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn parse_workspace(&mut self) -> Result<WorkspaceNode> {
        self.expect(TokenKind::Workspace)?;
        let name = self.parse_string()?;
        let description = self.parse_optional_string()?;

        self.expect(TokenKind::OpenBrace)?;

        let mut directives = Vec::new();
        let mut model = None;
        let mut views = None;

        while !self.check(TokenKind::CloseBrace) {
            match self.current_kind() {
                Some(TokenKind::Directive(d)) => {
                    directives.push(self.parse_directive()?);
                }
                Some(TokenKind::Model) => {
                    model = Some(self.parse_model()?);
                }
                Some(TokenKind::Views) => {
                    views = Some(self.parse_views()?);
                }
                _ => return Err(self.unexpected_token()),
            }
        }

        self.expect(TokenKind::CloseBrace)?;

        Ok(WorkspaceNode { name, description, directives, model, views })
    }
}
```

## AST Nodes

### Workspace Node

```rust
pub struct WorkspaceNode {
    pub name: Option<String>,
    pub description: Option<String>,
    pub directives: Vec<Directive>,
    pub model: Option<ModelNode>,
    pub views: Option<ViewsNode>,
    pub properties: HashMap<String, String>,
}
```

### Model Nodes

```rust
pub struct ModelNode {
    pub elements: Vec<ElementNode>,
    pub relationships: Vec<RelationshipNode>,
    pub groups: Vec<GroupNode>,
}

pub enum ElementNode {
    Person(PersonNode),
    SoftwareSystem(SoftwareSystemNode),
}

pub struct PersonNode {
    pub identifier: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub properties: HashMap<String, String>,
}
```

### View Nodes

```rust
pub struct ViewsNode {
    pub system_landscape: Vec<SystemLandscapeViewNode>,
    pub system_context: Vec<SystemContextViewNode>,
    pub container: Vec<ContainerViewNode>,
    pub component: Vec<ComponentViewNode>,
    pub styles: Option<StylesNode>,
}

pub struct SystemContextViewNode {
    pub software_system: String,
    pub key: String,
    pub description: Option<String>,
    pub elements: Vec<ViewElementNode>,
    pub auto_layout: Option<AutoLayoutNode>,
    pub properties: ViewPropertiesNode,
}
```

## Directives

The parser supports several directives:

```rust
pub enum Directive {
    Docs(String),           // !docs "path"
    Adrs(String),           // !adrs "path"
    Include(String),        // !include "file.dsl"
    Constant(String, String), // !const NAME "value"
    ImpliedRelationships(bool),
}
```

### Directive Parsing

```rust
fn parse_directive(&mut self) -> Result<Directive> {
    match self.current_kind() {
        Some(TokenKind::Directive(name)) => {
            self.advance();
            match name.to_lowercase().as_str() {
                "docs" => {
                    let path = self.parse_string()?;
                    Ok(Directive::Docs(path))
                }
                "adrs" => {
                    let path = self.parse_string()?;
                    Ok(Directive::Adrs(path))
                }
                "include" => {
                    let file = self.parse_string()?;
                    Ok(Directive::Include(file))
                }
                _ => Err(Error::UnknownDirective(name.clone())),
            }
        }
        _ => Err(self.unexpected_token()),
    }
}
```

## Error Handling

```rust
pub enum Error {
    UnexpectedToken { expected: String, found: Token },
    UnexpectedEof,
    UnknownKeyword(String),
    UnknownDirective(String),
    DuplicateIdentifier(String),
    UnresolvedReference(String),
    ParseError(String),
}

impl Error {
    pub fn display(&self) -> String {
        match self {
            Error::UnexpectedToken { expected, found } => {
                format!(
                    "Expected {} but found {:?} at line {}, column {}",
                    expected, found.kind, found.line, found.column
                )
            }
            // ... other error formatting
        }
    }
}
```

## Include Processing

The parser can process `!include` directives:

```rust
fn process_includes(ast: WorkspaceNode, base: &Path) -> Result<WorkspaceNode> {
    let mut processed = ast;

    for directive in &ast.directives {
        if let Directive::Include(file) = directive {
            let include_path = base.join(file);
            let content = std::fs::read_to_string(&include_path)?;
            let included_ast = parse_to_ast(&content)?;

            // Merge included content into main AST
            processed = merge_ast(processed, included_ast)?;
        }
    }

    Ok(processed)
}
```

## Usage Example

```rust
use structurizr_dsl::parse;

let dsl = r#"
workspace "My System" "Description" {
    !docs "docs"

    model {
        user = person "User" "A user"
        system = softwareSystem "System" "Main system"
        user -> system "Uses"
    }

    views {
        systemContext system "Context" {
            include *
            autoLayout
        }
    }
}
"#;

let workspace = parse(dsl)?;
println!("Workspace: {}", workspace.name);
```
