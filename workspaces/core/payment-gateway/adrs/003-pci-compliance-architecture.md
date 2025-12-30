# ADR-003: PCI-DSS Level 1 Compliance Architecture

## Status
Accepted

## Context
FreshMart's Payment Gateway processes over 5 million transactions daily, exceeding 6 million annual Visa/Mastercard transactions, which mandates PCI-DSS Level 1 compliance. This is the most stringent compliance level requiring:

- Annual on-site assessment by Qualified Security Assessor (QSA)
- Quarterly network scans by Approved Scanning Vendor (ASV)
- Penetration testing twice annually
- Compliance with all 12 PCI-DSS requirements and 300+ sub-requirements

Key challenges include:
- **Distributed Infrastructure**: 25,000+ POS terminals across store locations
- **Cloud Deployment**: AWS-hosted payment processing with multiple services
- **Multi-Acquirer Integration**: Direct connections to Visa, Mastercard, Amex, and alternative payment providers
- **High Availability Requirements**: 99.999% uptime target with zero data loss
- **Regulatory Complexity**: Overlapping requirements with SOC 2, ISO 27001, and regional regulations

The current architecture lacks clear security zone boundaries, making it difficult to scope assessments and implement defense-in-depth controls.

## Decision
We will implement a segmented network architecture with clearly defined security zones, comprehensive encryption, and tamper-evident audit logging.

### Network Segmentation Strategy

#### Security Zones
1. **Cardholder Data Environment (CDE)** - PCI Zone
   - Tokenization Vault (HashiCorp Vault with CloudHSM)
   - Key Management Service
   - Payment Database (encrypted PAN storage during migration)
   - Acquirer Gateway (processes card data in transit)

2. **Secure Processing Zone**
   - Payment Orchestrator (handles tokenized data only)
   - Fraud Detection Engine (analyzes transaction patterns)
   - Settlement Engine (batch processing)

3. **Application Zone**
   - API Gateway (rate limiting, authentication)
   - Event Stream (Kafka for tokenized events)
   - Monitoring Service
   - Compliance Service

4. **Edge Zone**
   - POS Terminals (P2PE encrypted)
   - Store network equipment

#### Network Controls
- **Micro-segmentation**: AWS Security Groups and Network ACLs enforce least-privilege communication
- **Private Subnets**: CDE deployed in isolated private subnets with no direct internet access
- **VPC Peering**: Controlled connections between security zones with explicit allow-lists
- **PrivateLink**: AWS PrivateLink for all AWS service access (no internet routing)

### Encryption Architecture

#### Data at Rest
| Component | Encryption | Key Management |
|-----------|------------|----------------|
| Payment Database | TDE (AES-256) | AWS KMS with CMK |
| Tokenization Vault | AES-256-GCM | CloudHSM |
| Event Stream | Server-side encryption | AWS MSK managed keys |
| Cache (Redis) | Redis encryption at rest | AWS ElastiCache managed |
| Audit Logs | AES-256 | Dedicated KMS key |

#### Data in Transit
- **External Connections**: TLS 1.3 exclusively with strong cipher suites
- **Internal Services**: mTLS with certificate rotation every 90 days
- **POS to Gateway**: Point-to-Point Encryption (P2PE) with DUKPT key management
- **Acquirer Connections**: ISO 8583 over hardware-encrypted VPN tunnels

#### Key Hierarchy
```
CloudHSM Master Key (FIPS 140-2 Level 3)
├── Tokenization KEK
│   └── Token DEKs (monthly rotation)
├── Database KEK
│   └── TDE keys (annual rotation)
├── TLS CA Key
│   └── Service certificates (90-day rotation)
└── Audit Signing Key
    └── Log integrity verification
```

### Audit Logging Architecture

#### Log Collection
- **Audit Logger Component**: Captures all payment activities with tamper-proof storage
- **Retention**: 7 years for compliance (configurable per jurisdiction)
- **Integrity**: SHA-256 hash chain with CloudHSM-signed anchors
- **Real-time Streaming**: Critical events streamed to SIEM within 60 seconds

#### Logged Events
| Category | Events | Retention |
|----------|--------|-----------|
| Authentication | Login attempts, MFA events, session changes | 1 year |
| Authorization | Access grants, permission changes, role modifications | 7 years |
| Cardholder Data | Token access, detokenization requests, PAN exposure | 7 years |
| Administrative | Configuration changes, key rotation, system updates | 7 years |
| Network | Firewall changes, VPN connections, security group modifications | 1 year |

#### Access Controls
- **Role-Based Access**: Principle of least privilege across all systems
- **Multi-Party Authorization**: Detokenization requires approval from two authorized personnel
- **Privileged Access Management**: Just-in-time access for administrative functions
- **Session Recording**: All CDE access sessions recorded and retained

## Consequences

### Positive
- **Clear Compliance Scope**: Well-defined CDE reduces assessment complexity by 60%
- **Defense in Depth**: Multiple security layers prevent single-point compromise
- **Audit Readiness**: Comprehensive logging enables rapid response to QSA requests
- **Breach Containment**: Segmentation limits lateral movement in case of compromise
- **Regulatory Alignment**: Architecture satisfies PCI-DSS, SOC 2, and ISO 27001 requirements
- **Insurance Benefits**: Demonstrable controls reduce cyber insurance premiums

### Negative
- **Infrastructure Complexity**: Multiple security zones increase operational overhead
- **Performance Impact**: Encryption and logging add 10-15ms latency per transaction
- **Cost Overhead**: Estimated 25% increase in infrastructure costs for security controls
- **Developer Friction**: Strict access controls slow development and debugging
- **Vendor Lock-in**: Heavy reliance on AWS security services
- **Maintenance Burden**: Certificate rotation, key management, and security patching

### Mitigation
- Implement Infrastructure as Code (Terraform) for consistent security zone deployment
- Use automated certificate management (AWS ACM, cert-manager)
- Deploy log aggregation with search capabilities for efficient audit response
- Create developer sandbox environments with synthetic data outside CDE
- Maintain multi-cloud disaster recovery capability to reduce vendor dependency
- Automate quarterly ASV scans and penetration testing scheduling

## Implementation

### Phase 1: Network Segmentation (Weeks 1-6)
1. Design and document security zone architecture
2. Create VPC and subnet structure for CDE isolation
3. Implement Security Groups and Network ACLs
4. Deploy AWS PrivateLink endpoints for service access
5. Configure VPN tunnels for acquirer connections
6. Validate network isolation with penetration testing

### Phase 2: Encryption Deployment (Weeks 7-12)
1. Deploy CloudHSM cluster with key hierarchy
2. Enable TDE on Payment Database with KMS integration
3. Configure mTLS across all internal services
4. Implement P2PE for POS terminal communications
5. Enable encryption at rest for all data stores
6. Document key rotation procedures and test recovery

### Phase 3: Audit Infrastructure (Weeks 13-18)
1. Deploy centralized Audit Logger component
2. Implement hash chain integrity verification
3. Configure SIEM integration with alert rules
4. Create compliance dashboards for 12 PCI requirements
5. Establish 7-year log retention with lifecycle policies
6. Train SOC team on audit log investigation procedures

### Phase 4: Access Control Hardening (Weeks 19-24)
1. Implement role-based access across all CDE systems
2. Deploy Privileged Access Management solution
3. Configure multi-party authorization workflows
4. Enable session recording for administrative access
5. Conduct access review and remove excessive permissions
6. Document emergency access procedures

### Phase 5: Validation & Certification (Weeks 25-30)
1. Engage QSA for pre-assessment gap analysis
2. Remediate identified gaps
3. Complete ASV quarterly scan validation
4. Execute annual penetration test
5. Submit Report on Compliance (ROC) to payment brands
6. Establish continuous compliance monitoring

### Ongoing Operations
- **Daily**: PCI Scanner automated compliance checks
- **Weekly**: Access review for CDE systems
- **Monthly**: Key rotation verification
- **Quarterly**: ASV network scans, access certification
- **Annually**: QSA assessment, penetration testing, policy review

## References
- [PCI-DSS v4.0 Requirements](https://www.pcisecuritystandards.org/document_library/)
- [AWS PCI-DSS Compliance Guide](https://docs.aws.amazon.com/whitepapers/latest/pci-dss-scoping-on-aws/pci-dss-scoping-on-aws.html)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [CIS AWS Foundations Benchmark](https://www.cisecurity.org/benchmark/amazon_web_services)
- [FreshMart Information Security Policy](https://wiki.freshmart.com/security-policy)
- [CloudHSM Compliance Documentation](https://aws.amazon.com/cloudhsm/compliance/)
