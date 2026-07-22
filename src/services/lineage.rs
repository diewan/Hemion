//! Accountability lineage graph (HEM-04).
//!
//! A directed graph over accountability nodes that a user can walk in both
//! directions — mandate → action → receipt → dispute → verdict → anchor, and
//! back. The model is pure and framework-free so the page can render either a
//! walkable graph or an accessible mirror table from the same data, and so the
//! traversal is unit-testable.
//!
//! Two honesty rules are encoded here:
//! - A **gap** is a first-class node type. A withheld or missing branch is shown
//!   as an explicit gap, never silently omitted.
//! - Edges carry no weight and the model exposes no centrality: graph shape is
//!   structure, not truth. Nothing in this module ranks a node as more valid.

use std::collections::BTreeSet;

/// The type of a lineage node. Beyond the accountability object kinds this adds
/// the evidence-node classes (`claim`/`observation`/`attestation`/`gap`) the
/// filters expose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineageNodeType {
    /// A pre-action mandate.
    Mandate,
    /// An execution action/attempt.
    Action,
    /// An execution receipt.
    Receipt,
    /// A dispute bundle.
    Dispute,
    /// An assurance result / verdict.
    Assurance,
    /// A chain anchor.
    Anchor,
    /// A disclosed claim evidence node.
    Claim,
    /// An observation-plane evidence node.
    Observation,
    /// An attestation evidence node.
    Attestation,
    /// An explicit gap: a withheld or missing branch. Never omitted silently.
    Gap,
}

impl LineageNodeType {
    /// A stable, lowercase label used for filters and the table view.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mandate => "mandate",
            Self::Action => "action",
            Self::Receipt => "receipt",
            Self::Dispute => "dispute",
            Self::Assurance => "assurance",
            Self::Anchor => "anchor",
            Self::Claim => "claim",
            Self::Observation => "observation",
            Self::Attestation => "attestation",
            Self::Gap => "gap",
        }
    }
}

/// A direction to walk the lineage graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Toward the objects this node references (upstream: e.g. receipt → mandate).
    Upstream,
    /// Toward the objects that reference this node (downstream: e.g. mandate →
    /// receipt).
    Downstream,
}

/// One node in the lineage graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageNode {
    /// Stable node id.
    pub id: String,
    /// The node type.
    pub node_type: LineageNodeType,
    /// A short human label.
    pub title: String,
}

/// A directed edge from `from` to `to`, meaning `from` references `to`
/// (upstream). Carries a label, never a weight.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineageEdge {
    /// The referencing node id.
    pub from: String,
    /// The referenced node id.
    pub to: String,
    /// The relationship label.
    pub label: String,
}

/// A directed accountability lineage graph.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct LineageGraph {
    /// The nodes, in insertion order.
    pub nodes: Vec<LineageNode>,
    /// The directed (upstream) edges.
    pub edges: Vec<LineageEdge>,
}

impl LineageGraph {
    /// An empty graph.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a node.
    pub fn add_node(&mut self, id: &str, node_type: LineageNodeType, title: &str) {
        self.nodes.push(LineageNode {
            id: id.to_string(),
            node_type,
            title: title.to_string(),
        });
    }

    /// Adds a directed edge `from` references `to`.
    pub fn add_edge(&mut self, from: &str, to: &str, label: &str) {
        self.edges.push(LineageEdge {
            from: from.to_string(),
            to: to.to_string(),
            label: label.to_string(),
        });
    }

    /// Looks up a node by id.
    #[must_use]
    pub fn node(&self, id: &str) -> Option<&LineageNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// The neighbours of `id` in the given direction. Upstream returns the nodes
    /// `id` references; downstream returns the nodes that reference `id`.
    #[must_use]
    pub fn neighbors(&self, id: &str, direction: Direction) -> Vec<&LineageNode> {
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for edge in &self.edges {
            let neighbor_id = match direction {
                Direction::Upstream if edge.from == id => &edge.to,
                Direction::Downstream if edge.to == id => &edge.from,
                _ => continue,
            };
            if seen.insert(neighbor_id.clone())
                && let Some(node) = self.node(neighbor_id)
            {
                out.push(node);
            }
        }
        out
    }

    /// The nodes whose type is in `types`, preserving insertion order. An empty
    /// filter returns every node.
    #[must_use]
    pub fn filter_by_type(&self, types: &[LineageNodeType]) -> Vec<&LineageNode> {
        self.nodes
            .iter()
            .filter(|node| types.is_empty() || types.contains(&node.node_type))
            .collect()
    }
}

/// Builds the demo lineage for the Scenario-A trace: a full
/// mandate → action → receipt → dispute → assurance chain with an anchor gap and
/// an observation node, so the graph and its filters have every node class.
#[must_use]
pub fn demo_lineage() -> LineageGraph {
    use LineageNodeType::{
        Action, Anchor, Assurance, Dispute, Gap, Mandate, Observation, Receipt,
    };
    let mut graph = LineageGraph::new();
    graph.add_node("mandate:m1", Mandate, "Deployment mandate");
    graph.add_node("action:a1", Action, "Execution attempt");
    graph.add_node("receipt:r1", Receipt, "Deployment receipt");
    graph.add_node("observation:o1", Observation, "Tuppira observation");
    graph.add_node("dispute:d1", Dispute, "Dispute bundle");
    graph.add_node("assurance:v1", Assurance, "Independent verdict");
    graph.add_node("anchor:gap", Gap, "Anchor not present (ANCHOR-01)");

    // Upstream edges (from references to).
    graph.add_edge("action:a1", "mandate:m1", "under mandate");
    graph.add_edge("receipt:r1", "action:a1", "records");
    graph.add_edge("receipt:r1", "mandate:m1", "under mandate");
    graph.add_edge("observation:o1", "receipt:r1", "observes");
    graph.add_edge("dispute:d1", "receipt:r1", "about receipt");
    graph.add_edge("dispute:d1", "mandate:m1", "about mandate");
    graph.add_edge("assurance:v1", "dispute:d1", "assesses");
    graph.add_edge("mandate:m1", "anchor:gap", "corroboration absent");
    graph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn demo_lineage_walks_both_directions_through_the_full_chain() {
        let graph = demo_lineage();
        // Downstream from the mandate reaches the action and receipt.
        let downstream: Vec<&str> = graph
            .neighbors("mandate:m1", Direction::Downstream)
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        assert!(downstream.contains(&"action:a1"));
        assert!(downstream.contains(&"receipt:r1"));

        // Upstream from the verdict reaches the dispute, then the receipt/mandate.
        let up_verdict = graph.neighbors("assurance:v1", Direction::Upstream);
        assert_eq!(up_verdict.len(), 1);
        assert_eq!(up_verdict[0].id, "dispute:d1");
        let up_dispute: Vec<&str> = graph
            .neighbors("dispute:d1", Direction::Upstream)
            .iter()
            .map(|n| n.id.as_str())
            .collect();
        assert!(up_dispute.contains(&"receipt:r1"));
        assert!(up_dispute.contains(&"mandate:m1"));
    }

    #[test]
    fn gap_is_a_first_class_node() {
        let graph = demo_lineage();
        let gaps = graph.filter_by_type(&[LineageNodeType::Gap]);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].id, "anchor:gap");
    }

    #[test]
    fn filter_by_type_selects_only_requested_classes() {
        let graph = demo_lineage();
        let disputes = graph.filter_by_type(&[LineageNodeType::Dispute, LineageNodeType::Anchor]);
        // Only the dispute matches; the anchor slot is a gap, not an Anchor node.
        assert_eq!(disputes.len(), 1);
        assert_eq!(disputes[0].node_type, LineageNodeType::Dispute);
        // Empty filter returns everything.
        assert_eq!(graph.filter_by_type(&[]).len(), graph.nodes.len());
    }

    #[test]
    fn unknown_node_has_no_neighbors() {
        let graph = demo_lineage();
        assert!(graph.neighbors("nope", Direction::Upstream).is_empty());
        assert!(graph.node("nope").is_none());
    }
}
