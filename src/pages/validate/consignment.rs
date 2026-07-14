//! Validate consignment page.
//!
//! The legacy consignment validator is retired. Consignments must be validated
//! by the runtime-backed acceptance flow, which owns finality and replay checks.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

/// Validation step result for UI display.
#[derive(Clone, PartialEq)]
struct ValidationStepUI {
    name: String,
    passed: bool,
    details: String,
}

/// Overall validation status.
#[derive(Clone, PartialEq)]
enum ValidationStatus {
    Idle,
    Validating,
    Passed(Vec<ValidationStepUI>),
    Failed(Vec<ValidationStepUI>),
    Error(String),
}

pub fn ValidateConsignment() -> Element {
    let mut consignment_json = use_signal(String::new);
    let mut status = use_signal(|| ValidationStatus::Idle);

    rsx! {
        div { class: "max-w-4xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Validate {}, class: "{btn_secondary_class()}", "\u{2190} Back" }
                h1 { class: "text-xl font-bold", "Validate Consignment" }
            }

            // Info box explaining validation
            div { class: "bg-blue-900/20 border border-blue-700/30 rounded-lg p-4",
                h3 { class: "text-sm font-medium text-blue-300 mb-2", "\u{2139} 5-Step Validation" }
                p { class: "text-xs text-blue-200 mb-2",
                    "Every consignment undergoes rigorous validation before being accepted into your wallet:"
                }
                ol { class: "text-xs text-blue-200 list-decimal list-inside space-y-1",
                    li { "Structural: Version, schema, and required fields" }
                    li { "Commitment ChainId: Genesis to latest integrity" }
                    li { "Seal Consumption: Double-spend detection" }
                    li { "State Transitions: Valid evolution rules" }
                    li { "Final Acceptance: All checks must pass" }
                }
            }

            div { class: "{card_class()} p-6 space-y-5",
                {form_field("Consignment JSON", rsx! {
                    textarea {
                        class: "{input_mono_class()} h-48",
                        placeholder: "Paste consignment JSON here...",
                        value: "{consignment_json.read()}",
                        oninput: move |evt| consignment_json.set(evt.value().to_string()),
                    }
                })}

                // Validation results display
                match status.read().clone() {
                    ValidationStatus::Idle => rsx!{},
                    ValidationStatus::Validating => rsx!{
                        div { class: "p-4 bg-blue-900/30 border border-blue-700/50 rounded-lg",
                            p { class: "text-blue-300", "Validating..." }
                        }
                    },
                    ValidationStatus::Passed(steps) => rsx!{
                        div { class: "space-y-3",
                            div { class: "p-4 bg-green-900/30 border border-green-700/50 rounded-lg",
                                p { class: "text-green-300 font-medium", "\u{2713} Consignment Valid" }
                                p { class: "text-green-200 text-sm mt-1",
                                    "All 5 validation steps passed. This consignment is safe to accept."
                                }
                            }
                            {validation_steps_list(steps)}
                        }
                    },
                    ValidationStatus::Failed(steps) => rsx!{
                        div { class: "space-y-3",
                            div { class: "p-4 bg-red-900/30 border border-red-700/50 rounded-lg",
                                p { class: "text-red-300 font-medium", "\u{2717} Consignment Invalid" }
                                p { class: "text-red-200 text-sm mt-1",
                                    "One or more validation steps failed. Do not accept this consignment."
                                }
                            }
                            {validation_steps_list(steps)}
                        }
                    },
                    ValidationStatus::Error(msg) => rsx!{
                        div { class: "p-4 bg-red-900/30 border border-red-700/50 rounded-lg",
                            p { class: "text-red-300 font-medium", "Validation Error" }
                            p { class: "text-red-200 text-sm mt-1", "{msg}" }
                        }
                    },
                }

                button {
                    onclick: move |_| {
                        let json = consignment_json.read().clone();
                        if json.trim().is_empty() {
                            status.set(ValidationStatus::Error("Please enter consignment JSON".to_string()));
                            return;
                        }

                        status.set(ValidationStatus::Validating);

                        // Run validation
                        let result = validate_consignment_json(&json);
                        match result {
                            Ok(steps) => {
                                let all_passed = steps.iter().all(|s| s.passed);
                                if all_passed {
                                    status.set(ValidationStatus::Passed(steps));
                                } else {
                                    status.set(ValidationStatus::Failed(steps));
                                }
                            }
                            Err(e) => {
                                status.set(ValidationStatus::Error(e));
                            }
                        }
                    },
                    disabled: matches!(*status.read(), ValidationStatus::Validating),
                    class: if matches!(*status.read(), ValidationStatus::Validating) {
                        "{btn_full_primary_class()} opacity-50 cursor-not-allowed"
                    } else {
                        "{btn_full_primary_class()}"
                    },
                    if matches!(*status.read(), ValidationStatus::Validating) {
                        "Validating..."
                    } else {
                        "Validate Consignment"
                    }
                }
            }
        }
    }
}

/// Render validation steps list.
fn validation_steps_list(steps: Vec<ValidationStepUI>) -> Element {
    rsx! {
        div { class: "space-y-2",
            for step in steps {
                div {
                    class: if step.passed {
                        "p-3 bg-green-900/20 border border-green-700/30 rounded-lg"
                    } else {
                        "p-3 bg-red-900/20 border border-red-700/30 rounded-lg"
                    },
                    div { class: "flex items-center gap-2",
                        span {
                            class: if step.passed { "text-green-400" } else { "text-red-400" },
                            if step.passed { "\u{2713}" } else { "\u{2717}" }
                        }
                        span { class: "font-medium", "{step.name}" }
                    }
                    p { class: "text-sm text-gray-400 mt-1", "{step.details}" }
                }
            }
        }
    }
}

/// Reject legacy JSON consignments rather than presenting structural parsing as
/// a validated, safe-to-accept consignment.
fn validate_consignment_json(json: &str) -> Result<Vec<ValidationStepUI>, String> {
    if json.trim().is_empty() {
        return Err("A consignment is required for validation".to_string());
    }

    Err(
        "Consignment validation is unavailable in the wallet until it is routed through the runtime acceptance flow. No legacy or structural-only validation is accepted."
            .to_string(),
    )
}

#[cfg(test)]
mod tests {
    use super::validate_consignment_json;

    #[test]
    fn legacy_consignment_validation_fails_closed() {
        let result = validate_consignment_json(r#"{\"version\": 1}"#);
        assert!(result.is_err());
    }
}
