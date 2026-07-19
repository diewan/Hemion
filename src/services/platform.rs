//! Platform-neutral ports used by the Dioxus presentation.
//!
//! A browser is not a reduced native runtime.  It talks to a remote runtime
//! over a versioned contract and keeps canonical intent validation and signing
//! local.  Neither adapter accepts key material as a command argument.

use async_trait::async_trait;
use csv_sdk::contract::{ContractArtifact, SigningIntent};
use csv_sdk::protocol::hash::ChainId;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashSet;
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
    #[error("inbound intent rejected: {0}")]
    InboundIntent(String),
    #[error("portable wallet file operation failed: {0}")]
    PortableFile(String),
}

/// Presentation target used to report capability support honestly. Pages
/// consume this service and never branch on compilation targets themselves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlatformTarget {
    Desktop,
    Web,
    Mobile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityState {
    Supported,
    RequiresRemoteRuntime,
    Unavailable,
}

/// Per-target support matrix. `Unavailable` is a product state, not a signal
/// to substitute simulated behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformCapabilities {
    pub target: PlatformTarget,
    pub file_exchange: CapabilityState,
    pub camera_qr: CapabilityState,
    pub deep_links: CapabilityState,
    pub local_vault: CapabilityState,
    pub runtime_orchestration: CapabilityState,
    pub local_notifications: CapabilityState,
    pub background_push: CapabilityState,
}

impl PlatformCapabilities {
    pub const fn for_target(target: PlatformTarget) -> Self {
        match target {
            PlatformTarget::Desktop => Self {
                target,
                file_exchange: CapabilityState::Supported,
                camera_qr: CapabilityState::Unavailable,
                deep_links: CapabilityState::Supported,
                local_vault: CapabilityState::Supported,
                runtime_orchestration: CapabilityState::Supported,
                local_notifications: CapabilityState::Supported,
                background_push: CapabilityState::Unavailable,
            },
            PlatformTarget::Web => Self {
                target,
                file_exchange: CapabilityState::Supported,
                camera_qr: CapabilityState::Unavailable,
                deep_links: CapabilityState::Supported,
                local_vault: CapabilityState::Supported,
                runtime_orchestration: CapabilityState::RequiresRemoteRuntime,
                local_notifications: CapabilityState::Supported,
                background_push: CapabilityState::Unavailable,
            },
            PlatformTarget::Mobile => Self {
                target,
                file_exchange: CapabilityState::Unavailable,
                camera_qr: CapabilityState::Unavailable,
                deep_links: CapabilityState::Unavailable,
                local_vault: CapabilityState::Unavailable,
                runtime_orchestration: CapabilityState::RequiresRemoteRuntime,
                local_notifications: CapabilityState::Unavailable,
                background_push: CapabilityState::Unavailable,
            },
        }
    }

    pub const fn current() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::for_target(PlatformTarget::Web)
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::for_target(PlatformTarget::Desktop)
        }
    }
}

/// Maximum encoded inbound intent accepted before any decoder is invoked.
/// This is deliberately small enough for local delivery channels and prevents
/// QR/deep-link/file input from turning into an allocation attack.
pub const MAX_INBOUND_INTENT_BYTES: usize = 64 * 1024;

/// Where an untrusted delivery originated. The source is presentation data;
/// it never changes what the canonical intent means or authorizes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InboundOrigin {
    ScannedQr,
    BrowserLink,
    ImportedFile,
    Relay,
}

impl InboundOrigin {
    pub const fn label(self) -> &'static str {
        match self {
            Self::ScannedQr => "scanned QR code",
            Self::BrowserLink => "browser link",
            Self::ImportedFile => "imported file",
            Self::Relay => "encrypted relay",
        }
    }
}

/// A decoded, unapproved inbound request. Holding this value cannot mutate
/// wallet state; callers must render it in the review/accept flow first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboundIntent {
    pub id: String,
    pub origin: InboundOrigin,
    pub intent: SigningIntent,
}

impl InboundIntent {
    /// Text rendered by the review surface; it is intentionally not supplied
    /// by the untrusted package or deep-link query string.
    pub fn origin_display(&self) -> String {
        self.origin.label().to_string()
    }
}

/// Result of passing an inbound package through the one delivery choke point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InboundDelivery {
    PendingReview(Box<InboundIntent>),
    Duplicate { id: String },
}

/// Platform delivery boundary for QR, camera/share, files, deep links, and
/// relays. Implementations must return an unapproved typed intent only.
pub trait DeliveryPort {
    fn receive(
        &mut self,
        origin: InboundOrigin,
        payload: &[u8],
        now: u64,
    ) -> Result<InboundDelivery, PlatformError>;

    fn receive_deep_link(&mut self, url: &str, now: u64) -> Result<InboundDelivery, PlatformError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortableFileOutcome {
    Saved,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PortableOpenOutcome {
    Opened(Vec<u8>),
    Cancelled,
}

/// Platform-owned save boundary for encrypted portable wallet bytes. Pages
/// never call browser APIs or native filesystem dialogs directly.
pub trait PortableFilePort {
    fn save_encrypted_wallet(
        &self,
        suggested_name: &str,
        bytes: &[u8],
    ) -> Result<PortableFileOutcome, PlatformError>;

    fn open_encrypted_wallet(&self) -> Result<PortableOpenOutcome, PlatformError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlatformPortableFilePort;

#[cfg(not(target_arch = "wasm32"))]
impl PortableFilePort for PlatformPortableFilePort {
    fn save_encrypted_wallet(
        &self,
        suggested_name: &str,
        bytes: &[u8],
    ) -> Result<PortableFileOutcome, PlatformError> {
        use std::io::Write;
        #[cfg(unix)]
        use std::os::unix::fs::OpenOptionsExt;

        let Some(path) = rfd::FileDialog::new()
            .add_filter("Encrypted CSV wallet", &["csvw"])
            .set_file_name(suggested_name)
            .save_file()
        else {
            return Ok(PortableFileOutcome::Cancelled);
        };
        let mut options = std::fs::OpenOptions::new();
        options.create(true).truncate(true).write(true);
        #[cfg(unix)]
        options.mode(0o600);
        let mut file = options
            .open(path)
            .map_err(|error| PlatformError::PortableFile(error.to_string()))?;
        file.write_all(bytes)
            .and_then(|_| file.sync_all())
            .map_err(|error| PlatformError::PortableFile(error.to_string()))?;
        Ok(PortableFileOutcome::Saved)
    }

    fn open_encrypted_wallet(&self) -> Result<PortableOpenOutcome, PlatformError> {
        let Some(path) = rfd::FileDialog::new()
            .add_filter("Encrypted CSV wallet", &["csvw"])
            .pick_file()
        else {
            return Ok(PortableOpenOutcome::Cancelled);
        };
        let metadata = std::fs::metadata(&path)
            .map_err(|error| PlatformError::PortableFile(error.to_string()))?;
        if metadata.len() > 16 * 1024 * 1024 {
            return Err(PlatformError::PortableFile(
                "wallet file exceeds the 16 MiB import limit".to_string(),
            ));
        }
        std::fs::read(path)
            .map(PortableOpenOutcome::Opened)
            .map_err(|error| PlatformError::PortableFile(error.to_string()))
    }
}

#[cfg(target_arch = "wasm32")]
impl PortableFilePort for PlatformPortableFilePort {
    fn save_encrypted_wallet(
        &self,
        suggested_name: &str,
        bytes: &[u8],
    ) -> Result<PortableFileOutcome, PlatformError> {
        use wasm_bindgen::JsCast;

        let window = web_sys::window().ok_or(PlatformError::UnsupportedCapability {
            platform: "web",
            capability: "encrypted wallet download",
        })?;
        let options = web_sys::BlobPropertyBag::new();
        options.set_type("application/octet-stream");
        let blob = web_sys::Blob::new_with_u8_array_sequence_and_options(
            &js_sys::Array::from_iter([js_sys::Uint8Array::from(bytes)]),
            &options,
        )
        .map_err(|error| PlatformError::PortableFile(format!("blob creation failed: {error:?}")))?;
        let url = web_sys::Url::create_object_url_with_blob(&blob).map_err(|error| {
            PlatformError::PortableFile(format!("download URL failed: {error:?}"))
        })?;
        let document = window
            .document()
            .ok_or(PlatformError::UnsupportedCapability {
                platform: "web",
                capability: "document download",
            })?;
        let element = document.create_element("a").map_err(|error| {
            PlatformError::PortableFile(format!("download element failed: {error:?}"))
        })?;
        let anchor = element
            .dyn_ref::<web_sys::HtmlAnchorElement>()
            .ok_or_else(|| {
                PlatformError::PortableFile("download element is not an anchor".to_string())
            })?;
        anchor.set_href(&url);
        anchor.set_download(suggested_name);
        anchor.click();
        web_sys::Url::revoke_object_url(&url).map_err(|error| {
            PlatformError::PortableFile(format!("download cleanup failed: {error:?}"))
        })?;
        Ok(PortableFileOutcome::Saved)
    }

    fn open_encrypted_wallet(&self) -> Result<PortableOpenOutcome, PlatformError> {
        Err(PlatformError::UnsupportedCapability {
            platform: "web",
            capability: "native open dialog; use the encrypted file picker",
        })
    }
}

/// In-memory idempotency gate for the current wallet session. It deliberately
/// records only delivery fingerprints, never acceptance or transfer state.
#[derive(Debug, Default)]
pub struct LocalDeliveryGate {
    delivered: HashSet<String>,
}

impl LocalDeliveryGate {
    #[cfg(test)]
    fn delivery_count(&self) -> usize {
        self.delivered.len()
    }

    fn decode(
        origin: InboundOrigin,
        payload: &[u8],
        now: u64,
    ) -> Result<InboundIntent, PlatformError> {
        if payload.len() > MAX_INBOUND_INTENT_BYTES {
            return Err(PlatformError::InboundIntent(format!(
                "package exceeds the {} byte limit",
                MAX_INBOUND_INTENT_BYTES
            )));
        }
        let intent: SigningIntent = csv_sdk::canonical::app::decode(payload).map_err(|error| {
            PlatformError::InboundIntent(format!(
                "package is not a canonical signing intent: {error}"
            ))
        })?;
        // `decode` recognizes the artifact header; byte-for-byte re-encoding
        // rejects alternate encodings and trailing/partial-trust formats.
        let canonical = csv_sdk::canonical::app::encode(&intent).map_err(|error| {
            PlatformError::InboundIntent(format!("intent cannot be canonicalized: {error}"))
        })?;
        if canonical != payload {
            return Err(PlatformError::InboundIntent(
                "package is not canonical encoding".to_string(),
            ));
        }
        validate_signing_intent(&intent, &intent.network, now).map_err(|error| {
            PlatformError::InboundIntent(format!("local package validation failed: {error}"))
        })?;
        let id = hex::encode(sha2::Sha256::digest(payload));
        Ok(InboundIntent { id, origin, intent })
    }
}

impl DeliveryPort for LocalDeliveryGate {
    fn receive(
        &mut self,
        origin: InboundOrigin,
        payload: &[u8],
        now: u64,
    ) -> Result<InboundDelivery, PlatformError> {
        let inbound = Self::decode(origin, payload, now)?;
        if !self.delivered.insert(inbound.id.clone()) {
            return Ok(InboundDelivery::Duplicate { id: inbound.id });
        }
        Ok(InboundDelivery::PendingReview(Box::new(inbound)))
    }

    fn receive_deep_link(&mut self, url: &str, now: u64) -> Result<InboundDelivery, PlatformError> {
        let query = url.strip_prefix("hemion://accept?").ok_or_else(|| {
            PlatformError::InboundIntent("link must use the hemion://accept endpoint".to_string())
        })?;
        let mut encoded = None;
        for pair in query.split('&') {
            let Some((key, value)) = pair.split_once('=') else {
                return Err(PlatformError::InboundIntent(
                    "malformed link query".to_string(),
                ));
            };
            match key {
                "intent" if encoded.is_none() => encoded = Some(value),
                "intent" => {
                    return Err(PlatformError::InboundIntent(
                        "link contains duplicate intent data".to_string(),
                    ));
                }
                _ => {
                    return Err(PlatformError::InboundIntent(
                        "link contains an unsupported parameter".to_string(),
                    ));
                }
            }
        }
        let encoded = encoded.ok_or_else(|| {
            PlatformError::InboundIntent("link has no intent package".to_string())
        })?;
        if encoded.len() > MAX_INBOUND_INTENT_BYTES * 2 {
            return Err(PlatformError::InboundIntent(
                "link package exceeds the size limit".to_string(),
            ));
        }
        let payload = hex::decode(encoded).map_err(|_| {
            PlatformError::InboundIntent(
                "link intent must be hexadecimal canonical CBOR".to_string(),
            )
        })?;
        self.receive(InboundOrigin::BrowserLink, &payload, now)
    }
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

/// Read-only runtime port. Transfer mutation remains at the separate
/// `transfer_authority` boundary, which delegates lifecycle authority through
/// the SDK coordinator; this port cannot mutate protocol lifecycle state.
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
    let wire = csv_sdk::canonical::app::encode(intent)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    let codec = csv_sdk::canonical::to_canonical_cbor(intent)
        .map_err(|error| PlatformError::InvalidSigningIntent(error.to_string()))?;
    if wire != codec {
        return Err(PlatformError::InvalidSigningIntent(
            "intent has inconsistent canonical encoding".to_string(),
        ));
    }
    let decoded: SigningIntent = csv_sdk::canonical::app::decode(&wire)
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
    use super::{
        CapabilityState, DeliveryPort, InboundDelivery, InboundOrigin, LocalDeliveryGate,
        MAX_INBOUND_INTENT_BYTES, PlatformCapabilities, PlatformError, PlatformTarget,
        validate_signing_intent,
    };
    use csv_sdk::canonical::{SanadIdWire, SealPointWire};
    use csv_sdk::contract::{IntentOperation, IntentValue, SigningIntent};

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
    fn platform_capabilities_never_claim_unimplemented_mobile_support() {
        let mobile = PlatformCapabilities::for_target(PlatformTarget::Mobile);
        assert_eq!(mobile.camera_qr, CapabilityState::Unavailable);
        assert_eq!(mobile.local_vault, CapabilityState::Unavailable);
        assert_eq!(mobile.background_push, CapabilityState::Unavailable);

        let web = PlatformCapabilities::for_target(PlatformTarget::Web);
        assert_eq!(
            web.runtime_orchestration,
            CapabilityState::RequiresRemoteRuntime
        );
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

    #[test]
    fn inbound_payloads_are_size_limited_before_decode() {
        let mut gate = LocalDeliveryGate::default();
        let oversized = vec![0_u8; MAX_INBOUND_INTENT_BYTES + 1];
        assert!(matches!(
            gate.receive(InboundOrigin::ImportedFile, &oversized, 11),
            Err(PlatformError::InboundIntent(_))
        ));
        assert_eq!(gate.delivery_count(), 0);
    }

    #[test]
    fn malformed_or_truncated_canonical_payload_never_becomes_pending() {
        let mut gate = LocalDeliveryGate::default();
        let mut encoded = csv_sdk::canonical::app::encode(&intent(10)).expect("canonical intent");
        encoded.pop();
        assert!(matches!(
            gate.receive(InboundOrigin::ScannedQr, &encoded, 11),
            Err(PlatformError::InboundIntent(_))
        ));
        assert_eq!(gate.delivery_count(), 0);
    }

    #[test]
    fn inbound_delivery_is_pending_review_and_duplicate_delivery_is_idempotent() {
        let mut gate = LocalDeliveryGate::default();
        let encoded = csv_sdk::canonical::app::encode(&intent(10)).expect("canonical intent");
        let first = gate
            .receive(InboundOrigin::BrowserLink, &encoded, 11)
            .expect("pending review");
        let id = match first {
            InboundDelivery::PendingReview(inbound) => {
                assert_eq!(inbound.origin_display(), "browser link");
                inbound.id
            }
            InboundDelivery::Duplicate { .. } => panic!("first delivery must require review"),
        };
        assert_eq!(gate.delivery_count(), 1);
        assert_eq!(
            gate.receive(InboundOrigin::BrowserLink, &encoded, 11),
            Ok(InboundDelivery::Duplicate { id })
        );
    }

    #[test]
    fn deep_link_is_only_an_unapproved_browser_delivery() {
        let mut gate = LocalDeliveryGate::default();
        let encoded =
            hex::encode(csv_sdk::canonical::app::encode(&intent(10)).expect("canonical intent"));
        let delivery = gate
            .receive_deep_link(&format!("hemion://accept?intent={encoded}"), 11)
            .expect("deep link is parsed");
        assert!(matches!(delivery, InboundDelivery::PendingReview(_)));
        // The delivery gate has only recorded idempotency; no wallet or
        // transfer authority is reachable from this parser.
        assert_eq!(gate.delivery_count(), 1);
    }
}
