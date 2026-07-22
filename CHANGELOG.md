# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
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
