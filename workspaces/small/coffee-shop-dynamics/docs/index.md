# Coffee Shop Ordering System

## Overview

This example demonstrates **dynamic diagrams** - a powerful Structurizr feature for visualizing workflows and sequences of interactions over time.

## Business Context

**Bean & Brew** is a small artisan coffee shop serving approximately 100 customers daily. They need a simple system to:

- Allow customers to place orders via touch-screen terminals
- Display pending orders on a queue screen for baristas
- Track ingredient inventory to prevent out-of-stock situations

## Available Views

### System Context
Shows the coffee shop system and its users (customers and baristas).

### Container Diagram
Shows the internal architecture:
- **POS Terminal** - Customer-facing ordering interface
- **Order Queue Service** - Manages order lifecycle
- **Inventory Service** - Tracks ingredient stock
- **Database** - Persistent storage

### Dynamic: OrderFlow
**The main feature of this example!**

Shows the step-by-step sequence when a customer places an order:

| Step | From | To | Description |
|------|------|-----|-------------|
| 1 | Customer | POS Terminal | Selects drink and customizations |
| 2 | POS Terminal | Order Queue | Submits order with details |
| 3 | Order Queue | Inventory | Checks ingredient availability |
| 4 | Inventory | Database | Reads current stock levels |
| 5 | Inventory | Order Queue | Confirms ingredients available |
| 6 | Order Queue | Database | Saves order with pending status |
| 7 | Order Queue | Barista | Displays order on queue screen |

## Running This Example

```bash
cd workspaces/small/coffee-shop-dynamics
./serve.sh
```

Navigate to the **OrderFlow** view to see the animated dynamic diagram.

## DSL Features Demonstrated

- `dynamic` view type with step sequences
- Arrow syntax: `source -> destination "description"`
- Numbered steps for clear ordering
- `autoLayout lr` for horizontal layout
