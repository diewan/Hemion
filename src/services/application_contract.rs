//! Fail-closed boundary for runtime artifacts rendered by the wallet.
//!
//! The UI is handed only canonical, versioned `csv-wire` contracts.  Keeping
//! this conversion here makes it impossible for a presentation component to
//! serialize an ad-hoc lifecycle result or treat a local record as authority.

use csv_sdk::contract::ContractArtifact;

/// Validate an SDK artifact and round-trip it through the canonical wire form.
///
/// The byte equality check makes the codec dependency explicit: both routes
/// must produce the same canonical CBOR representation before the wallet may
/// render the artifact.
pub fn canonical_artifact<T>(artifact: &T) -> Result<T, String>
where
    T: ContractArtifact,
{
    artifact
        .validate()
        .map_err(|error| format!("invalid runtime contract: {error}"))?;

    let wire_bytes = csv_sdk::canonical::app::encode(artifact)
        .map_err(|error| format!("could not encode runtime contract: {error}"))?;
    let codec_bytes = csv_sdk::canonical::to_canonical_cbor(artifact)
        .map_err(|error| format!("could not canonically encode runtime contract: {error}"))?;
    if wire_bytes != codec_bytes {
        return Err("runtime contract has inconsistent canonical encoding".to_string());
    }

    csv_sdk::canonical::app::decode(&wire_bytes)
        .map_err(|error| format!("could not decode runtime contract: {error}"))
}

#[cfg(test)]
mod tests {
    use super::canonical_artifact;
    use csv_sdk::canonical::SanadIdWire;
    use csv_sdk::contract::{
        NextAction, ReceiptBody, SendBody, TransferEvent, TransferMode, TransferPhase,
        TransferReceipt,
    };

    #[test]
    fn rejects_an_incomplete_runtime_artifact() {
        let event = TransferEvent::new(
            TransferMode::Materialize,
            "transfer".to_string(),
            None,
            SanadIdWire {
                bytes: hex::encode([0x11u8; 32]),
            },
            "bitcoin".to_string(),
            TransferPhase::AwaitingFinality {
                evidence: csv_sdk::contract::FinalityEvidence::ObservedTip {
                    confirming_block_height: 100,
                    observed_tip_height: 100,
                    confirmations: 0,
                    required_confirmations: 0,
                },
            },
            vec![NextAction::Resume],
            0,
        );

        assert!(canonical_artifact(&event).is_err());
    }

    #[test]
    fn rejects_a_fabricated_receipt_with_an_invalid_transition() {
        let receipt = TransferReceipt::new(
            ReceiptBody::Send(SendBody {
                transfer_id: "fabricated".to_string(),
                sanad_id: SanadIdWire {
                    bytes: hex::encode([0x11u8; 32]),
                },
                source_chain: "bitcoin".to_string(),
                source_seal: csv_sdk::canonical::SealPointWire {
                    id: hex::encode([0x22u8; 32]),
                    nonce: None,
                    version: None,
                },
                destination_seal: csv_sdk::canonical::SealPointWire {
                    id: hex::encode([0x33u8; 32]),
                    nonce: None,
                    version: None,
                },
                invoice_id: vec![0x44; 32],
                consignment_digest: vec![0x55; 32],
            }),
            // Resume is a materialize action and cannot be attached to a send
            // receipt by a wallet that merely wants to show a completion.
            vec![NextAction::Resume],
            1_700_000_000,
        );

        assert!(canonical_artifact(&receipt).is_err());
    }
}
