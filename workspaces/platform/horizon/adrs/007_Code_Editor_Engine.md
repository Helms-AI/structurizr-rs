# ADR-007: Monaco Editor as Code Editor Engine

## Status

Accepted

## Context

The platform requires a professional-grade code editor embedded in the browser. Requirements include:

- Syntax highlighting for 50+ languages
- IntelliSense/autocomplete
- Multi-cursor editing
- Find and replace with regex
- Code folding
- Minimap navigation
- Theming support
- Language Server Protocol integration
- Vim/Emacs keybinding modes
- Collaborative editing support

**Options Considered:**

1. **Monaco Editor** (VS Code's editor)
   - Battle-tested at Microsoft scale
   - Full LSP support
   - Rich extension model

2. **CodeMirror 6**
   - Modern architecture
   - Mobile-friendly
   - Lighter weight

3. **Ace Editor**
   - Mature, stable
   - Good documentation
   - Simpler API

4. **Custom Editor**
   - Full control
   - Optimized for our needs

## Decision

We will use **Monaco Editor** as the code editing engine, wrapped in a custom React component with collaboration and AI integration.

**Key Design:**

1. **Monaco Core**: VS Code's editor for editing experience
2. **CRDT Binding**: y-monaco for collaborative editing
3. **LSP Proxy**: Monaco connected to language servers via WebSocket
4. **Custom Extensions**: AI completions, collaboration UI

## Alternatives Considered

### CodeMirror 6

**Pros:**
- Lighter weight (~200KB vs ~500KB)
- Better mobile touch support
- Modern reactive architecture
- Easier to extend

**Cons:**
- Less mature ecosystem
- Fewer built-in features
- Would need more custom development
- LSP integration less polished

**Why Rejected:** Monaco's maturity and VS Code compatibility outweigh size benefits.

### Ace Editor

**Pros:**
- Simple, stable API
- Good performance
- Extensive documentation
- Easy to embed

**Cons:**
- Older architecture
- Less active development
- Limited collaboration support
- Basic LSP support

**Why Rejected:** Monaco offers significantly better feature set.

### Custom Editor

**Pros:**
- Complete control
- Optimized bundle size
- No external dependencies
- Tailored UX

**Cons:**
- Massive development effort
- Years of edge cases to handle
- Ongoing maintenance burden
- Would never match Monaco quality

**Why Rejected:** Not feasible given development timeline and resources.

## Consequences

### Positive

- **VS Code familiarity**: Users already know Monaco from VS Code
- **Rich features**: Syntax highlighting, IntelliSense, minimap out of box
- **LSP ready**: Built-in Language Server Protocol support
- **Theming**: Easy to match platform design system
- **Active development**: Regular updates from Microsoft

### Negative

- **Bundle size**: ~500KB gzipped minimum
- **Complex API**: Steep learning curve for customization
- **Memory usage**: Heavier than alternatives
- **Web Worker requirement**: LSP features need workers
- **Mobile limitations**: Not optimized for touch

### Mitigation

| Issue | Mitigation |
|-------|------------|
| Bundle size | Dynamic import, code splitting |
| Complex API | Wrapper components, abstraction layer |
| Memory usage | Editor recycling, virtualization |
| Web Workers | Service worker caching, preload |
| Mobile | Simplified mobile experience |

## Implementation

### Monaco React Wrapper

```typescript
import * as monaco from 'monaco-editor';
import { useRef, useEffect, useCallback } from 'react';

interface EditorProps {
  value: string;
  language: string;
  theme?: string;
  onChange?: (value: string) => void;
  onCursorChange?: (position: Position) => void;
  readOnly?: boolean;
  options?: monaco.editor.IStandaloneEditorConstructionOptions;
}

export function MonacoEditor({
  value,
  language,
  theme = 'horizon-dark',
  onChange,
  onCursorChange,
  readOnly = false,
  options = {},
}: EditorProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);

  useEffect(() => {
    if (!containerRef.current) return;

    // Create editor instance
    const editor = monaco.editor.create(containerRef.current, {
      value,
      language,
      theme,
      readOnly,
      automaticLayout: true,
      minimap: { enabled: true },
      scrollBeyondLastLine: false,
      fontSize: 14,
      lineNumbers: 'on',
      wordWrap: 'on',
      folding: true,
      formatOnPaste: true,
      formatOnType: true,
      ...options,
    });

    editorRef.current = editor;

    // Content change handler
    const contentDisposable = editor.onDidChangeModelContent(() => {
      onChange?.(editor.getValue());
    });

    // Cursor change handler
    const cursorDisposable = editor.onDidChangeCursorPosition((e) => {
      onCursorChange?.({
        line: e.position.lineNumber,
        column: e.position.column,
      });
    });

    return () => {
      contentDisposable.dispose();
      cursorDisposable.dispose();
      editor.dispose();
    };
  }, []);

  // Update content when value prop changes (external updates)
  useEffect(() => {
    if (editorRef.current) {
      const currentValue = editorRef.current.getValue();
      if (value !== currentValue) {
        editorRef.current.setValue(value);
      }
    }
  }, [value]);

  return <div ref={containerRef} style={{ width: '100%', height: '100%' }} />;
}
```

### Collaborative Editing Integration

```typescript
import * as Y from 'yjs';
import { MonacoBinding } from 'y-monaco';
import { WebsocketProvider } from 'y-websocket';

interface CollaborativeEditorProps extends EditorProps {
  fileId: string;
  userId: string;
  userName: string;
  userColor: string;
}

export function CollaborativeEditor({
  fileId,
  userId,
  userName,
  userColor,
  ...editorProps
}: CollaborativeEditorProps) {
  const editorRef = useRef<monaco.editor.IStandaloneCodeEditor | null>(null);
  const bindingRef = useRef<MonacoBinding | null>(null);
  const providerRef = useRef<WebsocketProvider | null>(null);

  useEffect(() => {
    if (!editorRef.current) return;

    // Create Y.js document
    const ydoc = new Y.Doc();
    const ytext = ydoc.getText('content');

    // WebSocket provider for sync
    const provider = new WebsocketProvider(
      'wss://collab.horizonplatform.io',
      `file:${fileId}`,
      ydoc
    );
    providerRef.current = provider;

    // Set awareness state (cursor, user info)
    provider.awareness.setLocalState({
      user: { id: userId, name: userName, color: userColor },
      cursor: null,
    });

    // Bind Y.js to Monaco
    const binding = new MonacoBinding(
      ytext,
      editorRef.current.getModel()!,
      new Set([editorRef.current]),
      provider.awareness
    );
    bindingRef.current = binding;

    return () => {
      binding.destroy();
      provider.destroy();
      ydoc.destroy();
    };
  }, [fileId, userId]);

  return (
    <MonacoEditor
      {...editorProps}
      onMount={(editor) => {
        editorRef.current = editor;
      }}
    />
  );
}
```

### LSP Integration

```typescript
import { MonacoLanguageClient, MessageConnection } from 'monaco-languageclient';
import { toSocket, WebSocketMessageReader, WebSocketMessageWriter } from 'vscode-ws-jsonrpc';

class LSPManager {
  private clients: Map<string, MonacoLanguageClient> = new Map();
  private connections: Map<string, WebSocket> = new Map();

  async connectLanguage(language: string): Promise<void> {
    if (this.clients.has(language)) return;

    const ws = new WebSocket(`wss://lsp.horizonplatform.io/${language}`);
    this.connections.set(language, ws);

    await new Promise((resolve, reject) => {
      ws.onopen = resolve;
      ws.onerror = reject;
    });

    const socket = toSocket(ws);
    const reader = new WebSocketMessageReader(socket);
    const writer = new WebSocketMessageWriter(socket);
    const connection = createConnection(reader, writer, () => ws.close());

    const client = new MonacoLanguageClient({
      name: `${language} Language Client`,
      clientOptions: {
        documentSelector: [language],
        errorHandler: {
          error: () => ErrorAction.Continue,
          closed: () => CloseAction.Restart,
        },
      },
      connectionProvider: {
        get: () => Promise.resolve(connection),
      },
    });

    client.start();
    this.clients.set(language, client);
  }

  disconnect(language: string): void {
    const client = this.clients.get(language);
    if (client) {
      client.stop();
      this.clients.delete(language);
    }

    const ws = this.connections.get(language);
    if (ws) {
      ws.close();
      this.connections.delete(language);
    }
  }
}
```

### AI Completion Provider

```typescript
import * as monaco from 'monaco-editor';

class AICompletionProvider implements monaco.languages.InlineCompletionItemProvider {
  private aiClient: AIClient;
  private debounceMs = 300;
  private pendingRequest: AbortController | null = null;

  constructor(aiClient: AIClient) {
    this.aiClient = aiClient;
  }

  async provideInlineCompletionItems(
    model: monaco.editor.ITextModel,
    position: monaco.Position,
    context: monaco.languages.InlineCompletionContext,
    token: monaco.CancellationToken
  ): Promise<monaco.languages.InlineCompletions> {
    // Cancel pending request
    if (this.pendingRequest) {
      this.pendingRequest.abort();
    }

    // Debounce
    await new Promise((resolve) => setTimeout(resolve, this.debounceMs));

    if (token.isCancellationRequested) {
      return { items: [] };
    }

    // Gather context
    const prefix = model.getValueInRange({
      startLineNumber: Math.max(1, position.lineNumber - 50),
      startColumn: 1,
      endLineNumber: position.lineNumber,
      endColumn: position.column,
    });

    const suffix = model.getValueInRange({
      startLineNumber: position.lineNumber,
      startColumn: position.column,
      endLineNumber: Math.min(model.getLineCount(), position.lineNumber + 20),
      endColumn: model.getLineMaxColumn(
        Math.min(model.getLineCount(), position.lineNumber + 20)
      ),
    });

    // Request completion
    this.pendingRequest = new AbortController();

    try {
      const completion = await this.aiClient.getCompletion({
        prefix,
        suffix,
        language: model.getLanguageId(),
        signal: this.pendingRequest.signal,
      });

      if (!completion || token.isCancellationRequested) {
        return { items: [] };
      }

      return {
        items: [
          {
            insertText: completion,
            range: new monaco.Range(
              position.lineNumber,
              position.column,
              position.lineNumber,
              position.column
            ),
          },
        ],
      };
    } catch (error) {
      if (error.name === 'AbortError') {
        return { items: [] };
      }
      throw error;
    }
  }
}

// Register provider
monaco.languages.registerInlineCompletionsProvider(
  { pattern: '**' },
  new AICompletionProvider(aiClient)
);
```

### Custom Theme

```typescript
import * as monaco from 'monaco-editor';

monaco.editor.defineTheme('horizon-dark', {
  base: 'vs-dark',
  inherit: true,
  rules: [
    { token: 'comment', foreground: '6A9955', fontStyle: 'italic' },
    { token: 'keyword', foreground: '569CD6' },
    { token: 'string', foreground: 'CE9178' },
    { token: 'number', foreground: 'B5CEA8' },
    { token: 'type', foreground: '4EC9B0' },
    { token: 'function', foreground: 'DCDCAA' },
    { token: 'variable', foreground: '9CDCFE' },
    { token: 'constant', foreground: '4FC1FF' },
  ],
  colors: {
    'editor.background': '#0D1117',
    'editor.foreground': '#C9D1D9',
    'editor.lineHighlightBackground': '#161B22',
    'editor.selectionBackground': '#264F78',
    'editorCursor.foreground': '#AEAFAD',
    'editorWhitespace.foreground': '#3B4048',
    'editorIndentGuide.background': '#21262D',
    'editorIndentGuide.activeBackground': '#30363D',
    'editor.selectionHighlightBackground': '#3A3D41',
    'editorLineNumber.foreground': '#6E7681',
    'editorLineNumber.activeForeground': '#C9D1D9',
  },
});
```

### Keybinding Configuration

```typescript
// Vim mode integration
import { initVimMode } from 'monaco-vim';

function enableVimMode(editor: monaco.editor.IStandaloneCodeEditor): VimMode {
  const statusNode = document.getElementById('vim-status');
  return initVimMode(editor, statusNode);
}

// Custom keybindings
function registerKeybindings(editor: monaco.editor.IStandaloneCodeEditor): void {
  // Save file: Cmd/Ctrl + S
  editor.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS,
    () => {
      saveFile(editor.getValue());
    }
  );

  // Format document: Shift + Alt + F
  editor.addCommand(
    monaco.KeyMod.Shift | monaco.KeyMod.Alt | monaco.KeyCode.KeyF,
    () => {
      editor.getAction('editor.action.formatDocument')?.run();
    }
  );

  // Toggle AI completion: Ctrl + Space
  editor.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyCode.Space,
    () => {
      editor.trigger('keyboard', 'editor.action.inlineSuggest.trigger', {});
    }
  );

  // Command palette: Cmd/Ctrl + Shift + P
  editor.addCommand(
    monaco.KeyMod.CtrlCmd | monaco.KeyMod.Shift | monaco.KeyCode.KeyP,
    () => {
      editor.trigger('keyboard', 'editor.action.quickCommand', {});
    }
  );
}
```

## Bundle Optimization

```typescript
// Lazy load Monaco editor
const MonacoEditor = React.lazy(() =>
  import('./MonacoEditor').then((module) => ({
    default: module.MonacoEditor,
  }))
);

// Load only needed language workers
self.MonacoEnvironment = {
  getWorker(_, label) {
    switch (label) {
      case 'json':
        return new Worker(new URL('monaco-editor/esm/vs/language/json/json.worker', import.meta.url));
      case 'css':
      case 'scss':
      case 'less':
        return new Worker(new URL('monaco-editor/esm/vs/language/css/css.worker', import.meta.url));
      case 'html':
      case 'handlebars':
      case 'razor':
        return new Worker(new URL('monaco-editor/esm/vs/language/html/html.worker', import.meta.url));
      case 'typescript':
      case 'javascript':
        return new Worker(new URL('monaco-editor/esm/vs/language/typescript/ts.worker', import.meta.url));
      default:
        return new Worker(new URL('monaco-editor/esm/vs/editor/editor.worker', import.meta.url));
    }
  },
};
```

## References

- [Monaco Editor Documentation](https://microsoft.github.io/monaco-editor/)
- [y-monaco Binding](https://github.com/yjs/y-monaco)
- [Monaco Language Client](https://github.com/TypeFox/monaco-languageclient)
- [VS Code Themes](https://code.visualstudio.com/api/extension-guides/color-theme)
