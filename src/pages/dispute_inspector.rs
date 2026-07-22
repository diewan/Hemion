//! G-05 — accessible evidence graph and dispute inspector.

use crate::services::bundle_verifier::{
    EvidenceGraphInspection, LocalVerificationResult, import_and_verify, import_context,
    inspect_evidence_graph,
};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
enum NodeFilter {
    All,
    Claims,
    Observations,
    Attestations,
    Gaps,
    Withheld,
}

impl NodeFilter {
    const fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Claims => "Claims",
            Self::Observations => "Observations",
            Self::Attestations => "Attestations",
            Self::Gaps => "Gaps",
            Self::Withheld => "Withheld",
        }
    }
}

#[component]
pub fn DisputeInspector() -> Element {
    let mut input = use_signal(String::new);
    let mut graph = use_signal(|| None::<EvidenceGraphInspection>);
    let mut error = use_signal(|| None::<String>);
    let mut filter = use_signal(|| NodeFilter::All);
    let mut table_view = use_signal(|| false);
    // Independent verdict: a separately-pasted, hash-bound verification context
    // drives the pinned Parwana verifier so an overreach surfaces as a failed
    // Authority dimension with its reason codes (DEMO-03). The bundle can never
    // choose its own trust inputs, so the context is imported separately.
    let mut context_input = use_signal(String::new);
    let mut verdict = use_signal(|| None::<LocalVerificationResult>);
    let mut verdict_error = use_signal(|| None::<String>);
    rsx! {
        section { class: "console-home dispute-inspector", aria_labelledby: "dispute-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "dispute-title", "Evidence graph and dispute inspector" }
            p { class: "console-lede", "Inspect SDK-decoded relationships locally. Triage signals are not verifier conclusions, and missing or withheld evidence never establishes non-occurrence." }
            label { class: "console-panel inspector-import", r#for: "dispute-input",
                h2 { "Bundle DisputeBundle" }
                textarea { id: "dispute-input", rows: 8, value: "{input}", oninput: move |event| input.set(event.value()), placeholder: "Paste org.diewan.accountability.local-verification.v1 JSON" }
            }
            button { class: "console-action", r#type: "button", disabled: input().is_empty(), onclick: move |_| match inspect_evidence_graph(input().as_bytes()) {
                Ok(value) => { graph.set(Some(value)); error.set(None); }
                Err(value) => { graph.set(None); error.set(Some(format!("Inspection did not run · {value:?}. Unsupported, malformed, or inconsistent graphs are rejected."))); }
            }, "Inspect evidence" }
            if let Some(message) = error() { output { class: "console-notice", aria_live: "assertive", "{message}" } }

            label { class: "console-panel inspector-import", r#for: "context-input",
                h2 { "Verification context (independent verdict)" }
                textarea { id: "context-input", rows: 6, value: "{context_input}", oninput: move |event| context_input.set(event.value()), placeholder: "Paste org.diewan.accountability.verification-context.v1 JSON" }
            }
            button { class: "console-action", r#type: "button", disabled: input().is_empty() || context_input().is_empty(), onclick: move |_| {
                match import_context(context_input().as_bytes()) {
                    Ok(choice) => {
                        let selected = choice.name.clone();
                        match import_and_verify(input().as_bytes(), std::slice::from_ref(&choice), &selected) {
                            Ok(result) => { verdict.set(Some(result)); verdict_error.set(None); }
                            Err(value) => { verdict.set(None); verdict_error.set(Some(format!("Verification did not run · {value:?}. The bundle is unsupported, malformed, or inconsistent with the context."))); }
                        }
                    }
                    Err(value) => { verdict.set(None); verdict_error.set(Some(format!("Context rejected · {value:?}. Supply a supported, hash-bound verification context."))); }
                }
            }, "Assess authority (run local verifier)" }
            if let Some(message) = verdict_error() { output { class: "console-notice", aria_live: "assertive", "{message}" } }
            if let Some(result) = verdict() {
                section { class: "console-panel assurance-verdict", aria_label: "Independent assurance verdict",
                    h2 { "Assurance dimensions" }
                    p { class: "console-limitation", "Verdict under context \"{result.context_name}\". This is the pinned Parwana verifier's conclusion, limited to the selected, hash-bound context — never a claim of factual truth." }
                    ul { class: "assurance-dimension-list",
                        for dimension in result.assurance.dimensions.iter() {
                            li { class: dimension_class(dimension.status),
                                strong { "{dimension.dimension:?}" }
                                span { class: "assurance-status", " — {dimension.status:?}" }
                                if !dimension.reason_codes.is_empty() { ul { class: "assurance-reason-codes", for code in dimension.reason_codes.iter() { li { code { "{code}" } } } } }
                                if !dimension.limitations.is_empty() { ul { class: "assurance-limitations", for limitation in dimension.limitations.iter() { li { "{limitation}" } } } }
                            }
                        }
                    }
                }
            }

            if let Some(value) = graph() {
                section { class: "dispute-alerts", aria_label: "Dispute signals",
                    article { class: "dispute-alert dispute-gap", h2 { "Evidence gaps" } strong { "{value.gap_count}" } p { "Explicit gap nodes report unavailable evidence; they do not report non-occurrence." } }
                    article { class: "dispute-alert dispute-withheld", h2 { "Withheld branches" } strong { "{value.withheld_count}" } p { "Committed source details are intentionally undisclosed. The commitment remains visible." } }
                    article { class: "dispute-alert dispute-conflict", h2 { "Potential contradictions" } strong { "{value.potential_contradictions.len()}" } p { "Triage only; Hemion does not convert this signal into a protocol verdict." } }
                }
                div { class: "dispute-controls",
                    fieldset { legend { "Filter node types" }
                        for choice in [NodeFilter::All, NodeFilter::Claims, NodeFilter::Observations, NodeFilter::Attestations, NodeFilter::Gaps, NodeFilter::Withheld] {
                            button { class: "console-action", r#type: "button", aria_pressed: filter() == choice, onclick: move |_| filter.set(choice), "{choice.label()}" }
                        }
                    }
                    button { class: "console-action", r#type: "button", aria_pressed: table_view(), onclick: move |_| { let next = !table_view(); table_view.set(next); }, if table_view() { "Show graph view" } else { "Show accessible table view" } }
                }
                if !value.potential_contradictions.is_empty() { ul { class: "dispute-conflict-list", aria_label: "Potential contradiction details", for item in value.potential_contradictions.iter() { li { code { "{item.left}" } " ↔ " code { "{item.right}" } p { "{item.explanation}" } } } } }
                if table_view() {
                    table { class: "dispute-table", caption { "Evidence nodes and disclosure status" } thead { tr { th { scope: "col", "Type" } th { scope: "col", "Identifier" } th { scope: "col", "Producer" } th { scope: "col", "Disclosure" } th { scope: "col", "Collected" } } } tbody {
                        for node in value.nodes.iter().filter(|node| matches_filter(node, filter())) { tr { class: status_class(node), td { "{node.kind_label}" } td { code { "{node.id}" } } td { "{node.producer}" } td { if node.is_withheld { "Withheld" } else { "Disclosed" } } td { "{node.collected_at} UTC seconds" } } }
                    } }
                } else {
                    div { class: "evidence-graph", role: "list", aria_label: "Evidence relationship graph",
                        for node in value.nodes.iter().filter(|node| matches_filter(node, filter())) { article { class: status_class(node), role: "listitem", tabindex: "0", h3 { "{node.kind_label}" } code { "{node.short_id}" } p { "{node.kind_id}" } p { "Source: {node.source}" } p { "Classification: {node.classification}" } } }
                    }
                    section { class: "console-panel edge-list", h2 { "Relationships" } if value.edges.is_empty() { p { class: "console-limitation", "No relationships are declared. This does not establish that no relationship exists outside the disclosed graph." } } else { ul { for edge in value.edges.iter() { li { code { "{edge.from}" } " depends on " code { "{edge.to}" } } } } } }
                }
                aside { class: "console-notice", aria_label: "Dispute inspector limitations", strong { "What this view does not establish:" } span { " factual truth, completeness, authority, or a contradiction verdict. Withheld and absent branches remain unknown." } }
            }
        }
    }
}

fn matches_filter(
    node: &crate::services::bundle_verifier::EvidenceGraphNode,
    filter: NodeFilter,
) -> bool {
    match filter {
        NodeFilter::All => true,
        NodeFilter::Claims => node.kind_label == "Claim",
        NodeFilter::Observations => node.kind_label == "Observation",
        NodeFilter::Attestations => node.kind_label == "Attestation",
        NodeFilter::Gaps => node.is_gap,
        NodeFilter::Withheld => node.is_withheld,
    }
}
fn dimension_class(status: csv_sdk::accountability::DimensionStatus) -> &'static str {
    use csv_sdk::accountability::DimensionStatus;
    match status {
        DimensionStatus::NotSatisfied => "assurance-dimension assurance-dimension-failed",
        DimensionStatus::Indeterminate => "assurance-dimension assurance-dimension-indeterminate",
        DimensionStatus::Satisfied => "assurance-dimension assurance-dimension-satisfied",
        DimensionStatus::NotApplicable => "assurance-dimension assurance-dimension-na",
    }
}
fn status_class(node: &crate::services::bundle_verifier::EvidenceGraphNode) -> &'static str {
    if node.is_gap {
        "evidence-node evidence-node-gap"
    } else if node.is_withheld {
        "evidence-node evidence-node-withheld"
    } else {
        "evidence-node"
    }
}
