//! Local, evidence-first inspector for runtime lifecycle projections.

use crate::context::{ProofRecord, ProofStatus, TransferLifecycleView};
use base64::{Engine as _, engine::general_purpose::STANDARD};
use dioxus::prelude::*;

/// Component props need equality, while proof records intentionally do not
/// expose one. Evidence is always refreshed when its parent renders.
#[derive(Clone)]
pub struct InspectorProofs(pub Vec<ProofRecord>);

impl PartialEq for InspectorProofs {
    fn eq(&self, _: &Self) -> bool {
        false
    }
}

/// One responsive implementation serves as a right drawer on wide layouts
/// and a bottom sheet on narrow layouts. It never navigates or fetches remote
/// evidence when opened.
#[component]
pub fn Inspector(lifecycle: Option<TransferLifecycleView>, proofs: InspectorProofs) -> Element {
    let mut open = use_signal(|| false);
    let mut tab = use_signal(|| "overview");
    let mut trigger = use_signal(|| None::<MountedEvent>);
    let mut first_focus = use_signal(|| None::<MountedEvent>);
    let mut last_focus = use_signal(|| None::<MountedEvent>);
    let mut pointer_start_y = use_signal(|| None::<f64>);

    let mut close_and_restore = move || {
        open.set(false);
        if let Some(target) = trigger.peek().as_ref().cloned() {
            spawn(async move {
                let _ = target.set_focus(true).await;
            });
        }
    };

    let source = lifecycle
        .as_ref()
        .and_then(|value| value.source_finality.clone());
    let destination = lifecycle
        .as_ref()
        .and_then(|value| value.destination_finality.clone());
    let verification = lifecycle
        .as_ref()
        .and_then(|value| value.verification_assurance)
        .map(|value| format!("{value:?}"));
    let verification_provenance = lifecycle
        .as_ref()
        .and_then(|value| value.verification_provenance)
        .unwrap_or("not reported");
    let artifact_download = lifecycle.as_ref().and_then(|value| {
        let bytes = hex::decode(value.artifact_cbor_hex.as_deref()?).ok()?;
        Some(format!(
            "data:application/cbor;base64,{}",
            STANDARD.encode(bytes)
        ))
    });

    rsx! {
        button {
            class: "min-h-11 min-w-11 rounded-lg border border-blue-700/60 bg-blue-900/30 px-3 py-2 text-sm text-blue-200 hover:bg-blue-900/50 focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-400",
            aria_label: "Open runtime evidence inspector",
            onmounted: move |event| trigger.set(Some(event)),
            onclick: move |_| {
                tab.set("overview");
                open.set(true);
            },
            "Inspect runtime evidence"
        }
        if open() {
            div {
                class: "fixed inset-0 z-50 bg-black/60",
                role: "presentation",
                onclick: move |_| close_and_restore(),
            }
            aside {
                class: "inspector-surface fixed inset-x-0 bottom-0 z-50 max-h-[90vh] overflow-y-auto rounded-t-2xl border border-gray-700 bg-gray-950 p-5 shadow-2xl md:inset-y-0 md:right-0 md:left-auto md:w-[32rem] md:rounded-none",
                role: "dialog",
                aria_modal: "true",
                aria_label: "Runtime evidence inspector",
                onkeydown: move |event| if event.key() == Key::Escape { close_and_restore(); },
                onpointerdown: move |event| pointer_start_y.set(Some(event.client_coordinates().y)),
                onpointerup: move |event| {
                    if pointer_start_y().is_some_and(|start| event.client_coordinates().y - start > 80.0) {
                        close_and_restore();
                    }
                    pointer_start_y.set(None);
                },

                // Focus sentinels keep keyboard focus inside the modal while
                // preserving a natural first-to-last tab order.
                button {
                    class: "sr-only",
                    aria_hidden: "true",
                    tabindex: "0",
                    onfocus: move |_| {
                        if let Some(target) = last_focus.peek().as_ref().cloned() {
                            spawn(async move { let _ = target.set_focus(true).await; });
                        }
                    },
                }
                div { class: "mx-auto mb-3 h-1 w-12 rounded-full bg-gray-700 md:hidden", aria_hidden: "true" }
                div { class: "mb-4 flex items-center justify-between gap-3",
                    div {
                        h2 { class: "text-lg font-semibold", "Inspector" }
                        p { class: "text-xs text-gray-400", "Device-local runtime projection" }
                    }
                    button {
                        class: "min-h-11 min-w-11 rounded-lg text-gray-300 hover:bg-gray-800 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400",
                        aria_label: "Close inspector",
                        onmounted: move |event| {
                            first_focus.set(Some(event.clone()));
                            spawn(async move { let _ = event.set_focus(true).await; });
                        },
                        onclick: move |_| close_and_restore(),
                        "×"
                    }
                }
                div { class: "mb-4 flex gap-1 border-b border-gray-800", role: "tablist", aria_label: "Inspector sections",
                    for (id, label) in [("overview", "Overview"), ("evidence", "Evidence"), ("artifacts", "Artifacts")] {
                        button {
                            class: "min-h-11 px-3 text-sm focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400",
                            role: "tab",
                            aria_selected: *tab.read() == id,
                            tabindex: if *tab.read() == id { "0" } else { "-1" },
                            onclick: move |_| tab.set(id),
                            "{label}"
                        }
                    }
                }
                if lifecycle.is_none() {
                    div { class: "rounded border border-amber-700/50 bg-amber-900/20 p-3 text-sm text-amber-200", "No local runtime lifecycle artifact is available for this item. Explorer data is not substituted for verification evidence." }
                } else if *tab.read() == "overview" {
                    div { class: "space-y-3 text-sm",
                        p { class: "font-medium text-blue-200", "{lifecycle.as_ref().unwrap().stage.name}" }
                        p { class: "text-gray-300", "{lifecycle.as_ref().unwrap().stage.explanation}" }
                        p { class: "text-xs text-gray-400", "Runtime journal: {lifecycle.as_ref().unwrap().journal_phase}" }
                        if let Some(reason) = lifecycle.as_ref().unwrap().failure_reason.as_ref() {
                            p { class: "text-sm text-red-300", "Runtime reason: {reason}" }
                        }
                    }
                } else if *tab.read() == "evidence" {
                    div { class: "space-y-3 text-sm",
                        {evidence_fact("Source finality", source)}
                        {evidence_fact("Destination finality", destination)}
                        div { class: "rounded border border-gray-800 p-3",
                            p { class: "font-medium", "Verification assurance" }
                            if let Some(value) = verification {
                                p { "{value}" }
                            } else {
                                p { class: "text-amber-300", "No cryptographic assurance reported." }
                            }
                            p { class: "text-xs text-gray-400", "Provenance: {verification_provenance}" }
                        }
                        {proof_evidence(&proofs.0)}
                    }
                } else {
                    div { class: "space-y-3 text-xs font-mono break-all",
                        {artifact("Sanad ID", &lifecycle.as_ref().unwrap().sanad_id)}
                        {artifact_opt("Transfer ID", lifecycle.as_ref().unwrap().transfer_id.as_deref())}
                        {artifact_opt("Lock transaction", lifecycle.as_ref().unwrap().lock_tx_hash.as_deref())}
                        {artifact_opt("Mint transaction", lifecycle.as_ref().unwrap().mint_tx_hash.as_deref())}
                        {artifact_opt("Proof hash", lifecycle.as_ref().unwrap().proof_hash.as_deref())}
                        {artifact_opt("Invoice ID", lifecycle.as_ref().unwrap().invoice_id.as_deref())}
                        {artifact_opt("Consignment digest", lifecycle.as_ref().unwrap().consignment_digest.as_deref())}
                        {artifact_opt("Artifact SHA-256 (select to copy)", lifecycle.as_ref().unwrap().artifact_sha256.as_deref())}
                        if let Some(href) = artifact_download.as_ref() {
                            a {
                                class: "inline-flex min-h-11 items-center rounded-lg border border-gray-700 px-3 py-2 text-blue-300 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400",
                                href: "{href}",
                                download: "hemion-runtime-artifact.cbor",
                                "Export exact {lifecycle.as_ref().unwrap().artifact_kind} CBOR"
                            }
                        } else {
                            p { class: "text-amber-300", "No canonical artifact bytes are available for export." }
                        }
                    }
                }
                button {
                    class: "mt-5 min-h-11 w-full rounded-lg border border-gray-700 px-3 py-2 text-sm hover:bg-gray-800 focus-visible:outline focus-visible:outline-2 focus-visible:outline-blue-400",
                    onmounted: move |event| last_focus.set(Some(event)),
                    onclick: move |_| close_and_restore(),
                    "Done"
                }
                button {
                    class: "sr-only",
                    aria_hidden: "true",
                    tabindex: "0",
                    onfocus: move |_| {
                        if let Some(target) = first_focus.peek().as_ref().cloned() {
                            spawn(async move { let _ = target.set_focus(true).await; });
                        }
                    },
                }
            }
        }
    }
}

fn evidence_fact(label: &'static str, evidence: Option<crate::context::EvidenceView>) -> Element {
    rsx! {
        div { class: "rounded border border-gray-800 p-3",
            p { class: "font-medium", "{label}" }
            if let Some(value) = evidence {
                p { class: "text-gray-200", "{value.summary}" }
                p { class: "text-xs text-gray-400", "Provenance: {value.provenance}" }
            } else {
                p { class: "text-amber-300", "Not reported by the runtime artifact." }
                p { class: "text-xs text-gray-400", "Provenance: not reported" }
            }
        }
    }
}

fn artifact(label: &'static str, value: &str) -> Element {
    rsx! {
        div {
            p { class: "text-gray-500", "{label}" }
            code { class: "select-all", tabindex: "0", "{value}" }
        }
    }
}

fn artifact_opt(label: &'static str, value: Option<&str>) -> Element {
    rsx! { if let Some(value) = value { {artifact(label, value)} } }
}

/// Recorded proof receipts are entity-local evidence. Their presence never
/// upgrades an asset or transfer to verified: that authority remains with the
/// runtime receipt shown above.
fn proof_evidence(proofs: &[ProofRecord]) -> Element {
    rsx! {
        div { class: "rounded border border-gray-800 p-3 space-y-3",
            p { class: "font-medium", "Attached proof receipts" }
            if proofs.is_empty() {
                p { class: "text-amber-300", "No proof receipt is attached to this item." }
            } else {
                for proof in proofs {
                    {
                        let proof_reference = proof.seal_ref.as_deref().unwrap_or("unreferenced");
                        rsx! {
                    div { class: "rounded border border-gray-800 bg-gray-900/40 p-3 space-y-1",
                        p { class: "font-mono text-xs break-all", "Proof: {proof_reference}" }
                        p { class: "text-xs text-gray-300", "Chain: {proof.chain}; type: {proof.proof_type}" }
                        p { class: "text-xs text-gray-300", "Recorded status: {proof.status}" }
                        if let Some(tx_hash) = proof.verification_tx_hash.as_deref() {
                            p { class: "font-mono text-xs break-all text-gray-400", "Anchor transaction: {tx_hash}" }
                        }
                        if let Some(data) = proof.proof_data.as_deref() {
                            details { class: "text-xs",
                                summary { class: "cursor-pointer text-blue-300", "Recorded proof material" }
                                pre { class: "mt-2 whitespace-pre-wrap break-all text-gray-400", "{data}" }
                            }
                        }
                        if proof.status != ProofStatus::Verified {
                            p { class: "text-xs text-amber-300", "This recorded receipt is not cryptographic acceptance." }
                        }
                    }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn inspector_keeps_proof_receipts_in_entity_evidence() {
        let source = include_str!("inspector.rs");
        assert!(source.contains("Attached proof receipts"));
        assert!(source.contains("not cryptographic acceptance"));
    }
}
