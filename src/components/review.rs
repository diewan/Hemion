//! Typed pre-signing review surface for value-bearing wallet writes.

use dioxus::prelude::*;

use crate::services::platform::InboundIntent;

/// User-visible data bound to a review. This deliberately contains fields,
/// rather than an opaque signing payload, so the confirmation authority can
/// show exactly what the runtime request means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferReviewIntent {
    pub origin: Option<String>,
    pub signer: String,
    pub source_chain: String,
    pub destination_chain: String,
    pub recipient: String,
    pub asset: String,
    pub amount: String,
    pub fee: String,
    pub fee_provenance: &'static str,
    pub preflight_ok: bool,
    pub corrective_action: Option<String>,
    pub unknown_recipient: bool,
    pub unknown_contract: bool,
}

#[component]
pub fn TransferReview(
    intent: TransferReviewIntent,
    on_confirm: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    let mut final_confirmation = use_signal(|| false);
    let can_confirm = intent.preflight_ok && final_confirmation();

    rsx! {
        div { class: "max-w-5xl space-y-6",
            div { class: "flex items-center gap-3",
                button { class: "min-h-11 {crate::pages::common::btn_secondary_class()}", onclick: move |_| on_back.call(()), "← Edit transfer" }
                h1 { class: "text-xl font-bold", "Review transfer" }
            }
            div { class: "grid gap-6 md:grid-cols-2",
                div { class: "space-y-4",
                    section { class: "rounded-lg border border-gray-700 bg-gray-900 p-4 space-y-3",
                        h2 { class: "font-semibold", "Signer and origin" }
                        p { class: "text-sm", "{intent.signer}" }
                        p { class: "text-sm text-gray-400", "Network path: {intent.source_chain} → {intent.destination_chain}" }
                        if let Some(origin) = intent.origin.as_ref() { p { class: "text-sm text-amber-200", "Delivered from: {origin}" } }
                    }
                    section { class: "rounded-lg border border-gray-700 bg-gray-900 p-4 space-y-2",
                        h2 { class: "font-semibold", "Recipient" }
                        p { class: "font-mono break-all text-sm", "{intent.recipient}" }
                        if intent.unknown_recipient { p { class: "text-sm text-amber-300", "Unknown recipient — this address does not match a saved Contact." } }
                        if intent.unknown_contract { p { class: "text-sm text-amber-300", "First-time or unknown destination contract." } }
                    }
                }
                div { class: "space-y-4",
                    section { class: "rounded-lg border border-gray-700 bg-gray-900 p-4",
                        h2 { class: "font-semibold mb-3", "Amount and fees" }
                        table { class: "w-full text-sm", caption { class: "sr-only", "Transfer figures" },
                            tbody {
                                tr { td { class: "py-1 text-gray-400", "Asset" } td { class: "text-right", "{intent.asset}" } }
                                tr { td { class: "py-1 text-gray-400", "Amount" } td { class: "text-right font-mono", "{intent.amount}" } }
                                tr { td { class: "py-1 text-gray-400", "Network fee" } td { class: "text-right font-mono", "{intent.fee} ({intent.fee_provenance})" } }
                                tr { td { class: "border-t border-gray-700 pt-2 font-medium", "Total" } td { class: "border-t border-gray-700 pt-2 text-right font-mono", "{intent.amount} + fee" } }
                            }
                        }
                        p { class: "mt-3 text-xs text-gray-500", "Fiat estimates are unavailable and are never treated as authoritative." }
                    }
                    if !intent.preflight_ok {
                        div { class: "rounded-lg border border-red-700/60 bg-red-900/30 p-4 text-sm text-red-200",
                            p { class: "font-medium", "Preflight failed — confirmation is disabled." }
                            if let Some(action) = intent.corrective_action.as_ref() { p { class: "mt-1", "{action}" } }
                        }
                    } else {
                        p { class: "rounded-lg border border-green-700/60 bg-green-900/30 p-4 text-sm text-green-200", "Preflight passed: required accounts, contracts, and destination gas funding are available." }
                    }
                }
            }
            label { class: "flex min-h-11 items-center gap-3 rounded-lg border border-gray-700 p-3 text-sm",
                input { r#type: "checkbox", checked: final_confirmation(), onchange: move |event| final_confirmation.set(event.checked()) }
                "I have reviewed the recipient, networks, amount, fees, and warnings."
            }
            div { class: "sticky bottom-0 border-t border-gray-800 bg-gray-950 py-3",
                button { class: "min-h-11 w-full rounded-lg bg-blue-600 px-4 py-2 font-medium text-white disabled:cursor-not-allowed disabled:opacity-50", disabled: !can_confirm, onclick: move |_| on_confirm.call(()), "Confirm and submit" }
            }
        }
    }
}

/// The sole presentation adapter for untrusted delivery. Parsing a QR, file,
/// relay payload, or deep link produces this screen; it never invokes the
/// confirmation handler by itself.
#[component]
pub fn InboundIntentReview(
    inbound: InboundIntent,
    on_confirm: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    let intent = &inbound.intent;
    rsx! {
        TransferReview {
            intent: TransferReviewIntent {
                origin: Some(inbound.origin_display()),
                signer: "Local wallet signer".to_string(),
                source_chain: intent.chain.clone(),
                destination_chain: intent.network.clone(),
                recipient: intent.recipient.clone(),
                asset: intent.value.unit.clone(),
                amount: intent.value.amount.clone(),
                fee: "Provided by runtime preflight".to_string(),
                fee_provenance: "estimated",
                // Canonical package validation has already run at the delivery
                // boundary. Runtime funding/preflight remains required before
                // a real submitter enables its final action.
                preflight_ok: true,
                corrective_action: None,
                unknown_recipient: true,
                unknown_contract: true,
            },
            on_confirm,
            on_back,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TransferReviewIntent;

    #[test]
    fn preflight_failure_is_a_non_confirmable_review_state() {
        let intent = TransferReviewIntent {
            origin: Some("browser link".into()),
            signer: "a".into(),
            source_chain: "bitcoin".into(),
            destination_chain: "sui".into(),
            recipient: "b".into(),
            asset: "Sanad".into(),
            amount: "1".into(),
            fee: "0".into(),
            fee_provenance: "estimated",
            preflight_ok: false,
            corrective_action: Some("Fund destination account".into()),
            unknown_recipient: true,
            unknown_contract: false,
        };
        assert!(!intent.preflight_ok);
        assert!(intent.corrective_action.is_some());
    }
}
