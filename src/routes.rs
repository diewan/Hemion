//! Application routes.

use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::*;
#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[layout(Layout)]
    // Main entry — Dashboard (shows wallet or create/import modal)
    #[route("/")]
    Dashboard {},

    // Five task-oriented destinations.
    #[route("/assets")]
    Assets {},
    #[route("/assets/create")]
    AssetCreate {},
    #[route("/assets/:id")]
    AssetDetail { id: String },
    #[route("/assets/:id/journey")]
    AssetJourney { id: String },
    #[route("/assets/transfer")]
    AssetTransfer {},
    #[route("/assets/consume")]
    AssetConsume {},
    #[route("/assets/seals")]
    AssetSeals {},
    #[route("/assets/seals/create")]
    AssetSealCreate {},
    #[route("/assets/seals/consume")]
    AssetSealConsume { seal_ref: Option<String> },
    #[route("/assets/seals/registry")]
    AssetSealRegistry {},
    #[route("/assets/seals/verify")]
    AssetSealVerify {},
    #[route("/assets/collectibles")]
    AssetCollectibles {},
    #[route("/assets/collectibles/collections")]
    AssetCollections {},
    #[route("/assets/collectibles/:id")]
    AssetCollectibleDetail { id: String },
    #[route("/assets/wallet")]
    AssetWallet {},
    #[route("/activity")]
    Activity {},
    #[route("/activity/move")]
    ActivityMove {},
    #[route("/activity/status")]
    ActivityStatus {},
    #[route("/activity/retry")]
    ActivityRetry {},
    #[route("/activity/transfers/:id")]
    ActivityTransferDetail { id: String },
    #[route("/activity/transactions/:id")]
    ActivityTransactionDetail { id: String },
    #[route("/activity/accounts/:id")]
    ActivityAccountTransactions { id: String },
    #[route("/contacts")]
    Contacts {},

    // Sanads
    #[route("/sanads")]
    LegacySanads {},
    #[route("/sanads/create")]
    LegacyCreateSanad {},
    #[route("/sanads/:id")]
    LegacyShowSanad { id: String },
    #[route("/sanads/:id/journey")]
    LegacySanadJourney { id: String },
    #[route("/sanads/transfer")]
    LegacyTransferSanad {},
    #[route("/sanads/consume")]
    LegacyConsumeSanad {},

    // Proofs
    #[route("/proofs")]
    LegacyProofs {},
    #[route("/proofs/:id/bundle")]
    RedirectProofBundle { id: String },
    #[route("/proofs/generate")]
    RedirectToAdvanced {},
    #[route("/proofs/verify")]
    LegacyVerifyProof {},
    #[route("/proofs/verify-cross-chain")]
    LegacyVerifyCrossChainProof {},

    // Cross-ChainId
    #[route("/cross-chain")]
    LegacyCrossChain {},
    #[route("/cross-chain/transfer")]
    LegacyCrossChainTransfer {},
    #[route("/cross-chain/status")]
    LegacyCrossChainStatus {},
    #[route("/cross-chain/retry")]
    LegacyCrossChainRetry {},
    #[route("/cross-chain/transfer/:id")]
    LegacyTransferDetail { id: String },

    // Seals
    #[route("/seals")]
    LegacySeals {},
    #[route("/seals/create")]
    LegacyCreateSeal {},
    #[route("/seals/consume")]
    LegacyConsumeSeal { seal_ref: Option<String> },
    #[route("/seals/registry")]
    LegacySealRegistry {},
    #[route("/seals/verify")]
    LegacyVerifySeal {},

    // Validate
    #[route("/validate")]
    LegacyValidate {},
    #[route("/validate/consignment")]
    LegacyValidateConsignment {},
    #[route("/validate/offline")]
    LegacyOfflineVerify {},
    #[route("/validate/proof")]
    LegacyValidateProof {},
    #[route("/validate/seal")]
    LegacyValidateSeal {},
    #[route("/validate/commitment-chain")]
    LegacyValidateCommitmentChain {},

    // ZK Proofs. These routes deliberately render typed unavailability states
    // until a real prover/verifier backend is wired through the runtime.
    #[route("/zk/generate")]
    ZkGenerateProof {},
    #[route("/zk/verify")]
    ZkVerifyProof {},

    // NFT Gallery
    #[route("/nfts")]
    LegacyNftGallery {},
    #[route("/nfts/collections")]
    LegacyNftCollections {},
    #[route("/nfts/:id")]
    LegacyNftDetail { id: String },

    // Wallet management sub-page
    #[route("/wallet")]
    LegacyWalletPage {},

    // Account-specific views
    #[route("/account/:id/transactions")]
    LegacyAccountTransactions { id: String },

    // Transactions
    #[route("/transactions")]
    LegacyTransactions {},
    #[route("/transactions/:id")]
    LegacyTransactionDetail { id: String },

    // Settings
    #[route("/settings")]
    Settings {},
    #[route("/settings/advanced")]
    SettingsAdvanced {},
    #[route("/settings/advanced/proofs")]
    Proofs {},
    #[route("/settings/advanced/proofs/generate")]
    GenerateProof {},
    #[route("/settings/advanced/proofs/verify")]
    VerifyProof {},
    #[route("/settings/advanced/proofs/verify-cross-chain")]
    VerifyCrossChainProof {},
    #[route("/settings/advanced/validate")]
    Validate {},
    #[route("/settings/advanced/validate/consignment")]
    ValidateConsignment {},
    #[route("/settings/advanced/validate/offline")]
    OfflineVerify {},
    #[route("/settings/advanced/validate/proof")]
    ValidateProof {},
    #[route("/settings/advanced/validate/seal")]
    ValidateSeal {},
    #[route("/settings/advanced/validate/commitment-chain")]
    ValidateCommitmentChain {},
}
