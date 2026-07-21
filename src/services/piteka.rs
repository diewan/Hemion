//! Read-only Piteka environment connection.
//!
//! Piteka remains the source of live workflow state. Hemion uses only its
//! authorized HTTP API and treats every downloaded object as untrusted until
//! the pinned Parwana verifier has checked it locally.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::services::bundle_verifier::{
    LocalVerificationError, LocalVerificationResult, import_and_verify, import_context,
};

pub const PITEKA_CONNECTION_VERSION: u16 = 1;
const MAX_IDENTIFIER_LEN: usize = 128;
const MAX_EXPORT_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PitekaEnvironment {
    pub api_base_url: String,
    pub tenant_id: String,
    pub access_token: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AuthorizedGet {
    pub url: String,
    pub authorization: String,
    pub tenant_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptExport {
    pub schema_version: u16,
    pub receipt_id: String,
    pub bundle: Vec<u8>,
    pub verification_context: Vec<u8>,
}

/// The assembled accountability chain for one mandate, as served by Piteka's
/// read API (`GET /api/v1/mandates/{id}/chain`). This is discovery detail only:
/// it is never trusted for a verdict — validity is recomputed locally with the
/// Parwana verifier via [`download_and_verify`].
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MandateChain {
    pub mandate: MandateDetail,
    pub timeline: Vec<ChainStep>,
    pub attempts: Vec<ChainAttempt>,
    pub receipts: Vec<ReceiptDetail>,
    pub evidence: Vec<ChainEvidence>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MandateDetail {
    pub mandate_id: String,
    pub state: String,
    pub version: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainStep {
    pub at: i64,
    pub actor: Option<String>,
    pub action: String,
    pub decision: String,
    pub detail: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainAttempt {
    pub attempt_id: String,
    pub executor_identity: String,
    pub state: String,
    /// Absent until the provider call completes (the server omits it when None).
    #[serde(default)]
    pub github_deployment_id: Option<u64>,
    pub started_at: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptDetail {
    pub receipt_id: String,
    pub mandate_id: String,
    pub intent_id: String,
    pub attempt_id: String,
    pub outcome: String,
    pub created_at: i64,
    pub dispatch_evidence_refs: Vec<String>,
    pub target_evidence_refs: Vec<String>,
    pub evidence_gaps: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChainEvidence {
    pub node_id: String,
    pub registry_id: String,
    pub source: String,
    pub producer_identity: String,
    pub content_digest: String,
    pub media_type: String,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PitekaConnectionError {
    #[error("Piteka API URL must use https (http is allowed only for localhost development)")]
    UnsafeApiUrl,
    #[error("an environment tenant is required")]
    MissingTenant,
    #[error("an access token is required")]
    MissingAccessToken,
    #[error("the receipt identifier is malformed")]
    MalformedReceiptId,
    #[error("the mandate identifier is malformed")]
    MalformedMandateId,
    #[error("Piteka returned a malformed accountability chain")]
    MalformedChain,
    #[error("Piteka returned an unsupported export version")]
    UnsupportedVersion,
    #[error("Piteka returned an export for a different receipt")]
    ReceiptMismatch,
    #[error("Piteka API request was rejected or unavailable: {0}")]
    Api(String),
    #[error("the downloaded object failed local Parwana verification: {0:?}")]
    LocalVerification(LocalVerificationError),
}

#[async_trait(?Send)]
pub trait PitekaApiPort {
    async fn get_receipt_export(
        &self,
        request: AuthorizedGet,
    ) -> Result<ReceiptExport, PitekaConnectionError>;

    /// Fetch the assembled accountability chain for a mandate. Read-only
    /// discovery detail; never a verdict.
    async fn get_mandate_chain(
        &self,
        request: AuthorizedGet,
    ) -> Result<MandateChain, PitekaConnectionError>;
}

/// Production HTTP adapter. It has no storage or database capability.
#[derive(Clone, Copy, Debug, Default)]
pub struct LivePitekaApi;

#[cfg(not(target_arch = "wasm32"))]
#[async_trait(?Send)]
impl PitekaApiPort for LivePitekaApi {
    async fn get_receipt_export(
        &self,
        request: AuthorizedGet,
    ) -> Result<ReceiptExport, PitekaConnectionError> {
        let response = reqwest::Client::new()
            .get(request.url)
            .header(reqwest::header::AUTHORIZATION, request.authorization)
            .header("X-Tenant-Id", request.tenant_id)
            .send()
            .await
            .map_err(|error| PitekaConnectionError::Api(error.to_string()))?;
        if !response.status().is_success() {
            return Err(PitekaConnectionError::Api(format!(
                "HTTP {}",
                response.status()
            )));
        }
        if response
            .content_length()
            .is_some_and(|size| size > MAX_EXPORT_BYTES as u64)
        {
            return Err(PitekaConnectionError::Api(
                "export exceeds size limit".into(),
            ));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| PitekaConnectionError::Api(error.to_string()))?;
        if bytes.len() > MAX_EXPORT_BYTES {
            return Err(PitekaConnectionError::Api(
                "export exceeds size limit".into(),
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| PitekaConnectionError::Api("malformed export envelope".into()))
    }

    async fn get_mandate_chain(
        &self,
        request: AuthorizedGet,
    ) -> Result<MandateChain, PitekaConnectionError> {
        let response = reqwest::Client::new()
            .get(request.url)
            .header(reqwest::header::AUTHORIZATION, request.authorization)
            .header("X-Tenant-Id", request.tenant_id)
            .send()
            .await
            .map_err(|error| PitekaConnectionError::Api(error.to_string()))?;
        if !response.status().is_success() {
            return Err(PitekaConnectionError::Api(format!(
                "HTTP {}",
                response.status()
            )));
        }
        let bytes = response
            .bytes()
            .await
            .map_err(|error| PitekaConnectionError::Api(error.to_string()))?;
        if bytes.len() > MAX_EXPORT_BYTES {
            return Err(PitekaConnectionError::Api("chain exceeds size limit".into()));
        }
        serde_json::from_slice(&bytes).map_err(|_| PitekaConnectionError::MalformedChain)
    }
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl PitekaApiPort for LivePitekaApi {
    async fn get_receipt_export(
        &self,
        request: AuthorizedGet,
    ) -> Result<ReceiptExport, PitekaConnectionError> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;
        let init = web_sys::RequestInit::new();
        init.set_method("GET");
        init.set_mode(web_sys::RequestMode::Cors);
        let headers = web_sys::Headers::new()
            .map_err(|_| PitekaConnectionError::Api("headers unavailable".into()))?;
        headers
            .set("Authorization", &request.authorization)
            .map_err(|_| PitekaConnectionError::Api("invalid authorization header".into()))?;
        headers
            .set("X-Tenant-Id", &request.tenant_id)
            .map_err(|_| PitekaConnectionError::Api("invalid tenant header".into()))?;
        init.set_headers(&headers);
        let web_request = web_sys::Request::new_with_str_and_init(&request.url, &init)
            .map_err(|_| PitekaConnectionError::Api("invalid request".into()))?;
        let window = web_sys::window()
            .ok_or_else(|| PitekaConnectionError::Api("browser unavailable".into()))?;
        let value = JsFuture::from(window.fetch_with_request(&web_request))
            .await
            .map_err(|_| PitekaConnectionError::Api("request failed".into()))?;
        let response: web_sys::Response = value
            .dyn_into()
            .map_err(|_| PitekaConnectionError::Api("invalid response".into()))?;
        if !response.ok() {
            return Err(PitekaConnectionError::Api(format!(
                "HTTP {}",
                response.status()
            )));
        }
        let buffer = JsFuture::from(
            response
                .array_buffer()
                .map_err(|_| PitekaConnectionError::Api("response unreadable".into()))?,
        )
        .await
        .map_err(|_| PitekaConnectionError::Api("response unreadable".into()))?;
        let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
        if bytes.len() > MAX_EXPORT_BYTES {
            return Err(PitekaConnectionError::Api(
                "export exceeds size limit".into(),
            ));
        }
        serde_json::from_slice(&bytes)
            .map_err(|_| PitekaConnectionError::Api("malformed export envelope".into()))
    }

    async fn get_mandate_chain(
        &self,
        request: AuthorizedGet,
    ) -> Result<MandateChain, PitekaConnectionError> {
        let bytes = wasm_fetch_bytes(request).await?;
        serde_json::from_slice(&bytes).map_err(|_| PitekaConnectionError::MalformedChain)
    }
}

/// Shared browser fetch-to-bytes for the wasm read paths.
#[cfg(target_arch = "wasm32")]
async fn wasm_fetch_bytes(request: AuthorizedGet) -> Result<Vec<u8>, PitekaConnectionError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_mode(web_sys::RequestMode::Cors);
    let headers = web_sys::Headers::new()
        .map_err(|_| PitekaConnectionError::Api("headers unavailable".into()))?;
    headers
        .set("Authorization", &request.authorization)
        .map_err(|_| PitekaConnectionError::Api("invalid authorization header".into()))?;
    headers
        .set("X-Tenant-Id", &request.tenant_id)
        .map_err(|_| PitekaConnectionError::Api("invalid tenant header".into()))?;
    init.set_headers(&headers);
    let web_request = web_sys::Request::new_with_str_and_init(&request.url, &init)
        .map_err(|_| PitekaConnectionError::Api("invalid request".into()))?;
    let window = web_sys::window()
        .ok_or_else(|| PitekaConnectionError::Api("browser unavailable".into()))?;
    let value = JsFuture::from(window.fetch_with_request(&web_request))
        .await
        .map_err(|_| PitekaConnectionError::Api("request failed".into()))?;
    let response: web_sys::Response = value
        .dyn_into()
        .map_err(|_| PitekaConnectionError::Api("invalid response".into()))?;
    if !response.ok() {
        return Err(PitekaConnectionError::Api(format!(
            "HTTP {}",
            response.status()
        )));
    }
    let buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|_| PitekaConnectionError::Api("response unreadable".into()))?,
    )
    .await
    .map_err(|_| PitekaConnectionError::Api("response unreadable".into()))?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    if bytes.len() > MAX_EXPORT_BYTES {
        return Err(PitekaConnectionError::Api("chain exceeds size limit".into()));
    }
    Ok(bytes)
}

impl PitekaEnvironment {
    pub fn receipt_export_request(
        &self,
        receipt_id: &str,
    ) -> Result<AuthorizedGet, PitekaConnectionError> {
        let base = self.api_base_url.trim().trim_end_matches('/');
        let local_http = base.starts_with("http://localhost")
            || base.starts_with("http://127.0.0.1")
            || base.starts_with("http://[::1]");
        if !(base.starts_with("https://") || local_http) {
            return Err(PitekaConnectionError::UnsafeApiUrl);
        }
        validate_identifier(&self.tenant_id).map_err(|_| PitekaConnectionError::MissingTenant)?;
        validate_identifier(receipt_id).map_err(|_| PitekaConnectionError::MalformedReceiptId)?;
        let token = self.access_token.trim();
        if token.is_empty() || token.contains(['\r', '\n']) {
            return Err(PitekaConnectionError::MissingAccessToken);
        }
        Ok(AuthorizedGet {
            url: format!("{base}/api/v1/receipts/{receipt_id}/export"),
            authorization: format!("Bearer {token}"),
            tenant_id: self.tenant_id.clone(),
        })
    }

    /// Builds an authorized request for a mandate's assembled chain.
    pub fn mandate_chain_request(
        &self,
        mandate_id: &str,
    ) -> Result<AuthorizedGet, PitekaConnectionError> {
        let base = self.api_base_url.trim().trim_end_matches('/');
        let local_http = base.starts_with("http://localhost")
            || base.starts_with("http://127.0.0.1")
            || base.starts_with("http://[::1]");
        if !(base.starts_with("https://") || local_http) {
            return Err(PitekaConnectionError::UnsafeApiUrl);
        }
        validate_identifier(&self.tenant_id).map_err(|_| PitekaConnectionError::MissingTenant)?;
        validate_identifier(mandate_id).map_err(|_| PitekaConnectionError::MalformedMandateId)?;
        let token = self.access_token.trim();
        if token.is_empty() || token.contains(['\r', '\n']) {
            return Err(PitekaConnectionError::MissingAccessToken);
        }
        Ok(AuthorizedGet {
            url: format!("{base}/api/v1/mandates/{mandate_id}/chain"),
            authorization: format!("Bearer {token}"),
            tenant_id: self.tenant_id.clone(),
        })
    }
}

/// Fetch the assembled accountability chain for a mandate through the authorized
/// read API. This is discovery detail only; receipts within it are verified
/// independently and locally via [`download_and_verify`].
pub async fn fetch_chain<P: PitekaApiPort>(
    api: &P,
    environment: &PitekaEnvironment,
    mandate_id: &str,
) -> Result<MandateChain, PitekaConnectionError> {
    let request = environment.mandate_chain_request(mandate_id)?;
    api.get_mandate_chain(request).await
}

/// Download through the authorized API and independently verify with Parwana.
pub async fn download_and_verify<P: PitekaApiPort>(
    api: &P,
    environment: &PitekaEnvironment,
    receipt_id: &str,
) -> Result<LocalVerificationResult, PitekaConnectionError> {
    let request = environment.receipt_export_request(receipt_id)?;
    let export = api.get_receipt_export(request).await?;
    if export.schema_version != PITEKA_CONNECTION_VERSION {
        return Err(PitekaConnectionError::UnsupportedVersion);
    }
    if export.receipt_id != receipt_id {
        return Err(PitekaConnectionError::ReceiptMismatch);
    }
    let context = import_context(&export.verification_context)
        .map_err(PitekaConnectionError::LocalVerification)?;
    let selected = context.name.clone();
    import_and_verify(&export.bundle, &[context], &selected)
        .map_err(PitekaConnectionError::LocalVerification)
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct RecordingApi {
        request: RefCell<Option<AuthorizedGet>>,
        response: Result<ReceiptExport, PitekaConnectionError>,
    }

    #[async_trait(?Send)]
    impl PitekaApiPort for RecordingApi {
        async fn get_receipt_export(
            &self,
            request: AuthorizedGet,
        ) -> Result<ReceiptExport, PitekaConnectionError> {
            self.request.replace(Some(request));
            self.response.clone()
        }

        async fn get_mandate_chain(
            &self,
            request: AuthorizedGet,
        ) -> Result<MandateChain, PitekaConnectionError> {
            self.request.replace(Some(request));
            Err(PitekaConnectionError::MalformedChain)
        }
    }

    impl Clone for PitekaConnectionError {
        fn clone(&self) -> Self {
            match self {
                Self::Api(value) => Self::Api(value.clone()),
                Self::UnsafeApiUrl => Self::UnsafeApiUrl,
                Self::MissingTenant => Self::MissingTenant,
                Self::MissingAccessToken => Self::MissingAccessToken,
                Self::MalformedReceiptId => Self::MalformedReceiptId,
                Self::MalformedMandateId => Self::MalformedMandateId,
                Self::MalformedChain => Self::MalformedChain,
                Self::UnsupportedVersion => Self::UnsupportedVersion,
                Self::ReceiptMismatch => Self::ReceiptMismatch,
                Self::LocalVerification(value) => Self::LocalVerification(value.clone()),
            }
        }
    }

    fn environment() -> PitekaEnvironment {
        PitekaEnvironment {
            api_base_url: "https://piteka.example".into(),
            tenant_id: "environment-prod".into(),
            access_token: "secret-token".into(),
        }
    }

    #[test]
    fn builds_an_authorized_api_request_without_database_access() {
        let request = environment().receipt_export_request("receipt-42").unwrap();
        assert_eq!(
            request.url,
            "https://piteka.example/api/v1/receipts/receipt-42/export"
        );
        assert_eq!(request.authorization, "Bearer secret-token");
        assert_eq!(request.tenant_id, "environment-prod");
    }

    #[test]
    fn rejects_unsafe_or_ambiguous_connection_inputs() {
        let mut value = environment();
        value.api_base_url = "http://piteka.example".into();
        assert_eq!(
            value.receipt_export_request("receipt-42"),
            Err(PitekaConnectionError::UnsafeApiUrl)
        );
        let mut value = environment();
        value.tenant_id = "tenant/other".into();
        assert_eq!(
            value.receipt_export_request("receipt-42"),
            Err(PitekaConnectionError::MissingTenant)
        );
        assert_eq!(
            environment().receipt_export_request("../receipt"),
            Err(PitekaConnectionError::MalformedReceiptId)
        );
    }

    #[test]
    fn localhost_http_is_explicitly_available_for_development() {
        let mut value = environment();
        value.api_base_url = "http://localhost:8080/".into();
        assert!(value.receipt_export_request("receipt-42").is_ok());
    }

    #[tokio::test]
    async fn downloaded_malformed_objects_fail_local_verification() {
        let api = RecordingApi {
            request: RefCell::new(None),
            response: Ok(ReceiptExport {
                schema_version: PITEKA_CONNECTION_VERSION,
                receipt_id: "receipt-42".into(),
                bundle: b"not a bundle".to_vec(),
                verification_context: b"not a context".to_vec(),
            }),
        };
        assert!(matches!(
            download_and_verify(&api, &environment(), "receipt-42").await,
            Err(PitekaConnectionError::LocalVerification(_))
        ));
        assert!(api.request.borrow().is_some());
    }

    #[tokio::test]
    async fn cross_receipt_response_is_rejected_before_verification() {
        let api = RecordingApi {
            request: RefCell::new(None),
            response: Ok(ReceiptExport {
                schema_version: PITEKA_CONNECTION_VERSION,
                receipt_id: "receipt-other".into(),
                bundle: vec![],
                verification_context: vec![],
            }),
        };
        assert!(matches!(
            download_and_verify(&api, &environment(), "receipt-42").await,
            Err(PitekaConnectionError::ReceiptMismatch)
        ));
    }
}
