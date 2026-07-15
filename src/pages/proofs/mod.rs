//! Proof management pages.

pub mod generate;
pub mod list;
pub mod verify;
pub mod verify_cross_chain;

pub use generate::GenerateProof;
pub use list::Proofs;
pub use verify::VerifyProof;
pub use verify_cross_chain::VerifyCrossChainProof;
