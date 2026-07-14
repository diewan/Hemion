//! Direct proof acceptance is unavailable outside the runtime verification flow.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn VerifyProof() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Proofs {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Proof Verification" }
            }
            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "Proof verification unavailable" }
                p { class: "text-sm text-gray-300",
                    "Direct wallet verification is disabled. A proof is accepted only by the runtime after trusted signer binding, inclusion, observed-tip finality, and replay checks."
                }
                p { class: "text-sm text-gray-400", "No pasted or structural-only proof is treated as valid." }
            }
        }
    }
}
