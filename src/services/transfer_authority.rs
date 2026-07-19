//! Application contract for submitting a transfer to the CSV runtime.
//!
//! This is the sole wallet boundary allowed to invoke an SDK application host.
//! No host transport is configured in this build, so mutation fails closed;
//! presentation code never receives adapters, coordinators, or authority.

use csv_sdk::contract::{TransferEvent, TransferReceipt};
use csv_sdk::protocol::hash::{ChainId, SanadId};

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

/// Submit a transfer through a configured SDK application host.
///
/// No local wallet state is changed here. Until the host port is configured,
/// this operation is deliberately unavailable.
#[cfg(not(target_arch = "wasm32"))]
pub async fn submit_transfer(_request: TransferRequest) -> Result<TransferSubmission, String> {
    Err("transfer mutation requires a configured SDK application host".to_string())
}

/// Advance an interrupted transfer from its runtime-journalled phase.
///
/// A retry is never inferred from wallet-local state. Until an SDK application
/// host can decide whether resumption is legal, this operation fails closed.
#[cfg(not(target_arch = "wasm32"))]
pub async fn resume_transfer(_request: ResumeRequest) -> Result<TransferSubmission, String> {
    Err("transfer mutation requires a configured SDK application host".to_string())
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
