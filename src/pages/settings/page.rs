//! Settings page.

use crate::context::use_wallet_context;
use crate::pages::common::*;
use crate::routes::Route;
use csv_sdk::protocol::version::PROTOCOL_VERSION;
use dioxus::prelude::*;

pub fn Settings() -> Element {
    let wallet_ctx = use_wallet_context();
    let mut show_clear_data = use_signal(|| false);
    let mut show_unlock = use_signal(|| false);
    let mut unlock_passphrase = use_signal(String::new);
    let mut erase_confirmation = use_signal(String::new);
    let mut custody_error = use_signal(|| None::<String>);
    let is_initialized = wallet_ctx.is_initialized();
    let has_wallet = is_initialized;
    let is_locked = wallet_ctx.is_locked();

    // Clone for closures
    let mut ctx_clear = wallet_ctx.clone();

    rsx! {
        div { class: "max-w-2xl space-y-6 stagger-children",
            h1 { class: "text-2xl font-bold", "Settings" }

            // Wallet section
            div { class: "{card_class()} overflow-hidden",
                div { class: "{card_header_class()}",
                    h3 { class: "font-semibold text-sm", "Wallet" }
                }
                div { class: "p-6 space-y-4",
                    // Status
                    div { class: "flex items-center justify-between",
                        span { class: "text-sm text-gray-400", "Status" }
                        div { class: "flex items-center gap-2",
                            span { class: "w-2 h-2 rounded-full", class: if !has_wallet { "bg-gray-500" } else if is_locked { "bg-amber-500" } else { "bg-green-500 status-online" } }
                            span { class: "text-sm", if !has_wallet { "No local wallet data" } else if is_locked { "Locked — metadata retained" } else { "Unlocked signing session" } }
                        }
                    }

                    div { class: "flex items-center justify-between",
                        span { class: "text-sm text-gray-400", "Initialized" }
                        span { class: "text-sm", if is_initialized { "Yes" } else { "No" } }
                    }

                    div { class: "flex gap-3 pt-2",
                        if has_wallet && is_locked {
                            button {
                                onclick: move |_| { custody_error.set(None); show_unlock.set(true); },
                                class: "min-h-11 px-4 py-2 rounded-lg bg-blue-600 hover:bg-blue-700 text-sm font-medium",
                                "Unlock"
                            }
                        } else if has_wallet {
                            button {
                                onclick: {
                                    let mut ctx = wallet_ctx.clone();
                                    move |_| ctx.lock()
                                },
                                class: "min-h-11 px-4 py-2 rounded-lg border border-gray-700 text-sm font-medium hover:bg-gray-800",
                                "Lock"
                            }
                        }
                        button {
                            onclick: move |_| show_clear_data.set(true),
                            class: "px-4 py-2 rounded-lg bg-red-900/30 hover:bg-red-900/50 border border-red-700/50 text-sm font-medium transition-colors text-red-300",
                            "\u{1F5D1}\u{FE0F} Erase All Local Data"
                        }
                    }
                    if let Some(error) = custody_error() {
                        p { class: "text-sm text-red-300", role: "alert", "{error}" }
                    }
                }
            }

            // About
            div { class: "{card_class()} overflow-hidden",
                div { class: "{card_header_class()}",
                    h3 { class: "font-semibold text-sm", "About" }
                }
                div { class: "p-6 space-y-3",
                    div { class: "flex justify-between",
                        span { class: "text-sm text-gray-400", "Version" }
                        span { class: "text-sm font-mono", "{PROTOCOL_VERSION}" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm text-gray-400", "Chains" }
                        span { class: "text-sm", "Bitcoin, Ethereum, Sui, Aptos" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm text-gray-400", "Framework" }
                        span { class: "text-sm font-mono", "Dioxus 0.7" }
                    }
                    div { class: "flex justify-between",
                        span { class: "text-sm text-gray-400", "Storage" }
                        span { class: "text-sm", "localStorage (persistent)" }
                    }
                }
            }
        }

        if show_unlock() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50", role: "dialog", aria_modal: "true", aria_label: "Unlock wallet",
                div { class: "{card_class()} p-6 max-w-sm mx-4 space-y-4",
                    h3 { class: "font-semibold", "Unlock wallet" }
                    p { class: "text-sm text-gray-400", "The passphrase opens a 15-minute in-memory signing session. Your encrypted wallet remains on this device." }
                    input {
                        r#type: "password",
                        value: "{unlock_passphrase}",
                        autocomplete: "current-password",
                        aria_label: "Wallet passphrase",
                        class: "w-full min-h-11 rounded-lg border border-gray-700 bg-gray-950 px-3",
                        oninput: move |event| unlock_passphrase.set(event.value()),
                    }
                    div { class: "flex gap-3",
                        button { class: "flex-1 {btn_secondary_class()}", onclick: move |_| show_unlock.set(false), "Cancel" }
                        button {
                            class: "flex-1 min-h-11 rounded-lg bg-blue-600 px-4 py-2 font-medium",
                            onclick: {
                                let mut ctx = wallet_ctx.clone();
                                move |_| match ctx.unlock(&unlock_passphrase()) {
                                    Ok(()) => { unlock_passphrase.set(String::new()); custody_error.set(None); show_unlock.set(false); }
                                    Err(error) => custody_error.set(Some(error)),
                                }
                            },
                            "Unlock"
                        }
                    }
                }
            }
        }

        // Clear data confirmation modal
        if *show_clear_data.read() {
            div { class: "fixed inset-0 z-50 flex items-center justify-center bg-black/50 modal-backdrop", role: "dialog", aria_modal: "true", aria_label: "Erase all local data confirmation",
                div { class: "{card_class()} p-6 max-w-sm mx-4 modal-content",
                    div { class: "flex items-center gap-2 mb-4",
                        span { class: "text-red-400 text-xl", "\u{26A0}\u{FE0F}" }
                        h3 { class: "font-semibold text-red-300", "Clear All Data?" }
                    }
                    p { class: "text-sm text-gray-400 mb-4",
                        "This permanently deletes local wallet data. Recovery requires your encrypted backup or recovery material. Type ERASE to continue."
                    }
                    input {
                        value: "{erase_confirmation}",
                        aria_label: "Type ERASE to confirm",
                        class: "mb-4 w-full min-h-11 rounded-lg border border-red-700 bg-gray-950 px-3",
                        oninput: move |event| erase_confirmation.set(event.value()),
                    }
                    div { class: "flex gap-3",
                        button {
                            onclick: move |_| show_clear_data.set(false),
                            class: "flex-1 {btn_secondary_class()}",
                            "Cancel"
                        }
                        button {
                            onclick: move |_| {
                                match ctx_clear.erase(&erase_confirmation()) {
                                    Ok(()) => {
                                        erase_confirmation.set(String::new());
                                        custody_error.set(None);
                                        show_clear_data.set(false);
                                    }
                                    Err(error) => {
                                        custody_error.set(Some(error));
                                        show_clear_data.set(false);
                                    }
                                }
                            },
                            disabled: erase_confirmation() != "ERASE",
                            class: "flex-1 px-4 py-2 rounded-lg bg-red-600 hover:bg-red-700 text-sm font-medium transition-colors",
                            "Erase permanently"
                        }
                    }
                }
            }
        }
    }
}

/// Expert-facing protocol tools remain available one level below Settings,
/// rather than competing with everyday wallet destinations.
#[component]
pub fn SettingsAdvanced() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3", Link { to: Route::Settings {}, class: "{btn_secondary_class()}", "← Settings" } h1 { class: "text-2xl font-bold", "Advanced tools" } }
            p { class: "text-sm text-gray-400", "Verification and proof tools are available here. They retain their existing runtime-backed behavior." }
            div { class: "grid gap-3 sm:grid-cols-2",
                Link { to: Route::VerifyProof {}, class: "{card_class()} min-h-11 p-4", "Verify proof" }
                Link { to: Route::VerifyCrossChainProof {}, class: "{card_class()} min-h-11 p-4", "Verify cross-chain proof" }
                Link { to: Route::Validate {}, class: "{card_class()} min-h-11 p-4", "Validation tools" }
                Link { to: Route::GenerateProof {}, class: "{card_class()} min-h-11 p-4", "Proof generation" }
            }
            div { class: "rounded-lg border border-gray-700 p-4", h2 { class: "font-semibold", "Validator tools" } p { class: "mt-1 text-sm text-gray-400", "Validator-mode functionality is consolidated here; it is no longer a global persona." } }
        }
    }
}
