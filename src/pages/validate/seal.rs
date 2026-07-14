//! Legacy seal validation is deliberately disabled.

use crate::pages::common::*;
use crate::routes::Route;
use dioxus::prelude::*;

#[component]
pub fn ValidateSeal() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-6",
            Link { to: Route::Validate {}, class: "{btn_secondary_class()}", "← Back" }
            h1 { class: "text-xl font-bold", "Seal Validation" }
            div { class: "{card_class()} p-6 border-amber-500/30 space-y-3",
                h2 { class: "text-lg font-semibold text-amber-300", "Unavailable outside the runtime" }
                p { class: "text-sm text-gray-300", "Seal consumption cannot be inferred from a reference string. The runtime owns the chain-backed registry and replay protection." }
            }
        }
    }
}
