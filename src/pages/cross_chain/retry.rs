//! Cross-chain transfer retry page.

use crate::context::{ReceiptBody, TransferViewModel};
use crate::pages::common::*;
use crate::routes::Route;
use crate::services::transfer_authority::{ResumeRequest, TransferSubmission, resume_transfer};
use csv_hash::{ChainId, SanadId};
use dioxus::prelude::*;

fn parse_sanad_id(value: &str) -> Result<SanadId, String> {
    let bytes = hex::decode(value.strip_prefix("0x").unwrap_or(value))
        .map_err(|error| format!("invalid Sanad ID hex: {error}"))?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_: Vec<u8>| "invalid Sanad ID: expected exactly 32 bytes".to_string())?;
    Ok(SanadId::new(bytes))
}

#[component]
pub fn CrossChainRetry() -> Element {
    let mut transfer_id = use_signal(String::new);
    let mut sanad_id = use_signal(String::new);
    let mut source_chain = use_signal(|| ChainId::new("bitcoin"));
    let mut destination_chain = use_signal(|| ChainId::new("sui"));
    let mut result = use_signal(|| Option::<String>::None);
    let mut error = use_signal(|| Option::<String>::None);
    let mut executing = use_signal(|| false);

    rsx! {
        div { class: "max-w-2xl space-y-6",
            div { class: "flex items-center gap-3",
                Link { to: Route::CrossChain {}, class: "{btn_secondary_class()}", "\u{2190} Back" }
                h1 { class: "text-xl font-bold", "Retry Transfer" }
            }

            div { class: "{card_class()} p-6 space-y-5",
                {form_field("Transfer ID", rsx! {
                    input {
                        value: "{transfer_id.read()}",
                        oninput: move |evt| { transfer_id.set(evt.value()); },
                        class: "{input_mono_class()}",
                        r#type: "text"
                    }
                })}
                {form_field("Sanad ID", rsx! {
                    input { value: "{sanad_id}", oninput: move |evt| sanad_id.set(evt.value()), class: "{input_mono_class()}", r#type: "text" }
                })}
                div { class: "grid grid-cols-2 gap-4",
                    {form_field("Source chain", chain_select(move |v| { if let Ok(chain) = v.value().parse::<ChainId>() { source_chain.set(chain); } }, source_chain()))}
                    {form_field("Destination chain", chain_select(move |v| { if let Ok(chain) = v.value().parse::<ChainId>() { destination_chain.set(chain); } }, destination_chain()))}
                }

                button {
                    onclick: move |_| {
                        let sanad = match parse_sanad_id(&sanad_id()) {
                            Ok(sanad) => sanad,
                            Err(message) => { error.set(Some(message)); return; }
                        };
                        executing.set(true);
                        error.set(None);
                        result.set(None);
                        let request = ResumeRequest {
                            transfer_id: transfer_id(), sanad_id: sanad,
                            source_chain: source_chain(), destination_chain: destination_chain(),
                            destination_address: None,
                        };
                        spawn(async move {
                            match resume_transfer(request).await {
                                Ok(TransferSubmission::Settled(receipt)) => match &receipt.body {
                                    ReceiptBody::Materialize(body) => result.set(Some(format!(
                                        "Transfer settled by runtime. Mint transaction: {}. Permitted next actions: {:?}",
                                        body.mint_tx_hash, TransferViewModel::from(&receipt).permitted_actions))),
                                    _ => error.set(Some("runtime returned an unexpected receipt mode".to_string())),
                                },
                                Ok(TransferSubmission::AwaitingFinality(event)) => {
                                    let view = TransferViewModel::from(&event);
                                    result.set(Some(format!("Runtime journal reports {}. Permitted next actions: {:?}", view.phase, view.permitted_actions)));
                                }
                                Err(message) => error.set(Some(message)),
                            }
                            executing.set(false);
                        });
                    },
                    disabled: executing(),
                    class: "{btn_full_primary_class()}",
                    if executing() { "Contacting Runtime..." } else { "Resume via Runtime" }
                }

                if let Some(message) = error() { div { class: "p-4 bg-red-900/30 border border-red-700/50 rounded-lg", p { class: "text-red-300", "{message}" } } }

                if let Some(msg) = result.read().as_ref() {
                    div { class: "p-4 bg-blue-900/30 border border-blue-700/50 rounded-lg",
                        p { class: "text-blue-300", "{msg}" }
                    }
                }
            }
        }
    }
}
