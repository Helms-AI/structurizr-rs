# Transpiler Implementation

This document describes the Groovy→Lua transpiler architecture in structurizr-rs.

## Overview

The `GroovyTranspiler` converts common Groovy patterns used in Structurizr scripts to equivalent Lua code. It achieves approximately 80% compatibility with typical Structurizr DSL scripts.

## Architecture

```rust
pub struct GroovyTranspiler {
    // Compiled regex patterns
    property_get: Regex,
    property_set: Regex,
    method_call: Regex,
    each_closure: Regex,
    string_interp: Regex,
    line_comment: Regex,
    block_comment: Regex,
    def_var: Regex,
    println: Regex,
}
```

The transpiler uses the `regex_lite` crate for pattern matching, providing a lightweight alternative to the full `regex` crate.

## Pattern Conversion Rules

### 1. Comments

**Line comments:**
```
Groovy: // comment
Lua:    -- comment
```

Pattern: `//(.*)$` → `--$1`

**Block comments:**
```
Groovy: /* multi-line */
Lua:    --[[ multi-line ]]
```

Handled by state machine (tracking `in_block_comment` flag).

### 2. Variable Declarations

**def to local:**
```
Groovy: def name = "value"
Lua:    local name = "value"
```

Pattern: `\bdef\s+(\w+)\s*=` → `local $1 =`

### 3. Print Statements

```
Groovy: println("message")
Lua:    print("message")
```

Pattern: `\bprintln\s*\(` → `print(`

### 4. Method Calls

Workspace methods are converted from dot to colon notation:

```
Groovy: workspace.addPerson("x")
Lua:    workspace:addPerson("x")
```

**Supported method patterns:**

| Groovy | Lua |
|--------|-----|
| `workspace.addPerson(` | `workspace:addPerson(` |
| `workspace.addSoftwareSystem(` | `workspace:addSoftwareSystem(` |
| `workspace.addContainer(` | `workspace:addContainer(` |
| `workspace.addComponent(` | `workspace:addComponent(` |
| `workspace.addRelationship(` | `workspace:addRelationship(` |
| `workspace.setName(` | `workspace:setName(` |
| `workspace.setDescription(` | `workspace:setDescription(` |
| `workspace.getName(` | `workspace:getName(` |
| `workspace.getDescription(` | `workspace:getDescription(` |
| `workspace.findElementByName(` | `workspace:findElementByName(` |
| `workspace.getPeople(` | `workspace:getPeople(` |
| `workspace.getSoftwareSystems(` | `workspace:getSoftwareSystems(` |

### 5. Property Access

**Property getters:**
```
Groovy: workspace.name
Lua:    workspace:getName()
```

Pattern: `workspace\.name\b` → `workspace:getName()`

**Property setters:**
```
Groovy: workspace.name = "value"
Lua:    workspace:setName("value")
```

Pattern: `workspace\.name\s*=\s*"([^"]*)"` → `workspace:setName("$1")`

**Order of operations:**
1. Convert setters first (to avoid matching assignment targets as getters)
2. Then convert remaining getters

### 6. Each Closures

**Basic iteration:**
```groovy
list.each { item ->
    println(item)
}
```
→
```lua
for _, item in ipairs(list) do
    print(item)
end
```

Pattern: `(\w+)\.each\s*\{\s*(\w+)\s*->\s*` → `for _, $2 in ipairs($1) do`

**Brace tracking:**
- Track brace depth to convert closing `}` to `end`
- Maintain stack of iteration variables

### 7. String Interpolation

```
Groovy: "Hello ${name}"
Lua:    "Hello " .. name .. ""
```

Pattern: `\$\{(\w+)\}` → `" .. $1 .. "`

## Implementation Details

### Transpile Method

```rust
pub fn transpile(&self, groovy: &str) -> Result<String> {
    let mut lua = String::new();
    let mut in_block_comment = false;
    let mut brace_depth = 0;
    let mut each_vars: Vec<String> = Vec::new();

    for line in groovy.lines() {
        // Handle block comments
        if in_block_comment { /* ... */ }

        // Convert the line
        let converted = self.transpile_line(line, &mut brace_depth, &mut each_vars)?;
        lua.push_str(&converted);
        lua.push('\n');
    }

    Ok(lua)
}
```

### Line-by-Line Processing

```rust
fn transpile_line(
    &self,
    line: &str,
    brace_depth: &mut i32,
    each_vars: &mut Vec<String>,
) -> Result<String> {
    let mut result = line.to_string();

    // 1. Convert line comments
    result = self.line_comment.replace_all(&result, "--$1").to_string();

    // 2. Convert println
    result = self.println.replace_all(&result, "print(").to_string();

    // 3. Convert def to local
    result = self.def_var.replace_all(&result, "local $1 =").to_string();

    // 4. Handle .each closures
    if let Some(caps) = self.each_closure.captures(&result) {
        // Convert to for loop
    }

    // 5. Handle closing braces
    if result.trim() == "}" && *brace_depth > 0 {
        result = "end".to_string();
        *brace_depth -= 1;
    }

    // 6. Convert string interpolation
    result = self.string_interp.replace_all(&result, /* ... */).to_string();

    // 7. Convert method calls
    result = self.convert_method_calls(&result);

    // 8. Convert property access
    result = self.convert_property_access(&result);

    Ok(result)
}
```

## Compatibility Checking

The transpiler can check scripts for unsupported features:

```rust
pub fn check_compatibility(&self, groovy: &str) -> Vec<String> {
    let mut issues = Vec::new();

    let unsupported = [
        (r"@\w+", "Annotations (@) are not supported"),
        (r"import\s+", "Import statements are not supported"),
        (r"class\s+\w+", "Class definitions are not supported"),
        (r"new\s+\w+\(", "Object instantiation (new) is not supported"),
        (r"\.metaClass", "MetaClass modifications are not supported"),
        (r"\.with\s*\{", ".with closures are not supported"),
        (r"switch\s*\(", "Switch statements need manual conversion"),
        (r"try\s*\{", "Try-catch blocks need manual conversion"),
    ];

    for (pattern, message) in unsupported {
        if let Ok(re) = Regex::new(pattern) {
            if re.is_match(groovy) {
                issues.push(message.to_string());
            }
        }
    }

    issues
}
```

## Unsupported Features

| Feature | Why | Migration Path |
|---------|-----|----------------|
| Annotations (`@Grab`) | Groovy-specific | Remove; not needed |
| Import statements | Java/Groovy packages | Remove; use built-in Lua |
| Class definitions | OOP feature | Convert to functions |
| `new` keyword | Object instantiation | Use Lua tables |
| MetaClass | Runtime modification | No equivalent |
| `.with` closures | Groovy DSL feature | Explicit method calls |
| Switch statements | Different syntax | Use if-elseif |
| Try-catch | Different syntax | Use pcall |

## Testing

```rust
#[test]
fn test_transpile_simple_assignment() {
    let transpiler = GroovyTranspiler::new();
    let groovy = r#"workspace.name = "Modified""#;
    let lua = transpiler.transpile(groovy).unwrap();
    assert!(lua.contains("workspace:setName(\"Modified\")"));
}

#[test]
fn test_transpile_method_call() {
    let transpiler = GroovyTranspiler::new();
    let groovy = r#"workspace.addPerson("Alice", "A user")"#;
    let lua = transpiler.transpile(groovy).unwrap();
    assert!(lua.contains("workspace:addPerson(\"Alice\", \"A user\")"));
}

#[test]
fn test_compatibility_check() {
    let transpiler = GroovyTranspiler::new();

    let groovy = r#"
        @Grab('something')
        import groovy.json.JsonSlurper
        class MyPlugin { }
    "#;
    let issues = transpiler.check_compatibility(groovy);
    assert!(!issues.is_empty());
}
```

## Limitations

1. **Single-pass conversion**: Complex nested structures may not convert correctly
2. **No semantic analysis**: Pattern-based only, no AST parsing
3. **Limited each support**: Only simple `.each { item -> }` patterns
4. **No type inference**: Groovy's dynamic typing not fully handled
5. **String interpolation**: Only simple `${var}` patterns, not expressions

## Future Improvements

1. **AST-based transpilation**: Parse Groovy into AST for better accuracy
2. **Expression support**: Handle `${expr}` in string interpolation
3. **Nested closures**: Support for nested iteration patterns
4. **Better error messages**: Line number tracking for conversion errors

## See Also

- [Groovy Migration Guide](../features/groovy-migration.md) - User documentation
- [Scripting Implementation](scripting-impl.md) - Engine architecture
