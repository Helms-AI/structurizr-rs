# Online Banking Platform

A secure online banking platform demonstrating **multiple dynamic diagrams** for authentication and transaction workflows.

## Overview

This example models a modern digital banking platform with:
- Multi-factor authentication (MFA)
- Real-time fraud detection
- Secure fund transfers
- Comprehensive audit logging

## Dynamic Diagram Features

This example showcases **two dynamic diagrams**:

### LoginFlow (8 steps)
The customer authentication flow with MFA:
1. Customer enters credentials
2. Credentials submitted to API Gateway
3. Auth Service validates credentials
4. User account verified in database
5. OTP sent to registered device
6. Customer enters OTP
7. OTP validated
8. Session created in cache

### TransferFlow (10 steps)
The fund transfer workflow:
1. Customer initiates transfer
2. Request submitted to API Gateway
3. Session validated
4. Pending transaction created
5. Fraud check performed
6. Balance validated
7. Account data read
8. Transfer executed in core banking
9. Audit log recorded
10. Confirmation sent to customer

## Running the Example

```bash
./serve.sh
```

Navigate to the **LoginFlow** or **TransferFlow** dynamic views.

## Architecture Highlights

- **Security-first design** - MFA, fraud detection, audit logging
- **External integrations** - OTP provider, fraud detection, core banking
- **Microservices** - Auth, Account, Transaction, Notification services
- **Defense in depth** - Multiple security checkpoints

## DSL Features Demonstrated

- Multiple dynamic views in one workspace
- External system integrations
- Constants (`!const`) for reusable values
- Complex tag-based styling
- Various container shapes (WebBrowser, MobileDevicePortrait, Hexagon, Cylinder)
