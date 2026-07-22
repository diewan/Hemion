//! Portfolio-of-mandates home surface (HEM-05).
//!
//! The default entry point. It summarizes mandates grouped by accountable entity
//! and state, with the anchored-vs-buffered split shown per tile. Cards are
//! projected from **real** Piteka mandate-chain reads loaded by id — nothing is
//! simulated. With no mandates loaded it shows an explicit empty state, and the
//! local-verifier boundary language is carried as a badge rather than the whole
//! home (the austere console remains available at `/console`).

use dioxus::prelude::*;

use crate::routes::Route;
use crate::services::piteka::{LivePitekaApi, PitekaEnvironment, fetch_chain};
use crate::services::portfolio::{MandateCard, group_by_entity, portfolio_counts};

/// GET / — the portfolio home.
#[component]
pub fn PortfolioHome() -> Element {
    let mut api_url = use_signal(|| "http://127.0.0.1:3000".to_string());
    let mut tenant_id = use_signal(|| "demo-tenant".to_string());
    let mut access_token = use_signal(|| "demo-read-token".to_string());
    let mut mandate_id = use_signal(String::new);
    let mut cards = use_signal(Vec::<MandateCard>::new);
    let mut status = use_signal(|| None::<String>);

    let current = cards();
    let groups = group_by_entity(&current);
    let counts = portfolio_counts(&current);

    rsx! {
        section { class: "console-home portfolio-home", aria_labelledby: "portfolio-title",
            div { class: "portfolio-masthead",
                p { class: "console-eyebrow", "HEMION / PORTFOLIO" }
                h1 { id: "portfolio-title", "Portfolio of mandates" }
                span { class: "boundary-badge",
                    title: "Hemion accepts no verdict merely because another service recorded it.",
                    "Local verifier · results recorded elsewhere are not locally verified"
                }
            }
            p { class: "console-lede",
                "Mandates grouped by accountable entity and state, with the anchored-vs-buffered \
                 split per tile. Cards are projected from real Piteka reads; none are simulated."
            }

            // Portfolio totals.
            div { class: "portfolio-summary",
                for (label, value) in [
                    ("Mandates", counts.total),
                    ("Active", counts.active),
                    ("Disputed", counts.disputed),
                    ("Anchored", counts.anchored),
                    ("Buffered", counts.buffered),
                ] {
                    div { class: "portfolio-stat",
                        span { class: "portfolio-stat-value", "{value}" }
                        span { class: "portfolio-stat-label", "{label}" }
                    }
                }
            }

            // Load a real mandate by id from the authorized read API.
            details { class: "console-panel portfolio-loader",
                summary { "Load a mandate by id" }
                div { class: "console-grid",
                    label { r#for: "pf-api", "Piteka API URL" input { id: "pf-api", r#type: "url", value: "{api_url}", oninput: move |e| api_url.set(e.value()) } }
                    label { r#for: "pf-tenant", "Tenant" input { id: "pf-tenant", value: "{tenant_id}", oninput: move |e| tenant_id.set(e.value()) } }
                    label { r#for: "pf-token", "Access token" input { id: "pf-token", r#type: "password", autocomplete: "off", value: "{access_token}", oninput: move |e| access_token.set(e.value()) } }
                    label { r#for: "pf-mandate", "Mandate id" input { id: "pf-mandate", value: "{mandate_id}", oninput: move |e| mandate_id.set(e.value()) } }
                }
                button { class: "console-action", r#type: "button",
                    disabled: mandate_id().trim().is_empty(),
                    onclick: move |_| {
                        let environment = PitekaEnvironment { api_base_url: api_url(), tenant_id: tenant_id(), access_token: access_token() };
                        let id = mandate_id().trim().to_string();
                        status.set(Some(format!("Loading mandate {id}…")));
                        spawn(async move {
                            match fetch_chain(&LivePitekaApi, &environment, &id).await {
                                Ok(chain) => {
                                    let card = MandateCard::from_chain(&chain);
                                    let mut updated = cards();
                                    if !updated.iter().any(|c| c.id == card.id) { updated.push(card); }
                                    cards.set(updated);
                                    status.set(Some(format!("Loaded mandate {id} · recorded elsewhere, not locally verified.")));
                                }
                                Err(error) => status.set(Some(format!("Load failed · {error}"))),
                            }
                        });
                    },
                    "Add to portfolio"
                }
                if let Some(message) = status() { output { class: "console-notice", aria_live: "polite", "{message}" } }
            }

            if groups.is_empty() {
                div { class: "console-panel portfolio-empty",
                    h2 { "No mandates loaded" }
                    p { class: "console-limitation",
                        "Load a mandate by id above to populate the portfolio, or open the "
                        Link { to: Route::ConsoleHome {}, "developer console" }
                        " for the local inspection tools."
                    }
                }
            } else {
                div { class: "portfolio-entities",
                    for group in groups.iter() {
                        article { class: "portfolio-entity console-panel", aria_label: "Entity {group.entity}",
                            header { class: "portfolio-entity-head",
                                h2 { class: "console-mono", "{group.entity}" }
                                span { class: "portfolio-entity-counts",
                                    "{group.active()} active · {group.disputed()} disputed · {group.anchored()} anchored"
                                }
                            }
                            ul { class: "portfolio-tiles",
                                for card in group.cards.iter() {
                                    li { class: "portfolio-tile", "data-lifecycle": "{card.lifecycle.label()}", "data-disputed": if card.disputed { "true" } else { "false" },
                                        Link {
                                            to: Route::ObjectPage { kind: "mandate".to_string(), id: card.id.clone() },
                                            class: "portfolio-tile-id console-mono",
                                            "{card.id}"
                                        }
                                        div { class: "portfolio-tile-badges",
                                            span { class: "portfolio-badge", "{card.lifecycle.label()}" }
                                            if card.disputed { span { class: "portfolio-badge portfolio-badge-disputed", "disputed" } }
                                            span { class: "portfolio-badge", if card.anchored { "anchored" } else { "buffered only" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
