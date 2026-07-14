//! Wallet-local presentation for stable SDK errors.
//!
//! This module deliberately translates SDK errors only. It does not introduce
//! protocol error variants or prescribe protocol recovery behavior.

use csv_sdk::CsvError;

/// Presentation metadata for an error shown by the wallet UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletErrorPresentation {
    /// Short, user-visible summary.
    pub title: &'static str,
    /// Contextual detail safe to display in the wallet.
    pub detail: String,
    /// Whether the UI may offer another attempt without changing protocol state.
    pub can_retry: bool,
}

impl From<&CsvError> for WalletErrorPresentation {
    fn from(error: &CsvError) -> Self {
        match error {
            CsvError::InsufficientFunds { available, needed, chain } => Self {
                title: "Insufficient funds",
                detail: format!(
                    "{chain} has {available} available, but this operation requires {needed}."
                ),
                can_retry: false,
            },
            CsvError::SanadAlreadyConsumed(id) => Self {
                title: "Sanad already consumed",
                detail: format!("Sanad '{id}' is single-use and cannot be used again."),
                can_retry: false,
            },
            CsvError::ProofVerificationFailed(_) => Self {
                title: "Proof verification failed",
                detail: "The wallet could not verify the proof. Do not continue this operation; inspect the proof and chain evidence.".to_string(),
                can_retry: false,
            },
            CsvError::NetworkError(message) => Self {
                title: "Network unavailable",
                detail: format!("Unable to reach the configured network service: {message}"),
                can_retry: true,
            },
            CsvError::RuntimeError(message) | CsvError::CoordinatorNotAvailable(message) => Self {
                title: "Runtime unavailable",
                detail: format!("The transfer runtime is unavailable: {message}"),
                can_retry: false,
            },
            _ => Self {
                title: "Wallet operation failed",
                detail: error.to_string(),
                can_retry: error.is_retryable(),
            },
        }
    }
}

/// Convert a stable SDK error into wallet UI metadata.
pub fn present_sdk_error(error: &CsvError) -> WalletErrorPresentation {
    WalletErrorPresentation::from(error)
}

#[cfg(test)]
mod tests {
    use super::present_sdk_error;
    use csv_sdk::CsvError;

    #[test]
    fn presents_transient_network_errors_as_retryable() {
        let presentation = present_sdk_error(&CsvError::NetworkError("offline".to_string()));
        assert_eq!(presentation.title, "Network unavailable");
        assert!(presentation.can_retry);
    }

    #[test]
    fn does_not_offer_retry_after_proof_verification_failure() {
        let presentation = present_sdk_error(&CsvError::ProofVerificationFailed(
            "malformed inclusion proof".to_string(),
        ));
        assert_eq!(presentation.title, "Proof verification failed");
        assert!(!presentation.can_retry);
    }

    #[test]
    fn does_not_offer_retry_for_a_consumed_sanad() {
        let presentation =
            present_sdk_error(&CsvError::SanadAlreadyConsumed("sanad-1".to_string()));
        assert_eq!(presentation.title, "Sanad already consumed");
        assert!(!presentation.can_retry);
    }
}
