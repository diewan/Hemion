//! Wallet Management page with portable encrypted wallet import/export.

use crate::chains::supported_wallet_chains;
use crate::components::{Card, ChainDisplay, NetworkDisplay, all_chain_displays};
use crate::context::{PortableImportMode, WalletContext, use_wallet_context};
use crate::routes::Route;
use crate::wallet_core::ChainAccount;
use csv_hash::ChainId;
use dioxus::prelude::*;
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq)]
enum WalletTab {
    Accounts,
    AddAccount,
    Export,
    Import,
}

impl std::fmt::Display for WalletTab {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Accounts => write!(f, "Accounts"),
            Self::AddAccount => write!(f, "Add Account"),
            Self::Export => write!(f, "Export"),
            Self::Import => write!(f, "Import"),
        }
    }
}

#[component]
pub fn WalletPage() -> Element {
    let wallet_ctx = use_wallet_context();
    let accounts = wallet_ctx.accounts();
    let selected_chain = wallet_ctx.selected_chain();
    let selected_network = wallet_ctx.selected_network();
    let mut active_tab = use_signal(|| WalletTab::Accounts);
    let account_status_text = if !accounts.is_empty() {
        format!(
            "{} local account{}",
            accounts.len(),
            if accounts.len() == 1 { "" } else { "s" }
        )
    } else {
        "No accounts".to_string()
    };

    let tabs = vec![
        WalletTab::Accounts,
        WalletTab::AddAccount,
        WalletTab::Export,
        WalletTab::Import,
    ];

    rsx! {
        div { class: "space-y-6 stagger-children",
            div { class: "wallet-page-title flex items-center justify-between gap-3",
                h1 { class: "text-3xl font-bold text-gray-100", "Wallet Management" }
                div { class: "flex items-center gap-2 text-sm text-gray-400",
                    span { class: "w-2 h-2 rounded-full", class: if !accounts.is_empty() { "bg-blue-500" } else { "bg-yellow-500" } }
                    "{account_status_text}"
                }
            }

            div { class: "rounded-lg border border-gray-800 bg-gray-900/60 px-4 py-3 text-sm text-gray-400",
                span { class: "font-medium text-gray-200", "Active context: " }
                "{ChainDisplay(selected_chain.clone())} on {NetworkDisplay(selected_network)}. "
                "Account records are local; connection and signing availability are shown per action."
            }

            // Tab navigation
            div { class: "bg-gray-900 rounded-xl border border-gray-800 p-1",
                div { class: "flex gap-1 overflow-x-auto", role: "tablist", aria_label: "Wallet sections",
                    for tab in tabs {
                        button {
                            key: "tab-{tab}",
                            onclick: move |_| active_tab.set(tab),
                            class: "px-4 py-2 rounded-lg text-sm font-medium transition-all whitespace-nowrap",
                            role: "tab",
                            aria_selected: if active_tab() == tab { "true" } else { "false" },
                            class: if active_tab() == tab { "bg-blue-600 text-white" } else { "text-gray-400 hover:text-gray-200 hover:bg-gray-800" },
                            "{tab}"
                        }
                    }
                }
            }

            // Tab content
            match active_tab() {
                WalletTab::Accounts => rsx! { AccountsTab {} },
                WalletTab::AddAccount => rsx! { AddAccountTab {} },
                WalletTab::Export => rsx! { ExportTab {} },
                WalletTab::Import => rsx! { ImportTab {} },
            }
        }
    }
}

#[component]
fn AccountsTab() -> Element {
    let wallet_ctx = use_wallet_context();
    let accounts = wallet_ctx.accounts();

    if accounts.is_empty() {
        return rsx! {
            Card {
                title: "Accounts",
                children: rsx! {
                    div { class: "text-center py-12 space-y-3",
                        div { class: "text-5xl", "\u{1F4CB}" }
                        p { class: "text-gray-400 text-lg", "No accounts" }
                        p { class: "text-sm text-gray-500", "Add an account for a chain or import an encrypted .csvw wallet file." }
                        Link { to: Route::Dashboard {}, class: "inline-block mt-4 px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors", "Go to Dashboard" }
                    }
                }
            }
        };
    }

    rsx! {
        div { class: "space-y-6",
            for chain in supported_wallet_chains() {
                ChainAccountsSection { key: "chain-{chain:?}", chain }
            }
        }
    }
}

#[component]
fn ChainAccountsSection(chain: ChainId) -> Element {
    let wallet_ctx = use_wallet_context();
    let chain_accounts = wallet_ctx.accounts_for_chain(chain.clone());

    if chain_accounts.is_empty() {
        return rsx! {};
    }

    rsx! {
        Card {
            title: format!("{chain_name} ({count})", chain_name = chain_name(&chain), count = chain_accounts.len()),
            children: rsx! {
                div { class: "space-y-2",
                    for account in chain_accounts {
                        ChainAccountRow { key: "account-{account.id}", account: account.clone(), wallet_ctx: wallet_ctx.clone() }
                    }
                }
            }
        }
    }
}

#[component]
fn ChainAccountRow(account: ChainAccount, mut wallet_ctx: WalletContext) -> Element {
    let mut show_storage_details = use_signal(|| false);
    let mut show_remove_confirm = use_signal(|| false);
    let is_bitcoin = account.chain == ChainId::new("bitcoin");

    rsx! {
        div { class: "account-row flex items-center justify-between bg-gray-800/50 rounded-lg p-3 gap-3",
            div { class: "flex-1 min-w-0",
                p { class: "font-mono text-sm text-gray-200 truncate", "{account.address}" }
                p { class: "text-xs text-gray-500 mt-0.5", "{account.name}" }
                if show_storage_details() {
                    div { class: "mt-2 p-2 bg-gray-900 rounded",
                        p { class: "text-xs text-gray-300", "The private key is encrypted in the local keystore and is not displayed here." }
                    }
                }
            }
            div { class: "account-actions flex gap-2",
                if is_bitcoin {
                    button {
                        onclick: {
                            let mut wallet_ctx = wallet_ctx.clone();
                            let account_id = account.id.clone();
                            move |_| {
                                if let Ok(updated) = wallet_ctx.refresh_account_address(&account_id)
                                    && updated
                                {
                                    web_sys::console::log_1(&"Bitcoin address refreshed to new Taproot derivation".into());
                                }
                            }
                        },
                        class: "min-h-6 px-2 py-1 rounded text-xs bg-blue-900/30 text-blue-400 hover:bg-blue-900/50 transition-colors",
                        "Refresh Address"
                    }
                }
                button {
                    onclick: move |_| {
                        show_storage_details.set(!show_storage_details());
                    },
                    class: "min-h-6 px-2 py-1 rounded text-xs bg-gray-700 text-gray-300 hover:bg-gray-600 transition-colors",
                    if show_storage_details() { "Hide storage details" } else { "Key storage details" }
                }
                button {
                    onclick: move |_| show_remove_confirm.set(true),
                    class: "min-h-6 px-2 py-1 rounded text-xs bg-red-900/30 text-red-400 hover:bg-red-900/50 transition-colors",
                    "Remove"
                }
            }
        }
        if show_remove_confirm() {
            div { class: "fixed inset-0 z-[60] flex items-center justify-center bg-black/60 p-4", role: "dialog", aria_modal: "true", aria_label: "Remove account confirmation",
                div { class: "w-full max-w-md rounded-xl border border-red-500/30 bg-gray-900 p-6 shadow-2xl",
                    h2 { class: "text-lg font-semibold text-gray-100", "Remove account?" }
                    p { class: "mt-3 text-sm text-gray-400", "This removes the local {chain_name(&account.chain)} account record and its encrypted key from this wallet. Make sure you have a backup before continuing." }
                    p { class: "mt-2 break-all font-mono text-xs text-gray-500", "{account.address}" }
                    div { class: "mt-6 flex justify-end gap-3",
                        button { class: "min-h-10 rounded-lg bg-gray-800 px-4 text-sm font-medium text-gray-200 hover:bg-gray-700", onclick: move |_| show_remove_confirm.set(false), "Cancel" }
                        button {
                            class: "min-h-10 rounded-lg bg-red-600 px-4 text-sm font-medium text-white hover:bg-red-700",
                            onclick: {
                                let mut wallet_ctx = wallet_ctx.clone();
                                let chain = account.chain.clone();
                                let address = account.address.clone();
                                move |_| {
                                    wallet_ctx.remove_account(chain.clone(), &address);
                                    show_remove_confirm.set(false);
                                }
                            },
                            "Remove account"
                        }
                    }
                }
            }
        }
    }
}

fn chain_name(chain: &ChainId) -> &'static str {
    match chain.as_str() {
        "bitcoin" => "Bitcoin",
        "ethereum" => "Ethereum",
        "sui" => "Sui",
        "aptos" => "Aptos",
        "solana" => "Solana",
        _ => "Unknown",
    }
}

#[component]
fn AddAccountTab() -> Element {
    let mut wallet_ctx = use_wallet_context();
    let mut selected_chain = use_signal(|| ChainDisplay(ChainId::new("bitcoin")));
    let mut pk_input = use_signal(String::new);
    let mut name_input = use_signal(String::new);
    let mut passphrase_input = use_signal(String::new);
    let mut message = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);

    rsx! {
        Card {
            title: "Add Account",
            children: rsx! {
                div { class: "space-y-6 stagger-children",
                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Blockchain" }
                        div { class: "relative",
                            select {
                                class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                                value: "{selected_chain.read()}",
                                onchange: move |evt| {
                                    let val = evt.value();
                                    if let Ok(c) = val.parse::<ChainId>() {
                                        selected_chain.set(ChainDisplay(c));
                                    }
                                },
                                for cd in all_chain_displays() {
                                    option { key: "chain-opt-{cd.0}", value: "{cd.0}", selected: cd.0 == selected_chain.read().0, "{cd.0}" }
                                }
                            }
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Account Name (optional)" }
                        input {
                            value: "{name_input.read()}",
                            oninput: move |evt| name_input.set(evt.value()),
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                            r#type: "text"
                        }
                    }

                   div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Private Key (hex)" }
                        textarea {
                            value: "{pk_input.read()}",
                            oninput: move |evt| { pk_input.set(evt.value()); error.set(None); },
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-3 text-sm text-gray-100 font-mono focus:outline-none focus:ring-2 focus:ring-blue-500 resize-none",
                            rows: 3,
                            autocomplete: "off",
                        }
                    }

                    div {
                        label { class: "block text-sm font-medium text-gray-300 mb-2", "Encryption Passphrase" }
                        input {
                            value: "{passphrase_input.read()}",
                            oninput: move |evt| { passphrase_input.set(evt.value()); error.set(None); },
                            class: "w-full bg-gray-800 border border-gray-700 rounded-lg px-4 py-2 text-sm text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500",
                            r#type: "password",
                            placeholder: "Enter a passphrase to encrypt your private key",
                            autocomplete: "new-password",
                        }
                        button {
                            onclick: move |_| {
                                let chain = selected_chain.read().0.clone();
                                let key = generate_key_for_chain(chain);
                                pk_input.set(key);
                                error.set(None);
                            },
                            class: "mt-2 px-4 py-2 bg-green-600 hover:bg-green-700 rounded-lg text-sm font-medium transition-all duration-200 text-white",
                            "Generate Key"
                        }
                    }

                    if let Some(e) = error.read().clone() {
                        div { class: "bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-sm text-red-300 flex items-center justify-between",
                            span { "{e}" }
                            button { onclick: move |_| error.set(None), class: "text-red-400", "\u{2715}" }
                        }
                    }

                    if let Some(msg) = message.read().clone() {
                        div { class: "bg-green-500/10 border border-green-500/30 rounded-xl p-4 text-sm text-green-300 flex items-center justify-between",
                            span { "{msg}" }
                            button { onclick: move |_| message.set(None), class: "text-green-400", "\u{2715}" }
                        }
                    }

               button {
                        onclick: move |_| {
                            let chain = selected_chain.read().0.clone();
                            let name = {
                                let n = name_input.read().clone();
                                if n.is_empty() { format!("{:?}", chain) } else { n }
                            };
                            let pk = pk_input.read().clone();
                            let passphrase = passphrase_input.read().clone();
                            if passphrase.is_empty() {
                                error.set(Some("Passphrase is required".to_string()));
                                return;
                            }
                            // Create account from private key via keystore
                            let chain_for_msg = chain.clone();
                            match wallet_ctx.import_account_from_key(chain, &name, &pk, &passphrase) {
                                Ok(_) => {
                                    message.set(Some(format!("Account added for {}!", chain_for_msg)));
                                    pk_input.set(String::new());
                                    name_input.set(String::new());
                                }
                                Err(e) => error.set(Some(e.to_string())),
                            }
                        },
                        class: "w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-all duration-200 text-white btn-ripple",
                        "Add Account"
                    }

                    div { class: "bg-blue-500/10 border border-blue-500/20 rounded-lg p-4 text-sm text-gray-400",
                        span { class: "text-blue-400 font-medium", "\u{2139}\u{FE0F} Tip: " }
                        "Only paste a private key you control. It is encrypted locally after import; never paste a recovery phrase here."
                    }
                }
            }
        }
    }
}

#[component]
fn ExportTab() -> Element {
    let wallet_ctx = use_wallet_context();
    let accounts = wallet_ctx.accounts();
    let mut message = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut vault_password = use_signal(String::new);
    let mut file_password = use_signal(String::new);
    let mut file_password_confirmation = use_signal(String::new);

    rsx! {
        Card {
            title: "Export Wallet",
            children: rsx! {
                div { class: "space-y-6 stagger-children",
                    if let Some(e) = error.read().clone() {
                        div { class: "bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-sm text-red-300", "{e}" }
                    }
                    if let Some(msg) = message.read().clone() {
                        div { class: "bg-green-500/10 border border-green-500/30 rounded-xl p-4 text-sm text-green-300", "{msg}" }
                    }

                    div { class: "bg-gray-800/50 rounded-lg p-4 border border-gray-700",
                        p { class: "text-sm text-gray-400 mb-2", "Accounts to export:" }
                        p { class: "text-lg font-bold text-gray-100", "{accounts.len()}" }
                    }

                    div {
                        label { class: "mb-2 block text-sm font-medium text-gray-300", "Platform vault password" }
                        input { r#type: "password", placeholder: "Enter platform vault password", autocomplete: "current-password", value: "{vault_password}",
                        oninput: move |event| vault_password.set(event.value()), class: "w-full rounded-lg bg-gray-800 p-3 text-gray-100" }
                    }
                    div {
                        label { class: "mb-2 block text-sm font-medium text-gray-300", "New export-file password" }
                        input { r#type: "password", placeholder: "At least 12 characters", autocomplete: "new-password", value: "{file_password}",
                        oninput: move |event| file_password.set(event.value()), class: "w-full rounded-lg bg-gray-800 p-3 text-gray-100" }
                        p { class: "mt-2 text-xs text-gray-500", "Use a unique passphrase of at least 12 characters. This password cannot be recovered." }
                    }
                    div {
                        label { class: "mb-2 block text-sm font-medium text-gray-300", "Confirm export-file password" }
                        input { r#type: "password", placeholder: "Re-enter export-file password", autocomplete: "new-password", value: "{file_password_confirmation}",
                        oninput: move |event| file_password_confirmation.set(event.value()), class: "w-full rounded-lg bg-gray-800 p-3 text-gray-100" }
                    }

                    button {
                        onclick: move |_| {
                            if file_password().len() < 12 {
                                error.set(Some("Use an export-file password with at least 12 characters.".to_string()));
                                return;
                            }
                            if file_password() != file_password_confirmation() {
                                error.set(Some("The export-file passwords do not match.".to_string()));
                                return;
                            }
                            match wallet_ctx.export_wallet_file(&vault_password(), &file_password()) {
                                Ok(bytes) => {
                                    trigger_download_bytes("hemion-export.csvw", &bytes);
                                    message.set(Some("Wallet exported! Check your downloads folder.".to_string()));
                                }
                                Err(e) => error.set(Some(e)),
                            }
                        },
                        disabled: accounts.is_empty(),
                        class: "w-full px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg font-medium transition-all duration-200 text-white btn-ripple disabled:opacity-50 disabled:cursor-not-allowed",
                        "\u{1F4E4} Download Encrypted Wallet File"
                    }

                    div { class: "bg-yellow-500/10 border border-yellow-500/20 rounded-lg p-4 text-sm text-gray-400",
                        span { class: "text-yellow-400 font-medium", "\u{26A0}\u{FE0F} Warning: " }
                        "The file is encrypted with the export password. Store it securely and never share either password."
                    }
                }
            }
        }
    }
}

#[component]
fn ImportTab() -> Element {
    let wallet_ctx = use_wallet_context();
    let message = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut success = use_signal(|| false);
    let mut vault_password = use_signal(String::new);
    let mut file_password = use_signal(String::new);
    let mut mode = use_signal(|| PortableImportMode::Profile);
    let mut replace_confirmed = use_signal(|| false);

    if *success.read() {
        return rsx! {
            Card {
                title: "Import Complete",
                children: rsx! {
                    div { class: "text-center py-8 space-y-4",
                        div { class: "text-green-400 text-4xl", "\u{2705}" }
                        p { class: "text-green-400 text-lg font-medium", "Wallet imported successfully!" }
                        p { class: "text-sm text-gray-400", "All accounts have been loaded." }
                        Link { to: Route::Dashboard {}, class: "inline-block px-6 py-3 bg-blue-600 hover:bg-blue-700 rounded-lg text-sm font-medium transition-colors", "Go to Dashboard" }
                    }
                }
            }
        };
    }

    rsx! {
        Card {
            title: "Import Encrypted Wallet File",
            children: rsx! {
                div { class: "space-y-6 stagger-children",
                    if let Some(e) = error.read().clone() {
                        div { class: "bg-red-500/10 border border-red-500/30 rounded-xl p-4 text-sm text-red-300", "{e}" }
                    }
                    if let Some(msg) = message.read().clone() {
                        div { class: "bg-green-500/10 border border-green-500/30 rounded-xl p-4 text-sm text-green-300", "{msg}" }
                    }

                    div {
                        label { class: "mb-2 block text-sm font-medium text-gray-300", "Wallet-file password" }
                        input { r#type: "password", placeholder: "Enter wallet-file password", autocomplete: "current-password", value: "{file_password}",
                            oninput: move |event| file_password.set(event.value()), class: "w-full rounded-lg bg-gray-800 p-3 text-gray-100" }
                    }
                    div {
                        label { class: "mb-2 block text-sm font-medium text-gray-300", "Platform vault password" }
                        input { r#type: "password", placeholder: "Enter platform vault password", autocomplete: "current-password", value: "{vault_password}",
                            oninput: move |event| vault_password.set(event.value()), class: "w-full rounded-lg bg-gray-800 p-3 text-gray-100" }
                    }
                    label { class: "flex gap-2 text-sm text-gray-300",
                        input { r#type: "radio", name: "import-mode", checked: mode() == PortableImportMode::Profile,
                            onchange: move |_| mode.set(PortableImportMode::Profile) }
                        "Import as profile (watch-only; never changes signing keys)"
                    }
                    label { class: "flex gap-2 text-sm text-gray-300",
                        input { r#type: "radio", name: "import-mode", checked: mode() == PortableImportMode::Replace,
                            onchange: move |_| mode.set(PortableImportMode::Replace) }
                        "Replace active identity"
                    }
                    if mode() == PortableImportMode::Replace {
                        label { class: "flex gap-2 text-sm text-red-300",
                            input { r#type: "checkbox", checked: replace_confirmed(), onchange: move |event| replace_confirmed.set(event.data().checked()) }
                            "I understand this replaces the active identity."
                        }
                    }

                    input {
                        r#type: "file",
                        accept: ".csvw,application/octet-stream",
                        id: "wallet-import-input",
                        class: "w-full text-sm text-gray-400 file:mr-4 file:py-2.5 file:px-4 file:rounded-lg file:border-0 file:text-sm file:font-medium file:bg-gray-800 file:text-gray-300 hover:file:bg-gray-700 cursor-pointer",
                        onchange: move |_| {
                            if let Some(window) = web_sys::window()
                                && let Some(document) = window.document()
                                && let Some(el) = document.get_element_by_id("wallet-import-input")
                                && let Some(input) = el.dyn_ref::<web_sys::HtmlInputElement>()
                                && let Some(files) = input.files()
                                && let Some(file) = files.get(0)
                            {
                                let mut ctx = wallet_ctx.clone();
                                if let Ok(reader) = web_sys::FileReader::new() {
                                                            let onload = Closure::wrap(Box::new(move |e: web_sys::ProgressEvent| {
                                                                if let Some(target) = e.target()
                                                                    && let Some(r) = target.dyn_ref::<web_sys::FileReader>()
                                                                    && let Ok(result) = r.result()
                                                                {
                                                                        let bytes = js_sys::Uint8Array::new(&result).to_vec();
                                                                        match ctx.import_wallet_file(&bytes, &file_password(), &vault_password(), mode(), replace_confirmed()) {
                                                                            Ok(_) => success.set(true),
                                                                            Err(e) => error.set(Some(e)),
                                                                        }
                                                                }
                                                            }) as Box<dyn FnMut(_)>);
                                                            reader.set_onload(Some(onload.as_ref().unchecked_ref()));
                                                            onload.forget();
                                                            let _ = reader.read_as_array_buffer(&file);
                                }
                            }
                        },
                    }

                    div { class: "bg-gray-800/50 rounded-lg p-4 border border-gray-700 text-sm text-gray-400",
                        p { class: "font-medium mb-1", "Expected format: encrypted .csvw portable wallet file." }
                        p { "The upload is decoded directly from an in-memory byte buffer; it is never placed in UI state or logs." }
                    }
                }
            }
        }
    }
}

fn trigger_download_bytes(filename: &str, content: &[u8]) {
    if let Some(window) = web_sys::window() {
        let opts = web_sys::BlobPropertyBag::new();
        opts.set_type("application/octet-stream");
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::from_iter([js_sys::Uint8Array::from(content)]),
            &opts,
        )
        .ok();

        if let Some(blob) = blob {
            let url = web_sys::Url::create_object_url_with_blob(&blob).ok();
            if let Some(url) = url {
                let a = window.document().and_then(|d| d.create_element("a").ok());
                if let Some(a) = a
                    && let Some(a) = a.dyn_ref::<web_sys::HtmlAnchorElement>()
                {
                    a.set_href(&url);
                    a.set_download(filename);
                    a.click();
                }
                let _ = web_sys::Url::revoke_object_url(&url);
            }
        }
    }
}

/// Generate a random 32-byte private key for any chain.
/// All supported chains (Bitcoin/Ethereum secp256k1, Sui/Aptos/Solana ed25519) use 32-byte keys.
fn generate_key_for_chain(_chain: ChainId) -> String {
    use rand::RngCore;
    use rand::rngs::OsRng;

    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    format!("0x{}", hex::encode(key))
}
