//! Ethereum chain integration.

use csv_hash::ChainId;

/// Format Ethereum address.
pub fn format_address(address_bytes: &[u8; 20]) -> String {
    format!("0x{}", hex::encode(address_bytes))
}

/// Get chain type.
pub fn chain() -> ChainId {
    ChainId::new("ethereum")
}
