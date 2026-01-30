//! Key store abstraction and types.
//! See research/06-cryptographic-requirements.md §3.

use async_trait::async_trait;
use std::collections::HashMap;
use thiserror::Error;

use crate::keys::{KeyAlgorithm, KeyPurpose};

pub mod memory;

pub use memory::InMemoryKeyStore;

/// Key fingerprint for identification.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyFingerprint(pub [u8; 32]);

impl KeyFingerprint {
    pub fn compute(public_key: &[u8], algorithm: KeyAlgorithm) -> Self {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&[algorithm as u8]);
        hasher.update(public_key);
        let result = hasher.finalize();
        let mut fingerprint = [0u8; 32];
        fingerprint.copy_from_slice(&result);
        Self(fingerprint)
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.0)
    }

    pub fn to_base64(&self) -> String {
        use base64::Engine;
        base64::engine::general_purpose::STANDARD.encode(&self.0)
    }
}

#[derive(Debug, Clone)]
pub struct KeyMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub fingerprint: KeyFingerprint,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub public_key: Vec<u8>,
    pub metadata: KeyMetadata,
}

#[derive(Debug, Clone)]
pub struct Signature {
    pub algorithm: KeyAlgorithm,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum KeyStoreError {
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Key already exists: {0}")]
    KeyAlreadyExists(String),
    #[error("Invalid key format: {0}")]
    InvalidKeyFormat(String),
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
    #[error("Storage error: {0}")]
    StorageError(String),
    #[error("Access denied: {0}")]
    AccessDenied(String),
}

/// Key store trait for secure key management.
#[async_trait]
pub trait KeyStore: Send + Sync {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError>;

    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError>;

    async fn export_public_key(&self, fingerprint: &KeyFingerprint) -> Result<Vec<u8>, KeyStoreError>;

    async fn sign(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError>;

    async fn verify(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KeyStoreError>;

    async fn delete_key(&self, fingerprint: &KeyFingerprint) -> Result<(), KeyStoreError>;

    async fn list_keys(&self) -> Result<Vec<KeyInfo>, KeyStoreError>;

    async fn get_key_info(&self, fingerprint: &KeyFingerprint) -> Result<KeyInfo, KeyStoreError>;
}
