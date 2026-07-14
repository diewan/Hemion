//! Verify seal page.

use crate::context::use_wallet_context;
use crate::pages::common::*;
use crate::routes::Route;
use csv_hash::ChainId;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn VerifySeal() -> Element {
    let _wallet_ctx = use_wallet_context();
    let mut selected_chain = use_signal(|| ChainId::new("bitcoin"));
    let mut seal_ref = use_signal(String::new);
    let mut result = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Seals {}, class: "{btn_secondary_class()}", "\u{2190} Back" }
                h1 { class: "text-xl font-bold", "Verify Seal" }
            }

            div { class: "{card_class()} p-6 space-y-5",
                {form_field("ChainId", chain_select(move |v: Rc<FormData>| {
                    if let Ok(c) = v.value().parse::<ChainId>() { selected_chain.set(c); }
                }, selected_chain.read().clone()))}

                {form_field("Seal Reference (hex)", rsx! {
                    input {
                        value: "{seal_ref.read()}",
                        oninput: move |evt| { seal_ref.set(evt.value()); result.set(None); },
                        class: "{input_mono_class()}",
                        r#type: "text"
                    }
                })}

                if let Some(message) = result.read().as_ref() {
                    div { class: "p-4 bg-yellow-900/30 border border-yellow-700/50 rounded-lg",
                        p { class: "text-yellow-300", "{message}" }
                    }
                }

                button {
                    onclick: move |_| {
                        result.set(Some("Seal status must be obtained from the configured CSV runtime host. Cached wallet records are not authority for consumption or availability.".to_string()));
                    },
                    class: "{btn_full_primary_class()}",
                    "Verify Seal"
                }
            }
        }
    }
}
