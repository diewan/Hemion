//! Cross-chain proof acceptance belongs exclusively to the runtime.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn VerifyCrossChainProof() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Proofs {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Cross-Chain Proof Verification" }
            }
            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "Cross-chain verification unavailable" }
                p { class: "text-sm text-gray-300",
                    "The wallet does not independently verify or accept cross-chain proofs. Use the runtime transfer flow, which returns a typed verification receipt."
                }
            }
        }
    }
}
