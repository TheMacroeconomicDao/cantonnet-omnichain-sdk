# Cryptographic Requirements for Canton SDK

## 1. Overview

This document covers the cryptographic requirements and implementations needed for the Canton Network SDK, including key management, signing, encryption, and cross-chain cryptographic operations.

## 2. Canton Cryptographic Primitives

### 2.1 Supported Algorithms

| Category | Algorithm | Usage | Rust Crate |
|----------|-----------|-------|------------|
| Signing | Ed25519 | Transaction signing, identity | `ed25519-dalek` |
| Signing | ECDSA P-256 | Alternative signing | `p256` |
| Signing | ECDSA secp256k1 | Ethereum compatibility | `k256` |
| Hashing | SHA-256 | Content addressing | `sha2` |
| Hashing | Blake2b | Fast hashing | `blake2` |
| Hashing | Blake3 | Modern fast hashing | `blake3` |
| Hashing | Keccak-256 | Ethereum compatibility | `sha3` |
| Encryption | AES-256-GCM | Data encryption | `aes-gcm` |
| Key Exchange | X25519 | ECDH key exchange | `x25519-dalek` |
| KDF | HKDF-SHA256 | Key derivation | `hkdf` |
| MAC | HMAC-SHA256 | Message authentication | `hmac` |

### 2.2 Canton Key Types

```rust
//! Canton key type definitions

use ed25519_dalek::{SigningKey, VerifyingKey};
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret as X25519SecretKey};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Canton key purpose
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyPurpose {
    /// Signing transactions and messages
    Signing,
    /// Encrypting data
    Encryption,
    /// Namespace delegation
    NamespaceDelegation,
    /// Identity binding
    IdentityBinding,
}

/// Canton key algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyAlgorithm {
    Ed25519,
    EcdsaP256,
    EcdsaSecp256k1,
    X25519,
}

/// Canton signing key
#[derive(ZeroizeOnDrop)]
pub struct CantonSigningKey {
    algorithm: KeyAlgorithm,
    #[zeroize(skip)]
    purpose: KeyPurpose,
    inner: SigningKeyInner,
}

#[derive(Zeroize, ZeroizeOnDrop)]
enum SigningKeyInner {
    Ed25519(#[zeroize(skip)] SigningKey),
    EcdsaP256(Vec<u8>),
    EcdsaSecp256k1(Vec<u8>),
}

/// Canton verifying key (public)
#[derive(Debug, Clone)]
pub struct CantonVerifyingKey {
    algorithm: KeyAlgorithm,
    purpose: KeyPurpose,
    inner: VerifyingKeyInner,
    /// Key fingerprint
    fingerprint: KeyFingerprint,
}

#[derive(Debug, Clone)]
enum VerifyingKeyInner {
    Ed25519(VerifyingKey),
    EcdsaP256(Vec<u8>),
    EcdsaSecp256k1(Vec<u8>),
}

/// Key fingerprint for identification
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyFingerprint([u8; 32]);

impl KeyFingerprint {
    pub fn compute(public_key: &[u8], algorithm: KeyAlgorithm) -> Self {
        use sha2::{Sha256, Digest};
        
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
        base64::engine::general_purpose::STANDARD.encode(&self.0)
    }
}
```

### 2.3 Canton Party ID Format (External Party / Wallet)

Официальный формат Party ID для external party (wallet) в Canton: **`{partyHint}::{fingerprint}`**.

- **partyHint** — человекопонятное имя (alice, bob, my-wallet-1), уникальное для данного fingerprint.
- **fingerprint** — отпечаток публичного ключа (см. KeyFingerprint); в документации Canton часто в виде hex (например `1220` + 32 байта в hex для protobuf/ledger).

```rust
/// Build Canton external party ID from hint and public key fingerprint
pub fn canton_party_id(party_hint: &str, fingerprint: &KeyFingerprint) -> String {
    format!("{}::{}", party_hint, fingerprint.to_hex())
}
```

Валидация PartyId в canton-core должна допускать формат `hint::hex` (см. research/09-canton-wallet-evm-integration.md).

### 2.4 BIP-39 Mnemonic (Canton and EVM)

Canton поддерживает генерацию ключей из мнемоники по **BIP-39**: мнемоника → seed (PBKDF2) → первые 32 байта seed как приватный ключ Ed25519 для Canton. Для EVM используется тот же seed с путём деривации (например BIP-44: m/44'/60'/0'/0/0). В Rust: крейты `bip39`, `ed25519-dalek` для Canton; `bip39` + `k256` (или Alloy) для EVM.

## 3. Key Management

### 3.1 Key Store Interface

```rust
//! Key store abstraction

use async_trait::async_trait;
use std::collections::HashMap;

/// Key store trait for secure key management
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Generate a new key pair
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError>;
    
    /// Import an existing key
    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError>;
    
    /// Export public key
    async fn export_public_key(
        &self,
        fingerprint: &KeyFingerprint,
    ) -> Result<Vec<u8>, KeyStoreError>;
    
    /// Sign data
    async fn sign(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError>;
    
    /// Verify signature
    async fn verify(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KeyStoreError>;
    
    /// Delete key
    async fn delete_key(
        &self,
        fingerprint: &KeyFingerprint,
    ) -> Result<(), KeyStoreError>;
    
    /// List all keys
    async fn list_keys(&self) -> Result<Vec<KeyInfo>, KeyStoreError>;
    
    /// Get key info
    async fn get_key_info(
        &self,
        fingerprint: &KeyFingerprint,
    ) -> Result<KeyInfo, KeyStoreError>;
}

/// Key metadata
#[derive(Debug, Clone)]
pub struct KeyMetadata {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: HashMap<String, String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Key information
#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub fingerprint: KeyFingerprint,
    pub algorithm: KeyAlgorithm,
    pub purpose: KeyPurpose,
    pub public_key: Vec<u8>,
    pub metadata: KeyMetadata,
}

/// Signature
#[derive(Debug, Clone)]
pub struct Signature {
    pub algorithm: KeyAlgorithm,
    pub bytes: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
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
```

### 3.2 In-Memory Key Store

```rust
//! In-memory key store implementation

use std::sync::Arc;
use tokio::sync::RwLock;
use rand::rngs::OsRng;

/// In-memory key store (for development/testing)
pub struct InMemoryKeyStore {
    keys: Arc<RwLock<HashMap<KeyFingerprint, StoredKey>>>,
}

struct StoredKey {
    signing_key: Option<CantonSigningKey>,
    verifying_key: CantonVerifyingKey,
    metadata: KeyMetadata,
}

impl InMemoryKeyStore {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl KeyStore for InMemoryKeyStore {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError> {
        let (signing_key, verifying_key) = match algorithm {
            KeyAlgorithm::Ed25519 => {
                let signing = SigningKey::generate(&mut OsRng);
                let verifying = signing.verifying_key();
                
                (
                    CantonSigningKey {
                        algorithm,
                        purpose,
                        inner: SigningKeyInner::Ed25519(signing),
                    },
                    CantonVerifyingKey {
                        algorithm,
                        purpose,
                        inner: VerifyingKeyInner::Ed25519(verifying),
                        fingerprint: KeyFingerprint::compute(
                            verifying.as_bytes(),
                            algorithm,
                        ),
                    },
                )
            }
            KeyAlgorithm::EcdsaP256 => {
                use p256::ecdsa::SigningKey as P256SigningKey;
                
                let signing = P256SigningKey::random(&mut OsRng);
                let verifying = signing.verifying_key();
                
                (
                    CantonSigningKey {
                        algorithm,
                        purpose,
                        inner: SigningKeyInner::EcdsaP256(
                            signing.to_bytes().to_vec()
                        ),
                    },
                    CantonVerifyingKey {
                        algorithm,
                        purpose,
                        inner: VerifyingKeyInner::EcdsaP256(
                            verifying.to_encoded_point(false).as_bytes().to_vec()
                        ),
                        fingerprint: KeyFingerprint::compute(
                            verifying.to_encoded_point(false).as_bytes(),
                            algorithm,
                        ),
                    },
                )
            }
            KeyAlgorithm::EcdsaSecp256k1 => {
                use k256::ecdsa::SigningKey as K256SigningKey;
                
                let signing = K256SigningKey::random(&mut OsRng);
                let verifying = signing.verifying_key();
                
                (
                    CantonSigningKey {
                        algorithm,
                        purpose,
                        inner: SigningKeyInner::EcdsaSecp256k1(
                            signing.to_bytes().to_vec()
                        ),
                    },
                    CantonVerifyingKey {
                        algorithm,
                        purpose,
                        inner: VerifyingKeyInner::EcdsaSecp256k1(
                            verifying.to_encoded_point(false).as_bytes().to_vec()
                        ),
                        fingerprint: KeyFingerprint::compute(
                            verifying.to_encoded_point(false).as_bytes(),
                            algorithm,
                        ),
                    },
                )
            }
            KeyAlgorithm::X25519 => {
                return Err(KeyStoreError::InvalidKeyFormat(
                    "X25519 is for key exchange, not signing".into()
                ));
            }
        };
        
        let fingerprint = verifying_key.fingerprint.clone();
        
        let stored = StoredKey {
            signing_key: Some(signing_key),
            verifying_key,
            metadata,
        };
        
        self.keys.write().await.insert(fingerprint.clone(), stored);
        
        Ok(fingerprint)
    }
    
    async fn sign(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys.get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        
        let signing_key = stored.signing_key.as_ref()
            .ok_or_else(|| KeyStoreError::AccessDenied(
                "No signing key available".into()
            ))?;
        
        let bytes = match &signing_key.inner {
            SigningKeyInner::Ed25519(key) => {
                use ed25519_dalek::Signer;
                key.sign(data).to_bytes().to_vec()
            }
            SigningKeyInner::EcdsaP256(key_bytes) => {
                use p256::ecdsa::{SigningKey, signature::Signer};
                let key = SigningKey::from_bytes(key_bytes.as_slice().into())
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig: p256::ecdsa::Signature = key.sign(data);
                sig.to_bytes().to_vec()
            }
            SigningKeyInner::EcdsaSecp256k1(key_bytes) => {
                use k256::ecdsa::{SigningKey, signature::Signer};
                let key = SigningKey::from_bytes(key_bytes.as_slice().into())
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig: k256::ecdsa::Signature = key.sign(data);
                sig.to_bytes().to_vec()
            }
        };
        
        Ok(Signature {
            algorithm: signing_key.algorithm,
            bytes,
        })
    }
    
    async fn verify(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys.get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        
        let result = match &stored.verifying_key.inner {
            VerifyingKeyInner::Ed25519(key) => {
                use ed25519_dalek::{Signature as Ed25519Sig, Verifier};
                let sig = Ed25519Sig::from_bytes(
                    signature.bytes.as_slice().try_into()
                        .map_err(|_| KeyStoreError::InvalidKeyFormat(
                            "Invalid signature length".into()
                        ))?
                );
                key.verify(data, &sig).is_ok()
            }
            VerifyingKeyInner::EcdsaP256(key_bytes) => {
                use p256::ecdsa::{VerifyingKey, Signature, signature::Verifier};
                let key = VerifyingKey::from_sec1_bytes(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig = Signature::from_bytes(signature.bytes.as_slice().into())
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                key.verify(data, &sig).is_ok()
            }
            VerifyingKeyInner::EcdsaSecp256k1(key_bytes) => {
                use k256::ecdsa::{VerifyingKey, Signature, signature::Verifier};
                let key = VerifyingKey::from_sec1_bytes(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig = Signature::from_bytes(signature.bytes.as_slice().into())
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                key.verify(data, &sig).is_ok()
            }
        };
        
        Ok(result)
    }
    
    // ... other methods
}
```

### 3.3 Hardware Security Module (HSM) Integration

```rust
//! HSM key store interface

use async_trait::async_trait;

/// HSM configuration
#[derive(Debug, Clone)]
pub struct HsmConfig {
    /// HSM provider type
    pub provider: HsmProvider,
    /// Connection URL
    pub url: String,
    /// Authentication credentials
    pub credentials: HsmCredentials,
    /// Slot/partition ID
    pub slot_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum HsmProvider {
    /// AWS CloudHSM
    AwsCloudHsm,
    /// Azure Dedicated HSM
    AzureDedicatedHsm,
    /// Google Cloud HSM
    GoogleCloudHsm,
    /// HashiCorp Vault
    HashiCorpVault,
    /// PKCS#11 compatible
    Pkcs11,
    /// YubiHSM
    YubiHsm,
}

#[derive(Clone)]
pub struct HsmCredentials {
    /// Username or key ID
    pub key_id: String,
    /// Password or secret
    pub secret: zeroize::Zeroizing<String>,
    /// Additional auth data
    pub additional: HashMap<String, String>,
}

impl std::fmt::Debug for HsmCredentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HsmCredentials")
            .field("key_id", &self.key_id)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

/// HSM key store implementation
pub struct HsmKeyStore {
    config: HsmConfig,
    client: Box<dyn HsmClient>,
}

#[async_trait]
trait HsmClient: Send + Sync {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: &str,
    ) -> Result<String, HsmError>;
    
    async fn sign(
        &self,
        key_id: &str,
        data: &[u8],
    ) -> Result<Vec<u8>, HsmError>;
    
    async fn verify(
        &self,
        key_id: &str,
        data: &[u8],
        signature: &[u8],
    ) -> Result<bool, HsmError>;
    
    async fn get_public_key(
        &self,
        key_id: &str,
    ) -> Result<Vec<u8>, HsmError>;
    
    async fn delete_key(&self, key_id: &str) -> Result<(), HsmError>;
}

#[derive(Debug, thiserror::Error)]
pub enum HsmError {
    #[error("Connection error: {0}")]
    ConnectionError(String),
    #[error("Authentication error: {0}")]
    AuthenticationError(String),
    #[error("Key not found: {0}")]
    KeyNotFound(String),
    #[error("Operation not supported: {0}")]
    NotSupported(String),
    #[error("HSM error: {0}")]
    HsmError(String),
}

/// HashiCorp Vault HSM client
pub struct VaultHsmClient {
    client: reqwest::Client,
    url: String,
    token: String,
    mount_path: String,
}

impl VaultHsmClient {
    pub async fn new(config: &HsmConfig) -> Result<Self, HsmError> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| HsmError::ConnectionError(e.to_string()))?;
        
        Ok(Self {
            client,
            url: config.url.clone(),
            token: config.credentials.secret.to_string(),
            mount_path: config.credentials.additional
                .get("mount_path")
                .cloned()
                .unwrap_or_else(|| "transit".to_string()),
        })
    }
}

#[async_trait]
impl HsmClient for VaultHsmClient {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        label: &str,
    ) -> Result<String, HsmError> {
        let key_type = match algorithm {
            KeyAlgorithm::Ed25519 => "ed25519",
            KeyAlgorithm::EcdsaP256 => "ecdsa-p256",
            KeyAlgorithm::EcdsaSecp256k1 => "ecdsa-p256", // Vault doesn't support secp256k1 directly
            KeyAlgorithm::X25519 => return Err(HsmError::NotSupported(
                "X25519 not supported for signing".into()
            )),
        };
        
        let url = format!(
            "{}/v1/{}/keys/{}",
            self.url, self.mount_path, label
        );
        
        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({
                "type": key_type,
                "exportable": false,
            }))
            .send()
            .await
            .map_err(|e| HsmError::ConnectionError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(HsmError::HsmError(
                response.text().await.unwrap_or_default()
            ));
        }
        
        Ok(label.to_string())
    }
    
    async fn sign(
        &self,
        key_id: &str,
        data: &[u8],
    ) -> Result<Vec<u8>, HsmError> {
        let url = format!(
            "{}/v1/{}/sign/{}",
            self.url, self.mount_path, key_id
        );
        
        let input = base64::engine::general_purpose::STANDARD.encode(data);
        
        let response = self.client
            .post(&url)
            .header("X-Vault-Token", &self.token)
            .json(&serde_json::json!({
                "input": input,
                "prehashed": false,
            }))
            .send()
            .await
            .map_err(|e| HsmError::ConnectionError(e.to_string()))?;
        
        if !response.status().is_success() {
            return Err(HsmError::HsmError(
                response.text().await.unwrap_or_default()
            ));
        }
        
        let result: serde_json::Value = response.json().await
            .map_err(|e| HsmError::HsmError(e.to_string()))?;
        
        let signature_str = result["data"]["signature"]
            .as_str()
            .ok_or_else(|| HsmError::HsmError("Missing signature".into()))?;
        
        // Vault returns signature as "vault:v1:base64signature"
        let sig_parts: Vec<&str> = signature_str.split(':').collect();
        let sig_b64 = sig_parts.last()
            .ok_or_else(|| HsmError::HsmError("Invalid signature format".into()))?;
        
        base64::engine::general_purpose::STANDARD.decode(sig_b64)
            .map_err(|e| HsmError::HsmError(e.to_string()))
    }
    
    // ... other methods
}
```

## 4. Encryption

### 4.1 Symmetric Encryption

```rust
//! Symmetric encryption utilities

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;

/// Encryption key (256-bit)
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct EncryptionKey([u8; 32]);

impl EncryptionKey {
    /// Generate a new random key
    pub fn generate() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self(key)
    }
    
    /// Create from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() != 32 {
            return Err(CryptoError::InvalidKeyLength);
        }
        let mut key = [0u8; 32];
        key.copy_from_slice(bytes);
        Ok(Self(key))
    }
    
    /// Derive key from password using Argon2
    pub fn derive_from_password(
        password: &[u8],
        salt: &[u8],
    ) -> Result<Self, CryptoError> {
        use argon2::{Argon2, password_hash::PasswordHasher};
        
        let argon2 = Argon2::default();
        let salt_string = base64::engine::general_purpose::STANDARD_NO_PAD
            .encode(salt);
        
        let hash = argon2
            .hash_password(password, &salt_string)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        
        let hash_bytes = hash.hash
            .ok_or(CryptoError::KeyDerivationError("No hash output".into()))?;
        
        Self::from_bytes(hash_bytes.as_bytes())
    }
}

/// Encrypted data with nonce
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// Nonce (12 bytes for AES-GCM)
    pub nonce: [u8; 12],
    /// Ciphertext with authentication tag
    pub ciphertext: Vec<u8>,
}

/// Encrypt data using AES-256-GCM
pub fn encrypt(key: &EncryptionKey, plaintext: &[u8]) -> Result<EncryptedData, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(&key.0)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    Ok(EncryptedData {
        nonce: nonce_bytes,
        ciphertext,
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt(key: &EncryptionKey, encrypted: &EncryptedData) -> Result<Vec<u8>, CryptoError> {
    let cipher = Aes256Gcm::new_from_slice(&key.0)
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))?;
    
    let nonce = Nonce::from_slice(&encrypted.nonce);
    
    cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionError(e.to_string()))
}

/// Encrypt with additional authenticated data (AAD)
pub fn encrypt_with_aad(
    key: &EncryptionKey,
    plaintext: &[u8],
    aad: &[u8],
) -> Result<EncryptedData, CryptoError> {
    use aes_gcm::aead::Payload;
    
    let cipher = Aes256Gcm::new_from_slice(&key.0)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let payload = Payload {
        msg: plaintext,
        aad,
    };
    
    let ciphertext = cipher.encrypt(nonce, payload)
        .map_err(|e| CryptoError::EncryptionError(e.to_string()))?;
    
    Ok(EncryptedData {
        nonce: nonce_bytes,
        ciphertext,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid key length")]
    InvalidKeyLength,
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Decryption error: {0}")]
    DecryptionError(String),
    #[error("Key derivation error: {0}")]
    KeyDerivationError(String),
    #[error("Signature error: {0}")]
    SignatureError(String),
    #[error("Hash error: {0}")]
    HashError(String),
}
```

### 4.2 Asymmetric Encryption (ECIES)

```rust
//! ECIES (Elliptic Curve Integrated Encryption Scheme)

use x25519_dalek::{PublicKey, StaticSecret, SharedSecret};
use hkdf::Hkdf;
use sha2::Sha256;

/// ECIES encrypted message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EciesMessage {
    /// Ephemeral public key
    pub ephemeral_public_key: [u8; 32],
    /// Encrypted data
    pub encrypted: EncryptedData,
}

/// Encrypt using ECIES with X25519
pub fn ecies_encrypt(
    recipient_public_key: &[u8; 32],
    plaintext: &[u8],
) -> Result<EciesMessage, CryptoError> {
    // Generate ephemeral key pair
    let ephemeral_secret = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = PublicKey::from(&ephemeral_secret);
    
    // Compute shared secret
    let recipient_key = PublicKey::from(*recipient_public_key);
    let shared_secret = ephemeral_secret.diffie_hellman(&recipient_key);
    
    // Derive encryption key using HKDF
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
    let mut encryption_key = [0u8; 32];
    hkdf.expand(b"ecies-encryption-key", &mut encryption_key)
        .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
    
    // Encrypt
    let key = EncryptionKey::from_bytes(&encryption_key)?;
    let encrypted = encrypt(&key, plaintext)?;
    
    // Zeroize sensitive data
    encryption_key.zeroize();
    
    Ok(EciesMessage {
        ephemeral_public_key: ephemeral_public.to_bytes(),
        encrypted,
    })
}

/// Decrypt using ECIES with X25519
pub fn ecies_decrypt(
    recipient_secret_key: &StaticSecret,
    message: &EciesMessage,
) -> Result<Vec<u8>, CryptoError> {
    // Compute shared secret
    let ephemeral_public = PublicKey::from(message.ephemeral_public_key);
    let shared_secret = recipient_secret_key.diffie_hellman(&ephemeral_public);
    
    // Derive encryption key using HKDF
    let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
    let mut encryption_key = [0u8; 32];
    hkdf.expand(b"ecies-encryption-key", &mut encryption_key)
        .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
    
    // Decrypt
    let key = EncryptionKey::from_bytes(&encryption_key)?;
    let plaintext = decrypt(&key, &message.encrypted)?;
    
    // Zeroize sensitive data
    encryption_key.zeroize();
    
    Ok(plaintext)
}
```

## 5. Hashing

### 5.1 Hash Functions

```rust
//! Hash function utilities

use sha2::{Sha256, Sha512, Digest};
use blake2::{Blake2b512, Blake2s256};
use blake3::Hasher as Blake3Hasher;

/// Hash algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashAlgorithm {
    Sha256,
    Sha512,
    Blake2b,
    Blake2s,
    Blake3,
    Keccak256,
}

/// Compute hash
pub fn hash(algorithm: HashAlgorithm, data: &[u8]) -> Vec<u8> {
    match algorithm {
        HashAlgorithm::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Blake2b => {
            let mut hasher = Blake2b512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Blake2s => {
            let mut hasher = Blake2s256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgorithm::Blake3 => {
            let mut hasher = Blake3Hasher::new();
            hasher.update(data);
            hasher.finalize().as_bytes().to_vec()
        }
        HashAlgorithm::Keccak256 => {
            use sha3::Keccak256;
            let mut hasher = Keccak256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
    }
}

/// Incremental hasher
pub struct IncrementalHasher {
    algorithm: HashAlgorithm,
    inner: HasherInner,
}

enum HasherInner {
    Sha256(Sha256),
    Sha512(Sha512),
    Blake2b(Blake2b512),
    Blake2s(Blake2s256),
    Blake3(Blake3Hasher),
    Keccak256(sha3::Keccak256),
}

impl IncrementalHasher {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        let inner = match algorithm {
            HashAlgorithm::Sha256 => HasherInner::Sha256(Sha256::new()),
            HashAlgorithm::Sha512 => HasherInner::Sha512(Sha512::new()),
            HashAlgorithm::Blake2b => HasherInner::Blake2b(Blake2b512::new()),
            HashAlgorithm::Blake2s => HasherInner::Blake2s(Blake2s256::new()),
            HashAlgorithm::Blake3 => HasherInner::Blake3(Blake3Hasher::new()),
            HashAlgorithm::Keccak256 => HasherInner::Keccak256(sha3::Keccak256::new()),
        };
        
        Self { algorithm, inner }
    }
    
    pub fn update(&mut self, data: &[u8]) {
        match &mut self.inner {
            HasherInner::Sha256(h) => h.update(data),
            HasherInner::Sha512(h) => h.update(data),
            HasherInner::Blake2b(h) => h.update(data),
            HasherInner::Blake2s(h) => h.update(data),
            HasherInner::Blake3(h) => { h.update(data); },
            HasherInner::Keccak256(h) => h.update(data),
        }
    }
    
    pub fn finalize(self) -> Vec<u8> {
        match self.inner {
            HasherInner::Sha256(h) => h.finalize().to_vec(),
            HasherInner::Sha512(h) => h.finalize().to_vec(),
            HasherInner::Blake2b(h) => h.finalize().to_vec(),
            HasherInner::Blake2s(h) => h.finalize().to_vec(),
            HasherInner::Blake3(h) => h.finalize().as_bytes().to_vec(),
            HasherInner::Keccak256(h) => h.finalize().to_vec(),
        }
    }
}

/// Merkle tree for efficient proofs
pub struct MerkleTree {
    leaves: Vec<[u8; 32]>,
    nodes: Vec<Vec<[u8; 32]>>,
    algorithm: HashAlgorithm,
}

impl MerkleTree {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        Self {
            leaves: Vec::new(),
            nodes: Vec::new(),
            algorithm,
        }
    }
    
    pub fn add_leaf(&mut self, data: &[u8]) {
        let hash = hash(self.algorithm, data);
        let mut leaf = [0u8; 32];
        leaf.copy_from_slice(&hash[..32]);
        self.leaves.push(leaf);
    }
    
    pub fn build(&mut self) {
        if self.leaves.is_empty() {
            return;
        }
        
        self.nodes.clear();
        self.nodes.push(self.leaves.clone());
        
        let mut current_level = self.leaves.clone();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for chunk in current_level.chunks(2) {
                let combined = if chunk.len() == 2 {
                    let mut data = Vec::with_capacity(64);
                    data.extend_from_slice(&chunk[0]);
                    data.extend_from_slice(&chunk[1]);
                    data
                } else {
                    let mut data = Vec::with_capacity(64);
                    data.extend_from_slice(&chunk[0]);
                    data.extend_from_slice(&chunk[0]);
                    data
                };
                
                let hash = hash(self.algorithm, &combined);
                let mut node = [0u8; 32];
                node.copy_from_slice(&hash[..32]);
                next_level.push(node);
            }
            
            self.nodes.push(next_level.clone());
            current_level = next_level;
        }
    }
    
    pub fn root(&self) -> Option<[u8; 32]> {
        self.nodes.last().and_then(|level| level.first().copied())
    }
    
    pub fn proof(&self, index: usize) -> Option<MerkleProof> {
        if index >= self.leaves.len() {
            return None;
        }
        
        let mut proof = Vec::new();
        let mut current_index = index;
        
        for level in &self.nodes[..self.nodes.len() - 1] {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };
            
            if sibling_index < level.len() {
                proof.push(ProofElement {
                    hash: level[sibling_index],
                    is_left: current_index % 2 == 1,
                });
            }
            
            current_index /= 2;
        }
        
        Some(MerkleProof {
            leaf_index: index,
            proof,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MerkleProof {
    pub leaf_index: usize,
    pub proof: Vec<ProofElement>,
}

#[derive(Debug, Clone)]
pub struct ProofElement {
    pub hash: [u8; 32],
    pub is_left: bool,
}

impl MerkleProof {
    pub fn verify(
        &self,
        leaf: &[u8],
        root: &[u8; 32],
        algorithm: HashAlgorithm,
    ) -> bool {
        let leaf_hash = hash(algorithm, leaf);
        let mut current = [0u8; 32];
        current.copy_from_slice(&leaf_hash[..32]);
        
        for element in &self.proof {
            let combined = if element.is_left {
                let mut data = Vec::with_capacity(64);
                data.extend_from_slice(&element.hash);
                data.extend_from_slice(&current);
                data
            } else {
                let mut data = Vec::with_capacity(64);
                data.extend_from_slice(&current);
                data.extend_from_slice(&element.hash);
                data
            };
            
            let h = hash(algorithm, &combined);
            current.copy_from_slice(&h[..32]);
        }
        
        &current == root
    }
}
```

## 6. Cross-Chain Cryptography

### 6.1 Multi-Chain Address Derivation

```rust
//! Cross-chain address derivation

/// Derive addresses for different chains from a master key
pub struct MultiChainAddressDeriver {
    master_key: [u8; 32],
}

impl MultiChainAddressDeriver {
    pub fn new(master_key: [u8; 32]) -> Self {
        Self { master_key }
    }
    
    /// Derive Canton party ID (external party / wallet format: partyHint::fingerprint)
    pub fn derive_canton_party(
        &self,
        index: u32,
        party_hint: &str,
    ) -> Result<String, CryptoError> {
        let path = format!("canton/party/{}", index);
        let derived = self.derive_key(&path)?;
        
        // Generate Ed25519 key from derived bytes
        let signing_key = SigningKey::from_bytes(&derived);
        let verifying_key = signing_key.verifying_key();
        let fingerprint = KeyFingerprint::compute(
            verifying_key.as_bytes(),
            KeyAlgorithm::Ed25519,
        );
        
        // Canton external party ID format: partyHint::fingerprint (see 09)
        Ok(format!("{}::{}", party_hint, fingerprint.to_hex()))
    }
    
    /// Derive Ethereum address
    pub fn derive_ethereum_address(&self, index: u32) -> Result<String, CryptoError> {
        let path = format!("ethereum/account/{}", index);
        let derived = self.derive_key(&path)?;
        
        // Generate secp256k1 key
        use k256::ecdsa::SigningKey;
        let signing_key = SigningKey::from_bytes(&derived.into())
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        let verifying_key = signing_key.verifying_key();
        
        // Ethereum address = last 20 bytes of Keccak256(public_key)
        let public_key_bytes = verifying_key.to_encoded_point(false);
        let hash = hash(HashAlgorithm::Keccak256, &public_key_bytes.as_bytes()[1..]);
        
        Ok(format!("0x{}", hex::encode(&hash[12..])))
    }
    
    /// Derive Cosmos address
    pub fn derive_cosmos_address(
        &self,
        index: u32,
        prefix: &str,
    ) -> Result<String, CryptoError> {
        let path = format!("cosmos/account/{}", index);
        let derived = self.derive_key(&path)?;
        
        // Generate secp256k1 key
        use k256::ecdsa::SigningKey;
        let signing_key = SigningKey::from_bytes(&derived.into())
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        let verifying_key = signing_key.verifying_key();
        
        // Cosmos address = bech32(ripemd160(sha256(compressed_public_key)))
        let public_key_bytes = verifying_key.to_encoded_point(true);
        let sha256_hash = hash(HashAlgorithm::Sha256, public_key_bytes.as_bytes());
        
        use ripemd::Ripemd160;
        let mut ripemd = Ripemd160::new();
        ripemd.update(&sha256_hash);
        let ripemd_hash = ripemd.finalize();
        
        // Bech32 encode
        use bech32::{Bech32, Hrp};
        let hrp = Hrp::parse(prefix)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        let address = bech32::encode::<Bech32>(hrp, &ripemd_hash)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        
        Ok(address)
    }
    
    fn derive_key(&self, path: &str) -> Result<[u8; 32], CryptoError> {
        let hkdf = Hkdf::<Sha256>::new(Some(path.as_bytes()), &self.master_key);
        let mut derived = [0u8; 32];
        hkdf.expand(b"key-derivation", &mut derived)
            .map_err(|e| CryptoError::KeyDerivationError(e.to_string()))?;
        Ok(derived)
    }
}
```

### 6.2 Cross-Chain Signature Verification

```rust
//! Cross-chain signature verification

/// Verify signatures from different chains
pub struct CrossChainSignatureVerifier;

impl CrossChainSignatureVerifier {
    /// Verify Canton signature (Ed25519)
    pub fn verify_canton(
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        use ed25519_dalek::{VerifyingKey, Signature, Verifier};
        
        let key = VerifyingKey::from_bytes(
            public_key.try_into()
                .map_err(|_| CryptoError::InvalidKeyLength)?
        ).map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        let sig = Signature::from_bytes(
            signature.try_into()
                .map_err(|_| CryptoError::SignatureError("Invalid signature length".into()))?
        );
        
        Ok(key.verify(message, &sig).is_ok())
    }
    
    /// Verify Ethereum signature (secp256k1 + recovery)
    pub fn verify_ethereum(
        address: &str,
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};
        use k256::ecdsa::signature::hazmat::PrehashVerifier;
        
        if signature.len() != 65 {
            return Err(CryptoError::SignatureError("Invalid signature length".into()));
        }
        
        // Ethereum message prefix
        let prefixed_message = format!(
            "\x19Ethereum Signed Message:\n{}{}",
            message.len(),
            String::from_utf8_lossy(message)
        );
        let message_hash = hash(HashAlgorithm::Keccak256, prefixed_message.as_bytes());
        
        // Extract r, s, v from signature
        let r_s = &signature[..64];
        let v = signature[64];
        let recovery_id = RecoveryId::try_from(if v >= 27 { v - 27 } else { v })
            .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        let sig = Signature::from_bytes(r_s.into())
            .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        // Recover public key
        let recovered_key = VerifyingKey::recover_from_prehash(
            &message_hash,
            &sig,
            recovery_id,
        ).map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        // Derive address from recovered key
        let public_key_bytes = recovered_key.to_encoded_point(false);
        let key_hash = hash(HashAlgorithm::Keccak256, &public_key_bytes.as_bytes()[1..]);
        let recovered_address = format!("0x{}", hex::encode(&key_hash[12..]));
        
        Ok(recovered_address.to_lowercase() == address.to_lowercase())
    }
    
    /// Verify Cosmos signature (secp256k1)
    pub fn verify_cosmos(
        public_key: &[u8],
        message: &[u8],
        signature: &[u8],
    ) -> Result<bool, CryptoError> {
        use k256::ecdsa::{Signature, VerifyingKey, signature::Verifier};
        
        let key = VerifyingKey::from_sec1_bytes(public_key)
            .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        let sig = Signature::from_bytes(signature.into())
            .map_err(|e| CryptoError::SignatureError(e.to_string()))?;
        
        Ok(key.verify(message, &sig).is_ok())
    }
}
```

## 7. Security Best Practices

### 7.1 Secure Random Generation

```rust
//! Secure random number generation

use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Secure random generator
pub struct SecureRandom {
    rng: ChaCha20Rng,
}

impl SecureRandom {
    /// Create from OS entropy
    pub fn new() -> Self {
        Self {
            rng: ChaCha20Rng::from_entropy(),
        }
    }
    
    /// Generate random bytes
    pub fn random_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = vec![0u8; len];
        self.rng.fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random 32-byte array
    pub fn random_32(&mut self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        self.rng.fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random UUID v4
    pub fn random_uuid(&mut self) -> uuid::Uuid {
        let mut bytes = [0u8; 16];
        self.rng.fill_bytes(&mut bytes);
        uuid::Builder::from_random_bytes(bytes).into_uuid()
    }
}

impl Default for SecureRandom {
    fn default() -> Self {
        Self::new()
    }
}
```

### 7.2 Constant-Time Operations

```rust
//! Constant-time comparison utilities

use subtle::{Choice, ConstantTimeEq};

/// Constant-time byte comparison
pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    
    a.ct_eq(b).into()
}

/// Constant-time select
pub fn constant_time_select<T: Copy>(condition: bool, a: T, b: T) -> T {
    let choice = Choice::from(condition as u8);
    // This requires T to implement ConditionallySelectable
    // For simplicity, using if here but in production use subtle crate
    if condition { a } else { b }
}
```

## 8. Testing Cryptographic Code

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encryption_roundtrip() {
        let key = EncryptionKey::generate();
        let plaintext = b"Hello, Canton!";
        
        let encrypted = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &encrypted).unwrap();
        
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
    
    #[test]
    fn test_ecies_roundtrip() {
        let secret = StaticSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        
        let plaintext = b"Secret message";
        
        let encrypted = ecies_encrypt(&public.to_bytes(), plaintext).unwrap();
        let decrypted = ecies_decrypt(&secret, &encrypted).unwrap();
        
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }
    
    #[test]
    fn test_merkle_tree() {
        let mut tree = MerkleTree::new(HashAlgorithm::Sha256);
        
        tree.add_leaf(b"leaf1");
        tree.add_leaf(b"leaf2");
        tree.add_leaf(b"leaf3");
        tree.add_leaf(b"leaf4");
        
        tree.build();
        
        let root = tree.root().unwrap();
        let proof = tree.proof(2).unwrap();
        
        assert!(proof.verify(b"leaf3", &root, HashAlgorithm::Sha256));
        assert!(!proof.verify(b"wrong", &root, HashAlgorithm::Sha256));
    }
    
    #[test]
    fn test_signing_roundtrip() {
        let store = InMemoryKeyStore::new();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let fingerprint = store.generate_key(
                KeyAlgorithm::Ed25519,
                KeyPurpose::Signing,
                KeyMetadata {
                    name: Some("test-key".into()),
                    description: None,
                    tags: HashMap::new(),
                    created_at: chrono::Utc::now(),
                    expires_at: None,
                },
            ).await.unwrap();
            
            let message = b"Test message";
            let signature = store.sign(&fingerprint, message).await.unwrap();
            let valid = store.verify(&fingerprint, message, &signature).await.unwrap();
            
            assert!(valid);
        });
    }
}
```
