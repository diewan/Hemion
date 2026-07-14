//! Zero-knowledge proof pages.
//!
//! The protocol's ZK envelope types remain available to supported backends, but
//! the wallet deliberately reports generation and verification as unavailable
//! until it can use a real prover and pairing verifier.

pub mod generate;
pub mod verify;

pub use generate::ZkGenerateProof;
pub use verify::ZkVerifyProof;
