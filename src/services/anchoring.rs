//! Anchoring — a first-class Hemion capability (HEM-01).
//!
//! "Anchoring" here means *committing an accountability object to a chain and
//! reading back finality*, not holding balances. It is peer to local bundle
//! verification, not a wallet feature, so it is surfaced in the primary console
//! navigation rather than under the preserved legacy wallet.
//!
//! This module owns the **network read model**: the selectable set of chains is
//! projected from the canonical Parwana chain specs (`parwana/chains/*.toml`),
//! embedded at build time so the projection is identical on native and the wasm
//! web bundle (no filesystem, no RPC at import). The extraction mirrors
//! `csv_runtime::chain_discovery::ChainDiscovery::load_from_toml`: fields are
//! read out of a `toml::Value` rather than bound to one Rust struct, because the
//! spec's `[rpc_policy]` / `[finality_guarantee]` / `[capabilities]` blocks do
//! not match a single serde type.
//!
//! The **anchor actions** deliberately report [`AnchorAvailability::Unavailable`]
//! until the on-chain commitment/finality protocol backing lands in **ANCHOR-01**
//! (`csv-accountability` `Anchor` node + `csv-chain-ports` `MintAdapter` /
//! `ProofAdapter` for accountability commitments). Hemion renders real chain
//! state or an explicit unavailable state — it never fabricates finality or a
//! passing anchor. When ANCHOR-01 is wired, [`anchor_bundle`] / [`verify_anchor`]
//! become the call sites for the real adapters; the `Unavailable` arm is the
//! only thing that changes.

/// The ticket whose completion unblocks real anchoring. Surfaced verbatim so the
/// unavailable state is traceable to the protocol work it waits on.
pub const ANCHOR_BACKING_TICKET: &str = "ANCHOR-01";

/// Each entry is `(chain_id, embedded TOML spec)`. Embedding at build time keeps
/// the network list byte-identical across the native and wasm targets.
const CHAIN_SPECS: &[(&str, &str)] = &[
    (
        "aptos-testnet",
        include_str!("../../../parwana/chains/aptos-testnet.toml"),
    ),
    (
        "bitcoin-signet",
        include_str!("../../../parwana/chains/bitcoin-signet.toml"),
    ),
    (
        "ethereum",
        include_str!("../../../parwana/chains/ethereum.toml"),
    ),
    (
        "ethereum-sepolia",
        include_str!("../../../parwana/chains/ethereum-sepolia.toml"),
    ),
    (
        "solana-devnet",
        include_str!("../../../parwana/chains/solana-devnet.toml"),
    ),
    (
        "sui-testnet",
        include_str!("../../../parwana/chains/sui-testnet.toml"),
    ),
];

/// The finality profile projected from a chain spec, shown so an operator can
/// judge how strong an anchor on this network would be. Absent fields are
/// reported as absent, never defaulted to a favourable value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityProfile {
    /// `finality_guarantee.is_probabilistic` — whether finality is probabilistic
    /// (reorg-prone) rather than deterministic.
    pub probabilistic: Option<bool>,
    /// `finality_guarantee.max_reorg_depth` — the deepest reorg the spec plans
    /// for; the anchor cannot be treated as final below this depth.
    pub max_reorg_depth: Option<u64>,
    /// `finality_guarantee.proof_system.type` — the finality proof system, e.g.
    /// `EthereumPos`.
    pub proof_system: Option<String>,
    /// `capabilities.deterministic_finality` — the capability matrix's view of
    /// whether this chain finalises deterministically.
    pub deterministic_finality: Option<bool>,
}

/// A network an accountability commitment could be anchored to, projected from a
/// canonical Parwana chain spec. Read-only presentation data; carries no keys and
/// makes no RPC call on construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchoringNetwork {
    /// Canonical `chain_id` (e.g. `ethereum-sepolia`).
    pub id: String,
    /// Human-readable `chain_name`.
    pub name: String,
    /// `default_network` (e.g. `sepolia`).
    pub network: String,
    /// Read-capable RPC endpoint URLs from `[rpc_policy].endpoints` (those whose
    /// `capabilities` include `read`). Shown, not dialled, at import.
    pub rpc_urls: Vec<String>,
    /// `block_explorer_urls` for operator drill-down.
    pub block_explorer_urls: Vec<String>,
    /// Projected finality profile.
    pub finality: FinalityProfile,
}

fn read_capable_rpc_urls(value: &toml::Value) -> Vec<String> {
    value
        .get("rpc_policy")
        .and_then(|policy| policy.get("endpoints"))
        .and_then(toml::Value::as_array)
        .map(|endpoints| {
            endpoints
                .iter()
                .filter(|endpoint| {
                    endpoint
                        .get("capabilities")
                        .and_then(toml::Value::as_array)
                        .is_some_and(|caps| {
                            caps.iter().any(|cap| cap.as_str() == Some("read"))
                        })
                })
                .filter_map(|endpoint| {
                    endpoint
                        .get("url")
                        .and_then(toml::Value::as_str)
                        .map(ToString::to_string)
                })
                .collect()
        })
        .unwrap_or_default()
}

fn string_array(value: &toml::Value, key: &str) -> Vec<String> {
    value
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(ToString::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn finality_profile(value: &toml::Value) -> FinalityProfile {
    let guarantee = value.get("finality_guarantee");
    let capabilities = value.get("capabilities");
    FinalityProfile {
        probabilistic: guarantee
            .and_then(|g| g.get("is_probabilistic"))
            .and_then(toml::Value::as_bool),
        max_reorg_depth: guarantee
            .and_then(|g| g.get("max_reorg_depth"))
            .and_then(toml::Value::as_integer)
            .and_then(|v| u64::try_from(v).ok()),
        proof_system: guarantee
            .and_then(|g| g.get("proof_system"))
            .and_then(|p| p.get("type"))
            .and_then(toml::Value::as_str)
            .map(ToString::to_string),
        deterministic_finality: capabilities
            .and_then(|c| c.get("deterministic_finality"))
            .and_then(toml::Value::as_bool),
    }
}

fn parse_network(fallback_id: &str, spec: &str) -> Option<AnchoringNetwork> {
    let value: toml::Value = toml::from_str(spec).ok()?;
    let id = value
        .get("chain_id")
        .and_then(toml::Value::as_str)
        .unwrap_or(fallback_id)
        .to_string();
    let name = value
        .get("chain_name")
        .and_then(toml::Value::as_str)
        .unwrap_or(&id)
        .to_string();
    let network = value
        .get("default_network")
        .and_then(toml::Value::as_str)
        .unwrap_or("mainnet")
        .to_string();
    Some(AnchoringNetwork {
        id,
        name,
        network,
        rpc_urls: read_capable_rpc_urls(&value),
        block_explorer_urls: string_array(&value, "block_explorer_urls"),
        finality: finality_profile(&value),
    })
}

/// The selectable anchoring networks, projected from the embedded Parwana chain
/// specs and sorted by canonical id for a stable UI order.
///
/// A network is identified by its canonical `chain_id`; specs that resolve to the
/// same id (the repo ships both `ethereum.toml` and `ethereum-sepolia.toml` for
/// `ethereum-sepolia`) are deduplicated the way the canonical registry's
/// id-keyed map is, keeping the first spec in `CHAIN_SPECS` order. A spec that
/// fails to parse is dropped rather than faked.
#[must_use]
pub fn available_networks() -> Vec<AnchoringNetwork> {
    let mut networks: Vec<AnchoringNetwork> = Vec::new();
    for (id, spec) in CHAIN_SPECS {
        if let Some(net) = parse_network(id, spec)
            && !networks.iter().any(|existing| existing.id == net.id)
        {
            networks.push(net);
        }
    }
    networks.sort_by(|a, b| a.id.cmp(&b.id));
    networks
}

/// Look up one network by canonical id.
#[must_use]
pub fn network(id: &str) -> Option<AnchoringNetwork> {
    available_networks().into_iter().find(|n| n.id == id)
}

/// The outcome of an anchor action. There is deliberately no `Ok`/`Anchored`
/// success arm that can be produced without a real adapter: until ANCHOR-01
/// wires the on-chain path, every action resolves to [`AnchorAvailability::Unavailable`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorAvailability {
    /// The anchor path is not wired. Carries the human-readable reason and the
    /// ticket (`depends_on`) that unblocks it. This is an explicit unavailable
    /// state, not a failed or passing verdict.
    Unavailable {
        /// Why the action cannot run.
        reason: String,
        /// The ticket whose completion enables the real path (`ANCHOR-01`).
        depends_on: &'static str,
    },
}

impl AnchorAvailability {
    /// Convenience: is this the unavailable state?
    #[must_use]
    pub const fn is_unavailable(&self) -> bool {
        matches!(self, Self::Unavailable { .. })
    }
}

/// Anchor an accountability bundle onto `network`.
///
/// Wires to the `csv-chain-ports` `MintAdapter` for accountability commitments
/// once **ANCHOR-01** lands. Until then it returns an explicit unavailable state
/// naming the network so the operator sees the network was recognised but the
/// on-chain commitment path is absent — never a synthesized anchor id.
#[must_use]
pub fn anchor_bundle(network_id: &str) -> AnchorAvailability {
    let known = network(network_id).is_some();
    let reason = if known {
        format!(
            "Anchoring to `{network_id}` is unavailable: no on-chain accountability \
             commitment path is wired yet. Hemion will not synthesize an anchor."
        )
    } else {
        format!("Unknown anchoring network `{network_id}`.")
    };
    AnchorAvailability::Unavailable {
        reason,
        depends_on: ANCHOR_BACKING_TICKET,
    }
}

/// Verify a previously produced anchor on `network` and read its finality.
///
/// Wires to the `csv-chain-ports` `ProofAdapter` / finality read once
/// **ANCHOR-01** lands. Until then it returns an explicit unavailable state
/// rather than a fabricated `final` verdict.
#[must_use]
pub fn verify_anchor(network_id: &str) -> AnchorAvailability {
    let known = network(network_id).is_some();
    let reason = if known {
        format!(
            "Verifying an anchor on `{network_id}` is unavailable: no on-chain \
             finality read is wired yet. Hemion will not report finality it did \
             not observe."
        )
    } else {
        format!("Unknown anchoring network `{network_id}`.")
    };
    AnchorAvailability::Unavailable {
        reason,
        depends_on: ANCHOR_BACKING_TICKET,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_embedded_chain_specs_deduped_by_id() {
        let networks = available_networks();
        let ids: Vec<&str> = networks.iter().map(|n| n.id.as_str()).collect();
        // Sorted, canonical, de-duplicated ids. `ethereum.toml` and
        // `ethereum-sepolia.toml` both declare `ethereum-sepolia`, so it appears
        // exactly once.
        assert_eq!(
            ids,
            vec![
                "aptos-testnet",
                "bitcoin-signet",
                "ethereum-sepolia",
                "solana-devnet",
                "sui-testnet",
            ]
        );
        // No id appears twice.
        let mut sorted = ids.clone();
        sorted.dedup();
        assert_eq!(sorted, ids, "network ids must be unique");
    }

    #[test]
    fn ethereum_sepolia_projection_carries_real_rpc_and_finality() {
        let net = network("ethereum-sepolia").expect("sepolia present");
        assert_eq!(net.network, "sepolia");
        // A read-capable endpoint from [rpc_policy] is projected, not invented.
        assert!(
            net.rpc_urls
                .iter()
                .any(|url| url.contains("publicnode.com")),
            "read-capable rpc url projected: {:?}",
            net.rpc_urls
        );
        assert!(net.block_explorer_urls.iter().any(|u| u.contains("etherscan")));
        assert_eq!(net.finality.proof_system.as_deref(), Some("EthereumPos"));
        assert_eq!(net.finality.max_reorg_depth, Some(12));
        assert_eq!(net.finality.deterministic_finality, Some(true));
    }

    #[test]
    fn anchor_actions_report_unavailable_never_a_pass() {
        // The whole point of the capability matrix: with ANCHOR-01 absent, both
        // actions must resolve to an explicit unavailable state that names the
        // backing ticket — never a fabricated anchor or finality verdict.
        for id in ["ethereum-sepolia", "bitcoin-signet", "sui-testnet"] {
            let anchored = anchor_bundle(id);
            let verified = verify_anchor(id);
            assert!(anchored.is_unavailable(), "anchor {id} must be unavailable");
            assert!(verified.is_unavailable(), "verify {id} must be unavailable");
            for outcome in [anchored, verified] {
                let AnchorAvailability::Unavailable { depends_on, reason } = outcome;
                assert_eq!(depends_on, "ANCHOR-01");
                assert!(!reason.is_empty());
            }
        }
    }

    #[test]
    fn unknown_network_is_rejected_not_faked() {
        let outcome = anchor_bundle("not-a-chain");
        let AnchorAvailability::Unavailable { reason, .. } = outcome;
        assert!(reason.contains("Unknown anchoring network"));
    }
}
