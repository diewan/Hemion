//! Offline Accountability bundle verification through the pinned Parwana SDK.

use csv_sdk::accountability::{
    ActionIntent, ActionMandate, ContextBoundOutput, EvidenceNode, EvidenceNodeId,
    ExecutionAttempt, ExecutionReceipt, VerificationContext, VerificationContextId,
};
use csv_sdk::accountability_verification::{
    AlgorithmStatus, AuthenticityStatus, ImportError, ReplayStatus, RevocationStatus,
    VerificationDisposition, VerificationInput, VerificationReport, decode_local_context,
    decode_local_verification_bundle, verify,
};

/// Maximum local import size. The limit is applied before decoding.
pub const MAX_LOCAL_BUNDLE_BYTES: usize = 64 * 1024 * 1024;

/// A fully decoded bundle. The members are Parwana types, never Hemion copies.
pub struct LocalVerificationBundle {
    pub intent: ActionIntent,
    pub mandate: ActionMandate,
    pub attempt: ExecutionAttempt,
    pub receipt: ExecutionReceipt,
    pub evidence: Vec<(EvidenceNodeId, EvidenceNode)>,
}

/// Contexts explicitly available to the operator for this verification run.
pub struct ContextChoice {
    pub name: String,
    pub context: VerificationContext,
    pub revocation_status: RevocationStatus,
    pub algorithm_status: AlgorithmStatus,
    pub replay_status: ReplayStatus,
    pub evidence_authenticity: Vec<(EvidenceNodeId, AuthenticityStatus)>,
    pub expected_executor: Vec<u8>,
}

/// Fail-closed errors at the local import and context-selection boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LocalVerificationError {
    EmptyImport,
    ImportTooLarge,
    UnsupportedBundleEncoding,
    NoContexts,
    ContextNotFound,
    ContextInvalid,
}

/// Result Hemion may render as locally computed.
pub struct LocalVerificationResult {
    pub context_name: String,
    pub context_id: VerificationContextId,
    pub report: VerificationReport,
}

/// Select an explicit context and invoke the side-effect-free Parwana verifier.
pub fn verify_locally(
    bundle: &LocalVerificationBundle,
    contexts: &[ContextChoice],
    selected_context: &str,
) -> Result<LocalVerificationResult, LocalVerificationError> {
    if contexts.is_empty() {
        return Err(LocalVerificationError::NoContexts);
    }
    let choice = contexts
        .iter()
        .find(|choice| choice.name == selected_context)
        .ok_or(LocalVerificationError::ContextNotFound)?;
    choice
        .context
        .validate()
        .map_err(|_| LocalVerificationError::ContextInvalid)?;

    let ContextBoundOutput {
        verification_context_id,
        result,
    } = verify(
        &choice.context,
        VerificationInput {
            intent: &bundle.intent,
            mandate: &bundle.mandate,
            attempt: &bundle.attempt,
            receipt: &bundle.receipt,
            evidence: &bundle.evidence,
            evidence_authenticity: &choice.evidence_authenticity,
            expected_executor: &choice.expected_executor,
            revocation_status: choice.revocation_status,
            algorithm_status: choice.algorithm_status,
            replay_status: choice.replay_status,
        },
    )
    .map_err(|_| LocalVerificationError::ContextInvalid)?;

    Ok(LocalVerificationResult {
        context_name: choice.name.clone(),
        context_id: verification_context_id,
        report: result,
    })
}

/// Reject bytes before any decoder is called. Canonical bundle decoding is deliberately
/// not reimplemented in Hemion; it must arrive through the pinned SDK contract.
pub fn validate_import_bytes(bytes: &[u8]) -> Result<(), LocalVerificationError> {
    if bytes.is_empty() {
        return Err(LocalVerificationError::EmptyImport);
    }
    if bytes.len() > MAX_LOCAL_BUNDLE_BYTES {
        return Err(LocalVerificationError::ImportTooLarge);
    }
    decode_local_verification_bundle(bytes)
        .map(|_| ())
        .map_err(|error| match error {
            ImportError::Empty => LocalVerificationError::EmptyImport,
            ImportError::TooLarge => LocalVerificationError::ImportTooLarge,
            ImportError::Malformed
            | ImportError::UnsupportedVersion
            | ImportError::InvalidObject => LocalVerificationError::UnsupportedBundleEncoding,
        })
}

/// Decode an imported envelope through the SDK and verify it under an explicit context.
pub fn import_and_verify(
    bytes: &[u8],
    contexts: &[ContextChoice],
    selected_context: &str,
) -> Result<LocalVerificationResult, LocalVerificationError> {
    let decoded = decode_local_verification_bundle(bytes).map_err(|error| match error {
        ImportError::Empty => LocalVerificationError::EmptyImport,
        ImportError::TooLarge => LocalVerificationError::ImportTooLarge,
        ImportError::Malformed | ImportError::UnsupportedVersion | ImportError::InvalidObject => {
            LocalVerificationError::UnsupportedBundleEncoding
        }
    })?;
    let bundle = LocalVerificationBundle {
        intent: decoded.intent,
        mandate: decoded.mandate,
        attempt: decoded.attempt,
        receipt: decoded.receipt,
        evidence: decoded.evidence,
    };
    verify_locally(&bundle, contexts, selected_context)
}

/// Decode a context package separately so imported evidence cannot choose its own trust policy.
pub fn import_context(bytes: &[u8]) -> Result<ContextChoice, LocalVerificationError> {
    let decoded = decode_local_context(bytes).map_err(|error| match error {
        ImportError::Empty => LocalVerificationError::EmptyImport,
        ImportError::TooLarge => LocalVerificationError::ImportTooLarge,
        ImportError::Malformed | ImportError::UnsupportedVersion | ImportError::InvalidObject => {
            LocalVerificationError::ContextInvalid
        }
    })?;
    Ok(ContextChoice {
        name: decoded.name,
        context: decoded.context,
        revocation_status: decoded.revocation_status,
        algorithm_status: decoded.algorithm_status,
        replay_status: decoded.replay_status,
        evidence_authenticity: decoded.evidence_authenticity,
        expected_executor: decoded.expected_executor,
    })
}

/// Stable plain-language label for the overall local computation.
pub const fn disposition_label(disposition: VerificationDisposition) -> &'static str {
    match disposition {
        VerificationDisposition::Valid => "Requirements met",
        VerificationDisposition::Invalid => "Requirements not met",
        VerificationDisposition::Indeterminate => "Cannot be determined",
    }
}
