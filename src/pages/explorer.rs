//! Live accountability explorer — a blockchain-explorer experience for the
//! Piteka→Tuppira→Hemion trace.
//!
//! Left pane: a live feed polled from Tuppira's `GET /api/v1/observations`
//! discovery read-model. Right pane: click any observation to open its
//! mandate/receipt subjects, then click a subject to drill into Piteka's
//! assembled accountability chain (`GET /api/v1/mandates/{id}/chain`):
//! authority → action → provider deployment → receipt → evidence.
//!
//! Discovery is never verification. Tuppira and Piteka supply detail for
//! navigation; validity is always recomputed locally with the pinned Parwana
//! verifier via [`download_and_verify`], never copied from either service.

use std::time::Duration;

use dioxus::prelude::*;

use crate::services::bundle_verifier::disposition_label;
use crate::services::piteka::{
    LivePitekaApi, MandateChain, PitekaEnvironment, download_and_verify, fetch_chain,
};
use crate::services::tuppira::{
    LiveTuppiraApi, ObservationProjection, TuppiraEnvironment, list_observations,
};

/// How many observations to request per feed poll, and how often to poll.
const FEED_LIMIT: u32 = 50;
const POLL_SECONDS: u64 = 3;
/// Compact feed rows shown per page.
const FEED_PAGE_SIZE: usize = 8;

#[component]
pub fn Explorer() -> Element {
    // Tuppira (discovery feed) connection — demo pre-fills for the local trace.
    let tuppira_url = use_signal(|| "http://127.0.0.1:8081".to_string());
    let tuppira_tenant = use_signal(|| "demo-tenant".to_string());
    let tuppira_token = use_signal(|| "demo-observation-token".to_string());

    // Piteka (chain drill-down + bundle export) connection.
    let piteka_url = use_signal(|| "http://127.0.0.1:3000".to_string());
    let piteka_tenant = use_signal(|| "demo-tenant".to_string());
    let piteka_token = use_signal(|| "demo-read-token".to_string());
    // GitHub demo repo (owner/name) — used to build outbound links to the repo
    // and its deployments from the chain's execution attempts.
    let github_repo = use_signal(|| "zorvan/piteka-demo".to_string());

    // Live feed state.
    let mut feed = use_signal(Vec::<ObservationProjection>::new);
    let mut feed_status = use_signal(|| None::<String>);
    let live = use_signal(|| true);
    let mut page = use_signal(|| 0usize);

    // Selection / drill-down state.
    let mut selected = use_signal(|| None::<ObservationProjection>);
    let mut chain = use_signal(|| None::<MandateChain>);
    let mut chain_status = use_signal(|| None::<String>);
    let mut verify_status = use_signal(|| None::<(String, String)>);

    // Live polling loop. `use_hook` runs exactly once; the spawned task is tied
    // to this component's scope and is cancelled when the page unmounts. Reads
    // use `.peek()` so the loop never subscribes the component to a re-render,
    // and edits to the connection fields take effect on the next tick.
    use_hook(move || {
        spawn(async move {
            loop {
                if *live.peek() {
                    let environment = TuppiraEnvironment {
                        api_base_url: tuppira_url.peek().clone(),
                        tenant_id: tuppira_tenant.peek().clone(),
                        access_token: tuppira_token.peek().clone(),
                    };
                    if !environment.api_base_url.is_empty() {
                        match list_observations(&LiveTuppiraApi, &environment, FEED_LIMIT).await {
                            Ok(items) => {
                                let count = items.len();
                                feed.set(items);
                                feed_status.set(Some(format!(
                                    "Live · {count} observation(s) · recorded elsewhere, not locally verified."
                                )));
                            }
                            Err(error) => {
                                feed_status.set(Some(format!("Feed unavailable · {error}")));
                            }
                        }
                    }
                }
                tokio::time::sleep(Duration::from_secs(POLL_SECONDS)).await;
            }
        });
    });

    // Resolve a subject ref (`mandate:<id>` or `receipt:<id>`) to the mandate id
    // whose chain we load. A receipt subject reuses the sibling mandate subject
    // from the same observation, so any chip in a row opens the same chain.
    let mandate_for = move |subject: &str, observation: &ObservationProjection| -> Option<String> {
        if let Some(id) = subject.strip_prefix("mandate:") {
            return Some(id.to_string());
        }
        if subject.starts_with("receipt:") {
            return observation
                .subject_refs
                .iter()
                .find_map(|other| other.strip_prefix("mandate:").map(str::to_string));
        }
        None
    };

    // Load a mandate's assembled chain from Piteka's read API.
    let load_chain = move |mandate_id: String| {
        let environment = PitekaEnvironment {
            api_base_url: piteka_url.peek().clone(),
            tenant_id: piteka_tenant.peek().clone(),
            access_token: piteka_token.peek().clone(),
        };
        chain_status.set(Some(format!("Loading chain for mandate {mandate_id}…")));
        verify_status.set(None);
        spawn(async move {
            match fetch_chain(&LivePitekaApi, &environment, &mandate_id).await {
                Ok(loaded) => {
                    chain.set(Some(loaded));
                    chain_status.set(Some(
                        "Chain assembled from Piteka discovery detail · not a verdict.".into(),
                    ));
                }
                Err(error) => {
                    chain.set(None);
                    chain_status.set(Some(format!("Chain unavailable · {error}")));
                }
            }
        });
    };

    // Pagination over the newest-first feed. Computed each render so it tracks
    // live updates; the page index is clamped in case the feed shrank.
    let feed_items = feed();
    let total = feed_items.len();
    let page_count = total.div_ceil(FEED_PAGE_SIZE).max(1);
    let current_page = page().min(page_count - 1);
    let page_items: Vec<ObservationProjection> = feed_items
        .iter()
        .skip(current_page * FEED_PAGE_SIZE)
        .take(FEED_PAGE_SIZE)
        .cloned()
        .collect();

    rsx! {
        section { class: "console-home explorer", aria_labelledby: "explorer-title",
            p { class: "console-eyebrow", "HEMION / ACCOUNTABILITY EXPLORER" }
            h1 { id: "explorer-title", "Live accountability explorer" }
            p { class: "console-lede",
                "A streaming feed of activity from Tuppira's discovery read-model. Open any entity to drill into Piteka's assembled chain — authority, action, provider deployment, receipt, and evidence — and verify each receipt locally with Parwana."
            }

            // Connection settings (collapsed detail; demo pre-fills work as-is).
            details { class: "console-panel",
                summary { "Connections (Tuppira discovery · Piteka chain)" }
                div { class: "console-grid",
                    label { class: "inspector-import", r#for: "tp-url", h2 { "Tuppira API URL" }
                        input { id: "tp-url", r#type: "url", value: "{tuppira_url}", oninput: move |e| tuppira_url.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "tp-tenant", h2 { "Tuppira tenant" }
                        input { id: "tp-tenant", value: "{tuppira_tenant}", oninput: move |e| tuppira_tenant.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "tp-token", h2 { "Tuppira token" }
                        input { id: "tp-token", r#type: "password", autocomplete: "off", value: "{tuppira_token}", oninput: move |e| tuppira_token.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "pk-url", h2 { "Piteka API URL" }
                        input { id: "pk-url", r#type: "url", value: "{piteka_url}", oninput: move |e| piteka_url.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "pk-tenant", h2 { "Piteka tenant" }
                        input { id: "pk-tenant", value: "{piteka_tenant}", oninput: move |e| piteka_tenant.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "pk-token", h2 { "Piteka token" }
                        input { id: "pk-token", r#type: "password", autocomplete: "off", value: "{piteka_token}", oninput: move |e| piteka_token.clone().set(e.value()) } }
                    label { class: "inspector-import", r#for: "gh-repo", h2 { "GitHub repo (owner/name)" }
                        input { id: "gh-repo", value: "{github_repo}", oninput: move |e| github_repo.clone().set(e.value()) } }
                }
            }

            // Two-pane explorer: feed on the left, chain drill-down on the right.
            div { class: "explorer-panes",

                // ── Live feed pane ──────────────────────────────────────────
                section { class: "console-panel explorer-pane", aria_labelledby: "feed-title",
                    div { class: "explorer-feed-head",
                        h2 { id: "feed-title", "Live feed" }
                        button { class: "console-action", r#type: "button",
                            onclick: move |_| { let now = *live.peek(); live.clone().set(!now); },
                            if live() { "⏸ Pause" } else { "▶ Resume" }
                        }
                    }
                    if let Some(message) = feed_status() {
                        output { class: "console-notice", aria_live: "polite", "{message}" }
                    }
                    if total == 0 {
                        p { class: "console-limitation", "No observations yet. New activity appears here within a few seconds." }
                    } else {
                        // Compact one-line rows; full detail opens in the right pane.
                        ol { class: "explorer-feed",
                            for item in page_items.iter() {
                                li {
                                    div {
                                        role: "button",
                                        tabindex: "0",
                                        class: "explorer-feed-row",
                                        aria_pressed: if selected().as_ref().is_some_and(|s| s.observation_id == item.observation_id) { "true" } else { "false" },
                                        onclick: {
                                            let item = item.clone();
                                            move |_| { selected.set(Some(item.clone())); }
                                        },
                                        div { class: "explorer-feed-head",
                                            strong { "{short_label(item)}" }
                                            span { class: "explorer-badge", "{retraction_badge(&item.retraction_status)}" }
                                        }
                                        small { "{primary_ref(item)} · {short_time(item.observed_at)}" }
                                    }
                                }
                            }
                        }
                        div { class: "explorer-pager",
                            button { class: "console-action", r#type: "button",
                                disabled: current_page == 0,
                                onclick: move |_| { let p = page(); page.set(p.saturating_sub(1)); },
                                "‹ Newer"
                            }
                            span { class: "console-limitation", "{total} event(s) · page {current_page + 1} of {page_count}" }
                            button { class: "console-action", r#type: "button",
                                disabled: current_page + 1 >= page_count,
                                onclick: move |_| { let p = page(); page.set(p + 1); },
                                "Older ›"
                            }
                        }
                    }
                }

                // ── Chain drill-down pane ───────────────────────────────────
                section { class: "console-panel explorer-pane", aria_labelledby: "chain-title",
                    h2 { id: "chain-title", "Accountability chain" }
                    if let Some(observation) = selected() {
                        article { class: "explorer-card",
                            h3 { "{short_label(&observation)}" }
                            p { "Source: {observation.source_id} · {short_time(observation.observed_at)}" }
                            p { class: "console-limitation", "Profile: {observation.normalized_profile_id} v{observation.normalized_profile_version}" }
                            p { class: "console-limitation", "Observation " code { "{observation.observation_id}" } }
                            p { class: "console-limitation", "Digest " code { "{observation.normalized_payload_digest}" } }
                        }
                        p { class: "console-limitation", "Subjects — click one to open Piteka's chain for its mandate:" }
                        div { class: "explorer-chips",
                            for subject in observation.subject_refs.iter() {
                                if let Some(mandate_id) = mandate_for(subject, &observation) {
                                    div {
                                        role: "button",
                                        tabindex: "0",
                                        class: "explorer-chip",
                                        onclick: {
                                            let load = load_chain.clone();
                                            move |_| load.clone()(mandate_id.clone())
                                        },
                                        "{subject}"
                                    }
                                } else {
                                    span { class: "explorer-chip explorer-chip-static", "{subject}" }
                                }
                            }
                        }
                    } else {
                        p { class: "console-limitation", "Select an observation from the live feed to begin." }
                    }

                    if let Some(message) = chain_status() {
                        output { class: "console-notice", aria_live: "polite", "{message}" }
                    }

                    if let Some(loaded) = chain() {
                        {render_chain(loaded, piteka_url, piteka_tenant, piteka_token, github_repo, verify_status)}
                    }
                }
            }

            aside { class: "console-notice", aria_label: "Explorer limitations",
                strong { "Discovery is not verification: " }
                span { "The feed and chain are observations recorded elsewhere. They do not establish completeness, factual truth, organizational authorization, or a protocol verdict. Only the local Parwana verifier produces a disposition." }
            }
        }
    }
}

/// Renders the assembled chain: mandate → timeline → attempts → receipts (with
/// inline local verify) → evidence.
fn render_chain(
    loaded: MandateChain,
    piteka_url: Signal<String>,
    piteka_tenant: Signal<String>,
    piteka_token: Signal<String>,
    github_repo: Signal<String>,
    mut verify_status: Signal<Option<(String, String)>>,
) -> Element {
    rsx! {
        div { class: "explorer-chain",
            // Mandate at the root.
            article { class: "explorer-card", tabindex: "0",
                h3 { "Mandate" }
                p { code { "{loaded.mandate.mandate_id}" } }
                p { "State: " strong { "{loaded.mandate.state}" } " · version {loaded.mandate.version}" }
            }

            // Audit timeline (propose → approve → reserve → consume …).
            if !loaded.timeline.is_empty() {
                p { class: "explorer-section-title", "Timeline" }
                ol { class: "explorer-timeline",
                    for step in loaded.timeline.iter() {
                        li {
                            strong { "{step.action}" }
                            span { " · {step.decision} · {short_time(step.at)}" }
                            if let Some(actor) = step.actor.as_ref() { span { " · by {actor}" } }
                            if !step.detail.is_empty() { br {} small { "{step.detail}" } }
                        }
                    }
                }
            }

            // Execution attempts, carrying the provider (GitHub) deployment id.
            if !loaded.attempts.is_empty() {
                p { class: "explorer-section-title", "Execution attempts" }
                for attempt in loaded.attempts.iter() {
                    article { class: "explorer-card", tabindex: "0",
                        p { "Attempt " code { "{attempt.attempt_id}" } }
                        p { "Executor: {attempt.executor_identity} · state " strong { "{attempt.state}" } }
                        if let Some(deployment) = attempt.github_deployment_id {
                            p { "GitHub deployment " strong { "#{deployment}" } }
                            div { class: "explorer-links",
                                span {
                                    class: "explorer-link", role: "button", tabindex: "0",
                                    onclick: move |_| {
                                        let repo = github_repo.peek().clone();
                                        let _ = webbrowser::open(&format!("https://github.com/{repo}"));
                                    },
                                    "Repository ↗"
                                }
                                span {
                                    class: "explorer-link", role: "button", tabindex: "0",
                                    onclick: move |_| {
                                        let repo = github_repo.peek().clone();
                                        let _ = webbrowser::open(&format!("https://github.com/{repo}/deployments"));
                                    },
                                    "Deployments ↗"
                                }
                                span {
                                    class: "explorer-link", role: "button", tabindex: "0",
                                    onclick: move |_| {
                                        let repo = github_repo.peek().clone();
                                        let _ = webbrowser::open(&format!("https://github.com/{repo}/actions"));
                                    },
                                    "Actions run ↗"
                                }
                            }
                        }
                    }
                }
            }

            // Receipts, each with an inline local-verify action.
            if !loaded.receipts.is_empty() {
                p { class: "explorer-section-title", "Receipts" }
                for receipt in loaded.receipts.iter() {
                    article { class: "explorer-card", tabindex: "0",
                        p { "Receipt " code { "{receipt.receipt_id}" } }
                        p { "Outcome: " strong { "{receipt.outcome}" } " · {short_time(receipt.created_at)}" }
                        if !receipt.evidence_gaps.is_empty() {
                            p { class: "console-limitation", "Evidence gaps: {receipt.evidence_gaps.len()}" }
                        }
                        button {
                            class: "console-action",
                            r#type: "button",
                            onclick: {
                                let receipt_id = receipt.receipt_id.clone();
                                move |_| {
                                    let environment = PitekaEnvironment {
                                        api_base_url: piteka_url.peek().clone(),
                                        tenant_id: piteka_tenant.peek().clone(),
                                        access_token: piteka_token.peek().clone(),
                                    };
                                    let receipt_id = receipt_id.clone();
                                    verify_status.set(Some((receipt_id.clone(), "Verifying locally with Parwana…".into())));
                                    spawn(async move {
                                        let message = match download_and_verify(&LivePitekaApi, &environment, &receipt_id).await {
                                            Ok(result) => format!(
                                                "Local disposition: {} · context {}",
                                                disposition_label(result.report.disposition),
                                                hex::encode(result.context_id.as_bytes()),
                                            ),
                                            Err(error) => format!("No local verification result · {error}"),
                                        };
                                        verify_status.set(Some((receipt_id, message)));
                                    });
                                }
                            },
                            "Verify locally"
                        }
                        if let Some((verified_id, message)) = verify_status() {
                            if verified_id == receipt.receipt_id {
                                output { class: "console-notice", aria_live: "polite", "{message}" }
                            }
                        }
                    }
                }
            }

            // Evidence nodes referenced by those receipts.
            if !loaded.evidence.is_empty() {
                p { class: "explorer-section-title", "Evidence" }
                for node in loaded.evidence.iter() {
                    article { class: "explorer-card", tabindex: "0",
                        p { strong { "{node.registry_id}" } " · {node.source} · {node.media_type}" }
                        p { class: "console-limitation", "digest " code { "{node.content_digest}" } }
                    }
                }
            }
        }
    }
}

/// A short, human label for an observation — a friendly name for known profiles,
/// falling back to the event type or profile id.
fn short_label(item: &ObservationProjection) -> String {
    let profile = item.normalized_profile_id.as_str();
    if profile.contains("deployment-attestation") {
        "Deployment attestation".to_string()
    } else if profile.contains("deployment") {
        "Deployment observation".to_string()
    } else if !item.source_event_type.is_empty() {
        item.source_event_type.clone()
    } else if !profile.is_empty() {
        profile.to_string()
    } else {
        "Observation".to_string()
    }
}

/// The primary subject of an observation, shortened for a one-line feed row.
/// Prefers the receipt subject, else the mandate, else the first subject.
fn primary_ref(item: &ObservationProjection) -> String {
    let pick = item
        .subject_refs
        .iter()
        .find(|s| s.starts_with("receipt:"))
        .or_else(|| item.subject_refs.iter().find(|s| s.starts_with("mandate:")))
        .or_else(|| item.subject_refs.first())
        .cloned()
        .unwrap_or_default();
    match pick.split_once(':') {
        Some((kind, id)) => format!("{kind}:{}", truncate_mid(id, 8, 6)),
        None => truncate_mid(&pick, 10, 6),
    }
}

/// Collapses a long identifier to `head…tail`, leaving short ones untouched.
fn truncate_mid(value: &str, head: usize, tail: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= head + tail + 1 {
        return value.to_string();
    }
    let start: String = chars[..head].iter().collect();
    let end: String = chars[chars.len() - tail..].iter().collect();
    format!("{start}…{end}")
}

fn retraction_badge(status: &str) -> String {
    match status {
        "active" => "active".to_string(),
        other => other.to_string(),
    }
}

/// Renders a Unix-seconds timestamp as a readable UTC datetime. Absent/zero
/// renders as `—`.
fn short_time(unix_seconds: i64) -> String {
    if unix_seconds <= 0 {
        return "—".to_string();
    }
    match chrono::DateTime::from_timestamp(unix_seconds, 0) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        None => unix_seconds.to_string(),
    }
}
