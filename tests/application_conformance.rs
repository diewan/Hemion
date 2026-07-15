//! Versioned application conformance fixtures (APPS-CONFORMANCE-015).
//!
//! The CLI presents `csv_sdk::contract` directly.  The wallet must render the
//! exact same canonical artifacts, while explorer data remains an explicitly
//! untrusted projection.  These tests deliberately use public application and
//! protocol APIs so they protect the release boundary rather than an internal
//! implementation detail.

#[path = "../src/services/application_contract.rs"]
mod wallet_contract;

use chrono::Utc;
use tuppira_shared::{
    TUPPIRA_EVENT_SCHEMA_VERSION, TuppiraEventDto, TuppiraEventPayload, TuppiraEventType,
    TuppiraFinality, FeedProvenance, IndexerFreshness, IndexerFreshnessStatus, Network,
    ObservedBlock, WALLET_FEED_SCHEMA_VERSION, WalletFeedEnvelope, WalletFeedProjection,
};
use csv_hash::chain_id::ChainId;
use csv_hash::sanad::SanadId;
use csv_sdk::contract::{
    self, FinalityEvidence, NextAction, ReceiptBody, TransferMode, TransferReceipt,
};
use csv_testkit::fixtures::TestProofBundle;
use csv_wallet::format::{
    FormatError, GOLDEN_WALLET_V1, GOLDEN_WALLET_V1_PASSPHRASE, decrypt, encrypt,
    golden_wallet_v1_payload,
};
use csv_wire::{Consignment, Invoice, SanadIdWire, SealDefinition};

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
    let source_seal = csv_hash::seal::SealPoint::new(vec![0x22; 32], Some(1), None)
        .expect("fixture source seal is well formed");
    let destination_seal = csv_hash::seal::SealPoint::new(vec![0x33; 32], Some(2), None)
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
        &csv_hash::seal::SealPoint::new(vec![0x22; 32], Some(1), None).unwrap(),
        &csv_hash::seal::SealPoint::new(vec![0x33; 32], Some(2), None).unwrap(),
        &[0x44; 32],
        b"conformance-consignment",
    )
    .unwrap();
    assert_eq!(send.mode(), TransferMode::Send);

    let pending = csv_sdk::transfers::TransferOutcome::Pending {
        transfer_id: "conformance-materialize-1".to_string(),
        lock_tx_hash: "aa".repeat(32),
        finality: csv_runtime::FinalityObservation {
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
        &csv_runtime::FinalityObservation {
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
fn canonical_invoice_consignment_and_proof_bytes_round_trip() {
    let invoice = Invoice::new(
        SealDefinition::sui(vec![0x55; 32], 7).expect("fixture seal"),
        vec![0x66; 32],
        9,
    )
    .expect("fixture invoice");
    let proof = TestProofBundle::minimal();
    let consignment = Consignment::new(
        invoice.clone(),
        SanadIdWire {
            bytes: hex::encode([0x11; 32]),
        },
        proof.clone(),
    );

    let invoice_bytes = invoice.canonical_cbor().expect("invoice encodes");
    assert_eq!(
        csv_codec::from_canonical_cbor::<Invoice>(&invoice_bytes).expect("invoice decodes"),
        invoice
    );
    let proof_bytes = csv_codec::to_canonical_cbor(&proof).expect("proof encodes");
    assert_eq!(
        csv_codec::from_canonical_cbor::<csv_protocol::proof_taxonomy::ProofBundle>(&proof_bytes)
            .expect("proof decodes"),
        proof
    );
    assert!(
        csv_codec::from_canonical_cbor::<csv_protocol::proof_taxonomy::ProofBundle>(
            &proof_bytes[..proof_bytes.len() / 2]
        )
        .is_err()
    );
    let consignment_bytes = consignment.canonical_cbor().expect("consignment encodes");
    let decoded: Consignment =
        csv_codec::from_canonical_cbor(&consignment_bytes).expect("consignment decodes");
    assert_eq!(decoded.invoice, invoice);
    assert_eq!(decoded.proof_bundle, proof);
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

fn feed(finality: TuppiraFinality, freshness: IndexerFreshnessStatus) -> WalletFeedEnvelope {
    let event = TuppiraEventDto {
        schema_version: TUPPIRA_EVENT_SCHEMA_VERSION,
        chain_id: ChainId::new("bitcoin"),
        network: Network::Testnet,
        contract: "contract".to_string(),
        event_type: TuppiraEventType::TransferSent,
        block_height: 100,
        block_hash: "block-100".to_string(),
        transaction_id: "tx-1".to_string(),
        log_index: 0,
        finality,
        payload: TuppiraEventPayload::TransferSent {
            transfer_id: "feed-transfer".to_string(),
            sanad_id: hex::encode([0x11; 32]),
            destination_chain: ChainId::new("sui"),
            destination_owner: "recipient".to_string(),
        },
    };
    WalletFeedEnvelope {
        schema_version: WALLET_FEED_SCHEMA_VERSION,
        protocol_version: csv_protocol::version::PROTOCOL_VERSION.to_string(),
        observation_id: "observation-1".to_string(),
        sequence: 1,
        chain_id: ChainId::new("bitcoin"),
        network: Network::Testnet,
        observed_block: ObservedBlock {
            height: 100,
            hash: "block-100".to_string(),
        },
        freshness: IndexerFreshness {
            indexed_at: Utc::now(),
            tip: ObservedBlock {
                height: 100,
                hash: "tip-100".to_string(),
            },
            lag_blocks: 0,
            status: freshness,
        },
        finality,
        provenance: FeedProvenance {
            producer: "conformance-indexer".to_string(),
            source_cursor: "cursor-1".to_string(),
            cryptographically_verified: false,
        },
        event,
        reorg_replacement: None,
    }
}

#[test]
fn explorer_staleness_and_reorg_never_become_wallet_authority() {
    let mut stale = feed(TuppiraFinality::Finalized, IndexerFreshnessStatus::Stale);
    // A stale indexer can describe a finality state, but it cannot claim the
    // verifier's authority.  The envelope validator enforces that distinction.
    stale.provenance.cryptographically_verified = true;
    assert!(stale.validate().is_err());

    let mut projection = WalletFeedProjection::default();
    let finalized = feed(TuppiraFinality::Finalized, IndexerFreshnessStatus::Fresh);
    assert!(
        projection
            .apply(finalized)
            .expect("first observation applies")
    );
    assert!(
        !projection
            .apply(feed(
                TuppiraFinality::Finalized,
                IndexerFreshnessStatus::Fresh
            ))
            .expect("replayed observation is ignored")
    );
    let mut regressed = feed(TuppiraFinality::Observed, IndexerFreshnessStatus::Fresh);
    regressed.observation_id = "observation-2".to_string();
    regressed.sequence = 2;
    assert!(
        projection.apply(regressed).is_err(),
        "reorg must be explicit"
    );
}
