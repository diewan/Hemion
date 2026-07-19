//! Page components - modular structure.
//!
//! Organized into feature modules:
//! - `accounts` - Dashboard and account management
//! - `sanads` - Sanads management (list, create, show, transfer, consume)
//! - `proofs` - Proof generation and verification
//! - `cross_chain` - Cross-chain transfers
//! - `seals` - Seal creation and verification
//! - `validate` - Validation utilities
//! - `transactions` - Transaction history
//! - `settings` - Application settings
//! - `common` - Shared UI helpers
//!
//! The route layer exposes only the current application URLs; removed URLs are
//! intentionally not redirected.

// Common UI helpers (fully migrated)
pub mod assurance_inspector;
pub mod bundle_verify;
pub mod common;
pub mod console;
pub mod dispute_inspector;
pub mod object_inspector;
pub mod piteka_environment;

// NFT and Wallet pages (already separate files)
pub mod nft_page;
pub mod wallet_page;

// Feature modules (re-exporting from old_pages during migration)
pub mod accounts;
pub mod cross_chain;
pub mod proofs;
pub mod redirect;
pub mod sanads;
pub mod seals;
pub mod settings;
pub mod transactions;
pub mod validate;
pub mod zk_proofs;

// Re-exports from nft_page and wallet_page (standalone files)
pub use assurance_inspector::AssuranceInspector;
pub use bundle_verify::BundleVerify;
pub use console::ConsoleHome;
pub use dispute_inspector::DisputeInspector;
pub use nft_page::{NftCollections, NftDetail, NftGallery};
pub use object_inspector::ObjectInspector;
pub use piteka_environment::PitekaEnvironmentReceipt;
pub use wallet_page::WalletPage;

// Re-exports from accounts module
pub use accounts::{AccountTransactions, Dashboard};

// Re-exports from sanads module (already migrated)
pub use sanads::{ConsumeSanad, CreateSanad, SanadJourney, Sanads, ShowSanad, TransferSanad};

// Re-exports from proofs module
pub use proofs::{GenerateProof, Proofs, VerifyCrossChainProof, VerifyProof};
pub use redirect::{
    Activity, ActivityAccountTransactions, ActivityMove, ActivityRetry, ActivityStatus,
    ActivityTransactionDetail, ActivityTransferDetail, AssetCollectibleDetail, AssetCollectibles,
    AssetCollections, AssetConsume, AssetCreate, AssetDetail, AssetJourney, AssetSealConsume,
    AssetSealCreate, AssetSealRegistry, AssetSealVerify, AssetSeals, AssetTransfer, AssetWallet,
    Assets, Contacts,
};

// Re-exports from cross_chain module
pub use cross_chain::{CrossChain, CrossChainRetry, CrossChainStatus, CrossChainTransfer};

// Re-exports from seals module
pub use seals::{ConsumeSeal, CreateSeal, SealRegistry, Seals, VerifySeal};

// Re-exports from validate module
pub use validate::{
    OfflineVerify, Validate, ValidateCommitmentChain, ValidateConsignment, ValidateProof,
    ValidateSeal,
};

// Re-exports from zk_proofs module (Phase 5)
pub use zk_proofs::{ZkGenerateProof, ZkVerifyProof};

// Re-exports from transactions module
pub use transactions::{TransactionDetail, Transactions};

// Re-exports from settings module
pub use settings::{Settings, SettingsAdvanced};

// Common UI helpers - re-export everything from common module for convenience

// Migration complete: old_pages.rs has been removed
