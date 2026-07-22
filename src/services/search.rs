//! Universal accountability search resolution (HEM-04).
//!
//! One search field classifies a query — a mandate id, receipt, action, dispute,
//! assurance, commitment/anchor, accountable entity, chain tx, or an
//! environment/receipt path — and routes it to the right destination. The
//! resolver is pure and target-neutral; the page maps a [`SearchTarget`] to a
//! concrete route.
//!
//! The safety rule: the resolver **never guesses a wrong object**. A bare 32-byte
//! digest could be several object kinds, so it resolves to [`SearchResolution::Ambiguous`]
//! with the candidate kinds rather than picking one; anything unrecognized is an
//! explicit [`SearchResolution::NoMatch`]. Only a typed, unambiguous query
//! resolves to a single object.

use crate::services::object_model::AccountabilityObjectKind;

/// A resolved search destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchTarget {
    /// A single accountability object with a known kind and id.
    Object {
        /// The object kind.
        kind: AccountabilityObjectKind,
        /// The object identifier (hex digest or opaque reference).
        id: String,
    },
    /// A Piteka environment/receipt path (`environments/<env>/receipts/<rcpt>`).
    EnvironmentReceipt {
        /// The environment identifier.
        environment_id: String,
        /// The receipt identifier.
        receipt_id: String,
    },
    /// An accountable entity. Entity profiles are HEM-06; the resolver classifies
    /// the query so the UI can route to entity lineage without inventing an
    /// object page.
    Entity {
        /// The entity identifier.
        entity: String,
    },
    /// A chain transaction reference (optionally chain-qualified).
    ChainTx {
        /// The chain id, if the query qualified one (`tx:<chain>/<hash>`).
        chain: Option<String>,
        /// The transaction hash.
        tx: String,
    },
}

/// The outcome of classifying a search query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchResolution {
    /// The query resolved to exactly one destination.
    Resolved(SearchTarget),
    /// The query is well-formed but could be several object kinds; the caller
    /// must disambiguate. Never routed to a single object automatically.
    Ambiguous {
        /// The original query.
        query: String,
        /// The candidate kinds the bare identifier could name.
        candidates: Vec<AccountabilityObjectKind>,
    },
    /// The query is empty or unrecognized.
    NoMatch {
        /// The original query.
        query: String,
        /// A short, human-readable reason.
        reason: String,
    },
}

fn is_hex_digest(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

/// A typed `kind:<hex>` prefix maps to a single object kind.
fn kind_for_prefix(prefix: &str) -> Option<AccountabilityObjectKind> {
    match prefix {
        "mandate" => Some(AccountabilityObjectKind::Mandate),
        "action" | "attempt" => Some(AccountabilityObjectKind::Action),
        "receipt" => Some(AccountabilityObjectKind::Receipt),
        "dispute" => Some(AccountabilityObjectKind::Dispute),
        "assurance" | "verdict" => Some(AccountabilityObjectKind::Assurance),
        "anchor" | "commitment" => Some(AccountabilityObjectKind::Anchor),
        _ => None,
    }
}

/// Classifies a search query into a resolution.
#[must_use]
pub fn classify(raw: &str) -> SearchResolution {
    let query = raw.trim();
    if query.is_empty() {
        return SearchResolution::NoMatch {
            query: query.to_string(),
            reason: "Enter a mandate, receipt, action, dispute, assurance, anchor, entity, \
                     chain tx, or environment/receipt path."
                .to_string(),
        };
    }

    // Environment/receipt path: environments/<env>/receipts/<rcpt>.
    if let Some(target) = classify_environment_path(query) {
        return SearchResolution::Resolved(target);
    }

    // Typed prefixes. A single `:` splits `kind:value`.
    if let Some((prefix, value)) = query.split_once(':') {
        let value = value.trim();
        if !value.is_empty() {
            if prefix == "entity" {
                return SearchResolution::Resolved(SearchTarget::Entity {
                    entity: value.to_string(),
                });
            }
            if prefix == "tx" {
                // `tx:<chain>/<hash>` or `tx:<hash>`.
                let (chain, tx) = match value.split_once('/') {
                    Some((chain, tx)) if !chain.is_empty() && !tx.is_empty() => {
                        (Some(chain.to_string()), tx.to_string())
                    }
                    _ => (None, value.to_string()),
                };
                return SearchResolution::Resolved(SearchTarget::ChainTx { chain, tx });
            }
            if let Some(kind) = kind_for_prefix(prefix) {
                return SearchResolution::Resolved(SearchTarget::Object {
                    kind,
                    id: value.to_string(),
                });
            }
            // A recognized-looking prefix that is not a known kind.
            return SearchResolution::NoMatch {
                query: query.to_string(),
                reason: format!("`{prefix}` is not a known object type."),
            };
        }
    }

    // A bare 32-byte digest could be several object kinds — never guess.
    if is_hex_digest(query) {
        return SearchResolution::Ambiguous {
            query: query.to_string(),
            candidates: vec![
                AccountabilityObjectKind::Mandate,
                AccountabilityObjectKind::Receipt,
                AccountabilityObjectKind::Anchor,
            ],
        };
    }

    SearchResolution::NoMatch {
        query: query.to_string(),
        reason: "Unrecognized identifier. Prefix it with its type, e.g. `mandate:<digest>`."
            .to_string(),
    }
}

fn classify_environment_path(query: &str) -> Option<SearchTarget> {
    // environments/<env>/receipts/<rcpt>
    let parts: Vec<&str> = query.split('/').collect();
    if parts.len() == 4
        && parts[0] == "environments"
        && parts[2] == "receipts"
        && !parts[1].is_empty()
        && !parts[3].is_empty()
    {
        return Some(SearchTarget::EnvironmentReceipt {
            environment_id: parts[1].to_string(),
            receipt_id: parts[3].to_string(),
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> String {
        "a".repeat(64)
    }

    #[test]
    fn typed_object_prefixes_resolve_to_the_correct_kind() {
        let cases = [
            ("mandate", AccountabilityObjectKind::Mandate),
            ("receipt", AccountabilityObjectKind::Receipt),
            ("action", AccountabilityObjectKind::Action),
            ("attempt", AccountabilityObjectKind::Action),
            ("dispute", AccountabilityObjectKind::Dispute),
            ("assurance", AccountabilityObjectKind::Assurance),
            ("verdict", AccountabilityObjectKind::Assurance),
            ("anchor", AccountabilityObjectKind::Anchor),
            ("commitment", AccountabilityObjectKind::Anchor),
        ];
        for (prefix, expected) in cases {
            let query = format!("{prefix}:{}", digest());
            match classify(&query) {
                SearchResolution::Resolved(SearchTarget::Object { kind, id }) => {
                    assert_eq!(kind, expected, "{prefix}");
                    assert_eq!(id, digest());
                }
                other => panic!("{prefix} resolved to {other:?}"),
            }
        }
    }

    #[test]
    fn environment_receipt_path_resolves() {
        let resolution = classify("environments/prod/receipts/rcpt-123");
        assert_eq!(
            resolution,
            SearchResolution::Resolved(SearchTarget::EnvironmentReceipt {
                environment_id: "prod".to_string(),
                receipt_id: "rcpt-123".to_string(),
            })
        );
    }

    #[test]
    fn entity_and_chain_tx_resolve() {
        assert_eq!(
            classify("entity:svc:diewan-demo-agent"),
            SearchResolution::Resolved(SearchTarget::Entity {
                entity: "svc:diewan-demo-agent".to_string()
            })
        );
        assert_eq!(
            classify("tx:ethereum-sepolia/0xabc"),
            SearchResolution::Resolved(SearchTarget::ChainTx {
                chain: Some("ethereum-sepolia".to_string()),
                tx: "0xabc".to_string(),
            })
        );
        assert_eq!(
            classify("tx:0xdef"),
            SearchResolution::Resolved(SearchTarget::ChainTx {
                chain: None,
                tx: "0xdef".to_string()
            })
        );
    }

    #[test]
    fn bare_digest_is_ambiguous_never_a_wrong_object() {
        match classify(&digest()) {
            SearchResolution::Ambiguous { candidates, .. } => {
                assert!(candidates.contains(&AccountabilityObjectKind::Mandate));
                assert!(candidates.contains(&AccountabilityObjectKind::Receipt));
            }
            other => panic!("bare digest resolved to {other:?}"),
        }
    }

    #[test]
    fn empty_unknown_and_bad_prefix_are_no_match() {
        assert!(matches!(classify(""), SearchResolution::NoMatch { .. }));
        assert!(matches!(
            classify("just some text"),
            SearchResolution::NoMatch { .. }
        ));
        assert!(matches!(
            classify("banana:xyz"),
            SearchResolution::NoMatch { .. }
        ));
    }
}
