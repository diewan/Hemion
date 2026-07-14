//! Proof construction is unavailable until the wallet can submit a typed
//! request to the runtime and render its receipt.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn GenerateProof() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Proofs {}, class: "{btn_secondary_class()}", "← Back" }
                h1 { class: "text-xl font-bold", "Proof Construction" }
            }
            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "Proof construction unavailable" }
                p { class: "text-sm text-gray-300",
                    "This wallet cannot construct a proof locally. It will not create synthetic proof data or mark a proof as generated."
                }
                p { class: "text-sm text-gray-400",
                    "Start a supported transfer through the runtime. Proof records shown here must originate from typed runtime receipts."
                }
            }
        }
    }
}
