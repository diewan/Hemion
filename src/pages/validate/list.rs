//! Validate page list.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

// ===== Validate Pages =====
#[component]
pub fn Validate() -> Element {
    rsx! {
        div { class: "space-y-6",
            h1 { class: "text-2xl font-bold", "Validation" }

            div { class: "p-4 bg-amber-900/20 border border-amber-500/30 rounded-lg text-sm text-gray-300",
                "Transfer acceptance is runtime-only. The legacy entries below are retained as explicit unavailable states; none marks data as valid."
            }

            div { class: "grid grid-cols-1 sm:grid-cols-2 gap-4",
                Link { to: Route::ValidateConsignment {}, class: "{card_class()} p-6 hover:bg-gray-800/50 transition-colors block",
                    div { class: "flex items-center gap-3", span { class: "text-2xl", "\u{1F4C3}" }, div { h3 { class: "font-semibold", "Consignment" } p { class: "text-sm text-gray-400", "Unavailable: requires runtime acceptance" } } }
                }
                Link { to: Route::ValidateProof {}, class: "{card_class()} p-6 hover:bg-gray-800/50 transition-colors block",
                    div { class: "flex items-center gap-3", span { class: "text-2xl", "\u{1F50D}" }, div { h3 { class: "font-semibold", "Proof" } p { class: "text-sm text-gray-400", "Unavailable: requires runtime verification" } } }
                }
                Link { to: Route::ValidateSeal {}, class: "{card_class()} p-6 hover:bg-gray-800/50 transition-colors block",
                    div { class: "flex items-center gap-3", span { class: "text-2xl", "\u{1F512}" }, div { h3 { class: "font-semibold", "Seal" } p { class: "text-sm text-gray-400", "Unavailable: runtime owns seal status" } } }
                }
                Link { to: Route::ValidateCommitmentChain {}, class: "{card_class()} p-6 hover:bg-gray-800/50 transition-colors block",
                    div { class: "flex items-center gap-3", span { class: "text-2xl", "\u{1F517}" }, div { h3 { class: "font-semibold", "Commitment ChainId" } p { class: "text-sm text-gray-400", "Unavailable: requires observed-tip finality" } } }
                }
            }
        }
    }
}
