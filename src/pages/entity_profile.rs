//! Tenant-authorized accountable-entity profile and observational analytics.

use crate::services::tuppira::{
    EntityAggregateProjection, LiveTuppiraApi, TuppiraEnvironment, fetch_entity,
};
use dioxus::prelude::*;

#[component]
pub fn EntityProfile(entity_id: String) -> Element {
    let mut api_url = use_signal(|| "http://127.0.0.1:4000".to_string());
    let mut tenant_id = use_signal(|| "demo-tenant".to_string());
    let mut access_token = use_signal(|| "demo-read-token".to_string());
    let mut profile = use_signal(|| None::<EntityAggregateProjection>);
    let mut status = use_signal(|| None::<String>);

    rsx! {
        section { class: "console-home", aria_labelledby: "entity-title",
            p { class: "console-eyebrow", "HEMION / ACCOUNTABLE ENTITY" }
            h1 { id: "entity-title", "Entity profile" }
            p { class: "console-limitation", "Standing: Indeterminate. These source projections support investigation; they do not establish authorization or truth." }
            details { class: "console-panel",
                summary { "Authorized Tuppira connection" }
                label { r#for: "entity-api", "Tuppira API URL" input { id: "entity-api", value: "{api_url}", oninput: move |e| api_url.set(e.value()) } }
                label { r#for: "entity-tenant", "Tenant" input { id: "entity-tenant", value: "{tenant_id}", oninput: move |e| tenant_id.set(e.value()) } }
                label { r#for: "entity-token", "Access token" input { id: "entity-token", r#type: "password", autocomplete: "off", value: "{access_token}", oninput: move |e| access_token.set(e.value()) } }
                button { class: "console-action", r#type: "button", onclick: {
                    let requested = entity_id.clone();
                    move |_| {
                        let environment = TuppiraEnvironment { api_base_url: api_url(), tenant_id: tenant_id(), access_token: access_token() };
                        let id = requested.clone();
                        spawn(async move { match fetch_entity(&LiveTuppiraApi, &environment, &id).await {
                            Ok(value) => { profile.set(Some(value)); status.set(Some("Tenant-authorized projection loaded.".into())); }
                            Err(error) => { profile.set(None); status.set(Some(format!("Profile unavailable · {error}"))); }
                        }});
                    }
                }, "Load profile" }
                if let Some(message) = status() { output { class: "console-notice", aria_live: "polite", "{message}" } }
            }
            if let Some(value) = profile() {
                article { class: "console-panel", aria_labelledby: "entity-identity",
                    h2 { id: "entity-identity", "{value.entity.display_name}" }
                    p { class: "console-mono", "{value.entity.entity_id} · {value.entity.entity_kind}" }
                    p { "{value.references.len()} accountability references · {value.relationships.len()} disclosed relationship records" }
                }
                section { class: "console-panel", aria_labelledby: "entity-facts",
                    h2 { id: "entity-facts", "Accountability facts" }
                    ul { for item in value.references.iter() { li { "{item.kind} · {item.disclosure_state} · " {item.object_id.clone().unwrap_or_else(|| String::from("not disclosed"))} } } }
                }
                section { class: "console-panel", aria_labelledby: "entity-relations",
                    h2 { id: "entity-relations", "Relationships" }
                    ul { for item in value.relationships.iter() { li { "{item.kind} · {item.disclosure_state} · " {item.related_entity_id.clone().unwrap_or_else(|| String::from("not disclosed"))} } } }
                }
            } else { p { class: "console-limitation", "No tenant-authorized entity facts loaded. Responsibility remains Indeterminate." } }
        }
    }
}
