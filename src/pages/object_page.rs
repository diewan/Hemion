//! Deep-linkable accountability object pages (HEM-03).
//!
//! `/object/<kind>/<id>` resolves to a consistent detail shell for any
//! accountability object type — summary → canonical bytes → relationships —
//! cross-linked along the evidence DAG. The shell reuses the existing inspectors
//! as the destinations for canonical-byte decoding and per-kind relationships, so
//! it adds pages without replacing `object_inspector`, `dispute_inspector`,
//! `assurance_inspector`, or the Anchoring capability.

use dioxus::prelude::*;

use crate::routes::Route;
use crate::services::object_model::AccountabilityObjectKind;

/// The inspector route that decodes canonical bytes for a given object kind.
/// Object pages link here rather than re-implementing decoding.
fn inspector_route(kind: AccountabilityObjectKind) -> Route {
    match kind {
        // Mandates, actions, and receipts are decoded by the object inspector.
        AccountabilityObjectKind::Mandate
        | AccountabilityObjectKind::Action
        | AccountabilityObjectKind::Receipt => Route::ObjectInspector {},
        AccountabilityObjectKind::Dispute => Route::DisputeInspector {},
        AccountabilityObjectKind::Assurance => Route::AssuranceInspector {},
        AccountabilityObjectKind::Anchor => Route::Anchoring {},
    }
}

/// True when `id` looks like a 32-byte content digest (64 lowercase hex chars),
/// which the accountability model uses for object identifiers.
fn looks_like_digest(id: &str) -> bool {
    id.len() == 64 && id.chars().all(|c| c.is_ascii_hexdigit())
}

/// GET /object/:kind/:id — the object detail shell.
#[component]
pub fn ObjectPage(kind: String, id: String) -> Element {
    let Some(resolved) = AccountabilityObjectKind::from_slug(&kind) else {
        return rsx! {
            section { class: "console-home object-page", aria_labelledby: "object-title",
                p { class: "console-eyebrow", "HEMION / OBJECT" }
                h1 { id: "object-title", "Unknown object type" }
                p { class: "console-limitation",
                    "`{kind}` is not a known accountability object type. Known types: \
                     mandate, action, receipt, dispute, assurance, anchor."
                }
                Link { to: Route::ConsoleHome {}, class: "console-action", "← Console home" }
            }
        };
    };

    let digest_note = if looks_like_digest(&id) {
        "This identifier is a 32-byte content digest."
    } else {
        "This identifier is not a 32-byte content digest; treat it as an opaque reference."
    };

    rsx! {
        section { class: "console-home object-page", aria_labelledby: "object-title",
            p { class: "console-eyebrow", "HEMION / OBJECT · {resolved.display_name().to_uppercase()}" }
            h1 { id: "object-title", "{resolved.display_name()} object" }
            p { class: "console-lede",
                "A local object page for one {resolved.display_name()} "
                code { "{resolved.protocol_type()}" }
                ". Canonical bytes are decoded by the linked inspector; relationships \
                 follow the evidence DAG. Nothing here is a verdict."
            }

            div { class: "console-grid",
                article { class: "console-panel", aria_labelledby: "object-summary",
                    h2 { id: "object-summary", "Summary" }
                    dl {
                        div { dt { "Type" } dd { class: "console-mono", "{resolved.protocol_type()}" } }
                        div { dt { "Identifier" } dd { class: "console-mono object-id", "{id}" } }
                        div { dt { "Kind" } dd { "{resolved.display_name()}" } }
                    }
                    p { class: "console-limitation", "{digest_note}" }
                }

                article { class: "console-panel", aria_labelledby: "object-bytes",
                    h2 { id: "object-bytes", "Canonical bytes" }
                    p {
                        "Decode and inspect the canonical bytes for this object in the "
                        "{resolved.display_name()} inspector. Withheld or redacted fields "
                        "stay protected there; selective disclosure never implies an "
                        "undisclosed branch is absent."
                    }
                    Link { to: inspector_route(resolved), class: "console-action",
                        "Open {resolved.display_name()} inspector"
                    }
                }

                article { class: "console-panel", aria_labelledby: "object-responsibility",
                    h2 { id: "object-responsibility", "Accountable entity" }
                    strong { "Indeterminate" }
                    p { class: "console-limitation",
                        "This deep link does not contain a disclosed delegation chain. Hemion will not infer responsibility from an executor or display name. Load the trace or open an entity profile to resolve explicit authority."
                    }
                }
            }

            section { class: "console-panel", aria_labelledby: "object-relationships",
                h2 { id: "object-relationships", "Relationships" }
                p { class: "console-limitation",
                    "Links follow the evidence DAG. A related object opens its own page; \
                     absence of a link is not evidence the related object does not exist."
                }
                ul { class: "object-relationships",
                    for rel in resolved.relationships().iter() {
                        li {
                            span { class: "object-rel-label", "{rel.label} → " }
                            // Deep-link to the related kind's object page, carrying the
                            // current id as the initial reference until a resolver
                            // (HEM-04) supplies the concrete related id.
                            Link {
                                to: Route::ObjectPage { kind: rel.kind.slug().to_string(), id: id.clone() },
                                class: "object-rel-link",
                                "{rel.kind.display_name()}"
                            }
                            span { class: "object-rel-inspect",
                                " · "
                                Link { to: inspector_route(rel.kind), "inspect" }
                            }
                        }
                    }
                }
            }

            aside { class: "console-notice", aria_label: "Object page limitations",
                strong { "A page is not a verdict." }
                span {
                    " This view organizes a local object and its links. Authority, \
                     factual truth, and completeness are established only by the pinned \
                     Parwana verifier under an explicit context."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_shaped_ids_are_recognized() {
        let digest = "a".repeat(64);
        assert!(looks_like_digest(&digest));
        assert!(!looks_like_digest("short"));
        assert!(!looks_like_digest(&"g".repeat(64)));
    }

    #[test]
    fn every_kind_maps_to_a_real_inspector_route() {
        // Each relationship destination and each kind must map to an existing
        // inspector route (no dead links).
        for kind in AccountabilityObjectKind::all() {
            let _route = inspector_route(kind);
        }
    }
}
