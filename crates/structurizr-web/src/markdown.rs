//! Markdown rendering with heading extraction using comrak (GitHub Flavored Markdown).

use comrak::nodes::{AstNode, NodeValue};
use comrak::{format_html, parse_document, Arena, Options};

/// Extracted heading metadata from markdown parsing.
#[derive(Debug, Clone)]
pub struct ExtractedHeading {
    /// Heading level (1-6)
    pub level: u8,
    /// Heading text content
    pub title: String,
    /// Generated anchor ID
    pub id: String,
}

/// Result of rendering markdown with heading extraction.
pub struct MarkdownResult {
    /// Rendered HTML content
    pub html: String,
    /// Extracted headings for navigation
    pub headings: Vec<ExtractedHeading>,
}

/// Convert text to a URL-safe slug.
pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Escape HTML special characters to prevent XSS.
pub fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Get comrak options with all GFM extensions enabled.
fn get_options() -> Options {
    let mut options = Options::default();

    // Enable GFM extensions
    options.extension.strikethrough = true;
    options.extension.table = true;
    options.extension.autolink = true;
    options.extension.tasklist = true;
    options.extension.footnotes = true;
    options.extension.description_lists = true;
    options.extension.multiline_block_quotes = true;

    // Render options
    options.render.unsafe_ = true; // Allow raw HTML in markdown
    options.render.github_pre_lang = true; // Use GitHub-style language class for code blocks

    options
}

/// Render markdown to HTML without heading extraction.
pub fn render_markdown(md: &str) -> String {
    let arena = Arena::new();
    let options = get_options();

    let root = parse_document(&arena, md, &options);

    let mut html = vec![];
    format_html(root, &options, &mut html).unwrap();

    String::from_utf8(html).unwrap_or_default()
}

/// Render markdown to HTML with heading extraction for navigation.
///
/// Generates unique IDs for each heading based on section_idx and heading index.
pub fn render_markdown_with_headings(md: &str, section_idx: usize) -> MarkdownResult {
    let arena = Arena::new();
    let options = get_options();

    let root = parse_document(&arena, md, &options);

    // Extract headings and collect their info
    let mut headings = Vec::new();
    let mut heading_idx = 0;

    extract_and_inject_heading_ids(root, section_idx, &mut heading_idx, &mut headings);

    // Render to HTML
    let mut html = vec![];
    format_html(root, &options, &mut html).unwrap();

    MarkdownResult {
        html: String::from_utf8(html).unwrap_or_default(),
        headings,
    }
}

/// Recursively traverse the AST to extract headings and inject IDs.
fn extract_and_inject_heading_ids<'a>(
    node: &'a AstNode<'a>,
    section_idx: usize,
    heading_idx: &mut usize,
    headings: &mut Vec<ExtractedHeading>,
) {
    // Check if this is a heading node
    if let NodeValue::Heading(ref heading) = node.data.borrow().value {
        let level = heading.level;

        // Extract the heading text by collecting all text nodes within
        let title = extract_text_content(node);

        // Generate the ID
        let slug = slugify(&title);
        let id = format!("s{}-h{}-{}", section_idx, *heading_idx, slug);

        headings.push(ExtractedHeading {
            level,
            title,
            id: id.clone(),
        });

        *heading_idx += 1;

        // Inject the ID into the heading node by wrapping it in an HTML anchor
        // comrak doesn't natively support heading IDs, so we'll use the
        // sourcepos attribute to store the ID and process it post-render
        // Actually, let's use a different approach: we'll render normally
        // and then post-process the HTML to add IDs
    }

    // Recurse into children
    for child in node.children() {
        extract_and_inject_heading_ids(child, section_idx, heading_idx, headings);
    }
}

/// Extract plain text content from an AST node and its children.
fn extract_text_content<'a>(node: &'a AstNode<'a>) -> String {
    let mut text = String::new();

    match &node.data.borrow().value {
        NodeValue::Text(t) => {
            text.push_str(t);
        }
        NodeValue::Code(c) => {
            text.push_str(&c.literal);
        }
        NodeValue::SoftBreak | NodeValue::LineBreak => {
            text.push(' ');
        }
        _ => {}
    }

    for child in node.children() {
        text.push_str(&extract_text_content(child));
    }

    text
}

/// Post-process rendered HTML to inject heading IDs.
///
/// This takes the headings extracted during parsing and inserts
/// corresponding `id` attributes into the rendered `<h1>` through `<h6>` tags.
pub fn inject_heading_ids(html: &str, headings: &[ExtractedHeading]) -> String {
    let mut result = html.to_string();
    let mut heading_iter = headings.iter();

    // Process each heading level from h1 to h6
    for level in 1..=6 {
        let open_tag = format!("<h{}>", level);
        let mut search_start = 0;

        loop {
            if let Some(pos) = result[search_start..].find(&open_tag) {
                let absolute_pos = search_start + pos;

                // Find the matching heading in our list
                // We need to iterate through headings in order
                if let Some(heading) = heading_iter.next() {
                    if heading.level == level {
                        // Replace <hN> with <hN id="...">
                        let new_tag = format!("<h{} id=\"{}\">", level, heading.id);
                        result = format!(
                            "{}{}{}",
                            &result[..absolute_pos],
                            new_tag,
                            &result[absolute_pos + open_tag.len()..]
                        );

                        // Move past this tag
                        search_start = absolute_pos + new_tag.len();
                    } else {
                        // Wrong level, this shouldn't happen in well-ordered headings
                        // Reset iterator and continue
                        search_start = absolute_pos + open_tag.len();
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Reset iterator for next level
        heading_iter = headings.iter();
    }

    result
}

/// Render markdown to HTML with heading IDs injected.
///
/// This is the main entry point that combines parsing, heading extraction,
/// and ID injection into a single function.
pub fn render_markdown_with_heading_ids(md: &str, section_idx: usize) -> MarkdownResult {
    let arena = Arena::new();
    let options = get_options();

    let root = parse_document(&arena, md, &options);

    // Extract headings
    let mut headings = Vec::new();
    let mut heading_idx = 0;
    collect_headings(root, section_idx, &mut heading_idx, &mut headings);

    // Render to HTML
    let mut html_bytes = vec![];
    format_html(root, &options, &mut html_bytes).unwrap();
    let html = String::from_utf8(html_bytes).unwrap_or_default();

    // Inject heading IDs into the HTML
    let html_with_ids = inject_heading_ids_sequential(&html, &headings);

    MarkdownResult {
        html: html_with_ids,
        headings,
    }
}

/// Collect headings from the AST without modifying it.
fn collect_headings<'a>(
    node: &'a AstNode<'a>,
    section_idx: usize,
    heading_idx: &mut usize,
    headings: &mut Vec<ExtractedHeading>,
) {
    if let NodeValue::Heading(ref heading) = node.data.borrow().value {
        let level = heading.level;
        let title = extract_text_content(node);
        let slug = slugify(&title);
        let id = format!("s{}-h{}-{}", section_idx, *heading_idx, slug);

        headings.push(ExtractedHeading {
            level,
            title,
            id,
        });

        *heading_idx += 1;
    }

    for child in node.children() {
        collect_headings(child, section_idx, heading_idx, headings);
    }
}

/// Inject heading IDs into HTML by processing headings in document order.
fn inject_heading_ids_sequential(html: &str, headings: &[ExtractedHeading]) -> String {
    let mut result = html.to_string();
    let mut byte_offset: usize = 0;

    for heading in headings {
        let level = heading.level;
        let open_tag = format!("<h{}>", level);
        let new_tag = format!("<h{} id=\"{}\">", level, heading.id);

        // Search for the tag starting from byte_offset (using byte-safe substring)
        if let Some(relative_pos) = result[byte_offset..].find(&open_tag) {
            let absolute_pos = byte_offset + relative_pos;

            // Build the new string by concatenating parts
            let before = &result[..absolute_pos];
            let after = &result[absolute_pos + open_tag.len()..];
            result = format!("{}{}{}", before, new_tag, after);

            // Update byte offset to just after the new tag
            byte_offset = absolute_pos + new_tag.len();
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("API Reference & Guide"), "api-reference-guide");
        assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
        assert_eq!(slugify("CamelCase"), "camelcase");
        assert_eq!(slugify("test-with-dashes"), "test-with-dashes");
    }

    #[test]
    fn test_render_table() {
        let md = "| a | b |\n|---|---|\n| c | d |";
        let html = render_markdown(md);
        assert!(html.contains("<table>"));
        assert!(html.contains("<th>a</th>"));
        assert!(html.contains("<td>c</td>"));
    }

    #[test]
    fn test_render_strikethrough() {
        let md = "~~deleted~~";
        let html = render_markdown(md);
        assert!(html.contains("<del>deleted</del>"));
    }

    #[test]
    fn test_render_task_list() {
        let md = "- [x] Done\n- [ ] Todo";
        let html = render_markdown(md);
        assert!(html.contains("checked"));
        assert!(html.contains("checkbox"));
    }

    #[test]
    fn test_render_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let html = render_markdown(md);
        // comrak uses <pre lang="rust"> format
        assert!(html.contains("<pre"), "Expected <pre in: {}", html);
        assert!(html.contains("<code"), "Expected <code in: {}", html);
        assert!(html.contains("rust"), "Expected rust in: {}", html);
    }

    #[test]
    fn test_heading_extraction() {
        let md = "# Title\n## Section\n### Subsection";
        let result = render_markdown_with_heading_ids(md, 0);
        assert_eq!(result.headings.len(), 3);
        assert_eq!(result.headings[0].title, "Title");
        assert_eq!(result.headings[0].level, 1);
        assert_eq!(result.headings[1].title, "Section");
        assert_eq!(result.headings[1].level, 2);
        assert_eq!(result.headings[2].title, "Subsection");
        assert_eq!(result.headings[2].level, 3);
    }

    #[test]
    fn test_heading_id_generation() {
        let md = "# Hello World";
        let result = render_markdown_with_heading_ids(md, 5);
        assert!(result.html.contains("id=\"s5-h0-hello-world\""));
    }

    #[test]
    fn test_ordered_list() {
        let md = "1. First\n2. Second\n3. Third";
        let html = render_markdown(md);
        assert!(html.contains("<ol>"));
        assert!(html.contains("<li>First</li>"));
    }

    #[test]
    fn test_footnotes() {
        let md = "Here is a footnote[^1].\n\n[^1]: This is the footnote.";
        let html = render_markdown(md);
        assert!(html.contains("footnote"));
    }

    #[test]
    fn test_autolinks() {
        let md = "Visit https://example.com for more info.";
        let html = render_markdown(md);
        assert!(html.contains("<a href=\"https://example.com\""));
    }

    #[test]
    fn test_inline_formatting() {
        let md = "**bold** and *italic* and `code`";
        let html = render_markdown(md);
        assert!(html.contains("<strong>bold</strong>"));
        assert!(html.contains("<em>italic</em>"));
        assert!(html.contains("<code>code</code>"));
    }

    #[test]
    fn test_blockquote() {
        let md = "> This is a quote";
        let html = render_markdown(md);
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn test_image() {
        let md = "![Alt text](https://example.com/image.png)";
        let html = render_markdown(md);
        assert!(html.contains("<img"));
        assert!(html.contains("src=\"https://example.com/image.png\""));
        assert!(html.contains("alt=\"Alt text\""));
    }

    #[test]
    fn test_validate_all_example_docs() {
        use std::fs;
        use std::path::Path;

        let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("examples");

        if !examples_dir.exists() {
            eprintln!("Examples directory not found, skipping validation");
            return;
        }

        let mut files_tested = 0;
        let mut issues = Vec::new();

        fn visit_dir(dir: &Path, files_tested: &mut usize, issues: &mut Vec<String>) {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_dir() {
                        visit_dir(&path, files_tested, issues);
                    } else if path.extension().map_or(false, |ext| ext == "md") {
                        *files_tested += 1;
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                // Test rendering
                                let html = super::render_markdown(&content);
                                let result = super::render_markdown_with_heading_ids(&content, 0);

                                // Verify tables render correctly
                                let has_table_syntax = content.contains("| ")
                                    && content.contains(" |")
                                    && content.lines().any(|l| l.contains("---") && l.contains("|"));
                                let has_table_html = html.contains("<table");

                                if has_table_syntax && !has_table_html {
                                    issues.push(format!(
                                        "{}: Table syntax found but not rendered",
                                        path.display()
                                    ));
                                }

                                // Verify code blocks render
                                let has_code_fence = content.contains("```");
                                let has_pre_tag = html.contains("<pre");

                                if has_code_fence && !has_pre_tag {
                                    issues.push(format!(
                                        "{}: Code block syntax found but not rendered",
                                        path.display()
                                    ));
                                }

                                // Check headings were extracted if present
                                let has_headings = content.lines().any(|l| l.starts_with('#'));
                                if has_headings && result.headings.is_empty() {
                                    issues.push(format!(
                                        "{}: Has heading syntax but none extracted",
                                        path.display()
                                    ));
                                }

                                eprintln!(
                                    "  OK: {} ({} chars, {} headings)",
                                    path.file_name().unwrap().to_string_lossy(),
                                    html.len(),
                                    result.headings.len()
                                );
                            }
                            Err(e) => {
                                issues.push(format!("{}: Failed to read - {}", path.display(), e));
                            }
                        }
                    }
                }
            }
        }

        eprintln!("\nValidating example documentation with comrak...");
        visit_dir(&examples_dir, &mut files_tested, &mut issues);

        eprintln!("\n========================================");
        eprintln!("Summary: {} markdown files tested", files_tested);

        if !issues.is_empty() {
            eprintln!("Issues found:");
            for issue in &issues {
                eprintln!("  - {}", issue);
            }
            panic!("{} issues found in example documentation", issues.len());
        }

        eprintln!("All example documentation validated successfully!");
        assert!(files_tested > 0, "No markdown files found to test");
    }
}
