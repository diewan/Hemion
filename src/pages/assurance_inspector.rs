//! S-H3 — complete, locally computed AssuranceProfile inspection.

use crate::services::bundle_verifier::{
    LocalVerificationResult, import_and_verify, import_context,
};
use csv_sdk::accountability::{AssuranceDimension, DimensionStatus};
use dioxus::prelude::*;

fn dimension_labels(dimension: AssuranceDimension) -> (&'static str, &'static str) {
    match dimension {
        AssuranceDimension::Structural => ("Format & integrity", "Structural"),
        AssuranceDimension::Cryptographic => ("Signatures & hashes", "Cryptographic"),
        AssuranceDimension::Identity => ("Who signed", "Identity"),
        AssuranceDimension::Authority => ("Approval match", "Authority"),
        AssuranceDimension::Temporal => ("Timing & validity", "Temporal"),
        AssuranceDimension::SingleUse => ("Used only once", "Single-use"),
        AssuranceDimension::Execution => ("Did it happen", "Execution"),
        AssuranceDimension::ExternalCorroboration => {
            ("External source support", "External corroboration")
        }
        AssuranceDimension::Completeness => ("Required evidence present", "Completeness"),
        AssuranceDimension::Custody => ("Handling history", "Custody"),
        AssuranceDimension::Preservation => ("Still verifiable", "Preservation"),
    }
}

fn status_label(status: DimensionStatus) -> (&'static str, &'static str) {
    match status {
        DimensionStatus::Satisfied => ("✓", "Requirement met"),
        DimensionStatus::NotSatisfied => ("×", "Not met"),
        DimensionStatus::Indeterminate => ("?", "Cannot be determined"),
        DimensionStatus::NotApplicable => ("—", "Not applicable"),
    }
}

fn reason_sentence(status: DimensionStatus) -> &'static str {
    match status {
        DimensionStatus::Satisfied => {
            "The verifier found the evaluated requirement met under this context."
        }
        DimensionStatus::NotSatisfied => {
            "The verifier found that an evaluated requirement was not met."
        }
        DimensionStatus::Indeterminate => {
            "The available evidence or context cannot decide this requirement."
        }
        DimensionStatus::NotApplicable => {
            "The selected accountability profile does not evaluate this requirement."
        }
    }
}

#[component]
pub fn AssuranceInspector() -> Element {
    let mut bundle = use_signal(String::new);
    let mut context = use_signal(String::new);
    let mut result = use_signal(|| None::<Result<LocalVerificationResult, String>>);
    rsx! {
        section { class: "console-home assurance-inspector", aria_labelledby: "assurance-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "assurance-title", "Assurance AssuranceProfile" }
            p { class: "console-lede", "Compute and inspect all eleven assurance dimensions locally. No imported verdict is trusted, and no single trust score is produced." }
            div { class: "console-grid assurance-inputs",
                label { class: "console-panel", r#for: "assurance-bundle",
                    h2 { "Bundle DisputeBundle" }
                    textarea { id: "assurance-bundle", rows: 8, value: "{bundle}", oninput: move |event| bundle.set(event.value()), placeholder: "Paste org.diewan.accountability.local-verification.v1 JSON" }
                }
                label { class: "console-panel", r#for: "assurance-context",
                    h2 { "Context VerificationContext" }
                    textarea { id: "assurance-context", rows: 8, value: "{context}", oninput: move |event| context.set(event.value()), placeholder: "Paste org.diewan.accountability.verification-context.v1 JSON" }
                }
            }
            button { class: "console-action", r#type: "button", disabled: bundle().is_empty() || context().is_empty(), onclick: move |_| {
                let computed = import_context(context().as_bytes())
                    .and_then(|choice| {
                        let selected = choice.name.clone();
                        import_and_verify(bundle().as_bytes(), &[choice], &selected)
                    })
                    .map_err(|error| format!("Assurance computation did not run · {error:?}. Check both local inputs."));
                result.set(Some(computed));
            }, "Compute assurance locally" }

            if let Some(computed) = result() {
                match computed {
                    Err(message) => rsx! { output { class: "console-notice", aria_live: "assertive", "{message}" } },
                    Ok(local) => rsx! {
                        section { class: "console-panel assurance-context", aria_labelledby: "context-heading",
                            h2 { id: "context-heading", "Effective context VerificationContext" }
                            dl {
                                div { dt { "Local policy label" } dd { "{local.context_name}" } }
                                div { dt { "Context digest" } dd { class: "console-mono", "{hex::encode(local.context_id.as_bytes())}" } }
                                div { dt { "Context schema version" } dd { class: "console-mono", "{local.context.context_version.get()}" } }
                                div { dt { "Protocol version" } dd { class: "console-mono", "{local.context.protocol_version.major()}.{local.context.protocol_version.minor()}" } }
                                div { dt { "Evaluation time" } dd { class: "console-mono", "{local.context.evaluation_time} UTC seconds" } }
                                div { dt { "Verifier policy digest" } dd { class: "console-mono", "{hex::encode(local.context.verifier_policy_digest)}" } }
                                div { dt { "Trust package digest" } dd { class: "console-mono", "{hex::encode(local.context.trust_package_digest)}" } }
                                div { dt { "Revocation snapshot digest" } dd { class: "console-mono", "{hex::encode(local.context.revocation_snapshot_digest)}" } }
                                div { dt { "Algorithm policy digest" } dd { class: "console-mono", "{hex::encode(local.context.algorithm_policy_digest)}" } }
                                div { dt { "External evidence policy digest" } dd { class: "console-mono", "{hex::encode(local.context.external_evidence_policy_digest)}" } }
                                div { dt { "Assurance thresholds digest" } dd { class: "console-mono", "{hex::encode(local.context.assurance_thresholds_digest)}" } }
                                div { dt { "Extensions" } dd {
                                    if local.context.extensions.is_empty() { "None" }
                                    for extension in local.context.extensions.iter() {
                                        code { class: "reason-code", "{extension.registry_id} · {hex::encode(extension.parameters_digest)}" }
                                    }
                                } }
                            }
                        }
                        table { class: "assurance-table", tabindex: "0",
                            caption { "All 11 checks under the selected context" }
                            thead { tr {
                                th { scope: "col", "Dimension" }
                                th { scope: "col", "Status" }
                                th { scope: "col", "Reason codes and meaning" }
                                th { scope: "col", "Limitations" }
                            } }
                            tbody {
                                for dimension in local.assurance.dimensions.iter() {
                                    {
                                        let (plain, protocol) = dimension_labels(dimension.dimension);
                                        let (icon, status) = status_label(dimension.status);
                                        rsx! { tr {
                                            th { scope: "row", span { "{plain}" } code { "{protocol}" } }
                                            td { span { class: "assurance-status", aria_label: "{status}", span { aria_hidden: "true", "{icon}" } " {status}" } }
                                            td { p { "{reason_sentence(dimension.status)}" }
                                                for code in dimension.reason_codes.iter() { code { class: "reason-code", "{code}" } }
                                            }
                                            td { ul { for limitation in dimension.limitations.iter() { li { "{limitation}" } } } }
                                        } }
                                    }
                                }
                            }
                        }
                    },
                }
            }
            aside { class: "console-notice", aria_label: "Assurance limitations",
                strong { "What this record does not establish:" }
                span { " that the action was legally authorized beyond this approval policy, that every statement inside the evidence is factually true, or that all relevant events were captured. See Required evidence present and External source support." }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{dimension_labels, status_label};
    use csv_sdk::accountability::{AssuranceDimension, DimensionStatus};

    #[test]
    fn labels_preserve_uncertainty_and_rev03_external_wording() {
        assert_eq!(
            status_label(DimensionStatus::Indeterminate).1,
            "Cannot be determined"
        );
        assert_eq!(
            status_label(DimensionStatus::NotApplicable).1,
            "Not applicable"
        );
        assert_eq!(
            dimension_labels(AssuranceDimension::ExternalCorroboration).0,
            "External source support"
        );
    }
}
