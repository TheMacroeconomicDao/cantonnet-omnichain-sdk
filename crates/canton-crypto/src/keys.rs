//! Key algorithm and purpose.
//! See research/06-cryptographic-requirements.md §2.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAlgorithm {
    Ed25519,
    EcdsaP256,
    EcdsaSecp256k1,
    X25519,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPurpose {
    Signing,
    Encryption,
    NamespaceDelegation,
    IdentityBinding,
}
