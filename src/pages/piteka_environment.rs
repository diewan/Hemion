//! Deep-link target for a Piteka receipt.

use crate::services::bundle_verifier::disposition_label;
use crate::services::piteka::{LivePitekaApi, PitekaEnvironment, download_and_verify};
use dioxus::prelude::*;

#[component]
pub fn PitekaEnvironmentReceipt(environment_id: String, receipt_id: String) -> Element {
    let mut api_url = use_signal(String::new);
    let mut access_token = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);
    rsx! {
        section { class: "console-home piteka-environment", aria_labelledby: "piteka-title",
            p { class: "console-eyebrow", "HEMION / PITEKA CONNECTION" }
            h1 { id: "piteka-title", "Environment receipt" }
            p { class: "console-lede", "Download through Piteka's authorized API, then re-verify locally with the pinned Parwana verifier. Hemion never reads Piteka's database." }
            dl { class: "console-panel",
                div { dt { "Environment tenant" } dd { class: "console-mono", "{environment_id}" } }
                div { dt { "Receipt" } dd { class: "console-mono", "{receipt_id}" } }
            }
            label { class: "console-panel inspector-import", r#for: "piteka-api-url",
                h2 { "Piteka API URL" }
                input { id: "piteka-api-url", r#type: "url", value: "{api_url}", oninput: move |event| api_url.set(event.value()), placeholder: "https://piteka.example" }
            }
            label { class: "console-panel inspector-import", r#for: "piteka-token",
                h2 { "Access token" }
                input { id: "piteka-token", r#type: "password", autocomplete: "off", value: "{access_token}", oninput: move |event| access_token.set(event.value()) }
            }
            button { class: "console-action", r#type: "button", onclick: move |_| {
                let connection = PitekaEnvironment { api_base_url: api_url(), tenant_id: environment_id.clone(), access_token: access_token() };
                let receipt = receipt_id.clone();
                status.set(Some("Downloading from the authorized Piteka API…".into()));
                spawn(async move {
                    let message = match download_and_verify(&LivePitekaApi, &connection, &receipt).await {
                        Ok(local) => format!("Locally verified · {} · context {}", disposition_label(local.report.disposition), hex::encode(local.context_id.as_bytes())),
                        Err(error) => format!("No verification result · {error}"),
                    };
                    status.set(Some(message));
                });
            }, "Download and verify locally" }
            if let Some(message) = status() { output { class: "console-notice", aria_live: "polite", "{message}" } }
            aside { class: "console-notice", aria_label: "Connection limitations",
                strong { "Downloaded does not mean verified: " }
                span { "Piteka supplies the object; only the local Parwana verification result is presented as verification." }
            }
        }
    }
}
