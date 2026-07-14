//! Platform-neutral ports used by the Dioxus presentation.
//!
//! A browser is not a reduced native runtime.  It talks to a remote runtime
//! over a versioned contract and keeps canonical intent validation and signing
//! local.  Neither adapter accepts key material as a command argument.

use async_trait::async_trait;
use csv_hash::ChainId;
use csv_sdk::contract::{ContractArtifact, SigningIntent};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Errors returned at the presentation/platform boundary.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PlatformError {
    #[error("{capability} is unsupported on {platform}")]
    UnsupportedCapability {
        platform: &'static str,
        capability: &'static str,
    },
    #[error("remote runtime is not configured")]
    RemoteRuntimeUnavailable,
    #[error("remote runtime returned an invalid contract: {0}")]
    InvalidRemoteContract(String),
    #[error("remote runtime transport failed: {0}")]
    RemoteTransport(String),
    #[error("balance unavailable for {address} on {chain}: {reason}")]
    BalanceUnavailable {
        chain: String,
        address: String,
        reason: String,
    },
    #[error("signing intent is invalid: {0}")]
    InvalidSigningIntent(String),
    #[error("signing intent network `{actual}` does not match wallet network `{expected}`")]
    NetworkMismatch { expected: String, actual: String },
    #[error("vault signing failed: {0}")]
    Vault(String),
}

/// Commands that may cross the runtime boundary.  They deliberately contain
/// no mnemonic, private key, seed, or opaque signing payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeCommand {
    Balance { chain: String, address: String },
}

/// Versioned request envelope for the browser-to-runtime boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequest {
    pub schema_version: u16,
    pub command: RuntimeCommand,
}

impl RuntimeRequest {
    const SCHEMA_VERSION: u16 = 1;

    fn new(command: RuntimeCommand) -> Self {
        Self {
            schema_version: Self::SCHEMA_VERSION,
            command,
        }
    }
}

/// Events returned by a runtime command.  A balance is read-only and is never
/// interpreted as transfer completion or proof/finality evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeEvent {
    Balance {
        chain: String,
        address: String,
        total: String,
    },
}

/// Versioned response envelope for the browser-to-runtime boundary.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeResponse {
    pub schema_version: u16,
    pub event: RuntimeEvent,
}

/// Runtime orchestration port.  Transfer commands remain at the existing
/// `transfer_authority` application contract, which already delegates to the
/// runtime coordinator and returns canonical receipts/events only.
#[async_trait(?Send)]
pub trait RuntimePort {
    async fn command(&self, command: RuntimeCommand) -> Result<RuntimeEvent, PlatformError>;
}

/// Secure local vault port.  It accepts a typed intent only; raw bytes cannot
/// be sent to a vault through this interface.
pub trait VaultPort {
    fn sign_intent(
        &self,
        intent: &SigningIntent,
        expected_network: &str,
    ) -> Result<Vec<u8>, PlatformError>;
}

/// The local canonical gate used by both desktop and WASM before signing.
/// This is contract validation, not a claim that a proof was verified.
pub trait CanonicalIntentPort {
    fn validate_intent(
        &self,
        intent: &SigningIntent,
        expected_network: &str,
        now: u64,
    ) -> Result<(), PlatformError>;
}

/// Canonical intent validation backed by the shared wire/codec authorities.
#[derive(Debug, Default)]
pub struct LocalCanonicalIntent;

impl CanonicalIntentPort for LocalCanonicalIntent {
    fn validate_intent(
        &self,
        intent: &SigningIntent,
        expected_network: &str,
        now: u64,
    ) -> Result<(), PlatformError> {
        validate_signing_intent(intent, expected_network, now)
    }
}

/// Reject an incomplete, expired, mismatched-network, or non-canonical intent.
pub fn validate_signing_intent(
    intent: &SigningIntent,
    expected_network: &str,
    now: u64,
) -> Result<(), PlatformError> {
    intent
        .validate_at(now)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    if intent.network != expected_network {
        return Err(PlatformError::NetworkMismatch {
            expected: expected_network.to_string(),
            actual: intent.network.clone(),
        });
    }

    // Re-encode via both canonical authorities.  This detects a non-canonical
    // or mismatched representation before any local key is used.
    let wire = csv_wire::app::encode(intent)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    let codec = csv_codec::to_canonical_cbor(intent)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    if wire != codec {
        return Err(PlatformError::InvalidSigningIntent(
            "intent has inconsistent canonical encoding".to_string(),
        ));
    }
    let decoded: SigningIntent = csv_wire::app::decode(&wire)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    if decoded != *intent {
        return Err(PlatformError::InvalidSigningIntent(
            "intent canonical round-trip changed signed meaning".to_string(),
        ));
    }
    Ok(())
}

/// Platform configuration.  `remote_runtime_url` is an orchestration endpoint,
/// never an RPC endpoint and never a destination for secrets.
#[derive(Debug, Clone, Default)]
pub struct PlatformConfig {
    pub remote_runtime_url: Option<String>,
}

/// The only platform service presentation code needs for chain reads.
#[derive(Clone)]
pub struct WalletPlatform {
    runtime: Arc<dyn RuntimePort>,
}

impl std::fmt::Debug for WalletPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalletPlatform").finish_non_exhaustive()
    }
}

impl WalletPlatform {
    pub fn new(config: PlatformConfig) -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let runtime: Arc<dyn RuntimePort> = {
            let _ = config;
            Arc::new(NativeRuntimePort::new())
        };
        #[cfg(target_arch = "wasm32")]
        let runtime: Arc<dyn RuntimePort> =
            Arc::new(WebRuntimePort::new(config.remote_runtime_url));
        Self { runtime }
    }

    pub async fn balance(&self, address: &str, chain: ChainId) -> Result<String, PlatformError> {
        let chain_name = chain.to_string();
        let requested_address = address.to_string();
        match self
            .runtime
            .command(RuntimeCommand::Balance {
                chain: chain_name.clone(),
                address: requested_address.clone(),
            })
            .await?
        {
            RuntimeEvent::Balance {
                chain,
                address,
                total,
            } if chain == chain_name && address == requested_address => Ok(total),
            RuntimeEvent::Balance { .. } => Err(PlatformError::InvalidRemoteContract(
                "runtime response does not match its balance request".to_string(),
            )),
        }
    }
}

impl Default for WalletPlatform {
    fn default() -> Self {
        Self::new(PlatformConfig::default())
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeRuntimePort {
    runtime: csv_sdk::runtime::ChainRuntime,
}

#[cfg(not(target_arch = "wasm32"))]
impl NativeRuntimePort {
    fn new() -> Self {
        let manager =
            csv_sdk::runtime::RuntimeManager::new(csv_sdk::runtime::RuntimeConfig::default());
        Self {
            runtime: manager.chain_runtime().clone(),
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait(?Send)]
impl RuntimePort for NativeRuntimePort {
    async fn command(&self, command: RuntimeCommand) -> Result<RuntimeEvent, PlatformError> {
        match command {
            RuntimeCommand::Balance { chain, address } => {
                let id = ChainId::new(&chain);
                let balance = self
                    .runtime
                    .get_balance(id, &address)
                    .await
                    .map_err(|error| PlatformError::BalanceUnavailable {
                        chain: chain.clone(),
                        address: address.clone(),
                        reason: error.to_string(),
                    })?;
                Ok(RuntimeEvent::Balance {
                    chain,
                    address,
                    total: balance.total.to_string(),
                })
            }
        }
    }
}

/// Browser runtime adapter.  It is intentionally explicit about the remote
/// contract; with no configured host it fails closed rather than impersonating
/// a native runtime or returning a placeholder value.
#[cfg(target_arch = "wasm32")]
struct WebRuntimePort {
    endpoint: Option<String>,
}

#[cfg(target_arch = "wasm32")]
impl WebRuntimePort {
    fn new(endpoint: Option<String>) -> Self {
        Self { endpoint }
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl RuntimePort for WebRuntimePort {
    async fn command(&self, command: RuntimeCommand) -> Result<RuntimeEvent, PlatformError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let endpoint = self
            .endpoint
            .as_ref()
            .ok_or(PlatformError::RemoteRuntimeUnavailable)?;
        let payload = serde_wasm_bindgen::to_value(&RuntimeRequest::new(command))
            .map_err(|error| PlatformError::InvalidRemoteContract(error.to_string()))?;
        let mut init = web_sys::RequestInit::new();
        init.method("POST");
        init.body(Some(&payload));
        let url = format!("{}/v1/wallet/runtime", endpoint.trim_end_matches('/'));
        let request = web_sys::Request::new_with_str_and_init(&url, &init).map_err(|error| {
            PlatformError::RemoteTransport(format!("invalid request: {error:?}"))
        })?;
        let window = web_sys::window().ok_or(PlatformError::UnsupportedCapability {
            platform: "web",
            capability: "browser window",
        })?;
        let response = JsFuture::from(window.fetch_with_request(&request))
            .await
            .map_err(|error| PlatformError::RemoteTransport(format!("request failed: {error:?}")))?
            .dyn_into::<web_sys::Response>()
            .map_err(|_| {
                PlatformError::RemoteTransport("runtime returned a non-response value".to_string())
            })?;
        if !response.ok() {
            return Err(PlatformError::RemoteTransport(format!(
                "runtime returned HTTP {}",
                response.status()
            )));
        }
        let value = JsFuture::from(response.json().map_err(|error| {
            PlatformError::RemoteTransport(format!("invalid JSON response: {error:?}"))
        })?)
        .await
        .map_err(|error| {
            PlatformError::RemoteTransport(format!("response body failed: {error:?}"))
        })?;
        let response: RuntimeResponse = serde_wasm_bindgen::from_value(value)
            .map_err(|error| PlatformError::InvalidRemoteContract(error.to_string()))?;
        if response.schema_version != RuntimeRequest::SCHEMA_VERSION {
            return Err(PlatformError::InvalidRemoteContract(format!(
                "unsupported schema version {}",
                response.schema_version
            )));
        }
        Ok(response.event)
    }
}

/// Local signer used by desktop and WASM vault implementations.  The key
/// manager is retained locally; only the signature leaves this port.
pub struct LocalVault {
    keys: crate::core::key_manager::KeyManager,
}

impl LocalVault {
    pub fn new(keys: crate::core::key_manager::KeyManager) -> Self {
        Self { keys }
    }
}

impl VaultPort for LocalVault {
    fn sign_intent(
        &self,
        intent: &SigningIntent,
        expected_network: &str,
    ) -> Result<Vec<u8>, PlatformError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?
            .as_secs();
        LocalCanonicalIntent.validate_intent(intent, expected_network, now)?;
        let digest = intent
            .binding_digest()
            .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
        let digest: [u8; 32] = digest.try_into().map_err(|_| {
            PlatformError::InvalidSigningIntent("intent binding digest is not 32 bytes".to_string())
        })?;
        self.keys
            .sign(&ChainId::new(&intent.chain), &digest)
            .map_err(|error| PlatformError::Vault(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{PlatformError, validate_signing_intent};
    use csv_sdk::contract::{IntentOperation, IntentValue, SigningIntent};
    use csv_wire::{SanadIdWire, SealPointWire};

    fn intent(created_at: u64) -> SigningIntent {
        SigningIntent::new(
            "ethereum".to_string(),
            "sepolia".to_string(),
            IntentOperation::LockSanad,
            SanadIdWire {
                bytes: "11".repeat(32),
            },
            SealPointWire {
                id: "22".repeat(32),
                nonce: Some(1),
                version: Some(1),
            },
            "0x1234".to_string(),
            IntentValue {
                amount: "1".to_string(),
                unit: "ETH".to_string(),
            },
            7,
            None,
            "Lock one sanad for the named recipient".to_string(),
            vec![3; 32],
            created_at,
            60,
        )
    }

    #[test]
    fn signing_refuses_expired_intents() {
        assert!(matches!(
            validate_signing_intent(&intent(10), "sepolia", 70),
            Err(PlatformError::InvalidSigningIntent(_))
        ));
    }

    #[test]
    fn signing_refuses_network_mismatch() {
        assert!(matches!(
            validate_signing_intent(&intent(10), "mainnet", 11),
            Err(PlatformError::NetworkMismatch { .. })
        ));
    }

    #[test]
    fn signing_refuses_incomplete_intents() {
        let mut invalid = intent(10);
        invalid.summary.clear();
        assert!(matches!(
            validate_signing_intent(&invalid, "sepolia", 11),
            Err(PlatformError::InvalidSigningIntent(_))
        ));
    }
}
