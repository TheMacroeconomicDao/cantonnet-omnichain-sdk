// Key store implementations for Canton Wallet SDK

use async_trait::async_trait;
use canton_wallet_core::{
    KeyAlgorithm, KeyId, KeyInfo, KeyMetadata, KeyPurpose, KeyStore as KeyStoreTrait,
    PublicKey, Signature, WalletError, WalletResult,
};
use dashmap::DashMap;
use ed25519_dalek::{Keypair, PublicKey as Ed25519PublicKey, SecretKey, Signer, Verifier};
use parking_lot::RwLock;
use rand::rngs::OsRng;
use rand_core::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroize;

/// In-memory key store for development and testing
pub struct InMemoryKeyStore {
    keys: Arc<DashMap<KeyId, StoredKey>>,
}

struct StoredKey {
    keypair: Keypair,
    metadata: KeyMetadata,
}

impl StoredKey {
    fn new(keypair: Keypair, metadata: KeyMetadata) -> Self {
        Self { keypair, metadata }
    }
}

impl InMemoryKeyStore {
    /// Create a new in-memory key store
    pub fn new() -> Self {
        Self {
            keys: Arc::new(DashMap::new()),
        }
    }
}

impl Default for InMemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyStoreTrait for InMemoryKeyStore {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId> {
        match algorithm {
            KeyAlgorithm::Ed25519 => {
                let mut csprng = OsRng;
                let keypair = Keypair::generate(&mut csprng);
                let key_id = KeyId::generate();
                let stored_key = StoredKey::new(keypair, metadata);
                self.keys.insert(key_id.clone(), stored_key);
                Ok(key_id)
            }
            KeyAlgorithm::Secp256k1 | KeyAlgorithm::Secp256r1 => {
                return Err(WalletError::KeyGenerationFailed(format!(
                    "Algorithm {:?} not yet implemented",
                    algorithm
                )));
            }
        }
    }

    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId> {
        match algorithm {
            KeyAlgorithm::Ed25519 => {
                if key_bytes.len() != 64 {
                    return Err(WalletError::KeyImportFailed(
                        "Invalid Ed25519 key length (expected 64 bytes)".to_string(),
                    ));
                }
                let secret = SecretKey::from_bytes(&key_bytes[..32])
                    .map_err(|e| WalletError::KeyImportFailed(e.to_string()))?;
                let public = Ed25519PublicKey::from_bytes(&key_bytes[32..])
                    .map_err(|e| WalletError::KeyImportFailed(e.to_string()))?;
                let keypair = Keypair { secret, public };
                let key_id = KeyId::generate();
                let stored_key = StoredKey::new(keypair, metadata);
                self.keys.insert(key_id.clone(), stored_key);
                Ok(key_id)
            }
            KeyAlgorithm::Secp256k1 | KeyAlgorithm::Secp256r1 => {
                return Err(WalletError::KeyImportFailed(format!(
                    "Algorithm {:?} not yet implemented",
                    algorithm
                )));
            }
        }
    }

    async fn export_public_key(&self, key_id: &KeyId) -> WalletResult<PublicKey> {
        let stored_key = self
            .keys
            .get(key_id)
            .ok_or_else(|| WalletError::KeyNotFound(key_id.to_string()))?;
        let public_bytes = stored_key.keypair.public.to_bytes();
        Ok(PublicKey::new(public_bytes.to_vec(), "ed25519"))
    }

    async fn sign(&self, key_id: &KeyId, data: &[u8]) -> WalletResult<Signature> {
        let stored_key = self
            .keys
            .get(key_id)
            .ok_or_else(|| WalletError::KeyNotFound(key_id.to_string()))?;
        let signature = stored_key.keypair.sign(data);
        Ok(Signature::new(signature.to_bytes().to_vec(), "ed25519"))
    }

    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &Signature,
    ) -> WalletResult<bool> {
        let stored_key = self
            .keys
            .get(key_id)
            .ok_or_else(|| WalletError::KeyNotFound(key_id.to_string()))?;
        let sig = ed25519_dalek::Signature::from_bytes(&signature.bytes)
            .map_err(|_| WalletError::InvalidSignature)?;
        Ok(stored_key.keypair.public.verify(data, &sig).is_ok())
    }

    async fn delete_key(&self, key_id: &KeyId) -> WalletResult<()> {
        self.keys
            .remove(key_id)
            .ok_or_else(|| WalletError::KeyNotFound(key_id.to_string()))?;
        Ok(())
    }

    async fn list_keys(&self) -> WalletResult<Vec<KeyInfo>> {
        let mut keys = Vec::new();
        for entry in self.keys.iter() {
            let key_id = entry.key().clone();
            let public_key = PublicKey::new(
                entry.value().keypair.public.to_bytes().to_vec(),
                "ed25519",
            );
            let metadata = entry.value().metadata.clone();
            keys.push(KeyInfo {
                key_id,
                public_key,
                metadata,
            });
        }
        Ok(keys)
    }

    async fn get_key_info(&self, key_id: &KeyId) -> WalletResult<KeyInfo> {
        let stored_key = self
            .keys
            .get(key_id)
            .ok_or_else(|| WalletError::KeyNotFound(key_id.to_string()))?;
        let public_key = PublicKey::new(
            stored_key.keypair.public.to_bytes().to_vec(),
            "ed25519",
        );
        let metadata = stored_key.metadata.clone();
        Ok(KeyInfo {
            key_id: key_id.clone(),
            public_key,
            metadata,
        })
    }

    async fn rotate_key(
        &self,
        old_key_id: &KeyId,
        new_algorithm: KeyAlgorithm,
    ) -> WalletResult<KeyId> {
        let old_key = self
            .keys
            .get(old_key_id)
            .ok_or_else(|| WalletError::KeyNotFound(old_key_id.to_string()))?;
        let metadata = old_key.metadata.clone();
        let new_key_id = self
            .generate_key(new_algorithm, KeyPurpose::Signing, metadata)
            .await?;
        self.delete_key(old_key_id).await?;
        Ok(new_key_id)
    }
}

/// Encrypted key store for production use
pub struct EncryptedKeyStore {
    inner: Arc<RwLock<InMemoryKeyStore>>,
    encryption_key: Vec<u8>,
}

impl EncryptedKeyStore {
    /// Create a new encrypted key store
    ///
    /// # Arguments
    ///
    /// * `encryption_key` - 32-byte encryption key for AES-256-GCM
    pub fn new(encryption_key: Vec<u8>) -> WalletResult<Self> {
        if encryption_key.len() != 32 {
            return Err(WalletError::EncryptionFailed(
                "Encryption key must be 32 bytes for AES-256-GCM".to_string(),
            ));
        }
        Ok(Self {
            inner: Arc::new(RwLock::new(InMemoryKeyStore::new())),
            encryption_key,
        })
    }

    /// Create a new encrypted key store from a password
    ///
    /// # Arguments
    ///
    /// * `password` - Password to derive encryption key from
    /// * `salt` - Salt for key derivation
    pub fn from_password(password: &[u8], salt: &[u8]) -> WalletResult<Self> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(password);
        hasher.update(salt);
        let encryption_key = hasher.finalize().to_vec();
        Self::new(encryption_key)
    }

    /// Encrypt data using AES-256-GCM
    fn encrypt(&self, plaintext: &[u8]) -> WalletResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use aes_gcm::aead::{Aead, OsRng};

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| WalletError::EncryptionFailed(e.to_string()))?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| WalletError::EncryptionFailed(e.to_string()))?;

        let mut result = Vec::with_capacity(nonce.len() + ciphertext.len());
        result.extend_from_slice(&nonce);
        result.extend_from_slice(&ciphertext);
        Ok(result)
    }

    /// Decrypt data using AES-256-GCM
    fn decrypt(&self, ciphertext: &[u8]) -> WalletResult<Vec<u8>> {
        use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
        use aes_gcm::aead::Aead;

        if ciphertext.len() < 12 {
            return Err(WalletError::DecryptionFailed(
                "Ciphertext too short".to_string(),
            ));
        }

        let cipher = Aes256Gcm::new_from_slice(&self.encryption_key)
            .map_err(|e| WalletError::DecryptionFailed(e.to_string()))?;
        let (nonce, ciphertext) = ciphertext.split_at(12);
        let nonce = Nonce::from_slice(nonce)
            .map_err(|_| WalletError::DecryptionFailed("Invalid nonce".to_string()))?;

        cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| WalletError::DecryptionFailed(e.to_string()))
    }
}

#[async_trait]
impl KeyStoreTrait for EncryptedKeyStore {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId> {
        let inner = self.inner.read();
        inner.generate_key(algorithm, purpose, metadata).await
    }

    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> WalletResult<KeyId> {
        let inner = self.inner.read();
        inner.import_key(key_bytes, algorithm, purpose, metadata).await
    }

    async fn export_public_key(&self, key_id: &KeyId) -> WalletResult<PublicKey> {
        let inner = self.inner.read();
        inner.export_public_key(key_id).await
    }

    async fn sign(&self, key_id: &KeyId, data: &[u8]) -> WalletResult<Signature> {
        let inner = self.inner.read();
        inner.sign(key_id, data).await
    }

    async fn verify(
        &self,
        key_id: &KeyId,
        data: &[u8],
        signature: &Signature,
    ) -> WalletResult<bool> {
        let inner = self.inner.read();
        inner.verify(key_id, data, signature).await
    }

    async fn delete_key(&self, key_id: &KeyId) -> WalletResult<()> {
        let inner = self.inner.write();
        inner.delete_key(key_id).await
    }

    async fn list_keys(&self) -> WalletResult<Vec<KeyInfo>> {
        let inner = self.inner.read();
        inner.list_keys().await
    }

    async fn get_key_info(&self, key_id: &KeyId) -> WalletResult<KeyInfo> {
        let inner = self.inner.read();
        inner.get_key_info(key_id).await
    }

    async fn rotate_key(
        &self,
        old_key_id: &KeyId,
        new_algorithm: KeyAlgorithm,
    ) -> WalletResult<KeyId> {
        let inner = self.inner.write();
        inner.rotate_key(old_key_id, new_algorithm).await
    }
}

/// Secure storage for encrypted data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureStorage {
    pub version: u32,
    pub keys: HashMap<String, EncryptedKeyData>,
}

impl SecureStorage {
    /// Create a new secure storage
    pub fn new() -> Self {
        Self {
            version: 1,
            keys: HashMap::new(),
        }
    }

    /// Add encrypted key data
    pub fn add_key(&mut self, key_id: &str, data: EncryptedKeyData) {
        self.keys.insert(key_id.to_string(), data);
    }

    /// Get encrypted key data
    pub fn get_key(&self, key_id: &str) -> Option<&EncryptedKeyData> {
        self.keys.get(key_id)
    }

    /// Remove encrypted key data
    pub fn remove_key(&mut self, key_id: &str) -> Option<EncryptedKeyData> {
        self.keys.remove(key_id)
    }

    /// Serialize to bytes
    pub fn to_bytes(&self) -> WalletResult<Vec<u8>> {
        bincode::serialize(self)
            .map_err(|e| WalletError::SerializationError(e.to_string()))
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> WalletResult<Self> {
        bincode::deserialize(bytes)
            .map_err(|e| WalletError::DeserializationError(e.to_string()))
    }
}

impl Default for SecureStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Encrypted key data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedKeyData {
    pub algorithm: String,
    pub purpose: String,
    pub encrypted_key: Vec<u8>,
    pub nonce: Vec<u8>,
    pub metadata: KeyMetadata,
}

impl EncryptedKeyData {
    /// Create new encrypted key data
    pub fn new(
        algorithm: impl Into<String>,
        purpose: impl Into<String>,
        encrypted_key: Vec<u8>,
        nonce: Vec<u8>,
        metadata: KeyMetadata,
    ) -> Self {
        Self {
            algorithm: algorithm.into(),
            purpose: purpose.into(),
            encrypted_key,
            nonce,
            metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_keystore_generate_key() {
        let keystore = InMemoryKeyStore::new();
        let key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();
        assert!(!key_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_in_memory_keystore_sign_verify() {
        let keystore = InMemoryKeyStore::new();
        let key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();

        let data = b"test data";
        let signature = keystore.sign(&key_id, data).await.unwrap();
        let verified = keystore.verify(&key_id, data, &signature).await.unwrap();
        assert!(verified);
    }

    #[tokio::test]
    async fn test_in_memory_keystore_list_keys() {
        let keystore = InMemoryKeyStore::new();
        let key_id1 = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();
        let key_id2 = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();

        let keys = keystore.list_keys().await.unwrap();
        assert_eq!(keys.len(), 2);
    }

    #[tokio::test]
    async fn test_in_memory_keystore_delete_key() {
        let keystore = InMemoryKeyStore::new();
        let key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();

        keystore.delete_key(&key_id).await.unwrap();
        let result = keystore.get_key_info(&key_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_in_memory_keystore_rotate_key() {
        let keystore = InMemoryKeyStore::new();
        let old_key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();

        let new_key_id = keystore
            .rotate_key(&old_key_id, KeyAlgorithm::Ed25519)
            .await
            .unwrap();

        assert_ne!(old_key_id, new_key_id);
        let result = keystore.get_key_info(&old_key_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_encrypted_keystore_generate_key() {
        let encryption_key = vec![0u8; 32];
        let keystore = EncryptedKeyStore::new(encryption_key).unwrap();
        let key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();
        assert!(!key_id.as_str().is_empty());
    }

    #[tokio::test]
    async fn test_encrypted_keystore_sign_verify() {
        let encryption_key = vec![0u8; 32];
        let keystore = EncryptedKeyStore::new(encryption_key).unwrap();
        let key_id = keystore
            .generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata::default(),
            )
            .await
            .unwrap();

        let data = b"test data";
        let signature = keystore.sign(&key_id, data).await.unwrap();
        let verified = keystore.verify(&key_id, data, &signature).await.unwrap();
        assert!(verified);
    }

    #[tokio::test]
    fn test_encrypted_keystore_invalid_key() {
        let encryption_key = vec![0u8; 16];
        let result = EncryptedKeyStore::new(encryption_key);
        assert!(result.is_err());
    }

    #[tokio::test]
    fn test_encrypted_keystore_from_password() {
        let password = b"test-password";
        let salt = b"test-salt";
        let keystore = EncryptedKeyStore::from_password(password, salt);
        assert!(keystore.is_ok());
    }

    #[test]
    fn test_secure_storage() {
        let mut storage = SecureStorage::new();
        let data = EncryptedKeyData::new(
            "ed25519",
            "signing",
            vec![1, 2, 3],
            vec![4, 5, 6],
            KeyMetadata::default(),
        );
        storage.add_key("key1", data.clone());
        assert_eq!(storage.get_key("key1"), Some(&data));
    }

    #[test]
    fn test_secure_storage_serialization() {
        let mut storage = SecureStorage::new();
        let data = EncryptedKeyData::new(
            "ed25519",
            "signing",
            vec![1, 2, 3],
            vec![4, 5, 6],
            KeyMetadata::default(),
        );
        storage.add_key("key1", data);

        let bytes = storage.to_bytes().unwrap();
        let restored = SecureStorage::from_bytes(&bytes).unwrap();
        assert_eq!(restored.keys.len(), 1);
    }
}
