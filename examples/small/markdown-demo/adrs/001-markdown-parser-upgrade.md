# 001. Use comrak for Markdown Parsing

Date: 2024-12-24

## Status

Accepted

## Context

The structurizr-rs web server needs to render markdown documentation. The original implementation used a custom regex-based parser that only supported basic features:

- Headers (h1-h3)
- Unordered lists
- Code blocks
- Blockquotes
- Inline formatting (bold, italic, code, links)

Users needed support for more advanced features, particularly **tables**, which are commonly used in technical documentation.

## Decision

We will use **comrak** for markdown parsing because:

1. **Full GFM Compliance**: GitHub Flavored Markdown support out of the box
2. **Comprehensive Extensions**:
   - Tables with column alignment
   - Task lists (checkboxes)
   - Strikethrough text
   - Footnotes
   - Description lists
   - Autolinks
3. **Well-Maintained**: Used by crates.io, docs.rs, and GitLab
4. **AST-Based**: Allows flexible heading extraction and ID injection
5. **Production Quality**: Battle-tested in major Rust projects

### Alternatives Considered

**pulldown-cmark**:
- Lighter dependency
- Iterator-based (no full AST)
- Fewer extensions than comrak
- Used by rustdoc

We chose comrak for its more complete GFM support and richer extension set.

## Consequences

### Positive

- Full GFM markdown support
- Better documentation authoring experience
- Tables render correctly with proper styling
- Task lists allow tracking items in documentation
- Footnotes enable academic-style references
- Consistent rendering with GitHub

### Negative

- Larger dependency footprint than custom parser
- Slightly different rendering than previous implementation
- Need to implement heading ID injection manually (comrak doesn't add IDs natively)

### Neutral

- CSS updated to style new elements (tables, task lists, footnotes, etc.)
- This markdown-demo example created to verify all features work correctly
