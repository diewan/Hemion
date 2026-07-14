//! Wallet module - Consolidated wallet management.
//!
//! This module combines:
//! - HD wallet generation and metadata (from core/wallet.rs)
//! - ChainId account management (from wallet_core.rs)
//! - UI context with persistence (from context/wallet.rs)
//! - Storage management (from storage.rs)

pub mod account;
pub mod context;
pub mod data;
pub mod hd;
pub mod metadata;
pub mod storage;

// Re-export commonly used types for convenience
#[allow(unused_imports)]
pub use account::ChainAccount;
#[allow(unused_imports)]
pub use context::WalletContext;
#[allow(unused_imports)]
pub use data::WalletData;
#[allow(unused_imports)]
pub use hd::{BitcoinNetwork, ExtendedWallet};
#[allow(unused_imports)]
pub use metadata::WalletMetadata;
#[allow(unused_imports)]
pub use storage::WalletStorage;
