# IDE & Editor Subsystem

## Overview

The IDE subsystem provides a complete browser-based development environment built on Monaco Editor (the same editor powering VS Code). It includes code editing, file management, integrated terminal, and live preview capabilities.

## Components

### Monaco Editor Integration

The code editor is built on Monaco Editor, providing:

- **Syntax Highlighting**: 50+ languages with TextMate grammars
- **IntelliSense**: Autocomplete, parameter hints, quick info
- **Code Navigation**: Go to definition, find references, peek
- **Refactoring**: Rename symbol, extract method/variable
- **Formatting**: Language-specific formatters
- **Multi-cursor**: Simultaneous editing at multiple positions

```typescript
// Monaco configuration
const editor = monaco.editor.create(container, {
  value: initialCode,
  language: 'typescript',
  theme: 'vs-dark',
  automaticLayout: true,
  minimap: { enabled: true },
  fontSize: 14,
  tabSize: 2,
  wordWrap: 'on',
  scrollBeyondLastLine: false,

  // LSP integration
  suggest: {
    showMethods: true,
    showFunctions: true,
    showConstructors: true,
    showFields: true,
    showVariables: true,
    showClasses: true,
    showStructs: true,
    showInterfaces: true,
    showModules: true,
    showProperties: true,
    showEvents: true,
    showOperators: true,
    showUnits: true,
    showValues: true,
    showConstants: true,
    showEnums: true,
    showEnumMembers: true,
    showKeywords: true,
    showWords: true,
    showColors: true,
    showFiles: true,
    showReferences: true,
    showSnippets: true,
  }
});
```

### Language Server Protocol (LSP)

The LSP Hub proxies language server requests between Monaco and language-specific servers:

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  Monaco Editor  │────▶│    LSP Hub      │────▶│ Language Server │
│  (Browser)      │◀────│  (TypeScript)   │◀────│ (Container)     │
└─────────────────┘     └─────────────────┘     └─────────────────┘
```

**Supported Languages:**

| Language | Server | Features |
|----------|--------|----------|
| TypeScript/JavaScript | tsserver | Full LSP + refactoring |
| Python | pylsp | Completion, diagnostics |
| Go | gopls | Full LSP |
| Rust | rust-analyzer | Full LSP + macros |
| Java | Eclipse JDT | Full LSP |
| C/C++ | clangd | Full LSP |
| Ruby | solargraph | Completion, docs |
| PHP | intelephense | Full LSP |
| C# | OmniSharp | Full LSP |
| Swift | sourcekit-lsp | Basic LSP |

**Server Lifecycle:**

```typescript
// On-demand server spawning
class LSPPool {
  private servers: Map<string, LanguageServer> = new Map();

  async getServer(language: string, workspaceId: string): Promise<LanguageServer> {
    const key = `${workspaceId}:${language}`;

    if (!this.servers.has(key)) {
      const server = await this.spawnServer(language, workspaceId);
      this.servers.set(key, server);

      // Auto-shutdown after inactivity
      server.onIdle(() => {
        setTimeout(() => {
          if (server.isIdle()) {
            server.shutdown();
            this.servers.delete(key);
          }
        }, 5 * 60 * 1000); // 5 minutes
      });
    }

    return this.servers.get(key)!;
  }
}
```

### File Browser

The file browser provides workspace navigation:

**Features:**
- Tree view with lazy loading
- Drag-and-drop file/folder operations
- Context menu (new, rename, delete, copy, move)
- File search with fuzzy matching
- Filter by file type
- Show/hide hidden files

**Virtual File System Abstraction:**

```typescript
interface VFS {
  // Basic operations
  readFile(path: string): Promise<Uint8Array>;
  writeFile(path: string, content: Uint8Array): Promise<void>;
  deleteFile(path: string): Promise<void>;

  // Directory operations
  readDirectory(path: string): Promise<DirEntry[]>;
  createDirectory(path: string): Promise<void>;
  deleteDirectory(path: string, recursive?: boolean): Promise<void>;

  // Metadata
  stat(path: string): Promise<FileStat>;
  exists(path: string): Promise<boolean>;

  // Watch for changes
  watch(path: string, callback: (event: FSEvent) => void): Disposable;

  // Search
  search(query: string, options?: SearchOptions): Promise<SearchResult[]>;
}
```

### Integrated Terminal

Built on xterm.js with PTY support:

**Features:**
- Full PTY emulation
- 256-color support
- Unicode and emoji support
- Scrollback buffer (10,000 lines)
- Copy/paste with keyboard shortcuts
- Link detection and click handling
- Multiple terminal tabs
- Split panes

```typescript
// Terminal initialization
const terminal = new Terminal({
  cursorBlink: true,
  cursorStyle: 'block',
  fontSize: 14,
  fontFamily: 'Fira Code, monospace',
  theme: {
    background: '#1e1e1e',
    foreground: '#d4d4d4',
    cursor: '#ffffff',
    selection: '#264f78',
  },
  scrollback: 10000,
  allowProposedApi: true,
});

// WebGL renderer for performance
const webglAddon = new WebglAddon();
terminal.loadAddon(webglAddon);

// Fit to container
const fitAddon = new FitAddon();
terminal.loadAddon(fitAddon);
fitAddon.fit();
```

**PTY Communication (Crosis Protocol):**

```typescript
// Open shell channel
const shellChannel = await client.openChannel({
  service: 'shell',
  name: 'shell-0'
});

// Send input
function sendInput(data: string) {
  shellChannel.send({ input: data });
}

// Receive output
shellChannel.onCommand((cmd) => {
  if (cmd.output) {
    terminal.write(cmd.output);
  }
});

// Handle resize
terminal.onResize(({ cols, rows }) => {
  shellChannel.send({ resize: { cols, rows } });
});
```

### Live Preview

The preview panel displays running web applications:

**Features:**
- Automatic port detection
- Hot module replacement (HMR)
- Multiple preview tabs
- External preview URL
- Mobile viewport simulation
- Console forwarding

```typescript
// Port detection
async function detectPorts(workspaceId: string): Promise<Port[]> {
  const channel = await client.openChannel({ service: 'portDetect' });

  return new Promise((resolve) => {
    channel.onCommand((cmd) => {
      if (cmd.portOpen) {
        ports.push({
          port: cmd.portOpen.port,
          name: cmd.portOpen.name,
          url: `https://${workspaceId}-${cmd.portOpen.port}.preview.horizonplatform.io`
        });
      }
    });
  });
}

// Preview iframe
<iframe
  src={previewUrl}
  sandbox="allow-scripts allow-same-origin allow-forms allow-popups"
  style={{ width: '100%', height: '100%', border: 'none' }}
/>
```

### Collaboration Overlay

Real-time multiplayer features:

**Features:**
- Remote cursor positions with names
- Selection highlighting
- Active file indicators
- Follow mode (follow collaborator's view)
- Presence awareness (who's online)

```typescript
// Yjs awareness for presence
const awareness = ydoc.awareness;

awareness.setLocalState({
  user: {
    name: currentUser.name,
    color: currentUser.color,
  },
  cursor: {
    position: editor.getPosition(),
    selection: editor.getSelection(),
  },
  activeFile: currentFile,
});

// Render remote cursors
awareness.on('change', () => {
  const states = awareness.getStates();

  for (const [clientId, state] of states) {
    if (clientId !== awareness.clientID) {
      renderRemoteCursor(state.user, state.cursor);
    }
  }
});
```

### AI Assistant Panel

Integrated AI coding assistance:

**Features:**
- Chat interface
- Inline code suggestions
- Command palette integration
- Context-aware prompts
- Streaming responses

```typescript
// AI chat interface
interface AIChatMessage {
  role: 'user' | 'assistant';
  content: string;
  timestamp: Date;
  codeBlocks?: CodeBlock[];
}

// Inline completions
editor.onDidChangeCursorPosition(async (e) => {
  const position = e.position;
  const context = getEditorContext(editor, position);

  const completion = await fetchAICompletion({
    prefix: context.prefix,
    suffix: context.suffix,
    language: context.language,
    fileContext: context.relatedFiles,
  });

  if (completion) {
    showInlineCompletion(editor, position, completion);
  }
});
```

## Performance Optimization

### Virtual Scrolling

Large files are handled with virtual scrolling:

```typescript
// Monaco handles this internally, but we optimize file tree
const VirtualizedFileTree: React.FC = ({ files }) => {
  return (
    <FixedSizeList
      height={containerHeight}
      itemCount={files.length}
      itemSize={24}
      width="100%"
    >
      {({ index, style }) => (
        <FileItem file={files[index]} style={style} />
      )}
    </FixedSizeList>
  );
};
```

### Web Worker Offloading

Heavy operations run in Web Workers:

```typescript
// Syntax highlighting in worker
const highlightWorker = new Worker('/workers/highlight.js');

highlightWorker.postMessage({
  type: 'highlight',
  code: fileContent,
  language: 'typescript',
});

highlightWorker.onmessage = (e) => {
  const { tokens } = e.data;
  applyTokens(editor, tokens);
};
```

### Lazy Loading

Components load on demand:

```typescript
// Lazy load terminal
const Terminal = lazy(() => import('./Terminal'));

// Lazy load preview
const Preview = lazy(() => import('./Preview'));

// In render
<Suspense fallback={<Loading />}>
  {showTerminal && <Terminal />}
  {showPreview && <Preview />}
</Suspense>
```

## Keyboard Shortcuts

| Action | Windows/Linux | macOS |
|--------|---------------|-------|
| Save file | Ctrl+S | Cmd+S |
| Open file | Ctrl+P | Cmd+P |
| Command palette | Ctrl+Shift+P | Cmd+Shift+P |
| Find | Ctrl+F | Cmd+F |
| Replace | Ctrl+H | Cmd+Option+F |
| Go to line | Ctrl+G | Cmd+G |
| Toggle terminal | Ctrl+` | Cmd+` |
| New terminal | Ctrl+Shift+` | Cmd+Shift+` |
| Run code | F5 | F5 |
| Toggle sidebar | Ctrl+B | Cmd+B |
| Split editor | Ctrl+\ | Cmd+\ |
