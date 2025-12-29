# Drag-and-Drop Implementation

## Overview

This document describes the implementation of drag-and-drop positioning for diagram elements in structurizr-rs, including position persistence and undo/redo functionality.

## Architecture

### Data Structures (Rust)

The core position management is implemented in `structurizr-core/src/view.rs`:

```rust
pub struct LayoutState {
    pub positions: HashMap<ElementId, ElementPosition>,
    pub history: VecDeque<LayoutChange>,
    pub redo_stack: VecDeque<LayoutChange>,
    pub dirty: bool,
    pub last_change_timestamp: Option<u64>,
}

pub enum LayoutChange {
    Move {
        element_id: ElementId,
        from: ElementPosition,
        to: ElementPosition,
        timestamp: u64,
    },
    BatchMove {
        moves: Vec<(ElementId, ElementPosition, ElementPosition)>,
        timestamp: u64,
    },
}
```

Key features:
- **Position tracking**: Maintains current positions for all elements
- **History management**: Tracks changes for undo/redo
- **Change coalescing**: Merges rapid drag events (within 500ms) to reduce history noise
- **Dirty tracking**: Marks when changes need to be saved

### SVG Rendering

The SVG renderer (`structurizr-render/src/svg.rs`) wraps each element in a draggable group:

```xml
<g class="draggable-element" data-element-id="{element_id}" transform="translate(0, 0)">
  <!-- Element shape and text content -->
</g>
```

This structure enables:
- CSS transforms for smooth dragging
- Element identification via `data-element-id`
- Independent positioning of each element

### Frontend JavaScript

The edit interface uses inline SVG with drag handlers:

1. **Coordinate transformation**: Converts mouse coordinates to SVG space
2. **Drag tracking**: Maintains drag state and original positions
3. **WebSocket communication**: Sends position updates to server
4. **Undo/redo support**: Local tracking with server synchronization

Key functions:
- `setupDragHandlers()`: Attaches mouse event listeners to draggable elements
- `loadSVG()`: Fetches and injects SVG content into the editor
- Mouse event handlers for drag, pan, and zoom operations

### WebSocket Protocol

Position updates use the existing `EditorMessage` protocol:

```javascript
{
    type: 'element_moved',
    view_key: 'SystemContext',
    element_id: 'uuid-here',
    x: 150,
    y: 200
}
```

The server:
1. Updates in-memory position state
2. Broadcasts to other connected clients
3. Updates the workspace model
4. Queues for persistence (when implemented)

## Features

### Implemented

✅ **Drag-and-drop positioning**
- Elements can be dragged with the mouse
- Positions update in real-time
- Visual feedback during dragging

✅ **SVG coordinate transformation**
- Proper conversion between screen and SVG coordinates
- Works correctly with zoom and pan

✅ **WebSocket communication**
- Position updates sent to server
- Multi-client synchronization supported

✅ **Undo/redo infrastructure**
- Command pattern for position changes
- History tracking with coalescing
- Client-side undo stack

✅ **Pan and zoom**
- Drag on background to pan (matches viewer behavior)
- Scroll wheel for zooming
- Minimap for navigation

✅ **Position persistence**
- Positions saved to `.positions.json` sidecar file
- Positions loaded automatically on workspace startup
- Auto-save with 2-second debounce after dragging stops
- Manual save via Ctrl/Cmd+S keyboard shortcut
- Missing position files handled gracefully (falls back to auto-layout)

### To Be Implemented

⏳ **Visual feedback**
- Hover effects on draggable elements
- Snap-to-grid functionality
- Alignment guides

## Usage

1. Start the server:
```bash
cargo run -- serve --workspaces-dir workspaces
```

2. Navigate to a diagram view and click "Edit"

3. Drag elements to reposition them:
   - Click and drag any element to move it
   - Click and drag on the background to pan the canvas
   - Scroll to zoom in/out

4. Use keyboard shortcuts:
   - Ctrl/Cmd+Z: Undo
   - Ctrl/Cmd+Shift+Z: Redo
   - Ctrl/Cmd+S: Save positions

5. Positions are automatically saved after 2 seconds of inactivity

## Implementation Notes

### Coordinate Systems

The implementation manages three coordinate systems:

1. **Screen coordinates**: Mouse position in browser viewport
2. **SVG viewport coordinates**: Position within the SVG canvas
3. **Model coordinates**: Logical position stored in the workspace

### Performance Optimizations

- **CSS transforms**: Used for smooth 60fps dragging
- **Change coalescing**: Reduces WebSocket traffic during rapid dragging
- **Debounced persistence**: Saves positions after drag ends, not during

### Browser Compatibility

The implementation uses modern web APIs:
- SVG DOM manipulation
- Pointer events
- CSS transforms
- WebSocket

Tested in Chrome, Firefox, and Safari.

## Future Enhancements

1. **Multi-select drag**: Move multiple elements together
2. **Constraint-based layout**: Maintain relationships during drag
3. **Animation**: Smooth transitions for auto-layout
4. **Touch support**: Enable dragging on tablets/phones
5. **Collaborative cursors**: Show other users' selections