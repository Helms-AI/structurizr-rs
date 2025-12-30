# POS Terminal System Documentation

## System Overview

The FreshMart POS Terminal System is a next-generation point of sale platform that powers the checkout experience across all FreshMart retail locations. Built with an edge-first architecture and plugin extensibility, the system processes over 5 million transactions daily through 25,000+ terminals while maintaining sub-30-second average transaction times.

## Architecture Documentation

- [Offline-First Architecture](../adrs/001-offline-first-architecture.md) - Queue-and-sync for network resilience
- [Plugin Architecture](../adrs/002-plugin-architecture.md) - Extensible checkout via WASM plugins
- [Edge Deployment](../adrs/003-edge-deployment.md) - Store-level K3s deployment strategy

## Key Capabilities

### Transaction Processing
- High-performance Rust-based core engine with <10ms processing time
- Complete transaction lifecycle management: Open, Suspended, Completed, Voided
- Multi-tender support: Cash, Card, Gift Cards, EBT
- Tax calculation across multiple jurisdictions (State, County, City)
- Real-time pricing with BOGO, percentage, dollar-off, and mix-match promotions

### Hardware Integration
- Unified Hardware Abstraction Layer supporting multiple device types
- Barcode scanner integration (1D, 2D, QR codes via USB/Serial)
- NTEP-certified weight scale support with 0.01 lb precision
- ESC/POS and Star thermal receipt printer protocols
- EMV-compliant payment terminals with P2PE certification
- Cash drawer control integration

### Offline-First Architecture
- SQLite + RocksDB local storage for complete offline capability
- Queue capacity of 10,000 transactions during network outages
- Store-and-forward payment processing with encrypted SAF
- CRDT-based conflict resolution (LWW-Register, OR-Set)
- Bidirectional sync with LZ4 compression via WebSocket + REST

### Plugin Architecture
- 50+ available plugins for extended functionality
- WASM-based sandbox with memory, CPU, and I/O isolation
- Hot-reload capability for zero-downtime updates
- Lifecycle hooks: Pre-scan, Post-scan, Pre-tender, Post-tender
- Third-party integration support for custom extensions

## Integration Guide

### Starting a Transaction
```rust
// Initialize transaction via POS Core Engine
let transaction = TransactionManager::new()
    .with_cashier(cashier_id)
    .with_lane(lane_id)
    .open();

// Process scanned item
let item = ItemProcessor::process_barcode("0123456789012")?;
let priced_item = PricingCalculator::calculate(item)?;
let taxed_item = TaxEngine::apply_taxes(priced_item)?;
transaction.add_item(taxed_item);
```

### Processing Payment
```rust
// Accept card payment
let tender = TenderManager::new(TenderType::Card)
    .with_amount(transaction.total())
    .process()?;

// The system automatically:
// - Encrypts PAN via EncryptionService
// - Routes to Payment Gateway
// - Falls back to store-and-forward if offline
// - Generates receipt via ReceiptGenerator
```

### Plugin Development
```rust
// Implement the PluginAPI trait
#[wasm_bindgen]
impl PreScanHook for AgeVerificationPlugin {
    fn on_pre_scan(&self, item: &Item) -> HookResult {
        if item.requires_age_verification() {
            HookResult::RequireVerification(VerificationType::Age21)
        } else {
            HookResult::Continue
        }
    }
}
```

### Event Streaming
Subscribe to POS events via Kafka:
- `pos.transaction.completed` - Transaction finalized
- `pos.transaction.voided` - Transaction voided
- `pos.inventory.update` - Inventory adjustment
- `pos.hardware.alert` - Device status change
- `pos.sync.completed` - Cloud sync finished

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Transaction Processing | <10ms | 8ms |
| End-to-End Transaction | <30s | 24s |
| Daily Transaction Volume | 5M | 5.2M |
| System Availability | 99.99% | 99.99% |
| Offline Queue Capacity | 10,000 txn | 10,000 txn |
| Payment Auth Latency | <500ms | 420ms |
| Plugin Execution | <50ms | 35ms |
| Sync Throughput | 1,000 txn/s | 1,200 txn/s |

## Security & Compliance

- **PCI-DSS & PA-DSS** compliant infrastructure
- **AES-256-GCM** encryption for data at rest and in transit
- **RSA-2048** for key exchange
- **P2PE certified** payment terminal integration
- **Tamper-proof** audit logging with 7-year retention
- Multi-factor authentication: PIN, Badge, Biometric

## Deployment Architecture

The POS Terminal System uses an edge deployment model:

**Per Store:**
- Store Server running Store Controller (active-passive HA)
- 10-20 POS Lanes each running the full POS stack

**Per Lane:**
- POS User Interface (React Native on Windows/Android)
- POS Core Engine (Rust)
- Plugin Manager (Rust + WASM Runtime)
- Hardware Manager (Rust)
- Offline Engine (SQLite + RocksDB)
- Security Manager (Rust)

## Support

- **Store IT Helpdesk**: 1-800-FRESH-IT (ext. 4)
- **Escalation**: pos-support@freshmart.com
- **24/7 Operations**: +1-555-POS-HELP
- **Slack Channel**: #pos-terminal-support
- **Wiki**: https://wiki.freshmart.com/pos-terminal
- **Training Portal**: https://learn.freshmart.com/pos
