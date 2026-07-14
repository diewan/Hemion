//! Portable wallet RPC preferences.
//!
//! Endpoint policy is shared with `csv-sdk`; Hemion owns only persistence and
//! user intent. Credential values remain in the platform vault and are never
//! serialized here.

use csv_sdk::rpc_policy::{
    ChainRpcPolicy, RpcEndpoint, RpcEndpointSource, RpcPolicyError, RpcSelectionMode,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage key for versioned, non-secret RPC preferences.
pub const RPC_POLICY_STORAGE_KEY: &str = "rpc_policy_v1";

/// Current wallet RPC preference schema.
pub const RPC_POLICY_SCHEMA_VERSION: u16 = 1;

/// Per-chain endpoint preferences persisted by Hemion.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WalletRpcPreferences {
    /// Persistence schema version.
    pub schema_version: u16,
    /// Policies keyed by canonical chain name.
    pub policies: HashMap<String, ChainRpcPolicy>,
}

impl Default for WalletRpcPreferences {
    fn default() -> Self {
        Self {
            schema_version: RPC_POLICY_SCHEMA_VERSION,
            policies: HashMap::new(),
        }
    }
}

impl WalletRpcPreferences {
    /// Load preferences from Hemion's platform storage.
    pub fn load() -> Result<Self, String> {
        let storage = crate::storage::wallet_storage().map_err(|error| error.to_string())?;
        let Some(preferences) = storage.try_load::<Self>(RPC_POLICY_STORAGE_KEY) else {
            return Ok(Self::default());
        };
        preferences.validate()?;
        Ok(preferences)
    }

    /// Persist non-secret preferences.
    pub fn save(&self) -> Result<(), String> {
        self.validate()?;
        let storage = crate::storage::wallet_storage().map_err(|error| error.to_string())?;
        storage
            .save(RPC_POLICY_STORAGE_KEY, self)
            .map_err(|error| error.to_string())
    }

    /// Validate schema and every chain policy.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != RPC_POLICY_SCHEMA_VERSION {
            return Err(format!(
                "unsupported RPC preference schema {}",
                self.schema_version
            ));
        }
        for (chain, policy) in &self.policies {
            if chain != &policy.chain {
                return Err(format!(
                    "RPC preference key {chain} does not match policy chain {}",
                    policy.chain
                ));
            }
            policy.validate().map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    /// Add or replace a user endpoint.
    ///
    /// The chain immediately becomes `user_only`; fallback requires a separate,
    /// explicit call to [`Self::set_selection`].
    pub fn inject_user_endpoint(
        &mut self,
        chain: impl Into<String>,
        network: impl Into<String>,
        endpoint: RpcEndpoint,
    ) -> Result<(), RpcPolicyError> {
        if endpoint.source != RpcEndpointSource::User {
            return Err(RpcPolicyError::InvalidEndpoint(
                "Hemion can inject only source = user endpoints".to_string(),
            ));
        }
        let chain = chain.into();
        let network = network.into();
        if let Some(policy) = self.policies.get(&chain)
            && policy.network != network
        {
            return Err(RpcPolicyError::InvalidEndpoint(format!(
                "cannot inject {network} endpoint into {} policy",
                policy.network
            )));
        }
        let policy = self
            .policies
            .entry(chain.clone())
            .or_insert_with(|| ChainRpcPolicy {
                chain,
                network,
                selection: RpcSelectionMode::UserOnly,
                endpoints: Vec::new(),
            });
        policy.use_user_endpoint(endpoint)
    }

    /// Change source/fallback mode explicitly.
    pub fn set_selection(
        &mut self,
        chain: &str,
        selection: RpcSelectionMode,
    ) -> Result<(), RpcPolicyError> {
        let policy = self
            .policies
            .get_mut(chain)
            .ok_or(RpcPolicyError::NoCandidate {
                capability: csv_sdk::rpc_policy::RpcCapability::Read,
                selection,
            })?;
        policy.selection = selection;
        policy.validate()
    }

    /// Remove all user preferences for a chain, returning it to reviewed
    /// application defaults when the runtime is rebuilt.
    pub fn reset_chain(&mut self, chain: &str) {
        self.policies.remove(chain);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use csv_sdk::rpc_policy::{RpcCapability, RpcTransport};

    fn user_endpoint() -> RpcEndpoint {
        RpcEndpoint {
            id: "private-solana".to_string(),
            url: "https://solana.example.test".to_string(),
            transport: RpcTransport::JsonRpcHttp,
            capabilities: vec![RpcCapability::Read, RpcCapability::Broadcast],
            source: RpcEndpointSource::User,
            provider: "self-hosted".to_string(),
            priority: 0,
            credential: None,
        }
    }

    #[test]
    fn injection_is_user_only_and_round_trips_without_secret_values() {
        let mut preferences = WalletRpcPreferences::default();
        preferences
            .inject_user_endpoint("solana", "devnet", user_endpoint())
            .expect("valid endpoint");
        let policy = &preferences.policies["solana"];
        assert_eq!(policy.selection, RpcSelectionMode::UserOnly);

        let encoded = serde_json::to_string(&preferences).expect("encode preferences");
        let decoded: WalletRpcPreferences =
            serde_json::from_str(&encoded).expect("decode preferences");
        assert_eq!(decoded, preferences);
        assert!(!encoded.contains("api_key"));
        assert!(!encoded.contains("bearer"));
    }

    #[test]
    fn built_in_fallback_requires_a_separate_choice() {
        let mut preferences = WalletRpcPreferences::default();
        preferences
            .inject_user_endpoint("solana", "devnet", user_endpoint())
            .expect("valid endpoint");
        preferences
            .set_selection("solana", RpcSelectionMode::UserThenBuiltIn)
            .expect("explicit fallback");
        assert_eq!(
            preferences.policies["solana"].selection,
            RpcSelectionMode::UserThenBuiltIn
        );
    }
}
