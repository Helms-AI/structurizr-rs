# ADR-013: Authentication Provider Migration (Auth0 to Keycloak)

## Status

**Accepted**

## Date

2024-12-31

## Context

The Horizon Platform (formerly Replit Clone) requires a robust identity and access management (IAM) solution. The original architecture specified Auth0 as the identity provider, which is a cloud-hosted SaaS solution.

### Requirements

1. OAuth 2.0 and OpenID Connect (OIDC) support
2. Single Sign-On (SSO) capabilities
3. Multi-factor authentication (MFA)
4. Social login integration (GitHub, Google, GitLab)
5. User federation and directory integration
6. Fine-grained access control
7. API authentication and authorization

### Constraints

1. Prefer open-source, self-hosted solutions to reduce vendor lock-in
2. Must support Kubernetes deployment
3. Need to maintain compatibility with existing authentication flows
4. Cost considerations for scaling

## Decision

We will migrate from Auth0 to **Keycloak** as our identity provider.

### Why Keycloak?

| Criteria | Auth0 | Keycloak |
|----------|-------|----------|
| **License** | Proprietary (paid) | Apache 2.0 (free) |
| **Hosting** | Cloud-only | Self-hosted or cloud |
| **OAuth 2.0/OIDC** | ✅ | ✅ |
| **SSO** | ✅ | ✅ |
| **MFA** | ✅ | ✅ |
| **Social Login** | ✅ | ✅ |
| **User Federation** | Limited | Full LDAP/AD support |
| **Kubernetes Native** | ❌ | ✅ (Operator available) |
| **Cost at Scale** | Per-user pricing | Infrastructure cost only |
| **Vendor Lock-in** | High | None |

### Alternatives Considered

1. **Ory Stack (Kratos + Hydra)**: Modern, cloud-native, but requires more integration work
2. **Authentik**: Good UI, but less mature than Keycloak
3. **Authelia**: Lightweight, but less feature-complete
4. **FusionAuth**: Similar to Auth0 but with self-hosted option

## Implementation

### Keycloak Configuration

```yaml
# Kubernetes Deployment
apiVersion: apps/v1
kind: Deployment
metadata:
  name: keycloak
  namespace: horizon
spec:
  replicas: 2
  template:
    spec:
      containers:
        - name: keycloak
          image: quay.io/keycloak/keycloak:latest
          args: ["start", "--optimized"]
          env:
            - name: KC_DB
              value: postgres
            - name: KC_DB_URL
              value: jdbc:postgresql://postgres:5432/keycloak
            - name: KC_HOSTNAME
              value: auth.horizonplatform.io
```

### Realm Configuration

```json
{
  "realm": "horizon",
  "enabled": true,
  "sslRequired": "external",
  "registrationAllowed": true,
  "loginWithEmailAllowed": true,
  "duplicateEmailsAllowed": false,
  "resetPasswordAllowed": true,
  "editUsernameAllowed": false,
  "bruteForceProtected": true,
  "clients": [
    {
      "clientId": "horizon-web",
      "protocol": "openid-connect",
      "publicClient": true,
      "redirectUris": ["https://horizonplatform.io/*"],
      "webOrigins": ["https://horizonplatform.io"]
    },
    {
      "clientId": "horizon-api",
      "protocol": "openid-connect",
      "publicClient": false,
      "serviceAccountsEnabled": true
    }
  ],
  "identityProviders": [
    {
      "alias": "github",
      "providerId": "github",
      "enabled": true
    },
    {
      "alias": "google",
      "providerId": "google",
      "enabled": true
    }
  ]
}
```

### Token Validation

```python
# API Gateway token validation
from keycloak import KeycloakOpenID

keycloak_openid = KeycloakOpenID(
    server_url="https://auth.horizonplatform.io/",
    client_id="horizon-api",
    realm_name="horizon",
    client_secret_key="your-secret"
)

def validate_token(token: str) -> dict:
    """Validate JWT token with Keycloak."""
    return keycloak_openid.decode_token(
        token,
        key=keycloak_openid.public_key(),
        options={"verify_signature": True, "verify_aud": True}
    )
```

## Consequences

### Positive

1. **No vendor lock-in**: Full control over identity infrastructure
2. **Cost savings**: No per-user pricing at scale
3. **Self-hosted**: Data sovereignty and compliance
4. **Kubernetes native**: Easy deployment with Helm or Operator
5. **Feature-rich**: Enterprise-grade IAM out of the box
6. **Active community**: Strong open-source ecosystem

### Negative

1. **Operational overhead**: Need to manage Keycloak infrastructure
2. **Learning curve**: Team needs to learn Keycloak administration
3. **Migration effort**: Existing Auth0 configurations need migration
4. **High availability**: Need to configure clustering for production

### Mitigations

| Risk | Mitigation |
|------|------------|
| Operational complexity | Use Keycloak Operator for Kubernetes |
| Learning curve | Comprehensive documentation and training |
| Migration effort | Phased migration with parallel running |
| High availability | Multi-replica deployment with PostgreSQL backend |

## References

- [Keycloak Documentation](https://www.keycloak.org/documentation)
- [Keycloak Operator](https://www.keycloak.org/operator/installation)
- [OAuth 2.0 Specification](https://oauth.net/2/)
- [OpenID Connect](https://openid.net/connect/)
