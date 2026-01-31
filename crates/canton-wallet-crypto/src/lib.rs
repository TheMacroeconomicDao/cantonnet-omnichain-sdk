// Canton Wallet SDK - Cryptographic Operations and Key Management
//
// This crate provides cryptographic operations and key management
// for Canton Wallet SDK.

#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(clippy::module_name_repetitions)]

pub mod keystore;
pub mod hd;
pub mod crypto;

pub use keystore::{KeyStore, InMemoryKeyStore, EncryptedKeyStore};
pub use hd::{HDWallet, HDAccount};
pub use crypto::{CryptoOps, KeyPair, generate_keypair, sign, verify};

/// Re-export commonly used types for convenience
pub mod prelude {
    pub use crate::keystore::{KeyStore, InMemoryKeyStore, EncryptedKeyStore};
    pub use crate::hd::{HDWallet, HDAccount};
    pub use crate::crypto::{CryptoOps, KeyPair, generate_keypair, sign, verify};
}
