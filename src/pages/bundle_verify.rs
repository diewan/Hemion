//! S-H2 — local Accountability bundle import and verification.

use crate::services::bundle_verifier::{import_and_verify, import_context};
use dioxus::prelude::*;

/// Imports a bundle and a separately supplied VerificationContext package, then computes locally.
#[component]
pub fn BundleVerify() -> Element {
    let mut bundle = use_signal(String::new);
    let mut context = use_signal(String::new);
    let mut result = use_signal(|| None::<String>);
    rsx! {
        section { class: "console-home bundle-verify", aria_labelledby: "bundle-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "bundle-title", "Import and verify bundle" }
            p { class: "console-lede", "Paste a local verification envelope and a separately obtained Context VerificationContext package. Hemion performs no network request." }
            div { class: "console-grid",
                label { class: "console-panel", r#for: "bundle-input",
                    h2 { "Bundle DisputeBundle" }
                    textarea { id: "bundle-input", rows: 12, value: "{bundle}", oninput: move |event| bundle.set(event.value()), placeholder: "Paste org.diewan.accountability.local-verification.v1 JSON" }
                }
                label { class: "console-panel", r#for: "context-input",
                    h2 { "Context VerificationContext" }
                    textarea { id: "context-input", rows: 12, value: "{context}", oninput: move |event| context.set(event.value()), placeholder: "Paste org.diewan.accountability.verification-context.v1 JSON" }
                }
            }
            button { class: "console-action", r#type: "button", disabled: bundle().is_empty() || context().is_empty(), onclick: move |_| {
                let message = match import_context(context().as_bytes()).and_then(|choice| {
                    let selected = choice.name.clone();
                    import_and_verify(bundle().as_bytes(), &[choice], &selected)
                }) {
                    Ok(local) => format!("{} · context {} · {}", crate::services::bundle_verifier::disposition_label(local.report.disposition), local.context_name, hex::encode(local.context_id.as_bytes())),
                    Err(error) => format!("Verification did not run · {error:?}. Check the bundle and context formats."),
                };
                result.set(Some(message));
            }, "Verify locally" }
            if let Some(message) = result() { output { class: "console-notice", aria_live: "polite", "{message}" } }
            aside { class: "console-notice", aria_label: "Verification limitations",
                strong { "What this result does not establish:" }
                span { " that every statement is factually true, that all relevant events were captured, or that authority exists beyond the selected policy." }
            }
        }
    }
}
