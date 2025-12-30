# Supply Chain Integration Platform Documentation

## System Overview

The FreshMart Supply Chain Platform is a comprehensive B2B integration platform that manages supplier relationships, procurement operations, demand forecasting, and logistics optimization across FreshMart's grocery retail network.

## Architecture Documentation

- [Architecture Overview](architecture.md) - System design and component interactions
- [Integration Guide](integration.md) - Supplier and carrier integration specifications
- [Operations Runbook](runbook.md) - Operational procedures and incident response

## Key Capabilities

### EDI Gateway
- Multi-standard EDI support including ANSI X12 and EDIFACT
- Transaction types: 850 (Purchase Order), 855 (Acknowledgment), 856 (Ship Notice), 810 (Invoice)
- 200+ trading partner configurations via Sterling B2B Integrator
- AS2 protocol support for secure document exchange
- 500+ translation maps for partner-specific formats

### Demand Forecasting
- ML-based demand prediction using Prophet and LSTM ensemble models
- 92% forecast accuracy at SKU-Store-Day granularity
- 12-week planning horizon with daily refresh cycles
- Promotion lift modeling with 85% accuracy via XGBoost
- External factor integration including weather, holidays, and local events

### Automated Procurement
- 85% automation rate for purchase order generation
- Multi-level approval workflow via Camunda with 24-hour SLA
- Intelligent supplier selection based on price, quality, lead time, and risk scoring
- Contract management for 1,000+ active supplier agreements
- Real-time reorder point triggers from inventory events

### Supplier Portal
- Self-service interface for 2,000+ supplier users
- Order management, invoice submission, and catalog maintenance
- Performance scorecard visibility and certification tracking
- React + Node.js technology stack with OAuth 2.0 authentication

### Logistics Management
- 100,000 shipment trackings per day across 50+ carriers
- Vehicle routing optimization via Google OR-Tools (15% fuel cost reduction)
- Real-time shipment visibility with 15-minute update intervals
- Cold chain monitoring for temperature-sensitive products
- Dock appointment scheduling in 15-minute slots

### Warehouse Operations
- Manhattan WMS integration across 5 distribution centers (10M sq ft total)
- Velocity-based putaway algorithms for optimal storage
- Wave, batch, and zone picking methods
- Automated labor scheduling and workforce management

## Integration Guide

### For Suppliers (EDI Integration)
Connect via AS2 protocol with the following transaction support:
```
Outbound from FreshMart:
- EDI 850 (Purchase Order)
- EDI 860 (Purchase Order Change)

Inbound to FreshMart:
- EDI 855 (Purchase Order Acknowledgment)
- EDI 856 (Advance Ship Notice)
- EDI 810 (Invoice)
```

### For Suppliers (REST API)
```http
POST /api/v1/orders/acknowledge
Authorization: Bearer {oauth_token}
Content-Type: application/json

{
  "purchase_order_id": "PO-2024-001234",
  "acknowledgment_type": "ACCEPT",
  "estimated_ship_date": "2024-01-15",
  "line_items": [
    {
      "line_number": 1,
      "sku": "SKU-12345",
      "quantity_confirmed": 500,
      "unit_price": 12.99
    }
  ]
}
```

### For Carriers (Tracking Integration)
```http
POST /api/v1/shipments/tracking
Authorization: Bearer {oauth_token}
Content-Type: application/json

{
  "shipment_id": "SHP-2024-056789",
  "carrier_tracking_number": "1Z999AA10123456784",
  "status": "IN_TRANSIT",
  "location": {
    "city": "Chicago",
    "state": "IL",
    "country": "US"
  },
  "estimated_delivery": "2024-01-16T14:00:00Z",
  "temperature_celsius": 4.2
}
```

### Event Streaming
Subscribe to supply chain events via Apache Kafka:
- `inventory.levels` - Real-time stock level changes
- `orders.created` - New purchase order creation
- `orders.acknowledged` - Supplier acknowledgment received
- `shipments.status` - Shipment tracking updates
- `receipts.completed` - Warehouse receiving completed
- `forecasts.updated` - Demand forecast refresh

## Performance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Active Suppliers | 500+ | 523 |
| Daily Purchase Orders | 50,000 | 52,400 |
| Forecast Accuracy | 90% | 92% |
| PO Automation Rate | 80% | 85% |
| Shipments Tracked Daily | 100,000 | 98,500 |
| EDI Transaction Success | 99.5% | 99.7% |
| API Response Time (p95) | <500ms | 380ms |
| Inventory Cost Reduction | 15% | 20% |

## Support

- **Supply Chain Operations**: +1-555-SUPPLY (Mon-Fri 6AM-8PM)
- **24/7 EDI Support**: edi-support@freshmart.com
- **Supplier Portal Help**: supplier-help@freshmart.com
- **Carrier Integration**: carrier-support@freshmart.com
- **Slack Channel**: #supply-chain-platform
- **Wiki**: https://wiki.freshmart.com/supply-chain
