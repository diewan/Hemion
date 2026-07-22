//! Read-only Tuppira explorer connection.
//!
//! Tuppira supplies discovery projections, never verdicts. Selected evidence is
//! matched by digest to an imported bundle and verified with the pinned Parwana
//! SDK before Hemion presents a local result.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::services::bundle_verifier::{
    EvidenceGraphInspection, LocalVerificationError, LocalVerificationResult, import_and_verify,
    import_context, inspect_evidence_graph,
};

const MAX_IDENTIFIER_LEN: usize = 512;
const MAX_RESPONSE_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TuppiraEnvironment {
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
pub struct ObservationProjection {
    pub observation_id: String,
    pub source_id: String,
    pub source_event_id: String,
    pub source_event_type: String,
    pub subject_refs: Vec<String>,
    pub asserted_event_time: Option<i64>,
    pub observed_at: i64,
    pub normalized_profile_id: String,
    pub normalized_profile_version: u16,
    pub normalized_payload_digest: String,
    pub authenticity_material_refs: Vec<String>,
    pub collection_run_id: String,
    pub supersedes: Option<String>,
    pub retraction_status: String,
    pub visibility_scope: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceHealth {
    pub source_id: String,
    pub connector_kind: String,
    pub display_name: String,
    pub last_run_started_at: Option<i64>,
    pub last_run_completed_at: Option<i64>,
    pub cursor_observed_at: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityProjection {
    pub entity_id: String,
    pub entity_kind: String,
    pub display_name: String,
    pub profile_digest: String,
    pub updated_at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityReferenceProjection {
    pub kind: String,
    pub disclosure_state: String,
    pub object_id: Option<String>,
    pub source_observation_id: Option<String>,
    pub observed_at: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityRelationshipProjection {
    pub kind: String,
    pub disclosure_state: String,
    pub related_entity_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityAggregateProjection {
    pub entity: EntityProjection,
    pub references: Vec<EntityReferenceProjection>,
    pub relationships: Vec<EntityRelationshipProjection>,
    pub semantics: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApiResponse<T> {
    data: T,
    success: bool,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TuppiraConnectionError {
    #[error("Tuppira API URL must use https (http is allowed only for localhost development)")]
    UnsafeApiUrl,
    #[error("a tenant identifier is required")]
    MissingTenant,
    #[error("an access token is required")]
    MissingAccessToken,
    #[error("the observation identifier is malformed")]
    MalformedObservationId,
    #[error("Tuppira request was rejected or unavailable: {0}")]
    Api(String),
    #[error("Tuppira returned a malformed discovery projection")]
    MalformedProjection,
    #[error("the selected observation is not disclosed in the imported bundle")]
    EvidenceNotDisclosed,
    #[error("the imported bundle failed local Parwana verification: {0:?}")]
    LocalVerification(LocalVerificationError),
}

#[async_trait(?Send)]
pub trait TuppiraApiPort {
    async fn get_lineage(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<ObservationProjection>, TuppiraConnectionError>;
    async fn get_source_health(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<SourceHealth>, TuppiraConnectionError>;
    async fn get_entity(
        &self,
        _request: AuthorizedGet,
    ) -> Result<EntityAggregateProjection, TuppiraConnectionError> {
        Err(TuppiraConnectionError::Api("entity API unavailable".into()))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct LiveTuppiraApi;

#[cfg(not(target_arch = "wasm32"))]
async fn native_get<T: for<'de> Deserialize<'de>>(
    request: AuthorizedGet,
) -> Result<T, TuppiraConnectionError> {
    let response = reqwest::Client::new()
        .get(request.url)
        .header(reqwest::header::AUTHORIZATION, request.authorization)
        .header("x-tuppira-tenant-id", request.tenant_id)
        .send()
        .await
        .map_err(|error| TuppiraConnectionError::Api(error.to_string()))?;
    if !response.status().is_success() {
        return Err(TuppiraConnectionError::Api(format!(
            "HTTP {}",
            response.status()
        )));
    }
    if response
        .content_length()
        .is_some_and(|size| size > MAX_RESPONSE_BYTES as u64)
    {
        return Err(TuppiraConnectionError::Api(
            "response exceeds size limit".into(),
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| TuppiraConnectionError::Api(error.to_string()))?;
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(TuppiraConnectionError::Api(
            "response exceeds size limit".into(),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|_| TuppiraConnectionError::MalformedProjection)
}

#[cfg(not(target_arch = "wasm32"))]
#[async_trait(?Send)]
impl TuppiraApiPort for LiveTuppiraApi {
    async fn get_lineage(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<ObservationProjection>, TuppiraConnectionError> {
        let response: ApiResponse<Vec<ObservationProjection>> = native_get(request).await?;
        if !response.success {
            return Err(TuppiraConnectionError::MalformedProjection);
        }
        validate_lineage(response.data)
    }

    async fn get_source_health(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<SourceHealth>, TuppiraConnectionError> {
        let response: ApiResponse<Vec<SourceHealth>> = native_get(request).await?;
        if !response.success {
            return Err(TuppiraConnectionError::MalformedProjection);
        }
        validate_health(response.data)
    }

    async fn get_entity(
        &self,
        request: AuthorizedGet,
    ) -> Result<EntityAggregateProjection, TuppiraConnectionError> {
        let response: ApiResponse<EntityAggregateProjection> = native_get(request).await?;
        validate_entity_response(response)
    }
}

#[cfg(target_arch = "wasm32")]
async fn wasm_get<T: for<'de> Deserialize<'de>>(
    request: AuthorizedGet,
) -> Result<T, TuppiraConnectionError> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;

    let init = web_sys::RequestInit::new();
    init.set_method("GET");
    init.set_mode(web_sys::RequestMode::Cors);
    let headers = web_sys::Headers::new()
        .map_err(|_| TuppiraConnectionError::Api("headers unavailable".into()))?;
    headers
        .set("Authorization", &request.authorization)
        .map_err(|_| TuppiraConnectionError::Api("invalid authorization header".into()))?;
    headers
        .set("x-tuppira-tenant-id", &request.tenant_id)
        .map_err(|_| TuppiraConnectionError::Api("invalid tenant header".into()))?;
    init.set_headers(&headers);
    let web_request = web_sys::Request::new_with_str_and_init(&request.url, &init)
        .map_err(|_| TuppiraConnectionError::Api("invalid request".into()))?;
    let window = web_sys::window()
        .ok_or_else(|| TuppiraConnectionError::Api("browser unavailable".into()))?;
    let value = JsFuture::from(window.fetch_with_request(&web_request))
        .await
        .map_err(|_| TuppiraConnectionError::Api("request failed".into()))?;
    let response: web_sys::Response = value
        .dyn_into()
        .map_err(|_| TuppiraConnectionError::Api("invalid response".into()))?;
    if !response.ok() {
        return Err(TuppiraConnectionError::Api(format!(
            "HTTP {}",
            response.status()
        )));
    }
    let buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|_| TuppiraConnectionError::Api("response unreadable".into()))?,
    )
    .await
    .map_err(|_| TuppiraConnectionError::Api("response unreadable".into()))?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    if bytes.len() > MAX_RESPONSE_BYTES {
        return Err(TuppiraConnectionError::Api(
            "response exceeds size limit".into(),
        ));
    }
    serde_json::from_slice(&bytes).map_err(|_| TuppiraConnectionError::MalformedProjection)
}

#[cfg(target_arch = "wasm32")]
#[async_trait(?Send)]
impl TuppiraApiPort for LiveTuppiraApi {
    async fn get_lineage(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<ObservationProjection>, TuppiraConnectionError> {
        let response: ApiResponse<Vec<ObservationProjection>> = wasm_get(request).await?;
        if !response.success {
            return Err(TuppiraConnectionError::MalformedProjection);
        }
        validate_lineage(response.data)
    }
    async fn get_source_health(
        &self,
        request: AuthorizedGet,
    ) -> Result<Vec<SourceHealth>, TuppiraConnectionError> {
        let response: ApiResponse<Vec<SourceHealth>> = wasm_get(request).await?;
        if !response.success {
            return Err(TuppiraConnectionError::MalformedProjection);
        }
        validate_health(response.data)
    }
    async fn get_entity(
        &self,
        request: AuthorizedGet,
    ) -> Result<EntityAggregateProjection, TuppiraConnectionError> {
        let response: ApiResponse<EntityAggregateProjection> = wasm_get(request).await?;
        validate_entity_response(response)
    }
}

impl TuppiraEnvironment {
    fn request(&self, path: &str) -> Result<AuthorizedGet, TuppiraConnectionError> {
        let base = self.api_base_url.trim().trim_end_matches('/');
        let local_http = base.starts_with("http://localhost")
            || base.starts_with("http://127.0.0.1")
            || base.starts_with("http://[::1]");
        if !(base.starts_with("https://") || local_http) {
            return Err(TuppiraConnectionError::UnsafeApiUrl);
        }
        validate_identifier(&self.tenant_id).map_err(|_| TuppiraConnectionError::MissingTenant)?;
        let token = self.access_token.trim();
        if token.is_empty() || token.contains(['\r', '\n']) {
            return Err(TuppiraConnectionError::MissingAccessToken);
        }
        Ok(AuthorizedGet {
            url: format!("{base}{path}"),
            authorization: format!("Bearer {token}"),
            tenant_id: self.tenant_id.clone(),
        })
    }

    pub fn lineage_request(
        &self,
        observation_id: &str,
    ) -> Result<AuthorizedGet, TuppiraConnectionError> {
        validate_identifier(observation_id)
            .map_err(|_| TuppiraConnectionError::MalformedObservationId)?;
        self.request(&format!("/api/v1/observations/{observation_id}/lineage"))
    }

    pub fn source_health_request(&self) -> Result<AuthorizedGet, TuppiraConnectionError> {
        self.request("/api/v1/observation-sources/health")
    }

    /// The live discovery feed: most-recent tenant-visible observations.
    pub fn list_request(&self, limit: u32) -> Result<AuthorizedGet, TuppiraConnectionError> {
        self.request(&format!("/api/v1/observations?limit={limit}"))
    }

    pub fn entity_request(&self, entity_id: &str) -> Result<AuthorizedGet, TuppiraConnectionError> {
        validate_identifier(entity_id)
            .map_err(|_| TuppiraConnectionError::MalformedObservationId)?;
        self.request(&format!("/api/v1/entities/{entity_id}/accountability"))
    }
}

pub async fn fetch_entity<P: TuppiraApiPort>(
    api: &P,
    environment: &TuppiraEnvironment,
    entity_id: &str,
) -> Result<EntityAggregateProjection, TuppiraConnectionError> {
    api.get_entity(environment.entity_request(entity_id)?).await
}

/// Fetch the most recent observations for the live explorer feed. The response
/// shape matches lineage, so the lineage read path is reused.
pub async fn list_observations<P: TuppiraApiPort>(
    api: &P,
    environment: &TuppiraEnvironment,
    limit: u32,
) -> Result<Vec<ObservationProjection>, TuppiraConnectionError> {
    api.get_lineage(environment.list_request(limit)?).await
}

pub async fn discover<P: TuppiraApiPort>(
    api: &P,
    environment: &TuppiraEnvironment,
    observation_id: &str,
) -> Result<(Vec<ObservationProjection>, Vec<SourceHealth>), TuppiraConnectionError> {
    let lineage = api
        .get_lineage(environment.lineage_request(observation_id)?)
        .await?;
    let health = api
        .get_source_health(environment.source_health_request()?)
        .await?;
    Ok((lineage, health))
}

/// Verify an imported bundle locally and require the selected observation's
/// normalized digest to be present as a disclosed evidence node.
pub fn verify_selected(
    observation: &ObservationProjection,
    bundle: &[u8],
    context: &[u8],
) -> Result<LocalVerificationResult, TuppiraConnectionError> {
    let graph =
        inspect_evidence_graph(bundle).map_err(TuppiraConnectionError::LocalVerification)?;
    require_disclosed_digest(observation, &graph)?;
    let context = import_context(context).map_err(TuppiraConnectionError::LocalVerification)?;
    let selected = context.name.clone();
    import_and_verify(bundle, &[context], &selected)
        .map_err(TuppiraConnectionError::LocalVerification)
}

fn require_disclosed_digest(
    observation: &ObservationProjection,
    graph: &EvidenceGraphInspection,
) -> Result<(), TuppiraConnectionError> {
    let digest = observation.normalized_payload_digest.to_ascii_lowercase();
    if graph
        .nodes
        .iter()
        .any(|node| !node.is_withheld && node.content_digest.to_ascii_lowercase() == digest)
    {
        Ok(())
    } else {
        Err(TuppiraConnectionError::EvidenceNotDisclosed)
    }
}

fn validate_lineage(
    records: Vec<ObservationProjection>,
) -> Result<Vec<ObservationProjection>, TuppiraConnectionError> {
    if records.iter().any(|record| {
        validate_identifier(&record.observation_id).is_err()
            || validate_identifier(&record.source_id).is_err()
            || record.observed_at <= 0
            || record.normalized_payload_digest.len() != 64
            || hex::decode(&record.normalized_payload_digest).is_err()
    }) {
        return Err(TuppiraConnectionError::MalformedProjection);
    }
    Ok(records)
}

fn validate_health(
    records: Vec<SourceHealth>,
) -> Result<Vec<SourceHealth>, TuppiraConnectionError> {
    if records.iter().any(|record| {
        validate_identifier(&record.source_id).is_err() || record.display_name.trim().is_empty()
    }) {
        return Err(TuppiraConnectionError::MalformedProjection);
    }
    Ok(records)
}

fn validate_entity_response(
    response: ApiResponse<EntityAggregateProjection>,
) -> Result<EntityAggregateProjection, TuppiraConnectionError> {
    let value = response.data;
    let valid_disclosure = |state: &str| matches!(state, "available" | "incomplete" | "withheld");
    if !response.success
        || value.semantics != "observational_not_authorization"
        || validate_identifier(&value.entity.entity_id).is_err()
        || value.entity.profile_digest.len() != 64
        || hex::decode(&value.entity.profile_digest).is_err()
        || value.references.iter().any(|item| {
            !valid_disclosure(&item.disclosure_state)
                || (item.disclosure_state == "available") != item.object_id.is_some()
        })
        || value.relationships.iter().any(|item| {
            !valid_disclosure(&item.disclosure_state)
                || (item.disclosure_state == "available") != item.related_entity_id.is_some()
        })
    {
        return Err(TuppiraConnectionError::MalformedProjection);
    }
    Ok(value)
}

fn validate_identifier(value: &str) -> Result<(), ()> {
    if value.is_empty()
        || value.len() > MAX_IDENTIFIER_LEN
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(digest: &str) -> ObservationProjection {
        ObservationProjection {
            observation_id: "obs:1".into(),
            source_id: "source:1".into(),
            source_event_id: "event:1".into(),
            source_event_type: "deployment".into(),
            subject_refs: vec![],
            asserted_event_time: None,
            observed_at: 1,
            normalized_profile_id: "profile:1".into(),
            normalized_profile_version: 1,
            normalized_payload_digest: digest.into(),
            authenticity_material_refs: vec![],
            collection_run_id: "run:1".into(),
            supersedes: None,
            retraction_status: "active".into(),
            visibility_scope: "tenant".into(),
        }
    }

    #[test]
    fn builds_only_authorized_tenant_scoped_requests() {
        let environment = TuppiraEnvironment {
            api_base_url: "https://tuppira.example/".into(),
            tenant_id: "tenant-1".into(),
            access_token: "secret".into(),
        };
        let request = environment.lineage_request("obs:1").unwrap();
        assert_eq!(
            request.url,
            "https://tuppira.example/api/v1/observations/obs:1/lineage"
        );
        assert_eq!(request.authorization, "Bearer secret");
        assert_eq!(request.tenant_id, "tenant-1");
    }

    #[test]
    fn rejects_unsafe_ambiguous_and_malformed_inputs() {
        let mut environment = TuppiraEnvironment {
            api_base_url: "http://remote.example".into(),
            tenant_id: "tenant-1".into(),
            access_token: "secret".into(),
        };
        assert_eq!(
            environment.lineage_request("obs:1"),
            Err(TuppiraConnectionError::UnsafeApiUrl)
        );
        environment.api_base_url = "https://tuppira.example".into();
        assert_eq!(
            environment.lineage_request("../foreign"),
            Err(TuppiraConnectionError::MalformedObservationId)
        );
        assert!(validate_lineage(vec![observation("not-a-digest")]).is_err());
    }

    #[test]
    fn ordinary_projection_rejects_raw_payload_fields() {
        let json = format!(
            r#"{{"observation_id":"obs:1","source_id":"source:1","source_event_id":"event:1","source_event_type":"deployment","subject_refs":[],"asserted_event_time":null,"observed_at":1,"normalized_profile_id":"profile:1","normalized_profile_version":1,"normalized_payload_digest":"{}","authenticity_material_refs":[],"collection_run_id":"run:1","supersedes":null,"retraction_status":"active","visibility_scope":"tenant","raw_payload":"secret"}}"#,
            "00".repeat(32)
        );
        assert!(serde_json::from_str::<ObservationProjection>(&json).is_err());
    }

    #[test]
    fn selected_evidence_must_be_disclosed_and_digest_bound() {
        let digest = "ab".repeat(32);
        let node = crate::services::bundle_verifier::EvidenceGraphNode {
            id: "evidence-1".into(),
            short_id: "evidence".into(),
            kind_id: "org.diewan.evidence.observation.v1".into(),
            kind_label: "Observation",
            producer: "source:1".into(),
            collected_at: 1,
            content_digest: digest.clone(),
            source: "tuppira".into(),
            classification: "external observation".into(),
            is_gap: false,
            is_withheld: false,
        };
        let mut graph = EvidenceGraphInspection {
            nodes: vec![node],
            edges: vec![],
            gap_count: 0,
            withheld_count: 0,
            potential_contradictions: vec![],
        };
        assert_eq!(
            require_disclosed_digest(&observation(&digest), &graph),
            Ok(())
        );
        graph.nodes[0].is_withheld = true;
        assert_eq!(
            require_disclosed_digest(&observation(&digest), &graph),
            Err(TuppiraConnectionError::EvidenceNotDisclosed)
        );
        graph.nodes[0].is_withheld = false;
        assert_eq!(
            require_disclosed_digest(&observation(&"cd".repeat(32)), &graph),
            Err(TuppiraConnectionError::EvidenceNotDisclosed)
        );
    }

    #[test]
    fn entity_projection_rejects_authorization_claims_and_implicit_withholding() {
        let entity = EntityAggregateProjection {
            entity: EntityProjection {
                entity_id: "org:1".into(),
                entity_kind: "organization".into(),
                display_name: "Org One".into(),
                profile_digest: "ab".repeat(32),
                updated_at: 1,
            },
            references: vec![EntityReferenceProjection {
                kind: "mandate".into(),
                disclosure_state: "withheld".into(),
                object_id: None,
                source_observation_id: None,
                observed_at: 1,
            }],
            relationships: vec![],
            semantics: "observational_not_authorization".into(),
        };
        assert!(
            validate_entity_response(ApiResponse {
                data: entity.clone(),
                success: true
            })
            .is_ok()
        );
        let mut overclaim = entity.clone();
        overclaim.semantics = "authorized".into();
        assert_eq!(
            validate_entity_response(ApiResponse {
                data: overclaim,
                success: true
            }),
            Err(TuppiraConnectionError::MalformedProjection)
        );
        let mut leaked = entity;
        leaked.references[0].object_id = Some("hidden-mandate".into());
        assert_eq!(
            validate_entity_response(ApiResponse {
                data: leaked,
                success: true
            }),
            Err(TuppiraConnectionError::MalformedProjection)
        );
    }
}
