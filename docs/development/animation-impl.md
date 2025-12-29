# Dynamic View Animation Implementation

## Overview

Implemented an animated viewer for dynamic views in structurizr-web that shows sequence of interactions step by step with smooth animations and interactive controls.

## Files Modified

### 1. `crates/structurizr-web/src/handlers.rs`

Added new handler function `view_dynamic_animated` that:
- Retrieves the workspace and finds the specified dynamic view
- Extracts step information from the dynamic view
- Builds JSON data for steps (order, source, destination, description)
- Generates an HTML page with embedded SVG viewer and animation controls

Key features:
- **Animation Controls**: Play, Pause, Previous, Next, Reset buttons
- **Step Counter**: Shows current step (e.g., "Step 3 of 7")
- **Speed Control**: Configurable playback speed (0.5s, 1s, 2s, 3s)
- **Step Overlay**: Displays step number and description
- **Keyboard Shortcuts**:
  - Space: Play/Pause
  - ← →: Navigate steps
  - R: Reset
  - 0-9: Jump to specific step

### 2. `crates/structurizr-web/src/server.rs`

Added new route:
```rust
.route("/view/:key/animate", get(handlers::view_dynamic_animated))
```

This creates the endpoint `/view/{view_key}/animate` for accessing animated dynamic views.

### 3. `crates/structurizr-web/src/handlers.rs` (index handler)

Enhanced the home page to show an "Animate" link for dynamic views:
- Collects all dynamic view keys in a HashSet
- Conditionally adds "Animate" link only for dynamic views
- Link appears between "Present" and "SVG" export options

## JavaScript Animation Implementation

The animation system works by:

1. **SVG Loading**: Fetches the dynamic view SVG from `/view/{key}/svg`
2. **Element Detection**:
   - Finds all arrow elements (`<line marker-end>`)
   - Finds all element boxes (large `<rect>` elements)
   - Tags them with animation classes
3. **Step-by-Step Reveal**:
   - Initially hides all arrows (opacity: 0)
   - Reveals arrows one at a time based on current step
   - Uses CSS transitions for smooth fade-in (0.5s ease-in-out)
4. **Element Highlighting**:
   - Applies glow effect to active elements
   - Uses CSS filter: `drop-shadow(0 0 10px #0066cc) brightness(1.2)`
5. **Step Overlay**:
   - Shows step description in a dark overlay at the bottom
   - Fades in/out with opacity transitions

## CSS Animations

Key CSS classes:
- `.step-arrow`: Base class for relationship arrows
- `.step-arrow.visible`: Opacity 1 when step is active
- `.step-element`: Base class for elements
- `.step-element.active`: Glow effect when element is active
- `.step-overlay`: Description overlay with fade transition

## User Interface

### Toolbar Layout
```
← Back | View Name | ⟲ Reset | ← Previous | ▶ Play | Next → | Step 0 of N | Speed: [Select] | View Static
```

### Controls
- **Reset**: Returns to step 0
- **Previous/Next**: Navigate one step at a time
- **Play/Pause**: Auto-advance through steps
- **Speed Control**: Dropdown with 4 options (0.5s to 3s)
- **View Static**: Link to non-animated view

### Keyboard Shortcuts Help
Displayed at bottom-left:
```
Space to play/pause • ← → to step • R to reset • 1-9 to jump to step
```

## Testing

A test DSL file has been created at `test_dynamic_animation.dsl` with:
- E-commerce system with multiple containers
- Dynamic view showing 10-step order placement flow
- Customer → Web App → API Gateway → Services → Database → Payment Gateway

### To Test:
1. Start the server with the test DSL file:
   ```bash
   cargo run -- serve --workspace test_dynamic_animation.dsl
   ```
2. Open http://localhost:8080 in browser
3. Click on the dynamic view "OrderPlacement"
4. Click the "Animate" link to see the animation

## API Endpoint

**Endpoint**: `GET /view/{key}/animate`

**Parameters**:
- `key`: The view key (must be a dynamic view)

**Response**: HTML page with animated viewer

**Error Cases**:
- View not found: Returns 404 with error message
- View is not a dynamic view: Returns 404 with error message

## Implementation Details

### Step Data Structure
```json
[
  {
    "order": 1,
    "sourceId": "element-id-1",
    "destId": "element-id-2",
    "description": "Step description"
  },
  ...
]
```

### Animation State
```javascript
{
  steps: Array,           // Step data from server
  totalSteps: Number,     // Total number of steps
  currentStep: Number,    // Current step (0 = initial state)
  isPlaying: Boolean,     // Whether auto-play is active
  playInterval: Timer,    // Interval for auto-play
  playSpeed: Number,      // Milliseconds per step
  svgElements: Array,     // Element rectangles
  arrowElements: Array    // Arrow lines
}
```

## Future Enhancements

Potential improvements (not implemented):
1. **Enhanced Element Matching**: Use element IDs from step data to highlight specific source/destination elements
2. **Path Highlighting**: Draw paths between communicating elements
3. **Timeline Scrubber**: Visual timeline with draggable position indicator
4. **Animation Progress Bar**: Show progress through the entire sequence
5. **Step Annotations**: Hover tooltips on arrows showing step descriptions
6. **Export Animation**: Save as animated GIF or video
7. **Customizable Colors**: Theme selection for highlight colors
8. **Pan/Zoom**: Add pan and zoom controls for large diagrams

## Code Quality

- **Type Safety**: Full Rust type safety with proper error handling
- **Error Handling**: Graceful error messages for missing views
- **Code Organization**: Clean separation between handler, routing, and view logic
- **CSS Transitions**: Smooth, performant animations using CSS transitions
- **Keyboard Accessibility**: Full keyboard navigation support
- **Responsive Design**: Flexbox layout adapts to window size

## Build Status

✅ Successfully compiles with no warnings
✅ All existing tests pass
✅ Route properly registered in server
✅ Handler properly exported and accessible

## Summary

This implementation provides a professional, user-friendly animated viewer for dynamic views that:
- Shows interactions step by step with smooth transitions
- Provides intuitive playback controls
- Supports keyboard navigation
- Displays step descriptions clearly
- Integrates seamlessly with existing structurizr-web UI
- Maintains code quality and type safety standards

The feature is production-ready and can be used immediately to visualize dynamic sequence diagrams in an engaging, animated format.
