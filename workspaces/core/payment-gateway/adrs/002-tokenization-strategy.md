# ADR-002: HSM-Backed Tokenization Strategy

## Status
Accepted

## Context
FreshMart's Payment Gateway processes 5M+ transactions daily, handling sensitive cardholder data including Primary Account Numbers (PANs), expiration dates, and CVV codes. We face several critical requirements:

- **PCI-DSS Compliance**: Minimize cardholder data exposure and reduce PCI scope across systems
- **Recurring Payments**: Enable subscription services and stored payment methods without retaining raw card data
- **Analytics & Reporting**: Support business intelligence on payment patterns without exposing sensitive data
- **Multi-Channel Support**: Provide consistent tokenization across POS terminals, mobile apps, and e-commerce
- **Fraud Prevention**: Enable transaction linking and velocity checks across tokenized payments

The current approach of encrypting card data at rest still exposes systems to cardholder data, requiring extensive PCI controls across the entire payment infrastructure.

## Decision
We will implement an HSM-backed tokenization vault using HashiCorp Vault with AWS CloudHSM integration. The solution includes:

### Token Generation Strategy
1. **Format-Preserving Encryption (FPE)**: Tokens retain the format of original PANs (16 digits, valid Luhn check) enabling integration with legacy systems that validate card number formats
2. **256-bit Entropy**: Token generation uses cryptographically secure random values with 256-bit entropy to prevent token prediction attacks
3. **Deterministic Mapping**: Same PAN always produces same token (per merchant) enabling recurring payments and customer identification

### Storage Architecture
1. **Separation of Concerns**: Token-to-PAN mappings stored in dedicated Vault cluster isolated from application databases
2. **Encrypted Index**: Token lookups use encrypted indexes preventing enumeration attacks
3. **Geographic Restrictions**: Token mappings geo-fenced to comply with data residency requirements

### Key Management
1. **HSM-Protected Master Keys**: All encryption keys stored in FIPS 140-2 Level 3 certified CloudHSM
2. **Key Hierarchy**: Master Key (HSM) -> Key Encryption Keys (KEK) -> Data Encryption Keys (DEK)
3. **Automated Rotation**: DEKs rotated monthly with zero-downtime key migration
4. **Key Derivation**: PBKDF2 with 100,000 iterations for derived keys

### Token Lifecycle
- **Creation**: Token generated on first card presentation, stored with encrypted PAN
- **Retrieval**: Applications request detokenization for authorized payment processing only
- **Deletion**: Token and mapping removed after 7 years or upon customer request (GDPR compliance)
- **Suspension**: Tokens can be temporarily suspended for fraud investigation

## Consequences

### Positive
- **PCI Scope Reduction**: 80% reduction in systems requiring PCI-DSS compliance
- **Security Enhancement**: Raw card data never persists in application databases
- **Recurring Payment Support**: Tokens enable stored payment methods without card-on-file risk
- **Analytics Enablement**: Tokenized data supports fraud detection and business analytics
- **Breach Impact Mitigation**: Compromised tokens useless without vault access
- **Regulatory Compliance**: Meets PCI-DSS, GDPR, and CCPA requirements for data minimization

### Negative
- **Additional Latency**: 15-30ms overhead for tokenization/detokenization operations
- **Infrastructure Complexity**: Requires dedicated Vault cluster and HSM integration
- **Operational Overhead**: Key rotation, HSM management, and vault monitoring
- **Cost**: CloudHSM cluster costs approximately $1.60/hour per HSM instance
- **Single Point of Dependency**: Vault availability critical for all payment operations
- **Migration Complexity**: Existing encrypted card data requires migration to tokenized format

### Mitigation
- Deploy Vault in multi-AZ configuration with automatic failover
- Implement token caching for frequently-used tokens (1-hour TTL)
- Maintain standby HSM cluster for disaster recovery
- Create offline token generation capability for catastrophic vault failure
- Implement gradual migration strategy for existing card data

## Implementation

### Phase 1: Infrastructure Setup (Weeks 1-4)
1. Deploy AWS CloudHSM cluster in PCI-compliant VPC
2. Configure HashiCorp Vault Enterprise cluster with HSM backend
3. Establish mTLS communication between Payment Orchestrator and Vault
4. Implement monitoring and alerting for Vault health metrics

### Phase 2: Token Service Development (Weeks 5-8)
1. Develop Token Generator component with FPE algorithm (AES-FF1)
2. Implement Token Mapper for encrypted storage of token-PAN mappings
3. Build Key Manager service for automated key rotation
4. Create audit logging for all tokenization operations

### Phase 3: Integration (Weeks 9-12)
1. Integrate tokenization into Payment Orchestrator workflow
2. Update Fraud Engine to work with tokenized data
3. Modify Settlement Engine for detokenization during settlement
4. Update POS Terminal integration for token-based recurring payments

### Phase 4: Migration & Validation (Weeks 13-16)
1. Migrate existing encrypted card data to tokenized format
2. Validate token consistency across all payment channels
3. Conduct penetration testing on tokenization infrastructure
4. Complete PCI-DSS assessment for scope reduction validation

### Operational Procedures
- **Key Rotation**: Automated monthly rotation with 7-day overlap period
- **HSM Backup**: Daily backup of HSM partition to secure offline storage
- **Vault Backup**: Continuous backup with point-in-time recovery capability
- **Access Control**: Role-based access with mandatory multi-party authorization for detokenization

## References
- [PCI-DSS Tokenization Guidelines](https://www.pcisecuritystandards.org/documents/Tokenization_Guidelines_Info_Supplement.pdf)
- [HashiCorp Vault Enterprise HSM Integration](https://developer.hashicorp.com/vault/docs/enterprise/hsm)
- [NIST SP 800-38G: Format-Preserving Encryption](https://csrc.nist.gov/publications/detail/sp/800-38g/final)
- [AWS CloudHSM Best Practices](https://docs.aws.amazon.com/cloudhsm/latest/userguide/best-practices.html)
- [FreshMart Security Architecture](https://wiki.freshmart.com/security-architecture)
