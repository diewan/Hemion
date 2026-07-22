//! Universal accountability search + lineage graph (HEM-04).
//!
//! One field classifies a query and routes it to the right object; a lineage
//! graph walks the accountability chain in both directions with a keyboard
//! accessible mirror table and node-type filters. Unknown or ambiguous queries
//! render an explicit no-match/disambiguation state — never a wrong object.

use dioxus::prelude::*;

use crate::routes::Route;
use crate::services::lineage::{Direction, LineageGraph, LineageNodeType, demo_lineage};
use crate::services::object_model::AccountabilityObjectKind;
use crate::services::search::{SearchResolution, SearchTarget, classify};

/// Maps a resolved single-object target to its route, when one exists.
fn target_route(target: &SearchTarget) -> Option<Route> {
    match target {
        SearchTarget::Object { kind, id } => Some(Route::ObjectPage {
            kind: kind.slug().to_string(),
            id: id.clone(),
        }),
        SearchTarget::EnvironmentReceipt {
            environment_id,
            receipt_id,
        } => Some(Route::PitekaEnvironmentReceipt {
            environment_id: environment_id.clone(),
            receipt_id: receipt_id.clone(),
        }),
        // A chain tx is the reference an anchor object carries.
        SearchTarget::ChainTx { tx, .. } => Some(Route::ObjectPage {
            kind: AccountabilityObjectKind::Anchor.slug().to_string(),
            id: tx.clone(),
        }),
        // Entity profiles are HEM-06; there is no object page to route to yet.
        SearchTarget::Entity { .. } => None,
    }
}

const FILTERABLE_TYPES: [LineageNodeType; 6] = [
    LineageNodeType::Observation,
    LineageNodeType::Claim,
    LineageNodeType::Attestation,
    LineageNodeType::Gap,
    LineageNodeType::Dispute,
    LineageNodeType::Anchor,
];

/// GET /search — universal search + lineage graph.
#[component]
pub fn Search() -> Element {
    let mut query = use_signal(String::new);
    let mut resolution = use_signal(|| None::<SearchResolution>);
    let mut active_filters = use_signal(Vec::<LineageNodeType>::new);

    let graph = demo_lineage();
    let filters = active_filters();
    let visible: Vec<_> = graph.filter_by_type(&filters);

    rsx! {
        section { class: "console-home accountability-search", aria_labelledby: "search-title",
            p { class: "console-eyebrow", "HEMION / SEARCH" }
            h1 { id: "search-title", "Universal accountability search" }
            p { class: "console-lede",
                "Resolve a mandate, receipt, action, dispute, assurance, anchor, entity, \
                 chain tx, or environment/receipt path. A bare digest is ambiguous and is \
                 never routed to a wrong object."
            }

            form {
                class: "search-form",
                onsubmit: move |event| {
                    event.prevent_default();
                    resolution.set(Some(classify(&query())));
                },
                label { r#for: "search-input", class: "sr-only", "Search identifier" }
                input {
                    id: "search-input",
                    r#type: "search",
                    value: "{query}",
                    oninput: move |event| query.set(event.value()),
                    placeholder: "mandate:<digest>, environments/prod/receipts/<id>, entity:<id>…",
                }
                button { class: "console-action", r#type: "submit", "Resolve" }
            }

            if let Some(result) = resolution() {
                {render_resolution(&result)}
            }

            section { class: "console-panel", aria_labelledby: "lineage-title",
                h2 { id: "lineage-title", "Lineage graph" }
                p { class: "console-limitation",
                    "Graph shape is structure, not truth: no node is ranked as more valid, \
                     and a withheld or missing branch is shown as an explicit gap."
                }

                fieldset { class: "search-filters",
                    legend { "Filter by node type" }
                    for node_type in FILTERABLE_TYPES {
                        button {
                            r#type: "button",
                            class: "console-action",
                            aria_pressed: if filters.contains(&node_type) { "true" } else { "false" },
                            onclick: move |_| {
                                let mut current = active_filters();
                                if let Some(index) = current.iter().position(|t| *t == node_type) {
                                    current.remove(index);
                                } else {
                                    current.push(node_type);
                                }
                                active_filters.set(current);
                            },
                            "{node_type.label()}"
                        }
                    }
                }

                // Accessible mirror table of the (filtered) graph. The table is the
                // keyboard-navigable alternative to the visual graph and carries the
                // same nodes, types, and both-direction links.
                table { class: "lineage-table",
                    caption { "Lineage nodes and their upstream/downstream links" }
                    thead {
                        tr {
                            th { scope: "col", "Node" }
                            th { scope: "col", "Type" }
                            th { scope: "col", "Upstream (references)" }
                            th { scope: "col", "Downstream (referenced by)" }
                        }
                    }
                    tbody {
                        if visible.is_empty() {
                            tr { td { colspan: "4", class: "console-limitation", "No nodes match the active filters." } }
                        }
                        for node in visible.iter() {
                            {render_node_row(&graph, node)}
                        }
                    }
                }
            }

            aside { class: "console-notice", aria_label: "Search limitations",
                strong { "Discovery is not verification." }
                span {
                    " Resolving an id or walking lineage organizes objects; it does not \
                     establish authority, factual truth, or completeness."
                }
            }
        }
    }
}

fn render_resolution(result: &SearchResolution) -> Element {
    match result {
        SearchResolution::Resolved(target) => {
            if let Some(route) = target_route(target) {
                rsx! {
                    div { class: "console-panel search-result", aria_live: "polite",
                        h2 { "Resolved" }
                        p { "This identifier resolves to a single object." }
                        Link { to: route, class: "console-action", "Go to object" }
                    }
                }
            } else {
                // Entity: classified, but entity profiles are HEM-06.
                rsx! {
                    div { class: "console-panel search-result", aria_live: "polite",
                        h2 { "Accountable entity" }
                        p { class: "console-limitation",
                            "This is an accountable entity. Entity profiles arrive in HEM-06; \
                             no single object page exists for it yet."
                        }
                    }
                }
            }
        }
        SearchResolution::Ambiguous { candidates, query } => rsx! {
            div { class: "console-panel search-result", aria_live: "polite",
                h2 { "Ambiguous identifier" }
                p { "A bare digest could be several object kinds. Choose one, or re-enter a typed query like " code { "mandate:<digest>" } "." }
                ul {
                    for kind in candidates.iter() {
                        li {
                            Link {
                                to: Route::ObjectPage { kind: kind.slug().to_string(), id: query.clone() },
                                class: "console-action",
                                "Open as {kind.display_name()}"
                            }
                        }
                    }
                }
            }
        },
        SearchResolution::NoMatch { reason, .. } => rsx! {
            div { class: "console-panel search-result", aria_live: "assertive",
                h2 { "No match" }
                p { class: "console-limitation", "{reason}" }
            }
        },
    }
}

fn render_node_row(graph: &LineageGraph, node: &crate::services::lineage::LineageNode) -> Element {
    let upstream = graph.neighbors(&node.id, Direction::Upstream);
    let downstream = graph.neighbors(&node.id, Direction::Downstream);
    rsx! {
        tr {
            th { scope: "row",
                span { class: "console-mono", "{node.id}" }
                span { class: "lineage-node-title", " · {node.title}" }
            }
            td { "{node.node_type.label()}" }
            td {
                if upstream.is_empty() { "—" }
                else {
                    ul { class: "lineage-links",
                        for neighbor in upstream.iter() {
                            li { span { class: "console-mono", "{neighbor.id}" } }
                        }
                    }
                }
            }
            td {
                if downstream.is_empty() { "—" }
                else {
                    ul { class: "lineage-links",
                        for neighbor in downstream.iter() {
                            li { span { class: "console-mono", "{neighbor.id}" } }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_object_targets_route_but_entity_does_not() {
        let object = SearchTarget::Object {
            kind: AccountabilityObjectKind::Receipt,
            id: "r1".to_string(),
        };
        assert!(target_route(&object).is_some());

        let env = SearchTarget::EnvironmentReceipt {
            environment_id: "prod".to_string(),
            receipt_id: "r1".to_string(),
        };
        assert!(target_route(&env).is_some());

        let entity = SearchTarget::Entity {
            entity: "svc:agent".to_string(),
        };
        assert!(target_route(&entity).is_none());
    }
}
