# Online Banking Platform

## Overview

This example demonstrates **multiple dynamic diagrams** for visualizing critical banking workflows with security checkpoints.

## Business Context

**SecureBank** is a regional bank serving 500,000 customers. They require:

- Secure multi-factor authentication for all logins
- Real-time fraud detection on transactions
- Comprehensive audit trails for compliance
- Integration with legacy core banking systems

## Available Views

### System Context
Shows the banking platform and external dependencies:
- Fraud Detection Service
- OTP Provider
- Core Banking System

### Container Diagram
Shows the internal architecture with 10 containers:
- **Web Application** - React customer portal
- **Mobile App** - React Native banking app
- **API Gateway** - Kong for routing and rate limiting
- **Auth Service** - Spring Security for authentication
- **Account Service** - Account management
- **Transaction Service** - Transfer processing
- **Notification Service** - Alerts and confirmations
- **Audit Log** - Immutable transaction records
- **Database** - Core banking data
- **Session Cache** - Redis for sessions

### Dynamic: LoginFlow
Shows the 8-step MFA authentication sequence:

| Step | Action |
|------|--------|
| 1 | Customer enters credentials |
| 2 | Web app submits to API Gateway |
| 3 | Gateway routes to Auth Service |
| 4 | Auth Service verifies in database |
| 5 | OTP sent to customer device |
| 6 | Customer enters OTP |
| 7 | Auth Service validates OTP |
| 8 | Session created in Redis cache |

### Dynamic: TransferFlow
Shows the 10-step fund transfer sequence:

| Step | Action |
|------|--------|
| 1 | Customer initiates transfer |
| 2 | Request goes to API Gateway |
| 3 | Session token validated |
| 4 | Pending transaction created |
| 5 | Fraud detection check |
| 6 | Balance validation |
| 7 | Account data retrieved |
| 8 | Core banking executes transfer |
| 9 | Audit log recorded |
| 10 | Confirmation sent to customer |

## Running This Example

```bash
cd examples/medium/banking-dynamics
./serve.sh
```

## Security Architecture

The system implements **defense in depth**:
1. **Authentication** - MFA with OTP
2. **Authorization** - Session-based access control
3. **Fraud Detection** - ML-based pattern analysis
4. **Audit Trail** - Immutable transaction logging
5. **Encryption** - HTTPS for all communications

## DSL Features Demonstrated

- Multiple `dynamic` views in one workspace
- `!const` for reusable constants
- External system styling with tags
- Complex container relationships
- Various shapes for different container types
