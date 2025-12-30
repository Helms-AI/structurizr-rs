# ADR-002: Plugin Architecture for Extensible Checkout

## Status
Accepted

## Context
FreshMart's checkout process must support a wide variety of business requirements that vary by:
- Store format (grocery, pharmacy, convenience)
- Geographic region (tax rules, age verification laws)
- Promotional campaigns (seasonal, partner-specific)
- Third-party integrations (delivery services, loyalty partners)

Hardcoding all these variations into the core POS engine would result in:
- Bloated codebase with excessive conditional logic
- Slow release cycles due to testing complexity
- Inability to customize per-store or per-region
- Risk to core transaction processing stability

We need an architecture that allows extending checkout functionality without modifying the core engine.

## Decision
We will implement a plugin architecture using WebAssembly (WASM) as the plugin format, with a sandboxed execution environment and well-defined lifecycle hooks.

### Plugin System Components

1. **Plugin Loader**: Loads WASM plugins from the Plugin Registry and initializes them in the sandbox.

2. **Plugin Sandbox**: WASM runtime providing memory, CPU, and I/O isolation to prevent plugins from affecting core system stability.

3. **Plugin API**: Rust traits that define the contract between plugins and the core engine, exposed via WASM bindings.

4. **Plugin Registry**: SQLite-backed configuration store that manages plugin enablement, configuration, and versioning per terminal.

### Lifecycle Hooks

Plugins can register for the following hooks:

| Hook | Trigger Point | Use Cases |
|------|---------------|-----------|
| `pre_scan` | Before item added to transaction | Age verification, restricted item checks |
| `post_scan` | After item added to transaction | Loyalty offers, upsell prompts |
| `pre_tender` | Before payment processing | Coupon validation, spend limits |
| `post_tender` | After payment completed | Receipt customization, survey prompts |
| `transaction_complete` | After transaction finalized | Analytics, external system sync |

### Plugin Interface

```rust
/// Core trait all plugins must implement
pub trait PosPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&mut self, config: PluginConfig) -> Result<()>;
    fn shutdown(&mut self) -> Result<()>;
}

/// Optional hook traits
pub trait PreScanHook {
    fn on_pre_scan(&self, item: &Item, transaction: &Transaction) -> HookResult;
}

pub trait PostTenderHook {
    fn on_post_tender(&self, tender: &Tender, transaction: &Transaction) -> HookResult;
}
```

### HookResult Enum

```rust
pub enum HookResult {
    Continue,                           // Proceed normally
    RequireVerification(VerificationType), // Prompt for verification
    ModifyItem(ItemModification),       // Apply discount, override
    BlockWithReason(String),            // Prevent action with message
    DisplayMessage(String),             // Show message to cashier
}
```

## Consequences

### Positive
- 50+ plugins already developed for various use cases
- Hot-reload capability enables zero-downtime updates
- Regional teams can develop store-specific extensions
- Third-party integrations isolated from core stability
- A/B testing of new features via gradual plugin rollout
- Clear separation of concerns improves maintainability

### Negative
- WASM adds ~35ms overhead per plugin execution
- Plugin developers require Rust/WASM expertise
- Debugging sandboxed plugins more challenging
- Plugin compatibility testing matrix grows with each release
- Memory overhead for WASM runtime (~50MB per terminal)

### Mitigation
- Plugin execution budget (50ms max per hook)
- Comprehensive plugin development SDK with examples
- Remote debugging capability via Store Controller
- Automated compatibility testing in CI pipeline
- Memory-efficient plugin pooling across hooks

## Implementation

1. **Plugin SDK**
   - Rust crate with macros for hook registration
   - WASM compilation toolchain integration
   - Local testing harness for development

2. **Plugin Distribution**
   - Central plugin repository with versioning
   - Signed plugins with certificate verification
   - Per-store plugin configuration via Store Operations

3. **Sandbox Configuration**
   - Memory limit: 64MB per plugin
   - CPU time limit: 50ms per hook invocation
   - I/O: Read-only access to specific directories
   - Network: No direct network access (use Plugin API)

4. **Monitoring**
   - Plugin execution time metrics
   - Error rate per plugin
   - Memory usage tracking
   - Automatic disable on repeated failures

## Example Plugins

| Plugin | Hook | Description |
|--------|------|-------------|
| Age Verification | pre_scan | Prompts for ID check on alcohol/tobacco |
| Loyalty Integration | post_scan | Displays member points and offers |
| Coupon Validator | pre_tender | Validates and applies digital coupons |
| Survey Prompt | post_tender | Displays customer satisfaction survey |
| WIC/EBT Handler | pre_scan | Validates WIC-eligible items |
| Pharmacy Pickup | post_tender | Triggers pharmacy notification |

## References
- [Plugin Manager Components](../docs/index.md)
- [WASM Plugin Development Guide](https://wiki.freshmart.com/wasm-plugins)
- [Plugin Security Review Process](https://wiki.freshmart.com/plugin-security)
