# Hemion

Hemion is the local-first developer console for Parwana. It is the place to
inspect accountability objects and, as those capabilities land, verify bundles
locally without treating a verdict recorded by another product as authoritative.
The existing multi-chain wallet remains available under **Legacy wallet**; its
routes and stored data are not migrated by this navigation refactor.

## Information architecture

- **Console home** (`/`) identifies the local verifier boundary and exposes only
  capabilities that work today.
- **Anchoring** (`/anchoring`) is a first-class console capability, peer to local
  bundle verification rather than a wallet feature: pick a Parwana-configured
  network (projected from `parwana/chains/*.toml`) and attempt to anchor a bundle
  or verify an existing anchor. The on-chain commitment / finality protocol
  backing is delivered by **ANCHOR-01**; until it lands, the anchor and verify
  actions render an explicit *unavailable* state naming that dependency — Hemion
  never fabricates an anchor or finality.
- **Legacy wallet** (`/wallet`) retains the previous wallet dashboard. Assets,
  activity, contacts, and settings keep their existing routes beneath that area.
  The chain services that back Anchoring are no longer quarantined here.
- Bundle verification, assurance inspection, and object inspection are the
  remaining Stage 7 console screens. They are deliberately absent from
  navigation until their implementation tickets are complete.

## Legacy wallet capabilities

- Multi-chain support (Bitcoin, Ethereum, Sui, Aptos, Solana)
- Encrypted key storage
- Seal monitoring and management
- Runtime-backed Sanad and transfer receipt presentation
- Cross-chain transfer tracking
- WebAssembly support for browser deployment

## Capability boundaries

Hemion never
creates simulated proofs, test results, balances, finality, or transfer
success. Proof construction, proof acceptance, seal consumption, and
cross-chain completion are runtime-owned operations; unavailable wallet
capabilities render an explicit unavailable state instead of a best-effort
result. An imported verdict is only "recorded elsewhere" until Hemion computes
it locally under an explicit verification context.

## Design traceability

The console shell implements Flow Spec Part 8 (`S-H1` and `H-RULE-1`) using the
Design System's Hemion “Instrument” skin. REV03 finding D-02 governs the
corrected metadata color. The checked-in
[`docs/accessibility/hemion-contrast-matrix.md`](docs/accessibility/hemion-contrast-matrix.md)
matrix is enforced by
`tests/console_shell.rs`; keyboard access uses native links and visible
two-pixel `:focus-visible` outlines, and reduced-motion preferences disable
shell animation.

## License

MIT OR Apache-2.0
