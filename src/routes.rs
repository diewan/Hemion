//! Application routes.

use dioxus::prelude::*;

use crate::layout::Layout;
use crate::pages::*;
#[derive(Routable, PartialEq, Clone, Debug)]
pub enum Route {
    #[layout(Layout)]
    // S-H1 — local developer-console entry.
    #[route("/")]
    ConsoleHome {},
    #[route("/verify")]
    BundleVerify {},
    #[route("/assurance")]
    AssuranceInspector {},
    #[route("/inspect")]
    ObjectInspector {},
    #[route("/disputes")]
    DisputeInspector {},

    // Legacy wallet entry. Existing wallet routes remain available unchanged.
    #[route("/wallet")]
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

    // ZK Proofs. These routes deliberately render typed unavailability states
    // until a real prover/verifier backend is wired through the runtime.
    #[route("/zk/generate")]
    ZkGenerateProof {},
    #[route("/zk/verify")]
    ZkVerifyProof {},

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
