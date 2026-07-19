//! Application state definition.

use crate::context::types::*;
use crate::wallet_core::WalletData;

/// Application state.
#[derive(Clone)]
pub struct AppState {
    pub wallet: WalletData,
    pub selected_chain: ChainId,
    pub selected_network: Network,
    pub sanads: Vec<TrackedSanad>,
    pub transfers: Vec<TrackedTransfer>,
    pub contracts: Vec<ContractRecord>,
    pub seals: Vec<SealRecord>,
    pub proofs: Vec<ProofRecord>,
    pub transactions: Vec<TransactionRecord>,
    pub nfts: Vec<NftRecord>,
    pub nft_collections: Vec<NftCollection>,
    pub notification: Option<Notification>,
    /// Session state is intentionally ephemeral and is never persisted with
    /// wallet metadata.
    pub locked: bool,
    pub session_expires_at: Option<u64>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wallet: WalletData::default(),
            selected_chain: csv_sdk::protocol::hash::ChainId::new("bitcoin"),
            selected_network: Network::Test,
            sanads: Vec::new(),
            transfers: Vec::new(),
            contracts: Vec::new(),
            seals: Vec::new(),
            proofs: Vec::new(),
            transactions: Vec::new(),
            nfts: Vec::new(),
            nft_collections: Vec::new(),
            notification: None,
            locked: true,
            session_expires_at: None,
        }
    }
}

impl PartialEq for AppState {
    fn eq(&self, other: &Self) -> bool {
        self.selected_chain == other.selected_chain
            && self.selected_network == other.selected_network
            && self.wallet.total_accounts() == other.wallet.total_accounts()
            && self.sanads.len() == other.sanads.len()
            && self.seals.len() == other.seals.len()
            && self.proofs.len() == other.proofs.len()
            && self.transfers.len() == other.transfers.len()
            && self.contracts.len() == other.contracts.len()
            && self.transactions.len() == other.transactions.len()
            && self.nfts.len() == other.nfts.len()
            && self.nft_collections.len() == other.nft_collections.len()
            && self.notification.is_some() == other.notification.is_some()
            && self.locked == other.locked
    }
}

impl AppState {
    pub fn end_wallet_session(&mut self) {
        self.locked = true;
        self.session_expires_at = None;
        self.notification = None;
    }

    pub fn start_wallet_session(&mut self, expires_at: u64) {
        self.locked = false;
        self.session_expires_at = Some(expires_at);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wallet::account::ChainAccount;

    #[test]
    fn ending_a_session_preserves_wallet_and_asset_metadata() {
        let mut state = AppState::default();
        state.wallet.add_account(ChainAccount::watch_only(
            csv_sdk::protocol::hash::ChainId::new("bitcoin"),
            "Savings",
            "bc1qexample",
        ));
        state.start_wallet_session(100);

        state.end_wallet_session();

        assert!(state.locked);
        assert_eq!(state.wallet.total_accounts(), 1);
        assert_eq!(state.wallet.accounts[0].name, "Savings");
    }
}
