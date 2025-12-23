# Presentation Mode

The Structurizr web server now includes a full-screen presentation mode for displaying architecture diagrams as slides.

## Features

- **Full-screen slideshow**: Displays diagrams one at a time in a clean, distraction-free interface
- **Keyboard navigation**: Navigate through slides using intuitive keyboard shortcuts
- **Slide counter**: Always know where you are in the presentation
- **Auto-preloading**: All diagrams are preloaded for smooth transitions
- **Responsive design**: Diagrams automatically scale to fit the screen
- **Help overlay**: Built-in keyboard shortcut reference

## Usage

### View all diagrams in presentation mode

Navigate to `/presentation` to show all views in the workspace:

```
http://localhost:8080/presentation
```

### Present specific views

Use the `views` query parameter to show only specific diagrams (comma-separated):

```
http://localhost:8080/presentation?views=SystemContext,ContainerView
```

### Present a single diagram

```
http://localhost:8080/presentation?views=SystemLandscape
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `→` `Space` `Enter` | Next slide |
| `←` `Backspace` | Previous slide |
| `Home` | Jump to first slide |
| `End` | Jump to last slide |
| `F` | Toggle fullscreen |
| `?` | Toggle help overlay |
| `Esc` | Exit presentation mode |

## Quick Access

The presentation mode can be accessed from multiple places:

1. **Navigation bar**: Click "Presentation Mode" in the top navigation
2. **View cards**: Click "Present" next to any view on the home page
3. **Direct URL**: Navigate directly to `/presentation` or `/presentation?views=...`

## UI Features

- **Black background**: Minimizes distractions and focuses attention on diagrams
- **Centered diagrams**: Diagrams are automatically centered and scaled
- **Slide titles**: Each slide shows the view name below the diagram
- **Progress indicator**: Bottom-right corner shows current slide number
- **Auto-hiding controls**: Mouse-activated controls at the bottom-left
- **Exit button**: Top-right corner (appears on hover)
- **Loading progress**: Shows percentage while diagrams are loading

## Technical Details

- All SVG diagrams are preloaded before the presentation starts
- Smooth opacity transitions between slides
- Responsive scaling ensures diagrams fit any screen size
- Fullscreen API support for immersive presentations
- No external dependencies - pure HTML/CSS/JavaScript
