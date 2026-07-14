//! Zero-knowledge proof verification is deliberately unavailable in the wallet.
//!
//! `csv-verifier` fails closed until a real Groth16 pairing backend is supplied.
//! The wallet therefore must not parse legacy JSON envelopes or claim that an
//! adapter-local structural check is cryptographic verification.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

const ZK_UNAVAILABLE_MESSAGE: &str = "ZK proof verification is unavailable: no supported pairing \
    verifier is configured, so this wallet accepts no ZK proofs.";

#[component]
pub fn ZkVerifyProof() -> Element {
    rsx! {
        div { class: "max-w-4xl mx-auto space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Proofs {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Verify ZK Proof" }
            }

            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "ZK proof verification unavailable" }
                p { class: "text-sm text-gray-300", "{ZK_UNAVAILABLE_MESSAGE}" }
                p { class: "text-sm text-gray-400",
                    "Proof input is disabled rather than performing structural-only or adapter-local verification. Use a supported runtime verification receipt instead."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ZK_UNAVAILABLE_MESSAGE;

    #[test]
    fn verification_fails_closed_when_no_pairing_backend_exists() {
        assert!(ZK_UNAVAILABLE_MESSAGE.contains("unavailable"));
        assert!(ZK_UNAVAILABLE_MESSAGE.contains("accepts no ZK proofs"));
    }
}
