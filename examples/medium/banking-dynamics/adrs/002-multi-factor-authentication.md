# ADR-002: Multi-Factor Authentication

## Status

Accepted

## Context

Password-only authentication is vulnerable to:
- Phishing attacks
- Credential stuffing
- Brute force attacks

Financial institutions require stronger authentication for customer protection.

## Decision

We will implement **multi-factor authentication (MFA)** using:

1. **First factor** - Username and password
2. **Second factor** - One-time password (OTP) via SMS or email

The OTP provider is an external service for reliability and deliverability.

## Consequences

### Positive

- **Stronger security** - Compromised passwords insufficient for access
- **Industry standard** - Meets banking security requirements
- **Customer trust** - Visible security measure
- **Flexible** - OTP provider can be swapped

### Negative

- **User friction** - Extra step in login flow
- **Delivery issues** - SMS/email can be delayed
- **External dependency** - OTP provider availability critical
- **Support burden** - Users may need help with OTP issues

## Notes

The LoginFlow dynamic diagram shows the complete MFA sequence.
