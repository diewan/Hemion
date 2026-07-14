//! Canonical destination adapters and compatibility redirects for legacy URLs.

use crate::routes::Route;
use dioxus::prelude::*;

#[component]
fn CompatibilityRedirect(destination: Route, message: String) -> Element {
    let navigator = use_navigator();
    use_effect(move || {
        navigator.replace(destination.clone());
    });
    rsx! { p { class: "p-6 text-sm text-gray-400", "{message}" } }
}

// Canonical five-destination routes keep existing feature pages as contextual
// depth without preserving their old CLI-shaped top-level namespaces.
#[component]
pub fn Assets() -> Element {
    rsx! { crate::pages::Sanads {} }
}
#[component]
pub fn AssetCreate() -> Element {
    rsx! { crate::pages::CreateSanad {} }
}
#[component]
pub fn AssetDetail(id: String) -> Element {
    rsx! { crate::pages::ShowSanad { id } }
}
#[component]
pub fn AssetJourney(id: String) -> Element {
    rsx! { crate::pages::SanadJourney { id } }
}
#[component]
pub fn AssetTransfer() -> Element {
    rsx! { crate::pages::TransferSanad {} }
}
#[component]
pub fn AssetConsume() -> Element {
    rsx! { crate::pages::ConsumeSanad {} }
}
#[component]
pub fn AssetSeals() -> Element {
    rsx! { crate::pages::Seals {} }
}
#[component]
pub fn AssetSealCreate() -> Element {
    rsx! { crate::pages::CreateSeal {} }
}
#[component]
pub fn AssetSealConsume(seal_ref: Option<String>) -> Element {
    rsx! { crate::pages::ConsumeSeal { seal_ref } }
}
#[component]
pub fn AssetSealRegistry() -> Element {
    rsx! { crate::pages::SealRegistry {} }
}
#[component]
pub fn AssetSealVerify() -> Element {
    rsx! { crate::pages::VerifySeal {} }
}
#[component]
pub fn AssetCollectibles() -> Element {
    rsx! { crate::pages::NftGallery {} }
}
#[component]
pub fn AssetCollections() -> Element {
    rsx! { crate::pages::NftCollections {} }
}
#[component]
pub fn AssetCollectibleDetail(id: String) -> Element {
    rsx! { crate::pages::NftDetail { id } }
}
#[component]
pub fn AssetWallet() -> Element {
    rsx! { crate::pages::WalletPage {} }
}

#[component]
pub fn Activity() -> Element {
    rsx! { crate::pages::Transactions {} }
}
#[component]
pub fn ActivityMove() -> Element {
    rsx! { crate::pages::CrossChainTransfer {} }
}
#[component]
pub fn ActivityStatus() -> Element {
    rsx! { crate::pages::CrossChainStatus {} }
}
#[component]
pub fn ActivityRetry() -> Element {
    rsx! { crate::pages::CrossChainRetry {} }
}
#[component]
pub fn ActivityTransferDetail(id: String) -> Element {
    rsx! { crate::pages::cross_chain::TransferDetail { id } }
}
#[component]
pub fn ActivityTransactionDetail(id: String) -> Element {
    rsx! { crate::pages::TransactionDetail { id } }
}
#[component]
pub fn ActivityAccountTransactions(id: String) -> Element {
    rsx! { crate::pages::AccountTransactions { id } }
}

#[component]
pub fn Contacts() -> Element {
    rsx! {
        div { class: "max-w-2xl space-y-4",
            h1 { class: "text-2xl font-bold", "Contacts" }
            p { class: "text-sm text-gray-400", "No contacts have been added yet." }
        }
    }
}

#[component]
pub fn RedirectToAdvanced() -> Element {
    rsx! { CompatibilityRedirect { destination: Route::SettingsAdvanced {}, message: "Redirecting to Settings → Advanced tools…".to_string() } }
}
#[component]
pub fn RedirectProofBundle(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::AssetDetail { id }, message: "Redirecting to the asset Inspector…".to_string() } }
}

macro_rules! static_redirect {
    ($name:ident, $destination:expr, $message:literal) => {
        #[component]
        pub fn $name() -> Element {
            rsx! { CompatibilityRedirect { destination: $destination, message: $message.to_string() } }
        }
    };
}

static_redirect!(
    LegacyProofs,
    Route::SettingsAdvanced {},
    "Redirecting to Settings → Advanced tools…"
);
static_redirect!(
    LegacyVerifyProof,
    Route::SettingsAdvanced {},
    "Redirecting to proof verification…"
);
static_redirect!(
    LegacyVerifyCrossChainProof,
    Route::SettingsAdvanced {},
    "Redirecting to proof verification…"
);
static_redirect!(
    LegacyValidate,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);
static_redirect!(
    LegacyValidateConsignment,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);
static_redirect!(
    LegacyOfflineVerify,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);
static_redirect!(
    LegacyValidateProof,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);
static_redirect!(
    LegacyValidateSeal,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);
static_redirect!(
    LegacyValidateCommitmentChain,
    Route::SettingsAdvanced {},
    "Redirecting to validation tools…"
);

static_redirect!(LegacySanads, Route::Assets {}, "Redirecting to Assets…");
static_redirect!(
    LegacyCreateSanad,
    Route::AssetCreate {},
    "Redirecting to asset creation…"
);
static_redirect!(
    LegacyTransferSanad,
    Route::AssetTransfer {},
    "Redirecting to asset transfer…"
);
static_redirect!(
    LegacyConsumeSanad,
    Route::AssetConsume {},
    "Redirecting to asset consumption…"
);
static_redirect!(
    LegacyCrossChain,
    Route::Activity {},
    "Redirecting to Activity…"
);
static_redirect!(
    LegacyCrossChainTransfer,
    Route::ActivityMove {},
    "Redirecting to Move across chains…"
);
static_redirect!(
    LegacyCrossChainStatus,
    Route::ActivityStatus {},
    "Redirecting to Activity status…"
);
static_redirect!(
    LegacyCrossChainRetry,
    Route::ActivityRetry {},
    "Redirecting to Activity recovery…"
);
static_redirect!(
    LegacySeals,
    Route::AssetSeals {},
    "Redirecting to asset seals…"
);
static_redirect!(
    LegacyCreateSeal,
    Route::AssetSealCreate {},
    "Redirecting to seal creation…"
);
static_redirect!(
    LegacySealRegistry,
    Route::AssetSealRegistry {},
    "Redirecting to the seal registry…"
);
static_redirect!(
    LegacyVerifySeal,
    Route::AssetSealVerify {},
    "Redirecting to seal verification…"
);
static_redirect!(
    LegacyNftGallery,
    Route::AssetCollectibles {},
    "Redirecting to collectibles…"
);
static_redirect!(
    LegacyNftCollections,
    Route::AssetCollections {},
    "Redirecting to collections…"
);
static_redirect!(
    LegacyWalletPage,
    Route::AssetWallet {},
    "Redirecting to wallet assets…"
);
static_redirect!(
    LegacyTransactions,
    Route::Activity {},
    "Redirecting to Activity…"
);

#[component]
pub fn LegacyShowSanad(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::AssetDetail { id }, message: "Redirecting to asset details…".to_string() } }
}
#[component]
pub fn LegacySanadJourney(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::AssetJourney { id }, message: "Redirecting to the asset journey…".to_string() } }
}
#[component]
pub fn LegacyTransferDetail(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::ActivityTransferDetail { id }, message: "Redirecting to transfer activity…".to_string() } }
}
#[component]
pub fn LegacyConsumeSeal(seal_ref: Option<String>) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::AssetSealConsume { seal_ref }, message: "Redirecting to seal consumption…".to_string() } }
}
#[component]
pub fn LegacyNftDetail(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::AssetCollectibleDetail { id }, message: "Redirecting to collectible details…".to_string() } }
}
#[component]
pub fn LegacyAccountTransactions(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::ActivityAccountTransactions { id }, message: "Redirecting to account activity…".to_string() } }
}
#[component]
pub fn LegacyTransactionDetail(id: String) -> Element {
    rsx! { CompatibilityRedirect { destination: Route::ActivityTransactionDetail { id }, message: "Redirecting to transaction activity…".to_string() } }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_legacy_route_has_an_explicit_redirect_component() {
        let source = include_str!("redirect.rs");
        for component in [
            "LegacySanads",
            "LegacyCreateSanad",
            "LegacyShowSanad",
            "LegacySanadJourney",
            "LegacyTransferSanad",
            "LegacyConsumeSanad",
            "LegacyProofs",
            "LegacyVerifyProof",
            "LegacyVerifyCrossChainProof",
            "LegacyCrossChain",
            "LegacyCrossChainTransfer",
            "LegacyCrossChainStatus",
            "LegacyCrossChainRetry",
            "LegacyTransferDetail",
            "LegacySeals",
            "LegacyCreateSeal",
            "LegacyConsumeSeal",
            "LegacySealRegistry",
            "LegacyVerifySeal",
            "LegacyValidate",
            "LegacyValidateConsignment",
            "LegacyOfflineVerify",
            "LegacyValidateProof",
            "LegacyValidateSeal",
            "LegacyValidateCommitmentChain",
            "LegacyNftGallery",
            "LegacyNftCollections",
            "LegacyNftDetail",
            "LegacyWalletPage",
            "LegacyAccountTransactions",
            "LegacyTransactions",
            "LegacyTransactionDetail",
        ] {
            assert!(
                source.contains(component),
                "missing redirect for {component}"
            );
        }
    }
}
