# Wallet UX follow-up tickets

## WUX-001 — Separate lock from erase and add a real unlock session

**Priority:** P0 before a custody-oriented desktop release.

`WalletContext::lock` currently clears persisted wallet data. Implement distinct operations:

- lock: retain encrypted data, end keystore session, clear decrypted material from memory;
- unlock: prompt for the vault passphrase and restore an encrypted local wallet;
- erase: require a typed confirmation, remove local data, and explain recovery requirements.

Include inactivity/session-expiry UI, lock status in the application header, and tests proving that locking cannot erase assets or account metadata.

## WUX-002 — Native credential-store and signer integration

**Priority:** P0 before distributing a desktop wallet with signing enabled.

Move desktop secret storage to the operating system credential store (Keychain, Credential Manager, Secret Service) and define the fallback security posture. Add hardware/external signer support and a signer-selection UI; never imply support where a typed signing port is unavailable.

## WUX-003 — Full mobile information architecture

**Priority:** P1 for mobile release.

Replace the desktop sidebar/header composition with a mobile-native navigation model, 44px targets, one-handed transaction approval, compact network/account selection, and responsive transaction review. Validate on iOS and Android assistive technologies.

## WUX-004 — Transaction review and phishing-resistant signing

**Priority:** P0 before users can sign value-bearing transactions.

Create a dedicated pre-signing review that shows the signer, chain/network, recipient, asset and amount, fees, approvals/permissions, warnings for unknown contracts, and an explicit final confirmation. Add simulation/error states and clear explorer links after submission.

## WUX-005 — Complete accessibility verification

**Priority:** P1.

Perform keyboard and screen-reader testing for all dialogs, dropdowns, routes, and form errors. Implement focus trapping/return, Escape handling, roving keyboard behavior for custom selects/tabs, and WCAG 2.2 contrast/target-size checks. The current semantic improvements are not a replacement for device-level validation.

## WUX-006 — WASM/web threat-model and release hardening

**Priority:** P0 before public web deployment.

Threat-model browser storage, extension injection, clipboard leakage, Content Security Policy, supply-chain integrity, phishing-resistant origin display, and recovery/export flows. Establish a separate supported-feature matrix for native desktop and browser/WASM builds.

## WUX-007 — Platform-native encrypted backup import and export

**Priority:** P0 before desktop backup recovery is advertised.

The present backup UI uses browser download and `FileReader` APIs. Provide desktop save/open dialogs, restrictive default file permissions, overwrite confirmation, cancellation/error states, and platform-specific implementations behind a shared interface. Keep the browser flow feature-gated so desktop actions cannot invoke Web APIs.
