//! Dual-lane finality for the trace view (HEM-02).
//!
//! Every mandate/receipt trace shows two finality lanes side by side, making the
//! "DB as a scalable buffer, chain as the slow-but-final settlement" model
//! visible:
//!
//! - **Buffered** — the observation/DB plane. Immediate, and either present or
//!   absent for a given object. Presence is *recorded elsewhere*, not a verdict.
//! - **Anchored** — the chain. `none` / `pending` / `final`, where the real
//!   pending→final transition is driven by chain reads delivered in **ANCHOR-01**.
//!   Until that lands the anchored lane renders an explicit *unavailable* state.
//!
//! The safety-critical invariant lives in [`AnchoredFinality::from_chain_read`]:
//! `Final` is reachable only when the observed confirmation depth meets the
//! required reorg-safe depth. A pending or unknown read is **never** rendered as
//! final — pending is preserved, absence is not settlement.

use dioxus::prelude::*;

/// The buffered (observation-plane) lane state for one object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferedFinality {
    /// The observation plane holds a record for this object (immediate, buffered).
    Present,
    /// No buffered observation is loaded. Absence is not non-occurrence.
    Absent,
}

/// The anchored (chain) lane state for one object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchoredFinality {
    /// The object has no chain anchor. Anchoring is optional corroborating
    /// evidence; `none` is a limitation, never a failure.
    NotAnchored,
    /// An anchor exists but has not reached reorg-safe finality. Carries the
    /// observed and required confirmation depths so the gap is visible.
    Pending {
        /// Confirmations observed so far from chain reads.
        observed_depth: u64,
        /// Reorg-safe depth required before the anchor may be treated as final.
        required_depth: u64,
    },
    /// The anchor has reached reorg-safe finality from real chain reads.
    Final {
        /// The confirmation depth at which finality was reached.
        confirmed_depth: u64,
    },
    /// The anchored lane cannot be read yet. Carries the reason and the ticket
    /// (`depends_on`) that unblocks a real chain finality source.
    Unavailable {
        /// Why the anchored lane cannot be read.
        reason: String,
        /// The ticket whose completion wires the real path (`ANCHOR-01`).
        depends_on: &'static str,
    },
}

impl AnchoredFinality {
    /// Classifies a chain read into `Pending` or `Final`.
    ///
    /// This is the single place finality is decided. `Final` requires a positive
    /// `required_depth` and an observed depth that meets it; every other read —
    /// including an insufficient depth or a zero requirement — stays `Pending`.
    /// There is no path from a chain read to `Final` that skips the depth gate,
    /// so a probabilistic or shallow anchor can never be shown as settled.
    #[must_use]
    pub fn from_chain_read(observed_depth: u64, required_depth: u64) -> Self {
        if required_depth > 0 && observed_depth >= required_depth {
            Self::Final {
                confirmed_depth: observed_depth,
            }
        } else {
            Self::Pending {
                observed_depth,
                required_depth,
            }
        }
    }

    /// The explicit unavailable state used until ANCHOR-01 wires a real chain
    /// finality source.
    #[must_use]
    pub fn unavailable() -> Self {
        Self::Unavailable {
            reason: "No on-chain finality source is wired yet.".to_string(),
            depends_on: crate::services::anchoring::ANCHOR_BACKING_TICKET,
        }
    }

    /// Whether this lane is settled (reorg-safe final). Only [`Self::Final`] is.
    #[must_use]
    pub const fn is_final(&self) -> bool {
        matches!(self, Self::Final { .. })
    }

    /// A short lane label for display.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotAnchored => "none",
            Self::Pending { .. } => "pending",
            Self::Final { .. } => "final",
            Self::Unavailable { .. } => "unavailable",
        }
    }
}

/// The two finality lanes for one traced object.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinalityLanes {
    /// The immediate buffered/observation-plane lane.
    pub buffered: BufferedFinality,
    /// The slow-but-final chain lane.
    pub anchored: AnchoredFinality,
}

impl FinalityLanes {
    /// Builds the lanes for one object from whether a buffered observation is
    /// present and the anchored-lane state.
    #[must_use]
    pub fn new(observation_present: bool, anchored: AnchoredFinality) -> Self {
        Self {
            buffered: if observation_present {
                BufferedFinality::Present
            } else {
                BufferedFinality::Absent
            },
            anchored,
        }
    }

    /// A buffered-only object: present in the DB plane, no chain anchor.
    #[must_use]
    pub fn buffered_only(observation_present: bool) -> Self {
        Self::new(observation_present, AnchoredFinality::NotAnchored)
    }
}

fn anchored_detail(anchored: &AnchoredFinality) -> String {
    match anchored {
        AnchoredFinality::NotAnchored => {
            "Not anchored — optional corroborating evidence, not a failure.".to_string()
        }
        AnchoredFinality::Pending {
            observed_depth,
            required_depth,
        } => format!(
            "Pending — {observed_depth} of {required_depth} reorg-safe confirmations. \
             Not final."
        ),
        AnchoredFinality::Final { confirmed_depth } => {
            format!("Final at {confirmed_depth} confirmations from chain reads.")
        }
        AnchoredFinality::Unavailable { reason, depends_on } => {
            format!("Unavailable — {reason} (unblocked by {depends_on})")
        }
    }
}

/// Renders the two finality lanes for one traced object.
#[component]
pub fn FinalityLanesView(lanes: FinalityLanes) -> Element {
    let buffered_label = match lanes.buffered {
        BufferedFinality::Present => "present",
        BufferedFinality::Absent => "absent",
    };
    let anchored_label = lanes.anchored.label();
    rsx! {
        div { class: "finality-lanes", role: "group", aria_label: "Finality lanes",
            div { class: "finality-lane finality-lane-buffered", "data-state": "{buffered_label}",
                span { class: "finality-lane-name", "Buffered" }
                span { class: "finality-lane-state", "{buffered_label}" }
                span { class: "finality-lane-detail",
                    match lanes.buffered {
                        BufferedFinality::Present => "Recorded in the observation plane (immediate). Not a verdict.",
                        BufferedFinality::Absent => "No buffered observation. Absence is not non-occurrence.",
                    }
                }
            }
            div { class: "finality-lane finality-lane-anchored", "data-state": "{anchored_label}",
                span { class: "finality-lane-name", "Anchored" }
                span { class: "finality-lane-state", "{anchored_label}" }
                span { class: "finality-lane-detail", "{anchored_detail(&lanes.anchored)}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn buffered_only_object_shows_present_and_none() {
        let lanes = FinalityLanes::buffered_only(true);
        assert_eq!(lanes.buffered, BufferedFinality::Present);
        assert_eq!(lanes.anchored, AnchoredFinality::NotAnchored);
        assert!(!lanes.anchored.is_final());
    }

    #[test]
    fn absent_buffered_is_present_false() {
        let lanes = FinalityLanes::new(false, AnchoredFinality::unavailable());
        assert_eq!(lanes.buffered, BufferedFinality::Absent);
        assert_eq!(lanes.anchored.label(), "unavailable");
    }

    #[test]
    fn chain_read_below_required_depth_stays_pending() {
        // Adversarial: an anchor with fewer confirmations than the reorg-safe
        // depth must never be shown as final.
        let anchored = AnchoredFinality::from_chain_read(3, 12);
        assert_eq!(
            anchored,
            AnchoredFinality::Pending {
                observed_depth: 3,
                required_depth: 12
            }
        );
        assert!(!anchored.is_final());
        assert_eq!(anchored.label(), "pending");
    }

    #[test]
    fn chain_read_meeting_required_depth_is_final() {
        let anchored = AnchoredFinality::from_chain_read(12, 12);
        assert_eq!(anchored, AnchoredFinality::Final { confirmed_depth: 12 });
        assert!(anchored.is_final());
    }

    #[test]
    fn zero_required_depth_never_finalizes() {
        // A degenerate/zero finality requirement must not be treated as instantly
        // final — it stays pending rather than fabricating settlement.
        let anchored = AnchoredFinality::from_chain_read(100, 0);
        assert!(!anchored.is_final());
        assert_eq!(anchored.label(), "pending");
    }

    #[test]
    fn unavailable_names_anchor_backing_ticket() {
        let AnchoredFinality::Unavailable { depends_on, .. } = AnchoredFinality::unavailable() else {
            panic!("expected unavailable");
        };
        assert_eq!(depends_on, "ANCHOR-01");
    }
}
