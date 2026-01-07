# DSL Error Handling Fix

## Problem

When a workspace contained DSL parsing errors, the web server would incorrectly report "Workspace not found" instead of showing the actual DSL parsing error. This made debugging DSL issues very difficult as users couldn't see what was wrong with their workspace files.

## Root Cause

The issue was in the `get_workspace_by_id` method in `/crates/structurizr-web/src/state.rs`. The method was using `.ok()` to convert `Result` to `Option`, which silently discarded any errors including DSL parsing errors:

```rust
// Original problematic code:
reg.get_workspace(id).await.ok().flatten()
```

When a DSL parsing error occurred:
1. `load_workspace` would return an error
2. `get_workspace` would propagate the error
3. `get_workspace_by_id` would convert the error to `None` via `.ok()`
4. Handlers would interpret `None` as "workspace not found"

## Solution

The fix involved two main changes:

### 1. Enhanced State Methods

Added a new method `try_get_workspace_by_id` that preserves error information:

```rust
pub async fn try_get_workspace_by_id(&self, id: &str) -> crate::Result<Option<Workspace>>
```

Modified the existing `get_workspace_by_id` to log errors before discarding them for backward compatibility.

### 2. Updated Critical Handlers

Updated key handlers to use the new method and properly propagate DSL errors:

- `workspace_home`
- `workspace_view_diagram`
- `workspace_get_json`
- `workspace_home_nested`
- Wildcard dispatcher (`workspace_wildcard`)

### 3. Improved Error Response

Updated the error handling to return HTTP 400 (Bad Request) for DSL errors instead of 500 (Internal Server Error).

## Impact

- **Before**: DSL errors showed as "Workspace not found" (HTTP 404)
- **After**: DSL errors show the actual parsing error with line/column info (HTTP 400)

Example error message now shown:
```
DSL error: Parse error: 9:29: Expected '=' or '->' after identifier
```

## Files Modified

1. `/crates/structurizr-web/src/state.rs` - Added `try_get_workspace_by_id` method
2. `/crates/structurizr-web/src/handlers.rs` - Updated critical handlers to use new method
3. `/crates/structurizr-web/src/error.rs` - Updated error response handling

## Testing

Created test workspace with intentional DSL error at `/workspaces/test-error/broken.dsl`.

Verified that:
1. DSL errors are properly shown in web server responses
2. Valid workspaces still load correctly
3. The `validate` command shows proper error messages
4. Backward compatibility is maintained for handlers not yet updated

## Future Improvements

While critical handlers have been updated, there are still other handlers using the old method that could benefit from showing proper error messages. These can be updated incrementally as needed.