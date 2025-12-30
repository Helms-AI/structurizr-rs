# ADR-001: React Native + PWA Hybrid Approach

## Status
Accepted

## Context
FreshMart requires a mobile presence that serves customers across iOS, Android, and web browsers. Key requirements include:
- Consistent user experience across all platforms
- Access to native device features (camera for barcode scanning, push notifications, biometric authentication)
- Offline functionality for unreliable network conditions
- Cost-effective development with limited mobile engineering resources
- Fast time-to-market for new features across all platforms

We evaluated three options:
1. **Separate native apps** - Swift/Kotlin for iOS/Android plus React web app
2. **Pure PWA** - Single Progressive Web App for all platforms
3. **Hybrid approach** - React Native for native apps plus PWA for web

## Decision
We will implement a hybrid approach using React Native for iOS and Android native apps, combined with a Progressive Web App built with React for web browsers.

### Architecture

**Shared Code Strategy:**
- Core business logic (cart management, sync engine, API clients) in TypeScript shared across platforms
- UI components built platform-specific but following shared design system
- Redux state management with RTK Query shared across React Native and PWA

**Native Bridge Architecture:**
- Camera access via `react-native-camera` for barcode scanning
- Biometric authentication via `react-native-biometrics` (Face ID, Touch ID, Fingerprint)
- Push notifications via `react-native-firebase` (FCM) and `@react-native-community/push-notification-ios` (APNs)
- Geolocation via `react-native-geolocation-service` for store locator
- Secure storage via `react-native-keychain` for authentication tokens

**PWA Capabilities:**
- Workbox for service worker management and caching strategies
- Web Push API for browser notifications where supported
- IndexedDB via Dexie.js for offline data storage
- Web Geolocation API for store locator functionality
- Installable as home screen app (manifest.json)

### Cross-Platform Code Sharing

```
mobile-experience/
├── packages/
│   ├── core/                    # Shared TypeScript (70% of codebase)
│   │   ├── api/                 # GraphQL clients
│   │   ├── state/               # Redux slices
│   │   ├── sync/                # Offline sync logic
│   │   └── utils/               # Business logic
│   ├── native-app/              # React Native (20% of codebase)
│   │   ├── components/          # Native UI components
│   │   └── bridges/             # Native module wrappers
│   └── pwa/                     # React PWA (10% of codebase)
│       ├── components/          # Web UI components
│       └── workers/             # Service workers
```

## Consequences

### Positive
- 70% code sharing between native apps and PWA reduces development effort
- Single TypeScript codebase for business logic minimizes divergence
- Full access to native device capabilities on mobile apps
- PWA provides reach to users who prefer not to install apps
- React Native and React expertise transferable across platforms
- Faster feature delivery with shared state management and API layer

### Negative
- React Native requires native module updates for iOS/Android version changes
- Performance on complex animations may require platform-specific optimization
- PWA has limited access to native features compared to installed apps
- Three deployment targets (iOS App Store, Google Play, Web hosting) to maintain
- Team requires knowledge of both React Native and React web ecosystems

### Mitigation
- Establish clear guidelines for platform-specific vs shared code decisions
- Invest in automated testing across all platforms with Detox (native) and Cypress (web)
- Use feature flags via LaunchDarkly to gate features by platform capability
- Maintain native expertise through dedicated mobile specialists on the team
- Implement analytics to track feature usage and performance by platform

## Implementation
1. Set up monorepo with Yarn workspaces for shared packages
2. Create React Native app shell with navigation and native bridge modules
3. Build PWA with Workbox service worker and installable manifest
4. Implement shared Redux store with RTK Query for API layer
5. Develop platform abstraction layer for device features
6. Set up CI/CD pipelines for all three deployment targets
7. Launch beta to 5% of users on each platform for validation

## References
- [React Native Documentation](https://reactnative.dev/docs/getting-started)
- [PWA Best Practices](https://web.dev/progressive-web-apps/)
- [Workbox Documentation](https://developer.chrome.com/docs/workbox/)
- [FreshMart Mobile Design System](https://wiki.freshmart.com/mobile-design-system)
