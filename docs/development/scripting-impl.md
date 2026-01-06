# Scripting Implementation

This document describes the internal architecture of the scripting system in structurizr-rs.

## Architecture Overview

```
crates/structurizr-scripting/
├── src/
│   ├── lib.rs          # Public exports and crate documentation
│   ├── engine.rs       # ScriptEngine - main execution engine
│   ├── api.rs          # WorkspaceHandle - Lua bindings for workspace
│   ├── sandbox.rs      # SandboxConfig - security configuration
│   ├── transpiler.rs   # GroovyTranspiler - Groovy→Lua conversion
│   ├── error.rs        # Error types (ScriptError, Result)
│   └── plugin/         # WASM plugin system
│       ├── mod.rs      # PluginManifest, PluginCapabilities
│       └── wasm_runtime.rs  # PluginEngine, wasmtime integration
```

## Core Components

### ScriptEngine

The `ScriptEngine` (`engine.rs`) is the main entry point for script execution:

```rust
pub struct ScriptEngine {
    config: ScriptConfig,
    transpiler: GroovyTranspiler,
}
```

**Key methods:**

| Method | Description |
|--------|-------------|
| `execute()` | Language-agnostic execution (auto-detects/transpiles) |
| `execute_lua()` | Direct Lua script execution |
| `execute_file()` | Load and execute external script file |

**Execution flow:**

```
execute(script, language)
    ├─ if Lua: execute_lua(script)
    ├─ if Groovy: transpile() → execute_lua()
    └─ if Kotlin: transpile() → execute_lua()

execute_lua(script)
    ├─ Clone workspace into WorkspaceHandle
    ├─ Create sandboxed Lua state
    ├─ Register workspace API
    ├─ Execute script
    └─ Copy modified workspace back
```

### ScriptConfig

Configuration for script execution:

```rust
pub struct ScriptConfig {
    pub sandbox: SandboxConfig,    // Security settings
    pub auto_transpile: bool,      // Enable Groovy→Lua conversion
    pub base_path: Option<PathBuf>, // Base path for external scripts
}
```

### ScriptLanguage

Supported script languages:

```rust
pub enum ScriptLanguage {
    Lua,     // Native support
    Groovy,  // Auto-transpiled
    Kotlin,  // Auto-transpiled (limited)
}
```

Language detection from file extension:
- `.lua` → Lua
- `.groovy` → Groovy
- `.kts`, `.kt` → Kotlin

## WorkspaceHandle

The `WorkspaceHandle` (`api.rs`) provides thread-safe access to the workspace from Lua:

```rust
pub struct WorkspaceHandle {
    inner: Arc<Mutex<Workspace>>,
}
```

**Design decisions:**

1. **Cloning strategy**: The workspace is cloned before script execution to isolate changes
2. **Arc<Mutex<>>**: Allows safe mutable access from the Lua runtime
3. **UserData trait**: Implements `mlua::UserData` to expose methods to Lua

### Registered Methods

Methods are registered using `mlua::UserDataMethods`:

```rust
impl UserData for WorkspaceHandle {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        // Getters
        methods.add_method("getName", |_, this, ()| { ... });
        methods.add_method("getDescription", |_, this, ()| { ... });

        // Setters
        methods.add_method_mut("setName", |_, this, name: String| { ... });
        methods.add_method_mut("setDescription", |_, this, desc: String| { ... });

        // Element creation
        methods.add_method_mut("addPerson", |_, this, (name, desc)| { ... });
        methods.add_method_mut("addSoftwareSystem", |_, this, (name, desc)| { ... });
        // ...
    }
}
```

### Element Lookup

The `find_element_id_by_name` helper searches for elements:

```rust
fn find_element_id_by_name(ws: &Workspace, name: &str) -> Option<ElementId> {
    // Search people
    for person in &ws.model().people {
        if person.name() == name { return Some(person.id()); }
    }

    // Search software systems and their containers
    for system in &ws.model().software_systems {
        if system.name() == name { return Some(system.id()); }
        for container in &system.containers {
            if container.name() == name { return Some(container.id()); }
        }
    }

    None
}
```

## Lua Runtime Configuration

### StdLib Selection

The Lua runtime is configured based on sandbox settings:

```rust
fn create_sandboxed_lua(&self) -> Result<Lua> {
    // Base libraries (always included)
    let mut libs = StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::COROUTINE;

    // Conditional libraries
    if self.config.sandbox.allow_io { libs |= StdLib::IO; }
    if self.config.sandbox.allow_os { libs |= StdLib::OS; }
    if self.config.sandbox.allow_debug { libs |= StdLib::DEBUG; }
    if self.config.sandbox.allow_package { libs |= StdLib::PACKAGE; }

    let lua = Lua::new_with(libs, mlua::LuaOptions::default())?;
    // ...
}
```

### Memory Limits

Memory limits are enforced via mlua:

```rust
if self.config.sandbox.max_memory > 0 {
    lua.set_memory_limit(self.config.sandbox.max_memory)?;
}
```

### Globals Removal

Dangerous globals are removed for security:

```rust
if !self.config.sandbox.allow_require {
    globals.set("require", mlua::Value::Nil)?;
    globals.set("dofile", mlua::Value::Nil)?;
    globals.set("loadfile", mlua::Value::Nil)?;
}
```

## Helper Functions

Additional helper functions are registered for scripts:

```rust
pub fn register_helpers(lua: &Lua) -> LuaResult<()> {
    let globals = lua.globals();

    // print function with [Script] prefix
    let print_fn = lua.create_function(|_, args: mlua::Variadic<Value>| {
        let output: Vec<String> = args.iter().map(|v| format!("{:?}", v)).collect();
        println!("[Script] {}", output.join("\t"));
        Ok(())
    })?;
    globals.set("print", print_fn)?;

    // ipairs iterator
    // ...
}
```

## DSL Parser Integration

The parser (`structurizr-dsl`) integrates with the scripting engine:

```rust
// In parser.rs
fn parse_script_directive(&mut self) -> Result<()> {
    let language = self.expect_identifier()?; // "lua" or "groovy"

    // Inline script: !script lua { ... }
    if self.check(TokenKind::LBrace) {
        let script = self.read_script_block()?;
        self.execute_script(&script, &language)?;
    }
    // External file: !script "path/to/file.lua"
    else if let Some(path) = self.try_string()? {
        self.execute_script_file(&path)?;
    }

    Ok(())
}

fn execute_script(&mut self, script: &str, language: &str) -> Result<()> {
    #[cfg(feature = "scripting")]
    {
        let engine = ScriptEngine::with_defaults()?;
        let lang = ScriptLanguage::from_str(language)
            .ok_or_else(|| ParseError::UnsupportedLanguage(language.to_string()))?;
        engine.execute(&mut self.workspace, script, lang)?;
    }
    #[cfg(not(feature = "scripting"))]
    {
        // Skip script execution when feature is disabled
    }
    Ok(())
}
```

## Error Handling

Script errors are wrapped in `ScriptError`:

```rust
pub enum ScriptError {
    Lua(mlua::Error),           // Lua runtime errors
    Io(std::io::Error),         // File I/O errors
    FileNotFound(String),       // Missing script file
    UnsupportedLanguage(String), // Unknown language
    Transpilation(String),      // Groovy transpilation error
    Configuration(String),      // Invalid configuration
}
```

Error messages include context:

```
Error executing script: [string "script"]:3: attempt to call a nil value (method 'unknownMethod')
```

## Testing

Unit tests verify each component:

```rust
#[test]
fn test_execute_simple_lua() {
    let engine = ScriptEngine::with_defaults().unwrap();
    let mut workspace = Workspace::new("Test", "Test workspace");

    engine.execute_lua(&mut workspace, r#"
        workspace:setName("Modified by Lua")
    "#).unwrap();

    assert_eq!(workspace.name, "Modified by Lua");
}

#[test]
fn test_execute_add_elements() {
    let engine = ScriptEngine::with_defaults().unwrap();
    let mut workspace = Workspace::new("Test", "Test workspace");

    engine.execute_lua(&mut workspace, r#"
        workspace:addPerson("User", "A user of the system")
        workspace:addSoftwareSystem("Backend", "The backend system")
    "#).unwrap();

    assert_eq!(workspace.model().people.len(), 1);
    assert_eq!(workspace.model().software_systems.len(), 1);
}
```

## Performance Considerations

1. **Workspace cloning**: Full workspace is cloned before script execution
   - Ensures isolation but has memory overhead
   - Future optimization: Copy-on-write or partial cloning

2. **Lua state creation**: New Lua state per script execution
   - Clean slate for each script
   - Could cache state for repeated executions

3. **Transpilation**: Groovy scripts are transpiled on every execution
   - Could cache transpiled Lua for external files

## See Also

- [Scripting Guide](../features/scripting.md) - User documentation
- [Transpiler Implementation](transpiler-impl.md) - Groovy transpiler details
- [Sandbox Implementation](sandbox-impl.md) - Security configuration
