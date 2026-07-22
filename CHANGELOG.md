# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- **HEM-05** — a portfolio-of-mandates home and a deliberate light+dark design
  system. The default route is now `PortfolioHome`, which groups mandates by
  accountable entity and state with the anchored-vs-buffered split per tile; cards
  are projected from **real** Piteka mandate-chain reads (new pure
  `services::portfolio` with `MandateCard::from_chain`, entity grouping, and
  portfolio counts) — nothing is simulated, and with none loaded the home shows an
  explicit empty state. The developer console moves to `/console` and stays
  reachable; the local-verifier boundary becomes a badge rather than the whole
  home. The instrument tokens gain a light theme (served on
  `prefers-color-scheme: light`, with `:root[data-theme]` overrides winning either
  way); both themes clear the WCAG-AA text matrix, now covered by a light-theme
  contrast test in `tests/console_shell.rs`. The wallet is retained but no longer
  framed as a second-class "legacy" tool in navigation (routes unchanged).
- **HEM-04** — a universal accountability search and lineage graph. A new pure
  `services::search` classifies a query (typed `mandate:`/`receipt:`/`action:`/
  `dispute:`/`assurance:`/`anchor:` digests, `entity:`, `tx:`, or an
  `environments/<env>/receipts/<id>` path) and routes it to the correct object
  page — a bare digest resolves to an explicit *ambiguous* state with candidate
  kinds rather than guessing, and anything unrecognized is a clear *no-match*, so
  a query never routes to a wrong object. A new pure `services::lineage` models a
  directed graph walked in both directions with node-type filters and a gap as a
  first-class node (withheld/missing branches are never silently omitted; the
  model exposes no centrality). The `/search` page renders the resolver result, a
  node-type-filtered lineage graph, and a keyboard-accessible mirror table. Search
  is a first-class console nav item. Resolver/graph unit tests per identifier type
  and both-direction traversal; native + wasm32 build green.
- **HEM-03** — each accountability object type now has a deep-linkable object
  page at `/object/<kind>/<id>` (mandate, action, receipt, dispute, assurance,
  anchor). A new pure `services::object_model` names the kinds, their stable
  route slugs, and the cross-links that follow the evidence DAG, plus two
  disclosure helpers: `reason_code_display` preserves the exact stable namespaced
  identifier, and `FieldDisclosure` keeps withheld/redacted values protected (the
  value is never returned, and a withheld marker never claims the field is empty).
  The `ObjectPage` shell renders summary → canonical bytes → relationships and
  reuses the existing inspectors as the canonical-byte decoders (no protocol or
  internal-struct imports). Tests cover slug round-tripping, the full
  mandate→receipt→dispute→assurance DAG traversal, reason-code stability, and
  redaction protection. Native + wasm32 build green.
- **HEM-02** — the trace view now shows **dual-lane finality**. A new
  `components::finality_lanes` module models a *buffered* lane (present/absent in
  the observation plane) beside an *anchored* lane (`none` / `pending` / `final` /
  `unavailable`), and `FinalityLanesView` renders both on each Tuppira-explorer
  lineage item. The finality decision lives in a single gate,
  `AnchoredFinality::from_chain_read`, which reaches `Final` only when the
  observed confirmation depth meets a positive reorg-safe required depth — a
  shallow, zero-requirement, or unknown read stays `pending` and is never shown
  as final. The real chain finality source is ANCHOR-01; until it lands the
  anchored lane renders an explicit unavailable state. Component tests cover every
  lane-state combination including the pending and below-depth cases.
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
