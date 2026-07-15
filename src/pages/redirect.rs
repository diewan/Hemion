//! Canonical destination adapters for the application routes.

use dioxus::prelude::*;

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
