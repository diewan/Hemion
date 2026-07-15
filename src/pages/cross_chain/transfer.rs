//! Cross-chain transfer page.

use crate::components::{Inspector, InspectorProofs, TransferReview, TransferReviewIntent};
use crate::context::{SanadStatus, TransferLifecycleView, use_wallet_context};
use crate::pages::common::*;
use crate::routes::Route;
use crate::services::transfer_authority::{TransferRequest, TransferSubmission, submit_transfer};
use csv_hash::{ChainId, SanadId};
use dioxus::prelude::*;
use std::rc::Rc;

/// Decode the user-visible identifier without padding or truncating it.
///
/// A Sanad identifier is a 32-byte protocol value. Accepting a short value by
/// zero-padding it would silently target a different protocol object.
fn parse_sanad_id(value: &str) -> Result<SanadId, String> {
    let hex_value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = hex::decode(hex_value).map_err(|error| format!("invalid Sanad ID hex: {error}"))?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_: Vec<u8>| "invalid Sanad ID: expected exactly 32 bytes".to_string())?;
    Ok(SanadId::new(bytes))
}

#[cfg(test)]
mod tests {
    use super::parse_sanad_id;

    #[test]
    fn accepts_an_exact_32_byte_sanad_id() {
        let value = format!("0x{}", "ab".repeat(32));
        assert!(parse_sanad_id(&value).is_ok());
    }

    #[test]
    fn rejects_short_sanad_ids_instead_of_padding_them() {
        assert!(parse_sanad_id("ab").is_err());
    }

    #[test]
    fn rejects_non_hex_sanad_ids() {
        assert!(parse_sanad_id("not-a-sanad-id").is_err());
    }
}

#[component]
pub fn CrossChainTransfer() -> Element {
    let wallet_ctx = use_wallet_context();
    let mut from_chain = use_signal(|| ChainId::new("bitcoin"));
    let mut to_chain = use_signal(|| ChainId::new("sui"));
    let mut selected_sanad_index = use_signal(|| 0usize);
    let mut dest_owner = use_signal(String::new);
    let result = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut executing = use_signal(|| false);
    let mut selected_account_index = use_signal(|| 0usize);
    let mut selected_target_contract_index = use_signal(|| 0usize);
    let mut lifecycle = use_signal(|| Option::<TransferLifecycleView>::None);
    let mut review_open = use_signal(|| false);
    let mut review_approved = use_signal(|| false);

    // Get sanads for the source chain (filtered to active only)
    let from_chain_val = from_chain.read().clone();
    let from_chain_display = from_chain_val.clone();
    let sanads_for_source: Vec<_> = wallet_ctx
        .sanads_for_chain(from_chain_val)
        .into_iter()
        .filter(|r| r.status == SanadStatus::Active)
        .collect();
    let has_sanads = !sanads_for_source.is_empty();

    // Reset sanad selection when chain changes
    use_effect(move || {
        selected_sanad_index.set(0);
    });

    // Clone for use in memo
    let sanads_for_memo = sanads_for_source.clone();
    // Get the selected sanad ID
    let sanad_id = use_memo(move || {
        sanads_for_memo
            .get(*selected_sanad_index.read())
            .map(|r| r.id.clone())
            .unwrap_or_default()
    });

    // Get accounts for the source chain
    let accounts = wallet_ctx.accounts_for_chain(from_chain.read().clone());
    let has_account = !accounts.is_empty();

    // Check if selected account is watch-only (can't sign)
    let selected_account = accounts.get(*selected_account_index.read());
    let is_watch_only = selected_account.map(|a| a.is_watch_only()).unwrap_or(false);

    // Get accounts for the destination chain (needed for gas payment)
    let dest_accounts = wallet_ctx.accounts_for_chain(to_chain.read().clone());
    let has_dest_account = !dest_accounts.is_empty();

    // Track fetched destination balance (in raw chain units: satoshis, lamports, MIST, octas, wei)
    let mut dest_balance_raw = use_signal(|| 0u64);
    let mut dest_balance_loading = use_signal(|| false);

    // Fetch destination balance when chain or account changes
    use_effect({
        let to_chain_val = to_chain.read().clone();
        let dest_addr = dest_accounts.first().map(|a| a.address.clone());
        move || {
            if let Some(addr) = &dest_addr {
                dest_balance_loading.set(true);
                let addr = addr.clone();
                let chain_for_balance = to_chain_val.clone();
                spawn(async move {
                    use crate::services::platform::WalletPlatform;
                    let platform = WalletPlatform::default();
                    if let Ok(balance_str) = platform.balance(&addr, chain_for_balance).await
                        && let Ok(balance) = balance_str.parse::<u64>()
                    {
                        dest_balance_raw.set(balance);
                    }
                    dest_balance_loading.set(false);
                });
            }
        }
    });

    // Check if destination account has minimum balance for gas (in raw chain units)
    // Sui: ~0.01 SUI = 10_000_000 MIST, Aptos: ~0.01 APT = 1_000_000 octas
    let min_dest_balance_raw = match to_chain.read().as_str() {
        "sui" => 10_000_000u64,                 // 0.01 SUI in MIST
        "aptos" => 1_000_000u64,                // 0.01 APT in octas
        "ethereum" => 1_000_000_000_000_000u64, // ~0.001 ETH in wei
        "solana" => 1_000_000u64,               // ~0.001 SOL in lamports
        _ => 0u64, // Bitcoin doesn't need pre-funded destination for minting
    };
    let dest_has_enough_balance = *dest_balance_raw.read() >= min_dest_balance_raw;

    // Get contracts for source and target chains
    let source_contracts = wallet_ctx.contracts_for_chain(from_chain.read().clone());
    let target_contracts = wallet_ctx.contracts_for_chain(to_chain.read().clone());
    let _has_source_contract =
        !source_contracts.is_empty() || from_chain.read().as_str() == "bitcoin";
    let has_target_contract = !target_contracts.is_empty();

    // Reset target contract selection when target chain changes
    use_effect(move || {
        selected_target_contract_index.set(0);
    });

    // Check for globally selected contract and pre-populate if it matches target chain
    use_effect({
        let target_contracts = target_contracts.clone();
        let selected = wallet_ctx.selected_contract();
        move || {
            if let Some(ref contract) = selected {
                // Find the contract in target contracts list
                if let Some(index) = target_contracts
                    .iter()
                    .position(|c| c.chain == contract.chain && c.address == contract.address)
                {
                    selected_target_contract_index.set(index);
                }
            }
        }
    });

    // Execute real cross-chain transfer using native signing
    let mut execute_transfer = move || {
        if !review_approved() {
            error.set(Some(
                "A value-bearing transfer must be confirmed on the review screen.".to_string(),
            ));
            return;
        }
        if !_has_source_contract {
            error.set(Some(format!(
                "No contract deployed on {:?}. Deploy contracts manually using Foundry/forge and set the address with `csv contracts set {} <address>`.",
                from_chain.read().clone(), from_chain.read().clone()
            )));
            return;
        }

        if !has_target_contract {
            error.set(Some(format!(
                "No contract deployed on {:?}. Deploy contracts manually using Foundry/forge and set the address with `csv contracts set {} <address>`.",
                to_chain.read().clone(), to_chain.read().clone()
            )));
            return;
        }

        if !has_dest_account {
            error.set(Some(format!(
                "No account available for destination chain {:?}. Please add an account first.",
                to_chain.read().clone()
            )));
            return;
        }

        if !dest_has_enough_balance {
            let min_balance = match to_chain.read().as_str() {
                "sui" => "0.01 SUI",
                "aptos" => "0.01 APT",
                "ethereum" => "0.001 ETH",
                "solana" => "0.001 SOL",
                _ => "funds",
            };
            error.set(Some(format!(
                "Destination account on {} needs at least {} for gas fees. Please fund your account first.",
                to_chain.read().as_str(), min_balance
            )));
            return;
        }

        if !has_account {
            error.set(Some(format!(
                "No account available for {:?}. Please add an account first.",
                from_chain.read().clone()
            )));
            return;
        }

        if !has_sanads {
            error.set(Some(format!(
                "No active sanads available for {:?}. Create a sanad first.",
                from_chain.read().clone()
            )));
            return;
        }

        // All chains now supported via proper BCS/ABI encoding
        // - Bitcoin: Native UTXO with mempool.space
        // - Ethereum: Native ABI encoding
        // - Sui: BCS encoding via sdk_tx
        // - Aptos: BCS encoding via sdk_tx (planned)

        let from = from_chain.read().clone();
        let to = to_chain.read().clone();

        executing.set(true);
        error.set(None);
        lifecycle.set(None);

        // Spawn async task for blockchain operations
        spawn({
            let sanad = sanad_id.read().clone();
            let dest = dest_owner.read().clone();
            let account_idx = *selected_account_index.read();
            let from_for_accounts = from.clone();
            let accounts = wallet_ctx.accounts_for_chain(from_for_accounts);
            let mut result_signal = result;
            let mut error_signal = error;
            let mut executing_signal = executing;
            let mut lifecycle_signal = lifecycle;
            async move {
                use crate::wallet_core::ChainAccount;

                // Get the selected account
                let account: ChainAccount = if let Some(acc) = accounts.get(account_idx) {
                    acc.clone()
                } else {
                    error_signal.set(Some("Selected account not found".to_string()));
                    executing_signal.set(false);
                    return;
                };

                // Check if account can sign transactions
                if account.is_watch_only() {
                    error_signal.set(Some(format!(
                        "Account '{}' is watch-only (no private key). \n\
                        Please import the private key or use a browser wallet like MetaMask.",
                        account.name
                    )));
                    executing_signal.set(false);
                    return;
                }

                // Determine destination owner (default to same address)
                let dest_addr: String = if dest.is_empty() {
                    account.address.clone()
                } else {
                    dest.to_string()
                };

                let sanad_id_for_transfer = match parse_sanad_id(&sanad) {
                    Ok(sanad_id) => sanad_id,
                    Err(message) => {
                        error_signal.set(Some(message));
                        executing_signal.set(false);
                        return;
                    }
                };
                match submit_transfer(TransferRequest {
                    sanad_id: sanad_id_for_transfer,
                    source_chain: from.clone(),
                    destination_chain: to.clone(),
                    destination_address: dest_addr,
                })
                .await
                {
                    Ok(TransferSubmission::Settled(contract_receipt)) => {
                        let view = TransferLifecycleView::from_receipt(&contract_receipt);
                        result_signal.set(Some(
                            "Runtime returned a settled materialization receipt.".to_string(),
                        ));
                        lifecycle_signal.set(Some(view));
                    }
                    Ok(TransferSubmission::AwaitingFinality(event)) => {
                        let view = TransferLifecycleView::from_event(&event);
                        result_signal.set(Some(
                            "Runtime reported a transfer lifecycle event.".to_string(),
                        ));
                        lifecycle_signal.set(Some(view));
                    }
                    Err(error) => error_signal.set(Some(format!("Transfer failed: {error}"))),
                }

                executing_signal.set(false);
            }
        });
    };

    let review_preflight_ok = has_sanads
        && has_account
        && !is_watch_only
        && has_target_contract
        && has_dest_account
        && dest_has_enough_balance;
    let selected_sanad = sanads_for_source.get(*selected_sanad_index.read());
    let review_intent = TransferReviewIntent {
        origin: None,
        signer: selected_account.map(|account| account.address.clone()).unwrap_or_else(|| "No source account selected".to_string()),
        source_chain: from_chain.read().to_string(),
        destination_chain: to_chain.read().to_string(),
        recipient: if dest_owner.read().is_empty() { selected_account.map(|account| account.address.clone()).unwrap_or_else(|| "No recipient available".to_string()) } else { dest_owner.read().clone() },
        asset: "Sanad".to_string(),
        amount: selected_sanad.map(|sanad| sanad.value.to_string()).unwrap_or_else(|| "No sanad selected".to_string()),
        fee: "Calculated by the runtime before submission".to_string(),
        fee_provenance: "estimated",
        preflight_ok: review_preflight_ok,
        corrective_action: (!review_preflight_ok).then(|| "Add the required accounts/contracts and fund the destination gas account, then review again.".to_string()),
        unknown_recipient: true,
        unknown_contract: !has_target_contract,
    };

    if review_open() {
        return rsx! {
            TransferReview {
                intent: review_intent,
                on_back: move |_| review_open.set(false),
                on_confirm: move |_| {
                    review_approved.set(true);
                    review_open.set(false);
                    execute_transfer();
                },
            }
        };
    }

    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::Activity {}, class: "{btn_secondary_class()}", "\u{2190} Back" }
                h1 { class: "text-xl font-bold", "Cross-ChainId Transfer" }
            }

            div { class: "{card_class()} p-6 space-y-5",
                // Account Selection Section
                div { class: "bg-gray-800/50 rounded-lg p-4 border border-gray-700",
                    h3 { class: "text-sm font-medium text-gray-300 mb-3", "1. Select Source Account" }
                    if accounts.is_empty() {
                        div { class: "text-sm text-red-400",
                            {format!("No accounts available for {:?}. Please add an account first.", from_chain.read().clone())}
                        }
                    } else {
                        select {
                            class: "{input_class()}",
                            onchange: move |evt| {
                                if let Ok(idx) = evt.value().parse::<usize>() {
                                    selected_account_index.set(idx);
                                }
                            },
                            for (idx, account) in accounts.iter().enumerate() {
                                option { key: "account-{idx}", value: idx.to_string(), selected: idx == *selected_account_index.read(),
                                    {format!("{} - {} (Balance: {:.8}){}",
                                        account.name,
                                        &account.address[..8.min(account.address.len())],
                                        account.balance_raw as f64 / 1e8, // Convert satoshis to BTC for display
                                        if account.is_watch_only() { " [WATCH-ONLY]" } else { "" }
                                    )}
                                }
                            }
                        }
                    }
                }

                div { class: "grid grid-cols-2 gap-4",
                    {form_field("From ChainId", chain_select(move |v: Rc<FormData>| {
                        if let Ok(c) = v.value().parse::<ChainId>() {
                            from_chain.set(c);
                            selected_account_index.set(0); // Reset account selection
                        }
                    }, from_chain.read().clone()))}

                    {form_field("To ChainId", chain_select(move |v: Rc<FormData>| {
                        if let Ok(c) = v.value().parse::<ChainId>() { to_chain.set(c); }
                    }, to_chain.read().clone()))}
                }

                // ChainId compatibility note
                div { class: "bg-blue-900/30 border border-blue-700/50 rounded-lg p-3",
                    p { class: "text-xs text-blue-300", "ChainId support (all via native signing):" }
                    div { class: "flex gap-2 mt-1 text-xs",
                        span { class: "text-green-400", "✓ Bitcoin: UTXO" }
                        span { class: "text-green-400", "✓ Ethereum: ABI" }
                        span { class: "text-green-400", "✓ Sui: BCS" }
                        span { class: "text-green-400", "✓ Aptos: BCS" }
                        span { class: "text-green-400", "✓ Solana: Native" }
                    }
                }

                // Contracts display section
                div { class: "bg-gray-800/50 rounded-lg p-4 border border-gray-700",
                    h3 { class: "text-sm font-medium text-gray-300 mb-3", "Deployed Contracts" }
                    div { class: "grid grid-cols-2 gap-4",
                        // Source chain contracts
                        div {
                            p { class: "text-xs text-gray-500 mb-1", {format!("Source ({:?})", from_chain.read().clone())} }
                            if source_contracts.is_empty() {
                                if matches!(from_chain.read().as_str(), "bitcoin") {
                                    p { class: "text-xs text-green-400", "✓ UTXO chain - no contract needed" }
                                } else {
                                    p { class: "text-xs text-red-400", "✗ No contract deployed" }
                                }
                            } else {
                                for (idx, contract) in source_contracts.iter().enumerate() {
                                    p { key: "source-contract-{idx}", class: "text-xs text-green-400 font-mono",
                                        {format!("✓ {}", &contract.address[..16.min(contract.address.len())])}
                                    }
                                }
                            }
                        }
                        // Target chain contracts
                        div {
                            p { class: "text-xs text-gray-500 mb-1", {format!("Target ({:?})", to_chain.read().clone())} }
                            if target_contracts.is_empty() {
                                p { class: "text-xs text-red-400", "✗ No contract deployed" }
                            } else if target_contracts.len() == 1 {
                                // Single contract - just display it
                                p { class: "text-xs text-green-400 font-mono",
                                    {format!("✓ {}", &target_contracts[0].address[..16.min(target_contracts[0].address.len())])}
                                }
                            } else {
                                // Multiple contracts - show selector
                                div { class: "space-y-1",
                                    p { class: "text-xs text-blue-400", "Select contract:" }
                                    select {
                                        class: "w-full bg-gray-800 border border-gray-700 rounded px-2 py-1 text-xs font-mono",
                                        onchange: move |evt| {
                                            if let Ok(idx) = evt.value().parse::<usize>() {
                                                selected_target_contract_index.set(idx);
                                            }
                                        },
                                        for (idx, contract) in target_contracts.iter().enumerate() {
                                            option { key: "target-contract-{idx}", value: idx.to_string(), selected: idx == *selected_target_contract_index.read(),
                                                {format!("{}...", &contract.address[..12.min(contract.address.len())])}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                 {form_field("Available Sanads", rsx! {
                    if sanads_for_source.is_empty() {
                        p { class: "text-sm text-red-400",
                            {format!("No active sanads available for {:?}. Create a sanad on this chain first.", from_chain_display)}
                        }
                    } else {
                        select {
                            class: "{input_mono_class()}",
                            onchange: move |evt| {
                                if let Ok(idx) = evt.value().parse::<usize>() {
                                    selected_sanad_index.set(idx);
                                }
                            },
                            for (idx, sanad) in sanads_for_source.iter().enumerate() {
                                option { key: "sanad-{idx}", value: idx.to_string(), selected: idx == *selected_sanad_index.read(),
                                    {format!("{}... - Value: {} - {}",
                                        &sanad.id[..16.min(sanad.id.len())],
                                        sanad.value,
                                        sanad.status
                                    )}
                                }
                            }
                        }
                    }
                })}

                // Show selected sanad details
                if let Some(sanad) = sanads_for_source.get(*selected_sanad_index.read()) {
                    div { class: "bg-gray-800/50 rounded-lg p-3 border border-gray-700",
                        p { class: "text-xs text-gray-400 mb-2", "Selected Sanad Details:" }
                        div { class: "grid grid-cols-2 gap-2 text-xs",
                            div { span { class: "text-gray-500", "Full ID: " }, span { class: "font-mono text-gray-300 break-all", "{&sanad.id}" } }
                            div { span { class: "text-gray-500", "Value: " }, span { class: "font-mono text-gray-300", "{sanad.value}" } }
                            div { span { class: "text-gray-500", "Status: " }, span { class: "{sanad_status_class(&sanad.status)}", "{sanad.status}" } }
                            div { span { class: "text-gray-500", "Owner: " }, span { class: "font-mono text-gray-300", "{truncate_address(&sanad.owner, 8)}" } }
                        }
                    }
                }

                {form_field("Destination Owner (optional)", rsx! {
                    input {
                        value: "{dest_owner.read()}",
                        oninput: move |evt| { dest_owner.set(evt.value()); },
                        class: "{input_mono_class()}",
                        r#type: "text",
                        disabled: *executing.read(),
                    }
                })}

                if let Some(view) = lifecycle.read().as_ref() {
                    Inspector { lifecycle: Some(view.clone()), proofs: InspectorProofs(Vec::new()) }
                }

                if let Some(err) = error.read().as_ref() {
                    div { class: "p-4 bg-red-900/30 border border-red-700/50 rounded-lg",
                        p { class: "text-red-300 text-sm", "{err}" }
                    }
                }

                if let Some(msg) = result.read().as_ref() {
                    div { class: "p-4 bg-green-900/30 border border-green-700/50 rounded-lg",
                        p { class: "text-green-300 font-mono text-sm break-all whitespace-pre-wrap", "{msg}" }
                    }
                }

                button {
                    onclick: move |_| { review_approved.set(false); review_open.set(true); },
                    disabled: *executing.read(),
                    class: "{btn_full_primary_class()}",
                    if *executing.read() {
                        "Executing..."
                    } else if !has_account {
                        "Add Source Account First"
                    } else if is_watch_only {
                        "Watch-Only Account (Cannot Sign)"
                    } else if !has_sanads {
                        "No Sanads Available"
                   } else if !has_target_contract {
                         "Set Target Contract Address"
                    } else if !has_dest_account {
                        "Add Destination Account First"
                    } else if !dest_has_enough_balance {
                        "Fund Destination Account"
                    } else {
                        "Review Cross-ChainId Transfer"
                    }
                }

                if !has_account {
                    p { class: "text-xs text-red-500 mt-2",
                        "Note: Add an account for the selected source chain"
                    }
                } else if is_watch_only {
                    p { class: "text-xs text-red-500 mt-2",
                        "Note: This account is watch-only. Import the private key to transfer."
                    }
                }
                if !has_sanads {
                    p { class: "text-xs text-red-500 mt-2",
                        {format!("Note: Create a Sanad on {:?} source chain first", from_chain.read().clone())}
                    }
                }
                if !has_target_contract {
                    p { class: "text-xs text-red-500 mt-2",
                        {format!("Note: Deploy a CSV contract on {:?} manually using Foundry/forge, then set the address with `csv contracts set`", to_chain.read().clone())}
                    }
                }
                if !has_dest_account {
                    p { class: "text-xs text-red-500 mt-2",
                        {format!("Note: Add an account for {:?} destination chain to pay gas fees", to_chain.read().clone())}
                    }
                } else if !dest_has_enough_balance {
                    p { class: "text-xs text-red-500 mt-2",
                        {format!("Note: Destination account on {} needs gas funds (min: {})",
                            to_chain.read().clone(),
                            match to_chain.read().clone().as_str() {
                                "sui" => "0.01 SUI",
                                "aptos" => "0.01 APT",
                                "ethereum" => "0.001 ETH",
                                "solana" => "0.001 SOL",
                                _ => "0.0",
                            }
                        )}
                    }
                }
            }
        }
    }
}

/// Render the exact runtime lifecycle without turning transaction hashes or
/// explorer data into lifecycle authority.  The normal summary and expert
/// details read from the same immutable projection.
fn lifecycle_panel(view: TransferLifecycleView) -> Element {
    let source_finality = view.source_finality.clone();
    let destination_finality = view.destination_finality.clone();
    let verification = view
        .verification_assurance
        .map(|value| format!("{value:?}"));
    let action_names = format!("{:?}", view.permitted_actions);
    rsx! {
        div { class: "space-y-4 mt-4",
            div { class: "bg-blue-900/30 border border-blue-700/50 rounded-lg p-4 space-y-2",
                h3 { class: "text-sm font-semibold text-blue-200", "Runtime lifecycle" }
                div { class: "flex items-center justify-between gap-3",
                    span { class: "text-sm text-blue-100", "{view.stage.name}" }
                    span { class: "text-xs font-mono text-blue-300", "{view.journal_phase}" }
                }
                p { class: "text-xs text-blue-200", "{view.stage.explanation}" }
                p { class: "text-xs text-blue-300", "Observed by runtime at unix time {view.observed_at}." }
                if let Some(reason) = view.failure_reason.as_ref() {
                    p { class: "text-sm text-red-300", "Runtime reason: {reason}" }
                }
            }

            div { class: "grid grid-cols-1 md:grid-cols-2 gap-3",
                div { class: "bg-gray-800/50 rounded-lg p-3 border border-gray-700",
                    p { class: "text-xs font-medium text-gray-300", "Source finality" }
                    if let Some(evidence) = source_finality {
                        p { class: "text-xs text-gray-400 mt-1", "{evidence.summary}" }
                        p { class: "text-xs text-gray-500", "Evidence: {evidence.provenance}" }
                        p { class: "text-xs mt-1",
                            span { class: if evidence.is_final { "text-green-400" } else { "text-yellow-400" },
                                if evidence.is_final { "Finality established" } else { "Finality not established" }
                            }
                        }
                    } else {
                        p { class: "text-xs text-gray-500 mt-1", "No source-finality evidence reported in this runtime artifact." }
                    }
                }
                div { class: "bg-gray-800/50 rounded-lg p-3 border border-gray-700",
                    p { class: "text-xs font-medium text-gray-300", "Destination finality" }
                    if let Some(evidence) = destination_finality {
                        p { class: "text-xs text-gray-400 mt-1", "{evidence.summary}" }
                        p { class: "text-xs text-gray-500", "Evidence: {evidence.provenance}" }
                    } else {
                        p { class: "text-xs text-yellow-400 mt-1", "Not reported by the runtime artifact; a mint hash is not finality evidence." }
                    }
                }
            }

            div { class: "bg-gray-800/50 rounded-lg p-3 border border-gray-700",
                p { class: "text-xs font-medium text-gray-300", "Verification assurance" }
                if let Some(assurance) = verification {
                    p { class: "text-sm text-gray-200 mt-1", "{assurance}" }
                } else {
                    p { class: "text-xs text-yellow-400 mt-1", "No cryptographic assurance was reported in this artifact." }
                }
                if let Some(provenance) = view.verification_provenance {
                    p { class: "text-xs text-gray-500", "Provenance: {provenance}" }
                }
            }

            if view.allows_resume() || view.allows_retry() {
                div { class: "flex gap-3",
                    if view.allows_resume() {
                        Link { to: Route::ActivityRetry {}, class: "text-sm text-blue-400 hover:text-blue-300", "Resume via runtime \u{2192}" }
                    }
                    if view.allows_retry() {
                        Link { to: Route::ActivityRetry {}, class: "text-sm text-yellow-400 hover:text-yellow-300", "Retry via runtime \u{2192}" }
                    }
                }
            }

            details { class: "bg-gray-900/50 rounded-lg p-3 border border-gray-700",
                summary { class: "cursor-pointer text-xs font-medium text-gray-300", "Expert runtime evidence" }
                div { class: "mt-3 space-y-1 text-xs font-mono text-gray-400 break-all",
                    p { "mode: {view.mode:?}" }
                    p { "transfer_id: {view.transfer_id:?}" }
                    p { "sanad_id: {view.sanad_id}" }
                    p { "replay_id: {view.replay_id:?}" }
                    p { "lock_tx_hash: {view.lock_tx_hash:?}" }
                    p { "mint_tx_hash: {view.mint_tx_hash:?}" }
                    p { "proof_hash: {view.proof_hash:?}" }
                    p { "invoice_id: {view.invoice_id:?}" }
                    p { "consignment_digest: {view.consignment_digest:?}" }
                    p { "journal_phase: {view.journal_phase}" }
                    p { "permitted_actions: {action_names}" }
                }
            }
        }
    }
}

/// Format fee amount for display with appropriate chain units.
fn format_fee(fee: u64, chain: &ChainId) -> String {
    match chain.as_str() {
        "bitcoin" => {
            // Bitcoin fees are in satoshis
            format!("{:.8} BTC", fee as f64 / 100_000_000.0)
        }
        "ethereum" => {
            // Ethereum fees are in wei
            format!("{:.6} ETH", fee as f64 / 1_000_000_000_000_000_000.0)
        }
        "sui" => {
            // Sui fees are in MIST (10^-9 SUI)
            format!("{:.6} SUI", fee as f64 / 1_000_000_000.0)
        }
        "aptos" => {
            // Aptos fees are in octas (10^-8 APT)
            format!("{:.6} APT", fee as f64 / 100_000_000.0)
        }
        "solana" => {
            // Solana fees are in lamports (10^-9 SOL)
            format!("{:.6} SOL", fee as f64 / 1_000_000_000.0)
        }
        _ => format!("{}", fee),
    }
}
