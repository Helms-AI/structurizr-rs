# Workspace Validation

The structurizr-rs validation system provides comprehensive workspace inspection capabilities to help identify common issues and improve architecture documentation quality.

## Features

The validator checks for six types of issues:

### 1. Empty Descriptions (Warning)
Elements without descriptions make documentation less useful. The validator checks:
- People
- Software Systems
- Containers
- Components
- Deployment Nodes (Info level)

**Example Issue:**
```
[WARN] Person 'Admin' has no description
```

### 2. Orphan Elements (Info)
Elements with no relationships may indicate:
- Incomplete modeling
- Elements that should be removed
- Missing relationship definitions

**Example Issue:**
```
[INFO] Software System 'Legacy System' has no relationships (neither incoming nor outgoing)
```

### 3. Missing Technology (Warning/Info)
Containers and components should specify their technology stack:
- Containers without technology: **Warning**
- Components without technology: **Info**

**Example Issue:**
```
[WARN] Container 'Web App' in system 'Main System' has no technology specified
```

### 4. Duplicate Names (Error)
Elements with the same name at the same level can cause confusion:
- Duplicate people names
- Duplicate software system names
- Duplicate container names within a system
- Duplicate component names within a container

**Example Issue:**
```
[ERROR] Duplicate person name 'User' (conflicts with element ID b89bd287-62c7-482e-ba48-5dcc16aefe24)
```

### 5. Circular Relationships (Warning)
Direct self-references (A→A) are usually modeling errors.

**Example Issue:**
```
[WARN] Circular relationship: 'External System' has a relationship to itself
```

### 6. Unused Elements (Info)
Elements not included in any view might be:
- Work in progress
- Legacy elements to be removed
- Missing view definitions

**Example Issue:**
```
[INFO] Container 'Database' in system 'Main System' is not included in any view
```

## Usage

### CLI Command

Validate a DSL file:

```bash
structurizr validate workspace.dsl
```

Example output:
```
Validating workspace.dsl...

✓ DSL Parsing: Valid workspace: My System
  People: 2
  Software Systems: 3
  Relationships: 5
  Views: 2

Running workspace inspections...

Issues found:
  Errors: 1, Warnings: 3, Info: 8

  [ERROR] Duplicate person name 'User' (conflicts with element ID ...)
  [WARN] Person 'Admin' has no description [Admin]
  [WARN] Container 'Web App' has no technology specified [Web App]
  [INFO] Person 'Admin' has no relationships [Admin]
  ...

✗ Validation failed with 1 error(s)
```

Exit codes:
- `0`: Validation passed (no errors, may have warnings/info)
- `1`: Validation failed (errors found or DSL parse error)

### API Endpoint

Validate the loaded workspace via HTTP:

```bash
curl http://localhost:8080/api/validate
```

Response (JSON):
```json
{
  "issues": [
    {
      "severity": "warning",
      "message": "Container 'Web App' in system 'System' has no technology specified",
      "element_id": "df333b8a-b0ea-467e-b260-f1c26aaf4fcc",
      "element_name": "Web App",
      "check_type": "missing_technology"
    }
  ],
  "error_count": 0,
  "warning_count": 1,
  "info_count": 2
}
```

### Programmatic Usage

Use the validation API in your Rust code:

```rust
use structurizr_dsl::{validate_workspace, ValidationConfig};
use structurizr_core::Workspace;

// Validate with default configuration
let workspace = Workspace::new("My System", "Description");
let result = validate_workspace(&workspace);

println!("Errors: {}", result.error_count);
println!("Warnings: {}", result.warning_count);
println!("Info: {}", result.info_count);

for issue in &result.issues {
    println!("[{}] {}", issue.severity, issue.message);
    if let Some(name) = &issue.element_name {
        println!("  Element: {}", name);
    }
}

// Validate with custom configuration
let config = ValidationConfig {
    check_empty_descriptions: true,
    check_orphan_elements: false,  // Disable this check
    check_missing_technology: true,
    check_duplicate_names: true,
    check_circular_relationships: true,
    check_unused_elements: false,  // Disable this check
};

let result = validate_workspace_with_config(&workspace, &config);
```

## Severity Levels

- **Error**: Critical issues that should be fixed immediately (e.g., duplicate names)
- **Warning**: Issues that should be addressed but don't prevent usage (e.g., missing descriptions, missing technology)
- **Info**: Informational suggestions for improvement (e.g., orphan elements, unused elements)

## Configuration

Validation checks can be enabled or disabled individually:

```rust
use structurizr_dsl::ValidationConfig;

// All checks enabled (default)
let config = ValidationConfig::all();

// All checks disabled
let config = ValidationConfig::none();

// Custom configuration
let config = ValidationConfig {
    check_empty_descriptions: true,
    check_orphan_elements: true,
    check_missing_technology: true,
    check_duplicate_names: true,
    check_circular_relationships: true,
    check_unused_elements: true,
};
```

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
name: Validate Architecture

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install structurizr-rs
        run: cargo install --path .
      - name: Validate workspace
        run: structurizr validate workspace.dsl
```

## Best Practices

1. **Run validation regularly**: Integrate validation into your development workflow
2. **Fix errors immediately**: Duplicate names and other errors can cause confusion
3. **Address warnings**: Missing descriptions and technology reduce documentation quality
4. **Review info items**: Orphan and unused elements may indicate incomplete modeling
5. **Use in CI/CD**: Automate validation to catch issues early

## Implementation Details

The validation system is implemented in the `structurizr-dsl` crate:

- **Module**: `crates/structurizr-dsl/src/validation.rs`
- **API**: `validate_workspace()` and `validate_workspace_with_config()`
- **CLI**: `src/main.rs` - `validate` command
- **Web API**: `crates/structurizr-web/src/handlers.rs` - `/api/validate` endpoint

Each check is implemented as a separate function that examines the workspace model and views, adding issues to the validation result as they are found.
