//! Offline Verification Mode - Phase 4.4 Competitive Advantage
//!
//! This page allows users to paste or upload a canonical `ProofBundle` and
//! inspect it without RPC calls.
//!
//! It is deliberately not an acceptance authority: checking live seal
//! consumption, observed-chain finality, and trusted signer configuration is
//! delegated to the SDK/runtime verification flow.

use crate::pages::common::*;
use crate::routes::Route;
use csv_protocol::ProofBundle;
use csv_verifier::verify_proof;
use dioxus::html::FileData;
use dioxus::prelude::*;

/// Handle file upload with validation and error handling
async fn handle_file_upload(
    file_data: &FileData,
    mut proof_input: Signal<String>,
    mut file_error: Signal<Option<String>>,
) {
    let file_name = file_data.name();
    let file_size = file_data.size() as f64;

    // Validate file size (10MB limit)
    const MAX_FILE_SIZE: f64 = 10.0 * 1024.0 * 1024.0; // 10MB in bytes
    if file_size > MAX_FILE_SIZE {
        file_error.set(Some(format!(
            "File too large: {} (max 10MB allowed)",
            format_file_size(file_size)
        )));
        return;
    }

    // Validate file extension
    let valid_extensions = ["proof", "hex", "txt"];
    let extension = file_name
        .split('.')
        .next_back()
        .unwrap_or("")
        .to_lowercase();

    if !valid_extensions.contains(&extension.as_str()) {
        file_error.set(Some(format!(
            "Unsupported file type: .{} (supported: .proof, .hex, .txt)",
            extension
        )));
        return;
    }

    // Clear any previous errors
    file_error.set(None);

    // Log file selection
    web_sys::console::log_2(
        &"Processing file:".into(),
        &format!("{} ({})", file_name, format_file_size(file_size)).into(),
    );

    // Read file content as raw text using Dioxus FileData API
    match file_data.read_string().await {
        Ok(text) if !text.is_empty() => {
            // Canonical proof bundles are hex-encoded binary, not JSON. The
            // canonical decoder performs the authoritative format validation
            // when the user starts inspection.
            if !text.trim().is_empty() {
                let text_len = text.len();
                proof_input.set(text);
                web_sys::console::log_1(
                    &format!("Successfully loaded {} bytes from {}", text_len, file_name).into(),
                );
            } else {
                file_error.set(Some(
                    "File does not contain a canonical proof bundle".to_string(),
                ));
                web_sys::console::log_1(&"Empty canonical proof file".into());
            }
        }
        Ok(_) => {
            file_error.set(Some("File content is empty".to_string()));
            web_sys::console::log_1(&"File content is empty".into());
        }
        Err(e) => {
            file_error.set(Some(format!("Failed to read file content: {}", e)));
            web_sys::console::log_1(&"Failed to read file content".into());
        }
    }
}

/// Format file size in human readable format
fn format_file_size(bytes: f64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
    let mut size = bytes;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Offline verification page - pure cryptographic verification
#[component]
pub fn OfflineVerify() -> Element {
    let mut proof_input = use_signal(String::new);
    let mut verification_result = use_signal(|| None::<VerificationResult>);
    let mut is_verifying = use_signal(|| false);
    let mut is_dragging = use_signal(|| false);
    let mut file_error = use_signal(|| None::<String>);

    rsx! {
        div { class: "max-w-4xl mx-auto space-y-6",
            // Header
            div { class: "flex items-center gap-3",
                Link { to: Route::Validate {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Offline Verification" }
            }

            // Explanation card
            div { class: "p-4 bg-gradient-to-r from-blue-900/30 to-purple-900/30 \
                          border border-blue-500/30 rounded-lg",
                h2 { class: "text-sm font-semibold text-blue-300 mb-2",
                    "✨ CSV Competitive Advantage"
                }
                p { class: "text-sm text-gray-300",
                    "Decode a canonical proof bundle locally. Transfer acceptance still requires \
                     the SDK/runtime to verify trusted signers, observed finality, and seal replay state."
                }
                p { class: "text-xs text-gray-400 mt-2",
                    "This is what makes CSV different from traditional bridges."
                }
            }

            // Input section with drag-and-drop
            div { class: "{card_class()} p-6",
                h2 { class: "text-lg font-semibold mb-4", "Import Proof Bundle" }

                // Drag and drop area
                div {
                    class: "border-2 border-dashed border-gray-600 rounded-lg p-8 text-center transition-colors duration-200",
                    class: if is_dragging() { "border-blue-500 bg-blue-900/20" } else { "border-gray-600" },
                    ondragover: move |e| {
                        e.prevent_default();
                        is_dragging.set(true);
                    },
                    ondragleave: move |_| {
                        is_dragging.set(false);
                    },
                    ondrop: move |e| {
                        e.prevent_default();
                        is_dragging.set(false);

                        let files = e.data_transfer().files();
                        if let Some(file_data) = files.first() {
                            let file_data = file_data.clone();
                            let proof_input_clone = proof_input;
                            let file_error_clone = file_error;
                            use_future(move || {
                                let file_data_clone = file_data.clone();
                                async move {
                                    handle_file_upload(&file_data_clone, proof_input_clone, file_error_clone).await;
                                }
                            });
                        }
                    },

                    div { class: "space-y-4",
                        div { class: "text-4xl", "📄" }
                        div {
                            h3 { class: "text-lg font-medium mb-2", "Drop your proof file here" }
                            p { class: "text-sm text-gray-400", "or click to browse" }
                        }

                        input {
                            r#type: "file",
                            accept: ".proof,.hex,.txt",
                            class: "hidden",
                            id: "file-input",
                            onchange: move |e| {
                                if let Some(file_data) = e.files().first() {
                                    let file_data = file_data.clone();
                                    let proof_input_clone = proof_input;
                                    let file_error_clone = file_error;
                                    use_future(move || {
                                        let file_data_clone = file_data.clone();
                                        async move {
                                            handle_file_upload(&file_data_clone, proof_input_clone, file_error_clone).await;
                                        }
                                    });
                                }
                            }
                        }
                    }
                }

                // Manual input option
                div { class: "mt-6 pt-6 border-t border-gray-800",
                    h3 { class: "text-md font-medium mb-3", "Or paste manually:" }

                    textarea {
                        class: "w-full h-64 p-4 bg-gray-900 border border-gray-700 rounded-lg \
                               font-mono text-sm resize-none focus:border-blue-500 focus:outline-none",
                        placeholder: "Paste hex-encoded canonical ProofBundle bytes here...",
                        value: "{proof_input}",
                        oninput: move |e| {
                            proof_input.set(e.value());
                            file_error.set(None); // Clear file error when typing
                        },
                    }
                }

                // Action buttons
                div { class: "flex gap-3 mt-4",
                    button {
                        class: "{btn_primary_class()}",
                        disabled: proof_input().is_empty() || is_verifying(),
                        onclick: move |_| {
                            is_verifying.set(true);
                            file_error.set(None);
                            // This is a fail-closed inspection only; it never accepts a proof.
                            let result = perform_offline_verification(&proof_input());
                            verification_result.set(Some(result));
                            is_verifying.set(false);
                        },
                        if is_verifying() {
                            "⏳ Verifying..."
                        } else {
                            "🔍 Verify Offline"
                        }
                    }
                    button {
                        class: "{btn_secondary_class()}",
                        onclick: move |_| {
                            proof_input.set(String::new());
                            verification_result.set(None);
                            file_error.set(None);
                        },
                        "Clear"
                    }
                }
            }

            // Verification result
            if let Some(result) = verification_result() {
                {verification_result_section(&result)}
            }

            // How it works
            {how_it_works_section()}
        }
    }
}

/// Verification result structure
#[derive(Clone, PartialEq)]
struct VerificationResult {
    success: bool,
    steps: Vec<VerificationStep>,
    summary: String,
}

/// Individual verification step
#[derive(Clone, PartialEq)]
struct VerificationStep {
    name: String,
    passed: bool,
    details: String,
}

/// Inspect a canonical proof bundle without creating a local acceptance path.
///
/// The canonical verifier is still invoked so malformed bundles and missing
/// authorization material are rejected consistently. This page intentionally
/// has no approved signer set or live replay/finality evidence, therefore it
/// can never report a proof as accepted.
fn perform_offline_verification(input: &str) -> VerificationResult {
    let mut steps = Vec::new();

    const MAX_CANONICAL_PROOF_INPUT_BYTES: usize = 10 * 1024 * 1024;
    if input.len() > MAX_CANONICAL_PROOF_INPUT_BYTES {
        steps.push(VerificationStep {
            name: "Decode Canonical Proof Bundle".to_string(),
            passed: false,
            details: format!(
                "Proof input exceeds the resource limit of {} MiB before decoding.",
                MAX_CANONICAL_PROOF_INPUT_BYTES / (1024 * 1024)
            ),
        });
        return VerificationResult {
            success: false,
            steps,
            summary: "Verification failed: proof input exceeds the resource limit.".to_string(),
        };
    }

    // Step 1: Decode the canonical proof representation. JSON is not a
    // canonical proof format and must never be used as protocol input.
    let trimmed = input.trim();
    let encoded = match trimmed.strip_prefix("0x") {
        Some(encoded) => encoded,
        None => trimmed,
    };
    let bundle_result = hex::decode(encoded)
        .map_err(|error| format!("Invalid hex-encoded canonical proof: {error}"))
        .and_then(|bytes| ProofBundle::from_canonical_bytes(&bytes));

    let parsed = bundle_result.is_ok();
    steps.push(VerificationStep {
        name: "Decode Canonical Proof Bundle".to_string(),
        passed: parsed,
        details: if parsed {
            "Canonical ProofBundle decoded successfully.".to_string()
        } else {
            format!(
                "Invalid canonical proof: {}",
                bundle_result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_default()
            )
        },
    });

    if !parsed {
        return VerificationResult {
            success: false,
            steps,
            summary: "Verification failed: Invalid proof bundle format".to_string(),
        };
    }

    let bundle = match bundle_result {
        Ok(bundle) => bundle,
        Err(error) => {
            return VerificationResult {
                success: false,
                steps,
                summary: format!("Verification failed: {error}"),
            };
        }
    };

    // Invoke the canonical verifier. Offline inspection has neither
    // trusted signer keys nor a chain-backed seal registry, so it must fail
    // closed rather than treating unavailable evidence as a successful check.
    let verification_result = verify_proof(&bundle, |_seal_id| true, bundle.signature_scheme, &[]);

    let canonical_valid = verification_result.is_valid;
    let errors = verification_result
        .errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    steps.push(VerificationStep {
        name: "Canonical Verifier".to_string(),
        passed: canonical_valid,
        details: if canonical_valid {
            "The canonical verifier completed. This inspector does not grant transfer acceptance."
                .to_string()
        } else {
            format!(
                "Rejected by the canonical verifier: {}",
                if errors.is_empty() {
                    "no verifier error was supplied".to_string()
                } else {
                    errors.join("; ")
                }
            )
        },
    });

    // The UI cannot substitute an empty verifier set, a local confirmation
    // threshold, or a guessed seal status for the runtime's trusted inputs.
    steps.push(VerificationStep {
        name: "SDK/Runtime Acceptance Required".to_string(),
        passed: false,
        details: "Not accepted offline: the runtime must establish approved signer binding, chain-native inclusion, observed-tip finality, and seal replay protection."
            .to_string(),
    });

    VerificationResult {
        success: false,
        steps,
        summary: "Offline inspection does not accept proof bundles. Submit the bundle to the SDK/runtime verification flow before continuing.".to_string(),
    }
}

/// Verification result display
fn verification_result_section(result: &VerificationResult) -> Element {
    let status_color = if result.success {
        "var(--proof-valid)"
    } else {
        "var(--proof-invalid)"
    };
    let status_bg = if result.success {
        "bg-green-900/20 border-green-500/30"
    } else {
        "bg-red-900/20 border-red-500/30"
    };

    rsx! {
        div { class: "{card_class()} p-6",
            h2 { class: "text-lg font-semibold mb-4", "Verification Result" }

            // Summary
            div { class: "p-4 {status_bg} border rounded-lg mb-4",
                p { class: "font-semibold flex items-center gap-2",
                    style: "color: {status_color}",
                    if result.success { "✓" } else { "✗" }
                    "{&result.summary}"
                }
            }

            // Step-by-step results
            div { class: "space-y-3",
                h3 { class: "text-sm font-semibold text-gray-400 uppercase", "Verification Steps" }

                for (i, step) in result.steps.iter().enumerate() {
                    div { class: "flex items-start gap-3 p-3 bg-gray-800/50 rounded-lg",
                        div { class: "flex-shrink-0 mt-0.5",
                            if step.passed {
                                span { class: "text-green-500", "✓" }
                            } else {
                                span { class: "text-red-500", "✗" }
                            }
                        }
                        div { class: "flex-1",
                            p { class: "font-medium", "{i + 1}. {&step.name}" }
                            p { class: "text-sm text-gray-400", "{&step.details}" }
                        }
                    }
                }
            }

            // Trust indicators
            div { class: "mt-4 p-3 bg-gray-800/50 rounded-lg",
                h4 { class: "text-sm font-medium mb-2", "Inspection Scope" }
                div { class: "flex flex-wrap gap-4 text-xs",
                    div { class: "flex items-center gap-1",
                        span { class: "text-blue-500", "●" }
                        span { class: "text-gray-400", "Canonical decoding" }
                    }
                    div { class: "flex items-center gap-1",
                        span { class: "text-yellow-500", "●" }
                        span { class: "text-gray-400", "No live replay check" }
                    }
                    div { class: "flex items-center gap-1",
                        span { class: "text-red-500", "●" }
                        span { class: "text-gray-400", "Not transfer acceptance" }
                    }
                }
            }
        }
    }
}

/// How offline verification works section
fn how_it_works_section() -> Element {
    rsx! {
        div { class: "{card_class()} p-6",
            h2 { class: "text-lg font-semibold mb-4", "How Offline Verification Works" }

            div { class: "space-y-4",
                div { class: "flex gap-4",
                    div { class: "flex-shrink-0 w-8 h-8 bg-blue-500/20 rounded-full \
                                  flex items-center justify-center",
                        span { class: "text-blue-400 font-semibold", "1" }
                    }
                    div {
                        h3 { class: "font-medium", "Parse" }
                        p { class: "text-sm text-gray-400",
                            "The proof bundle is parsed and validated for correct structure."
                        }
                    }
                }

                div { class: "flex gap-4",
                    div { class: "flex-shrink-0 w-8 h-8 bg-blue-500/20 rounded-full \
                                  flex items-center justify-center",
                        span { class: "text-blue-400 font-semibold", "2" }
                    }
                    div {
                        h3 { class: "font-medium", "Canonical verifier" }
                        p { class: "text-sm text-gray-400",
                            "The shared verifier evaluates the bundle and reports any rejection."
                        }
                    }
                }

                div { class: "flex gap-4",
                    div { class: "flex-shrink-0 w-8 h-8 bg-blue-500/20 rounded-full \
                                  flex items-center justify-center",
                        span { class: "text-blue-400 font-semibold", "3" }
                    }
                    div {
                        h3 { class: "font-medium", "Runtime acceptance" }
                        p { class: "text-sm text-gray-400",
                            "The SDK/runtime supplies trusted signer configuration, observed chain tips, and seal replay state."
                        }
                    }
                }

                div { class: "flex gap-4",
                    div { class: "flex-shrink-0 w-8 h-8 bg-green-500/20 rounded-full \
                                  flex items-center justify-center",
                        span { class: "text-green-400 font-semibold", "✓" }
                    }
                    div {
                        h3 { class: "font-medium", "Result" }
                        p { class: "text-sm text-gray-400",
                            "Local inspection never marks a proof valid for transfer acceptance."
                        }
                    }
                }
            }

            // Comparison with bridges
            div { class: "mt-6 p-4 bg-gray-800/50 rounded-lg",
                h3 { class: "font-medium mb-2", "CSV vs Traditional Bridges" }
                div { class: "grid grid-cols-2 gap-4 text-sm",
                    div {
                        p { class: "text-gray-500 mb-1", "Traditional Bridge" }
                        ul { class: "space-y-1 text-gray-400",
                            li { "• Requires RPC to source chain" }
                            li { "• Trusts bridge operator" }
                            li { "• Can't verify offline" }
                            li { "• Receipt = trust us" }
                        }
                    }
                    div {
                        p { class: "text-blue-400 mb-1", "CSV Protocol" }
                        ul { class: "space-y-1 text-blue-300",
                            li { "• Canonical proof data is portable" }
                            li { class: "font-semibold", "• Verification is fail-closed" }
                            li { class: "font-semibold", "• Runtime establishes live finality" }
                            li { class: "font-semibold", "• No UI-local transfer authority" }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::perform_offline_verification;

    #[test]
    fn rejects_malformed_canonical_proof_input() {
        let result = perform_offline_verification("not hex");

        assert!(!result.success);
        assert!(!result.steps[0].passed);
    }

    #[test]
    fn rejects_oversized_canonical_proof_input_before_decoding() {
        let result = perform_offline_verification(&"00".repeat(5 * 1024 * 1024 + 1));

        assert!(!result.success);
        assert!(result.steps[0].details.contains("resource limit"));
    }
}
