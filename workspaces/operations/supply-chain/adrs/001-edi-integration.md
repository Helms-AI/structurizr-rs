# ADR-001: EDI Integration Strategy with Apache Camel

## Status
Accepted

## Context
FreshMart sources products from 500+ suppliers ranging from large CPG manufacturers with sophisticated EDI systems to small local producers using only email and spreadsheets. We need a strategy to:
- Support industry-standard EDI transaction sets (850, 855, 856, 810)
- Enable both EDI and non-EDI suppliers to integrate seamlessly
- Ensure compliance with ANSI X12 and EDIFACT standards
- Handle high transaction volumes (50,000+ orders per day)
- Maintain 7-year document retention for audit requirements

## Decision
We will implement a comprehensive EDI integration platform using Sterling B2B Integrator with Apache Camel-based translation services:

### 1. EDI Standards Support
- **ANSI X12**: Primary standard for North American suppliers
- **EDIFACT**: Support for international suppliers
- **Transaction Sets**:
  - EDI 850 (Purchase Order) - Outbound to suppliers
  - EDI 855 (PO Acknowledgment) - Inbound confirmation
  - EDI 856 (Advance Ship Notice) - Shipment notification
  - EDI 810 (Invoice) - Supplier invoice
  - EDI 860 (PO Change) - Order modifications

### 2. Translation Architecture
- Sterling Mapper for complex EDI-to-JSON transformations
- 500+ pre-built translation maps for partner-specific formats
- Apache Camel routes for message orchestration and routing
- Schema validation against X12 standards before processing

### 3. Communication Protocols
- **AS2**: Primary protocol for secure B2B document exchange
- **SFTP**: Alternative for partners unable to support AS2
- **VAN (Value Added Network)**: Legacy support via TrueCommerce

### 4. Trading Partner Onboarding Process
1. Partner profile creation in Sterling Partner Manager
2. EDI map development and testing (2-3 weeks typical)
3. Pilot transaction testing with production-like data
4. Certification sign-off with compliance checklist
5. Production go-live with monitoring enabled

### 5. Non-EDI Partner Support
- Supplier Portal web interface for manual order management
- CSV/Excel upload capability for bulk operations
- Email notification workflows for order alerts
- Gradual migration path from portal to EDI

## Consequences

### Positive
- Standardized integration with 200+ EDI trading partners
- 99.7% transaction success rate through validation
- Reduced order cycle time from 48 hours to 4 hours
- Complete audit trail with 7-year retention in MongoDB
- Scalable to 100,000+ daily transactions

### Negative
- High initial investment in Sterling B2B licensing
- 2-3 week onboarding timeline for new EDI partners
- Ongoing maintenance of partner-specific translation maps
- Specialized EDI expertise required for support team

### Mitigation
- Leverage Sterling's pre-built map templates to accelerate onboarding
- Implement self-service portal for smaller suppliers
- Create comprehensive partner documentation and test harness
- Train dedicated EDI support team with rotation schedule

## Implementation
1. Deploy Sterling B2B Integrator in HA configuration
2. Configure AS2 certificates and trading partner profiles
3. Develop core transaction maps for each document type
4. Implement Kafka integration for event publishing
5. Create monitoring dashboards for transaction visibility
6. Establish partner onboarding playbook and SLAs

## References
- [Sterling B2B Integrator Documentation](https://wiki.freshmart.com/sterling-b2b)
- [ANSI X12 Transaction Set Reference](https://wiki.freshmart.com/x12-standards)
- [Trading Partner Onboarding Guide](https://wiki.freshmart.com/edi-onboarding)
- [EDI Compliance Checklist](https://wiki.freshmart.com/edi-compliance)
