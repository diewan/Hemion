//! Disposable, monotonic presentation projections of runtime transfer contracts.
//!
//! The projection deliberately has no transition methods that accept UI state:
//! only a validated `TransferEvent` or `TransferReceipt` may advance it.  It is
//! safe to discard and rebuild after reconnecting to the runtime.

use csv_sdk::contract::{
    ContractArtifact, FinalityEvidence, NextAction, ReceiptBody, TransferEvent, TransferMode,
    TransferPhase, TransferReceipt, VerificationAssuranceWire, VerificationRecord,
};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvidenceView {
    pub summary: String,
    pub provenance: &'static str,
    pub is_final: bool,
}

impl EvidenceView {
    fn from_evidence(evidence: &FinalityEvidence) -> Self {
        match evidence {
            FinalityEvidence::ObservedTip {
                confirming_block_height,
                observed_tip_height,
                confirmations,
                required_confirmations,
            } => Self {
                summary: format!(
                    "{confirmations}/{required_confirmations} confirmations; block {confirming_block_height}, observed tip {observed_tip_height}"
                ),
                provenance: "observed chain tip",
                is_final: evidence.is_final(),
            },
            FinalityEvidence::ChainReported {
                confirming_block_height,
                confirmations,
                required_confirmations,
            } => Self {
                summary: format!(
                    "{confirmations}/{required_confirmations} confirmations; confirming height {confirming_block_height}"
                ),
                provenance: "chain-reported finality",
                is_final: evidence.is_final(),
            },
            FinalityEvidence::JournalRecovered => Self {
                summary: "Recovered from the runtime journal; no fresh chain observation"
                    .to_string(),
                provenance: "runtime journal",
                is_final: false,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifecycleStage {
    pub name: &'static str,
    pub explanation: &'static str,
    rank: u8,
}

impl LifecycleStage {
    fn from_phase(phase: &TransferPhase) -> Self {
        match phase {
            TransferPhase::Admitted => Self {
                name: "Admitted",
                explanation: "The runtime admitted this transfer.",
                rank: 10,
            },
            TransferPhase::SealOwnershipVerified { .. } => Self {
                name: "Destination seal ownership verified",
                explanation: "The runtime verified the recipient-controlled seal.",
                rank: 20,
            },
            TransferPhase::Locked { .. } => Self {
                name: "Source locked",
                explanation: "The source seal was locked on-chain.",
                rank: 30,
            },
            TransferPhase::AwaitingFinality { .. } => Self {
                name: "Awaiting source finality",
                explanation: "The lock exists, but finality has not yet been established.",
                rank: 40,
            },
            TransferPhase::FinalityReached { .. } => Self {
                name: "Source finality reached",
                explanation: "The runtime observed source finality.",
                rank: 50,
            },
            TransferPhase::ProofBuilt { .. } => Self {
                name: "Proof built",
                explanation: "The runtime built an inclusion and finality proof bundle.",
                rank: 60,
            },
            TransferPhase::ProofVerified { .. } => Self {
                name: "Proof verified",
                explanation: "The canonical verifier reported the assurance shown below.",
                rank: 70,
            },
            TransferPhase::SubmittedToDestination { .. } => Self {
                name: "Submitted to destination",
                explanation: "The runtime submitted the verified proof to the destination chain.",
                rank: 80,
            },
            TransferPhase::Settled { .. } => Self {
                name: "Settled",
                explanation: "The destination materialization was confirmed by the runtime.",
                rank: 90,
            },
            TransferPhase::RecoveryRequired { .. } => Self {
                name: "Recovery required",
                explanation: "The runtime journal requires an allowed recovery action.",
                rank: 100,
            },
            TransferPhase::Failed { .. } => Self {
                name: "Failed",
                explanation: "The runtime reported a terminal failure.",
                rank: 100,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferLifecycleView {
    pub transfer_id: Option<String>,
    pub sanad_id: String,
    pub mode: TransferMode,
    pub stage: LifecycleStage,
    pub journal_phase: String,
    pub observed_at: u64,
    pub source_finality: Option<EvidenceView>,
    /// `None` means the runtime contract did not report a destination finality
    /// observation; it is never inferred from a mint hash.
    pub destination_finality: Option<EvidenceView>,
    pub verification_assurance: Option<VerificationAssuranceWire>,
    pub verification_provenance: Option<&'static str>,
    pub failure_reason: Option<String>,
    pub replay_id: Option<String>,
    pub lock_tx_hash: Option<String>,
    pub mint_tx_hash: Option<String>,
    pub proof_hash: Option<String>,
    pub invoice_id: Option<String>,
    pub consignment_digest: Option<String>,
    /// Exact versioned wire artifact from which this projection was built.
    /// Keeping the bytes alongside the disposable projection lets Inspector
    /// export evidence without reconstructing it from UI fields.
    pub artifact_kind: &'static str,
    pub artifact_cbor_hex: Option<String>,
    pub artifact_sha256: Option<String>,
    pub permitted_actions: Vec<NextAction>,
}

impl TransferLifecycleView {
    pub fn from_event(event: &TransferEvent) -> Self {
        let (artifact_cbor_hex, artifact_sha256) = encoded_artifact(event);
        let mut view = Self {
            transfer_id: Some(event.transfer_id.clone()),
            sanad_id: event.sanad_id.bytes.clone(),
            mode: event.mode,
            stage: LifecycleStage::from_phase(&event.phase),
            journal_phase: format!("{:?}", event.phase),
            observed_at: event.observed_at,
            source_finality: None,
            destination_finality: None,
            verification_assurance: None,
            verification_provenance: None,
            failure_reason: None,
            replay_id: event.replay_id.as_ref().map(|id| id.bytes.clone()),
            lock_tx_hash: None,
            mint_tx_hash: None,
            proof_hash: None,
            invoice_id: None,
            consignment_digest: None,
            artifact_kind: "runtime event",
            artifact_cbor_hex,
            artifact_sha256,
            permitted_actions: event.next_actions.clone(),
        };
        match &event.phase {
            TransferPhase::Locked { lock_tx_hash } => {
                view.lock_tx_hash = Some(lock_tx_hash.clone())
            }
            TransferPhase::AwaitingFinality { evidence }
            | TransferPhase::FinalityReached { evidence } => {
                view.source_finality = Some(EvidenceView::from_evidence(evidence))
            }
            TransferPhase::ProofBuilt { proof_hash } => {
                view.proof_hash = Some(hex::encode(proof_hash))
            }
            TransferPhase::ProofVerified { assurance } => {
                view.verification_assurance = Some(*assurance);
                view.verification_provenance = Some("canonical verifier");
            }
            TransferPhase::Settled { mint_tx_hash } => {
                view.mint_tx_hash = Some(mint_tx_hash.clone())
            }
            TransferPhase::RecoveryRequired { reason } => {
                view.failure_reason = Some(reason.clone())
            }
            TransferPhase::Failed { code, message } => {
                view.failure_reason = Some(format!("{code}: {message}"))
            }
            _ => {}
        }
        view
    }

    pub fn from_receipt(receipt: &TransferReceipt) -> Self {
        let (artifact_cbor_hex, artifact_sha256) = encoded_artifact(receipt);
        match &receipt.body {
            ReceiptBody::Materialize(body) => {
                let (assurance, provenance) = match body.verification {
                    VerificationRecord::Verified { assurance } => {
                        (Some(assurance), Some("canonical verifier"))
                    }
                    VerificationRecord::JournalRecorded => (
                        None,
                        Some("runtime journal; not re-verified on this execution"),
                    ),
                    VerificationRecord::NotYetVerified => (None, Some("not yet verified")),
                };
                Self {
                    transfer_id: Some(body.transfer_id.clone()),
                    sanad_id: body.sanad_id.bytes.clone(),
                    mode: receipt.mode(),
                    stage: LifecycleStage {
                        name: "Settled",
                        explanation: "The destination materialization was confirmed by the runtime.",
                        rank: 90,
                    },
                    journal_phase: "Settled (receipt)".to_string(),
                    observed_at: receipt.emitted_at,
                    source_finality: Some(EvidenceView::from_evidence(&body.finality)),
                    destination_finality: None,
                    verification_assurance: assurance,
                    verification_provenance: provenance,
                    failure_reason: None,
                    replay_id: Some(body.replay_id.bytes.clone()),
                    lock_tx_hash: Some(body.lock_tx_hash.clone()),
                    mint_tx_hash: (!body.mint_tx_hash.is_empty())
                        .then(|| body.mint_tx_hash.clone()),
                    proof_hash: None,
                    invoice_id: None,
                    consignment_digest: None,
                    artifact_kind: "runtime receipt",
                    artifact_cbor_hex: artifact_cbor_hex.clone(),
                    artifact_sha256: artifact_sha256.clone(),
                    permitted_actions: receipt.next_actions.clone(),
                }
            }
            ReceiptBody::Send(body) => Self {
                transfer_id: Some(body.transfer_id.clone()),
                sanad_id: body.sanad_id.bytes.clone(),
                mode: receipt.mode(),
                stage: LifecycleStage {
                    name: "Consignment emitted",
                    explanation: "The interactive send completed off-chain; no destination-chain submission occurred.",
                    rank: 90,
                },
                journal_phase: "Send receipt".to_string(),
                observed_at: receipt.emitted_at,
                source_finality: None,
                destination_finality: None,
                verification_assurance: None,
                verification_provenance: None,
                failure_reason: None,
                replay_id: None,
                lock_tx_hash: None,
                mint_tx_hash: None,
                proof_hash: None,
                invoice_id: Some(hex::encode(&body.invoice_id)),
                consignment_digest: Some(hex::encode(&body.consignment_digest)),
                artifact_kind: "runtime receipt",
                artifact_cbor_hex: artifact_cbor_hex.clone(),
                artifact_sha256: artifact_sha256.clone(),
                permitted_actions: receipt.next_actions.clone(),
            },
            ReceiptBody::Invoice(body) => Self {
                transfer_id: receipt.transfer_id().map(ToString::to_string),
                sanad_id: String::new(),
                mode: receipt.mode(),
                stage: LifecycleStage {
                    name: "Receipt issued",
                    explanation: "The runtime returned the mode-specific receipt.",
                    rank: 90,
                },
                journal_phase: format!("{:?} receipt", receipt.mode()),
                observed_at: receipt.emitted_at,
                source_finality: None,
                destination_finality: None,
                verification_assurance: None,
                verification_provenance: None,
                failure_reason: None,
                replay_id: None,
                lock_tx_hash: None,
                mint_tx_hash: None,
                proof_hash: None,
                invoice_id: Some(hex::encode(&body.invoice_id)),
                consignment_digest: None,
                artifact_kind: "runtime receipt",
                artifact_cbor_hex: artifact_cbor_hex.clone(),
                artifact_sha256: artifact_sha256.clone(),
                permitted_actions: receipt.next_actions.clone(),
            },
            ReceiptBody::Accept(body) => Self {
                transfer_id: None,
                sanad_id: body.sanad_id.bytes.clone(),
                mode: receipt.mode(),
                stage: LifecycleStage {
                    name: "Consignment accepted",
                    explanation: "The recipient-side runtime validation recorded ownership without a destination-chain submission.",
                    rank: 90,
                },
                journal_phase: "Accept receipt".to_string(),
                observed_at: receipt.emitted_at,
                source_finality: Some(EvidenceView::from_evidence(&body.finality)),
                destination_finality: None,
                verification_assurance: Some(body.assurance),
                verification_provenance: Some("canonical verifier"),
                failure_reason: None,
                replay_id: None,
                lock_tx_hash: None,
                mint_tx_hash: None,
                proof_hash: None,
                invoice_id: None,
                consignment_digest: None,
                artifact_kind: "runtime receipt",
                artifact_cbor_hex,
                artifact_sha256,
                permitted_actions: receipt.next_actions.clone(),
            },
        }
    }

    /// Apply a runtime event without allowing duplicate, stale, or regressive
    /// events to make the user-visible lifecycle move backwards.
    pub fn apply_event(&mut self, event: &TransferEvent) {
        let incoming = Self::from_event(event);
        if self.transfer_id != incoming.transfer_id
            || incoming.observed_at < self.observed_at
            || incoming.stage.rank < self.stage.rank
        {
            return;
        }
        if incoming.observed_at == self.observed_at
            && incoming.stage.rank == self.stage.rank
            && incoming.journal_phase == self.journal_phase
        {
            return;
        }
        *self = incoming;
    }

    pub fn allows_resume(&self) -> bool {
        self.permitted_actions.contains(&NextAction::Resume)
    }
    pub fn allows_retry(&self) -> bool {
        self.permitted_actions.contains(&NextAction::Retry)
    }
}

fn encoded_artifact<T: ContractArtifact>(artifact: &T) -> (Option<String>, Option<String>) {
    match csv_sdk::canonical::app::encode(artifact) {
        Ok(bytes) => {
            let digest = hex::encode(Sha256::digest(&bytes));
            (Some(hex::encode(bytes)), Some(digest))
        }
        Err(_) => (None, None),
    }
}

/// Backwards-compatible compact projection used by non-lifecycle pages.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferViewModel {
    pub phase: String,
    pub permitted_actions: Vec<NextAction>,
}
impl From<&TransferReceipt> for TransferViewModel {
    fn from(receipt: &TransferReceipt) -> Self {
        Self {
            phase: "settled".to_string(),
            permitted_actions: receipt.next_actions.clone(),
        }
    }
}
impl From<&TransferEvent> for TransferViewModel {
    fn from(event: &TransferEvent) -> Self {
        Self {
            phase: format!("{:?}", event.phase),
            permitted_actions: event.next_actions.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv_sdk::canonical::SanadIdWire;
    use csv_sdk::contract::FinalityEvidence;

    fn event(phase: TransferPhase, observed_at: u64) -> TransferEvent {
        TransferEvent::new(
            TransferMode::Materialize,
            "transfer-1".to_string(),
            None,
            SanadIdWire {
                bytes: hex::encode([1u8; 32]),
            },
            "bitcoin".to_string(),
            phase,
            vec![NextAction::Status],
            observed_at,
        )
    }
    #[test]
    fn ignores_out_of_order_duplicate_and_stale_events() {
        let mut view = TransferLifecycleView::from_event(&event(
            TransferPhase::ProofBuilt {
                proof_hash: vec![3; 32],
            },
            30,
        ));
        view.apply_event(&event(
            TransferPhase::Locked {
                lock_tx_hash: "old".to_string(),
            },
            31,
        ));
        view.apply_event(&event(
            TransferPhase::ProofBuilt {
                proof_hash: vec![3; 32],
            },
            30,
        ));
        view.apply_event(&event(
            TransferPhase::FinalityReached {
                evidence: FinalityEvidence::ObservedTip {
                    confirming_block_height: 10,
                    observed_tip_height: 12,
                    confirmations: 2,
                    required_confirmations: 2,
                },
            },
            29,
        ));
        assert_eq!(view.stage.name, "Proof built");
        assert_eq!(view.observed_at, 30);
    }
    #[test]
    fn exposes_only_runtime_permitted_recovery_actions() {
        let view = TransferLifecycleView::from_event(&event(
            TransferPhase::RecoveryRequired {
                reason: "journal interrupted".to_string(),
            },
            1,
        ));
        assert!(!view.allows_resume());
        assert!(!view.allows_retry());
    }

    #[test]
    fn inspector_export_preserves_the_exact_versioned_wire_artifact() {
        let source = event(TransferPhase::Admitted, 7);
        let expected = csv_sdk::canonical::app::encode(&source).expect("event wire artifact");
        let view = TransferLifecycleView::from_event(&source);

        assert_eq!(
            hex::decode(view.artifact_cbor_hex.expect("artifact bytes")).unwrap(),
            expected
        );
        assert_eq!(
            view.artifact_sha256.expect("artifact digest"),
            hex::encode(Sha256::digest(&expected))
        );
    }
}
