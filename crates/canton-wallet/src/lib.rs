//! Canton Wallet — Canton external party + OmniChain identities (гибко: одна мнемоника или разные источники).
//! See research/09, research/10-flexible-key-derivation.md.

pub mod derivation;
pub mod party_id;
pub mod wallet;

pub use derivation::{
    DerivationError, DerivationPath, DerivationStrategy, IdentitySource, NetworkId,
};
pub use party_id::{canton_party_id, canton_party_id_from_fingerprint};
pub use wallet::{
    MultiIdentityWallet, NetworkIdentity, Wallet, WalletBuilder,
};
