# Phase 2 Enterprise Documentation

This example demonstrates the enterprise boundary and configuration features implemented in Phase 2.

## Features Demonstrated

### Enterprise Block

The `enterprise` block defines your organizational boundary. Elements defined inside are automatically tagged as "Internal":

```dsl
enterprise "TechCorp Financial Services" {
    trader = person "Trader" "..."
    tradingPlatform = softwareSystem "Trading Platform" "..."
}
```

Elements defined **outside** the enterprise block are tagged as "External":

```dsl
client = person "Client" "..." "External"
exchange = softwareSystem "Stock Exchange" "..." "External"
```

### Configuration Block

The `configuration` block sets workspace-level options:

```dsl
configuration {
    scope softwaresystem
    visibility public

    terminology {
        person "Actor"
        softwareSystem "Application"
        enterprise "Organization"
    }
}
```

#### Configuration Options

| Option | Description |
|--------|-------------|
| `scope` | Default diagram scope: `softwaresystem`, `container`, `component` |
| `visibility` | Access control: `public`, `private` |
| `terminology` | Custom labels for C4 element types |

### Custom Terminology

Override default C4 terminology for domain-specific language:

- `person` -> "Actor"
- `softwareSystem` -> "Application"
- `container` -> "Module"
- `component` -> "Service"
- `enterprise` -> "Organization"

## Architecture Overview

### Internal Systems (TechCorp)

1. **Trading Platform** - Core order management and execution
2. **Risk Management System** - Real-time risk monitoring and VaR calculations
3. **Compliance System** - Regulatory reporting and audit trails

### External Entities

1. **Stock Exchange** - Primary trading venue
2. **Market Data Vendor** - Bloomberg/Reuters feeds
3. **Clearing House** - Trade settlement
4. **Clients** - Institutional and retail customers
5. **Regulators** - SEC, FINRA, etc.
