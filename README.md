# Hemion

Standalone multi-chain wallet for the Parwana with a Dioxus-based UI.

## Features

- Multi-chain support (Bitcoin, Ethereum, Sui, Aptos, Solana)
- Encrypted key storage
- Seal monitoring and management
- Runtime-backed Sanad and transfer receipt presentation
- Cross-chain transfer tracking
- WebAssembly support for browser deployment

## Capability boundaries

`Hemion` is the only graphical wallet product in this workspace. It never
creates simulated proofs, test results, balances, finality, or transfer
success. Proof construction, proof acceptance, seal consumption, and
cross-chain completion are runtime-owned operations; unavailable wallet
capabilities render an explicit unavailable state instead of a best-effort
result.

## License

MIT OR Apache-2.0
