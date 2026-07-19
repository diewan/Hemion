//! Application contract for submitting a transfer to the CSV runtime.
//!
//! This is the sole wallet boundary allowed to invoke the SDK transfer use
//! case. Presentation components receive only validated application artifacts;
//! they never receive adapters, coordinators, or mutable transfer authority.

use csv_sdk::contract::{TransferEvent, TransferReceipt};
use csv_sdk::protocol::hash::{ChainId, SanadId};

use super::application_contract::canonical_artifact;

/// The complete, user-reviewed transfer request passed to the runtime use case.
#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub sanad_id: SanadId,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub destination_address: String,
}

/// A request to advance an existing materialize transfer.  These fields are
/// inputs to the runtime's journal lookup, never wallet-local transition data.
#[derive(Debug, Clone)]
pub struct ResumeRequest {
    pub transfer_id: String,
    pub sanad_id: SanadId,
    pub source_chain: ChainId,
    pub destination_chain: ChainId,
    pub destination_address: Option<String>,
}

/// A runtime result represented exclusively with validated application contracts.
#[derive(Debug, Clone)]
pub enum TransferSubmission {
    Settled(TransferReceipt),
    AwaitingFinality(TransferEvent),
}

/// Submit a transfer through the SDK/runtime use case.
///
/// No local wallet state is changed here. The runtime remains authoritative for
/// leases, replay protection, recovery, and transfer phase transitions.
#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_transfer(request: TransferRequest) -> Result<TransferSubmission, String> {
    use csv_sdk::CsvClient;

    let client = CsvClient::builder()
        .with_chain(request.source_chain.clone())
        .with_chain(request.destination_chain.clone())
        .with_runtime_coordinator()
        .build()
        .await
        .map_err(|error| format!("failed to create CSV runtime client: {error}"))?;

    let outcome = client
        .transfers()
        .cross_chain(request.sanad_id.clone(), request.destination_chain.clone())
        .from_chain(request.source_chain.clone())
        .to_address(request.destination_address)
        .execute_outcome()
        .await
        .map_err(|error| format!("runtime transfer submission failed: {error}"))?;

    let event =
        csv_sdk::contract::materialize_event(&outcome, &request.sanad_id, &request.source_chain)
            .map_err(|error| format!("runtime event violates the application contract: {error}"))?;
    let event = canonical_artifact(&event)?;

    match outcome {
        csv_sdk::transfers::TransferOutcome::Completed(receipt) => {
            let receipt = csv_sdk::contract::materialize_sdk_receipt(
                &receipt,
                &request.sanad_id,
                &request.source_chain,
                &request.destination_chain,
            )
            .map_err(|error| {
                format!("runtime receipt violates the application contract: {error}")
            })?;
            Ok(TransferSubmission::Settled(canonical_artifact(&receipt)?))
        }
        csv_sdk::transfers::TransferOutcome::Pending { .. } => {
            Ok(TransferSubmission::AwaitingFinality(event))
        }
    }
}

/// Advance an interrupted transfer from its runtime-journalled phase.
///
/// A retry is not a locally inferred state change: the SDK coordinator decides
/// whether resumption is legal and returns canonical evidence or a receipt.
#[cfg(not(target_arch = "wasm32"))]
pub async fn resume_transfer(request: ResumeRequest) -> Result<TransferSubmission, String> {
    use csv_sdk::CsvClient;

    let client = CsvClient::builder()
        .with_chain(request.source_chain.clone())
        .with_chain(request.destination_chain.clone())
        .with_runtime_coordinator()
        .build()
        .await
        .map_err(|error| format!("failed to open CSV runtime journal: {error}"))?;

    let outcome = client
        .transfers()
        .resume(
            &request.transfer_id,
            request.sanad_id.clone(),
            request.source_chain.clone(),
            request.destination_chain.clone(),
            request.destination_address.clone(),
        )
        .await
        .map_err(|error| format!("runtime transfer resume failed: {error}"))?;

    let event =
        csv_sdk::contract::materialize_event(&outcome, &request.sanad_id, &request.source_chain)
            .map_err(|error| format!("runtime event violates the application contract: {error}"))?;
    let event = canonical_artifact(&event)?;

    match outcome {
        csv_sdk::transfers::TransferOutcome::Completed(receipt) => {
            let receipt = csv_sdk::contract::materialize_sdk_receipt(
                &receipt,
                &request.sanad_id,
                &request.source_chain,
                &request.destination_chain,
            )
            .map_err(|error| {
                format!("runtime receipt violates the application contract: {error}")
            })?;
            Ok(TransferSubmission::Settled(canonical_artifact(&receipt)?))
        }
        csv_sdk::transfers::TransferOutcome::Pending { .. } => {
            Ok(TransferSubmission::AwaitingFinality(event))
        }
    }
}

/// Browser builds require a remote runtime host. No local adapter fallback is
/// permitted because it would create a separate authority and runtime journal.
#[cfg(target_arch = "wasm32")]
pub async fn submit_transfer(_request: TransferRequest) -> Result<TransferSubmission, String> {
    Err("transfer submission is unavailable: configure a remote CSV runtime host".to_string())
}

#[cfg(target_arch = "wasm32")]
pub async fn resume_transfer(_request: ResumeRequest) -> Result<TransferSubmission, String> {
    Err("transfer resume is unavailable: configure a remote CSV runtime host".to_string())
}
