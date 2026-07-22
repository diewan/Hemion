# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **HEM-01** — the chain layer is now a first-class **Anchoring** capability in
  the primary console navigation, no longer quarantined under "Legacy wallet".
  A new `services::anchoring` module projects the selectable networks from the
  canonical Parwana chain specs (`parwana/chains/*.toml`, embedded at build time
  so the list is identical on native and the wasm web bundle and needs no
  filesystem), de-duplicated by canonical `chain_id`. The `/anchoring` page lets
  a developer select a network — showing its real read RPC endpoint and finality
  profile — and run "Anchor this bundle" / "Verify anchor". Because the on-chain
  commitment/finality protocol backing is **ANCHOR-01** and not yet wired, both
  actions resolve to an explicit `AnchorAvailability::Unavailable` that names the
  blocking ticket; there is deliberately no success arm that can be reached
  without a real adapter, so finality is never fabricated. Legacy wallet routes
  are untouched. Native unit tests cover the network projection and the
  unavailable capability matrix, and the module compiles on `wasm32-unknown-unknown`.
- **DEMO-03** — the dispute inspector now surfaces the independent assurance
  verdict. Paste a bundle plus a hash-bound verification context and run the
  pinned Parwana verifier locally (`import_and_verify`); the assurance dimensions
  render with their reason codes and limitations, so an agent overreach shows as
  a failed **Authority** dimension (`ACCOUNTABILITY.AUTHORITY.INTENT_MISMATCH`).
  The context is imported separately, so a bundle can never choose its own trust
  inputs.
- Initial release

### Changed
- (nothing yet)

### Fixed
- **wasm32 web build restored.** The `dioxus` dependency unconditionally enabled
  the `desktop` renderer, which pulled `wry`/`tao` (GTK/WebKit) plus a native-only
  stack — `tungstenite` → `native-tls`/`openssl`, `image` → `ravif`/`rav1e`, and
  `rand 0.9` → `getrandom 0.3` (whose `wasm32-unknown-unknown` `compile_error!`
  was the visible symptom). The renderer is now target-split: `dioxus/web` for
  `cfg(target_arch = "wasm32")`, `dioxus/desktop` for native, mirroring the
  existing `csv-sdk` split. `secp256k1` and the rest of the crypto stack compile
  to wasm unchanged (no K256 substitution needed). `dx build --platform web` now
  produces the servable bundle the Hemion container serves.
- The explorer's live-feed polling loop used `tokio::time::sleep`, which has no
  wasm timer driver. Replaced with a cross-platform `services::platform::sleep`
  (tokio on native, `gloo-timers` on wasm).
