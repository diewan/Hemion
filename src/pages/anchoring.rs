//! Anchoring — a first-class console capability (HEM-01).
//!
//! Lets a developer pick a Parwana-configured network and attempt to anchor a
//! bundle or verify an existing anchor. The network list is projected from the
//! canonical chain specs; the anchor actions render the real chain state or an
//! explicit unavailable state (until ANCHOR-01 wires the on-chain path), never a
//! fabricated finality.

use dioxus::prelude::*;

use crate::services::anchoring::{
    AnchorAvailability, AnchoringNetwork, anchor_bundle, available_networks, verify_anchor,
};

fn availability_message(outcome: &AnchorAvailability) -> String {
    match outcome {
        AnchorAvailability::Unavailable { reason, depends_on } => {
            format!("Unavailable · {reason} (unblocked by {depends_on})")
        }
    }
}

fn finality_summary(network: &AnchoringNetwork) -> String {
    let mut parts: Vec<String> = Vec::new();
    match network.finality.deterministic_finality {
        Some(true) => parts.push("deterministic finality".to_string()),
        Some(false) => parts.push("probabilistic finality".to_string()),
        None => {}
    }
    if let Some(depth) = network.finality.max_reorg_depth {
        parts.push(format!("max reorg depth {depth}"));
    }
    if let Some(system) = &network.finality.proof_system {
        parts.push(format!("proof system {system}"));
    }
    if parts.is_empty() {
        "Finality profile not stated in the chain spec.".to_string()
    } else {
        parts.join(" · ")
    }
}

/// S — Anchoring capability page.
#[component]
pub fn Anchoring() -> Element {
    let networks = available_networks();
    let default_id = networks.first().map(|n| n.id.clone()).unwrap_or_default();
    let mut selected = use_signal(|| default_id);
    let mut result = use_signal(|| None::<String>);

    let selected_network = {
        let id = selected();
        networks.iter().find(|n| n.id == id).cloned()
    };

    rsx! {
        section { class: "console-home anchoring", aria_labelledby: "anchoring-title",
            p { class: "console-eyebrow", "HEMION / ANCHORING" }
            h1 { id: "anchoring-title", "Anchoring" }
            p { class: "console-lede",
                "Commit an accountability object to a chain and read back finality. \
                 Networks are the Parwana-configured chains; Hemion shows real chain \
                 state or an explicit unavailable state, and never fabricates an anchor."
            }

            div { class: "console-grid",
                label { class: "console-panel", r#for: "anchoring-network",
                    h2 { "Network" }
                    select {
                        id: "anchoring-network",
                        value: "{selected}",
                        onchange: move |event| {
                            selected.set(event.value());
                            result.set(None);
                        },
                        for net in networks.iter() {
                            option { value: "{net.id}", "{net.name} ({net.network})" }
                        }
                    }
                    if let Some(net) = &selected_network {
                        dl {
                            div { dt { "Chain id" } dd { class: "console-mono", "{net.id}" } }
                            div { dt { "Finality" } dd { "{finality_summary(net)}" } }
                            div {
                                dt { "Read RPC" }
                                dd { class: "console-mono",
                                    if net.rpc_urls.is_empty() {
                                        "No read endpoint in spec"
                                    } else {
                                        "{net.rpc_urls[0]}"
                                    }
                                }
                            }
                        }
                    }
                }

                article { class: "console-panel",
                    h2 { "Actions" }
                    p { "Anchor a locally verified bundle, or verify an existing anchor and read its finality." }
                    button {
                        class: "console-action",
                        r#type: "button",
                        onclick: move |_| {
                            result.set(Some(availability_message(&anchor_bundle(&selected()))));
                        },
                        "Anchor this bundle"
                    }
                    button {
                        class: "console-action",
                        r#type: "button",
                        onclick: move |_| {
                            result.set(Some(availability_message(&verify_anchor(&selected()))));
                        },
                        "Verify anchor"
                    }
                    if let Some(message) = result() {
                        output { class: "console-notice", aria_live: "polite", "{message}" }
                    }
                }
            }

            aside { class: "console-notice", aria_label: "Anchoring limitations",
                strong { "Anchoring is corroborating evidence, and optional." }
                span {
                    " An absent anchor is a limitation, never a failure. The on-chain \
                     commitment and finality path is delivered by ANCHOR-01; until then \
                     these actions report an explicit unavailable state."
                }
            }
        }
    }
}
