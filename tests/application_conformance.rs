//! Versioned application conformance fixtures (APPS-CONFORMANCE-015).
//!
//! The CLI presents `csv_sdk::contract` directly.  The wallet must render the
//! exact same canonical artifacts, while explorer data remains an explicitly
//! untrusted projection.  These tests deliberately use public application and
//! protocol APIs so they protect the release boundary rather than an internal
//! implementation detail.

#[path = "../src/services/application_contract.rs"]
mod wallet_contract;

use csv_sdk::canonical::SanadIdWire;
use csv_sdk::contract::{
    self, FinalityEvidence, NextAction, ReceiptBody, TransferMode, TransferReceipt,
};
use csv_sdk::protocol::hash::chain_id::ChainId;
use csv_sdk::protocol::hash::sanad::SanadId;
use csv_sdk::wallet_format::format::{
    FormatError, GOLDEN_WALLET_V1, GOLDEN_WALLET_V1_PASSPHRASE, decrypt, encrypt,
    golden_wallet_v1_payload,
};

/// Bump only when every supported application deliberately changes its
/// externally visible fixture set.
const APPLICATION_CONFORMANCE_VERSION: u16 = 1;

fn sanad_id() -> SanadId {
    SanadId::new([0x11; 32])
}

fn source() -> ChainId {
    ChainId::new("bitcoin")
}

#[test]
fn cli_and_wallet_accept_identical_versioned_send_outcomes() {
    assert_eq!(APPLICATION_CONFORMANCE_VERSION, 1);
    let source_seal = csv_sdk::protocol::hash::seal::SealPoint::new(vec![0x22; 32], Some(1), None)
        .expect("fixture source seal is well formed");
    let destination_seal =
        csv_sdk::protocol::hash::seal::SealPoint::new(vec![0x33; 32], Some(2), None)
            .expect("fixture destination seal is well formed");

    // This is the CLI-facing constructor.  The wallet must render precisely
    // its canonical bytes, rather than assembling a UI-local completion.
    let cli = contract::send_receipt(
        "conformance-send-1",
        &sanad_id(),
        &source(),
        &source_seal,
        &destination_seal,
        &[0x44; 32],
        b"conformance-consignment",
    )
    .expect("reference CLI contract fixture");
    let cli_bytes = contract::encode(&cli).expect("CLI outcome encodes");
    let wallet: TransferReceipt = wallet_contract::canonical_artifact(&cli)
        .expect("wallet accepts the reference CLI outcome");

    assert_eq!(
        contract::encode(&wallet).expect("wallet outcome encodes"),
        cli_bytes
    );
    assert_eq!(wallet.mode(), TransferMode::Send);
    assert!(matches!(wallet.body, ReceiptBody::Send(_)));
    assert!(!wallet.next_actions.contains(&NextAction::Resume));
    assert!(!wallet.next_actions.contains(&NextAction::Retry));
}

#[test]
fn interactive_send_and_materialization_remain_distinct_lifecycle_fixtures() {
    let send = contract::send_receipt(
        "conformance-send-2",
        &sanad_id(),
        &source(),
        &csv_sdk::protocol::hash::seal::SealPoint::new(vec![0x22; 32], Some(1), None).unwrap(),
        &csv_sdk::protocol::hash::seal::SealPoint::new(vec![0x33; 32], Some(2), None).unwrap(),
        &[0x44; 32],
        b"conformance-consignment",
    )
    .unwrap();
    assert_eq!(send.mode(), TransferMode::Send);

    let pending = csv_sdk::transfers::TransferOutcome::Pending {
        transfer_id: "conformance-materialize-1".to_string(),
        lock_tx_hash: "aa".repeat(32),
        finality: csv_sdk::application::FinalityObservation {
            confirming_block_height: 100,
            observed_tip_height: Some(102),
            confirmations: 2,
            required_confirmations: 6,
        },
    };
    let materialize = contract::materialize_event(&pending, &sanad_id(), &source())
        .expect("materialize lifecycle fixture");
    assert_eq!(materialize.mode, TransferMode::Materialize);
    assert!(materialize.next_actions.contains(&NextAction::Resume));
    assert!(matches!(
        materialize.phase,
        csv_sdk::contract::TransferPhase::AwaitingFinality { .. }
    ));

    let plan = contract::awaiting_finality_plan(
        "conformance-materialize-1",
        &csv_sdk::application::FinalityObservation {
            confirming_block_height: 100,
            observed_tip_height: Some(102),
            confirmations: 2,
            required_confirmations: 6,
        },
    )
    .expect("runtime recovery plan");
    assert!(plan.permitted_actions.contains(&NextAction::Resume));
    assert!(!plan.permitted_actions.contains(&NextAction::Retry));
}

#[test]
fn wallet_file_golden_round_trips_and_fails_closed() {
    let payload = decrypt(GOLDEN_WALLET_V1, GOLDEN_WALLET_V1_PASSPHRASE)
        .expect("shared CLI/wallet golden file imports");
    assert_eq!(payload, golden_wallet_v1_payload());
    let exported = encrypt(&payload, "cross-app-roundtrip")
        .expect("wallet-format exporter uses shared encrypted envelope");
    assert_eq!(
        decrypt(&exported, "cross-app-roundtrip").expect("re-import encrypted file"),
        payload
    );
    assert!(matches!(
        decrypt(GOLDEN_WALLET_V1, "incorrect-passphrase"),
        Err(FormatError::Decryption)
    ));
}

#[test]
fn forged_or_insufficient_finality_contracts_are_rejected_by_wallet() {
    let event = csv_sdk::contract::TransferEvent::new(
        TransferMode::Materialize,
        "forged-finality".to_string(),
        None,
        SanadIdWire {
            bytes: hex::encode([0x11; 32]),
        },
        "bitcoin".to_string(),
        csv_sdk::contract::TransferPhase::AwaitingFinality {
            evidence: FinalityEvidence::ObservedTip {
                confirming_block_height: 100,
                observed_tip_height: 101,
                confirmations: 6,
                required_confirmations: 6,
            },
        },
        vec![NextAction::Resume],
        1,
    );
    assert!(wallet_contract::canonical_artifact(&event).is_err());
}
