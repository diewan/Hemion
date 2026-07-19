//! G-09 — browsable conformance vectors and local replay demonstration.

use crate::services::{
    bundle_verifier::import_context,
    fixture_lab::{FIXTURE_CASES, run_case},
};
use dioxus::prelude::*;

#[component]
pub fn FixtureLab() -> Element {
    let mut selected = use_signal(|| "valid".to_owned());
    let mut bundle = use_signal(String::new);
    let mut first_context = use_signal(String::new);
    let mut replay_context = use_signal(String::new);
    let mut result = use_signal(|| None::<String>);
    rsx! {
        section { class: "console-home fixture-lab", aria_labelledby: "fixture-title",
            p { class: "console-eyebrow", "HEMION / LOCAL INSTRUMENT" }
            h1 { id: "fixture-title", "Fixture lab" }
            p { class: "console-lede", "Browse the published Parwana accountability-v0.1 corpus, then compare its expected result with an actual result computed locally by the pinned verifier." }

            div { class: "fixture-layout",
                nav { class: "fixture-list", aria_label: "Conformance vectors",
                    for case in FIXTURE_CASES {
                        button { r#type: "button", class: "console-action", aria_pressed: selected() == case.id,
                            onclick: move |_| { selected.set(case.id.to_owned()); result.set(None); },
                            span { "{case.title}" }
                            small { "{case.expected}" }
                        }
                    }
                }
                article { class: "console-panel", aria_live: "polite",
                    for case in FIXTURE_CASES.iter().filter(|case| case.id == selected()) {
                        p { class: "console-eyebrow", "VECTOR {case.id}" }
                        h2 { "{case.title}" }
                        dl { div { dt { "Expected" } dd { "{case.expected}" } } }
                        p { "{case.explanation}" }
                    }
                }
            }

            label { class: "inspector-import", r#for: "fixture-bundle", "Local bundle envelope"
                textarea { id: "fixture-bundle", rows: 8, value: "{bundle}", oninput: move |e| bundle.set(e.value()), placeholder: "Paste the selected vector's local-verification.v1 envelope" }
            }
            label { class: "inspector-import", r#for: "fixture-context", "Selected verification context"
                textarea { id: "fixture-context", rows: 8, value: "{first_context}", oninput: move |e| first_context.set(e.value()), placeholder: "Paste its verification-context.v1 package" }
            }
            button { class: "console-action", r#type: "button", disabled: bundle().is_empty() || first_context().is_empty(), onclick: move |_| {
                let message = import_context(first_context().as_bytes())
                    .and_then(|context| run_case(&selected(), bundle().as_bytes(), context))
                    .map(|comparison| format!("Expected: {} · Actual: {} · {}", comparison.expected, comparison.actual, if comparison.matches { "Matches corpus" } else { "Does not match corpus" }))
                    .unwrap_or_else(|error| format!("Actual result unavailable · {error:?}. The imported inputs were rejected."));
                result.set(Some(message));
            }, "Run selected vector locally" }
            if let Some(message) = result() { output { class: "console-notice", aria_live: "polite", "{message}" } }

            section { class: "console-panel replay-demo", aria_labelledby: "replay-title",
                h2 { id: "replay-title", "Replay: first acceptance, second rejection" }
                p { "Use the same bundle twice with two independently supplied contexts. The first context must report a fresh mandate; the second must report that the mandate is replayed." }
                ol {
                    li { strong { "First attempt — accepted once." } " A fresh replay-journal status can satisfy the single-use check when every other requirement is met." }
                    li { strong { "Second attempt — rejected." } " The replayed status produces ReplayDetected. Hemion demonstrates verification only; it never dispatches to a provider." }
                }
                label { class: "inspector-import", r#for: "replay-context", "Second-run replay context"
                    textarea { id: "replay-context", rows: 8, value: "{replay_context}", oninput: move |e| replay_context.set(e.value()), placeholder: "Paste a context whose replay status is replayed" }
                }
                button { class: "console-action", r#type: "button", disabled: bundle().is_empty() || first_context().is_empty() || replay_context().is_empty(), onclick: move |_| {
                    let run = || -> Result<String, crate::services::bundle_verifier::LocalVerificationError> {
                        let first = run_case("valid", bundle().as_bytes(), import_context(first_context().as_bytes())?)?;
                        let second = run_case("replayed", bundle().as_bytes(), import_context(replay_context().as_bytes())?)?;
                        if !first.matches || !second.matches { return Ok(format!("Replay demonstration did not match · first actual: {} · second actual: {}", first.actual, second.actual)); }
                        Ok(format!("First acceptance: {} · Second rejection: {} · no provider dispatch was performed", first.actual, second.actual))
                    };
                    result.set(Some(run().unwrap_or_else(|error| format!("Replay demonstration rejected · {error:?}"))));
                }, "Run replay comparison" }
            }
            aside { class: "console-notice", aria_label: "Fixture limitations", strong { "Limits:" } " Expected values describe the pinned corpus. Actual values appear only after local verification. Missing evidence never establishes non-occurrence, and a valid result establishes integrity under a context—not factual truth." }
        }
    }
}
