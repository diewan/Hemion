//! Compatibility export for the platform-neutral runtime port.
//!
//! New presentation code should use [`crate::services::platform::WalletPlatform`]
//! directly.  This alias contains no target-specific implementation.

pub use super::platform::{
    PlatformConfig as ChainConfig, PlatformError as ChainApiError, WalletPlatform as ChainApi,
};
