//! Context types - data structures for wallet state management.
//!
//! Re-exports canonical types and application contracts rather than duplicating
//! runtime state or transition rules.
//!
//! Any records held by Dioxus are cached, non-authoritative projections. Transfer
//! lifecycle state, permitted actions, finality evidence, and recovery authority
//! are rendered from the versioned `csv_sdk::contract` artifacts instead.
//!
//! Canonical chain identifier: `csv_sdk::protocol::hash::ChainId` (string-based, extensible).
//! Wallet-specific types (NFT, SealContent) that don't exist in csv-store
//! are defined locally.

/// Network type (csv-wallet local).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Network {
    Dev,
    #[default]
    Test,
    Main,
}

impl Network {
    pub fn all() -> [Network; 3] {
        [Network::Dev, Network::Test, Network::Main]
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Network::Dev => "dev",
            Network::Test => "test",
            Network::Main => "main",
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// Re-export canonical domain types from csv-store (no duplicates).
#[allow(unused_imports)]
pub use csv_sdk::consumer_storage::state::domain::{
    ContractRecord, ProofRecord, ProofStatus, SanadRecord as TrackedSanad, SanadStatus, SealRecord,
    SealStatus, TransactionRecord, TransactionStatus, TransactionType,
    TransferRecord as TrackedTransfer, TransferStatus,
};

// Re-export wallet account types.
#[allow(unused_imports)]
pub use csv_sdk::consumer_storage::state::wallet::{
    FaucetConfig, GasAccount, WalletAccount, WalletConfig,
};

// Canonical chain identifier.
pub use csv_sdk::protocol::hash::ChainId;

/// Versioned runtime artifacts that the Dioxus presentation renders.
///
/// These are deliberately re-exported rather than mirrored in wallet-local
/// enums: a UI projection cannot create a lifecycle transition or offer an
/// action the runtime did not permit.
pub use csv_sdk::contract::{
    ContractArtifact, NextAction, ReceiptBody, TransferEvent, TransferPhase, TransferReceipt,
};

/// Specific proof data based on chain type (wallet-specific).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProofData {
    /// Bitcoin-style Merkle proof
    Merkle {
        /// Merkle root hash
        root: String,
        /// Proof path (sibling hashes)
        path: Vec<String>,
        /// Leaf index
        leaf_index: u64,
    },
    /// Ethereum MPT (Merkle Patricia Trie) proof
    Mpt {
        /// State root
        root: String,
        /// Account proof path
        account_proof: Vec<String>,
        /// Storage proof path
        storage_proof: Vec<String>,
    },
    /// Sui checkpoint proof
    Checkpoint {
        /// Checkpoint sequence number
        sequence: u64,
        /// Checkpoint digest
        digest: String,
        /// Validator signatures
        signatures: Vec<String>,
    },
    /// Aptos ledger proof
    Ledger {
        /// Ledger version
        version: u64,
        /// Proof data
        proof: String,
    },
    /// Solana proof
    Solana {
        /// Slot number
        slot: u64,
        /// Bank hash
        bank_hash: String,
        /// Merkle proof
        merkle_proof: Vec<String>,
    },
    /// Zero-Knowledge proof (Phase 5)
    Zk {
        /// Proof system used (SP1, Groth16, PlonK, etc.)
        proof_system: String,
        /// Serialized proof bytes (base64 encoded)
        proof_bytes: String,
        /// Public inputs from the proof
        seal_id: String,
        block_hash: String,
        block_height: u64,
        /// Verifier key hash (for identifying the circuit)
        verifier_key_hash: String,
    },
}

impl ProofData {
    /// Check if this is a ZK proof
    pub fn is_zk(&self) -> bool {
        matches!(self, ProofData::Zk { .. })
    }

    /// Get the proof system if this is a ZK proof
    pub fn zk_proof_system(&self) -> Option<&str> {
        match self {
            ProofData::Zk { proof_system, .. } => Some(proof_system),
            _ => None,
        }
    }
}

/// NFT status.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NftStatus {
    Owned,
    Transferred,
    Burned,
    Listed,
}

impl std::fmt::Display for NftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NftStatus::Owned => write!(f, "Owned"),
            NftStatus::Transferred => write!(f, "Transferred"),
            NftStatus::Burned => write!(f, "Burned"),
            NftStatus::Listed => write!(f, "Listed"),
        }
    }
}

/// An NFT (Non-Fungible Token) record.
#[derive(Clone, Debug, PartialEq)]
pub struct NftRecord {
    pub id: String,
    pub chain: ChainId,
    pub collection_id: Option<String>,
    pub name: String,
    pub symbol: Option<String>,
    pub description: Option<String>,
    pub owner: String,
    pub token_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub created_at: u64,
    pub status: NftStatus,
}

/// NFT collection information.
#[derive(Clone, Debug, PartialEq)]
pub struct NftCollection {
    pub id: String,
    pub chain: ChainId,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub external_url: Option<String>,
    pub total_supply: u64,
    pub owner_count: u64,
    pub floor_price: Option<f64>,
    pub created_at: u64,
}

/// The cryptographic content sealed for verification (wallet-specific).
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SealContent {
    /// Hash of the sealed sanad data
    pub content_hash: String,
    /// Owner address who created the seal
    pub owner: String,
    /// Block height/number when sealed
    pub block_number: Option<u64>,
    /// Transaction hash that created the seal
    pub lock_tx_hash: Option<String>,
}

/// Notification kind.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum NotificationKind {
    Success,
    Error,
    Warning,
    Info,
}

/// A notification.
#[derive(Clone, Debug)]
pub struct Notification {
    pub kind: NotificationKind,
    pub message: String,
}
