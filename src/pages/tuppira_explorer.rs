//! G-08 — Tuppira discovery with local verification of selected evidence.

use std::time::Duration;

use crate::components::finality_lanes::{AnchoredFinality, FinalityLanes, FinalityLanesView};
use crate::services::bundle_verifier::disposition_label;
use crate::services::tuppira::{
    LiveTuppiraApi, ObservationProjection, TuppiraEnvironment, discover, list_observations,
    verify_selected,
};
use dioxus::prelude::*;

const FEED_LIMIT: u32 = 50;
const POLL_SECONDS: u64 = 3;

#[component]
pub fn TuppiraExplorer() -> Element {
    // Demo pre-fills for the Piteka→Tuppira→Hemion trace. These point at the
    // local demo services; clear or overwrite them for any other environment.
    let mut api_url = use_signal(|| "http://127.0.0.1:8081".to_string());
    let mut tenant_id = use_signal(|| "demo-tenant".to_string());
    let mut access_token = use_signal(|| "demo-observation-token".to_string());
    let mut observation_id = use_signal(|| {
        "observation:piteka:rcpt-att-cd654dcda074f13369baf88c1599209abb8b7084cf9246a05ad0c2244069935f:revision:1"
            .to_string()
    });
    let mut lineage = use_signal(Vec::<ObservationProjection>::new);
    let mut feed = use_signal(Vec::<ObservationProjection>::new);
    let mut feed_status = use_signal(|| None::<String>);
    let mut source_health = use_signal(Vec::<crate::services::tuppira::SourceHealth>::new);
    let mut selected = use_signal(|| None::<ObservationProjection>);
    let mut bundle = use_signal(String::new);
    let mut context = use_signal(String::new);
    let mut status = use_signal(|| None::<String>);

    // Keep the discovery feed live. The lineage inspector below remains an
    // explicit exact-id query, while this feed makes newly ingested receipts
    // visible without requiring users to know or paste their observation ids.
    use_hook(move || {
        spawn(async move {
            loop {
                let environment = TuppiraEnvironment {
                    api_base_url: api_url.peek().clone(),
                    tenant_id: tenant_id.peek().clone(),
                    access_token: access_token.peek().clone(),
                };
                if !environment.api_base_url.is_empty() {
                    match list_observations(&LiveTuppiraApi, &environment, FEED_LIMIT).await {
                        Ok(items) => {
                            let count = items.len();
                            feed.set(items);
                            feed_status.set(Some(format!(
                                "Live · {count} observation(s) · refreshes every {POLL_SECONDS}s."
                            )));
                        }
                        Err(error) => {
                            feed_status.set(Some(format!("Feed unavailable · {error}")));
                        }
                    }
                }
                crate::services::platform::sleep(Duration::from_secs(POLL_SECONDS)).await;
            }
        });
    });

    // Demo convenience: when the page opens with pre-filled values, run
    // discovery once so the Piteka→Tuppira trace loads without extra clicks.
    // `.peek()` reads without subscribing, so this fires only on mount.
    use_effect(move || {
        let environment = TuppiraEnvironment {
            api_base_url: api_url.peek().clone(),
            tenant_id: tenant_id.peek().clone(),
            access_token: access_token.peek().clone(),
        };
        let id = observation_id.peek().clone();
        if environment.api_base_url.is_empty() || id.is_empty() {
            return;
        }
        status.set(Some(
            "Auto-loading tenant-visible lineage and source health…".into(),
        ));
        spawn(async move {
            match discover(&LiveTuppiraApi, &environment, &id).await {
                Ok((found, health)) => {
                    lineage.set(found);
                    source_health.set(health);
                    selected.set(None);
                    status.set(Some(
                        "Discovery projection loaded · recorded elsewhere, not locally verified."
                            .into(),
                    ));
                }
                Err(error) => {
                    lineage.set(vec![]);
                    source_health.set(vec![]);
                    selected.set(None);
                    status.set(Some(format!("Discovery unavailable · {error}")));
                }
            }
        });
    });

    rsx! {
        section { class: "console-home tuppira-explorer", aria_labelledby: "tuppira-title",
            p { class: "console-eyebrow", "HEMION / TUPPIRA EXPLORER" }
            h1 { id: "tuppira-title", "Observation discovery and lineage" }
            p { class: "console-lede", "Tuppira records observations for discovery. It does not authorize actions or determine validity; selected evidence is verified locally with Parwana." }
            div { class: "console-grid",
                label { class: "console-panel inspector-import", r#for: "tuppira-api", h2 { "Tuppira API URL" } input { id: "tuppira-api", r#type: "url", value: "{api_url}", oninput: move |event| api_url.set(event.value()), placeholder: "https://tuppira.example" } }
                label { class: "console-panel inspector-import", r#for: "tuppira-tenant", h2 { "Tenant" } input { id: "tuppira-tenant", value: "{tenant_id}", oninput: move |event| tenant_id.set(event.value()) } }
                label { class: "console-panel inspector-import", r#for: "tuppira-token", h2 { "Access token" } input { id: "tuppira-token", r#type: "password", autocomplete: "off", value: "{access_token}", oninput: move |event| access_token.set(event.value()) } }
                label { class: "console-panel inspector-import", r#for: "observation-id", h2 { "Observation ID" } input { id: "observation-id", value: "{observation_id}", oninput: move |event| observation_id.set(event.value()) } }
            }
            button { class: "console-action", r#type: "button", onclick: move |_| {
                let environment = TuppiraEnvironment { api_base_url: api_url(), tenant_id: tenant_id(), access_token: access_token() };
                let id = observation_id();
                status.set(Some("Querying tenant-visible lineage and source health…".into()));
                spawn(async move {
                    match discover(&LiveTuppiraApi, &environment, &id).await {
                        Ok((found, health)) => { lineage.set(found); source_health.set(health); selected.set(None); status.set(Some("Discovery projection loaded · recorded elsewhere, not locally verified.".into())); }
                        Err(error) => { lineage.set(vec![]); source_health.set(vec![]); selected.set(None); status.set(Some(format!("Discovery unavailable · {error}"))); }
                    }
                });
            }, "Discover and trace" }
            if let Some(message) = status() { output { class: "console-notice", aria_live: "polite", "{message}" } }

            section { class: "console-panel", aria_labelledby: "live-feed-title",
                h2 { id: "live-feed-title", "Live feed" }
                if let Some(message) = feed_status() {
                    output { class: "console-notice", aria_live: "polite", "{message}" }
                }
                if feed().is_empty() {
                    p { class: "console-limitation", "No observations yet. New receipts appear here automatically after ingestion." }
                } else {
                    ol {
                        for item in feed().iter() {
                            li {
                                button {
                                    class: "console-action",
                                    r#type: "button",
                                    onclick: {
                                        let id = item.observation_id.clone();
                                        move |_| observation_id.set(id.clone())
                                    },
                                    "{item.source_event_type} · {item.observation_id}"
                                }
                            }
                        }
                    }
                    p { class: "console-limitation", "Select an observation, then choose Discover and trace to load its lineage." }
                }
            }

            section { class: "console-panel", aria_labelledby: "source-health-title",
                h2 { id: "source-health-title", "Source health" }
                if source_health().is_empty() { p { class: "console-limitation", "No authenticated source-health result is available." } }
                else {
                    ul {
                        for source in source_health().iter() {
                            li {
                                strong { "{source.display_name}" }
                                span { " · {source.connector_kind} · cursor " }
                                code { "{health_cursor(source.cursor_observed_at)}" }
                            }
                        }
                    }
                }
            }
            section { class: "console-panel", aria_labelledby: "lineage-title",
                h2 { id: "lineage-title", "Lineage" }
                if lineage().is_empty() { p { class: "console-limitation", "No tenant-visible lineage is loaded. Absence does not establish non-occurrence." } }
                else {
                    ol {
                        for item in lineage().iter() {
                            li {
                                article { tabindex: "0",
                                    h3 { "{item.observation_id}" }
                                    p { "Source: {item.source_id} · {item.source_event_type}" }
                                    p { "Normalized digest: " code { "{item.normalized_payload_digest}" } }
                                    p { "Recorded elsewhere · not locally verified" }
                                    // Dual-lane finality: the object is present in the
                                    // observation (buffered) plane; the anchored lane has
                                    // no real chain finality source until ANCHOR-01, so it
                                    // renders an explicit unavailable state — never final.
                                    FinalityLanesView {
                                        lanes: FinalityLanes::new(true, AnchoredFinality::unavailable()),
                                    }
                                    button {
                                        class: "console-action",
                                        r#type: "button",
                                        onclick: { let item = item.clone(); move |_| selected.set(Some(item.clone())) },
                                        "Select evidence"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if let Some(item) = selected() {
                section { class: "console-panel", aria_labelledby: "local-evidence-title",
                    h2 { id: "local-evidence-title", "Verify selected evidence locally" }
                    p { "Selected " code { "{item.observation_id}" } ". Import a Parwana bundle and explicit verification context. Hemion requires the selected normalized digest to be disclosed in that bundle." }
                    label { r#for: "tuppira-bundle", "Bundle" textarea { id: "tuppira-bundle", rows: 8, value: "{bundle}", oninput: move |event| bundle.set(event.value()) } }
                    label { r#for: "tuppira-context", "Verification context" textarea { id: "tuppira-context", rows: 6, value: "{context}", oninput: move |event| context.set(event.value()) } }
                    button { class: "console-action", r#type: "button", onclick: move |_| {
                        let message = match verify_selected(&item, bundle().as_bytes(), context().as_bytes()) {
                            Ok(local) => format!("Locally verified selected evidence · {} · context {}", disposition_label(local.report.disposition), hex::encode(local.context_id.as_bytes())),
                            Err(error) => format!("No local verification result · {error}"),
                        };
                        status.set(Some(message));
                    }, "Verify selected evidence locally" }
                }
            }
            aside { class: "console-notice", aria_label: "Explorer limitations", strong { "Discovery is not verification: " } span { "Tuppira health and lineage are observations. They do not establish completeness, factual truth, organizational authorization, or a protocol verdict." } }
        }
    }
}

fn health_cursor(cursor: Option<i64>) -> String {
    cursor
        .map(|value| value.to_string())
        .unwrap_or_else(|| String::from("unknown"))
}
