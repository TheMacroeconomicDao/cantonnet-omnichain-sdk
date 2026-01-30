//! In-memory key store implementation.
//! See research/06-cryptographic-requirements.md §3.2.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use async_trait::async_trait;
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;

use crate::keys::{KeyAlgorithm, KeyPurpose};
use crate::keystore::{
    KeyFingerprint, KeyInfo, KeyMetadata, KeyStore, KeyStoreError, Signature,
};

struct StoredKey {
    signing_key: Option<SigningKeyInner>,
    verifying_key: VerifyingKeyInner,
    fingerprint: KeyFingerprint,
    metadata: KeyMetadata,
}

#[derive(Clone)]
enum SigningKeyInner {
    Ed25519(ed25519_dalek::SigningKey),
    EcdsaP256(Vec<u8>),
    EcdsaSecp256k1(Vec<u8>),
}

#[derive(Clone)]
enum VerifyingKeyInner {
    Ed25519(ed25519_dalek::VerifyingKey),
    EcdsaP256(Vec<u8>),
    EcdsaSecp256k1(Vec<u8>),
}

/// In-memory key store (development/testing).
pub struct InMemoryKeyStore {
    keys: Arc<RwLock<HashMap<KeyFingerprint, StoredKey>>>,
}

impl InMemoryKeyStore {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyStore for InMemoryKeyStore {
    async fn generate_key(
        &self,
        algorithm: KeyAlgorithm,
        _purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError> {
        let (signing_inner, verifying_inner, fingerprint) = match algorithm {
            KeyAlgorithm::Ed25519 => {
                let signing = SigningKey::generate(&mut OsRng);
                let verifying = signing.verifying_key();
                let fp = KeyFingerprint::compute(verifying.as_bytes(), algorithm);
                (
                    SigningKeyInner::Ed25519(signing),
                    VerifyingKeyInner::Ed25519(verifying),
                    fp,
                )
            }
            KeyAlgorithm::EcdsaP256 => {
                use p256::ecdsa::SigningKey as P256SigningKey;
                let signing = P256SigningKey::random(&mut OsRng);
                let verifying = signing.verifying_key();
                let pk = verifying.to_encoded_point(false);
                let fp = KeyFingerprint::compute(pk.as_bytes(), algorithm);
                (
                    SigningKeyInner::EcdsaP256(signing.to_bytes().to_vec()),
                    VerifyingKeyInner::EcdsaP256(pk.as_bytes().to_vec()),
                    fp,
                )
            }
            KeyAlgorithm::EcdsaSecp256k1 => {
                use k256::ecdsa::SigningKey as K256SigningKey;
                let signing = K256SigningKey::random(&mut OsRng);
                let verifying = signing.verifying_key();
                let pk = verifying.to_encoded_point(false);
                let fp = KeyFingerprint::compute(pk.as_bytes(), algorithm);
                (
                    SigningKeyInner::EcdsaSecp256k1(signing.to_bytes().to_vec()),
                    VerifyingKeyInner::EcdsaSecp256k1(pk.as_bytes().to_vec()),
                    fp,
                )
            }
            KeyAlgorithm::X25519 => {
                return Err(KeyStoreError::InvalidKeyFormat(
                    "X25519 is for key exchange, not signing".into(),
                ));
            }
        };

        let stored = StoredKey {
            signing_key: Some(signing_inner),
            verifying_key: verifying_inner,
            fingerprint: fingerprint.clone(),
            metadata,
        };
        self.keys.write().await.insert(fingerprint.clone(), stored);
        Ok(fingerprint)
    }

    async fn import_key(
        &self,
        key_bytes: &[u8],
        algorithm: KeyAlgorithm,
        _purpose: KeyPurpose,
        metadata: KeyMetadata,
    ) -> Result<KeyFingerprint, KeyStoreError> {
        let (signing_inner, verifying_inner, fingerprint) = match algorithm {
            KeyAlgorithm::Ed25519 => {
                let signing = SigningKey::from_bytes(
                    key_bytes
                        .try_into()
                        .map_err(|_| KeyStoreError::InvalidKeyFormat("Ed25519 key must be 32 bytes".into()))?,
                );
                let verifying = signing.verifying_key();
                let fp = KeyFingerprint::compute(verifying.as_bytes(), algorithm);
                (
                    SigningKeyInner::Ed25519(signing),
                    VerifyingKeyInner::Ed25519(verifying),
                    fp,
                )
            }
            KeyAlgorithm::EcdsaP256 => {
                use p256::ecdsa::SigningKey as P256SigningKey;
                let signing = P256SigningKey::from_slice(key_bytes)
                    .map_err(|e| KeyStoreError::InvalidKeyFormat(e.to_string()))?;
                let verifying = signing.verifying_key();
                let pk = verifying.to_encoded_point(false);
                let fp = KeyFingerprint::compute(pk.as_bytes(), algorithm);
                (
                    SigningKeyInner::EcdsaP256(key_bytes.to_vec()),
                    VerifyingKeyInner::EcdsaP256(pk.as_bytes().to_vec()),
                    fp,
                )
            }
            KeyAlgorithm::EcdsaSecp256k1 => {
                use k256::ecdsa::SigningKey as K256SigningKey;
                let signing = K256SigningKey::from_slice(key_bytes)
                    .map_err(|e| KeyStoreError::InvalidKeyFormat(e.to_string()))?;
                let verifying = signing.verifying_key();
                let pk = verifying.to_encoded_point(false);
                let fp = KeyFingerprint::compute(pk.as_bytes(), algorithm);
                (
                    SigningKeyInner::EcdsaSecp256k1(key_bytes.to_vec()),
                    VerifyingKeyInner::EcdsaSecp256k1(pk.as_bytes().to_vec()),
                    fp,
                )
            }
            KeyAlgorithm::X25519 => {
                return Err(KeyStoreError::InvalidKeyFormat(
                    "X25519 not supported for signing".into(),
                ));
            }
        };

        let stored = StoredKey {
            signing_key: Some(signing_inner),
            verifying_key: verifying_inner,
            fingerprint: fingerprint.clone(),
            metadata,
        };
        self.keys.write().await.insert(fingerprint.clone(), stored);
        Ok(fingerprint)
    }

    async fn export_public_key(&self, fingerprint: &KeyFingerprint) -> Result<Vec<u8>, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys
            .get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        let pk = match &stored.verifying_key {
            VerifyingKeyInner::Ed25519(k) => k.as_bytes().to_vec(),
            VerifyingKeyInner::EcdsaP256(k) => k.clone(),
            VerifyingKeyInner::EcdsaSecp256k1(k) => k.clone(),
        };
        Ok(pk)
    }

    async fn sign(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
    ) -> Result<Signature, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys
            .get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        let signing_key = stored
            .signing_key
            .as_ref()
            .ok_or_else(|| KeyStoreError::AccessDenied("No signing key".into()))?;
        let (algorithm, bytes) = match signing_key {
            SigningKeyInner::Ed25519(k) => {
                use ed25519_dalek::Signer;
                (KeyAlgorithm::Ed25519, k.sign(data).to_bytes().to_vec())
            }
            SigningKeyInner::EcdsaP256(key_bytes) => {
                use p256::ecdsa::{signature::Signer, Signature, SigningKey};
                let k = SigningKey::from_slice(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig: Signature = k.sign(data);
                (KeyAlgorithm::EcdsaP256, sig.to_bytes().to_vec())
            }
            SigningKeyInner::EcdsaSecp256k1(key_bytes) => {
                use k256::ecdsa::{signature::Signer, Signature, SigningKey};
                let k = SigningKey::from_slice(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig: Signature = k.sign(data);
                (KeyAlgorithm::EcdsaSecp256k1, sig.to_bytes().to_vec())
            }
        };
        Ok(Signature { algorithm, bytes })
    }

    async fn verify(
        &self,
        fingerprint: &KeyFingerprint,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys
            .get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        let ok = match &stored.verifying_key {
            VerifyingKeyInner::Ed25519(k) => {
                use ed25519_dalek::{Signature as EdSig, Verifier};
                let sig = EdSig::from_bytes(
                    signature.bytes.as_slice().try_into().map_err(|_| {
                        KeyStoreError::InvalidKeyFormat("Invalid Ed25519 signature length".into())
                    })?,
                );
                k.verify(data, &sig).is_ok()
            }
            VerifyingKeyInner::EcdsaP256(key_bytes) => {
                use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
                let k = VerifyingKey::from_sec1_bytes(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig = Signature::from_slice(&signature.bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                k.verify(data, &sig).is_ok()
            }
            VerifyingKeyInner::EcdsaSecp256k1(key_bytes) => {
                use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
                let k = VerifyingKey::from_sec1_bytes(key_bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                let sig = Signature::from_slice(&signature.bytes)
                    .map_err(|e| KeyStoreError::CryptoError(e.to_string()))?;
                k.verify(data, &sig).is_ok()
            }
        };
        Ok(ok)
    }

    async fn delete_key(&self, fingerprint: &KeyFingerprint) -> Result<(), KeyStoreError> {
        self.keys.write().await.remove(fingerprint);
        Ok(())
    }

    async fn list_keys(&self) -> Result<Vec<KeyInfo>, KeyStoreError> {
        let keys = self.keys.read().await;
        let mut out = Vec::with_capacity(keys.len());
        for stored in keys.values() {
            let pk = match &stored.verifying_key {
                VerifyingKeyInner::Ed25519(k) => k.as_bytes().to_vec(),
                VerifyingKeyInner::EcdsaP256(k) => k.clone(),
                VerifyingKeyInner::EcdsaSecp256k1(k) => k.clone(),
            };
            out.push(KeyInfo {
                fingerprint: stored.fingerprint.clone(),
                algorithm: match &stored.verifying_key {
                    VerifyingKeyInner::Ed25519(_) => KeyAlgorithm::Ed25519,
                    VerifyingKeyInner::EcdsaP256(_) => KeyAlgorithm::EcdsaP256,
                    VerifyingKeyInner::EcdsaSecp256k1(_) => KeyAlgorithm::EcdsaSecp256k1,
                },
                purpose: KeyPurpose::Signing,
                public_key: pk,
                metadata: stored.metadata.clone(),
            });
        }
        Ok(out)
    }

    async fn get_key_info(&self, fingerprint: &KeyFingerprint) -> Result<KeyInfo, KeyStoreError> {
        let keys = self.keys.read().await;
        let stored = keys
            .get(fingerprint)
            .ok_or_else(|| KeyStoreError::KeyNotFound(fingerprint.to_hex()))?;
        let pk = match &stored.verifying_key {
            VerifyingKeyInner::Ed25519(k) => k.as_bytes().to_vec(),
            VerifyingKeyInner::EcdsaP256(k) => k.clone(),
            VerifyingKeyInner::EcdsaSecp256k1(k) => k.clone(),
        };
        Ok(KeyInfo {
            fingerprint: stored.fingerprint.clone(),
            algorithm: match &stored.verifying_key {
                VerifyingKeyInner::Ed25519(_) => KeyAlgorithm::Ed25519,
                VerifyingKeyInner::EcdsaP256(_) => KeyAlgorithm::EcdsaP256,
                VerifyingKeyInner::EcdsaSecp256k1(_) => KeyAlgorithm::EcdsaSecp256k1,
            },
            purpose: KeyPurpose::Signing,
            public_key: pk,
            metadata: stored.metadata.clone(),
        })
    }
}
