# Dynamic View Animation - Usage Guide

## Quick Start

### 1. Create a Dynamic View in Your DSL

```dsl
workspace "My System" {
    model {
        user = person "User"
        system = softwareSystem "System" {
            webApp = container "Web App"
            api = container "API"
            database = container "Database"
        }

        user -> webApp "Uses"
        webApp -> api "Calls"
        api -> database "Reads/Writes"
    }

    views {
        dynamic system "UserFlow" "User interaction flow" {
            user -> webApp "1. Opens website"
            webApp -> api "2. Fetches data"
            api -> database "3. Queries database"
            database -> api "4. Returns results"
            api -> webApp "5. Returns JSON"
            webApp -> user "6. Displays page"
        }

        styles {
            element "Person" {
                shape Person
                background "#08427b"
            }
        }
    }
}
```

### 2. Start the Server

```bash
# From the project root
cargo run -- serve --workspace your-workspace.dsl

# Or specify a port
cargo run -- serve --workspace your-workspace.dsl --port 8080
```

### 3. Access the Animation

1. Open your browser to `http://localhost:8080`
2. Find your dynamic view (e.g., "UserFlow")
3. Click the **"Animate"** link
4. Use the controls to play through the animation

## Controls

### Button Controls

| Button | Function |
|--------|----------|
| ⟲ Reset | Return to initial state (step 0) |
| ← Previous | Go back one step |
| ▶ Play / ⏸ Pause | Auto-advance through steps |
| Next → | Go forward one step |
| Speed dropdown | Adjust playback speed |

### Speed Options

- **Slow (3s)**: 3 seconds per step
- **Normal (2s)**: 2 seconds per step (default)
- **Fast (1s)**: 1 second per step
- **Very Fast (0.5s)**: 0.5 seconds per step

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| Space | Toggle play/pause |
| → | Next step |
| ← | Previous step |
| R | Reset to beginning |
| 0 | Reset to beginning |
| 1-9 | Jump to step number |

## Visual Feedback

### Step Overlay
- Appears at the bottom of the screen
- Shows current step number and description
- Fades in when step becomes active
- Example: "Step 3" with description "Queries database"

### Element Highlighting
- Active elements glow with a blue shadow
- Source and destination elements are highlighted during each step
- Glow effect: Blue drop-shadow with increased brightness

### Arrow Animation
- Arrows fade in smoothly (0.5 second transition)
- Previously shown arrows remain visible
- Visual history of the sequence flow

### Step Counter
- Always visible in toolbar
- Format: "Step X of Y"
- Updates as you navigate

## Example Workflows

### Viewing an Animation

1. Click "Animate" link from home page
2. Click "Play" button
3. Watch the sequence unfold automatically
4. Animation pauses at the end

### Manual Step Through

1. Click "Animate" link
2. Use "Next" button to advance one step at a time
3. Read step description in overlay
4. Use "Previous" to review earlier steps

### Quick Navigation

1. Press number keys (1-9) to jump to specific steps
2. Press "0" or "R" to reset
3. Use arrow keys for fine-grained control

### Adjusting Speed

1. Open the Speed dropdown
2. Select desired speed (Slow/Normal/Fast/Very Fast)
3. If playing, animation restarts with new speed
4. Speed persists for current session

## Tips and Best Practices

### Writing Good Step Descriptions

```dsl
dynamic system "Flow" {
    # Good: Clear, descriptive action
    user -> webApp "1. Clicks login button"

    # Better: Include result or purpose
    webApp -> api "2. Sends credentials for authentication"

    # Best: Specific details
    api -> database "3. Queries users table with email lookup"
}
```

### Optimal Step Count

- **Ideal**: 5-12 steps per dynamic view
- Too few: Loses detail
- Too many: Hard to follow

### Organizing Complex Flows

Break complex flows into multiple dynamic views:

```dsl
dynamic system "Login" "User login process" {
    # 5-6 steps for login
}

dynamic system "Checkout" "Order checkout process" {
    # 8-10 steps for checkout
}
```

### Presentation Tips

1. **Start with Reset**: Begin presentations from step 0
2. **Use Slow Speed**: Give audience time to read
3. **Pause for Questions**: Use pause button during discussion
4. **Keyboard for Control**: Arrow keys give precise control
5. **Full Screen**: Press F11 for full-screen presentation

## Troubleshooting

### Animation Link Not Showing

**Problem**: "Animate" link not visible on home page

**Solution**:
- Ensure the view is a `dynamic` view, not other types
- Check DSL syntax is correct
- Restart server if recently modified

### Steps Not Animating

**Problem**: Clicking Next/Play doesn't show steps

**Solution**:
- Check browser console for errors (F12)
- Verify SVG loaded correctly
- Ensure dynamic view has steps defined

### Performance Issues

**Problem**: Animation is choppy or slow

**Solution**:
- Reduce number of elements in dynamic view
- Close other browser tabs
- Use faster playback speed
- Refresh page to reset state

### Step Descriptions Not Showing

**Problem**: Overlay appears but description is empty

**Solution**:
- Add descriptions to dynamic view steps in DSL
- Format: `element1 -> element2 "Description here"`

## Advanced Usage

### Combining with Presentation Mode

1. Open animation in one tab
2. Open `/presentation` in another
3. Use animation for detailed flow
4. Use presentation for overview

### Sharing Animations

Share the URL:
```
http://localhost:8080/view/YourViewKey/animate
```

Note: Recipient needs server running with same workspace

### Recording Animations

Use screen recording tools:
- **macOS**: QuickTime Player (Cmd+Shift+5)
- **Windows**: Xbox Game Bar (Win+G)
- **Linux**: SimpleScreenRecorder, OBS Studio

## API Reference

### Endpoint

```
GET /view/:key/animate
```

**Parameters**:
- `:key` - The view key from your DSL

**Response**:
- HTML page with animation viewer

**Example**:
```
http://localhost:8080/view/OrderPlacement/animate
```

### Link Format

From index page:
```html
<a href="/view/{view-key}/animate">Animate</a>
```

## Limitations

Current implementation limitations:

1. **Element Highlighting**: Simplified element detection
   - May not precisely match source/destination
   - Future: Use element IDs for accurate matching

2. **Large Diagrams**: No pan/zoom in animation view
   - Workaround: View static version for navigation
   - Future: Add pan/zoom controls

3. **Step Limit**: Keyboard shortcuts support 1-9
   - Steps 10+ use Next button
   - Future: Add timeline scrubber

4. **Concurrent Users**: Animation state is client-side
   - Each browser maintains its own state
   - No synchronization across viewers

## Support

For issues or questions:
- Check the implementation docs: `DYNAMIC_ANIMATION_IMPLEMENTATION.md`
- Review example DSL: `test_dynamic_animation.dsl`
- Report bugs via GitHub issues

## What's Next?

Planned enhancements:
- Timeline scrubber for direct navigation
- Export animations as GIF/video
- Synchronized multi-user viewing
- Custom highlight colors
- Zoom/pan controls
- Step annotations and tooltips
