//! The accountability object model for Hemion's object pages (HEM-03).
//!
//! Every accountability object type — mandate, action/attempt, receipt, dispute,
//! verdict/assurance, anchor — has a dedicated, deep-linkable detail page. This
//! module is the pure, framework-free backbone of those pages: it names the
//! object kinds, their stable route slugs, and the cross-links between them that
//! follow the evidence DAG. Rendering and routing live in the page component; the
//! model here is fully unit-testable and target-neutral.
//!
//! It also carries two small disclosure helpers the pages must get right:
//! reason-code rendering that preserves stable namespaced identifiers, and field
//! disclosure that keeps withheld/redacted values protected even in developer
//! mode (selective disclosure never implies an undisclosed branch is absent).

/// One accountability object type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AccountabilityObjectKind {
    /// A pre-action authorization (the strongest authority artifact).
    Mandate,
    /// An execution action / attempt against a mandate.
    Action,
    /// The receipt recording what an action did (may report `Unknown`).
    Receipt,
    /// A dispute bundle assembled from evidence.
    Dispute,
    /// An independent assurance result / verdict over a dispute.
    Assurance,
    /// A chain anchor corroborating a commitment (optional evidence).
    Anchor,
}

/// A labelled edge from one object kind to a related kind, following the evidence
/// DAG. Labels read from the subject object toward the related object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Relationship {
    /// The relationship label (e.g. `records`).
    pub label: &'static str,
    /// The related object kind.
    pub kind: AccountabilityObjectKind,
}

impl AccountabilityObjectKind {
    /// Every object kind, in a stable display order.
    #[must_use]
    pub const fn all() -> [Self; 6] {
        [
            Self::Mandate,
            Self::Action,
            Self::Receipt,
            Self::Dispute,
            Self::Assurance,
            Self::Anchor,
        ]
    }

    /// The stable URL slug for this kind (used in `/object/<slug>/<id>`).
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Mandate => "mandate",
            Self::Action => "action",
            Self::Receipt => "receipt",
            Self::Dispute => "dispute",
            Self::Assurance => "assurance",
            Self::Anchor => "anchor",
        }
    }

    /// The human-readable name.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Mandate => "Mandate",
            Self::Action => "Action",
            Self::Receipt => "Receipt",
            Self::Dispute => "Dispute",
            Self::Assurance => "Assurance",
            Self::Anchor => "Anchor",
        }
    }

    /// The canonical Parwana type name shown on the page.
    #[must_use]
    pub const fn protocol_type(self) -> &'static str {
        match self {
            Self::Mandate => "ActionMandate",
            Self::Action => "ExecutionAttempt",
            Self::Receipt => "ExecutionReceipt",
            Self::Dispute => "DisputeBundle",
            Self::Assurance => "AssuranceResult",
            Self::Anchor => "CommitmentAnchorRecord",
        }
    }

    /// Resolves a kind from its URL slug.
    #[must_use]
    pub fn from_slug(slug: &str) -> Option<Self> {
        Self::all().into_iter().find(|kind| kind.slug() == slug)
    }

    /// The cross-links from this object kind, following the evidence DAG. Edges
    /// point from an object toward the objects it references (child → parents)
    /// plus the sensible reverse links used to walk a trace.
    #[must_use]
    pub fn relationships(self) -> &'static [Relationship] {
        use AccountabilityObjectKind::{Action, Anchor, Assurance, Dispute, Mandate, Receipt};
        match self {
            Self::Mandate => &[
                Relationship {
                    label: "authorizes",
                    kind: Action,
                },
                Relationship {
                    label: "corroborated by",
                    kind: Anchor,
                },
                Relationship {
                    label: "receipted by",
                    kind: Receipt,
                },
            ],
            Self::Action => &[
                Relationship {
                    label: "under mandate",
                    kind: Mandate,
                },
                Relationship {
                    label: "produces",
                    kind: Receipt,
                },
            ],
            Self::Receipt => &[
                Relationship {
                    label: "records",
                    kind: Action,
                },
                Relationship {
                    label: "under mandate",
                    kind: Mandate,
                },
                Relationship {
                    label: "disputed in",
                    kind: Dispute,
                },
            ],
            Self::Dispute => &[
                Relationship {
                    label: "about receipt",
                    kind: Receipt,
                },
                Relationship {
                    label: "about mandate",
                    kind: Mandate,
                },
                Relationship {
                    label: "assessed by",
                    kind: Assurance,
                },
            ],
            Self::Assurance => &[Relationship {
                label: "assesses",
                kind: Dispute,
            }],
            Self::Anchor => &[Relationship {
                label: "corroborates",
                kind: Mandate,
            }],
        }
    }
}

/// A reason code split into its stable namespace and terminal, preserving the
/// exact identifier. Reason codes are stable namespaced strings such as
/// `ACCOUNTABILITY.AUTHORITY.INTENT_MISMATCH`; the full id is never rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReasonCodeDisplay {
    /// The exact, unmodified stable identifier.
    pub stable_id: String,
    /// The dotted namespace prefix (everything before the last segment), if any.
    pub namespace: Option<String>,
    /// The terminal segment (the last dotted component).
    pub terminal: String,
}

/// Renders a reason code for display while preserving its stable identifier.
#[must_use]
pub fn reason_code_display(code: &str) -> ReasonCodeDisplay {
    let trimmed = code.trim();
    match trimmed.rsplit_once('.') {
        Some((namespace, terminal)) => ReasonCodeDisplay {
            stable_id: trimmed.to_string(),
            namespace: Some(namespace.to_string()),
            terminal: terminal.to_string(),
        },
        None => ReasonCodeDisplay {
            stable_id: trimmed.to_string(),
            namespace: None,
            terminal: trimmed.to_string(),
        },
    }
}

/// The disclosure state of one field on an object page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldDisclosure {
    /// The value is disclosed and may be shown.
    Disclosed(String),
    /// The field was selectively withheld. Its value is never shown, and its
    /// absence does not imply the field is empty or the branch does not exist.
    Withheld,
    /// The field is redacted (e.g. a secret) and must stay protected even in
    /// developer mode.
    Redacted,
}

impl FieldDisclosure {
    /// The display string for this field. Withheld and redacted fields render a
    /// protected marker; the underlying value is never returned.
    #[must_use]
    pub fn display(&self) -> String {
        match self {
            Self::Disclosed(value) => value.clone(),
            Self::Withheld => {
                "Withheld — selectively disclosed. Absence here does not mean the field is empty."
                    .to_string()
            }
            Self::Redacted => "Redacted — protected; not shown in developer mode.".to_string(),
        }
    }

    /// Whether this field is protected (withheld or redacted).
    #[must_use]
    pub const fn is_protected(&self) -> bool {
        matches!(self, Self::Withheld | Self::Redacted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_kind_round_trips_through_its_slug() {
        for kind in AccountabilityObjectKind::all() {
            assert_eq!(AccountabilityObjectKind::from_slug(kind.slug()), Some(kind));
        }
        assert_eq!(AccountabilityObjectKind::from_slug("not-a-kind"), None);
    }

    #[test]
    fn dag_connects_the_full_mandate_receipt_dispute_assurance_chain() {
        use AccountabilityObjectKind::{Assurance, Dispute, Mandate, Receipt};
        // Walk the chain the navigation test exercises: each hop must be a real
        // relationship edge on the model.
        let has_edge = |from: AccountabilityObjectKind, to: AccountabilityObjectKind| {
            from.relationships().iter().any(|r| r.kind == to)
        };
        assert!(has_edge(Receipt, Mandate), "receipt → mandate");
        assert!(has_edge(Dispute, Receipt), "dispute → receipt");
        assert!(has_edge(Dispute, Mandate), "dispute → mandate");
        assert!(has_edge(Assurance, Dispute), "assurance → dispute");
        // And the reverse links used to walk forward through a trace.
        assert!(has_edge(Mandate, Receipt), "mandate → receipt");
        assert!(has_edge(Receipt, Dispute), "receipt → dispute");
    }

    #[test]
    fn relationships_never_point_to_an_unknown_kind() {
        for kind in AccountabilityObjectKind::all() {
            for rel in kind.relationships() {
                // Every related kind must itself be resolvable to a page.
                assert!(
                    AccountabilityObjectKind::from_slug(rel.kind.slug()).is_some(),
                    "{} → {} resolvable",
                    kind.slug(),
                    rel.kind.slug()
                );
            }
        }
    }

    #[test]
    fn reason_code_preserves_stable_identifier() {
        let display = reason_code_display("ACCOUNTABILITY.AUTHORITY.INTENT_MISMATCH");
        assert_eq!(
            display.stable_id,
            "ACCOUNTABILITY.AUTHORITY.INTENT_MISMATCH"
        );
        assert_eq!(
            display.namespace.as_deref(),
            Some("ACCOUNTABILITY.AUTHORITY")
        );
        assert_eq!(display.terminal, "INTENT_MISMATCH");
    }

    #[test]
    fn withheld_and_redacted_fields_never_reveal_their_value() {
        let secret = FieldDisclosure::Redacted;
        assert!(secret.is_protected());
        assert!(!secret.display().contains("supersecret"));
        assert!(secret.display().contains("Redacted"));

        let withheld = FieldDisclosure::Withheld;
        assert!(withheld.is_protected());
        // The protected marker must not claim the field is empty/absent.
        assert!(
            withheld
                .display()
                .contains("does not mean the field is empty")
        );

        let shown = FieldDisclosure::Disclosed("abc123".to_string());
        assert!(!shown.is_protected());
        assert_eq!(shown.display(), "abc123");
    }
}
