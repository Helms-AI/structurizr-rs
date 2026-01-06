# Sandbox Implementation

This document describes the security and sandboxing architecture for script execution in structurizr-rs.

## Overview

Scripts run in a sandboxed Lua environment with configurable security restrictions. The sandbox prevents malicious or resource-intensive scripts from affecting the host system.

## SandboxConfig

```rust
pub struct SandboxConfig {
    /// Maximum execution time before timeout (default: 5 seconds)
    pub timeout: Duration,

    /// Maximum memory usage in bytes (default: 10MB)
    pub max_memory: usize,

    /// Maximum number of instructions to execute (default: 1_000_000)
    pub max_instructions: u64,

    /// Allow file system access (default: false)
    pub allow_filesystem: bool,

    /// Allow network access (default: false)
    pub allow_network: bool,

    /// Allow loading external Lua modules (default: false)
    pub allow_require: bool,

    /// Allow os library functions (default: false)
    pub allow_os: bool,

    /// Allow io library functions (default: false)
    pub allow_io: bool,

    /// Allow debug library functions (default: false)
    pub allow_debug: bool,

    /// Allow package library functions (default: false)
    pub allow_package: bool,

    /// Custom allowed globals (beyond workspace API)
    pub allowed_globals: Vec<String>,
}
```

## Default Configuration

```rust
impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_memory: 10 * 1024 * 1024, // 10MB
            max_instructions: 1_000_000,
            allow_filesystem: false,
            allow_network: false,
            allow_require: false,
            allow_os: false,
            allow_io: false,
            allow_debug: false,
            allow_package: false,
            allowed_globals: Vec::new(),
        }
    }
}
```

## Permissive Configuration

For trusted scripts, a more permissive configuration is available:

```rust
impl SandboxConfig {
    pub fn permissive() -> Self {
        Self {
            timeout: Duration::from_secs(60),
            max_memory: 100 * 1024 * 1024, // 100MB
            max_instructions: 100_000_000,
            allow_filesystem: true,
            allow_network: false, // Still no network
            allow_require: true,
            allow_os: false,      // Still no OS commands
            allow_io: true,
            allow_debug: false,
            allow_package: true,
            allowed_globals: Vec::new(),
        }
    }
}
```

## Security Layers

### 1. Lua Standard Library Filtering

Scripts only have access to a subset of Lua's standard library:

| Library | Default | Permissive | Description |
|---------|---------|------------|-------------|
| `math` | Yes | Yes | Mathematical functions |
| `string` | Yes | Yes | String manipulation |
| `table` | Yes | Yes | Table utilities |
| `coroutine` | Yes | Yes | Coroutine functions |
| `io` | No | Yes | File I/O |
| `os` | No | No | Operating system functions |
| `debug` | No | No | Debug library |
| `package` | No | Yes | Module loading |

```rust
fn create_sandboxed_lua(&self) -> Result<Lua> {
    let mut libs = StdLib::MATH | StdLib::STRING | StdLib::TABLE | StdLib::COROUTINE;

    if self.config.sandbox.allow_io { libs |= StdLib::IO; }
    if self.config.sandbox.allow_os { libs |= StdLib::OS; }
    if self.config.sandbox.allow_debug { libs |= StdLib::DEBUG; }
    if self.config.sandbox.allow_package { libs |= StdLib::PACKAGE; }

    let lua = Lua::new_with(libs, mlua::LuaOptions::default())?;
    // ...
}
```

### 2. Dangerous Globals Removal

Even when libraries are loaded, dangerous functions are removed:

```rust
if !self.config.sandbox.allow_require {
    globals.set("require", mlua::Value::Nil)?;
    globals.set("dofile", mlua::Value::Nil)?;
    globals.set("loadfile", mlua::Value::Nil)?;
}
```

**Removed by default:**
- `require` - Module loading
- `dofile` - Execute Lua file
- `loadfile` - Load Lua file
- `load` - Load string as code (when debug disabled)
- `loadstring` - Load string as code

### 3. Memory Limits

Memory usage is constrained using mlua's built-in limits:

```rust
if self.config.sandbox.max_memory > 0 {
    lua.set_memory_limit(self.config.sandbox.max_memory)?;
}
```

When the limit is exceeded, the script terminates with a memory error.

### 4. Instruction Counting

Execution time is controlled through instruction counting:

```rust
// max_instructions controls how many Lua VM instructions can execute
pub max_instructions: u64,  // Default: 1_000_000
```

This prevents infinite loops and ensures scripts complete in reasonable time.

### 5. Network Isolation

Network access is always disabled:

```rust
pub allow_network: bool,  // Always false in both configs
```

Scripts cannot make HTTP requests, open sockets, or access external services.

## Builder Pattern

The sandbox configuration uses a builder pattern for customization:

```rust
impl SandboxConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_max_memory(mut self, bytes: usize) -> Self {
        self.max_memory = bytes;
        self
    }

    pub fn with_filesystem(mut self, allow: bool) -> Self {
        self.allow_filesystem = allow;
        self
    }

    pub fn with_require(mut self, allow: bool) -> Self {
        self.allow_require = allow;
        self
    }

    pub fn with_allowed_global(mut self, name: impl Into<String>) -> Self {
        self.allowed_globals.push(name.into());
        self
    }
}
```

**Example usage:**

```rust
let sandbox = SandboxConfig::new()
    .with_timeout(Duration::from_secs(30))
    .with_max_memory(50 * 1024 * 1024)
    .with_filesystem(true);

let config = ScriptConfig::default().with_sandbox(sandbox);
let engine = ScriptEngine::new(config)?;
```

## Security Considerations

### What Scripts CAN Do

- Read/modify the workspace being parsed
- Perform mathematical calculations
- Manipulate strings and tables
- Create local variables and functions
- Call the registered workspace API
- Print debug output

### What Scripts CANNOT Do (by default)

- Access the filesystem
- Make network requests
- Load external Lua modules
- Execute system commands
- Access process environment
- Spawn child processes
- Modify global Lua state

### Potential Risks

1. **Resource exhaustion**: Scripts can still consume CPU within limits
2. **Memory spikes**: Large allocations may cause temporary memory pressure
3. **Stack overflow**: Deep recursion could exhaust stack (mitigated by instruction limits)

### Mitigation Strategies

1. **Conservative defaults**: All dangerous features disabled by default
2. **Explicit opt-in**: Users must explicitly enable risky features
3. **Documentation**: Clear warnings about permissive mode risks
4. **Monitoring**: Memory and instruction limits catch runaway scripts

## Testing

```rust
#[test]
fn test_default_config() {
    let config = SandboxConfig::default();
    assert_eq!(config.timeout, Duration::from_secs(5));
    assert_eq!(config.max_memory, 10 * 1024 * 1024);
    assert!(!config.allow_filesystem);
    assert!(!config.allow_network);
}

#[test]
fn test_permissive_config() {
    let config = SandboxConfig::permissive();
    assert_eq!(config.timeout, Duration::from_secs(60));
    assert!(config.allow_filesystem);
    assert!(!config.allow_network); // Still disabled
}

#[test]
fn test_builder_pattern() {
    let config = SandboxConfig::new()
        .with_timeout(Duration::from_secs(10))
        .with_max_memory(50 * 1024 * 1024)
        .with_filesystem(true);

    assert_eq!(config.timeout, Duration::from_secs(10));
    assert_eq!(config.max_memory, 50 * 1024 * 1024);
    assert!(config.allow_filesystem);
}
```

## Comparison with Other Systems

| Feature | structurizr-rs | Node.js vm | Python restricted | Lua standalone |
|---------|---------------|------------|-------------------|----------------|
| Memory limits | Yes | Partial | No | No |
| Instruction limits | Yes | No | No | No |
| Library filtering | Yes | No | Partial | No |
| Globals removal | Yes | Yes | Yes | Manual |
| Network isolation | Yes | No | No | No |

## Future Enhancements

1. **Fine-grained permissions**: Per-function allow lists
2. **Capability tokens**: Request elevated permissions at runtime
3. **Audit logging**: Record what sandboxed code attempts to do
4. **Resource accounting**: Track CPU, memory, I/O per script

## See Also

- [Scripting Implementation](scripting-impl.md) - Engine architecture
- [Plugin System Implementation](plugin-system-impl.md) - WASM sandboxing
