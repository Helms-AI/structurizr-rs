# Markdown Feature Showcase

This documentation demonstrates all markdown features supported by structurizr-rs using the **comrak** parser with GitHub Flavored Markdown (GFM) extensions.

## Basic Text Formatting

Regular paragraph text with **bold text**, *italic text*, and ***bold italic text***.

You can also use ~~strikethrough~~ for deleted content.

Inline `code` looks like this, useful for `variable_names` or `commands`.

## Headers

All six levels of headers are supported:

# Header 1
## Header 2
### Header 3
#### Header 4
##### Header 5
###### Header 6

## Lists

### Unordered Lists

- First item
- Second item
  - Nested item 1
  - Nested item 2
    - Deeply nested
- Third item

### Ordered Lists

1. First step
2. Second step
   1. Sub-step A
   2. Sub-step B
3. Third step

### Task Lists

- [x] Completed task
- [x] Another completed task
- [ ] Pending task
- [ ] Another pending task

## Tables

### Simple Table

| Feature | Status | Notes |
|---------|--------|-------|
| Headers | Working | All levels h1-h6 |
| Bold/Italic | Working | Standard markdown |
| Tables | Working | GFM extension |
| Task Lists | Working | GFM extension |
| Strikethrough | Working | GFM extension |
| Footnotes | Working | Extended markdown |

### Table with Alignment

| Left Aligned | Center Aligned | Right Aligned |
|:-------------|:--------------:|--------------:|
| Left | Center | Right |
| `code` | **bold** | *italic* |
| Long content that might wrap | More content | 123.45 |

### Complex Table

| Element | Description | Technology | Status |
|---------|-------------|------------|--------|
| Parser | Parses markdown AST | comrak | Active |
| Renderer | Converts to HTML | Rust | Active |
| Styler | Applies CSS | CSS3 | Active |

## Code Blocks

### Inline Code

Use `cargo build` to compile and `cargo test` to run tests.

### Fenced Code Blocks

```rust
fn main() {
    println!("Hello, structurizr-rs!");

    let features = vec![
        "tables",
        "task lists",
        "strikethrough",
        "footnotes",
    ];

    for feature in features {
        println!("Supported: {}", feature);
    }
}
```

```json
{
  "name": "structurizr-rs",
  "version": "0.1.0",
  "features": {
    "gfm": true,
    "tables": true,
    "task_lists": true
  }
}
```

```bash
# Build and run the project
cargo build --release
cargo run -- serve --port 8080

# Run tests
cargo test
```

```yaml
workspace:
  name: "Markdown Demo"
  description: "Demonstrates all markdown features"
  documentation:
    format: markdown
    path: docs/
```

## Blockquotes

> This is a simple blockquote.

> Multi-line blockquotes
> continue with the > prefix
> on each line.

> ### Blockquote with Header
>
> Blockquotes can contain other markdown elements:
> - Lists
> - **Bold text**
> - `Code`

## Horizontal Rules

Content above the rule.

---

Content between rules.

***

Content after rules.

## Links

- [GitHub Repository](https://github.com/structurizr/structurizr-rs)
- [Rust Programming Language](https://www.rust-lang.org "The Rust Programming Language")
- [comrak parser](https://github.com/kivikakk/comrak)

### Autolinks

Autolinks are automatically converted: https://www.rust-lang.org

Email autolinks: user@example.com

## Images

Images are supported with responsive sizing:

![Rust Logo](https://www.rust-lang.org/logos/rust-logo-128x128.png)

## Footnotes

Here is a footnote reference[^1], and another[^longnote].

[^1]: This is the first footnote - short and simple.

[^longnote]: This is a longer footnote with multiple paragraphs.

    Second paragraph of the footnote.

    With `code` and **formatting**.

## Description Lists

Term 1
: Definition for term 1

Term 2
: Definition for term 2
: Another definition for term 2

Complex Term
: A longer definition that explains the term in detail.
  This can span multiple lines and include additional context.

## Special Characters & Escaping

Special characters can be escaped with backslashes:

\*not italic\* and \*\*not bold\*\*

Literal backticks: \`not code\`

## Nested Formatting

- **Bold item** with `code` inside
- *Italic with ~~strikethrough~~*
- [Link with **bold** text](https://example.com)
- `code with` regular text after

## Edge Cases

### Empty List Item

- Regular item
-
- Item after empty

### Adjacent Formatting

**bold***italic*`code`

### Long Words

Supercalifragilisticexpialidocious and Pneumonoultramicroscopicsilicovolcanoconiosis

### Unicode

Greek letters: alpha beta gamma delta epsilon

Mathematical symbols: plus-minus, approximately equals, not equal

Arrows: left arrow, right arrow, up arrow, down arrow

---

## Summary

This document demonstrates all major GFM features supported by the comrak-powered markdown renderer in structurizr-rs. If any element doesn't render correctly, please report it as an issue.
