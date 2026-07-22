//! Responsibility resolution and accountability analytics (HEM-06).
//!
//! This is a read-model projection, not protocol authority. It walks disclosed
//! parent-mandate links and returns `Indeterminate` whenever a link or entity is
//! missing instead of guessing from an executor, repository, or display name.

use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityKind {
    Organization,
    Agent,
    SubAgent,
    Merchant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountableEntity {
    pub id: String,
    pub display_name: String,
    pub kind: EntityKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MandateAuthority {
    pub mandate_id: String,
    pub parent_mandate_id: Option<String>,
    /// Present only when this mandate explicitly names the accountable entity.
    pub accountable_entity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponsibilityResolution {
    Resolved {
        entity: AccountableEntity,
        authority_path: Vec<String>,
    },
    Indeterminate {
        reason: String,
        authority_path: Vec<String>,
    },
}

/// Resolves responsibility from a mandate to the first explicitly attributed
/// entity in its authority chain. Cycles, withheld parents, and unknown entities
/// fail closed as `Indeterminate`.
#[must_use]
pub fn resolve_responsibility(
    start_mandate_id: &str,
    mandates: &[MandateAuthority],
    entities: &[AccountableEntity],
) -> ResponsibilityResolution {
    let by_mandate: BTreeMap<&str, &MandateAuthority> = mandates
        .iter()
        .map(|m| (m.mandate_id.as_str(), m))
        .collect();
    let by_entity: BTreeMap<&str, &AccountableEntity> =
        entities.iter().map(|e| (e.id.as_str(), e)).collect();
    let mut current = start_mandate_id;
    let mut visited = BTreeSet::new();
    let mut path = Vec::new();

    loop {
        if !visited.insert(current.to_string()) {
            return ResponsibilityResolution::Indeterminate {
                reason: "delegation cycle detected".into(),
                authority_path: path,
            };
        }
        path.push(current.to_string());
        let Some(mandate) = by_mandate.get(current) else {
            return ResponsibilityResolution::Indeterminate {
                reason: format!("mandate {current} is not disclosed"),
                authority_path: path,
            };
        };
        if let Some(entity_id) = mandate.accountable_entity_id.as_deref() {
            return match by_entity.get(entity_id) {
                Some(entity) => ResponsibilityResolution::Resolved {
                    entity: (*entity).clone(),
                    authority_path: path,
                },
                None => ResponsibilityResolution::Indeterminate {
                    reason: format!("accountable entity {entity_id} is not disclosed"),
                    authority_path: path,
                },
            };
        }
        match mandate.parent_mandate_id.as_deref() {
            Some(parent) => current = parent,
            None => {
                return ResponsibilityResolution::Indeterminate {
                    reason: "authority chain ends without an accountable entity".into(),
                    authority_path: path,
                };
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccountabilityFact {
    pub entity_id: String,
    pub mandate_id: String,
    pub consumed: bool,
    pub disputed: bool,
    pub buffered_at_ms: Option<u64>,
    pub anchored_at_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EntityAnalytics {
    pub issued: usize,
    pub consumed: usize,
    pub disputed: usize,
    pub dispute_rate: f64,
    pub mean_anchor_lag_ms: Option<u64>,
}

#[must_use]
pub fn analytics_for(entity_id: &str, facts: &[AccountabilityFact]) -> EntityAnalytics {
    let facts: Vec<_> = facts.iter().filter(|f| f.entity_id == entity_id).collect();
    let issued = facts.len();
    let consumed = facts.iter().filter(|f| f.consumed).count();
    let disputed = facts.iter().filter(|f| f.disputed).count();
    let lags: Vec<u64> = facts
        .iter()
        .filter_map(|f| Some(f.anchored_at_ms?.saturating_sub(f.buffered_at_ms?)))
        .collect();
    EntityAnalytics {
        issued,
        consumed,
        disputed,
        dispute_rate: if issued == 0 {
            0.0
        } else {
            disputed as f64 / issued as f64
        },
        mean_anchor_lag_ms: (!lags.is_empty())
            .then(|| lags.iter().sum::<u64>() / lags.len() as u64),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn org() -> AccountableEntity {
        AccountableEntity {
            id: "org:1".into(),
            display_name: "Org One".into(),
            kind: EntityKind::Organization,
        }
    }

    #[test]
    fn resolves_seeded_delegation_chain_to_explicit_org() {
        let mandates = vec![
            MandateAuthority {
                mandate_id: "sub".into(),
                parent_mandate_id: Some("agent".into()),
                accountable_entity_id: None,
            },
            MandateAuthority {
                mandate_id: "agent".into(),
                parent_mandate_id: Some("root".into()),
                accountable_entity_id: None,
            },
            MandateAuthority {
                mandate_id: "root".into(),
                parent_mandate_id: None,
                accountable_entity_id: Some("org:1".into()),
            },
        ];
        let ResponsibilityResolution::Resolved {
            entity,
            authority_path,
        } = resolve_responsibility("sub", &mandates, &[org()])
        else {
            panic!("must resolve")
        };
        assert_eq!(entity.id, "org:1");
        assert_eq!(authority_path, ["sub", "agent", "root"]);
    }

    #[test]
    fn withheld_link_and_cycle_are_indeterminate() {
        let missing = vec![MandateAuthority {
            mandate_id: "sub".into(),
            parent_mandate_id: Some("withheld".into()),
            accountable_entity_id: None,
        }];
        assert!(matches!(
            resolve_responsibility("sub", &missing, &[org()]),
            ResponsibilityResolution::Indeterminate { .. }
        ));
        let cycle = vec![
            MandateAuthority {
                mandate_id: "a".into(),
                parent_mandate_id: Some("b".into()),
                accountable_entity_id: None,
            },
            MandateAuthority {
                mandate_id: "b".into(),
                parent_mandate_id: Some("a".into()),
                accountable_entity_id: None,
            },
        ];
        assert!(matches!(
            resolve_responsibility("a", &cycle, &[org()]),
            ResponsibilityResolution::Indeterminate { .. }
        ));
    }

    #[test]
    fn analytics_use_only_real_supplied_facts() {
        let facts = vec![
            AccountabilityFact {
                entity_id: "org:1".into(),
                mandate_id: "m1".into(),
                consumed: true,
                disputed: false,
                buffered_at_ms: Some(100),
                anchored_at_ms: Some(250),
            },
            AccountabilityFact {
                entity_id: "org:1".into(),
                mandate_id: "m2".into(),
                consumed: false,
                disputed: true,
                buffered_at_ms: Some(300),
                anchored_at_ms: Some(450),
            },
            AccountabilityFact {
                entity_id: "other".into(),
                mandate_id: "m3".into(),
                consumed: true,
                disputed: true,
                buffered_at_ms: None,
                anchored_at_ms: None,
            },
        ];
        let value = analytics_for("org:1", &facts);
        assert_eq!((value.issued, value.consumed, value.disputed), (2, 1, 1));
        assert_eq!(value.dispute_rate, 0.5);
        assert_eq!(value.mean_anchor_lag_ms, Some(150));
    }
}
