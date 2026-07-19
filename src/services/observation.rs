//! Versioned, source-neutral boundary for observation-plane data.
//!
//! These records are projections only. They never authorize a mutation or
//! replace local SDK verification.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const OBSERVATION_API_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub schema_version: u16,
    pub observation_id: String,
    pub source: String,
    pub canonical_artifact: Vec<u8>,
}

impl Observation {
    pub fn validate(&self) -> Result<(), ObservationError> {
        if self.schema_version != OBSERVATION_API_VERSION {
            return Err(ObservationError::UnsupportedVersion(self.schema_version));
        }
        if self.observation_id.trim().is_empty() || self.source.trim().is_empty() {
            return Err(ObservationError::Malformed);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ObservationError {
    #[error("unsupported observation API version {0}")]
    UnsupportedVersion(u16),
    #[error("malformed observation envelope")]
    Malformed,
    #[error("observation service unavailable")]
    Unavailable,
}

#[async_trait(?Send)]
pub trait ObservationPort {
    async fn fetch(&self, observation_id: &str) -> Result<Observation, ObservationError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_unknown_api_versions() {
        let observation = Observation {
            schema_version: OBSERVATION_API_VERSION + 1,
            observation_id: "obs-1".into(),
            source: "tuppira".into(),
            canonical_artifact: vec![],
        };
        assert_eq!(
            observation.validate(),
            Err(ObservationError::UnsupportedVersion(2))
        );
    }

    #[test]
    fn accepts_a_well_formed_projection_without_claiming_verification() {
        let observation = Observation {
            schema_version: OBSERVATION_API_VERSION,
            observation_id: "obs-1".into(),
            source: "tuppira".into(),
            canonical_artifact: vec![0xa0],
        };
        assert_eq!(observation.validate(), Ok(()));
    }
}
