# ADR 003: DSL Parser Design

## Status

Accepted

## Context

The Structurizr DSL is a domain-specific language for defining software architecture models. We need to parse this DSL into our internal data structures. Several parsing approaches were considered:

1. **Parser generator** (pest, lalrpop) - Generate parser from grammar
2. **Parser combinator** (nom, combine) - Compose small parsers
3. **Hand-written recursive descent** - Manual implementation
4. **Tree-sitter** - Incremental parsing with error recovery

## Decision

We chose a **hand-written recursive descent parser** with a separate lexer phase.

### Architecture

```
Input → Lexer → Tokens → Parser → AST → Builder → Workspace
```

### Lexer

Tokenizes input into meaningful units:

```rust
pub enum TokenKind {
    Workspace, Model, Views, Person, SoftwareSystem,
    Container, Component, Include, Exclude, AutoLayout,
    String(String), Identifier(String), Number(i64),
    Arrow, Equals, Star, OpenBrace, CloseBrace,
    Directive(String), Comment, Whitespace, Eof,
}
```

### Parser

Recursive descent implementation:

```rust
impl Parser {
    pub fn parse_workspace(&mut self) -> Result<WorkspaceNode> {
        self.expect(TokenKind::Workspace)?;
        let name = self.parse_string()?;
        // ...recursive parsing...
    }
}
```

### AST

Intermediate representation before final workspace:

```rust
pub struct WorkspaceNode {
    pub name: Option<String>,
    pub description: Option<String>,
    pub directives: Vec<Directive>,
    pub model: Option<ModelNode>,
    pub views: Option<ViewsNode>,
}
```

## Consequences

### Positive

- **Full control**: Exact error messages and recovery
- **Performance**: Direct, efficient parsing
- **Maintainability**: Easy to understand and modify
- **No dependencies**: No external parser libraries needed
- **Error messages**: Context-aware, helpful errors

### Negative

- **More code**: Manual implementation of all rules
- **Grammar changes**: Must update parser manually
- **No formal grammar**: Grammar exists only in code

### Neutral

- Follows common Rust patterns (Logos, Chumsky patterns)
- Similar to Structurizr Java's approach

## Implementation Details

### Error Handling

```rust
pub enum Error {
    UnexpectedToken { expected: String, found: Token },
    UnexpectedEof,
    UnknownKeyword(String),
    UnresolvedReference(String),
}

impl Error {
    pub fn display(&self) -> String {
        match self {
            Error::UnexpectedToken { expected, found } =>
                format!("Expected {} at line {}, column {}",
                    expected, found.line, found.column),
            // ...
        }
    }
}
```

### Keyword Handling

Some keywords can also be identifiers (e.g., "Person" is both a keyword and a valid shape name):

```rust
fn expect_identifier_or_shape_keyword(&mut self) -> Result<String> {
    match self.current_kind() {
        Some(TokenKind::Identifier(s)) => Ok(s.clone()),
        Some(TokenKind::Person) => Ok("Person".to_string()),
        Some(TokenKind::Component) => Ok("Component".to_string()),
        _ => Err(self.unexpected_token()),
    }
}
```

### Include Processing

The parser supports `!include` directives:

```rust
fn process_includes(ast: WorkspaceNode, base: &Path) -> Result<WorkspaceNode> {
    for directive in &ast.directives {
        if let Directive::Include(file) = directive {
            let content = std::fs::read_to_string(base.join(file))?;
            let included = parse_to_ast(&content)?;
            // Merge included content
        }
    }
    Ok(ast)
}
```

## Alternatives Considered

### Pest Parser Generator

**Pros**: Formal grammar, automatic error handling
**Cons**: External dependency, less control over errors

### Nom Parser Combinator

**Pros**: Composable, zero-copy parsing
**Cons**: Cryptic error messages, steeper learning curve

### Tree-sitter

**Pros**: Incremental parsing, IDE integration
**Cons**: Heavy dependency, complex setup

## References

- [Crafting Interpreters](https://craftinginterpreters.com/)
- [Structurizr DSL Documentation](https://docs.structurizr.com/dsl/language)
