//! Create sanad page.

use crate::context::use_wallet_context;
use crate::pages::common::*;
use dioxus::prelude::*;

#[component]
pub fn CreateSanad() -> Element {
    let mut wallet_ctx = use_wallet_context();

    if let Some(n) = wallet_ctx.notification() {
        return rsx! {
            div { class: "max-w-2xl space-y-6",
                {notification_banner(n.kind, n.message, move || { wallet_ctx.clear_notification(); })}
                CreateSanadForm {}
            }
        };
    }

    rsx! {
        div { class: "max-w-2xl space-y-6",
            CreateSanadForm {}
        }
    }
}

#[component]
pub fn CreateSanadForm() -> Element {
    let _wallet_ctx = use_wallet_context();
    let mut error = use_signal(|| Option::<String>::None);

    rsx! {
        div { class: "{card_class()} p-6 space-y-5",
            div { class: "{card_header_class()} -mx-6 -mt-6 mb-4",
                h2 { class: "font-semibold text-sm", "Create New Sanad" }
            }

            if let Some(e) = error.read().as_ref().cloned() {
                div { class: "p-3 bg-red-900/30 border border-red-700/50 rounded-lg text-sm text-red-300", "{e}" }
            }

            button {
                onclick: move |_| {
                    error.set(Some("Sanad creation requires a configured CSV runtime host. The wallet does not fabricate Sanad, seal, or commitment records locally.".to_string()));
                },
                class: "{btn_full_primary_class()}",
                "Create Sanad via Runtime"
            }
        }
    }
}

fn notification_banner(
    kind: crate::context::NotificationKind,
    message: String,
    on_close: impl FnOnce() + 'static,
) -> Element {
    let (bg_class, icon) = match kind {
        crate::context::NotificationKind::Success => (
            "bg-green-900/30 border-green-700/50 text-green-300",
            "\u{2705}",
        ),
        crate::context::NotificationKind::Error => {
            ("bg-red-900/30 border-red-700/50 text-red-300", "\u{274C}")
        }
        crate::context::NotificationKind::Info => (
            "bg-blue-900/30 border-blue-700/50 text-blue-300",
            "\u{2139}",
        ),
        crate::context::NotificationKind::Warning => (
            "bg-yellow-900/30 border-yellow-700/50 text-yellow-300",
            "\u{26A0}",
        ),
    };

    let on_close_cell = std::cell::RefCell::new(Some(on_close));

    rsx! {
        div { class: "p-4 {bg_class} rounded-lg flex items-center justify-between",
            div { class: "flex items-center gap-2",
                span { "{icon}" }
                p { "{message}" }
            }
            button {
                onclick: move |_| {
                    if let Some(cb) = on_close_cell.borrow_mut().take() {
                        cb();
                    }
                },
                class: "text-sm hover:opacity-70",
                "\u{2715}"
            }
        }
    }
}
