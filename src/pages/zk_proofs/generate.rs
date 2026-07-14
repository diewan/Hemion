//! Zero-knowledge proof generation is deliberately unavailable in the wallet.
//!
//! The protocol has typed ZK proof envelopes, but no wallet-accessible prover
//! can establish the required inclusion, finality, and pairing-verification
//! guarantees. The UI must not fabricate witnesses or proofs.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

const ZK_UNAVAILABLE_MESSAGE: &str = "ZK proof generation is unavailable: this wallet has no \
    supported prover and will not create synthetic witnesses or proofs.";

#[component]
pub fn ZkGenerateProof() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Proofs {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Generate ZK Proof" }
            }

            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "ZK proof generation unavailable" }
                p { class: "text-sm text-gray-300", "{ZK_UNAVAILABLE_MESSAGE}" }
                p { class: "text-sm text-gray-400",
                    "Use the supported runtime proof and verification flow. A ZK proof may only be presented when a real prover and the verifier backend are available."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ZK_UNAVAILABLE_MESSAGE;

    #[test]
    fn generation_reports_that_synthetic_proofs_are_not_available() {
        assert!(ZK_UNAVAILABLE_MESSAGE.contains("unavailable"));
        assert!(ZK_UNAVAILABLE_MESSAGE.contains("will not create synthetic"));
    }
}
