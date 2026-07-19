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

/// Read-only presentation data derived from SDK-owned protocol objects.
/// Hemion never persists this projection or treats it as protocol authority.
#[derive(Clone)]
pub struct ObjectInspection {
    pub mandate: MandateInspection,
    pub receipt: ReceiptInspection,
    pub evidence: Vec<EvidenceInspection>,
    pub timeline: Vec<TimelineEntry>,
}

#[derive(Clone)]
pub struct MandateInspection {
    pub summary: String,
    pub id: String,
    pub canonical_hex: String,
    pub intent_id: String,
    pub issuer_identity: String,
    pub subject: String,
    pub authority_domain: String,
    pub validity: String,
    pub signature_algorithm: String,
    pub signer_key_id: String,
    pub constraints: Vec<String>,
    pub evidence_requirements: Vec<String>,
}

#[derive(Clone)]
pub struct ReceiptInspection {
    pub summary: String,
    pub id: String,
    pub canonical_hex: String,
    pub attempt_id: String,
    pub mandate_id: String,
    pub intent_id: String,
    pub executor_identity: String,
    pub producer_identity: String,
    pub producer_signature: String,
    pub attempt_state: String,
    pub outcome: String,
    pub dispatch_evidence: Vec<String>,
    pub target_evidence: Vec<String>,
}

#[derive(Clone)]
pub struct EvidenceInspection {
    pub id: String,
    pub kind: String,
    pub producer: String,
    pub collected_at: u64,
    pub content_digest: String,
    pub source: String,
    pub classification: String,
}

#[derive(Clone)]
pub struct TimelineEntry {
    pub timestamp: u64,
    pub label: &'static str,
    pub protocol_state: String,
    pub evidence: String,
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

/// Decode and inspect a local bundle through the exact pinned SDK contract.
pub fn inspect_bundle(bytes: &[u8]) -> Result<ObjectInspection, LocalVerificationError> {
    let decoded = decode_local_verification_bundle(bytes).map_err(map_import_error)?;
    let mandate_bytes = decoded
        .mandate
        .canonical_bytes()
        .map_err(|_| LocalVerificationError::UnsupportedBundleEncoding)?;
    let receipt_bytes = decoded
        .receipt
        .canonical_bytes(&decoded.mandate, &decoded.attempt)
        .map_err(|_| LocalVerificationError::UnsupportedBundleEncoding)?;
    let mandate_id = decoded
        .mandate
        .id()
        .map_err(|_| LocalVerificationError::UnsupportedBundleEncoding)?;
    let receipt_id = decoded
        .receipt
        .id(&decoded.mandate, &decoded.attempt)
        .map_err(|_| LocalVerificationError::UnsupportedBundleEncoding)?;
    let identity = |bytes: &[u8]| {
        format!(
            "{} · hex {}",
            String::from_utf8_lossy(bytes),
            hex::encode(bytes)
        )
    };
    let refs = |items: &[EvidenceNodeId]| {
        items
            .iter()
            .map(|id| hex::encode(id.as_bytes()))
            .collect::<Vec<_>>()
    };
    let evidence = decoded
        .evidence
        .iter()
        .map(|(id, node)| EvidenceInspection {
            id: hex::encode(id.as_bytes()),
            kind: node.kind.registry_id().to_owned(),
            producer: identity(&node.producer_identity),
            collected_at: node.collected_at,
            content_digest: hex::encode(node.content_digest),
            source: format!("{:?}", node.source_locator),
            classification: node.disclosure_classification.clone(),
        })
        .collect::<Vec<_>>();
    let mut timeline = vec![
        TimelineEntry {
            timestamp: decoded.mandate.issued_at,
            label: "Approval issued",
            protocol_state: "Mandate Issued".into(),
            evidence: hex::encode(mandate_id.as_bytes()),
        },
        TimelineEntry {
            timestamp: decoded.attempt.started_at,
            label: "Execution prepared",
            protocol_state: "Attempt Prepared".into(),
            evidence: hex::encode(decoded.attempt.reservation_token_digest),
        },
    ];
    if let Some(timestamp) = decoded.attempt.dispatch_boundary_at {
        timeline.push(TimelineEntry {
            timestamp,
            label: "Provider boundary crossed",
            protocol_state: "Attempt Dispatching".into(),
            evidence: hex::encode(decoded.attempt.provider_request_digest),
        });
    }
    timeline.push(TimelineEntry {
        timestamp: decoded
            .receipt
            .completed_at
            .unwrap_or(decoded.receipt.started_at),
        label: "Outcome recorded",
        protocol_state: format!(
            "Receipt {:?} · Attempt {:?}",
            decoded.receipt.outcome, decoded.attempt.state
        ),
        evidence: hex::encode(receipt_id.as_bytes()),
    });
    timeline.sort_by_key(|entry| entry.timestamp);
    Ok(ObjectInspection {
        mandate: MandateInspection {
            summary: format!(
                "Single-use authority for {} in profile {}",
                decoded.intent.action_type,
                hex::encode(decoded.intent.profile_id.as_bytes())
            ),
            id: hex::encode(mandate_id.as_bytes()),
            canonical_hex: hex::encode(mandate_bytes),
            intent_id: hex::encode(decoded.mandate.intent_id.as_bytes()),
            issuer_identity: identity(&decoded.mandate.issuer_identity),
            subject: format!("{:?}", decoded.mandate.subject),
            authority_domain: identity(&decoded.mandate.authority_domain),
            validity: format!(
                "{} UTC seconds through {} UTC seconds (exclusive)",
                decoded.mandate.valid_from, decoded.mandate.expires_at
            ),
            signature_algorithm: decoded.mandate.signature_requirements.algorithm.clone(),
            signer_key_id: identity(&decoded.mandate.signature_requirements.key_id),
            constraints: decoded
                .mandate
                .constraints
                .iter()
                .map(|item| {
                    format!(
                        "{} · {}",
                        item.registry_id,
                        hex::encode(item.parameters_digest)
                    )
                })
                .collect(),
            evidence_requirements: decoded
                .mandate
                .evidence_requirements
                .iter()
                .map(|item| {
                    format!(
                        "{} · {}",
                        item.registry_id,
                        hex::encode(item.parameters_digest)
                    )
                })
                .collect(),
        },
        receipt: ReceiptInspection {
            summary: format!(
                "Producer-reported outcome {:?}; this is a report, not a truth claim",
                decoded.receipt.outcome
            ),
            id: hex::encode(receipt_id.as_bytes()),
            canonical_hex: hex::encode(receipt_bytes),
            attempt_id: hex::encode(decoded.receipt.attempt_id.as_bytes()),
            mandate_id: hex::encode(decoded.receipt.mandate_id.as_bytes()),
            intent_id: hex::encode(decoded.receipt.intent_id.as_bytes()),
            executor_identity: identity(&decoded.receipt.executor_identity),
            producer_identity: identity(&decoded.receipt.producer_identity),
            producer_signature: hex::encode(&decoded.receipt.producer_signature),
            attempt_state: format!("{:?}", decoded.attempt.state),
            outcome: format!("{:?}", decoded.receipt.outcome),
            dispatch_evidence: refs(&decoded.receipt.dispatch_evidence_refs),
            target_evidence: refs(&decoded.receipt.target_evidence_refs),
        },
        evidence,
        timeline,
    })
}

fn map_import_error(error: ImportError) -> LocalVerificationError {
    match error {
        ImportError::Empty => LocalVerificationError::EmptyImport,
        ImportError::TooLarge => LocalVerificationError::ImportTooLarge,
        ImportError::Malformed | ImportError::UnsupportedVersion | ImportError::InvalidObject => {
            LocalVerificationError::UnsupportedBundleEncoding
        }
    }
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

#[cfg(test)]
mod inspector_tests {
    use super::{LocalVerificationError, inspect_bundle};

    #[test]
    fn inspector_rejects_empty_and_malformed_artifacts() {
        assert!(matches!(
            inspect_bundle(b""),
            Err(LocalVerificationError::EmptyImport)
        ));
        assert!(matches!(
            inspect_bundle(br#"{"format":"unsupported"}"#),
            Err(LocalVerificationError::UnsupportedBundleEncoding)
        ));
    }
}
