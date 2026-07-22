//! Portfolio-of-mandates model for the home surface (HEM-05).
//!
//! The home page summarizes mandates grouped by accountable entity and by state,
//! with the anchored-vs-buffered split visible. This module is the pure backbone:
//! it maps a **real** Piteka mandate-chain projection into a portfolio card and
//! groups cards by entity. It never fabricates mandates — cards exist only for
//! chains actually fetched from the read API; with none loaded the home shows an
//! explicit empty state.

use crate::services::piteka::MandateChain;

/// The lifecycle state of a mandate, projected from its live-state string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MandateLifecycle {
    /// Issued or reserved — authorized and not yet terminal.
    Active,
    /// Consumed — the single use was spent.
    Consumed,
    /// Quarantined — held after a possible external effect; needs attention.
    Quarantined,
    /// Abandoned — terminated without consumption.
    Abandoned,
    /// An unrecognized live-state string; preserved rather than guessed.
    Unknown,
}

impl MandateLifecycle {
    /// Projects a Piteka mandate live-state string into a lifecycle.
    #[must_use]
    pub fn from_state(state: &str) -> Self {
        match state.trim().to_ascii_lowercase().as_str() {
            "issued" | "reserved" => Self::Active,
            "consumed" => Self::Consumed,
            "quarantined" => Self::Quarantined,
            "abandoned" => Self::Abandoned,
            _ => Self::Unknown,
        }
    }

    /// A stable lowercase label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Consumed => "consumed",
            Self::Quarantined => "quarantined",
            Self::Abandoned => "abandoned",
            Self::Unknown => "unknown",
        }
    }
}

/// One mandate as a portfolio tile, projected from a real mandate chain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MandateCard {
    /// The mandate identifier.
    pub id: String,
    /// The accountable entity (the executor identity, when attributed).
    pub entity: String,
    /// The lifecycle state.
    pub lifecycle: MandateLifecycle,
    /// Whether this mandate carries a dispute signal (quarantined, or a receipt
    /// reports evidence gaps).
    pub disputed: bool,
    /// Whether the mandate is present in the buffered (DB/observation) plane. A
    /// fetched chain always is.
    pub buffered: bool,
    /// Whether the mandate carries an external chain-anchor evidence node.
    pub anchored: bool,
}

/// Registry id of an external commitment anchor (mirrors
/// `csv_accountability::anchor::EVIDENCE_CSV_SEAL_COMMITMENT_ANCHOR`).
const COMMITMENT_ANCHOR_REGISTRY_ID: &str = "evidence.csv-seal.commitment-anchor";

impl MandateCard {
    /// Projects a real mandate chain into a portfolio card.
    #[must_use]
    pub fn from_chain(chain: &MandateChain) -> Self {
        let lifecycle = MandateLifecycle::from_state(&chain.mandate.state);
        let entity = chain
            .attempts
            .first()
            .map(|attempt| attempt.executor_identity.clone())
            .or_else(|| {
                chain
                    .timeline
                    .iter()
                    .find_map(|step| step.actor.clone())
            })
            .filter(|identity| !identity.trim().is_empty())
            .unwrap_or_else(|| "unattributed".to_string());

        let has_gap = chain
            .receipts
            .iter()
            .any(|receipt| !receipt.evidence_gaps.is_empty());
        let disputed = lifecycle == MandateLifecycle::Quarantined || has_gap;

        let anchored = chain
            .evidence
            .iter()
            .any(|node| node.registry_id == COMMITMENT_ANCHOR_REGISTRY_ID);

        Self {
            id: chain.mandate.mandate_id.clone(),
            entity,
            lifecycle,
            disputed,
            // A fetched chain is, by construction, present in the DB buffer.
            buffered: true,
            anchored,
        }
    }
}

/// Mandates for one accountable entity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityGroup {
    /// The accountable entity.
    pub entity: String,
    /// The entity's mandate cards, in insertion order.
    pub cards: Vec<MandateCard>,
}

impl EntityGroup {
    /// Count of active mandates in this group.
    #[must_use]
    pub fn active(&self) -> usize {
        self.cards
            .iter()
            .filter(|c| c.lifecycle == MandateLifecycle::Active)
            .count()
    }

    /// Count of disputed mandates in this group.
    #[must_use]
    pub fn disputed(&self) -> usize {
        self.cards.iter().filter(|c| c.disputed).count()
    }

    /// Count of anchored mandates in this group.
    #[must_use]
    pub fn anchored(&self) -> usize {
        self.cards.iter().filter(|c| c.anchored).count()
    }
}

/// Groups cards by accountable entity, preserving first-seen entity order and
/// per-entity insertion order.
#[must_use]
pub fn group_by_entity(cards: &[MandateCard]) -> Vec<EntityGroup> {
    let mut groups: Vec<EntityGroup> = Vec::new();
    for card in cards {
        if let Some(group) = groups.iter_mut().find(|g| g.entity == card.entity) {
            group.cards.push(card.clone());
        } else {
            groups.push(EntityGroup {
                entity: card.entity.clone(),
                cards: vec![card.clone()],
            });
        }
    }
    groups
}

/// Portfolio-wide totals across all entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PortfolioCounts {
    /// Total mandates.
    pub total: usize,
    /// Active mandates.
    pub active: usize,
    /// Disputed mandates.
    pub disputed: usize,
    /// Anchored mandates.
    pub anchored: usize,
    /// Buffered mandates.
    pub buffered: usize,
}

/// Computes portfolio-wide totals.
#[must_use]
pub fn portfolio_counts(cards: &[MandateCard]) -> PortfolioCounts {
    PortfolioCounts {
        total: cards.len(),
        active: cards
            .iter()
            .filter(|c| c.lifecycle == MandateLifecycle::Active)
            .count(),
        disputed: cards.iter().filter(|c| c.disputed).count(),
        anchored: cards.iter().filter(|c| c.anchored).count(),
        buffered: cards.iter().filter(|c| c.buffered).count(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::piteka::{ChainAttempt, ChainEvidence, MandateChain, MandateDetail, ReceiptDetail};

    fn chain(id: &str, state: &str, executor: &str, anchored: bool, gap: bool) -> MandateChain {
        MandateChain {
            mandate: MandateDetail {
                mandate_id: id.to_string(),
                state: state.to_string(),
                version: 1,
            },
            timeline: vec![],
            attempts: vec![ChainAttempt {
                attempt_id: "att".to_string(),
                executor_identity: executor.to_string(),
                state: "dispatched".to_string(),
                github_deployment_id: None,
                started_at: 0,
            }],
            receipts: vec![ReceiptDetail {
                receipt_id: "r".to_string(),
                mandate_id: id.to_string(),
                intent_id: "i".to_string(),
                attempt_id: "att".to_string(),
                outcome: "success".to_string(),
                created_at: 0,
                dispatch_evidence_refs: vec![],
                target_evidence_refs: vec![],
                evidence_gaps: if gap { vec!["target".to_string()] } else { vec![] },
            }],
            evidence: if anchored {
                vec![ChainEvidence {
                    node_id: "n".to_string(),
                    registry_id: COMMITMENT_ANCHOR_REGISTRY_ID.to_string(),
                    source: "s".to_string(),
                    producer_identity: "p".to_string(),
                    content_digest: "d".to_string(),
                    media_type: "m".to_string(),
                }]
            } else {
                vec![]
            },
        }
    }

    #[test]
    fn lifecycle_projects_known_states_and_preserves_unknown() {
        assert_eq!(MandateLifecycle::from_state("issued"), MandateLifecycle::Active);
        assert_eq!(MandateLifecycle::from_state("Reserved"), MandateLifecycle::Active);
        assert_eq!(MandateLifecycle::from_state("consumed"), MandateLifecycle::Consumed);
        assert_eq!(
            MandateLifecycle::from_state("quarantined"),
            MandateLifecycle::Quarantined
        );
        assert_eq!(MandateLifecycle::from_state("weird"), MandateLifecycle::Unknown);
    }

    #[test]
    fn card_projection_maps_entity_dispute_and_anchor() {
        let anchored = MandateCard::from_chain(&chain("m1", "consumed", "svc:agent", true, false));
        assert_eq!(anchored.entity, "svc:agent");
        assert_eq!(anchored.lifecycle, MandateLifecycle::Consumed);
        assert!(anchored.anchored);
        assert!(anchored.buffered);
        assert!(!anchored.disputed);

        // Quarantined OR a receipt gap marks the card disputed; no anchor node.
        let quarantined = MandateCard::from_chain(&chain("m2", "quarantined", "svc:agent", false, false));
        assert!(quarantined.disputed);
        assert!(!quarantined.anchored);
        let gapped = MandateCard::from_chain(&chain("m3", "issued", "svc:agent", false, true));
        assert!(gapped.disputed);
    }

    #[test]
    fn grouping_and_counts_summarize_by_entity_and_state() {
        let cards = vec![
            MandateCard::from_chain(&chain("m1", "issued", "svc:a", false, false)),
            MandateCard::from_chain(&chain("m2", "consumed", "svc:a", true, false)),
            MandateCard::from_chain(&chain("m3", "quarantined", "svc:b", false, false)),
        ];
        let groups = group_by_entity(&cards);
        assert_eq!(groups.len(), 2);
        let a = groups.iter().find(|g| g.entity == "svc:a").unwrap();
        assert_eq!(a.cards.len(), 2);
        assert_eq!(a.active(), 1);
        assert_eq!(a.anchored(), 1);
        let b = groups.iter().find(|g| g.entity == "svc:b").unwrap();
        assert_eq!(b.disputed(), 1);

        let counts = portfolio_counts(&cards);
        assert_eq!(counts.total, 3);
        assert_eq!(counts.active, 1);
        assert_eq!(counts.disputed, 1);
        assert_eq!(counts.anchored, 1);
        assert_eq!(counts.buffered, 3);
    }
}
