# Legend/Key Auto-Generation Implementation

## Summary

This document describes the implementation of the diagram legend/key auto-generation feature for `structurizr-render`.

## Overview

A legend rendering system has been added to the SVG renderer that automatically generates a visual key showing all unique element types and relationship styles in the current diagram. The legend is positioned at the bottom-left corner of the diagram and displays:

1. All unique element types with their colors and shapes
2. All unique relationship types with their line styles

## Implementation Details

### Location

File: `/Users/kon1790/GitHub/structurizr-rs/crates/structurizr-render/src/svg.rs`

### Key Components

#### 1. SvgRenderer Structure

Added a new field to control legend visibility:

```rust
pub struct SvgRenderer {
    width: u32,
    height: u32,
    padding: u32,
    show_legend: bool,  // New field
}
```

#### 2. Public API Methods

Three new methods for controlling legend display:

- `with_legend(show_legend: bool)` - Builder pattern method
- `enable_legend()` - Enable legend rendering
- `disable_legend()` - Disable legend rendering

#### 3. Legend Data Structures

Two helper structs for legend entries:

```rust
struct LegendElementEntry {
    label: String,
    shape: Shape,
    background: String,
    stroke: String,
    stroke_width: u32,
    border: Border,
}

struct LegendRelationshipEntry {
    label: String,
    style: LineStyle,
    color: String,
    thickness: u32,
}
```

#### 4. Core Legend Rendering Function

`render_legend()` - The main function that:

1. **Collects unique element types**:
   - Scans all elements in the view
   - Deduplicates by element type
   - Resolves styles using the workspace's style configuration
   - Creates legend entries with appropriate labels

2. **Collects unique relationship styles**:
   - Scans all relationships in the view
   - Deduplicates by visual style (color, thickness, line style)
   - Creates legend entries with labels from tags or defaults

3. **Renders the legend SVG**:
   - Calculates dimensions based on number of entries
   - Positions at bottom-left corner (x=20, y=height-legendHeight-20)
   - Renders background box with border
   - Renders title "Legend" with separator
   - Renders element icons (20x20 pixels) with labels
   - Renders relationship line samples with labels

### Legend Layout

```
Position: Bottom-left corner (20px from left, 20px from bottom)
Size: 250px wide, dynamically calculated height
Background: White (#ffffff)
Border: Light gray (#cccccc), 2px, rounded corners (5px radius)
Padding: 15px internal padding
Item Height: 30px per entry
Icon Size: 20x20 pixels
```

### Element Type Detection

The legend automatically detects and displays these element types:

- Person (with Person shape icon)
- Software System (with RoundedBox icon)
- Container (with RoundedBox icon)
- Component (with Component shape icon)
- External System (with RoundedBox icon)
- Deployment Node (with RoundedBox icon)
- Infrastructure Node (with RoundedBox icon)

### Relationship Style Detection

Relationships are grouped by their visual properties:

- **Line Style**: Solid, Dashed, Dotted
- **Color**: Any configured color
- **Thickness**: Any configured thickness

Labels are derived from:
1. Custom tags on relationships (excluding "Relationship" tag)
2. Default based on line style (e.g., "Async Relationship" for dashed)

### Style Resolution

The legend uses the same `StyleResolver` as the main diagram rendering to ensure visual consistency:

- Element colors and shapes match the configured styles
- Relationship lines match the configured line styles
- Custom themes are respected

## Usage Examples

### Basic Usage

```rust
use structurizr_render::SvgRenderer;

// Create renderer with legend
let renderer = SvgRenderer::default().with_legend(true);
let svg = renderer.render_system_landscape(&workspace, &view)?;
```

### Runtime Toggle

```rust
let mut renderer = SvgRenderer::default();
renderer.enable_legend();
let svg = renderer.render_container(&workspace, &view)?;
```

### Custom Dimensions

```rust
let renderer = SvgRenderer::new(3000, 2000)
    .with_legend(true);
```

## Testing

Three new tests were added:

1. `test_render_with_legend()` - Verifies legend appears when enabled
2. `test_legend_toggle()` - Tests enable/disable functionality
3. Integration with existing tests to ensure no regression

All tests pass successfully:
```
test result: ok. 21 passed; 0 failed
```

## Performance

The legend generation has minimal performance impact:

- **Time Complexity**: O(n) where n is the number of elements + relationships
- **Space Complexity**: O(k) where k is the number of unique types/styles
- **SVG Output Size**: Adds approximately 1KB per diagram (with typical content)

Example: A diagram with 4 elements and 3 relationships adds ~1025 bytes to the SVG.

## Examples

Three example files were created:

1. **`examples/render_with_legend.rs`**
   - Demonstrates programmatic usage
   - Shows comparison between with/without legend
   - Saves output to files for inspection

2. **`examples/legend_example.dsl`**
   - Sample workspace with multiple element types
   - Demonstrates various relationship styles
   - Shows how custom tags affect legend labels

3. **`examples/README_legend.md`**
   - Comprehensive documentation
   - Usage examples
   - Implementation details
   - Future enhancement ideas

## Files Modified

1. `/Users/kon1790/GitHub/structurizr-rs/crates/structurizr-render/src/svg.rs`
   - Added `show_legend` field to `SvgRenderer`
   - Added three public API methods
   - Added two legend data structures
   - Implemented `render_legend()` function
   - Modified `render_svg()` to support legend
   - Added three new tests

## Files Created

1. `/Users/kon1790/GitHub/structurizr-rs/examples/render_with_legend.rs`
2. `/Users/kon1790/GitHub/structurizr-rs/examples/legend_example.dsl`
3. `/Users/kon1790/GitHub/structurizr-rs/examples/README_legend.md`
4. `/Users/kon1790/GitHub/structurizr-rs/LEGEND_IMPLEMENTATION.md` (this file)

## Build Verification

All builds and tests pass:

```bash
# Full workspace build
cargo build --all
# Result: Finished `dev` profile [unoptimized + debuginfo]

# Render crate tests
cargo test --package structurizr-render
# Result: test result: ok. 21 passed; 0 failed

# Example execution
cargo run --example render_with_legend
# Result: ✓ Legend successfully rendered!
```

## Future Enhancements

Potential improvements for future versions:

1. **Configurable Position**: Allow legend placement in different corners
2. **Custom Styling**: Support custom legend colors, fonts, sizes
3. **Filtering**: Option to hide specific element types from legend
4. **Grouping**: Group related element types or relationships
5. **Descriptions**: Add optional descriptions for each entry
6. **Collapsible**: Make legend collapsible in interactive views
7. **Export Options**: Separate legend export functionality

## Backwards Compatibility

The implementation is fully backwards compatible:

- Default behavior (legend disabled) matches previous version
- No changes to existing public API signatures
- All existing tests continue to pass
- Optional feature that must be explicitly enabled

## Conclusion

The legend auto-generation feature is complete and ready for use. It provides a professional, automatically-generated visual key for Structurizr diagrams that adapts to the content and styling of each diagram.
