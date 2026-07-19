//! S-H4 — read-only mandate and receipt inspectors.

use crate::services::bundle_verifier::{ObjectInspection, inspect_bundle};
use dioxus::prelude::*;

#[component]
pub fn ObjectInspector() -> Element {
    let mut input = use_signal(String::new);
    let mut inspection = use_signal(|| None::<ObjectInspection>);
    let mut error = use_signal(|| None::<String>);
    rsx! {
        section { class: "console-home object-inspector", aria_labelledby: "inspector-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "inspector-title", "Mandate and receipt inspector" }
            p { class: "console-lede", "Inspect SDK-decoded objects locally. Importing does not verify authenticity, authorize an action, or change live state." }
            label { class: "console-panel inspector-import", r#for: "inspector-input",
                h2 { "Bundle DisputeBundle" }
                textarea { id: "inspector-input", rows: 8, value: "{input}", oninput: move |event| input.set(event.value()), placeholder: "Paste org.diewan.accountability.local-verification.v1 JSON" }
            }
            button { class: "console-action", r#type: "button", disabled: input().is_empty(), onclick: move |_| {
                match inspect_bundle(input().as_bytes()) {
                    Ok(value) => { inspection.set(Some(value)); error.set(None); }
                    Err(value) => { inspection.set(None); error.set(Some(format!("Inspection did not run · {value:?}. Unsupported, malformed, or inconsistent objects are rejected."))); }
                }
            }, "Inspect objects" }
            if let Some(message) = error() { output { class: "console-notice", aria_live: "assertive", "{message}" } }
            if let Some(value) = inspection() {
                div { class: "inspector-columns",
                    article { class: "console-panel", aria_labelledby: "mandate-heading",
                        h2 { id: "mandate-heading", "Mandate " code { "ActionMandate" } }
                        p { "{value.mandate.summary}" }
                        Field { label: "Mandate identifier", value: value.mandate.id.clone() }
                        Field { label: "Intent binding", value: value.mandate.intent_id.clone() }
                        Field { label: "Issuer identity", value: value.mandate.issuer_identity.clone() }
                        Field { label: "Authorized subject", value: value.mandate.subject.clone() }
                        Field { label: "Authority domain", value: value.mandate.authority_domain.clone() }
                        Field { label: "Validity", value: value.mandate.validity.clone() }
                        Field { label: "Signature algorithm", value: value.mandate.signature_algorithm.clone() }
                        Field { label: "Signer key identity", value: value.mandate.signer_key_id.clone() }
                        List { heading: "Constraints", values: value.mandate.constraints.clone() }
                        List { heading: "Required evidence", values: value.mandate.evidence_requirements.clone() }
                        Bytes { heading: "Canonical bytes", value: value.mandate.canonical_hex.clone() }
                    }
                    article { class: "console-panel", aria_labelledby: "receipt-heading",
                        h2 { id: "receipt-heading", "Receipt " code { "ExecutionReceipt" } }
                        p { "{value.receipt.summary}" }
                        Field { label: "Receipt identifier", value: value.receipt.id.clone() }
                        Field { label: "Mandate binding", value: value.receipt.mandate_id.clone() }
                        Field { label: "Intent binding", value: value.receipt.intent_id.clone() }
                        Field { label: "Attempt binding", value: value.receipt.attempt_id.clone() }
                        Field { label: "Execution state", value: value.receipt.attempt_state.clone() }
                        Field { label: "Reported outcome", value: value.receipt.outcome.clone() }
                        Field { label: "Executor identity", value: value.receipt.executor_identity.clone() }
                        Field { label: "Producer identity", value: value.receipt.producer_identity.clone() }
                        Bytes { heading: "Producer signature", value: value.receipt.producer_signature.clone() }
                        List { heading: "Dispatch evidence", values: value.receipt.dispatch_evidence.clone() }
                        List { heading: "Target evidence", values: value.receipt.target_evidence.clone() }
                        Bytes { heading: "Canonical bytes", value: value.receipt.canonical_hex.clone() }
                    }
                }
                section { class: "console-panel inspector-timeline", aria_labelledby: "timeline-heading",
                    h2 { id: "timeline-heading", "Replay timeline" }
                    ol { for entry in value.timeline.iter() { li {
                        time { datetime: "{entry.timestamp}", "{entry.timestamp} UTC seconds" }
                        strong { "{entry.label}" } code { "{entry.protocol_state}" }
                        span { class: "console-mono", "Evidence {entry.evidence}" }
                    } } }
                    p { class: "console-limitation", "This reconstructs the exported artifact sequence. Absence of a row does not establish non-occurrence." }
                }
                section { class: "console-panel", aria_labelledby: "evidence-heading",
                    h2 { id: "evidence-heading", "Evidence records" }
                    div { class: "inspector-table", role: "table", aria_label: "Evidence records",
                        for item in value.evidence.iter() { article { class: "inspector-evidence", tabindex: "0",
                            h3 { code { "{item.kind}" } }
                            Field { label: "Evidence identifier", value: item.id.clone() }
                            Field { label: "Producer", value: item.producer.clone() }
                            Field { label: "Collected at", value: format!("{} UTC seconds", item.collected_at) }
                            Field { label: "Content digest", value: item.content_digest.clone() }
                            Field { label: "Source disclosure", value: item.source.clone() }
                            Field { label: "Classification", value: item.classification.clone() }
                        } }
                    }
                }
                aside { class: "console-notice", aria_label: "Inspector limitations",
                    strong { "What this record does not establish:" }
                    span { " that authority exists beyond the approval policy, that every statement is factually true, or that all relevant events were captured." }
                }
            }
        }
    }
}

#[component]
fn Field(label: &'static str, value: String) -> Element {
    rsx! { dl { class: "inspector-field", div { dt { "{label}" } dd { class: "console-mono", "{value}" } } } }
}

#[component]
fn List(heading: &'static str, values: Vec<String>) -> Element {
    rsx! { section { h3 { "{heading}" } if values.is_empty() { p { class: "console-limitation", "None declared. This describes the object; it does not establish that no external condition exists." } } else { ul { for value in values { li { code { "{value}" } } } } } } }
}

#[component]
fn Bytes(heading: &'static str, value: String) -> Element {
    rsx! { details { class: "inspector-bytes", summary { "{heading}" } pre { tabindex: "0", "{value}" } } }
}
