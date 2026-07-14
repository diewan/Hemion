//! Create seal page.
//!
//! A Seal cryptographically protects a Sanad. When you create a seal,
//! you're locking a Sanad's value for cross-chain transfer or secure storage.

use crate::context::{SanadStatus, use_wallet_context};
use crate::pages::common::*;
use crate::routes::Route;
use csv_hash::ChainId;
use dioxus::prelude::*;
use std::rc::Rc;

#[component]
pub fn CreateSeal() -> Element {
    let wallet_ctx = use_wallet_context();
    let mut selected_chain = use_signal(|| ChainId::new("bitcoin"));
    let mut selected_sanad_index = use_signal(|| 0usize);
    let mut result = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);

    // Get active sanads for the selected chain
    let sanads_for_chain: Vec<_> = wallet_ctx
        .sanads_for_chain(selected_chain.read().clone())
        .into_iter()
        .filter(|r| r.status == SanadStatus::Active)
        .collect();
    let has_sanads = !sanads_for_chain.is_empty();

    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::AssetSeals {}, class: "{btn_secondary_class()}", "\u{2190} Back" }
                h1 { class: "text-xl font-bold", "Create Seal" }
            }

            // Info box explaining seals
            div { class: "bg-blue-900/20 border border-blue-700/30 rounded-lg p-4",
                h3 { class: "text-sm font-medium text-blue-300 mb-2", "\u{2139} What is a Seal?" }
                p { class: "text-xs text-blue-200",
                    "A Seal cryptographically protects a Sanad. When you create a seal, you're locking a Sanad's value for cross-chain transfer or secure storage. \
                    The seal contains the proof that can be verified on another chain to mint a new Sanad."
                }
            }

            div { class: "{card_class()} p-6 space-y-5",
                {form_field("ChainId", chain_select(move |v: Rc<FormData>| {
                    if let Ok(c) = v.value().parse::<ChainId>() {
                        selected_chain.set(c);
                        selected_sanad_index.set(0); // Reset selection when chain changes
                    }
                }, selected_chain.read().clone()))}

                // Sanad selection
                {form_field("Sanad to Seal", rsx! {
                    if !has_sanads {
                        div { class: "text-sm text-red-400",
                            "No active sanads available for this chain. Create a Sanad first."
                        }
                    } else {
                        select {
                            class: "{input_mono_class()}",
                            onchange: move |evt| {
                                if let Ok(idx) = evt.value().parse::<usize>() {
                                    selected_sanad_index.set(idx);
                                }
                            },
                            for (idx, sanad) in sanads_for_chain.iter().enumerate() {
                                option { key: "sanad-{idx}", value: idx.to_string(), selected: idx == *selected_sanad_index.read(),
                                    {format!("{}... - Value: {}", &sanad.id[..16.min(sanad.id.len())], sanad.value)}
                                }
                            }
                        }
                    }
                })}

                if let Some(err) = error.read().as_ref() {
                    div { class: "p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-sm text-red-300", "{err}" }
                }

                if let Some(msg) = result.read().as_ref() {
                    div { class: "p-4 bg-green-900/30 border border-green-700/50 rounded-lg",
                        p { class: "text-green-300 font-mono text-sm break-all", "{msg}" }
                    }
                }

                button {
                    onclick: move |_| {
                        if !has_sanads {
                            error.set(Some("No sanads available to seal".to_string()));
                            return;
                        }

                        result.set(None);
                        error.set(Some(
                            "Seal creation requires a configured CSV runtime host. The wallet does not create or lock seals locally.".to_string(),
                        ));
                    },
                    disabled: !has_sanads,
                    class: if has_sanads { "{btn_full_primary_class()}" } else { "{btn_full_primary_class()} opacity-50 cursor-not-allowed" },
                    if has_sanads { "Create Seal" } else { "No Sanads Available" }
                }
            }
        }
    }
}
